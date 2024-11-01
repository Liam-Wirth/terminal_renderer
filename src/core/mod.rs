use std::f32::consts::PI;

pub mod camera;
pub mod geometry;
pub mod color;
pub mod transform;
pub mod entity;
pub mod scene;

const MAX_PITCH: f32 = PI / 2.0;

pub use color::Color;
