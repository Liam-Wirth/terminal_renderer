use crate::core::Color;
use glam::{Vec3, Vec4};

#[derive(Clone, Debug)]
pub struct ClipVertex {
    pub position: Vec4,
    pub color: Color,
    // Could add other attributes here like:
    // pub normal: Vec3,
    // pub uv: Vec2,
}

impl ClipVertex {
    pub fn lerp(&self, other: &ClipVertex, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            color: self.color.lerp(&other.color, t),
        }
    }
}

#[derive(Clone)]
pub struct ClipTriangle {
    pub vertices: [ClipVertex; 3],
}

pub struct Clipper {
    backface_culling: bool,
    frustum_planes: [Vec4; 6],
}

impl Clipper {
    pub fn new() -> Self {
        Self {
            backface_culling: true,
            frustum_planes: [Vec4::ZERO; 6],
        }
    }

    pub fn update_frustum_planes(&mut self, planes: &[Vec4; 6]) {
        self.frustum_planes = *planes;
    }

    pub fn set_backface_culling(&mut self, enabled: bool) {
        self.backface_culling = enabled;
    }

    pub fn clip_triangle(&self, triangle: &ClipTriangle) -> Vec<ClipTriangle> {
        if !self.should_process_triangle(triangle) {
            return Vec::new();
        }

        let mut triangles = vec![triangle.clone()];

        // Clip against each frustum plane
        for plane in &self.frustum_planes {
            triangles = self.clip_against_plane(triangles, *plane);
        }

        triangles
    }

    fn should_process_triangle(&self, triangle: &ClipTriangle) -> bool {
        // First check if triangle is degenerate
        if self.is_degenerate(triangle) {
            return false;
        }

        // Then do backface culling if enabled
        if self.backface_culling && !self.is_front_facing(triangle) {
            return false;
        }

        true
    }

    fn is_degenerate(&self, triangle: &ClipTriangle) -> bool {
        // Check if any two vertices are effectively the same point
        let epsilon = 1e-6;
        for i in 0..3 {
            let j = (i + 1) % 3;
            let diff = triangle.vertices[i].position - triangle.vertices[j].position;
            if diff.length_squared() < epsilon {
                return true;
            }
        }
        false
    }

    //fn is_front_facing(&self, triangle: &ClipTriangle) -> bool {
    //    let v0 = triangle.vertices[0].position;
    //    let v1 = triangle.vertices[1].position;
    //    let v2 = triangle.vertices[2].position;
    //
    //    // Project to screen space
    //    let v0 = Vec3::new(v0.x / v0.w, v0.y / v0.w, v0.z / v0.w);
    //    let v1 = Vec3::new(v1.x / v1.w, v1.y / v1.w, v1.z / v1.w);
    //    let v2 = Vec3::new(v2.x / v2.w, v2.y / v2.w, v2.z / v2.w);
    //
    //    // Calculate normal in view space
    //    let edge1 = v1 - v0;
    //    let edge2 = v2 - v0;
    //    let normal = edge1.cross(edge2);
    //
    //    // If z component is negative, triangle is facing camera
    //    normal.z > 0.0
    //}
    //
    fn is_front_facing(&self, tri: &ClipTriangle) -> bool {
        let v0 = tri.vertices[0].position;
        let v1 = tri.vertices[1].position;
        let v2 = tri.vertices[2].position;

        // NDC space instead of screen space cause fuck it
        let v0_ndc = Vec3::new(v0.x / v0.w, v0.y / v0.w, v0.z / v0.w);
        let v1_ndc = Vec3::new(v1.x / v1.w, v1.y / v1.w, v1.z / v1.w);
        let v2_ndc = Vec3::new(v2.x / v2.w, v2.y / v2.w, v2.z / v2.w);

        // calculate the normals
        let edge1 = v1_ndc - v0_ndc;
        let edge2 = v2_ndc - v0_ndc;

        let normal = edge1.cross(edge2);

        normal.z > 0.0 // I hate that this check being backwards was the ONLY reason clipping
        // wasn't working, after I already whent ahead and implemented everything else
    }


