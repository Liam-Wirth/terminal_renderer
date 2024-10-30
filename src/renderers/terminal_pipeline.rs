use crate::core::color::Color;
use crate::core::face::Face;
use crate::core::{camera::Camera, camera::ProjectedVertex, scene::Scene};
use crossterm::terminal;
use glam::{Vec2, Vec3};
use std::io::Write;
use std::sync::{Arc, Mutex};

use super::buffer::Buffer;
use super::renderer::{get_render_mode, RenderMode};
use rayon::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Pixel {
    pub ch: char,
    pub color: Color, // foreground color
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            ch: ' ',
            color: Color::WHITE,
        }
    }
}
