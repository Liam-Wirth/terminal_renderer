use crate::core::{Camera, Color, Colorf32, Pixel, Scene};
use crate::pipeline::{Fragment, Pipeline, ProcessedGeometry, ProjectedVertex};
use crate::renderers::window::WinBuffer;
use glam::{Mat4, Vec2, Vec3, Vec4};
use minifb::{Window, WindowOptions};
use std::io;
use std::time::{Duration, Instant};

pub struct WindowPipeline {
    pub(crate) window: Window,
    front_buffer: WinBuffer,
    // pub(crate) back_buffer: WinBuffer,

    // Performance metrics (migrated from old Engine)
    pub(crate) last_frame: Instant,
    pub(crate) frame_time: Duration,
    fps_counter: u32,
    fps_update_timer: Instant,
    current_fps: f32,
    frame_times: Vec<f32>,
    pub(crate) metrics: String,
}

impl WindowPipeline {
    pub fn new(width: usize, height: usize) -> io::Result<Self> {
        let opts = WindowOptions {
            resize: true,
            scale: minifb::Scale::X1,
            title: true,
            borderless: false,
            ..WindowOptions::default()
        };
        let window = Window::new("3D Renderer", width, height, opts)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Self {
            window,
            front_buffer: WinBuffer::new(width, height),
            last_frame: Instant::now(),
            frame_time: Duration::from_secs_f32(1.0 / 60.0),
            fps_counter: 0,
            fps_update_timer: Instant::now(),
            current_fps: 0.0,
            frame_times: Vec::with_capacity(120),
            metrics: String::new(),
        })
    }

    pub(crate) fn update_metrics(&mut self, frame_delta: Duration, camera: &Camera) {
        self.fps_counter += 1;
        self.frame_times.push(frame_delta.as_secs_f32() * 1000.0);
        if self.frame_times.len() > 120 {
            self.frame_times.remove(0);
        }
        if self.fps_update_timer.elapsed() >= Duration::from_secs(1) {
            self.current_fps = self.fps_counter as f32;
            self.fps_counter = 0;
            self.fps_update_timer = Instant::now();

            let avg_frame_time =
                self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;

            // Just update the metrics string
            self.metrics = format!(
                "FPS: {:.1} | Frame Time: {:.2}ms | Cam: ({:.1}, {:.1}, {:.1})",
                self.current_fps,
                avg_frame_time,
                camera.get_forward().x,
                camera.get_forward().y,
                camera.get_forward().z,
            );
        }
    }

    fn swap_buffers(&mut self, back: &mut WinBuffer) {
        std::mem::swap(&mut self.front_buffer, back);
    }

    pub(crate) fn handle_input(&mut self, camera: &mut Camera) -> std::io::Result<bool> {
        // Get delta time for smooth movement
        let delta = self.last_frame.elapsed().as_secs_f32();

        // Base speeds
        let move_speed = 2.0; // Units per second
        let rotate_speed = 1.0; // Radians per second

        // Calculate frame-adjusted movements
        let move_amount = move_speed * delta;
        let rotate_amount = rotate_speed * delta;

        if self.window.is_open() {
            if let Some(keys) = Some(self.window.get_keys()) {
                // LMFAO
                for key in keys {
                    match key {
                        minifb::Key::Escape | minifb::Key::Q => return Ok(true),
                        minifb::Key::W => camera.move_forward(move_amount),
                        minifb::Key::S => camera.move_forward(-move_amount),
                        minifb::Key::A => camera.rotate(0.0, rotate_amount),
                        minifb::Key::D => camera.rotate(0.0, -rotate_amount),
                        minifb::Key::Up => camera.rotate(rotate_amount, 0.0),
                        minifb::Key::Down => camera.rotate(-rotate_amount, 0.0),
                        _ => {}
                    }
                }
            }
        }
        Ok(false)
    }
}

impl Pipeline for WindowPipeline {
    type Scene = Scene;
    type Camera = Camera;
    type Buffer = WinBuffer;

    fn init(&mut self, width: usize, height: usize) {
        self.front_buffer = WinBuffer::new(width, height);
        // self.back_buffer = WinBuffer::new(width, height);
    }

    fn process_geometry(&mut self, scene: &Scene, camera: &Camera) -> Vec<ProcessedGeometry> {
        let view_proj = camera.get_projection_matrix() * camera.get_view_matrix();

        scene.entities.iter().enumerate()
            .map(|(id, entity)| {
                ProcessedGeometry {
                    entity_id: id,
                    transform: view_proj * glam::Mat4::from(entity.transform),
                }
            })
            .collect()
    }

