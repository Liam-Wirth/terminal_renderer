use crossterm::style::Color;
use nalgebra::Vector3;

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

        let edges = vec![
            (0, 1), (1, 2), (2, 3), (3, 0), // Back face
            (4, 5), (5, 6), (6, 7), (7, 4), // Front face
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7), // Connecting edges
        ];

        let faces = vec![
            Face {
                vertices: (0, 1, 2),
                color: Color::Blue,
            },
            Face {
                vertices: (2, 3, 0),
                color: Color::DarkBlue,
            }, // Back face
            Face {
                vertices: (4, 5, 6),
                color: Color::Green,
            },
            Face {
                vertices: (6, 7, 4),
                color: Color::DarkGreen,
            }, // Front face
            Face {
                vertices: (0, 1, 5),
                color: Color::Red,
            },
            Face {
                vertices: (5, 4, 0),
                color: Color::DarkRed,
            }, // Bottom face
            Face {
                vertices: (2, 3, 7),
                color: Color::Cyan,
            },
            Face {
                vertices: (7, 6, 2),
                color: Color::DarkCyan,
            }, // Top face
            Face {
                vertices: (0, 3, 7),
                color: Color::Yellow,
            },
            Face {
                vertices: (7, 4, 0),
                color: Color::DarkYellow,
            }, // Left face
            Face {
                vertices: (1, 2, 6),
                color: Color::Magenta,
            },
            Face {
                vertices: (6, 5, 1),
                color: Color::DarkMagenta,
            }, // Right face
        ];

        Mesh::new(vertices, edges, faces)
    }
}

