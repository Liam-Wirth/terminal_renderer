use crate::core::Color;
use crate::core::ProjectedVertex;
use glam::Mat4;
use glam::Vec2;

pub trait Pipeline {
    type Scene; // This is just for now
    type Camera;
    /// Process geometry and return processed vertices ready for rasterization
    fn process_geometry(
        &mut self,
        scene: &Self::Scene,
        camera: &Self::Camera,
    ) -> Vec<ProcessedGeometry>;

    /// Convert processed geometry into fragments
    fn rasterize(&mut self, geometry: Vec<ProcessedGeometry>) -> Vec<Fragment>;

    /// Process fragments and write to buffer
    fn process_fragments(&mut self, fragments: Vec<Fragment>);

    /// Present the final buffer to display
    fn present(&mut self) -> std::io::Result<()>;
}

#[derive(Clone)]
pub struct ProcessedGeometry {
    pub vertices: Vec<ProjectedVertex>,
    pub indices: Vec<u32>,
    pub transform: Mat4,
    pub visible: bool,
}

#[derive(Clone)]
pub struct Fragment {
    pub screen_pos: Vec2,
    pub depth: f32,
    pub color: Color,
}
