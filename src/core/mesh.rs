use crossterm::style::Color;
use nalgebra::Matrix4;
use nalgebra::Vector3;

pub struct Tri {
    pub vertices: (usize, usize, usize), // Indices into the vertex array
    pub color: Color,                    // Optional, or each face has a single color
    pub normal: Vector3<f64>,            // The surface normal for the triangle
}

impl Tri {
    pub fn calculate_normal(&mut self, v1: &Vector3<f64>, v2: &Vector3<f64>, v3: &Vector3<f64>) {
        self.normal = (v2 - v1).cross(&(v3 - v1)).normalize();
    }

    pub fn transform_normal(&mut self, transform: &Matrix4<f64>) {
        let rotation_scale_matrix = transform.fixed_view::<3, 3>(0, 0);
        self.normal = (rotation_scale_matrix * self.normal).normalize();
    }
}

pub struct Face {
    pub tris: Vec<Tri>,             // Triangles that make up the face
    pub edges: Vec<(usize, usize)>, // Edges of the face
    pub color: Color,               // Optional face color
    pub normal: Vector3<f64>,       // Surface normal for the face
}

impl Face {
    pub fn new(tris: Vec<Tri>) -> Self {
        let mut edges = Vec::new();

        // Collect edges from all triangles and avoid duplicates
        for tri in &tris {
            let tri_edges = vec![
                (tri.vertices.0, tri.vertices.1),
                (tri.vertices.1, tri.vertices.2),
                (tri.vertices.2, tri.vertices.0),
            ];

            for edge in tri_edges {
                if !edges.contains(&edge) && !edges.contains(&(edge.1, edge.0)) {
                    edges.push(edge);
                }
            }
        }

        // Calculate the face normal as an average of triangle normals
        let normal = tris
            .iter()
            .fold(Vector3::zeros(), |acc, tri| acc + tri.normal)
            .normalize();

        Self {
            tris,
            edges,
            color: Color::White,
            normal,
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vector3<f64>>, // 3D coordinates of vertices
    pub faces: Vec<Face>,            // Faces of the mesh
}

impl Mesh {
    pub fn new(vertices: Vec<Vector3<f64>>, faces: Vec<Face>) -> Self {
        Mesh { vertices, faces }
    }

    pub fn create_cube() -> Self {
        let vertices = vec![
            Vector3::new(-1.0, -1.0, -1.0),
            Vector3::new(1.0, -1.0, -1.0),
            Vector3::new(1.0, 1.0, -1.0),
            Vector3::new(-1.0, 1.0, -1.0),
            Vector3::new(-1.0, -1.0, 1.0),
            Vector3::new(1.0, -1.0, 1.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector3::new(-1.0, 1.0, 1.0),
        ];

        let faces = vec![
            Face::new(vec![
                Tri {
                    vertices: (0, 1, 2),
                    color: Color::Blue,
                    normal: Vector3::zeros(),
                },
                Tri {
                    vertices: (2, 3, 0),
                    color: Color::DarkBlue,
                    normal: Vector3::zeros(),
                },
            ]),
            Face::new(vec![
                Tri {
                    vertices: (4, 5, 6),
                    color: Color::Green,
                    normal: Vector3::zeros(),
                },
                Tri {
                    vertices: (6, 7, 4),
                    color: Color::DarkGreen,
                    normal: Vector3::zeros(),
                },
            ]),
            Face::new(vec![
                Tri {
                    vertices: (0, 1, 5),
                    color: Color::Red,
                    normal: Vector3::zeros(),
                },
                Tri {
                    vertices: (5, 4, 0),
                    color: Color::DarkRed,
                    normal: Vector3::zeros(),
                },
            ]),
        ];

        let mut out = Mesh::new(vertices, faces);
        out.calculate_normals();
        out
    }
    fn calculate_normals(&mut self) {
        for face in &mut self.faces {
            for tri in &mut face.tris {
                let v1 = self.vertices[tri.vertices.0];
                let v2 = self.vertices[tri.vertices.1];
                let v3 = self.vertices[tri.vertices.2];
                tri.calculate_normal(&v1, &v2, &v3);
            }
        }
    }

    // FIX: this jawn broken
    //    pub fn create_dodecahedron() -> Self {
    //        let a = (1.0 + 5.0_f64.sqrt()) / 2.0; // Golden ratio
    //
    //        // Generate vertices programmatically
    //        #[rustfmt::skip]
    //        let vertices = vec![
    //            // Permutations of (±1, ±1, ±1)
    //            Vector3::new(1.0, 1.0, 1.0), Vector3::new(1.0, 1.0, -1.0),
    //            Vector3::new(1.0, -1.0, 1.0), Vector3::new(1.0, -1.0, -1.0),
    //            Vector3::new(-1.0, 1.0, 1.0), Vector3::new(-1.0, 1.0, -1.0),
    //            Vector3::new(-1.0, -1.0, 1.0), Vector3::new(-1.0, -1.0, -1.0),
    //            Vector3::new(0.0, a, 1.0 / a), Vector3::new(0.0, a, -1.0 / a),
    //            Vector3::new(0.0, -a, 1.0 / a), Vector3::new(0.0, -a, -1.0 / a),
    //            // Permutations of (±1/φ, 0, ±φ)
    //            Vector3::new(1.0 / a, 0.0, a), Vector3::new(1.0 / a, 0.0, -a),
    //            Vector3::new(-1.0 / a, 0.0, a), Vector3::new(-1.0 / a, 0.0, -a),
    //            // Permutations of (±φ, ±1/φ, 0)
    //            Vector3::new(a, 1.0 / a, 0.0), Vector3::new(a, -1.0 / a, 0.0),
    //            Vector3::new(-a, 1.0 / a, 0.0), Vector3::new(-a, -1.0 / a, 0.0),
    //        ];
    //
    //        // Define the edges programmatically
    //        let edges = vec![
    //            // Define edges as pairs of vertex indices
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
    //        ];
    //
    //        // Define the faces programmatically (pentagons)
    //        #[rustfmt::skip]
    //        let faces = vec![
    //            // 17 cluster
    //            Face { vertices: (17, 12, 2), color: Color::Red },
    //            Face { vertices: (17, 16, 0), color: Color::Red },
    //            Face { vertices: (17, 12, 0), color: Color::Red },
    //
    //            Face { vertices: (17, 2, 10), color: Color::White },
    //            Face { vertices: (17, 3, 11), color: Color::White},
    //            Face { vertices: (17, 10, 11), color: Color::White },
    //
    //            Face { vertices: (17, 13, 3), color: Color::Green },
    //            Face { vertices: (17, 16, 1), color: Color::Green},
    //            Face { vertices: (17, 13, 1), color: Color::Green},
    //
    //            //the 15 cluster
    //            Face { vertices: (15, 1, 13), color: Color::Blue },
    //            Face { vertices: (15, 5, 9), color: Color::Blue },
    //            Face { vertices: (15, 1, 9), color: Color::Blue },
    //
    //            Face { vertices: (15, 13, 3), color: Color::Yellow },
    //            Face { vertices: (15, 11, 3), color: Color::Yellow },
    //            Face { vertices: (15, 11, 7), color: Color::Yellow },
    //
    //            Face { vertices: (15, 5, 18), color: Color::Magenta },
    //            Face { vertices: (15, 19, 18), color: Color::Magenta },
    //            Face { vertices: (15, 19, 7), color: Color::Magenta },
    //
    //            // the 6 cluster
    //            Face { vertices: (6, 14, 4), color: Color::Cyan },
    //            Face { vertices: (6, 18, 19), color: Color::Cyan },
    //            Face { vertices: (6, 4, 18), color: Color::Cyan },
    //
    //            Face { vertices: (6, 14, 12), color: Color::DarkRed },
    //            Face { vertices: (6, 10, 2), color: Color::DarkRed },
    //            Face { vertices: (6, 12, 2), color: Color::DarkRed },
    //
    //            Face { vertices: (6, 7, 19), color: Color::Green },
    //            Face { vertices: (6, 10, 11), color: Color::Green },
    //            Face { vertices: (6, 11, 7), color: Color::Green },
    //
    //            // the 8 cluster
    //            Face { vertices: (8, 0, 16), color: Color::DarkBlue },
    //            Face { vertices: (8, 9, 1), color: Color::DarkBlue },
    //            Face { vertices: (8, 16, 1), color: Color::DarkBlue},
    //
    //            Face { vertices: (8, 5, 9), color: Color::Grey},
    //            Face { vertices: (8, 18, 4), color: Color::Grey },
    //            Face { vertices: (8, 18, 5), color: Color::Grey },
    //
    //            Face { vertices: (8, 14, 4), color: Color::DarkYellow },
    //            Face { vertices: (8, 12, 0), color: Color::DarkYellow },
    //            Face { vertices: (8, 14, 12), color: Color::DarkYellow },
    //        ];
    //
    //        Mesh::new(vertices, edges, faces)
    //    }
}
