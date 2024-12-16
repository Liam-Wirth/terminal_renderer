use crate::core::{Camera, Color};
use glam::{Mat4, Vec2};

pub mod terminal_pipeline;
pub use terminal_pipeline::TerminalPipeline;
pub mod window_pipeline;
pub use window_pipeline::WindowPipeline;
pub trait Pipeline {
    type Scene;
    type Camera;
    type Buffer;

    fn init(&mut self, width: usize, height: usize);
    fn process_geometry(
        &mut self,
        scene: &Self::Scene,
        camera: &Self::Camera,
    ) -> Vec<ProcessedGeometry>;
    fn rasterize(&mut self, geometry: Vec<ProcessedGeometry>) -> Vec<Fragment>;
    fn process_fragments(&mut self, fragments: Vec<Fragment>, buffer: &mut Self::Buffer);
    fn present(&mut self, back: &mut Self::Buffer) -> std::io::Result<()>;
    fn cleanup(&mut self) -> std::io::Result<()>;
}

#[derive(Clone, Debug)]
pub struct ProcessedGeometry {
    pub transform: Mat4,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectedVertex {
    pub position: Vec2,
    pub depth: f32,
    pub color: Color,
}

#[derive(Clone)]
pub struct Fragment {
    pub screen_pos: Vec2,
    pub depth: f32,
    pub color: Color,
}
