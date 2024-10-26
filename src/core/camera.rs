use nalgebra::{Isometry3, Matrix4, Point2, Point3, Rotation3, Unit, Vector2, Vector3};

pub struct Camera {
    pub position: Vector3<f64>,
    pub direction: Vector3<f64>,
    pub right: Vector3<f64>, // Right vector for strafing
    pub up: Vector3<f64>,    // Up vector for the camera’s "up" direction
    pub fov: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Self {
        let direction = Vector3::new(0.0, 0.0, 1.0);
        let up = Vector3::y(); // Assuming positive Y is up in your world space
        let right = direction.cross(&up).normalize();

        Camera {
            position: Vector3::new(0.0, 0.0, -5.0),
            direction,
            right,
            up,
            fov: 90.0,
        }
    }

    pub fn update_orientation_vectors(&mut self) {
        // Ensure the right and up vectors stay consistent with the direction vector
        self.right = self.direction.cross(&Vector3::y()).normalize();
        self.up = self.right.cross(&self.direction).normalize();
    }

    pub fn move_forward(&mut self, dist: f64) {
        self.position += self.direction * dist;
    }

    pub fn move_backward(&mut self, dist: f64) {
        self.position -= self.direction * dist;
    }

    pub fn strafe_right(&mut self, amount: f64) {
        self.position += self.right * amount;
    }

    pub fn strafe_left(&mut self, amount: f64) {
        self.position -= self.right * amount;
    }

    pub fn move_up(&mut self, amount: f64) {
        self.position += Vector3::y() * amount;
    }

    pub fn move_down(&mut self, amount: f64) {
        self.position -= Vector3::y() * amount;
    }

    pub fn turn_left(&mut self, angle: f64) {
        let rotation = Rotation3::from_axis_angle(&Vector3::y_axis(), angle);
        self.direction = rotation * self.direction;
        self.update_orientation_vectors();
    }

    pub fn turn_right(&mut self, angle: f64) {
        let rotation = Rotation3::from_axis_angle(&Vector3::y_axis(), -angle);
        self.direction = rotation * self.direction;
        self.update_orientation_vectors();
    }

    pub fn turn_up(&mut self, angle: f64) {
        let rotation = Rotation3::from_axis_angle(&Unit::new_normalize(self.right), -angle);
        self.direction = rotation * self.direction;
        self.update_orientation_vectors();
    }

    pub fn turn_down(&mut self, angle: f64) {
        let rotation = Rotation3::from_axis_angle(&Unit::new_normalize(self.right), angle);
        self.direction = rotation * self.direction;
        self.update_orientation_vectors();
    }

    pub fn get_view_matrix(&self) -> Matrix4<f64> {
        Matrix4::look_at_rh(
            &Point3::from(self.position),                  // Camera (eye) position
            &Point3::from(self.position + self.direction), // Target to look at
            &self.up,                                      // Up direction
        )
    }

    pub fn project_vertex(
        &self,
        v: Vector3<f64>,
        screen_width: &usize,
        screen_height: &usize,
    ) -> Point2<usize> {
        let fov_adj = (self.fov / 2.0).to_radians().tan();
        let aspect = *screen_width as f64 / *screen_height as f64;

        let x = (v.x / (v.z * fov_adj * aspect) + 1.0) * 0.5 * *screen_width as f64;
        let y = (1.0 - v.y / (v.z * fov_adj)) * 0.5 * *screen_height as f64;

        Point2::new(x as usize, y as usize)
    }
}
