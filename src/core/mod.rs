use std::f32::consts::PI;

pub mod camera;
pub mod color;
pub mod engine;
pub mod entity;
pub mod geometry;
pub mod scene;
pub mod transform;

const MAX_PITCH: f32 = PI / 2.0;

pub use color::Color;

pub use camera::{Camera, ProjectedVertex};
pub use entity::Entity;
pub use geometry::{Mesh, Tri, Vert};
pub use scene::Scene;
pub use transform::Transform;
