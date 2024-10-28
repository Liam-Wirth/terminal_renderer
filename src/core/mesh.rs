use super::{face::Face, tri::Tri};
use crate::core::color::Color;
use glam::{Vec3};

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vec3>, // 3D coordinates of vertices
    pub faces: Vec<Face>,           // Faces of the mesh
    pub normals_dirty: bool,        // Will be used later hopefully to implement lazy rendering of
                                    // normals
}

impl Mesh {
    pub fn new(vertices: Vec<Vec3>, faces: Vec<Face>) -> Self {
        let mut out = Mesh { vertices, faces, normals_dirty: true};
        out.check_and_fix_winding_order();
        out
    }


    // NOTE: Might be better to move these "primitive"/hardcoded shapes/models elsewhere in the
    // code?
    pub fn create_cube() -> Self {
        let vertices = vec![
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
        ];

        #[rustfmt::skip]
        let faces = vec![
            Face::new(vec![
                Tri { vertices: (0, 1, 2), color: Color::BLUE,      normal: Vec3::ZERO},
                Tri { vertices: (2, 3, 0), color: Color::DARK_BLUE, normal: Vec3::ZERO},
            ]),
            Face::new(vec![
                Tri { vertices: (4, 5, 6), color: Color::GREEN, normal: Vec3::ZERO},
                Tri { vertices: (6, 7, 4), color: Color::PASTEL_GREEN, normal: Vec3::ZERO},
            ]),
            Face::new(vec![
                Tri { vertices: (0, 1, 5), color: Color::RED, normal: Vec3::ZERO},
                Tri { vertices: (5, 4, 0), color: Color::PASTEL_PINK, normal: Vec3::ZERO},
            ]),
            Face::new(vec![
                Tri { vertices: (2, 3, 7), color: Color::YELLOW, normal: Vec3::ZERO},
                Tri { vertices: (7, 6, 2), color: Color::CORAL, normal: Vec3::ZERO},
            ]),
            Face::new(vec![
                Tri { vertices: (0, 3, 7), color: Color::MAGENTA, normal: Vec3::ZERO},
                Tri { vertices: (7, 4, 0), color: Color::CRIMSON, normal: Vec3::ZERO},
            ]),
            Face::new(vec![
                Tri { vertices: (1, 2, 6), color: Color::CYAN, normal: Vec3::ZERO},
                Tri { vertices: (6, 5, 1), color: Color::DARK_CYAN, normal: Vec3::ZERO},
            ]),
        ];

        let mut out = Mesh::new(vertices, faces);
        out.mark_normals_dirty(); // Mark normals as dirty initially
        out.update_normals();
        out
    }

    pub fn update_normals(&mut self) {
        if !self.normals_dirty {
            return; // Skip if normals are not marked as dirty
        }

        // Loop through each face in the mesh
        for face in &mut self.faces {
            for tri in &mut face.tris {
                let v1 = self.vertices[tri.vertices.0];
                let v2 = self.vertices[tri.vertices.1];
                let v3 = self.vertices[tri.vertices.2];

                // Calculate the normal for each triangle
                tri.calculate_normal(&v1, &v2, &v3);
            }

            // Recalculate the face normal (average of all triangle normals)
            face.normal = face
                .tris
                .iter()
                .fold(Vec3::ZERO, |acc, tri| acc + tri.normal)
                .normalize();
        }

        self.normals_dirty = false; // Reset the flag once normals are updated
    }

    // Mark normals as dirty when transformations or changes occur
    pub fn mark_normals_dirty(&mut self) {
        self.normals_dirty = true;
    }

