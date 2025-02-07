use std::f32::consts::PI;

mod camera;
mod color;
mod scene;
pub use scene::{Scene, Background, Environment, RenderMode};
pub use scene::Entity;

pub use color::Color;

mod pixel;
pub use pixel::Pixel;
pub mod geometry{
    mod mesh;
    mod process;
    mod mat;

    pub use mat::Material;
    pub use mesh::{Mesh, Normal, Tri};
    pub use mesh::Vertex;
}
// maybe dumb
pub const MAX_PITCH: f32 = PI / 2.0;

pub use camera::Camera;

mod light;

