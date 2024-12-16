use crate::core::{Camera, Color, Scene};
use crate::pipeline::{Fragment, Pipeline, ProcessedGeometry, ProjectedVertex};
use crate::renderers::window::WinBuffer;
use glam::{Mat4, Vec2, Vec3};
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
            scale: minifb::Scale::X2,
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
        vec![ProcessedGeometry {
            transform: view_proj,
            visible: true,
        }]
    }

    fn rasterize(&mut self, _geometry: Vec<ProcessedGeometry>) -> Vec<Fragment> {
        // No rasterization logic for now
        Vec::new()
    }

    fn process_fragments(&mut self, _fragments: Vec<Fragment>, buffer: &mut WinBuffer) {
        // No fragment processing logic for now
        //
        buffer.draw_text(&self.metrics, 10, 10, Color::GREEN);
    }

    fn present(&mut self, back: &mut WinBuffer) -> std::io::Result<()> {
        let (win_width, win_height) = self.window.get_size();
        let buffer_size = win_width * win_height;

        // Get slice of exactly the size the window needs
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
