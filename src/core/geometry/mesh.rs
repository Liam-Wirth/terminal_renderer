use super::{process, Material};
use crate::core::color::Color;
use crate::debug_print;
use glam::{Affine3A, Vec2, Vec3};
use std::sync::{Arc, Mutex};

use std::collections::HashMap;
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
    pub name: String,
    pub vertices: Vec<Vertex>,          // Vertex buffer
    pub normals: Arc<Mutex<Vec<Vec3>>>, // Normal buffer
    pub tris: Vec<Tri>,                 // Triangles
    pub materials: Vec<Material>,       // Materials if available
    normals_dirty: Arc<Mutex<bool>>,
    welded: Arc<Mutex<bool>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct PositionKey(i32, i32, i32); // For hashing positions and normals, will help with deduplication/welding

impl PositionKey {
    fn from_vec3(pos: Vec3, epsilon: f32) -> Self {
        // Applying an epsilon to the position allows us to more "fuzzily" associate vertices and find ones that are close enough with eachother
        // Multiply by 1/epsilon and round to nearest integer
        // so positions within `epsilon` end up with the same (x,y,z).
        let scale = 1.0 / epsilon;
        let x = (pos.x * scale).round() as i32;
        let y = (pos.y * scale).round() as i32;
        let z = (pos.z * scale).round() as i32;
        PositionKey(x, y, z)
    }
}
impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Arc::new(Mutex::new(Vec::new())),
            tris: Vec::new(),
            materials: Vec::new(),
            normals_dirty: Arc::new(Mutex::new(true)),
            welded: Arc::new(Mutex::new(false)),
            name: String::from(""),
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
            // Get transformed vertices in world space
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

    #[deprecated(
        since = "0.3.5",
        note = "I made up the version number, but at this stage I am implementing materials processing and would need to rewrite this method to bake these normals into a diffuse/overlay for the models actual materials."
    )]
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
                single_index: true, // This gives shared indices, important for normals accross faces
                ..Default::default()
            },
        )
        .expect("Failed to load OBJ file");

        let mut outmesh = Mesh::new();

        // Load materials
        let materials = if let Ok(mats) = materials_result {
            about_mats(mats.clone(), models.clone());
            mats.into_iter()
                .map(Material::from_tobj)
                .collect::<Vec<_>>()
        } else {
            vec![Material::default()]
        };
        outmesh.materials = materials;

        // Process each model
        for model in models {
            let mesh_data = model.mesh;
            outmesh.name = model.name;

            // Create vertices
            for (i, pos) in mesh_data.positions.chunks(3).enumerate() {
                let uv = if !mesh_data.texcoords.is_empty() {
                    Some(Vec2::new(
                        mesh_data.texcoords[i * 2],
                        mesh_data.texcoords[i * 2 + 1],
                    ))
                } else {
                    None
                };
                // tobj does some silly stuff with face reading and I need to agregate stuff together in my rasterizer, so we are going to merge objects whom all have the same name into the same mesh (duh)
                outmesh.vertices.push(Vertex {
                    pos: Vec3::new(pos[0], pos[1], pos[2]),
                    uv,
                    color: None,
                    tangent: None,
                    bitangent: None,
                });
            }

            // Load or compute normals
            if !mesh_data.normals.is_empty() {
                let mut normals = outmesh.normals.lock().unwrap();
                *normals = mesh_data
                    .normals
                    .chunks(3)
                    .map(|n| Vec3::new(n[0], n[1], n[2]).normalize())
                    .collect();
            } else {
                outmesh.mark_normals_dirty();
            }

            // Create triangles with proper material assignment
            for (face_idx, indices) in mesh_data.indices.chunks(3).enumerate() {
                // Get material ID for this face
                let material_id = if let Some(material_ids) = &mesh_data.material_id {
                    Some(*material_ids)
                } else {
                    None
                };

                outmesh.tris.push(Tri {
                    vertices: [
                        indices[0] as usize,
                        indices[1] as usize,
                        indices[2] as usize,
                    ],
                    material: material_id,
                });
            }
        }
        if outmesh.needs_weld(0.0001) {
            println!("Welding the Mesh!!! {:?}", outmesh.vertices.len());
            outmesh.weld_vertices(0.0001);
            println!("After Welding new len is {:?}", outmesh.vertices.len());
        }

        outmesh
    }

    pub fn weld_vertices(&mut self, position_epsilon: f32) -> bool {
        if self.vertices.is_empty() {
            return false;
        }
        let mut new_vertices = Vec::new();
        new_vertices.reserve(self.vertices.len()); // this is helpful for performance I hear (or so a language model says)

        let mut lookup = HashMap::new();
        let mut idx_map = Vec::with_capacity(self.vertices.len());
        idx_map.resize(self.vertices.len(), 0); // fill with zeros

        // old verts in a temp slice
        let old_verts = &self.vertices;
        for (i, v) in old_verts.iter().enumerate() {
            let pos_key = PositionKey::from_vec3(v.pos, position_epsilon);

            if let Some(&existing_index) = lookup.get(&pos_key) {
                // Found an already‐welded vertex that's "the same."
                idx_map[i] = existing_index;
            } else {
                // It's a new unique position. Push into `new_vertices`:
                let new_index = new_vertices.len();
                new_vertices.push(*v); // Copy the vertex
                lookup.insert(pos_key, new_index);
                idx_map[i] = new_index;
            }
        }
        if new_vertices.len() == self.vertices.len() {
            return false; // No welding needed,
        }
        for tri in &mut self.tris {
            tri.vertices[0] = idx_map[tri.vertices[0]];
            tri.vertices[1] = idx_map[tri.vertices[1]];
            tri.vertices[2] = idx_map[tri.vertices[2]];
        }
        self.vertices = new_vertices;
        self.normals.lock().unwrap().clear();
        self.mark_normals_dirty();
        true
    }

    pub fn from_obj_to_set(path: &str) -> HashMap<String, Mesh> {
        let (models, materials_result) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .expect("Failed to load OBJ file");

        let mut meshes: HashMap<String, Mesh> = HashMap::new();

        // Load materials
        let materials = if let Ok(mats) = materials_result {
            mats.into_iter()
                .map(Material::from_tobj)
                .collect::<Vec<_>>()
        } else {
            vec![Material::default()]
        };

        // Process each model
        for model in models {
            let mesh_data = model.mesh;
            let mesh_name = model.name.clone();

            // Get or create the corresponding Mesh
            let mesh = meshes.entry(mesh_name.clone()).or_insert_with(Mesh::new);

            let base_vertex_index = mesh.vertices.len();

            // Create vertices
            for (i, pos) in mesh_data.positions.chunks(3).enumerate() {
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
                    color: None,
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

            // Create triangles with proper material assignment
            for (face_idx, indices) in mesh_data.indices.chunks(3).enumerate() {
                let material_id = if let Some(material_ids) = &mesh_data.material_id {
                    Some(*material_ids)
                } else {
                    None
                };

                mesh.tris.push(Tri {
                    vertices: [
                        base_vertex_index + indices[0] as usize,
                        base_vertex_index + indices[1] as usize,
                        base_vertex_index + indices[2] as usize,
                    ],
                    material: material_id,
                });
            }

            // Store materials
            mesh.materials = materials.clone();
        }

        // Weld vertices in each mesh
        for (name, mesh) in meshes.iter_mut() {
            if mesh.needs_weld(0.0001) {
                println!(
                    "Welding Mesh '{}' with {} vertices",
                    name,
                    mesh.vertices.len()
                );
                mesh.weld_vertices(0.0001);
                println!("After welding, new vertex count: {}", mesh.vertices.len());
            }
        }

        meshes
    }
    pub fn needs_weld(&self, position_epsilon: f32) -> bool {
        if self.vertices.is_empty() {
            return false;
        }
        let mut lookup = HashMap::new();
        for v in &self.vertices {
            let key = PositionKey::from_vec3(v.pos, position_epsilon);
            lookup.insert(key, true);
        }
        // If fewer unique keys than vertex count, there's duplication
        lookup.len() < self.vertices.len()
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

// Debug function copied form the TOBJ Documentation, using this to try and figure out why I'm not getting colors loaded right. Stuff might be getting dumped in the "Unexpected Things" HashMap
fn about_mats(materials: Vec<tobj::Material>, models: Vec<tobj::Model>) {
    // Materials might report a separate loading error if the MTL file wasn't found.
    // If you don't need the materials, you can generate a default here and use that
    // instead.
    println!("# of models: {}", models.len());
    println!("# of materials: {}", materials.len());

    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;

        println!("model[{}].name = \'{}\'", i, m.name);
        println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

        println!(
            "Size of model[{}].face_arities: {}",
            i,
            mesh.face_arities.len()
        );

        let mut next_face = 0;
        for f in 0..mesh.face_arities.len() {
            let end = next_face + mesh.face_arities[f] as usize;
            let face_indices: Vec<_> = mesh.indices[next_face..end].iter().collect();
            println!("    face[{}] = {:?}", f, face_indices);
            next_face = end;
        }

        // Normals and texture coordinates are also loaded, but not printed in this example
        println!("model[{}].vertices: {}", i, mesh.positions.len() / 3);

        assert!(mesh.positions.len() % 3 == 0);
        for v in 0..mesh.positions.len() / 3 {
            println!(
                "    v[{}] = ({}, {}, {})",
                v,
                mesh.positions[3 * v],
                mesh.positions[3 * v + 1],
                mesh.positions[3 * v + 2]
            );
        }
    }

    for (i, m) in materials.iter().enumerate() {
        println!("material[{}].name = \'{}\'", i, m.name);
        if let Some(ambient) = m.ambient {
            println!(
                "    material.Ka = ({}, {}, {})",
                ambient[0], ambient[1], ambient[2]
            );
        }
        if let Some(diffuse) = m.diffuse {
            println!(
                "    material.Kd = ({}, {}, {})",
                diffuse[0], diffuse[1], diffuse[2]
            );
        }
        if let Some(specular) = m.specular {
            println!(
                "    material.Ks = ({}, {}, {})",
                specular[0], specular[1], specular[2]
            );
        }
        if let Some(shininess) = m.shininess {
            println!("    material.Ns = {}", shininess);
        }
        if let Some(dissolve) = m.dissolve {
            println!("    material.d = {}", dissolve);
        }
        if let Some(ambient_texture) = &m.ambient_texture {
            println!("    material.map_Ka = {}", ambient_texture);
        }
        if let Some(diffuse_texture) = &m.diffuse_texture {
            println!("    material.map_Kd = {}", diffuse_texture);
        }
        if let Some(specular_texture) = &m.specular_texture {
            println!("    material.map_Ks = {}", specular_texture);
        }
        if let Some(shininess_texture) = &m.shininess_texture {
            println!("    material.map_Ns = {}", shininess_texture);
        }
        if let Some(normal_texture) = &m.normal_texture {
            println!("    material.map_Bump = {}", normal_texture);
        }
        if let Some(dissolve_texture) = &m.dissolve_texture {
            println!("    material.map_d = {}", dissolve_texture);
        }
        for (k, v) in &m.unknown_param {
            println!("    material.{} = {}", k, v);
        }
    }
}
