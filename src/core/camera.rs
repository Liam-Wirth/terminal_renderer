use std::cell::RefCell;

use super::MAX_PITCH;
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

impl Camera {
    pub fn new(pos: Vec3, facing: Vec3, aspect: f32) -> Self {
        let cam = Self {
            pos: RefCell::new(pos),
            facing: RefCell::new(facing.normalize()),
            up: RefCell::new(Vec3::Y),
            right: RefCell::new(facing.cross(Vec3::Y).normalize()),
            fov: 90.0_f32.to_radians(),
            aspect_ratio: RefCell::new(aspect),
            near: 0.1,
            far: 100.0,

            view_matrix: RefCell::new(Mat4::IDENTITY),
            projection_matrix: RefCell::new(Mat4::IDENTITY),

            view_dirty: RefCell::new(true),
            proj_dirty: RefCell::new(true),
            view_proj_dirty: RefCell::new(true),
        };
        cam.update_matrices();
        cam
    }
    fn update_matrices(&self) {
        let view = Mat4::look_at_rh(
            *self.pos.borrow(),
            *self.pos.borrow() + *self.facing.borrow(),
            *self.up.borrow(),
        );

        let projection = Mat4::perspective_rh(
            self.fov.to_radians(),
            *self.aspect_ratio.borrow(),
            self.near,
            self.far,
        );

        *self.view_matrix.borrow_mut() = view;
        *self.projection_matrix.borrow_mut() = projection;
        *self.view_proj_dirty.borrow_mut() = false;
    }

    pub fn get_view_projection_matrix(&self) -> Mat4 {
        if *self.view_proj_dirty.borrow() {
            self.update_matrices();
        }
        *self.projection_matrix.borrow() * *self.view_matrix.borrow()
    }

    /// Projects a vertex at some position in the world to screen space, and returns it's depth for the z-buffer
    ///**Screen Dim** is the dimensions of the screen in pixels, supply with the given crossterm context
    pub fn project_vertex_into(
        &self,
        world_pos: Vec3,
        screen_dim: &UVec2,
        out: &mut ProjectedVertex,
    ) {
        let view_proj = self.get_view_projection_matrix();
        let clip_pos = view_proj * world_pos.extend(1.0);

        if clip_pos.w <= 0.0 {
            out.pos = Vec2::new(-1.0, -1.0);
            out.depth = f32::INFINITY;
            return;
        }

        let w_recip = 1.0 / clip_pos.w;
        out.pos.x = ((clip_pos.x * w_recip + 1.0) * 0.5) * screen_dim.x as f32;
        out.pos.y = ((1.0 - clip_pos.y * w_recip) * 0.5) * screen_dim.y as f32;
        out.depth = clip_pos.z * w_recip;
    }

    /// Move the camera forward
    pub fn move_forward(&self, dist: f32) {
        *self.pos.borrow_mut() += *self.facing.borrow() * dist;
        *self.view_proj_dirty.borrow_mut() = true;
    }

    /// Move the camera backwards
    pub fn move_backward(&self, dist: f32) {
        *self.pos.borrow_mut() -= *self.facing.borrow() * dist;
        *self.view_proj_dirty.borrow_mut() = true;
    }

    /// Strafe the camera to the right
    pub fn move_right(&self, amount: f32) {
        *self.pos.borrow_mut() += *self.right.borrow() * amount;
        *self.view_proj_dirty.borrow_mut() = true;
    }

    /// Strafe the camera to the left
    pub fn move_left(&self, amount: f32) {
        *self.pos.borrow_mut() -= *self.right.borrow() * amount;
        *self.view_proj_dirty.borrow_mut() = true;
    }

    /// move the camera upwards on global y axis, irrelevant of local y axis
    pub fn move_up(&self, amount: f32) {
        *self.pos.borrow_mut() += Vec3::Y * amount;
        *self.view_proj_dirty.borrow_mut() = true;
    }

    /// move the camera downwards on global y axis, irrelevant of local y axis
    pub fn move_down(&self, amount: f32) {
        *self.pos.borrow_mut() -= Vec3::Y * amount;
        *self.view_proj_dirty.borrow_mut() = true;
    }

    /// Turn that jawn left and right (yaw)
    /// **theta** It is importatnt that this is in ***RADIANS***

    pub fn rotate_yaw(&self, angle: f32) {
        let rotation = Mat4::from_rotation_y(angle);
        let current_facing = *self.facing.borrow();
        let new_facing = (rotation * current_facing.extend(0.0)).truncate();
        *self.facing.borrow_mut() = new_facing;
        *self.right.borrow_mut() = new_facing.cross(Vec3::Y).normalize();
        *self.view_proj_dirty.borrow_mut() = true;
    }

    /// Turn that jawn Up and Down (Pitch)
    /// **theta** It is importatnt that this is in ***RADIANS***
    pub fn rotate_pitch(&self, angle: f32) {
        let right = *self.right.borrow();
        let current_facing = *self.facing.borrow();
        let rotation = Mat4::from_axis_angle(right, angle);
        let new_facing = (rotation * current_facing.extend(0.0)).truncate();
        let cur = new_facing.dot(Vec3::Y).asin();

        if cur.abs() < MAX_PITCH {
            *self.facing.borrow_mut() = new_facing;
            self.update_orientation_vectors();
        }
    }

    pub fn update_orientation_vectors(&self) {
        *self.right.borrow_mut() = self.facing.borrow().cross(Vec3::Y).normalize();
        *self.up.borrow_mut() = self.right.borrow().cross(*self.facing.borrow()).normalize();
        *self.view_proj_dirty.borrow_mut() = true;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectedVertex {
    pub pos: Vec2,
    pub depth: f32,
}

impl ProjectedVertex {
    pub fn new(pos: Vec2, depth: f32) -> Self {
        ProjectedVertex { pos, depth }
    }
}

impl Default for ProjectedVertex {
    fn default() -> Self {
        ProjectedVertex {
            pos: Vec2::ZERO,
            depth: f32::INFINITY,
        }
    }
}
