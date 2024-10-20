use nalgebra::{Matrix4, Point3, Vector3, Vector2};

pub struct Camera {
    pub position: Vector3<f64>,
    pub direction: Vector3<f64>,
    pub fov: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vector3::new(0.0, 0.0, -5.0),
            direction: Vector3::new(0.0, 0.0, 1.0),
            fov: 90.0,
        }
    }

    pub fn move_forward(&mut self, dist: f64) {
        self.position += self.direction * dist;
    }

    pub fn move_backward(&mut self, dist: f64) {
        self.position -= self.direction * dist;
    }

    pub fn strafe_right(&mut self, amount: f64) {
        let left = self.direction.cross(&crate::GLOBAL_UP).normalize();
        self.position -= left * amount;
    }

    pub fn strafe_left(&mut self, amount: f64) {
        let right = self.direction.cross(&crate::GLOBAL_UP).normalize();

        self.position += right * amount;
    }

    pub fn turn_left(&mut self, angle: f64) {
        let rotation = Matrix4::new_rotation(Vector3::y() * angle);
        self.direction = rotation.transform_vector(&self.direction);
    }

    pub fn turn_right(&mut self, angle: f64) {
        let rotation = Matrix4::new_rotation(Vector3::y() * -angle);
        self.direction = rotation.transform_vector(&self.direction);
    }

    pub fn get_view_matrix(&self) -> Matrix4<f64> {
        //Matrix4::look_at_rh(eye, target, up)
        //eye: The position of the camera.
        //target: The position the camera is looking at.
        //up: The direction that is considered up.
        Matrix4::look_at_rh(
            &Point3::from(self.position),                  // Camera (eye) position
            &Point3::from(self.position + self.direction), // Target to look at
            &Vector3::y(),                                 // Up direction
        )
    }
    pub fn project_vertex(&self, v: Vector3<f64>, screen_width: &usize, screen_height: &usize) -> Vector2<usize>{
        let fov_adj = (self.fov / 2.0).to_radians().tan();
        // z-division, basically adjusts perspective so farther away objects look smaller, closer
        // ones look bigger
        let zdiv_x = v.x / (v.z * fov_adj);
        let zdiv_y = v.y / (v.z * fov_adj);

        let screen_x = ((zdiv_x + 1.)/2. * *screen_width as f64) as usize;
        let screen_y = ((1.0 - (zdiv_y + 1.0)/2.0) * *screen_height as f64) as usize;

        Vector2::new(screen_x, screen_y)
    }
}
