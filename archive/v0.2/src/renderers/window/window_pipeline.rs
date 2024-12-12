use std::cell::RefCell;

use crate::core::{camera::Camera, scene::Scene};
use crate::renderers::Renderer;

use super::WinBuffer;
use minifb;

pub struct WindowPipeline {
    window: minifb::Window,
    pub frontbuffer: RefCell<WinBuffer>,
    pub backbuffer: RefCell<WinBuffer>,
}

impl Renderer for WindowPipeline {
    type PixelType = (); // We'll define this properly later
    type MetricsType = String;

    fn init(&mut self, width: usize, height: usize) {
        self.frontbuffer = RefCell::new(WinBuffer::new(width, height));
    }

    fn render_frame(&mut self, cam: &Camera, scene: &Scene, metrics: &String) {
        // Rendering implementation will go here
    }

    fn update_res(&mut self, width: usize, height: usize) {}
}

impl WindowPipeline {
    pub fn new(width: usize, height: usize) -> Self {
        todo!();
    }
}
