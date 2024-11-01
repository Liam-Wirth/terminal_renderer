use super::tri::Tri;
use crate::core::color::Color;
use glam::Vec3;
#[derive(Debug, Clone)]
pub struct Face {
    pub tris: Vec<Tri>,             // Triangles that make up the face
    pub edges: Vec<(usize, usize)>, // Edges of the face
    pub color: Color,               // Optional face color
    pub normal: Vec3,               // Surface normal for the face
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

        // Initialize with zero vector for normal as placeholder
        Self {
            tris,
            edges,
            color: Color::WHITE,
            normal: Vec3::ZERO,
        }
    }

    pub fn new_with_color(tris: Vec<Tri>, color: Color) -> Self {
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

        Self {
            tris,
            edges,
            color,
            normal: Vec3::ZERO,
        }
    }
}

