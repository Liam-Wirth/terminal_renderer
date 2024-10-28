use crate::core::color::Color;
use glam::{Mat4, Vec3};

#[derive(Debug, Clone)]
pub struct Tri {
    pub vertices: (usize, usize, usize), // Indices into the vertex array
    pub color: Color,                    // Optional, or each face has a single color
    pub normal: Vec3,                    // The surface normal for the triangle
}

impl Tri {
    pub fn calculate_normal(&mut self, v1: &Vec3, v2: &Vec3, v3: &Vec3) {
        // Calculate normal using the vectors derived from Point3 coordinates
        self.normal = (v2 - v1).cross(v3 - v1).normalize();
    }

    pub fn transform_normal(&mut self, transform: &Mat4) {
        // Apply rotation and scaling using the 3x3 rotation-scale part of Mat4, focusing purely on
        // rotation and scaling, not translation, (hence the use of transform_vector3)
        self.normal = (transform.transform_vector3(self.normal)).normalize();
    }
}
