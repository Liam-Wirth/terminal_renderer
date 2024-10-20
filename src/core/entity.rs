use crate::core::mesh::Mesh;
use crate::core::transform::Transform;

pub struct Entity {
    pub mesh: Mesh,
    pub transform: Transform,
}

impl Entity {
    pub fn new(mesh: Mesh, transform: Transform) -> Self {
        Entity {mesh, transform}
    }

    pub fn create_cube() -> Self {
        let mesh = Mesh::create_cube();
        let transform = Transform::new();
        Entity::new(mesh, transform)
    }

}
