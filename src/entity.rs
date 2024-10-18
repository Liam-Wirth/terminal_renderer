use nalgebra::Vector3;

pub struct Entity {
    pub vertices: Vec<Vector3<f64>>,     // Vertices of the 3D object
    pub edges: Vec<(usize, usize)>,       // Pairs of indices into the `vertices` array defining edges
}

impl Entity {
    pub fn new(vertices: Vec<Vector3<f64>>, edges: Vec<(usize, usize)>) -> Self {
        Entity { vertices, edges }
    }

    // Define a simple cube for testing
    pub fn create_cube() -> Self {
        let vertices = vec![
            Vector3::new(-1.0, -1.0, -1.0),
            Vector3::new( 1.0, -1.0, -1.0),
            Vector3::new( 1.0,  1.0, -1.0),
            Vector3::new(-1.0,  1.0, -1.0),
            Vector3::new(-1.0, -1.0,  1.0),
            Vector3::new( 1.0, -1.0,  1.0),
            Vector3::new( 1.0,  1.0,  1.0),
            Vector3::new(-1.0,  1.0,  1.0),
        ];

        let edges = vec![
            (0, 1), (1, 2), (2, 3), (3, 0), // Back face
            (4, 5), (5, 6), (6, 7), (7, 4), // Front face
            (0, 4), (1, 5), (2, 6), (3, 7), // Connecting edges
        ];

        Entity::new(vertices, edges)
    }
}

