// NOTE THIS IS UNUSED/DEAD CODE FOR NOW, MIGHT USE IN THE FUTURE THO
use std::time::Duration;

use crate::core::{camera::Camera, scene::Scene};
use crate::renderers::Renderer;

pub struct Engine<R: Renderer> {
    pub renderer: R,
    pub scene: Scene,
    pub camera: Camera,
    last_frame: std::time::Instant,
    frame_time: Duration,
}

impl<R: Renderer> Engine<R> {
    fn new(renderer: R, scene: Scene, camera: Camera) -> Self {
        Engine {
            renderer,
            scene,
            camera,
            last_frame: std::time::Instant::now(),
            frame_time: Duration::default(),
        }
    }
}
