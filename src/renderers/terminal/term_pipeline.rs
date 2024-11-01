use std::fmt::Write;

use crate::core::{camera::Camera, scene::Scene};
use crate::renderers::terminal::termbuffer::{Pixel, TermBuffer};
use glam::Vec2;
use glam::Vec4Swizzles;

pub struct TermPipeline {
    pub buffer: TermBuffer,
}

impl TermPipeline {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            buffer: TermBuffer::new(width, height),
        }
    }

    pub fn render(
        &mut self,
        scene: &mut Scene,
        camera: &mut Camera, ) {
        todo!();
    }
    fn draw_line(&mut self, start: Vec2, end: Vec2, depth0: f32, depth1: f32, pix: Pixel) {
        todo!();
    }
}
