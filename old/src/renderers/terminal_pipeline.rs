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

impl Pixel {
    pub fn new(ch: char, color: Color) -> Self {
        Pixel { ch, color }
    }

    /// this char will be primarily used for the general rendering mode
    pub fn new_full(color: Color) -> Self {
        Pixel { ch: '█', color }
    }

    pub fn reset(&mut self) {
        self.ch = ' ';
        self.color = Color::WHITE;
    }
}


struct RenderChunk {
    offset: Vec2, //Ideally we just do an x offset, and have tall chunks, I don't feel like working
    //with like a grid, maybe a grid is better, lets ask chatgpt
    //
    local_buffer: Buffer,
}

