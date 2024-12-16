use std::f32::consts::PI;

mod camera;
mod color;
mod colorf32;
mod scene;

pub use scene::Scene;
pub use scene::Entity;

pub use colorf32::Colorf32;

mod pixel;
pub use pixel::Pixel;
mod geometry{
    mod mesh;
    mod process;
    mod mat;

    pub use mat::Material;
    pub use mesh::{Mesh, Normal, Tri};
}
pub const MAX_PITCH: f32 = PI / 2.0;

pub use camera::Camera;
pub use color::Color;
