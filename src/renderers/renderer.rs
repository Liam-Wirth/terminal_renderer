
use std::sync::Mutex;

use glam::Vec2;
// Enum for rendering modes
use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Wireframe,
    Solid,
}

// Global mutable variable to store the current render mode
lazy_static! {
    pub static ref RENDER_MODE: Mutex<RenderMode> = Mutex::new(RenderMode::Wireframe);
    pub static ref COLOR_TRIS: Mutex<bool> = Mutex::new(false);
}

pub fn toggle_color_tris() {
    let mut color_tris = COLOR_TRIS.lock().unwrap();
    *color_tris = !*color_tris;
}
pub fn get_color_tris() -> bool {
    *COLOR_TRIS.lock().unwrap()
}
pub fn set_render_mode(mode: RenderMode) {
    let mut render_mode = RENDER_MODE.lock().unwrap();
    *render_mode = mode;
}

pub fn get_render_mode() -> RenderMode {
    *RENDER_MODE.lock().unwrap()
}

pub fn cycle_render_mode() {
    let mut render_mode = RENDER_MODE.lock().unwrap();
    *render_mode = match *render_mode {
        RenderMode::Wireframe => RenderMode::Solid,
        RenderMode::Solid => RenderMode::Wireframe,
    };
}

pub trait Renderer {
    type PixelType;
    fn initialize(&mut self, width: usize, height: usize);
    fn render_frame(&mut self) -> Vec<Self::PixelType>;
    fn set_resolution(&mut self, width: usize, height: usize);
}


#[derive(Debug, Clone, Copy)]
pub struct ProjectedVertex {
    pub position: Vec2,
    pub depth: f32,
}

impl ProjectedVertex {
    pub fn new(position: Vec2, depth: f32) -> Self {
        ProjectedVertex { position, depth }
    }
}
