use crate::core::camera::Camera;

use glam::{Affine3A, Mat3, Mat4, Quat, Vec3};

use crate::core::geometry::Mesh;
use std::cell::RefCell;

pub struct Entity {
    pub mesh: Mesh,
    pub transform: glam::Affine3A,
}

impl Entity {
    pub fn new(mesh: Mesh, transform: Affine3A) -> Self {
        Self { mesh, transform }
    }

    pub fn from_obj(path: &str) -> Self {
        let mesh = Mesh::from_obj(path);
        Self { mesh, transform: Affine3A::IDENTITY }
    }

    pub fn from_obj_with_transform(path: &str, transform: Affine3A) -> Self {
        let mesh = Mesh::from_obj(path);
        Self { mesh, transform }
    }
}

pub struct Scene {
    pub camera: Camera,
    pub entities: Vec<Entity>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self { camera, entities: Vec::new()}
    }

    pub fn new_with_entities(camera: Camera, entities: Vec<Entity>) -> Self {
        Self { camera, entities }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn new_test_scene(camera: Camera) -> Self {
        let mut scene = Scene::default();
        scene.camera = camera;
        let mut mesh = Mesh::new_test_mesh();
        scene.entities.push(Entity::new(mesh, Affine3A::IDENTITY));
        scene
    }
}

impl Default for Scene {
    fn default() -> Self {
        let cam = Camera::new(
            glam::Vec3::new(0.0, 2.4, -6.0),
            glam::Vec3::new(0.0, 0.0, 1.0),
            800.0 / 600.0,
        );
        Self {
            camera: cam,
            entities: Vec::new(),
        }
    }
}
