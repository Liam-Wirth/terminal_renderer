use glam::{Mat4, Vec2, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub direction: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub fov: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Self {
        let direction = Vec3::new(0.0, 0.0, 1.0);
        let up = Vec3::Y; // Positive Y as the up direction
        let right = direction.cross(up).normalize();

        Camera {
            position: Vec3::new(0.0, 0.0, -5.0),
            direction,
            right,
            up,
            fov: 90.0,
        }
    }

    // FIX: The movement is broken here
    pub fn update_orientation_vectors(&mut self) {
        self.right = self.direction.cross(Vec3::Y).normalize();
        self.up = self.right.cross(self.direction).normalize();
    }

    pub fn move_forward(&mut self, dist: f32) {
        self.position += self.direction * dist;
    }

    pub fn move_backward(&mut self, dist: f32) {
        self.position -= self.direction * dist;
    }

    pub fn strafe_right(&mut self, amount: f32) {
        self.position += self.right * amount;
    }

    pub fn strafe_left(&mut self, amount: f32) {
        self.position -= self.right * amount;
    }

    pub fn move_up(&mut self, amount: f32) {
        self.position += Vec3::Y * amount;
    }

    pub fn move_down(&mut self, amount: f32) {
        self.position -= Vec3::Y * amount;
    }

    pub fn turn_left(&mut self, angle: f32) {
        let rotation = Mat4::from_rotation_y(angle);
        self.direction = (rotation * self.direction.extend(0.0)).truncate();
        self.update_orientation_vectors();
    }

    pub fn turn_right(&mut self, angle: f32) {
        let rotation = Mat4::from_rotation_y(-angle);
        self.direction = (rotation * self.direction.extend(0.0)).truncate();
        self.update_orientation_vectors();
    }

    pub fn turn_up(&mut self, angle: f32) {
        let rotation = Mat4::from_axis_angle(self.right, -angle);
        self.direction = (rotation * self.direction.extend(0.0)).truncate();
        self.update_orientation_vectors();
    }

    pub fn turn_down(&mut self, angle: f32) {
        let rotation = Mat4::from_axis_angle(self.right, angle);
        self.direction = (rotation * self.direction.extend(0.0)).truncate();
        self.update_orientation_vectors();
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(
            self.position,
            self.position + self.direction,
            self.up,
        )
    }

    pub fn project_vertex(
        &self,
        v: Vec3,
        screen_width: usize,
        screen_height: usize,
    ) -> Vec2 {
        let fov_adj = (self.fov.to_radians() / 2.0).tan();
        let aspect = screen_width as f32 / screen_height as f32;

        let x = (v.x / (v.z * fov_adj * aspect) + 1.0) * 0.5 * screen_width as f32;
        let y = (1.0 - v.y / (v.z * fov_adj)) * 0.5 * screen_height as f32;

        Vec2::new(x, y)
    }
}

