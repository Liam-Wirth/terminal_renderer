use super::{process, Material};
use crate::core::color::Color;
use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: Vec3,            // Position in model space
    pub uv: Option<Vec2>,     // Optional texture coordinates
    pub color: Option<Color>, // Optional vertex color for debugging/flat shading
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
                ..Default::default()
            },
        )
        .expect("Failed to load OBJ file");

        let mut mesh = Mesh::new();

        // If there are materials in the .mtl, parse them into our Mesh
        if let Ok(mats) = materials_result {
            for mat in &mats {
                let mut m = Material::default();
                m.name = mat.name.clone();
                // Convert [r, g, b] to your Color
                let dif = mat.diffuse.unwrap();
                let (dr, dg, db) = (dif[0], dif[1], dif[2]);
                m.diffuse_color = Color::from_rgba(dr, dg, db, 1.0);
                m.shininess = mat.shininess;

                // If there's a texture path (like "my_texture.png"), store it:
                if let Some(dif_tex) = &mat.diffuse_texture {
                    // m.diffuse_texture = Some(load_texture(dif_tex));
                }

                mesh.materials.push(m);
            }
        }

        // For each model in the OBJ (often there's just one)
        for model in models {
            let mesh_data = model.mesh;

            // positions => each triple => a vertex
            for pos in mesh_data.positions.chunks(3) {
                mesh.vertices.push(Vertex {
                    pos: Vec3::new(pos[0], pos[1], pos[2]),
                    uv: None, // if you parse `mesh_data.texcoords`, put them here
                    color: None,
                });
            }

            // optional normals => if not present, we can compute
            if !mesh_data.normals.is_empty() {
                for norm in mesh_data.normals.chunks(3) {
                    mesh.normals.push(Normal {
                        norm: Vec3::new(norm[0], norm[1], norm[2]).normalize(),
                    });
                }
            } else {
                process::compute_normals(&mut mesh);
            }

            // for each face
            for face in mesh_data.indices.chunks(3) {
                mesh.tris.push(Tri {
                    vertices: [face[0] as usize, face[1] as usize, face[2] as usize],
                    normals: None,
                    material: mesh_data.material_id, // <-- tie to .materials
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
        };
        let v2: Vertex = Vertex {
            pos: Vec3::new(1.0, -1.0, 0.0),
            uv: None,
            color: Color::GREEN.into(),
        };
        let v3: Vertex = Vertex {
            pos: Vec3::new(0.0, 1.0, 0.0),
            uv: None,
            color: Color::BLUE.into(),
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
