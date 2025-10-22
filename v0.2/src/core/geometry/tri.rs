use crate::core::Vert;
use glam::{Mat4, Vec3};
use std::cell::RefCell;

#[derive(Clone, Debug, Default)]
pub struct Tri {
    pub indices: [u32; 3],
    pub norm: Vec3,
    pub centroid: RefCell<Vec3>,
    pub bounds: (Vec3, Vec3),
    // TODO: Fix this VVV
    pub dirty: bool,
    pub visible: RefCell<bool>,
    pub material_id: Option<usize>,
}

impl Tri {
    pub fn new(indices: [u32; 3], vert_buf: &[Vert]) -> Self {
        let v0 = vert_buf[indices[0] as usize].pos;
        let v1 = vert_buf[indices[1] as usize].pos;
        let v2 = vert_buf[indices[2] as usize].pos;

        // Right-hand rule: counter-clockwise winding for front faces
        // (v1 - v0) × (v2 - v0) points outward
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let norm = edge1.cross(edge2).normalize();

        Self {
            indices,
            norm,
            centroid: ((v0 + v1 + v2) / 3.0).into(),
            bounds: (v0.min(v1).min(v2), v0.max(v1).max(v2)),
            dirty: false,
            visible: true.into(),
            ..Default::default()
        }
    }

    pub fn new_with_meshID(indices: [u32; 3], vert_buf: &[Vert], mat_id: usize) -> Self {
        let mut tmp = Self::new(indices, vert_buf);
        tmp.material_id = Some(mat_id);
        tmp
    }
    pub fn is_facing_cam(&self, world_pos: Vec3, view_pos: Vec3) -> bool {
        let view = (world_pos - view_pos).normalize();
        self.norm.dot(view) < 0.
    }

    pub fn sort_vertices(&mut self, vert_buf: &[Vert]) {
        let mut v0 = vert_buf[self.indices[0] as usize].pos;
        let mut v1 = vert_buf[self.indices[1] as usize].pos;
        let mut v2 = vert_buf[self.indices[2] as usize].pos;
        let mut indices = self.indices;

        if v1.y < v0.y {
            indices.swap(0, 1);
            std::mem::swap(&mut v0, &mut v1);
        }
        if v2.y < v1.y {
            indices.swap(1, 2);
            std::mem::swap(&mut v1, &mut v2);
        }
        if v1.y < v0.y {
            indices.swap(0, 1);
            std::mem::swap(&mut v0, &mut v1);
        }
    }
    pub fn update(&mut self, vert_buf: &[Vert], model_mat: &Mat4) {
        let v0 = vert_buf[self.indices[0] as usize].pos;
        let v1 = vert_buf[self.indices[1] as usize].pos;
        let v2 = vert_buf[self.indices[2] as usize].pos;

        // Transform vertices
        let v0_w = model_mat.transform_point3(v0);
        let v1_w = model_mat.transform_point3(v1);
        let v2_w = model_mat.transform_point3(v2);

        // Calculate edges and normal in model space first
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let model_normal = edge1.cross(edge2).normalize();

        // Transform normal to world space
        let normal_mat = model_mat.transpose().inverse();
        self.norm = normal_mat.transform_vector3(model_normal).normalize();

        *self.centroid.borrow_mut() = (v0_w + v1_w + v2_w) / 3.0;
        self.bounds = (v0_w.min(v1_w).min(v2_w), v0_w.max(v1_w).max(v2_w));
        self.dirty = false;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}
