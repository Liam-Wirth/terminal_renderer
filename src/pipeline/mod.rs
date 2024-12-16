use crate::core::{Camera, Color};
use glam::{Mat4, Vec2, Vec3, Vec4};

pub mod terminal_pipeline;
pub use terminal_pipeline::TerminalPipeline;
pub mod window_pipeline;
pub mod rasterizer;
pub use window_pipeline::WindowPipeline;
pub trait Pipeline {
    type Scene;
    type Camera;
    type Buffer;

    fn init(&mut self, width: usize, height: usize);
    // Transform and project meshes into Clip Space
    fn process_geometry(
        &mut self,
        scene: &Self::Scene,
        camera: &Self::Camera,
    ) -> Vec<ProcessedGeometry>;
    // Convert the triangles, into fragments
    fn rasterize(&mut self, geometry: Vec<ProcessedGeometry>, scene: &Self::Scene) -> Vec<Fragment>;
    //Apply shading using lighting, textures, or mats
    fn process_fragments(&mut self, fragments: Vec<Fragment>, buffer: &mut Self::Buffer);
    // Draw
    fn present(&mut self, back: &mut Self::Buffer) -> std::io::Result<()>;
    fn cleanup(&mut self) -> std::io::Result<()>;
}

#[derive(Clone, Debug)]
pub struct ProcessedGeometry {
    /// Transformed vertices in Clip Space (MVP)
    pub transform: Mat4, 
    /// Index into the Scene's entity buffer
    pub entity_id: usize, 
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

pub trait Shader {
    fn vertex(&mut self, face: usize, vert: usize, pos: &mut Vec4);
    fn fragment(&mut self, barycentric: Vec3) -> Option<Color>;
}
