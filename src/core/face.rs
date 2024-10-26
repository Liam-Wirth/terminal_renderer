use super::tri::Tri;
use crate::core::color::Color;
use nalgebra::{Point3, Vector3};

pub struct Face {
    pub tris: Vec<Tri>,             // Triangles that make up the face
    pub edges: Vec<(usize, usize)>, // Edges of the face
    pub color: Color,               // Optional face color

    pub normal: Vector3<f64>,       // Surface normal for the face cannot be calculated here
                                    // because the face is constructed from triangles whose
                                    // vertices are just indices into an array in a parent struct
                                    // (I was told this was better for some reason?)
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
        //let normal = tris
        //    .iter()
        //    .fold(Vector3::zeros(), |acc, tri| acc + tri.normal)
        //    .normalize();

        Self {
            tris,
            edges,
            color: Color::WHITE,
            normal: Vector3::zeros(),
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
        // Calculate the face normal as an average of triangle normals
        //let normal = tris // not fully sure why I'm doing this when on construction the normals
        //will be wrong/dirty anyways?
        //    .iter()
        //    .fold(Vector3::zeros(), |acc, tri| acc + tri.normal)
        //    .normalize();

        Self {
            tris,
            edges,
            color,
            normal: Vector3::zeros(),
        }
    }
}
