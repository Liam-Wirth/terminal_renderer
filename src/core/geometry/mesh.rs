use crate::core::camera::{Camera, ProjectedVertex};
use crate::core::color::Color;
use crate::core::geometry::{Tri, Vert};
use glam::UVec2;
use glam::Vec4Swizzles;
use glam::{Mat4, Vec3};
use rayon::prelude::*;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::path::Path;
use tobj::LoadOptions;

use super::tri;

// TODO: Material Handling

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub verts: RefCell<Vec<Vert>>,
    pub projected_verts: RefCell<Vec<ProjectedVertex>>,
    pub indices: Vec<u32>,                      // Keep the indices
    pub tris: RefCell<Vec<Tri>>,                // And the processed triangles
    pub materials: Option<Vec<tobj::Material>>, // I'm lazy and am just gonna use the TOBJ material, but still use my personal? mesh struct
    pub face_material_ids: Option<Vec<usize>>,
    norms_dirty: RefCell<bool>,
    transform_dirty: RefCell<bool>,
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
            verts: verts.into(),
            indices,
            tris: tris.into(),
            norms_dirty: false.into(),
            transform_dirty: false.into(),
            ..Default::default()
        }
    }

    pub fn update_vertex(&mut self, index: usize, new_pos: Vec3) {
        let mut verts = self.verts.borrow_mut();
        if index < verts.len() {
            verts[index].pos = new_pos;
            // Mark affected triangles as dirty
            let mut tris = self.tris.borrow_mut();
            for tri in tris.iter_mut() {
                if tri.indices.contains(&(index as u32)) {
                    tri.mark_dirty();
                }
            }
            self.norms_dirty.replace(true);
        }
    }
    pub fn update_dirty(&self, dirty: bool) {
        self.transform_dirty.replace(dirty);
    }

    pub fn update_triangles(&self, model_mat: &Mat4) {
        if *self.transform_dirty.borrow() {
            let mut tris = self.tris.borrow_mut();
            for tri in tris.iter_mut() {
                if tri.dirty {
                    tri.update(&self.verts.borrow(), model_mat);
                }
            }
            self.transform_dirty.replace(false);
        }
    }
    pub fn calculate_face_normals(&self) {
        let mut tris = self.tris.borrow_mut();
        let verts = self.verts.borrow();
        for tri in tris.iter_mut() {
            let v0 = verts[tri.indices[0] as usize].pos;
            let v1 = verts[tri.indices[1] as usize].pos;
            let v2 = verts[tri.indices[2] as usize].pos;

            // Maintain consistent CCW winding
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            tri.norm = edge1.cross(edge2).normalize();
        }
    }

    pub fn calculate_vertex_normals(&self) {
        if !*self.norms_dirty.borrow() {
            return;
        }

        let mut verts = self.verts.borrow_mut();
        let mut tris = self.tris.borrow_mut();
        // Reset all vertex normals
        for vert in verts.iter_mut() {
            vert.norm = Vec3::ZERO;
        }

        // Add each face's contribution to its vertices
        for tri in tris.iter_mut() {
            let face_normal = tri.norm;
            // Each vertex gets a contribution from this face
            for &idx in &tri.indices {
                verts[idx as usize].norm += face_normal;
            }
        }

        // Normalize the accumulated normals
        for vert in verts.iter_mut() {
            if vert.norm != Vec3::ZERO {
                vert.norm = vert.norm.normalize();
            }
        }

        self.norms_dirty.replace(false);
    }

    pub fn update_visibility(&self, cam_pos: Vec3, model_mat: &Mat4) {
        if *self.transform_dirty.borrow() {
            self.update_triangles(model_mat);
            let mut tris = self.tris.borrow_mut();
            let verts = self.verts.borrow();
            for tri in tris.iter_mut() {
                let world = model_mat.transform_point3(verts[tri.indices[0] as usize].pos);
                tri.visible.replace(tri.is_facing_cam(world, cam_pos));
            }
            self.transform_dirty.replace(false);
        }
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

        let mesh = Self::new(verts, indices);
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

        let mesh = Self::new(verts, indices);
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

        let mesh = Self::new(verts, indices);
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

        for (i, vert) in self.verts.borrow().iter().enumerate() {
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
                single_index: true,
                ..Default::default()
            },
        )
        .expect("FAIL!!! BRUH!!!");

        let (models, mats) = temp;

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
    pub fn from_obj_with_materials(obj_path: &Path, mtl_path: &Path) -> Vec<Self> {
        todo!();
    }
}