    pub fn create_dodecahedron() -> Self {
        let a = (1.0 + 5.0_f32.sqrt()) / 2.0; // Golden Ratio

//        let edges = vec![
//            (0, 8), (0, 12), (0, 16),
//            (1, 9), (1, 12), (1, 17),
//            (2, 10), (2, 13), (2, 16),
//            (3, 11), (3, 13), (3, 17),
//            (4, 8), (4, 14), (4, 18),
//            (5, 9), (5, 14), (5, 19),
//            (6, 10), (6, 15), (6, 18),
//            (7, 11), (7, 15), (7, 19),
//            (8, 9), (10, 11), (12, 13),
//            (14, 15), (16, 17), (18, 19),
//        ]

        #[rustfmt::skip]
            let vertices = vec![
                // Permutations of (+-1, +-1, +-1)
                Vec3::new(1.0, 1.0, 1.0), Vec3::new(1.0, 1.0, -1.0),
                Vec3::new(1.0, -1.0, 1.0), Vec3::new(1.0, -1.0, -1.0),
                Vec3::new(-1.0, 1.0, 1.0), Vec3::new(-1.0, 1.0, -1.0),
                Vec3::new(-1.0, -1.0, 1.0), Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(0.0, a, 1.0 / a), Vec3::new(0.0, a, -1.0 / a),
                Vec3::new(0.0, -a, 1.0 / a), Vec3::new(0.0, -a, -1.0 / a),

                Vec3::new(1.0 / a, 0.0, a), Vec3::new(1.0 / a, 0.0, -a),
                Vec3::new(-1.0 / a, 0.0, a), Vec3::new(-1.0 / a, 0.0, -a),

                Vec3::new(a, 1.0 / a, 0.0), Vec3::new(a, -1.0 / a, 0.0),
                Vec3::new(-a, 1.0 / a, 0.0), Vec3::new(-a, -1.0 / a, 0.0),
            ];

        #[rustfmt::skip]
        let faces = vec![
            // 17 Cluster (Just three pentagonal faces that all rotate around one common indices)
            Face::new_with_color(
                vec![
                    Tri{vertices: (17, 12, 2), color: Color::RED, normal: Vec3::ZERO},
                    Tri{vertices: (17, 16, 0), color: Color::GREEN, normal: Vec3::ZERO},
                    Tri{vertices: (17, 12, 0), color: Color::BLUE, normal: Vec3::ZERO},
                ],
                Color::RED,
            ),
            Face::new_with_color(
                vec![
                    Tri {vertices: (17, 2, 10), color: Color::RED, normal: Vec3::ZERO},
                    Tri {vertices: (17, 3, 11), color: Color::GREEN, normal: Vec3::ZERO},
                    Tri {vertices: (17, 10, 11), color: Color::BLUE, normal: Vec3::ZERO},
                ],
                Color::GREEN,
            ),
            Face::new_with_color(
                vec![
                    Tri {vertices: (17, 13, 3), color: Color::RED, normal: Vec3::ZERO},
                    Tri {vertices: (17, 16, 1), color: Color::GREEN, normal: Vec3::ZERO},
                    Tri {vertices: (17, 13, 1), color: Color::BLUE, normal: Vec3::ZERO},
                ],
                Color::PURPLE,
            ),
            // 15 "Cluster" (Just three pentagonal faces that all rotate around one common indices)
            Face::new_with_color(
                vec![
                    Tri {vertices: (15, 1, 13), color: Color::RED, normal: Vec3::ZERO},
                    Tri {vertices: (15, 5, 9), color: Color::GREEN, normal: Vec3::ZERO},
                    Tri {vertices: (15, 1, 9), color: Color::BLUE, normal: Vec3::ZERO},
                ],
                Color::YELLOW,
            ),
            Face::new_with_color(
                vec![
                    Tri {vertices: (15, 13, 3), color: Color::RED, normal: Vec3::ZERO},
                    Tri {vertices: (15, 11, 3), color: Color::GREEN, normal: Vec3::ZERO},
                    Tri {vertices: (15, 11, 7), color: Color::BLUE, normal: Vec3::ZERO},
                ],
                Color::ORANGE,
            ),
            Face::new_with_color(
                vec![
                    Tri {vertices: (15, 5, 18), color: Color::RED, normal: Vec3::ZERO},
                    Tri {vertices: (15, 19, 18), color: Color::GREEN, normal: Vec3::ZERO},
                    Tri {vertices: (15, 19, 7), color: Color::BLUE, normal: Vec3::ZERO},
                ],
                Color::MAGENTA,
            ),
            // 6 "Cluster" (Just three pentagonal faces that all rotate around one common indices)
            Face::new_with_color( 
                vec![ 
                    Tri {vertices: (6, 4, 14), color: Color::RED, normal: Vec3::ZERO},
                    Tri {vertices: (6, 18, 19), color: Color::GREEN, normal: Vec3::ZERO},
                    Tri {vertices: (6, 4, 18), color: Color::BLUE, normal: Vec3::ZERO},
                ],
                Color::PASTEL_BLUE,
            ),
            Face::new_with_color(
                vec![
                    Tri {vertices: (6, 12, 14), color: Color::RED, normal: Vec3::ZERO},
                    Tri {vertices: (6, 2, 12), color: Color::GREEN, normal: Vec3::ZERO},
                    Tri {vertices: (6, 2, 10), color: Color::BLUE, normal: Vec3::ZERO},
                ],
                Color::BRIGHT_RED,
            ),
            Face::new_with_color(
                vec![
                    Tri {vertices: (6, 7, 19), color: Color::RED, normal: Vec3::ZERO},
                    Tri {vertices: (6, 11, 10), color: Color::GREEN, normal: Vec3::ZERO},
                    Tri {vertices: (6, 11, 7), color: Color::BLUE, normal: Vec3::ZERO},
                ],
                Color::MINT,
            ),
            // 8 "Cluster" (Just three pentagonal faces that all rotate around one common indices)
            Face::new_with_color(
                vec![
                    Tri {vertices: (8, 0, 16), color: Color::RED, normal: Vec3::ZERO},
                    Tri {vertices: (8, 9, 1), color: Color::GREEN, normal: Vec3::ZERO},
                    Tri {vertices: (8, 16, 1), color: Color::BLUE, normal: Vec3::ZERO},
                ],
                Color::ORANGE,
            ),
            Face::new_with_color(
                vec![
                    Tri {vertices: (8, 5, 9), color: Color::RED, normal: Vec3::ZERO},
                    Tri {vertices: (8, 18, 4), color: Color::GREEN, normal: Vec3::ZERO},
                    Tri {vertices: (8, 18, 5), color: Color::BLUE, normal: Vec3::ZERO},
                ],
                Color::BLUE,
            ),
            Face::new_with_color(
                vec![
                    Tri {vertices: (8, 14, 4), color: Color::RED, normal: Vec3::ZERO},
                    Tri {vertices: (8, 12, 0), color: Color::GREEN, normal: Vec3::ZERO},
                    Tri {vertices: (8, 14, 12), color: Color::BLUE, normal: Vec3::ZERO},
                ],
                Color::YELLOW,
            ),
        ];
        let mut out = Mesh::new(vertices, faces);
        out.mark_normals_dirty();
        out.update_normals(); // TODO: Make sure this works/ exists
        out.check_and_fix_winding_order();
        out
    }
        // THANK YOU CHATGPT O1 FOR HELPING ME LEARN ABOUT THIS!!! I WOULD NEVER!!! HAVE FIGURED
        // OUT WHY CULLING WASN'T WORKING ON MY OWN
        // (after learning I did a bunch of reading and stack overflow looking and eventually
        // learned how to implement it on my own sans-gpt ofc :) 
        pub fn check_and_fix_winding_order(&mut self) {
        let centroid = self.calculate_centroid();

        for face in &mut self.faces {
            for tri in &mut face.tris {
                let v0 = self.vertices[tri.vertices.0];
                let v1 = self.vertices[tri.vertices.1];
                let v2 = self.vertices[tri.vertices.2];

                let edge1 = v1 - v0;
                let edge2 = v2 - v0;
                let normal = edge1.cross(edge2).normalize();

                let face_center = (v0 + v1 + v2) / 3.0;

                let to_center = face_center - centroid;

                if normal.dot(to_center) < 0.0 {
                    // The normal is pointing inward; fix the winding order by swapping v2 and 3
                    tri.vertices = (tri.vertices.0, tri.vertices.2, tri.vertices.1);

                    // Recalculate the normal after fixing the winding order
                    let edge1 = self.vertices[tri.vertices.1] - self.vertices[tri.vertices.0];
                    let edge2 = self.vertices[tri.vertices.2] - self.vertices[tri.vertices.0];
                    tri.normal = edge1.cross(edge2).normalize();
                } else {
                    // The normal is correct
                    tri.normal = normal;
                }
            }

            // Recalculate face normal as the average of its triangles' normals
            face.normal = face
                .tris
                .iter()
                .fold(Vec3::ZERO, |acc, tri| acc + tri.normal)
                / (face.tris.len() as f32);
            face.normal = face.normal.normalize();
        }
    }
        // Function to calculate the centroid of the mesh
    fn calculate_centroid(&self) -> Vec3{
        let mut centroid = Vec3::ZERO;
        let num_vertices = self.vertices.len() as f32;

        for vertex in &self.vertices {
            centroid += vertex;
        }

        centroid / num_vertices
    }
}

