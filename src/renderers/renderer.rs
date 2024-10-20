use lazy_static::lazy_static;
use nalgebra::Vector2;
use std::sync::Mutex;

// Define a custom Color type to decouple from crossterm-specific logic
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

// Enum for rendering modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Wireframe,
    Solid,
}

// Global mutable variable to store the current render mode
lazy_static! {
    pub static ref RENDER_MODE: Mutex<RenderMode> = Mutex::new(RenderMode::Wireframe);
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

