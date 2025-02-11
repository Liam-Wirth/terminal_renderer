use crate::core::Color;
use glam::{Mat4, Vec2};

pub mod rasterizer;

mod buffer;
pub use buffer::Buffer;
pub use buffer::FrameBuffer;
pub use buffer::TermBuffer;
pub use buffer::MAX_DIMS;
mod clipper;
pub mod pipeline;
pub use clipper::ClipTriangle;
pub use clipper::ClipVertex;
pub use clipper::Clipper;
use glam::Vec3;

/// Represents geometry that has been processed through the transformation pipeline
// TODO: Use clipVertex buffer and store indices here
#[derive(Clone, Debug)]
pub struct ProcessedGeometry {
    /// Transformed vertices in Clip Space (Model-View-Projection matrix)
    pub transform: Mat4,
    /// Index into the Scene's entity buffer
    pub entity_id: usize,
    pub vertices: [ClipVertex; 3],

    pub material_id: Option<usize>,

    pub world_pos: [usize; 3],
}

/// **Represents a vertex that has been projected onto screen space**
#[derive(Debug, Clone, Copy)]
pub struct ProjectedVertex {
    /// 2D position in screen space coordinates
    pub position: Vec2,
    /// Depth value for z-buffer calculations
    pub depth: f32,
    /// Color of the vertex
    pub color: Color,
}
/// ***A Pixel To Be***
///
/// Represents a potential pixel in the graphics pipeline before final rasterization
/// A Fragment is an intermediate stage between vertex processing and final pixel output,
/// containing all the information needed to potentially become a pixel in the final image.
#[derive(Clone)]
pub struct Fragment {
    /// Position in screen space coordinates
    pub screen_pos: Vec2,
    /// Depth value for z-buffer calculations
    pub depth: f32,
    /// Color of the fragment (Diffuse, before lighting pass)
    pub albedo: Color,

    pub normal: Vec3,
    // index into the mesh's material buffer // NOTE: Below stuff might be unneccessary if we just
    // use the material buffer index

    //pub ambient: Option<Color>,
    //pub diffuse: Option<Color>,
    //pub specular: Option<Color>,
    //pub shininess: Option<f32>,
    //pub dissolve: Option<f32>, // Transparency
}

impl Default for Fragment {
    /// Creates a default Fragment with:
    /// - Screen position at (0,0)
    /// - Maximum depth (infinity)
    /// - White color
    fn default() -> Self {
        Self {
            screen_pos: Vec2::ZERO,
            depth: f32::INFINITY,
            albedo: Color::WHITE,
            normal: Vec3::ZERO,
        }
    }
}

pub const FP_SHIFT: i32 = 9; // Adjust this to control precision/wobble
pub const FP_ONE: i32 = 1 << FP_SHIFT;
#[inline(always)]
pub fn to_fixed(f: f32) -> i32 {
    (f * FP_ONE as f32) as i32
}

#[inline(always)]
pub fn from_fixed(f: i32) -> f32 {
    f as f32 / FP_ONE as f32
}
