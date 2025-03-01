use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::core::{camera::Camera, scene::Scene};

// TODO: Add a toggle to switch between tri coloring and face coloring
//
// Enum for rendering modes
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
    type MetricsType;

    fn init(&mut self, width: usize, height: usize);
    fn render_frame(&mut self, cam: &Camera, scene: &Scene, metrics: &Self::MetricsType);
    fn update_res(&mut self, width: usize, height: usize);
}
