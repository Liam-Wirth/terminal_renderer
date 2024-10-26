use crate::core::color::Color;
use nalgebra::{Matrix4, Point3, Vector3};

#[derive(Debug, Clone)]
pub struct Tri {
    pub vertices: (usize, usize, usize), // Indices into the vertex array
    pub color: Color,                    // Optional, or each face has a single color
    pub normal: Vector3<f64>,            // The surface normal for the triangle
}

impl Tri {
    pub fn calculate_normal(&mut self, v1: &Point3<f64>, v2: &Point3<f64>, v3: &Point3<f64>) {
        let v1 = v1.coords; // Get Vector3 representation of the point for math operations
        let v2 = v2.coords;
        let v3 = v3.coords;
        
        // Calculate normal using the vectors derived from Point3 coordinates
        self.normal = (v2 - v1).cross(&(v3 - v1)).normalize();
    }

    pub fn transform_normal(&mut self, transform: &Matrix4<f64>) {
        let rotation_scale_matrix = transform.fixed_view::<3, 3>(0, 0);
        self.normal = (rotation_scale_matrix * self.normal).normalize();
    }
}

