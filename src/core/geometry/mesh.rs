use std::default;

use super::{process, Material};
use crate::core::color::Color;
use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: Vec3,               // Position in model space
    pub uv: Option<Vec2>,        // Optional texture coordinates
    pub color: Option<Color>,    // Optional vertex color for debugging/flat shading
    pub normal: Option<Vec3>,    // Optional normal vector for per-vertex normals
    pub tangent: Option<Vec3>,   // Optional tangent vector for normal mapping
    pub bitangent: Option<Vec3>, // Optional bitangent vector for normal mapping
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            uv: None,
            color: None,
            normal: None,
            tangent: None,
            bitangent: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Normal {
    pub norm: Vec3, // Normal vector
}

#[derive(Debug, Clone, Copy)]
pub struct Tri {
    pub vertices: [usize; 3],        // Indices into the vertex buffer
    pub normals: Option<[usize; 3]>, // Optional per-vertex normals
    pub material: Option<usize>,     // Material ID
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,    // Vertex buffer
    pub normals: Vec<Normal>,     // Normal buffer
    pub tris: Vec<Tri>,           // Triangles
    pub materials: Vec<Material>, // Materials if available
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            tris: Vec::new(),
            materials: Vec::new(),
        }
    }

    pub fn bake_normals_to_colors(&mut self) {
        // First ensure we have normals
        if self.normals.is_empty() {
            process::compute_normals(self);
        }

        // For each vertex, update its color based on its normal
        for (i, vertex) in self.vertices.iter_mut().enumerate() {
            let normal = self.normals[i].norm;
            // Convert normal components from [-1,1] to [0,1] range
            vertex.color = Some(Color::new(
                (normal.x + 1.0) * 0.5,
                (normal.y + 1.0) * 0.5,
                (normal.z + 1.0) * 0.5,
            ));
        }
    }

    pub fn calculate_normals(&mut self) {
        process::compute_normals(self);
    }

    pub fn from_obj(path: &str) -> Self {
        let (models, materials_result) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ignore_points: true, // Not doing point clouds right now
                ignore_lines: true,  // TODO: Support line segments in the future
                ..Default::default()
            },
        )
        .expect("Failed to load OBJ file");

        let mut mesh = Mesh::new();

        // Load materials first by converting each tobj::Material
        // using our Material::from_tobj() function.
        let materials = if let Ok(mats) = materials_result {
            mats.into_iter()
                .map(Material::from_tobj)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        mesh.materials = materials;

        // For each model in the OBJ file...
        for model in models {
            let mesh_data = model.mesh;

            // Create vertices with material colors and UV coordinates (if available)
            for (i, pos) in mesh_data.positions.chunks(3).enumerate() {
                let color = if let Some(material_id) = mesh_data.material_id {
                    // Use the material’s base color (diffuse if available, else ambient)
                    Some(mesh.materials[material_id].get_base_color())
                } else {
                    None
                };

                let uv = if !mesh_data.texcoords.is_empty() {
                    // Each UV is stored as two consecutive floats
                    Some(Vec2::new(
                        mesh_data.texcoords[i * 2],
                        mesh_data.texcoords[i * 2 + 1],
                    ))
                } else {
                    None
                };

                mesh.vertices.push(Vertex {
                    pos: Vec3::new(pos[0], pos[1], pos[2]),
                    uv,
                    color,
                    normal: None,
                    tangent: None,
                    bitangent: None,
                });
            }

            // Process normals (or compute them if they are missing)
            if !mesh_data.normals.is_empty() {
                for norm in mesh_data.normals.chunks(3) {
                    mesh.normals.push(Normal {
                        norm: Vec3::new(norm[0], norm[1], norm[2]).normalize(),
                    });
                }
            } else {
                process::compute_normals(&mut mesh);
            }

            // Create triangles and store the material index from the OBJ
            for indices in mesh_data.indices.chunks(3) {
                mesh.tris.push(Tri {
                    vertices: [
                        indices[0] as usize,
                        indices[1] as usize,
                        indices[2] as usize,
                    ],
                    normals: None,
                    material: mesh_data.material_id,
                });
            }
        }

        mesh
    }

    pub fn new_test_mesh() -> Self {
        let mut mesh = Mesh::new();
        let v1 = Vertex {
            pos: Vec3::new(-1.0, -1.0, 0.0),
            uv: None,
            color: Color::RED.into(),
            ..Default::default()
        };
        let v2: Vertex = Vertex {
            pos: Vec3::new(1.0, -1.0, 0.0),
            uv: None,
            color: Color::GREEN.into(),
            ..Default::default()
        };
        let v3: Vertex = Vertex {
            pos: Vec3::new(0.0, 1.0, 0.0),
            uv: None,
            color: Color::BLUE.into(),
            ..Default::default()
        };

        let tri = Tri {
            vertices: [0, 1, 2],
            normals: None,
            material: None,
        };

        mesh.vertices.push(v1);
        mesh.vertices.push(v2);
        mesh.vertices.push(v3);

        mesh.tris.push(tri);

        process::compute_normals(&mut mesh);

        mesh
    }

    fn triangulate_face(idxs: &[usize]) -> Vec<[usize; 3]> {
        let mut tris = Vec::new();
        if idxs.len() == 3 {
            tris.push([idxs[0], idxs[1], idxs[2]]);
        } else {
            // fan triangulate
            for i in 1..idxs.len() - 1 {
                tris.push([idxs[0], idxs[i], idxs[i + 1]]);
            }
        }
        tris
    }
}
