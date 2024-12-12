use crate::core::{camera::Camera, scene::Scene};
use crate::renderers::renderer::{cycle_render_mode, set_render_mode, RenderMode};
use crate::renderers::terminal::term_pipeline::TermPipeline;
use std::time::{Duration, Instant};

pub struct Engine {
    pub renderer: TermPipeline,
    pub scene: Scene,
    pub camera: Camera,
    last_frame: Instant,
    frame_time: Duration,
    fps_counter: u32,
    fps_update_timer: Instant,
    current_fps: f32,
    frame_times: Vec<f32>, // Store last N frame times for averaging
    metrics: String,
    tris: u32,
}

impl Engine {
    pub fn new(width: u32, height: u32) -> Self {
        set_render_mode(RenderMode::Wireframe);
        Self {
            renderer: TermPipeline::new(width as usize, height as usize),
            scene: Scene::new(),
            camera: Camera::new(
                glam::Vec3::new(0.0, 2.4, -6.0),
                glam::Vec3::new(0.0, 0.0, 1.0),
                width as f32 / height as f32,
            ),
            last_frame: Instant::now(),
            frame_time: Duration::from_secs_f32(1.0 / 60.0),
            fps_counter: 0,
            fps_update_timer: Instant::now(),
            current_fps: 0.0,
            frame_times: Vec::with_capacity(120), // Store last 120 frames
            tris: 0,
            metrics: String::new(),
        }
    }
    fn update_metrics(&mut self, frame_delta: Duration) {
        // Update FPS counter
        self.fps_counter += 1;

        // Store frame time
        self.frame_times.push(frame_delta.as_secs_f32() * 1000.0);
        if self.frame_times.len() > 120 {
            self.frame_times.remove(0);
        }

        // Update FPS and other metrics every second
        if self.fps_update_timer.elapsed() >= Duration::from_secs(1) {
            self.current_fps = self.fps_counter as f32;
            self.fps_counter = 0;
            self.fps_update_timer = Instant::now();
            self.tris = self
                .scene
                .entities
                .iter()
                .map(|e| e.mesh.tris.borrow().len() as u32)
                .sum();

            // Calculate average frame time
            let avg_frame_time =
                self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;

            // Update the title with metrics
            self.metrics = format!(
                "FPS: {:.1} | Frame Time: {:.2}ms | Entities: {} | Cam: ({:.1}, {:.1}, {:.1}), Tris: {}",
                self.current_fps,
                avg_frame_time,
                self.scene.entities.len(),
                self.camera.pos.borrow().x,
                self.camera.pos.borrow().y,
                self.camera.pos.borrow().z,
                self.tris,
            );
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for entity in &mut self.scene.entities {
            entity.transform.rotate_quat(glam::Quat::from_rotation_y(0.01));
            entity.transform.rotate_quat(glam::Quat::from_rotation_x(0.02));
            entity.transform.rotate_quat(glam::Quat::from_rotation_z(0.04));
            entity.mesh.update_visibility(*self.camera.pos.borrow(), &entity.transform.model_mat())
        }
    }

    pub fn render_frame(&mut self) -> std::io::Result<()> {
        self.renderer
            .render_frame(&self.scene, &self.camera, &self.metrics)
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        loop {
            let now = Instant::now();
            let delta = now - self.last_frame;

            if delta >= self.frame_time {
                self.update(delta.as_secs_f32());
                self.update_metrics(delta);
                self.render_frame()?;
                self.last_frame = now;
            }

            // Handle input
            if let Ok(true) = self.handle_input() {
                break;
            }
        }
        Ok(())
    }

    fn handle_input(&mut self) -> std::io::Result<bool> {
        use crossterm::event::{self, Event, KeyCode};

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
                    KeyCode::Char('w') | KeyCode::Char('W') => {
                        self.camera.move_forward(move_amount)
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        self.camera.move_backward(move_amount)
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => self.camera.move_left(move_amount),
                    KeyCode::Char('d') | KeyCode::Char('D') => self.camera.move_right(move_amount),
                    KeyCode::Char(' ') => self.camera.move_up(move_amount),
                    KeyCode::Char('c') => self.camera.move_down(move_amount),
                    KeyCode::Left => self.camera.rotate_yaw(-rotate_amount * 0.5),
                    KeyCode::Right => self.camera.rotate_yaw(rotate_amount * 0.5),
                    KeyCode::Up => self.camera.rotate_pitch(rotate_amount * 0.5),
                    KeyCode::Down => self.camera.rotate_pitch(-rotate_amount * 0.5),
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        cycle_render_mode();
                    }
                    KeyCode::Char('1') => {
                        self.scene.entities[0].transform.rotate_quat(glam::Quat::from_rotation_y(0.01));
                    }
                    _ => {}
                }
            }
        }
        Ok(false)
    }
}
