use glam::{Mat4, Vec3, Vec4};

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale_factor: Vec3,
    pub computed_matrix: Mat4, // Cached transformation matrix
    needs_update: bool,         // Flag to track if recalculation is needed
}

// NOTE: In the glam docs, they talk about affine transformations and using a specific thing for
// that?
// they say something along the lines of the Mat4 being best for projections, look into that!
impl Transform {
    pub fn new() -> Self {
        Transform {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale_factor: Vec3::ONE,
            computed_matrix: Mat4::IDENTITY,
            needs_update: true,
        }
    }

    /// Get the transformation matrix, recalculating it only if necessary
    pub fn get_matrix(&mut self) -> Mat4 {
        if self.needs_update {
            // Rebuild transformation matrix from position, rotation, and scale
            let translation = Mat4::from_translation(self.position);
            let rotation_x = Mat4::from_rotation_x(self.rotation.x);
            let rotation_y = Mat4::from_rotation_y(self.rotation.y);
            let rotation_z = Mat4::from_rotation_z(self.rotation.z);
            let scale = Mat4::from_scale(self.scale_factor);

            self.computed_matrix = translation * rotation_z * rotation_y * rotation_x * scale;
            self.needs_update = false;
        }
        self.computed_matrix
    }

    pub fn mark_as_dirty(&mut self) {
        self.needs_update = true;
    }

    pub fn apply_to_point(&mut self, point: Vec3) -> Vec3 {
        let homogenous_point = Vec4::new(point.x, point.y, point.z, 1.0);
        let transformed = self.get_matrix() * homogenous_point;
        transformed.truncate()
    }

    pub fn apply_to_vector(&mut self, vector: Vec3) -> Vec3 {
        let rotation_scale_matrix = self.get_matrix().to_scale_rotation_translation().0;
        rotation_scale_matrix * vector
    }

    pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.rotation += Vec3::new(x, y, z);
        self.mark_as_dirty();
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.position += Vec3::new(x, y, z);
        self.mark_as_dirty();
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale_factor *= Vec3::new(x, y, z);
        self.mark_as_dirty();
    }

    pub fn scale_uniform(&mut self, factor: f32) {
        self.scale(factor, factor, factor);
    }

    pub fn reset(&mut self) {
        self.position = Vec3::ZERO;
        self.rotation = Vec3::ZERO;
        self.scale_factor = Vec3::ONE;
        self.mark_as_dirty();
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

