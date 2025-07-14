use std::f32::consts::PI;

mod camera;
mod color;
mod scene;
mod texture;
pub use scene::Entity;
pub use scene::{Background, Environment, RenderMode, Scene};

pub use color::Color;

mod pixel;
pub use pixel::Pixel;
pub mod geometry {
    mod mat;
    mod mesh;
    mod process;

    pub use mat::Material;
    pub use mesh::Vertex;
    pub use mesh::{Mesh, Tri};
}
// maybe dumb
pub const MAX_PITCH: f32 = PI / 2.0;

pub use camera::Camera;

mod light;
pub use light::BlinnPhongShading;
pub use light::FlatShading;
pub use light::Light;
pub use light::LightMode;
pub use light::LightingModel;
pub use light::LightType;

pub use texture::{Texture, TextureFilter, TextureManager};
