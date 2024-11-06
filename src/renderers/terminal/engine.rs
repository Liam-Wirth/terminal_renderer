use crate::core::{camera::Camera, scene::Scene};
use crate::renderers::terminal::term_pipeline::TermPipeline;
use std::time::{Duration, Instant};

pub struct Engine {
    pub renderer: TermPipeline,
    pub scene: Scene,
    pub camera: Camera,
    last_frame: Instant,
    frame_time: Duration,
}

impl Engine {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            renderer: TermPipeline::new(width as usize, height as usize),
            scene: Scene::new(),
            camera: Camera::new(
                glam::Vec3::new(0.0, 0.0, -113.0), // position
                glam::Vec3::new(0.0, 0.0, 1.0),    // facing
                width as f32 / height as f32,      // aspect ratio
            ),
            last_frame: Instant::now(),
            frame_time: Duration::from_secs_f32(1.0 / 144.0), // 60 FPS target
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for entity in &mut self.scene.entities {
            entity
                .transform
                .rotate_quat(glam::Quat::from_rotation_x(0.01));
            entity
                .transform
                .rotate_quat(glam::Quat::from_rotation_y(0.01));
        }
    }

    pub fn render_frame(&mut self) -> std::io::Result<()> {
        self.renderer.render_frame(&self.scene, &self.camera)
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        loop {
            let now = Instant::now();
            let delta = now - self.last_frame;

            if delta >= self.frame_time {
                self.update(delta.as_secs_f32());
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
                    _ => {}
                }
            }
        }
        Ok(false)
    }
}
