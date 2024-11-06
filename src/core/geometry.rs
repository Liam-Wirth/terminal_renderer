use std::cell::RefCell;

use crate::core::Color;
use crate::core::ProjectedVertex;
use glam::Mat4;
use glam::UVec2;
use glam::Vec3;
use glam::Vec4Swizzles;

use super::Camera;

#[derive(Clone, Copy, Debug)]
pub struct Vert {
    pub pos: Vec3,
    pub norm: Vec3,
    pub color: Color,
}

impl Default for Vert {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            norm: Vec3::ZERO,
            color: Color::WHITE,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Tri {
    pub indices: [u32; 3],
    pub norm: Vec3,
    pub centroid: Vec3,
    pub bounds: (Vec3, Vec3),
    dirty: bool,
}

impl Tri {
    pub fn new(indices: [u32; 3], vert_buf: &[Vert]) -> Self {
        let v0 = vert_buf[indices[0] as usize].pos;
        let v1 = vert_buf[indices[1] as usize].pos;
        let v2 = vert_buf[indices[2] as usize].pos;

        let norm = (v1 - v0).cross(v2 - v0).normalize();
        let centroid = (v0 + v1 + v2) / 3.0;

        Self {
            indices,
            norm,
            centroid,
            bounds: (v0.min(v1).min(v2), v0.max(v1).max(v2)),
            dirty: false,
        }
    }

