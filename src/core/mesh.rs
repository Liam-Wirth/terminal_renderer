use crossterm::style::Color;
use nalgebra::Vector3;

// Define primitve types for shapes as meshes/entities that can exists as enum members.
// do it in a separate file and just have some basic polyhedra

// TODO: After trying to hardcode a pentagon, It might be easier to define a Tri struct, and have
// the face struct be a collection of tris with a color.
pub struct Face {
    pub vertices: (usize, usize, usize), // Indices into the vertex array
    pub color: Color,                    // Face color
}

pub struct Mesh {
    pub vertices: Vec<Vector3<f64>>, // 3D coordinates of vertices
    pub edges: Vec<(usize, usize)>,  // Pairs of indices defining edges (for wireframe)
    pub faces: Vec<Face>,            // Faces of the mesh, using the Face struct
}

impl Mesh {
    pub fn new(vertices: Vec<Vector3<f64>>, edges: Vec<(usize, usize)>, faces: Vec<Face>) -> Self {
        Mesh {
            vertices,
            edges,
            faces,
        }
    }

    pub fn create_cube() -> Self {
        #[rustfmt::skip]
        let vertices = vec![
            Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, -1.0, -1.0),
            Vector3::new(1.0, 1.0, -1.0), Vector3::new(-1.0, 1.0, -1.0),
            Vector3::new(-1.0, -1.0, 1.0), Vector3::new(1.0, -1.0, 1.0),
            Vector3::new(1.0, 1.0, 1.0), Vector3::new(-1.0, 1.0, 1.0),
        ];

        #[rustfmt::skip]
        let edges = vec![
            (0, 1), (1, 2), (2, 3), (3, 0), // Back face
            (4, 5), (5, 6), (6, 7), (7, 4), // Front face
            (0, 4), (1, 5), (2, 6), (3, 7), // Connecting edges
        ];

        #[rustfmt::skip]
        let faces = vec![
            Face { vertices: (0, 1, 2), color: Color::Blue, },
            Face { vertices: (2, 3, 0), color: Color::DarkBlue, }, // Back face
            Face { vertices: (4, 5, 6), color: Color::Green, },
            Face { vertices: (6, 7, 4), color: Color::DarkGreen, }, // Front face
            Face { vertices: (0, 1, 5), color: Color::Red, },
            Face { vertices: (5, 4, 0), color: Color::DarkRed, }, // Bottom face
            Face { vertices: (2, 3, 7), color: Color::Cyan, },
            Face { vertices: (7, 6, 2), color: Color::DarkCyan, }, // Top face
            Face { vertices: (0, 3, 7), color: Color::Yellow, },
            Face { vertices: (7, 4, 0), color: Color::DarkYellow, }, // Left face
            Face { vertices: (1, 2, 6), color: Color::Magenta, },
            Face { vertices: (6, 5, 1), color: Color::DarkMagenta, }, // Right face
        ];

