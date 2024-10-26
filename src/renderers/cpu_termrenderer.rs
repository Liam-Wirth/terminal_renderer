use crate::core::color::Color;
use crossterm::style::{Color as crossColor, SetBackgroundColor};
use crossterm::{
    cursor::MoveTo,
    style::{Print, SetForegroundColor},
    terminal, QueueableCommand,
};
use nalgebra::{Point3, Vector2, Vector4};
use std::io::{stdout, Write};

use crate::core::{camera::Camera, entity, scene::Scene};

use super::renderer::{get_render_mode, RenderMode};


// TODO: would be cool to make this more abstract/ abstract a lot of the logic such that
// I can just have alot of these structs exist as the "backend" for whatever type of output
// rendering I end up trying to do (ppm file -> png, ppm gif or smth, I dunno)
#[derive(Clone, Copy)]
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
        let mut stdout = stdout();
        stdout.queue(SetBackgroundColor(crossColor::Black))?;
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = &self.data[x + y * self.width];
                stdout.queue(MoveTo(x as u16, y as u16))?;
                stdout.queue(SetForegroundColor(pixel.color.to_crossterm_color()))?;
                stdout.queue(Print(pixel.ch))?;
            }
        }
        stdout.flush()
    }
}

pub fn render_scene<W: Write>(
    stdout: &mut W,
    scene: &mut Scene,
    camera: &Camera,
) -> std::io::Result<()> {
    // Get terminal size dynamically
    let (width, height) = terminal::size().unwrap();
    let width = width as usize;
    let height = height as usize;

    // Initialize the buffer
    let mut buffer = Buffer::new(width, height);
    buffer.clear();

    let view_matrix = camera.get_view_matrix();
    let render_mode = get_render_mode();

    for entity in &mut scene.entities {
        let transform_matrix = entity.transform.get_matrix();
        if entity.mesh.normals_dirty {
            entity.mesh.update_normals();
        }
        let mut transformed_vertices = Vec::new();

        for vertex in &entity.mesh.vertices {
            let homogenous_vertex = vertex.to_homogeneous(); // Convert Point3 to Point4
            let world_vertex = transform_matrix * homogenous_vertex;
            let view_vertex = view_matrix * world_vertex;
            let projected_point = Point3::from_homogeneous(view_vertex).unwrap();
            transformed_vertices.push(projected_point);
        }

        let mut screen_coords = vec![];
        for vertex in transformed_vertices.iter() {
            let screen_vert = camera.project_vertex(vertex.coords, &width, &height);
            screen_coords.push(screen_vert);
        }

        for face in &entity.mesh.faces {
            for tri in &face.tris {
                let v0 = screen_coords[tri.vertices.0];
                let v1 = screen_coords[tri.vertices.1];
                let v2 = screen_coords[tri.vertices.2];

                if render_mode == RenderMode::Wireframe {
                    // Draw each edge of the triangle in wireframe
                    // TODO: allow toggling  of draw tricolors
                    draw_line(&mut buffer, &v0, &v1, &Pixel::new('#', tri.color));
                    draw_line(&mut buffer, &v1, &v2, &Pixel::new('#', tri.color));
                    draw_line(&mut buffer, &v2, &v0, &Pixel::new('#', tri.color));
                } else if render_mode == RenderMode::Solid {
                    draw_filled_triangle_scanline(&mut buffer, &v0, &v1, &v2, &Pixel::new('#', tri.color));
                    
                }
            }
        }
    }
    // Finally render buffer to terminal
    let _out = buffer.render_to_terminal();
    Ok(())
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

//https://groups.csail.mit.edu/graphics/classes/6.837/F98/Lecture7/triangles.html
fn draw_filled_triangle_scanline(buffer: &mut Buffer, v0: &Vector2<usize>, v1: &Vector2<usize>, v2: &Vector2<usize>, pix: &Pixel) {
    todo!();
}


