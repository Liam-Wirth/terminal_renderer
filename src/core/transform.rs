use nalgebra::{Isometry3, Matrix4, Rotation3, Similarity3, Vector3};

pub struct Transform {
    pub position: Vector3<f64>,
    pub rotation: Vector3<f64>,
    pub scale_factor: Vector3<f64>,
}

impl Transform {
    pub fn new() -> Self {
        Transform {
            position: Vector3::zeros(),
            rotation: Vector3::zeros(),
            scale_factor: Vector3::new(1., 1., 1.),
        }
    }

    pub fn get_matrix(&self) -> Matrix4<f64> {
        // Use Isometry3 for translation and rotation combined
        let isometry = Isometry3::new(self.position, self.rotation);
        let rotation_matrix = isometry.to_homogeneous();
        
        // Apply scaling using Similarity3 for better readability
        let scale = Matrix4::new_nonuniform_scaling(&self.scale_factor);
        
        // Combine rotation and scaling matrices
        rotation_matrix * scale
    }

    pub fn apply_to_vertex(&self, vertex: Vector3<f64>) -> Vector3<f64> {
        let homogenous_vertex = vertex.push(1.0); // Convert Vector3 to Vector4
        let transformed = self.get_matrix() * homogenous_vertex;
        transformed.xyz() // Convert Vector4 back to Vector3 after transformation
    }

    pub fn rotate(&mut self, x: f64, y: f64, z: f64) {
        self.rotation += Vector3::new(x, y, z);
    }

    pub fn translate(&mut self, x: f64, y: f64, z: f64) {
        self.position += Vector3::new(x, y, z);
    }

    pub fn scale(&mut self, x: f64, y: f64, z: f64) {
        self.scale_factor.component_mul_assign(&Vector3::new(x, y, z));
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
