use crate::entity::Entity;
use nalgebra::Vector3;

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
        let cube = Entity::new(vec![
            Vector3::new(-1.0, -1.0, -1.0),
            Vector3::new( 1.0, -1.0, -1.0),
            Vector3::new( 1.0,  1.0, -1.0),
            Vector3::new(-1.0,  1.0, -1.0),
            Vector3::new(-1.0, -1.0,  1.0),
            Vector3::new( 1.0, -1.0,  1.0),
            Vector3::new( 1.0,  1.0,  1.0),
            Vector3::new(-1.0,  1.0,  1.0),
        ]);

        Scene {
            entities: vec![cube],
        }
    }
}

