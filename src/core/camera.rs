use glam::{Mat4, Vec3, Quat};
use super::MAX_PITCH;

pub struct Camera {
    position: Vec3,
    orientation: Quat,
    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, aspect_ratio: f32) -> Self {
        let direction = (target - position).normalize();
        let orientation = Quat::from_rotation_arc(Vec3::Z, direction);

        Self {
            position,
            orientation,
            fov: 60.0_f32.to_radians(),
            aspect_ratio,
            near: 0.1,
            far: 1000.0,
        }
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(
            self.position,
            self.get_forward(),
            self.get_up(),
        )
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.fov,
            self.aspect_ratio,
            self.near,
            self.far,
        )
    }

    pub fn move_forward(&mut self, distance: f32) {
        self.position += self.get_forward() * distance;
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.position -= self.get_forward() * distance;
    }

    pub fn move_right(&mut self, amount: f32) {
        self.position += self.get_right() * amount;
    }

    pub fn move_left(&mut self, amount: f32) {
        self.position -= self.get_right() * amount;
    }

    pub fn move_up(&mut self, amount: f32) {
        self.position += Vec3::Y * amount;
    }

    pub fn move_down(&mut self, amount: f32) {
        self.position -= Vec3::Y * amount;
    }

    pub fn rotate(&mut self, pitch: f32, yaw: f32) {
        let pitch_rotation = Quat::from_axis_angle(self.get_right(), pitch);
        let yaw_rotation = Quat::from_axis_angle(Vec3::Y, yaw);

        self.orientation = yaw_rotation * pitch_rotation * self.orientation;
    }

    pub fn get_forward(&self) -> Vec3 {
        self.orientation * -Vec3::Z
    }

    pub fn get_right(&self) -> Vec3 {
        self.orientation * Vec3::X
    }

    pub fn get_up(&self) -> Vec3 {
        self.orientation * Vec3::Y
    }

    pub fn get_pitch(&self) -> f32 {
        self.get_forward().dot(Vec3::Y).asin()
    }
}