    pub fn update(&mut self, vert_buf: &[Vert]) {
        if self.dirty {
            let v0 = vert_buf[self.indices[0] as usize].pos;
            let v1 = vert_buf[self.indices[1] as usize].pos;
            let v2 = vert_buf[self.indices[2] as usize].pos;

            self.norm = (v1 - v0).cross(v2 - v0).normalize();
            self.centroid = (v0 + v1 + v2) / 3.0;
            self.bounds = (v0.min(v1).min(v2), v0.max(v1).max(v2));
            self.dirty = false;
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub verts: Vec<Vert>,
    pub projected_verts: RefCell<Vec<ProjectedVertex>>,
    pub indices: Vec<u32>, // Keep the indices
    pub tris: Vec<Tri>,    // And the processed triangles
    norms_dirty: bool,
}

impl Mesh {
    pub fn new(verts: Vec<Vert>, indices: Vec<u32>) -> Self {
        let mut tris = Vec::new();
        for chunk in indices.chunks(3) {
            if let &[i1, i2, i3] = chunk {
                tris.push(Tri::new([i1, i2, i3], &verts));
            }
        }
        Mesh {
            projected_verts: RefCell::new(vec![ProjectedVertex::default(); verts.len()]),
            verts,
            indices,
            tris,
            norms_dirty: false,
        }
    }

    pub fn update_vertex(&mut self, index: usize, new_pos: Vec3) {
        if index < self.verts.len() {
            self.verts[index].pos = new_pos;
            // Mark affected triangles as dirty
            for tri in &mut self.tris {
                if tri.indices.contains(&(index as u32)) {
                    tri.mark_dirty();
                }
            }
            self.norms_dirty = true;
        }
    }

    pub fn update_triangles(&mut self) {
        for tri in &mut self.tris {
            tri.update(&self.verts);
        }
    }
    pub fn calculate_face_normals(&mut self) {
        for tri in &mut self.tris {
            let v0 = self.verts[tri.indices[0] as usize].pos;
            let v1 = self.verts[tri.indices[1] as usize].pos;
            let v2 = self.verts[tri.indices[2] as usize].pos;

            tri.norm = (v1 - v0).cross(v2 - v0).normalize();
        }
    }
    pub fn calculate_vertex_normals(&mut self) {
        if !self.norms_dirty {
            return;
        }

        // Reset all vertex normals
        for vert in &mut self.verts {
            vert.norm = Vec3::ZERO;
        }

        // Add each face's contribution to its vertices
        for tri in &self.tris {
            let face_normal = tri.norm;
            // Each vertex gets a contribution from this face
            for &idx in &tri.indices {
                self.verts[idx as usize].norm += face_normal;
            }
        }

        // Normalize the accumulated normals
        for vert in &mut self.verts {
            if vert.norm != Vec3::ZERO {
                vert.norm = vert.norm.normalize();
            }
        }

        self.norms_dirty = false;
    }
    pub fn create_cube() -> Self {
        let verts = vec![
            // Front face (-Z)
            Vert {
                pos: Vec3::new(-1.0, -1.0, -1.0),
                color: Color::RED,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(1.0, -1.0, -1.0),
                color: Color::RED,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(1.0, 1.0, -1.0),
                color: Color::RED,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(-1.0, 1.0, -1.0),
                color: Color::RED,
                ..Default::default()
            },
            // Back face (+Z)
            Vert {
                pos: Vec3::new(-1.0, -1.0, 1.0),
                color: Color::BLUE,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(1.0, -1.0, 1.0),
                color: Color::BLUE,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(1.0, 1.0, 1.0),
                color: Color::BLUE,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(-1.0, 1.0, 1.0),
                color: Color::BLUE,
                ..Default::default()
            },
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0, // Front (-Z)
            1, 5, 6, 6, 2, 1, // Right (+X)
            7, 6, 5, 5, 4, 7, // Back (+Z)
            4, 0, 3, 3, 7, 4, // Left (-X)
            4, 5, 1, 1, 0, 4, // Bottom (-Y)
            3, 2, 6, 6, 7, 3, // Top (+Y)
        ];

        let mut mesh = Self::new(verts, indices);
        mesh.calculate_face_normals();
        mesh.calculate_vertex_normals();
        mesh
    }

    pub fn create_tri() -> Self {
        let verts = vec![
            Vert {
                pos: Vec3::new(-1.0, -1.0, 0.0),
                color: Color::RED,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(1.0, -1.0, 0.0),
                color: Color::GREEN,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(0.0, 1.0, 0.0),
                color: Color::BLUE,
                ..Default::default()
            },
        ];

        let indices = vec![0, 1, 2];

        let mut mesh = Self::new(verts, indices);
        mesh.calculate_face_normals();
        mesh.calculate_vertex_normals();
        mesh
    }
    pub fn create_tetrahedron() -> Self {
        todo!();

    }

    pub fn create_octahedron() -> Self {
        let verts = vec![
            Vert {
                pos: Vec3::new(0.0, 0.0, -1.0),
                color: Color::RED,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(1.0, 0.0, 0.0),
                color: Color::GREEN,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(0.0, 1.0, 0.0),
                color: Color::BLUE,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(-1.0, 0.0, 0.0),
                color: Color::YELLOW,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(0.0, -1.0, 0.0),
                color: Color::CYAN,
                ..Default::default()
            },
            Vert {
                pos: Vec3::new(0.0, 0.0, 1.0),
                color: Color::MAGENTA,
                ..Default::default()
            },
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0, // Front (-Z)
            0, 3, 4, 4, 1, 0, // Left (-X)
            1, 4, 5, 5, 2, 1, // Back (+Z)
            2, 5, 3, 3, 0, 2, // Right (+X)
            3, 4, 5, 5, 0, 3, // Bottom (-Y)
        ];

        let mut mesh = Self::new(verts, indices);
        mesh.calculate_face_normals();
        mesh.calculate_vertex_normals();
        mesh
    }


    pub fn update_projected_vertices(
        &self,
        model_mat: &Mat4,
        screen_dims: &UVec2,
        camera: &Camera,
    ) {
        let mut projected = self.projected_verts.borrow_mut();
        for (i, vert) in self.verts.iter().enumerate() {
            let world_pos = *model_mat * vert.pos.extend(1.0);
            camera.project_vertex_into(world_pos.xyz(), &screen_dims, &mut projected[i]);
        }
    }
}
