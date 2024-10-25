use nalgebra::Transform;

use crate::core::entity::Entity;

use super::transform;

pub struct Scene {
    pub entities: Vec<Entity>,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Self {
        //let mut cube = Entity::create_cube();
        let mut cube2 = Entity::create_cube();
        //let mut dodec = Entity::create_dodecahedron();

        //cube2.transform = transform::Transform::new();
        cube2.transform.translate(4.0, 0., 3.);
        Scene {
            entities: vec![cube2],
        }
    }

}
