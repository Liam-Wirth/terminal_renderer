use super::MAX_PITCH;
use glam::{Mat4, Vec2, Vec3, Vec4Swizzles};

pub struct Camera {
    /// The Global position of the camera
    pub pos: Vec3,
    /// The direction the camera is facing
    pub facing: Vec3,
    /// The right vector of the camera (independant from world up)
    pub right: Vec3,
    /// The Up Vector of the camera
    pub up: Vec3,
    ///Camera's FOV
    pub fov: f32,
    /// The aspect ratio of the camera
    pub aspect_ratio: f32,

    /// The near plane of the camera, anything closer than this will not be rendered
    pub near: f32,
    /// The far plane of the camera, anything beyond this will not be rendered
    pub far: f32,

    view_matrix: Mat4,
    projection_matrix: Mat4,

    view_dirty: bool,
    proj_dirty: bool,
}

impl Camera {
    pub fn new(pos: Vec3, facing: Vec3, aspect: f32) -> Self {
        let mut cam = Self {
            pos,
            facing: facing.normalize(),
            up: Vec3::Y,
            right: facing.cross(Vec3::Y).normalize(),
            fov: 90.0_f32.to_radians(),
            aspect_ratio: aspect,
            near: 0.1,
            far: 100.0,

            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,

            view_dirty: true,
            proj_dirty: true,
        };
        cam.update_view_matrix();
        cam.update_projection_matrix();
        cam
    }

    fn update_view_matrix(&mut self) {
        self.view_matrix = Mat4::look_at_rh(self.pos, self.pos + self.facing, self.up);
        self.view_dirty = false;
    }

    fn update_projection_matrix(&mut self) {
        self.projection_matrix =
            Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far);
        self.proj_dirty = false;
    }
    pub fn view_matrix(&mut self) -> Mat4 {
        if self.view_dirty {
            self.update_view_matrix();
        }
        self.view_matrix
    }

    pub fn projection_matrix(&mut self) -> Mat4 {
        if self.proj_dirty {
            self.update_projection_matrix();
        }
        self.projection_matrix
    }
    /// Move the camera in the direction it is facing
    pub fn move_forward(&mut self, distance: f32) {
        self.pos += self.facing * distance;
        self.view_dirty = true;
    }

    /// Move the camera in the opposite direction it is facing
    pub fn move_backward(&mut self, distance: f32) {
        self.pos -= self.facing * distance;
        self.view_dirty = true;
    }

    /// Strafe the camera to the right
    pub fn move_right(&mut self, distance: f32) {
        self.pos += self.right * distance;
        self.view_dirty = true;
    }

    /// Strafe the camera to the left
    pub fn move_left(&mut self, distance: f32) {
        self.pos -= self.right * distance;
        self.view_dirty = true;
    }

    /// Move the camera up
    pub fn move_up(&mut self, distance: f32) {
        self.pos += self.up * distance;
        self.view_dirty = true;
    }

    /// Move the camera down
    pub fn move_down(&mut self, distance: f32) {
        self.pos -= self.up * distance;
        self.view_dirty = true;
    }

    /// Turn that jawn left and right (yaw)
    /// **theta** It is importatnt that this is in ***RADIANS***
    pub fn rotate_yaw(&mut self, theta: f32) {
        let rotation = Mat4::from_rotation_y(theta);
        self.facing = (rotation * self.facing.extend(1.0)).xyz().normalize();
        self.view_dirty = true;
    }

    /// Turn that jawn Up and Down (Pitch)
    /// **theta** It is importatnt that this is in ***RADIANS***
    pub fn rotate_pitch(&mut self, theta: f32) {
        let rot = Mat4::from_axis_angle(self.right, theta);
        let new_facing = (rot * self.facing.extend(1.0)).xyz().normalize();

        let cur = new_facing.dot(Vec3::Y).asin();

        if cur.abs() < MAX_PITCH {
            self.facing = new_facing;
            self.view_dirty = true;
        }
    }

    /// Set the aspect ratio of the camera
    /// this will allow for taking changes in screen size into account
    pub fn set_aspect_ratio(&mut self, aspect: f32) {
        self.aspect_ratio = aspect;
        self.proj_dirty = true;
    }
}
