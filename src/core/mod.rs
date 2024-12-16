use std::f32::consts::PI;

mod camera;
mod color;
mod scene;

pub const MAX_PITCH: f32 = PI / 2.0;

pub use camera::Camera;
pub use color::Color;
pub use scene::Scene;