    fn clip_against_plane(&self, triangles: Vec<ClipTriangle>, plane: Vec4) -> Vec<ClipTriangle> {
        let mut result = Vec::new();

        for triangle in triangles {
            // Calculate distances to plane for each vertex
            let distances = [
                self.distance_to_plane(&triangle.vertices[0], plane),
                self.distance_to_plane(&triangle.vertices[1], plane),
                self.distance_to_plane(&triangle.vertices[2], plane),
            ];

            // Classify vertices
            let inside = [
                distances[0] >= 0.0,
                distances[1] >= 0.0,
                distances[2] >= 0.0,
            ];
            let inside_count = inside.iter().filter(|&&x| x).count();

            match inside_count {
                0 => continue,              // Triangle is completely outside
                3 => result.push(triangle), // Triangle is completely inside
                1 | 2 => {
                    // Triangle needs clipping
                    let clipped = self.clip_triangle_against_plane(triangle, plane, distances);
                    result.extend(clipped);
                }
                _ => unreachable!(),
            }
        }

        result
    }

    fn distance_to_plane(&self, vertex: &ClipVertex, plane: Vec4) -> f32 {
        let v = vertex.position;
        plane.x * v.x + plane.y * v.y + plane.z * v.z + plane.w * v.w
    }

    //fn clip_triangle_against_plane(
    //    &self,
    //    triangle: ClipTriangle,
    //    plane: Vec4,
    //    distances: [f32; 3],
    //) -> Vec<ClipTriangle> {
    //    let _ = plane;
    //    let mut result = Vec::new();
    //    let mut new_verts = Vec::new();
    //    // For each edge
    //    for i in 0..3 {
    //        let j = (i + 1) % 3;

    //        let v0 = &triangle.vertices[i];
    //        let v1 = &triangle.vertices[j];
    //        let d0 = distances[i];
    //        let d1 = distances[j];

    //        if d0 >= 0.0 {
    //            new_verts.push(v0.clone());
    //        }

    //        // If one vertex is inside and one outside, compute intersection
    //        if (d0 < 0.0) != (d1 < 0.0) {
    //            let t = d0 / (d0 - d1);
    //            new_verts.push(v0.lerp(v1, t));
    //        }
    //    }

    //    // Form new triangles from the clipped polygon
    //    for i in 1..new_verts.len() - 1 {
    //        result.push(ClipTriangle {
    //            vertices: [
    //                new_verts[0].clone(),
    //                new_verts[i].clone(),
    //                new_verts[i + 1].clone(),
    //            ],
    //        });
    //    }

    //    result
    //}

    fn clip_triangle_against_plane(
        &self,
        triangle: ClipTriangle,
        plane: Vec4,
        distances: [f32; 3],
    ) -> Vec<ClipTriangle> {
        let _ = plane;
        let mut result = Vec::new();
        let mut new_verts = Vec::new();
        // For each edge
        for i in 0..3 {
            let j = (i + 1) % 3;

            let v0 = &triangle.vertices[i];
            let v1 = &triangle.vertices[j];
            let d0 = distances[i];
            let d1 = distances[j];

            if d0 >= 0.0 {
                new_verts.push(v0.clone());
            }

            // If one vertex is inside and one outside, compute intersection
            if (d0 < 0.0) != (d1 < 0.0) {
                let t = d0 / (d0 - d1);
                new_verts.push(v0.lerp(v1, t));
            }
        }

        // Form new triangles from the clipped polygon
        for i in 1..new_verts.len() - 1 {
            result.push(ClipTriangle {
                vertices: [
                    new_verts[0].clone(),
                    new_verts[i].clone(),
                    new_verts[i + 1].clone(),
                ],
            });
        }

        result
    }
}