    fn rasterize(&mut self, geometry: Vec<ProcessedGeometry>, scene: &Scene) -> Vec<Fragment> {
        let mut fragments = Vec::new();

        for geo in geometry {
            let entity = &scene.entities[geo.entity_id];
            let transform = geo.transform;

            for tri in &entity.mesh.tris {
                let v_pos = [
                    entity.mesh.vertices[tri.vertices[0]].pos,
                    entity.mesh.vertices[tri.vertices[1]].pos,
                    entity.mesh.vertices[tri.vertices[2]].pos,
                ];

                let v_clip: [Vec4; 3] = [
                    transform * Vec4::new(v_pos[0].x, v_pos[0].y, v_pos[0].z, 1.0),
                    transform * Vec4::new(v_pos[1].x, v_pos[1].y, v_pos[1].z, 1.0),
                    transform * Vec4::new(v_pos[2].x, v_pos[2].y, v_pos[2].z, 1.0),
                ];

                let v_ndc: [Vec2; 3] = [
                    Vec2::new(v_clip[0].x / v_clip[0].w, v_clip[0].y / v_clip[0].w),
                    Vec2::new(v_clip[1].x / v_clip[1].w, v_clip[1].y / v_clip[1].w),
                    Vec2::new(v_clip[2].x / v_clip[2].w, v_clip[2].y / v_clip[2].w),
                ];

                let (w, h) = (self.window.get_size().0 as f32, self.window.get_size().1 as f32);

                let v_screen: [Vec2; 3] = [
                    Vec2::new((v_ndc[0].x + 1.0) * 0.5 * w, (v_ndc[0].y + 1.0) * 0.5 * h),
                    Vec2::new((v_ndc[1].x + 1.0) * 0.5 * w, (v_ndc[1].y + 1.0) * 0.5 * h),
                    Vec2::new((v_ndc[2].x + 1.0) * 0.5 * w, (v_ndc[2].y + 1.0) * 0.5 * h),
                ];

                let c_default = Colorf32::WHITE;
                let v_color = [
                    entity.mesh.vertices[tri.vertices[0]].color.unwrap_or(c_default),
                    entity.mesh.vertices[tri.vertices[1]].color.unwrap_or(c_default),
                    entity.mesh.vertices[tri.vertices[2]].color.unwrap_or(c_default),
                ];

                let mut draw_line = |start: Vec2, end: Vec2, c: Colorf32| {
                    use crate::pipeline::rasterizer::bresenham;
                    let col = Color {
                        r: (c.r * 255.0) as u8,
                        g: (c.g * 255.0) as u8,
                        b: (c.b * 255.0) as u8,
                    };
                    bresenham(start, end, Pixel::new_framebuffer(c), |pos, depth, _p| {
                        fragments.push(Fragment {
                            screen_pos: pos,
                            depth,
                            color: col,
                        });
                    });
                };

                // Draw wireframe edges
                draw_line(v_screen[0], v_screen[1], v_color[0]);
                draw_line(v_screen[1], v_screen[2], v_color[1]);
                draw_line(v_screen[2], v_screen[0], v_color[2]);
            }
        }

        fragments
    }

    fn process_fragments(&mut self, fragments: Vec<Fragment>, buffer: &mut WinBuffer) {
        buffer.clear(); // Clear before drawing new frame

        // Draw the fragments
        for frag in fragments {
            let x = frag.screen_pos.x as usize;
            let y = frag.screen_pos.y as usize;
            let c = Colorf32 {
                r: frag.color.r as f32 / 255.0,
                g: frag.color.g as f32 / 255.0,
                b: frag.color.b as f32 / 255.0,
            };
            buffer.set_pixel(x, y, frag.depth, Pixel::new_framebuffer(c));
        }

        // Draw the metrics text on top
        buffer.draw_text(&self.metrics, 10, 10, Colorf32::GREEN);
    }

    fn present(&mut self, back: &mut WinBuffer) -> std::io::Result<()> {
        let (win_width, win_height) = self.window.get_size();
        let buffer_size = win_width * win_height;

        let buffer_data = &self.front_buffer.data[..buffer_size];

        self.window
            .update_with_buffer(buffer_data, win_width, win_height)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        self.swap_buffers(back);
        Ok(())
    }

    fn cleanup(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
