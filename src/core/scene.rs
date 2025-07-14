use crate::core::light::Light;
use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::core::camera::Camera;

use glam::{Affine3A, Vec3};

use crate::geometry::Mesh;
use crate::core::TextureManager;

use super::{geometry::Material, Color};

#[derive(Clone, Debug, Copy)]
pub enum RenderMode {
    Solid,
    Wireframe,
    FixedPoint,
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

// TODO: Flesh out this class, need to add the functionality to more accurately/properly collect
// the submodels into one singular mesh, or not, regardless look into that
#[derive(Clone, Debug)]
pub struct Entity {
    pub name: String,
    pub mesh: Mesh,
    transform: glam::Affine3A,
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
            name,
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
            name,
        }
    }

    pub fn from_obj_set(path: &str) -> Vec<Self> {
        let meshes = Mesh::from_obj_to_set(path);
        let mut out = Vec::new();

        for (_, mesh) in meshes {
            out.push(Self {
                name: mesh.name.clone(),
                mesh,
                transform: Affine3A::IDENTITY,
                render_mode: Arc::new(Mutex::new(RenderMode::Solid)),
            })
        }
        out
    }

    pub fn transform(&self) -> &Affine3A {
        &self.transform
    }

    pub fn set_transform(&mut self, transform: Affine3A) {
        self.transform = transform;
        self.mesh.mark_normals_dirty();
    }

    pub fn update(&self) {
        self.mesh.update_normals(&self.transform);
    }

    pub fn from_obj_with_transform(path: &str, transform: Affine3A) -> Self {
        let mesh: Mesh = Mesh::from_obj(path);
        let name = path.split("/").last().unwrap().to_string();
        Self {
            mesh,
            transform,
            render_mode: Arc::new(Mutex::new(RenderMode::Solid)),
            name,
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
            name,
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
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            entities: Vec::new(),
            lights: Vec::new(),
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
    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn create_floor() -> Entity {
        let mut floormesh =  Mesh::create_floor_mesh(
            Vec3::new(-10.0, 0.0, -10.0),
            Vec3::new(10.0, 0.0, -10.0),
            Vec3::new(10.0, 0.0, 10.0),
            Vec3::new(-10.0, 0.0, 10.0),
        );

        let floormat = Material {
            name: "Floor".to_string(),
            diffuse: Some(Color::WHITE),
            shininess: Some(10.0),
            specular: Some(Color::WHITE), // Black Color
            ambient: Some(Color::from_hex("#000000").unwrap()), // Black Color
            ..Default::default()
        };

        floormesh.set_material(floormat);
        floormesh.mark_normals_dirty();

        let transform = Affine3A::from_translation(Vec3::new(0., -3., 0.));
        let mut out = Entity::new(floormesh, transform, "Floor".to_string());
        out.update();
        out
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
            lights: Vec::new(),
        }
    }
}

impl Entity {
    // Adding default intity constructors for some of the files located within the assets folder,
    // will make testing / debugging a bit easier
    //
    pub fn new_icosphere() -> Vec<Self> {
        let icos = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("models")
            .join("icosphere.obj");
        Self::from_obj_set(icos.to_str().unwrap())
    }

    pub fn new_suzanne() -> Vec<Self> {
        let monkey_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("models")
            .join("suzy.obj");
        Self::from_obj_set(monkey_path.to_str().unwrap())
    }

    pub fn new_penguin() -> Vec<Self> {
        let penguin_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("models")
            .join("Penguin.obj");
        
        let mut entities = Self::from_obj_set(penguin_path.to_str().unwrap());
        
        // Load textures for penguin materials
        let mut texture_manager = TextureManager::new();
        
        // Load textures for materials that specify them
        for entity in &mut entities {
            for material in entity.mesh.materials.iter_mut() {
                // Only update texture paths that are relative and make them relative to assets/models/
                if let Some(ref texture_path) = material.diffuse_texture.clone() {
                    // If it's just a filename, make it relative to models directory under assets/
                    if !texture_path.contains("/") && !texture_path.contains("\\") {
                        let relative_path = format!("models/{}", texture_path);
                        material.diffuse_texture = Some(relative_path);
                    }
                }
                
                // Load textures for this material
                material.load_textures(&mut texture_manager);
            }
        }
        
        entities
    }

    pub fn new_teapot() -> Vec<Self> {
        let teapot = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("models")
            .join("newell_teaset")
            .join("teapot.obj");
        Self::from_obj_set(teapot.to_str().unwrap())
    }

    pub fn new_textured_teapot() -> Vec<Self> {
        let teapot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("models")
            .join("teapot textured")
            .join("teapot.obj");
        
        let mut entities = Self::from_obj_set(teapot_path.to_str().unwrap());
        
        // Load textures for teapot materials
        let mut texture_manager = TextureManager::new();
        
        // The teapot texture path
        let texture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("models")
            .join("teapot textured")
            .join("default.png");
        
        // Update each entity's materials to load textures
        for entity in &mut entities {
            for material in &mut entity.mesh.materials {
                // Set the diffuse texture path and load it
                material.diffuse_texture = Some(texture_path.to_string_lossy().to_string());
                material.load_textures(&mut texture_manager);
            }
        }
        
        entities
    }

    pub fn new_skull() -> Vec<Self> {
        let skull = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("models")
            .join("skull.obj");
        Self::from_obj_set(skull.to_str().unwrap())
    }
    pub fn new_ferris() -> Vec<Self> {
        let ferris = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("models")
            .join("ferris.obj");
        Self::from_obj_set(ferris.to_str().unwrap())
    }
    pub fn new_sphere() -> Vec<Self> {
        let sphere = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("models")
            .join("platonics")
            .join("sphere.obj");
        Self::from_obj_set(sphere.to_str().unwrap())
    }
}