        Mesh::new(vertices, edges, faces)
    }

    // FIX: this jawn broken
    pub fn create_dodecahedron() -> Self {
        let a = (1.0 + 5.0_f64.sqrt()) / 2.0; // Golden ratio

        // Generate vertices programmatically
        #[rustfmt::skip]
        let vertices = vec![
            // Permutations of (±1, ±1, ±1)
            Vector3::new(1.0, 1.0, 1.0), Vector3::new(1.0, 1.0, -1.0),
            Vector3::new(1.0, -1.0, 1.0), Vector3::new(1.0, -1.0, -1.0),
            Vector3::new(-1.0, 1.0, 1.0), Vector3::new(-1.0, 1.0, -1.0),
            Vector3::new(-1.0, -1.0, 1.0), Vector3::new(-1.0, -1.0, -1.0),
            Vector3::new(0.0, a, 1.0 / a), Vector3::new(0.0, a, -1.0 / a),
            Vector3::new(0.0, -a, 1.0 / a), Vector3::new(0.0, -a, -1.0 / a),
            // Permutations of (±1/φ, 0, ±φ)
            Vector3::new(1.0 / a, 0.0, a), Vector3::new(1.0 / a, 0.0, -a),
            Vector3::new(-1.0 / a, 0.0, a), Vector3::new(-1.0 / a, 0.0, -a),
            // Permutations of (±φ, ±1/φ, 0)
            Vector3::new(a, 1.0 / a, 0.0), Vector3::new(a, -1.0 / a, 0.0),
            Vector3::new(-a, 1.0 / a, 0.0), Vector3::new(-a, -1.0 / a, 0.0),
        ];

        // Define the edges programmatically
        let edges = vec![
            // Define edges as pairs of vertex indices
            (0, 8), (0, 12), (0, 16),
            (1, 9), (1, 12), (1, 17),
            (2, 10), (2, 13), (2, 16),
            (3, 11), (3, 13), (3, 17),
            (4, 8), (4, 14), (4, 18),
            (5, 9), (5, 14), (5, 19),
            (6, 10), (6, 15), (6, 18),
            (7, 11), (7, 15), (7, 19),
            (8, 9), (10, 11), (12, 13),
            (14, 15), (16, 17), (18, 19),
        ];

        // Define the faces programmatically (pentagons)
        #[rustfmt::skip]
        let faces = vec![
            // 17 cluster
            Face { vertices: (17, 12, 2), color: Color::Red },
            Face { vertices: (17, 16, 0), color: Color::Red },
            Face { vertices: (17, 12, 0), color: Color::Red },

            Face { vertices: (17, 2, 10), color: Color::White },
            Face { vertices: (17, 3, 11), color: Color::White},
            Face { vertices: (17, 10, 11), color: Color::White },

            Face { vertices: (17, 13, 3), color: Color::Green },
            Face { vertices: (17, 16, 1), color: Color::Green},
            Face { vertices: (17, 13, 1), color: Color::Green},

            //the 15 cluster
            Face { vertices: (15, 1, 13), color: Color::Blue },
            Face { vertices: (15, 5, 9), color: Color::Blue },
            Face { vertices: (15, 1, 9), color: Color::Blue },

            Face { vertices: (15, 13, 3), color: Color::Yellow },
            Face { vertices: (15, 11, 3), color: Color::Yellow },
            Face { vertices: (15, 11, 7), color: Color::Yellow },

            Face { vertices: (15, 5, 18), color: Color::Magenta },
            Face { vertices: (15, 19, 18), color: Color::Magenta },
            Face { vertices: (15, 19, 7), color: Color::Magenta },

            // the 6 cluster
            Face { vertices: (6, 14, 4), color: Color::Cyan },
            Face { vertices: (6, 18, 19), color: Color::Cyan },
            Face { vertices: (6, 4, 18), color: Color::Cyan },

            Face { vertices: (6, 14, 12), color: Color::DarkRed },
            Face { vertices: (6, 10, 2), color: Color::DarkRed },
            Face { vertices: (6, 12, 2), color: Color::DarkRed },
            
            Face { vertices: (6, 7, 19), color: Color::Green },
            Face { vertices: (6, 10, 11), color: Color::Green },
            Face { vertices: (6, 11, 7), color: Color::Green },

            // the 8 cluster
            Face { vertices: (8, 0, 16), color: Color::DarkBlue },
            Face { vertices: (8, 9, 1), color: Color::DarkBlue },
            Face { vertices: (8, 16, 1), color: Color::DarkBlue},

            Face { vertices: (8, 5, 9), color: Color::Grey},
            Face { vertices: (8, 18, 4), color: Color::Grey },
            Face { vertices: (8, 18, 5), color: Color::Grey },

            Face { vertices: (8, 14, 4), color: Color::DarkYellow },
            Face { vertices: (8, 12, 0), color: Color::DarkYellow },
            Face { vertices: (8, 14, 12), color: Color::DarkYellow },




        ];

        Mesh::new(vertices, edges, faces)
    }
}
