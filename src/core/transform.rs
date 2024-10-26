use nalgebra::{Isometry3, Matrix4, Point3, Rotation3, SimdValue, Similarity3, Vector3};

/// A struct to represent a transformation in 3D space
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Point3<f64>,
    pub rotation: Vector3<f64>,
    pub scale_factor: Vector3<f64>,
    pub computed_matrix: Matrix4<f64>, // Cached transformation matrix
    needs_update: bool,                // Flag to track if a recalculation is needed
}

impl Transform {
    pub fn new() -> Self {
        Transform {
            position: Point3::new(0., 0., 0.),
            rotation: Vector3::zeros(),
            scale_factor: Vector3::new(1., 1., 1.),
            computed_matrix: Matrix4::identity(), // Initialize with identity matrix
            needs_update: true,                   // We need to update the matrix initially
        }
    }

    /// Get the transformation matrix, recalculating it only if necessary
    pub fn get_matrix(&mut self) -> Matrix4<f64> {
        if self.needs_update {
            // Use Isometry3 to create the transformation matrix including translation and rotation
            let isometry = Isometry3::new(self.position.coords, self.rotation);
            let rotation_matrix = isometry.to_homogeneous();

            // Apply scaling using a non-uniform scaling matrix
            let scale = Matrix4::new_nonuniform_scaling(&self.scale_factor);

            // Combine the rotation and scaling matrices to form the final transformation matrix
            self.computed_matrix = rotation_matrix * scale;

            self.needs_update = false; // Mark matrix as updated
        }
        self.computed_matrix
    }

    pub fn apply_to_point(&mut self, point: Point3<f64>) -> Point3<f64> {
        let homogenous_vertex = point.to_homogeneous(); // Convert Vector3 to Vector4
        let transformed = self.get_matrix() * homogenous_vertex;
        Point3::from_homogeneous(transformed).unwrap()
    }

    /// Apply transformation to a Vector3 (for directions)
    pub fn apply_to_vector(&mut self, vector: Vector3<f64>) -> Vector3<f64> {
        // Transform a vector using only rotation and scale, ignoring translation
        let mat = self.get_matrix();
        let rotation_scale_matrix = mat.fixed_view::<3, 3>(0, 0);
        rotation_scale_matrix * vector
    }

    /// Rotate the transformation and mark as needing update
    pub fn rotate(&mut self, x: f64, y: f64, z: f64) {
        self.rotation += Vector3::new(x, y, z);
        self.needs_update = true; // Mark matrix as needing an update
    }

    /// Translate the transformation and mark as needing update
    pub fn translate(&mut self, x: f64, y: f64, z: f64) {
        self.position += Vector3::new(x, y, z);
        self.needs_update = true; // Mark matrix as needing an update
    }

    /// Scale the transformation and mark as needing update
    pub fn scale(&mut self, x: f64, y: f64, z: f64) {
        self.scale_factor
            .component_mul_assign(&Vector3::new(x, y, z));
        self.needs_update = true; // Mark matrix as needing an update
    }

    /// Scale uniformly and mark as needing update
    pub fn scale_uniform(&mut self, factor: f64) {
        self.scale(factor, factor, factor);
    }

    /// Reset the transformation to its default state and mark as needing update
    pub fn reset(&mut self) {
        self.position = Point3::origin();
        self.rotation = Vector3::zeros();
        self.scale_factor = Vector3::new(1.0, 1.0, 1.0);
        self.needs_update = true; // Mark matrix as needing an update
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
