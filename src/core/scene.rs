use std::any::Any;

use crate::core::camera::Camera;

use glam::Affine3A;

use crate::core::geometry::Mesh;

#[derive(Clone)]
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
        Self {
            mesh,
            transform: Affine3A::IDENTITY,
        }
    }

    pub fn from_obj_with_transform(path: &str, transform: Affine3A) -> Self {
        let mesh = Mesh::from_obj(path);
        Self { mesh, transform }
    }
}

#[derive(Clone)]
pub struct Scene {
    pub camera: Camera,
    pub entities: Vec<Entity>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            entities: Vec::new(),
        }
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
        let mesh = Mesh::new_test_mesh();
        scene.entities.push(Entity::new(mesh, Affine3A::IDENTITY));
        scene
    }

    pub fn spin(&mut self, entity: usize) {
        self.entities[entity].transform *= glam::Affine3A::from_rotation_y(0.03);
        self.entities[entity].transform *= glam::Affine3A::from_rotation_x(0.01);
        self.entities[entity].transform *= glam::Affine3A::from_rotation_z(0.01);
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
