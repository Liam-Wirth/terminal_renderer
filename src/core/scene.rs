use std::{cell::RefCell, default, fmt::{self, Display, Formatter}, sync::{Arc, Mutex}};

use crate::core::camera::Camera;

use glam::{Affine3A, Vec3};

use crate::geometry::{Mesh, Tri, Vertex};

use super::Color;

#[derive(Clone, Debug, Copy)]
pub enum RenderMode {
    Solid,
    Wireframe,
    FixedPoint
}

impl Display for RenderMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RenderMode::Wireframe => write!(f, "Wireframe"),
            RenderMode::FixedPoint => write!(f, "Fixed Point"),
            RenderMode::Solid => write!(f, "Standard"),
        }
    }
}

impl Default for RenderMode {
    fn default() -> Self {
        Self::Solid
    }
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub name: String,
    pub mesh: Mesh,
    pub transform: glam::Affine3A,
    render_mode: Arc<Mutex<RenderMode>>,
}

#[derive(Clone)]
// FIX: Doesnt work !!!!!!!!!!
pub struct Environment {
    pub background: Background,
    // TODO: lighting
}

#[derive(Clone)]
pub enum Background {
    Void, // Nothing
    BlenderFloor {
        size: i32,
        cell_size: f32,

        primary_color: Color,
        secondary_color: Color,
    },
    Room {
        size: i32,
        cell_size: f32,

        wall_colors: [Color; 4],
    },
}

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Entity: {:?}", self.name)
    }
}

impl Entity {
    pub fn new(mesh: Mesh, transform: Affine3A, name: String) -> Self {
        Self {
            mesh,
            transform,
            render_mode: Arc::new(Mutex::new(RenderMode::Solid)),
            name
        }
    }

    pub fn from_obj(path: &str) -> Self {
        let mesh: Mesh = Mesh::from_obj(path);
        // make name be last part of path
        let name = path.split("/").last().unwrap().to_string();

        Self {
            mesh,
            transform: Affine3A::IDENTITY,
            render_mode: Arc::new(Mutex::new(RenderMode::Solid)),
            name
        }
    }

    pub fn from_obj_with_transform(path: &str, transform: Affine3A) -> Self {
        let mesh: Mesh = Mesh::from_obj(path);
        let name = path.split("/").last().unwrap().to_string();
        Self {
            mesh,
            transform,
            render_mode: Arc::new(Mutex::new(RenderMode::Solid)),
            name
        }
    }

    pub fn from_obj_with_scale(path: &str, scale: f32) -> Self {
        let mesh: Mesh = Mesh::from_obj(path);
        let transform: Affine3A = Affine3A::from_scale(glam::Vec3::splat(scale));
        let name = path.split("/").last().unwrap().to_string();
        Self {
            mesh,
            transform,
            render_mode: Arc::new(Mutex::new(RenderMode::Solid)),
            name
        }
    }

   pub fn set_render_mode(&self, mode: RenderMode) {
        if let Ok(mut current_mode) = self.render_mode.lock() {
            *current_mode = mode;
        }
    }

    pub fn render_mode(&self) -> Arc<Mutex<RenderMode>> {
        Arc::clone(&self.render_mode)
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

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn spin(&mut self, entity: usize) {
        self.entities[entity].transform *= glam::Affine3A::from_rotation_y(0.03);
        self.entities[entity].transform *= glam::Affine3A::from_rotation_x(0.01);
        self.entities[entity].transform *= glam::Affine3A::from_rotation_z(0.01);
    }
}

impl Default for Scene {
    fn default() -> Self {
        let cam: Camera = Camera::new(
            Vec3::new(0.0, 2.4, -6.0),
            Vec3::new(0.0, 0.0, 1.0),
            800.0 / 600.0,
        );
        Self {
            camera: cam,
            entities: Vec::new(),
        }
    }
}
