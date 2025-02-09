use super::{process, Material};
use crate::core::color::Color;
use glam::{Affine3A, Vec2, Vec3};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: Vec3,               // Position in model space
    pub uv: Option<Vec2>,        // Optional texture coordinates
    pub color: Option<Color>,    // Optional vertex color for debugging/flat shading
    pub tangent: Option<Vec3>,   // Optional tangent vector for normal mapping
    pub bitangent: Option<Vec3>, // Optional bitangent vector for normal mapping
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            uv: None,
            color: None,
            tangent: None,
            bitangent: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tri {
    pub vertices: [usize; 3],    // Indices into the vertex buffer
    pub material: Option<usize>, // Material ID
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,          // Vertex buffer
    pub normals: Arc<Mutex<Vec<Vec3>>>, // Normal buffer
    pub tris: Vec<Tri>,                 // Triangles
    pub materials: Vec<Material>,       // Materials if available
    normals_dirty: Arc<Mutex<bool>>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Arc::new(Mutex::new(Vec::new())),
            tris: Vec::new(),
            materials: Vec::new(),
            normals_dirty: Arc::new(Mutex::new(true)),
        }
    }

    pub fn mark_normals_dirty(&self) {
        *self.normals_dirty.lock().unwrap() = true;
    }

    pub fn update_normals(&self, transform: &Affine3A) {
        let mut dirty = self.normals_dirty.lock().unwrap();
        if *dirty {
            self.recalculate_normals(transform);
            *dirty = false;
        }
    }

    fn recalculate_normals(&self, transform: &Affine3A) {
        // get the normal buffer
        let mut normals = self.normals.lock().unwrap();
        normals.resize(self.vertices.len(), Vec3::ZERO);
        // Calculate the normal transformation matrix
        let normal_matrix = transform.matrix3.inverse().transpose();

        // For each triangle
        for tri in &self.tris {
            // Get transformed vertices
            let v0 = transform.transform_point3(self.vertices[tri.vertices[0]].pos);
            let v1 = transform.transform_point3(self.vertices[tri.vertices[1]].pos);
            let v2 = transform.transform_point3(self.vertices[tri.vertices[2]].pos);

            // Calculate face normal in world space
            let normal = (v1 - v0).cross(v2 - v0).normalize();

            // Transform the normal
            let transformed_normal = (normal_matrix * normal).normalize();

            // Add the normal contribution to each vertex
            for &vertex_idx in &tri.vertices {
                normals[vertex_idx] += transformed_normal;
            }
        }

        // Normalize all vertex normals
        for i in 0..normals.len() {
            if normals[i].length_squared() > 0.0 {
                normals[i] = normals[i].normalize();
            }
        }
    }

    pub fn bake_normals_to_colors(&mut self) {
        self.update_normals(&Affine3A::IDENTITY);

        // Get lock on normals
        let normals = self.normals.lock().unwrap();

        for (i, vertex) in self.vertices.iter_mut().enumerate() {
            let normal = normals[i];
            vertex.color = Some(Color::new(
                (normal.x + 1.0) * 0.5,
                (normal.y + 1.0) * 0.5,
                (normal.z + 1.0) * 0.5,
            ));
        }
    }

    pub fn from_obj(path: &str) -> Self {
        let (models, materials_result) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ignore_points: true,
                ignore_lines: true,
                ..Default::default()
            },
        )
        .expect("Failed to load OBJ file");
        println!("Materials Result: {:?}", materials_result);

        let mut mesh = Mesh::new();

        // Load materials
        let materials = if let Ok(mats) = materials_result {
            mats.into_iter()
                .map(Material::from_tobj)
                .collect::<Vec<_>>()
        } else {
            vec![Material::default()]
        };
        mesh.materials = materials;

        // Process each model
        for model in models {
            let mesh_data = model.mesh;

            // Create vertices
            for (i, pos) in mesh_data.positions.chunks(3).enumerate() {
                let material_id = mesh_data
                    .material_id
                    .filter(|&id| id < mesh.materials.len());
                println!("Creating triangle with material_id: {:?}", material_id);
                let color = mesh_data
                    .material_id
                    .filter(|&id| id < mesh.materials.len())
                    .map(|id| mesh.materials[id].get_base_color());

                let uv = if !mesh_data.texcoords.is_empty() {
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
                    tangent: None,
                    bitangent: None,
                });
            }

            // Load or compute normals
            if !mesh_data.normals.is_empty() {
                let mut normals = mesh.normals.lock().unwrap();
                *normals = mesh_data
                    .normals
                    .chunks(3)
                    .map(|n| Vec3::new(n[0], n[1], n[2]).normalize())
                    .collect();
            } else {
                mesh.mark_normals_dirty();
            }

            // Create triangles
            for indices in mesh_data.indices.chunks(3) {
                mesh.tris.push(Tri {
                    vertices: [
                        indices[0] as usize,
                        indices[1] as usize,
                        indices[2] as usize,
                    ],
                    material: mesh_data
                        .material_id
                        .filter(|&id| id < mesh.materials.len()),
                });
            }
        }

        mesh
    }

    pub fn new_test_mesh() -> Self {
        let mut mesh = Mesh::new();

        // Create vertices
        mesh.vertices = vec![
            Vertex {
                pos: Vec3::new(-1.0, -1.0, 0.0),
                color: Some(Color::RED),
                ..Default::default()
            },
            Vertex {
                pos: Vec3::new(1.0, -1.0, 0.0),
                color: Some(Color::GREEN),
                ..Default::default()
            },
            Vertex {
                pos: Vec3::new(0.0, 1.0, 0.0),
                color: Some(Color::BLUE),
                ..Default::default()
            },
        ];

        // Create triangle
        mesh.tris.push(Tri {
            vertices: [0, 1, 2],
            material: None,
        });

        // Calculate initial normals
        mesh.update_normals(&Affine3A::IDENTITY);

        mesh
    }
}
