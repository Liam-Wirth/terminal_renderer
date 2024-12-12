use glam::Vec3;
use super::super::Color;

#[derive(Clone, Copy, Debug)]
pub struct Vert {
    pub pos: Vec3,
    pub norm: Vec3,
    pub color: Color,
}

impl Default for Vert {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            norm: Vec3::ZERO,
            color: Color::WHITE,
        }
    }
}
