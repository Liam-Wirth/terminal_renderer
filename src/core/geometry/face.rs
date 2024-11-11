use crate::core::Color;

// TODO: implement the usage of faces with the general logic
use super::Vert;
use glam::{Mat4, Vec3};
use std::cell::RefCell;
#[derive(Clone, Debug)]
pub struct Face {
    /// Original vertices in this face (not triangulated)
    pub vertex_indices: Vec<u32>,
    /// Indices into the mesh's triangle list for triangles that make up this face
    pub tri_indices: Vec<usize>,
    /// Face normal in model space
    pub normal: Vec3,
    /// Face centroid in model space
    pub centroid: RefCell<Vec3>,
    /// Whether this face is currently visible
    pub visible: RefCell<bool>,
    /// Face color
    pub color: crate::core::Color,
}

impl Face {
    pub fn new(vertex_indices: Vec<u32>, vertices: &[Vert]) -> Self {
        let mut centroid = Vec3::ZERO;
        for &idx in &vertex_indices {
            centroid += vertices[idx as usize].pos;
        }
        centroid /= vertex_indices.len() as f32;

        // Calculate face normal using first three vertices (assumes CCW winding)
        let v0 = vertices[vertex_indices[0] as usize].pos;
        let v1 = vertices[vertex_indices[1] as usize].pos;
        let v2 = vertices[vertex_indices[2] as usize].pos;

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2).normalize();

        Self {
            vertex_indices,
            tri_indices: Vec::new(),
            normal,
            centroid: centroid.into(),
            visible: true.into(),
            // TODO: Fix this
            color: Color::GREEN,
        }
    }

    pub fn is_facing_camera(&self, model_mat: &Mat4, view_pos: Vec3) -> bool {
        // Transform normal to world space
        let normal_mat = model_mat.transpose().inverse();
        let world_normal = normal_mat.transform_vector3(self.normal).normalize();

        // Transform centroid to world space
        let world_centroid = model_mat.transform_point3(*self.centroid.borrow());

        // Calculate view direction
        let view_dir = (world_centroid - view_pos).normalize();

        // Face is visible if normal points away from viewer
        world_normal.dot(view_dir) < 0.0
    }

    /// Triangulates this face into triangles
    pub fn triangulate(&mut self) -> Vec<[u32; 3]> {
        let mut triangles = Vec::new();

        // Simple fan triangulation
        for i in 1..self.vertex_indices.len() - 1 {
            triangles.push([
                self.vertex_indices[0],
                self.vertex_indices[i],
                self.vertex_indices[i + 1],
            ]);
        }

        triangles
    }
    pub fn update_visibility(&self, model_mat: &Mat4, cam_pos: Vec3) {
        self.visible
            .replace(self.is_facing_camera(model_mat, cam_pos));
    }
}
