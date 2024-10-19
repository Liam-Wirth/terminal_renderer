use nalgebra::{Matrix4, Rotation, Rotation3, Vector3, Vector4};

// TODO: Look into using more of the Nalgebra functionality to instead be able to do this,
// especially for the get matrix function
//
// TODO: Also implement more helper methods, like rotate 90 x, rotate 90 y, etc
pub struct Transform {
    ///position (translation)
    pub position: Vector3<f64>,
    /// Rotation around global axis in radians
    pub rotation: Vector3<f64>,
    ///Scale Factor
    pub scale_factor: Vector3<f64>,
}

impl Transform {
    /// By default transform is not rotated, nor scaled, and is placed on the origin
    pub fn new() -> Self {
        Transform {
            position: Vector3::zeros(),
            rotation: Vector3::zeros(),
            scale_factor: Vector3::new(1., 1., 1.),
        }
    }
    pub fn get_matrix(&self) -> Matrix4<f64> {
        let translation = Matrix4::new_translation(&self.position);

        let rot_x = Matrix4::new_rotation(Vector3::x() * self.rotation.x);
        let rot_y = Matrix4::new_rotation(Vector3::y() * self.rotation.y);
        let rot_z = Matrix4::new_rotation(Vector3::z() * self.rotation.z);
        let rotation = rot_x * rot_y * rot_z;
        let scale = Matrix4::new_nonuniform_scaling(&self.scale_factor);

        return translation * rotation * scale;
    }

    pub fn apply_to_vertex(&self, vertex: Vector4<f64>) -> Vector4<f64> {
        return self.get_matrix() * vertex;
    }
    pub fn rotateX(&mut self, angle: f64) {
        self.rotation.x += angle;
    }
    pub fn rotateY(&mut self, angle: f64) {
        self.rotation.y += angle;
    }
    pub fn rotateZ(&mut self, angle: f64) {
        self.rotation.z += angle;
    }
    pub fn translate(&mut self, x: f64, y: f64, z: f64) {
        self.position.x += x;
        self.position.y += y;
        self.position.z += z;
    }
    pub fn scale(&mut self, x: f64, y: f64, z: f64) {
        self.scale_factor.x *= x;
        self.scale_factor.y *= y;
        self.scale_factor.z *= z;
    }
    pub fn scale_uniform(&mut self, factor: f64) {
        self.scale(factor, factor, factor);
    }
    pub fn reset(&mut self) {
        self.position = Vector3::zeros();
        self.rotation = Vector3::zeros();
        self.scale_factor = Vector3::new(1., 1., 1.);
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
