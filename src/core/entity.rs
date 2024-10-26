use crate::core::mesh::Mesh;
use crate::core::transform::Transform;
use crate::RENDERMODE;

#[derive(Debug, Clone)]
pub struct Entity {
    pub mesh: Mesh,
    pub transform: Transform,
    pub render_mode: RENDERMODE,
}

impl Entity {
    pub fn new(mesh: Mesh, transform: Transform) -> Self {
        Entity {mesh, transform, render_mode: RENDERMODE::default()}
    }

    pub fn create_cube() -> Self {
        let mesh = Mesh::create_cube();
        let transform = Transform::new();
        Entity::new(mesh, transform)
    }
    //TODO:
    pub fn create_dodecahedron() -> Self {
        let mesh = Mesh::create_dodecahedron();
        let transform = Transform::new();
        Entity::new(mesh, transform)
    }

    pub fn cycle_render_mode(&mut self) {
        self.render_mode = match self.render_mode {
            RENDERMODE::Wireframe => RENDERMODE::Solid,
            RENDERMODE::Solid => RENDERMODE::Wireframe,
            _ => RENDERMODE::Wireframe,
        }
    }

}
