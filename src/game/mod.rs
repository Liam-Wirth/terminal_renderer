// NOTE: Dead code for now, was wanting to try and maybe make like a first person asteroids type
// game but I'd rather fix the renderer first
pub mod player;
pub mod ship;

use crate::core::Scene;
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
        todo!()
    }

    pub fn update(&mut self, delta_time: f32) {
        self.ship.update(delta_time);

        // Update ship entity transform
        if let Some(ship_entity) = self.scene.entities.get_mut(0) {
            ship_entity.set_transform(self.ship.get_transform());
        }

        // Update chase camera if enabled
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
