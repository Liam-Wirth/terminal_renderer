use crate::core::{camera::Camera, scene::Scene};
use crate::renderers::Renderer;

pub struct WindowPipeline {}

impl Renderer for WindowPipeline {
    type PixelType = (); // We'll define this properly later
    type MetricsType = String;

    fn init(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    fn render_frame(&mut self, cam: &Camera, scene: &Scene, metrics: &String) {
        // Rendering implementation will go here
    }

    fn update_res(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }
}

impl WindowPipeline {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}
