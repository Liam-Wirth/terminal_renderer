use crate::core::{Camera, Color, Pixel, Scene};
use crate::pipeline::{Fragment, OldPipeline, ProcessedGeometry, ProjectedVertex};
use crate::renderers::terminal::TermBuffer;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal;
use glam::{Mat4, Vec2, Vec3, Vec4};
use rayon::vec;
use std::cell::RefCell;
use std::io;
use std::time::{Duration, Instant};

pub struct TerminalPipeline {
    pub width: usize,
    pub height: usize,
    front_buffer: TermBuffer,
    // back_buffer: RefCell<TermBuffer>, // Reference to RefCell<TermBuffer>
    pub(crate) last_frame: Instant,
    pub(crate) frame_time: Duration,
    fps_counter: u32,
    fps_update_timer: Instant,
    current_fps: f32,
    frame_times: Vec<f32>,
    metrics: String,
}

impl TerminalPipeline {
    pub fn new(width: usize, height: usize) -> io::Result<Self> {
        Ok(Self {
            width,
            height,
            front_buffer: TermBuffer::new(width, height),
            last_frame: Instant::now(),
            frame_time: Duration::from_secs_f32(1.0 / 60.0),
            fps_counter: 0,
            fps_update_timer: Instant::now(),
            current_fps: 0.0,
            frame_times: Vec::with_capacity(120),
            metrics: String::new(),
        })
    }

    fn swap_buffers(&mut self, back: &mut TermBuffer) {
        std::mem::swap(&mut self.front_buffer, &mut *back);
    }

    // Helper methods migrated from old Engine
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

            // Calculate average frame time
            let avg_frame_time =
                self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;

            // Update the title with metrics
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

    pub(crate) fn handle_input(&mut self, camera: &mut Camera) -> std::io::Result<bool> {
        // Get delta time for smooth movement
        let delta = self.last_frame.elapsed().as_secs_f32();

        // Base speeds
        let move_speed = 2.0; // Units per second
        let rotate_speed = 1.0; // Radians per second

        // Calculate frame-adjusted movements
        let move_amount = move_speed * delta;
        let rotate_amount = rotate_speed * delta;

        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
                    KeyCode::Char('w') | KeyCode::Char('W') => camera.move_forward(move_amount),
                    KeyCode::Char('s') | KeyCode::Char('S') => camera.move_forward(-move_amount),
                    KeyCode::Char('a') | KeyCode::Char('A') => camera.rotate(0.0, rotate_amount),
                    KeyCode::Char('d') | KeyCode::Char('D') => camera.rotate(0.0, -rotate_amount),
                    KeyCode::Up => camera.rotate(rotate_amount, 0.0),
                    KeyCode::Down => camera.rotate(-rotate_amount, 0.0),
                    _ => {}
                }
            }
        }
        Ok(false)
    }
}

impl OldPipeline for TerminalPipeline {
    type Scene = Scene;
    type Camera = Camera;
    type Buffer = TermBuffer;

    fn init(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.front_buffer = TermBuffer::new(width, height);
        // self.back_buffer = TermBuffer::new(width, height);
    }

    fn process_geometry(&mut self, scene: &Scene, camera: &Camera) -> Vec<ProcessedGeometry> {
        let view_proj = camera.get_projection_matrix() * camera.get_view_matrix();

        scene.entities.iter().enumerate()
            .map(|(id, entity)| {
                ProcessedGeometry {
                    entity_id: id,
                    transform: view_proj * Mat4::from(entity.transform),
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
                // Extract the three vertices of the triangle
                let v_pos = [
                    entity.mesh.vertices[tri.vertices[0]].pos,
                    entity.mesh.vertices[tri.vertices[1]].pos,
                    entity.mesh.vertices[tri.vertices[2]].pos,
                ];

                // Transform to clip space
                let v_clip: [Vec4; 3] = [
                    transform * Vec4::new(v_pos[0].x, v_pos[0].y, v_pos[0].z, 1.0),
                    transform * Vec4::new(v_pos[1].x, v_pos[1].y, v_pos[1].z, 1.0),
                    transform * Vec4::new(v_pos[2].x, v_pos[2].y, v_pos[2].z, 1.0),
                ];

                // Convert to NDC
                let v_ndc: [Vec2; 3] = [
                    Vec2::new(v_clip[0].x / v_clip[0].w, v_clip[0].y / v_clip[0].w),
                    Vec2::new(v_clip[1].x / v_clip[1].w, v_clip[1].y / v_clip[1].w),
                    Vec2::new(v_clip[2].x / v_clip[2].w, v_clip[2].y / v_clip[2].w),
                ];

                // Convert to screen space
                let w = self.width as f32;
                let h = self.height as f32;
                let v_screen: [Vec2; 3] = [
                    Vec2::new((v_ndc[0].x + 1.0) * 0.5 * w, (v_ndc[0].y + 1.0) * 0.5 * h),
                    Vec2::new((v_ndc[1].x + 1.0) * 0.5 * w, (v_ndc[1].y + 1.0) * 0.5 * h),
                    Vec2::new((v_ndc[2].x + 1.0) * 0.5 * w, (v_ndc[2].y + 1.0) * 0.5 * h),
                ];

                // Color per vertex (if available)
                let c_default = Color::WHITE;
                let v_color = [
                    entity.mesh.vertices[tri.vertices[0]].color.unwrap_or(c_default),
                    entity.mesh.vertices[tri.vertices[1]].color.unwrap_or(c_default),
                    entity.mesh.vertices[tri.vertices[2]].color.unwrap_or(c_default),
                ];

                // We'll just do wireframe rasterization for simplicity
                // Function to rasterize a line and push fragments
                let mut draw_line = |start: Vec2, end: Vec2, c: Color| {
                    use crate::pipeline::rasterizer::bresenham;
                    // Keep color as f32 values (0.0 to 1.0)
                    bresenham(start, end, Pixel::new_terminal('█', c), |pos, depth, _p| {
                        fragments.push(Fragment {
                            screen_pos: pos,
                            depth,
                            color: c,
                        });
                    });
                };

                // Draw the edges of the triangle
                draw_line(v_screen[0], v_screen[1], v_color[0]);
                draw_line(v_screen[1], v_screen[2], v_color[1]);
                draw_line(v_screen[2], v_screen[0], v_color[2]);
            }
        }

        fragments
    }
    fn process_fragments(&mut self, fragments: Vec<Fragment>, buffer: &mut TermBuffer) {
        for frag in fragments {
            let x = frag.screen_pos.x as usize;
            let y = frag.screen_pos.y as usize;

            // Convert Fragment color (which is Color) to Colorf32 for terminal pixel
            let c = Color {
                r: frag.color.r as f32 / 255.0,
                g: frag.color.g as f32 / 255.0,
                b: frag.color.b as f32 / 255.0,
            };

            buffer.set_pixel(x, y, frag.depth, Pixel::new_terminal('█', c));
        }
    }

    fn present(&mut self, back: &mut TermBuffer) -> std::io::Result<()> {
        self.front_buffer.render_to_terminal(&self.metrics)?;
        self.swap_buffers(back);
        Ok(())
    }

    fn cleanup(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
