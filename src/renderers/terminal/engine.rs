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
            frame_time: Duration::from_secs_f32(1.0 / 60.0), // 60 FPS target
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for entity in &mut self.scene.entities {
            entity
                .transform
                .rotate_quat(glam::Quat::from_rotation_x(0.01));
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

        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(true),
                    KeyCode::Char('W') => self.camera.move_forward(0.1),
                    KeyCode::Char('S') => self.camera.move_backward(0.1),
                    KeyCode::Char('A') => self.camera.move_left(0.1),
                    KeyCode::Char('D') => self.camera.move_right(0.1),
                    KeyCode::Char(' ') => self.camera.move_up(0.1),
                    KeyCode::Char('c') => self.camera.move_down(0.1),
                    KeyCode::Left => self.camera.rotate_yaw(-0.1),
                    KeyCode::Right => self.camera.rotate_yaw(0.1),
                    KeyCode::Up => self.camera.rotate_pitch(0.1),
                    KeyCode::Down => self.camera.rotate_pitch(-0.1),
                    _ => {}
                }
            }
        }
        Ok(false)
    }
}
