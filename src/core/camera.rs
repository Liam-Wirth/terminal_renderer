use glam::{Mat4, Quat, Vec3, Vec4, Vec4Swizzles};
use std::cell::RefCell;

#[derive(Clone)]
pub struct Camera {
    // TODO: implement a controls mode that allows for like elite/elitedangerous style controls for
    // the minigame
    position: Vec3,
    orientation: Quat,
    target: Vec3,
    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,

    // Cache
    dirty: RefCell<bool>,
    cached_view_matrix: RefCell<Mat4>,
    cached_proj_matrix: RefCell<Mat4>,
    cached_frustum_planes: RefCell<[Vec4; 6]>,
    cached_frustum_corners: RefCell<[Vec3; 8]>,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, aspect_ratio: f32) -> Self {
        let direction = (target - position).normalize();
        let orientation = Quat::from_rotation_arc(Vec3::Z, direction);

        let mut cam = Self {
            position,
            orientation,
            target,
            fov: 60.0_f32.to_radians(),
            aspect_ratio,
            near: 0.1,
            far: 100.0,

            dirty: RefCell::new(true),
            cached_view_matrix: RefCell::new(Mat4::IDENTITY),
            cached_proj_matrix: RefCell::new(Mat4::IDENTITY),
            cached_frustum_planes: RefCell::new([Vec4::ZERO; 6]),
            cached_frustum_corners: RefCell::new([Vec3::ZERO; 8]),
        };

        // Initial cache update
        cam.update_direction();
        cam.update_cache();
        cam
    }

    fn update_cache(&self) {
        if *self.dirty.borrow() {
            *self.cached_view_matrix.borrow_mut() =
                Mat4::look_at_rh(self.position, self.target, -Vec3::Y);
            *self.cached_proj_matrix.borrow_mut() =
                Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far);
            // Update frustum planes and corners
            self.update_frustum_planes();
            self.update_frustum_corners();

            *self.dirty.borrow_mut() = false;
        }
    }

    fn update_direction(&mut self) {
        let direction = (self.target - self.position).normalize();
        self.orientation = Quat::from_rotation_arc(Vec3::Z, direction);
    }

    fn update_frustum_planes(&self) {
        let vp = *self.cached_proj_matrix.borrow() * *self.cached_view_matrix.borrow();
        let mut planes = self.cached_frustum_planes.borrow_mut();

        // Construct the six frustum planes from the view-projection matrix
        // The planes are: Left, Right, Bottom, Top, Near, Far
        for (i, sign) in [(0, 1), (0, -1), (1, 1), (1, -1), (2, 1), (2, -1)].iter() {
            let row = vp.row(3) + vp.row(*i) * (*sign as f32);
            let normal = Vec3::new(row.x, row.y, row.z);
            let length = normal.length();

            planes[i * 2 + if *sign > 0 { 0 } else { 1 }] = row / length;
        }
    }

    fn update_frustum_corners(&self) {
        let fov_rad = self.fov;
        let near_height = 2.0 * self.near * (fov_rad / 2.0).tan();
        let near_width = near_height * self.aspect_ratio;
        let far_height = 2.0 * self.far * (fov_rad / 2.0).tan();
        let far_width = far_height * self.aspect_ratio;

        let forward = self.get_forward();
        let right = self.get_right();
        let up = self.get_up();

        let near_center = self.position + forward * self.near;
        let far_center = self.position + forward * self.far;

        *self.cached_frustum_corners.borrow_mut() = [
            near_center + up * (near_height / 2.0) - right * (near_width / 2.0),
            near_center + up * (near_height / 2.0) + right * (near_width / 2.0),
            near_center - up * (near_height / 2.0) - right * (near_width / 2.0),
            near_center - up * (near_height / 2.0) + right * (near_width / 2.0),
            far_center + up * (far_height / 2.0) - right * (far_width / 2.0),
            far_center + up * (far_height / 2.0) + right * (far_width / 2.0),
            far_center - up * (far_height / 2.0) - right * (far_width / 2.0),
            far_center - up * (far_height / 2.0) + right * (far_width / 2.0),
        ];
    }

    // Movement methods
    pub fn move_forward(&mut self, distance: f32) {
        self.position += self.get_forward() * distance;
        *self.dirty.borrow_mut() = true;
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.position -= self.get_forward() * distance;
        *self.dirty.borrow_mut() = true;
    }

    pub fn move_right(&mut self, amount: f32) {
        let right = self.get_right();
        self.position += right * amount;
        self.target += right * amount;
        // No need to update_direction() here since relative orientation stays the same
        *self.dirty.borrow_mut() = true;
    }
    pub fn move_left(&mut self, amount: f32) {
        let right = self.get_right();
        self.position -= right * amount;
        self.target -= right * amount;
        // No need to update_direction() here since relative orientation stays the same
        *self.dirty.borrow_mut() = true;
    }

    pub fn move_up(&mut self, amount: f32) {
        self.position += Vec3::Y * amount;
        *self.dirty.borrow_mut() = true;
    }

    pub fn move_down(&mut self, amount: f32) {
        self.position -= Vec3::Y * amount;
        *self.dirty.borrow_mut() = true;
    }

    pub fn rotate(&mut self, pitch: f32, yaw: f32) {
        let pitch_rotation = Quat::from_axis_angle(self.get_right(), pitch);
        let yaw_rotation = Quat::from_axis_angle(Vec3::Y, yaw);
        self.orientation = yaw_rotation * pitch_rotation * self.orientation;
        *self.dirty.borrow_mut() = true;
    }

    pub fn orbit(&mut self, angle: f32) {
        let radius = self.position.length();
        let new_x = radius * angle.cos();
        let new_z = radius * angle.sin();
        self.position = Vec3::new(new_x, self.position.y, new_z);
        let direction = (-self.position).normalize();
        self.orientation = Quat::from_rotation_arc(Vec3::Z, direction);
        *self.dirty.borrow_mut() = true;
    }

    // Getters
    pub fn get_view_matrix(&self) -> Mat4 {
        self.update_cache();
        *self.cached_view_matrix.borrow()
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        self.update_cache();
        *self.cached_proj_matrix.borrow()
    }

    pub fn get_frustum_planes(&self) -> [Vec4; 6] {
        self.update_cache();
        *self.cached_frustum_planes.borrow()
    }

    pub fn get_frustum_corners(&self) -> [Vec3; 8] {
        self.update_cache();
        *self.cached_frustum_corners.borrow()
    }

    pub fn get_forward(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    pub fn get_right(&self) -> Vec3 {
        self.get_forward().cross(-Vec3::Y).normalize()
    }

    pub fn get_up(&self) -> Vec3 {
        self.orientation * -Vec3::Y
    }

    pub fn get_pitch(&self) -> f32 {
        self.get_forward().dot(Vec3::Y).asin()
    }

    pub fn get_orbital_angle(&self) -> f32 {
        self.position.z.atan2(self.position.x)
    }
    pub fn reset(&mut self) {
        self.position = Vec3::ZERO;
        self.orientation = Quat::IDENTITY;
        *self.dirty.borrow_mut() = true;
    }

    pub fn target(&self) -> Vec3 {
        self.target
    }

    pub fn fov(&self) -> f32 {
        self.fov
    }

    pub fn orientation(&self) -> Quat {
        self.orientation
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }
}
