pub mod player;
pub mod ship;

use crate::core::{Scene, Camera};
use glam::Vec3;
use ship::Ship;

pub struct GameState {
    pub ship: Ship,
    pub scene: Scene,
    pub chase_camera: bool,
    pub camera_distance: f32,
    pub camera_height: f32,
}

impl GameState {
    pub fn new() -> Self {
        let camera = Camera::new(
            Vec3::new(0.0, 5.0, -10.0),
            Vec3::ZERO,
            16.0 / 9.0,
        );

        let mut scene = Scene::new(camera);
        // Load ship model TODO: come up with either a ship model, or just a crosshair
        //scene.add_entity(Entity::from_obj("assets/models/ship.obj"));

        Self {
            ship: Ship::new(),
            scene,
            chase_camera: true,
            camera_distance: 10.0,
            camera_height: 3.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.ship.update(delta_time);
        
        // Update ship entity transform
        if let Some(ship_entity) = self.scene.entities.get_mut(0) {
            ship_entity.transform = self.ship.get_transform();
        }

        // Update chase camera if enabled
    }
}


