use super::{process, Material};
use crate::core::{colorf32::Colorf32, Color};
use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: Vec3,               // Position in model space
    pub uv: Option<Vec2>,        // Optional texture coordinates
    pub color: Option<Colorf32>, // Optional vertex color for debugging/flat shading
}

#[derive(Debug, Clone, Copy)]
pub struct Normal {
    pub norm: Vec3, // Normal vector
}

#[derive(Debug, Clone)]
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

    pub fn from_obj(path: &str) -> Self {
        let (models, materials_result) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .expect("Failed to load OBJ file");

        let mut mesh = Mesh::new();

        for model in models {
            let mesh_data = model.mesh;

            // Load vertices
            for pos in mesh_data.positions.chunks(3) {
                mesh.vertices.push(Vertex {
                    pos: Vec3::new(pos[0], pos[1], pos[2]),
                    uv: None,
                    color: None, // Apply later based on material
                });
            }

            // Load normals if available
            if !mesh_data.normals.is_empty() {
                for norm in mesh_data.normals.chunks(3) {
                    mesh.normals.push(Normal {
                        norm: Vec3::new(norm[0], norm[1], norm[2]).normalize(),
                    });
                }
            } else {
                super::process::compute_normals(&mut mesh); // Compute normals if not provided
            }

            // Load triangles
            for face in mesh_data.indices.chunks(3) {
                mesh.tris.push(Tri {
                    vertices: [face[0] as usize, face[1] as usize, face[2] as usize],
                    normals: None, // Updated if normal data exists
                    material: mesh_data.material_id,
                });
            }

            // Load materials if available
            if let Ok(ref materials) = materials_result {
                if let Some(mat_id) = mesh_data.material_id {
                    if let Some(material) = materials.get(mat_id) {
                        let mut mat = Material {
                            name: material.name.clone(),
                            diffuse_color: Colorf32::WHITE,
                            diffuse_texture: None,
                            normal_texture: None,
                            specular_texture: None,
                            shininess: material.shininess,
                        };
                        // mesh.materials.push(Material {

                        if let Some(diffuse_col) = material.diffuse {
                            mat.diffuse_color = {
                                Colorf32::from_rgba(
                                    diffuse_col[0],
                                    diffuse_col[1],
                                    diffuse_col[2],
                                    1.0,
                                )
                            }
                        }
                        if let Some(dif_tex) = &material.diffuse_texture {
                            // TODO!: this
                        }

                        mesh.materials.push(mat);
                    }
                }
            }
        }
        mesh
    }

    pub fn new_test_mesh() -> Self {
        let mut mesh = Mesh::new();
        let v1 = Vertex {
            pos: Vec3::new(-1.0, -1.0, 0.0),
            uv: None,
            color: Colorf32::RED.into(),
        };
        let v2: Vertex = Vertex {
            pos: Vec3::new(1.0, -1.0, 0.0),
            uv: None,
            color: Colorf32::GREEN.into(),
        };
        let v3: Vertex = Vertex {
            pos: Vec3::new(0.0, 1.0, 0.0),
            uv: None,
            color: Colorf32::BLUE.into(),
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
}
