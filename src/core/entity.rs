use crate::RENDERMODE;

use super::{geometry::Mesh, transform::Transform};

#[derive(Debug, Clone)]
pub struct Entity {
    pub mesh: Mesh,
    pub transform: Transform,
    pub render_mode: RENDERMODE,
}

impl Entity {
    pub fn new(mesh: Mesh, transform: Transform) -> Self {
        Entity {
            mesh,
            transform,
            render_mode: RENDERMODE::default(),
        }
    }

    pub fn create_cube() -> Self {
        let mesh = Mesh::create_cube();
        let transform = Transform::new();
        Entity::new(mesh, transform)
    }

    pub fn create_tri() -> Self {
        let mesh = Mesh::create_tri();
        let transform = Transform::new();
        Entity::new(mesh, transform)
    }

    pub fn create_octahedron() -> Self {
        let mesh = Mesh::create_octahedron();
        let transform = Transform::new();
        Entity::new(mesh, transform)
    }
}
