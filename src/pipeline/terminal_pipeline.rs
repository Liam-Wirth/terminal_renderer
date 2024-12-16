use crate::core::{Camera, Color, Scene};
use crate::pipeline::{Fragment, Pipeline, ProcessedGeometry, ProjectedVertex};
use crate::renderers::terminal::TermBuffer;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal;
use glam::{Mat4, Vec2, Vec3};
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

impl Pipeline for TerminalPipeline {
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
        vec![ProcessedGeometry {
            transform: view_proj,
            visible: true,
        }]
    }

    fn rasterize(&mut self, _geometry: Vec<ProcessedGeometry>) -> Vec<Fragment> {
        // No rasterization logic for now
        Vec::new()
    }

    fn process_fragments(&mut self, _fragments: Vec<Fragment>, _buffer: &mut TermBuffer) {
        // No fragment processing logic for now
    }

    fn present(&mut self, back: &mut TermBuffer) -> std::io::Result<()> {
        self.front_buffer.render_to_terminal(&self.metrics);
        self.swap_buffers(back);
        Ok(())
    }

    fn cleanup(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
