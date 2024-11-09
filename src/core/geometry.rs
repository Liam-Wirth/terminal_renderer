use std::cell::RefCell;
use std::path::Path;

use crate::core::Color;
use crate::core::ProjectedVertex;
use glam::Mat4;
use glam::UVec2;
use glam::Vec3;
use glam::Vec4Swizzles;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use tobj::LoadOptions;

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

#[derive(Clone, Debug)]
pub struct Tri {
    pub indices: [u32; 3],
    pub norm: Vec3,
    pub centroid: RefCell<Vec3>,
    pub bounds: (Vec3, Vec3),
    dirty: bool,
    pub visible: RefCell<bool>,
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
            centroid: centroid.into(),
            bounds: (v0.min(v1).min(v2), v0.max(v1).max(v2)),
            dirty: false,
            visible: true.into(),
        }
    }
    pub fn is_facing_cam(&self, cam_pos: &Vec3) -> bool {
        let view_dir = (*self.centroid.borrow() - *cam_pos).normalize();
        *self.visible.borrow_mut() = view_dir.dot(self.norm) > 0.0;
        *self.visible.borrow()
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
        self.sort_vertices(vert_buf);
        let v0 = vert_buf[self.indices[0] as usize].pos;
        let v1 = vert_buf[self.indices[1] as usize].pos;
        let v2 = vert_buf[self.indices[2] as usize].pos;

        // Transform to world space
        let v0_w = model_mat.transform_point3(v0);
        let v1_w = model_mat.transform_point3(v1);
        let v2_w = model_mat.transform_point3(v2);

        // Recompute normal in world space
        self.norm = (v1_w - v0_w).cross(v2_w - v0_w).normalize();

        // Recompute centroid in world space
        *self.centroid.borrow_mut() = (v0_w + v1_w + v2_w) / 3.0;

        // Update bounds if necessary
        self.bounds = (v0_w.min(v1_w).min(v2_w), v0_w.max(v1_w).max(v2_w));

        self.dirty = false;
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

    pub fn update_triangles(&mut self, model_mat: &Mat4) {
        for tri in &mut self.tris {
            if tri.dirty {
                tri.update(&self.verts, model_mat);
            }
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

    pub fn update_visibility(&mut self, cam_pos: Vec3, model_mat: &Mat4) {
        // First, update triangles with the current model matrix
        self.update_triangles(model_mat);

        // Then, update visibility based on transformed triangles
        self.tris.par_iter_mut().for_each(|tri| {
            tri.is_facing_cam(&cam_pos);
        });
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
    pub fn from_obj(path: &Path) -> Vec<Mesh> {
        // Load the OBJ with triangulation enabled
        let temp = tobj::load_obj(
            path,
            &LoadOptions {
                triangulate: true,
                ..Default::default()
            },
        )
        .expect("FAIL!!! BRUH!!!");

        let (models, _mats) = temp;
        let mut meshes = Vec::new();

        // Iterate over each model to create a Mesh for each
        for model in models {
            let mesh_data = model.mesh;

            // Convert OBJ positions and normals to `Vert`s with positions and colors
            let verts = mesh_data
                .positions
                .chunks(3)
                .enumerate()
                .map(|(i, pos)| {
                    let position = Vec3::new(pos[0], pos[1], pos[2]);
                    let normal = if !mesh_data.normals.is_empty() {
                        // Get normal if it exists
                        let normal_index = mesh_data.normal_indices.get(i).copied().unwrap_or(0);
                        let normal = &mesh_data.normals
                            [(normal_index * 3) as usize..(normal_index * 3 + 3) as usize];
                        Vec3::new(normal[0], normal[1], normal[2]).normalize()
                    } else {
                        Vec3::ZERO // Default normal if not provided
                    };
                    Vert {
                        pos: position,
                        norm: normal,
                        color: Color::WHITE, // Adjust if you want specific colors per model
                    }
                })
                .collect::<Vec<Vert>>();

            // Generate triangles (Tris) based on OBJ indices
            let tris = mesh_data
                .indices
                .chunks(3)
                .map(|idx| {
                    let indices = [idx[0] as u32, idx[1] as u32, idx[2] as u32];
                    Tri::new(indices, &verts)
                })
                .collect::<Vec<Tri>>();

            // Construct the `Mesh` and append it to `meshes`
            let mesh = Mesh::new(verts, mesh_data.indices.clone());
            meshes.push(mesh);
        }

        meshes
    }
}
