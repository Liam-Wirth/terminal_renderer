use std::cell::RefCell;

use super::{Color, MAX_PITCH};
use glam::{Mat4, UVec2, Vec2, Vec3};

pub struct Camera {
    /// The Global.pos of the camera
    pub pos: RefCell<Vec3>,
    /// The direction the camera is facing
    pub facing: RefCell<Vec3>,
    /// The right vector of the camera (independant from world up)
    pub right: RefCell<Vec3>,
    /// The Up Vector of the camera
    pub up: RefCell<Vec3>,
    ///Camera's FOV
    pub fov: f32,
    /// The aspect ratio of the camera
    pub aspect_ratio: RefCell<f32>,

    /// The near plane of the camera, anything closer than this will not be rendered
    pub near: f32,
    /// The far plane of the camera, anything beyond this will not be rendered
    pub far: f32,

    view_dirty: RefCell<bool>,
    proj_dirty: RefCell<bool>,
    view_matrix: RefCell<Mat4>,
    projection_matrix: RefCell<Mat4>,
    view_proj_dirty: RefCell<bool>,
}

// TODO: Probably should maybe move this elsewhere into like a pipeline/fragmentation stage
#[derive(Debug, Clone, Copy)]
pub struct ProjectedVertex {
    pub pos: Vec2,
    pub depth: f32,
    pub color: crate::core::color::Color,
}

impl ProjectedVertex {
    pub fn new(pos: Vec2, depth: f32, color: Color) -> Self {
        ProjectedVertex { pos, depth, color }
    }
}

impl Default for ProjectedVertex {
    fn default() -> Self {
        ProjectedVertex {
            pos: Vec2::ZERO,
            depth: f32::INFINITY,
            color: super::Color::default(),
        }
    }
}
