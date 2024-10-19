use crate::core::entity::Entity;

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
        let cube = Entity::create_cube();
        Scene {
            entities: vec![cube],
        }
    }
}
