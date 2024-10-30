use core::camera::Camera;
use std::sync::Mutex;

use glam::{UVec2, Vec3};
use lazy_static::lazy_static;
use nalgebra::Vector3;
use renderers::cpu_termrenderer::Pixel;

pub mod core {
    pub mod camera;
    pub mod color;
    pub mod entity;
    pub mod face;
    pub mod frustum;
    pub mod mesh;
    pub mod scene;
    pub mod transform;
    pub mod tri;
}

pub mod renderers {
    pub mod buffer;
    pub mod cpu_termrenderer;
    pub mod renderer;
}

const GLOBAL_UP: Vector3<f64> = Vector3::new(0.0, 1.0, 0.0);

// TODO: Decouple or smth from the global render mode
#[derive(Debug, Default, Clone)]
pub enum RENDERMODE {
    #[default]
    Wireframe,
    Solid,
    WireframeTris, // TODO: logic for this
}

// TODO: Put inside of mutex/lazy static for world state
#[derive(Default)]
struct WorldState {
    pub frame_data: Vec<Pixel>, // Kinda dumb but hear me out!!!
    pub crosshair: Option<Pixel>,
    pub fps: u32,
    pub tris: u32,
    pub faces: u32,
    pub camerapos: Vec3,
    pub camerafacing: Vec3,
    pub dimensions: UVec2,
    pub max_dimensions: UVec2,
    last_frame: std::time::Instant,
}

impl WorldState {
    fn default() -> Self {
        WorldState {
            frame_data: Vec::with_capacity(1920*1080), // Setting it to be this large just to
            // avoid/minimize the chance of it getting resized at runtime
            crosshair: None,
            fps: 0,
            tris: 0,
            faces: 0,
            camerapos: Vec3::new(0.0, 0.0, 0.0),
            camerafacing: Vec3::new(0.0, 0.0, 0.0),
            dimensions: UVec2::new(0, 0),
            max_dimensions: UVec2::new(1920, 1080),
        }
    }

    fn init(&mut self, cam: &Camera, tris: u32, faces: u32) {
        self.camerapos = cam.position;
        self.tris = tris;
        self.faces = faces;
        self.camerafacing = cam.direction;
        self.fps =
    }

}

lazy_static! {
    pub static ref WORLDSTATE: Mutex<WorldState> = Mutex::new(WorldState);
}

