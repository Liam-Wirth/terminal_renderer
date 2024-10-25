use crossterm::{
    cursor::MoveTo,
    style::{Color, Print, SetForegroundColor},
    terminal, QueueableCommand,
};
use nalgebra::{Vector2, Vector3, Vector4};
use std::io::{stdout, Write};

use crate::core::{camera::Camera, entity, scene::Scene};

use super::renderer::{get_render_mode, RenderMode};

#[derive(Clone, Copy)]
pub struct Pixel {
    pub ch: char,
    pub color: Color, // foreground color
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            ch: ' ',
            color: Color::White,
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
        self.color = Color::White;
    }
}

pub struct Buffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Pixel>,
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Buffer {
            width,
            height,
            data: vec![Pixel::default(); width * height], // Fill buffer with spaces initially
        }
    }

    pub fn clear(&mut self) {
        for pixel in &mut self.data {
            pixel.reset();
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, ch: char, color: Color) {
        if x < self.width && y < self.height {
            self.data[x + y * self.width] = Pixel::new(ch, color);
        }
    }

    pub fn render_to_terminal(&self) -> std::io::Result<()> {
        todo!();
    }
}

pub fn render_scene<W: Write>(
    stdout: &mut W,
    scene: &Scene,
    camera: &Camera,
) -> std::io::Result<()> {
    todo!();
    // Get terminal size dynamically
    let (width, height) = terminal::size().unwrap();
    let width = width as usize;
    let height = height as usize;

    let mut buffer = Buffer::new(width, height);
    let camera_direction = camera.direction;

    buffer.clear();

    // Get the current render mode (Wireframe or Solid) from the global state
    let render_mode = get_render_mode();

    let view_matrix = camera.get_view_matrix();
}

// Basic Bresenham's Line Drawing Algorithm for drawing wireframe edges
fn draw_line(buffer: &mut Buffer, v0: &Vector2<usize>, v1: &Vector2<usize>, pix: &Pixel) {
    let mut v0: Vector2<isize> = v0.cast();
    let mut v1: Vector2<isize> = v1.cast();

    let dx = (v1.x - v0.x).abs();
    let dy = -(v1.y - v0.y).abs();
    let sx = if v0.x < v1.x { 1 } else { -1 };
    let sy = if v0.y < v1.y { 1 } else { -1 };
    let mut err = dx + dy;
    while v0.x != v1.x || v0.y != v1.y {
        buffer.set_pixel(v0.x as usize, v0.y as usize, pix.ch, pix.color);

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            v0.x += sx;
        }
        if e2 <= dx {
            err += dx;
            v0.y += sy;
        }
    }
}
