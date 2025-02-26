use glam::{Mat4, Quat, Vec3, Vec4};
use std::sync::{Arc, Mutex};

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
    dirty: Arc<Mutex<bool>>,
    cached_view_matrix: Arc<Mutex<Mat4>>,
    cached_proj_matrix: Arc<Mutex<Mat4>>,
    cached_frustum_planes: Arc<Mutex<[Vec4; 6]>>,
    cached_frustum_corners: Arc<Mutex<[Vec3; 8]>>,
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
            far: 50.1,

            dirty: Arc::new(Mutex::new(true)),
            cached_view_matrix: Arc::new(Mutex::new(Mat4::IDENTITY)),
            cached_proj_matrix: Arc::new(Mutex::new(Mat4::IDENTITY)),
            cached_frustum_planes: Arc::new(Mutex::new([Vec4::ZERO; 6])),
            cached_frustum_corners: Arc::new(Mutex::new([Vec3::ZERO; 8])),
        };

        // Initial cache update
        cam.update_direction();
        cam.update_cache();
        cam
    }

    fn update_cache(&self) {
        // Take all locks at once in a consistent order to avoid deadlocks
        let mut dirty = self.dirty.lock().unwrap();
        if *dirty {
            // Release the dirty lock before taking other locks
            *dirty = false;
            drop(dirty);

            // Now take the other locks
            let view_matrix = Mat4::look_at_rh(self.position, self.target, Vec3::Y);
            let proj_matrix =
                Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far);

            // Update matrices
            *self.cached_view_matrix.lock().unwrap() = view_matrix;
            *self.cached_proj_matrix.lock().unwrap() = proj_matrix;

            // Update frustum data using the new matrices
            // shoutout this paper
            // https://www.gamedevs.org/uploads/fast-extraction-viewing-frustum-planes-from-world-view-projection-matrix.pdf
            let vp = proj_matrix * view_matrix; // Matches scenario 2 "Frustum Planes in World Space"
            let mut planes = self.cached_frustum_planes.lock().unwrap();

            // Update frustum planes
            // reminder that glam is column major, so layout in memory of the Mat4 is
            // [M11, M21, M31, M41,  M12, M22, M32, M42,  M13, M23, M33, M43,  M14, M24, M34, M44]
            // but these values are accessed as follows:

            // Extract frustum planes - OpenGL formulas (column-major)
            // --- Left Plane ---
            planes[0] = Vec4::new(
                vp.col(3)[0] + vp.col(0)[0], // a = M41 + M11
                vp.col(3)[1] + vp.col(0)[1], // b = M42 + M12
                vp.col(3)[2] + vp.col(0)[2], // c = M43 + M13
                vp.col(3)[3] + vp.col(0)[3], // d = M44 + M14
            )
            .normalize();

            // --- Right Plane ---
            planes[1] = Vec4::new(
                vp.col(3)[0] - vp.col(0)[0], // a = M41 - M11
                vp.col(3)[1] - vp.col(0)[1], // b = M42 - M12
                vp.col(3)[2] - vp.col(0)[2], // c = M43 - M13
                vp.col(3)[3] - vp.col(0)[3], // d = M44 - M14
            )
            .normalize();

            // --- Bottom Plane ---
            planes[2] = Vec4::new(
                vp.col(3)[0] + vp.col(1)[0], // a = M41 + M21
                vp.col(3)[1] + vp.col(1)[1], // b = M42 + M22
                vp.col(3)[2] + vp.col(1)[2], // c = M43 + M23
                vp.col(3)[3] + vp.col(1)[3], // d = M44 + M24
            )
            .normalize();

            // --- Top Plane ---
            planes[3] = Vec4::new(
                vp.col(3)[0] - vp.col(1)[0], // a = M41 - M21
                vp.col(3)[1] - vp.col(1)[1], // b = M42 - M22
                vp.col(3)[2] - vp.col(1)[2], // c = M43 - M23
                vp.col(3)[3] - vp.col(1)[3], // d = M44 - M24
            )
            .normalize();

            // --- Near Plane ---
            planes[4] = Vec4::new(
                vp.col(3)[0] + vp.col(2)[0], // a = M41 + M31
                vp.col(3)[1] + vp.col(2)[1], // b = M42 + M32
                vp.col(3)[2] + vp.col(2)[2], // c = M43 + M33
                vp.col(3)[3] + vp.col(2)[3], // d = M44 + M34
            )
            .normalize();

            // --- Far Plane ---
            planes[5] = Vec4::new(
                vp.col(3)[0] - vp.col(2)[0], // a = M41 - M31
                vp.col(3)[1] - vp.col(2)[1], // b = M42 - M32
                vp.col(3)[2] - vp.col(2)[2], // c = M43 - M33
                vp.col(3)[3] - vp.col(2)[3], // d = M44 - M34
            )
            .normalize();

            // Update frustum corners
            let mut corners = self.cached_frustum_corners.lock().unwrap();
            let fov_rad = self.fov;
            let near_height = 2.0 * self.near * (fov_rad / 2.0).tan();
            let near_width = near_height * self.aspect_ratio;
            let far_height = 2.0 * self.far * (fov_rad / 2.0).tan();
            let far_width = far_height * self.aspect_ratio;

            let forward = self.forward();
            let right = self.right();
            let up = self.up();

            let near_center = self.position + forward * self.near;
            let far_center = self.position + forward * self.far;

            *corners = [
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
    }

    fn update_direction(&mut self) {
        let direction = (self.target - self.position).normalize();
        self.orientation = Quat::from_rotation_arc(Vec3::Z, direction);
    }

    // Movement methods
    pub fn move_forward(&mut self, distance: f32) {
        self.position += self.forward() * distance;
        self.target += self.forward() * distance;
        *self.dirty.lock().unwrap() = true;
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.position -= self.forward() * distance;
        self.target -= self.forward() * distance;
        *self.dirty.lock().unwrap() = true;
    }

    pub fn move_right(&mut self, amount: f32) {
        let right = self.right();
        self.position += right * amount;
        self.target += right * amount;
        *self.dirty.lock().unwrap() = true;
    }

    pub fn move_left(&mut self, amount: f32) {
        let right = self.right();
        self.position -= right * amount;
        self.target -= right * amount;
        *self.dirty.lock().unwrap() = true;
    }

    pub fn move_up(&mut self, amount: f32) {
        self.position += Vec3::Y * amount;
        *self.dirty.lock().unwrap() = true;
    }

    pub fn move_down(&mut self, amount: f32) {
        self.position -= Vec3::Y * amount;
        *self.dirty.lock().unwrap() = true;
    }

    //pub fn rotate(&mut self, pitch: f32, yaw: f32) {
    //    let pitch_rotation = Quat::from_axis_angle(self.right(), pitch);
    //    let yaw_rotation = Quat::from_axis_angle(Vec3::Y, yaw);
    //    self.orientation = yaw_rotation * pitch_rotation * self.orientation;
    //    *self.dirty.lock().unwrap() = true;
    //}

    pub fn orbit(&mut self, angle: f32) {
        let current_angle = self.position.z.atan2(self.position.x);
        let new_angle = current_angle + angle;
        let radius = (self.position.x * self.position.x + self.position.z * self.position.z).sqrt();

        self.position.x = radius * new_angle.cos();
        self.position.z = radius * new_angle.sin();

        // Update direction and orientation
        let direction = (-self.position).normalize();
        self.orientation = Quat::from_rotation_arc(Vec3::Z, direction);
        *self.dirty.lock().unwrap() = true;
    }

    // Getters
    pub fn view_matrix(&self) -> Mat4 {
        self.update_cache();
        *self.cached_view_matrix.lock().unwrap()
    }

    pub fn projection_matrix(&self) -> Mat4 {
        self.update_cache();
        *self.cached_proj_matrix.lock().unwrap()
    }

    pub fn frustum_planes(&self) -> [Vec4; 6] {
        self.update_cache();
        *self.cached_frustum_planes.lock().unwrap()
    }

    pub fn frustum_corners(&self) -> [Vec3; 8] {
        self.update_cache();
        *self.cached_frustum_corners.lock().unwrap()
    }

    pub fn forward(&self) -> Vec3 {
        //        (self.target - self.position).normalize()
        self.orientation * Vec3::Z // Moving to this because the camera is first person/free moving
    }

    pub fn right(&self) -> Vec3 {
        //self.forward().cross(-Vec3::Y).normalize()
        self.orientation * Vec3::X
    }

    pub fn up(&self) -> Vec3 {
        self.orientation * -Vec3::Y
    }

    // pub fn pitch(&self) -> f32 {
    //self.forward().dot(Vec3::Y).asin()
    //}
    /// Rotate the camera by a combined pitch (rotation about the right vector)
    /// and yaw (rotation about the global Y axis).
    pub fn rotate(&mut self, pitch: f32, yaw: f32) {
        // Rotate around the camera’s local right axis for pitch.
        let pitch_rot = Quat::from_axis_angle(self.right(), pitch);
        // Rotate around the global Y axis for yaw.
        let yaw_rot = Quat::from_axis_angle(Vec3::Y, yaw);
        // The order of multiplication matters!
        self.orientation = yaw_rot * pitch_rot * self.orientation;
        // Update the target based on the new forward direction.
        self.target = self.position + self.forward();
        *self.dirty.lock().unwrap() = true;
    }

    /// Rotate the camera by a pitch angle (rotation about the camera’s right vector).
    pub fn pitch(&mut self, angle: f32) {
        let pitch_rot = Quat::from_axis_angle(self.right(), angle);
        self.orientation = pitch_rot * self.orientation;
        self.target = self.position + self.forward();
        *self.dirty.lock().unwrap() = true;
    }

    /// Rotate the camera by a yaw angle (rotation about the global Y axis).
    pub fn yaw(&mut self, angle: f32) {
        let yaw_rot = Quat::from_axis_angle(Vec3::Y, angle);
        self.orientation = yaw_rot * self.orientation;
        self.target = self.position + self.forward();
        *self.dirty.lock().unwrap() = true;
    }

    /// Rotate the camera by a roll angle (rotation about the camera’s forward axis).
    pub fn roll(&mut self, angle: f32) {
        let roll_rot = Quat::from_axis_angle(self.forward(), angle);
        self.orientation = roll_rot * self.orientation;
        self.target = self.position + self.forward();
        *self.dirty.lock().unwrap() = true;
    }

    pub fn orbital_angle(&self) -> f32 {
        self.position.z.atan2(self.position.x)
    }

    pub fn reset(&mut self) {
        self.position = Vec3::ZERO;
        self.orientation = Quat::IDENTITY;
        *self.dirty.lock().unwrap() = true;
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
