use crate::core::color::Color;
use crate::core::face::Face;
use core::f64;
use crossterm::terminal;
use nalgebra::{Point2, Point3, Vector3};
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};

use crate::core::{camera::Camera, scene::Scene};

use super::renderer::{get_render_mode, RenderMode};
use rayon::prelude::*;

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
    pub depth: Vec<f64>,
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Buffer {
            width,
            height,
            data: vec![Pixel::default(); width * height], // Fill buffer with spaces initially
            depth: vec![f64::INFINITY; width * height],   // Fill buffer with spaces initially
        }
    }

    pub fn clear(&mut self) {
        for pixel in &mut self.data {
            pixel.reset();
        }
        for depth in &mut self.depth {
            *depth = f64::INFINITY;
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, ch: char, color: Color) {
        if x < self.width && y < self.height {
            self.data[x + y * self.width] = Pixel::new(ch, color);
        }
    }
    pub fn render_to_terminal(&self) -> std::io::Result<()> {
        let mut stdout = stdout();

        let mut output = String::new();

        // Keep track of the last color to minimize color changes
        let mut last_color = None;

        // Hide the cursor and clear the screen once
        output.push_str("\x1B[?25l"); // Hide cursor
        output.push_str("\x1B[2J"); // Clear screen
        output.push_str("\x1B[H"); // Move cursor to home position

        // For each line
        for y in 0..self.height {
            let mut x = 0;
            // Move cursor to the beginning of the line once
            output.push_str(&format!("\x1B[{};{}H", y + 1, 1));

            while x < self.width {
                let index = x + y * self.width;
                let pixel = &self.data[index];
                let current_color = pixel.color.to_ansii_escape(); // returns the ANSI escape code string

                // Accumulate characters with the same color
                let mut pixel_chars = String::new();
                while x < self.width && self.data[x + y * self.width].color == pixel.color {
                    pixel_chars.push(self.data[x + y * self.width].ch);
                    x += 1;
                }

                // Change color if necessary
                if last_color != Some(current_color.clone()) {
                    output.push_str(&current_color);
                    last_color = Some(current_color.clone());
                }

                // Append the accumulated characters
                output.push_str(&pixel_chars);
            }
        }

        // Show the cursor again
        output.push_str("\x1B[?25h");

        stdout.write_all(output.as_bytes())?;
        stdout.flush()
    }
}
pub fn render_scene<W: Write>(
    stdout: &mut W,
    scene: &mut Scene,
    camera: &Camera,
) -> std::io::Result<()> {
    let (width, height) = terminal::size().unwrap();
    let width = width as usize;
    let height = height as usize;

    let buffer = Arc::new(Mutex::new(Buffer::new(width, height)));
    buffer.lock().unwrap().clear();

    let view_matrix = camera.get_view_matrix();
    let render_mode = get_render_mode();

    for entity in &mut scene.entities {
        let transform_matrix = entity.transform.get_matrix();
        entity.mesh.update_normals();

        let transformed_vertices: Vec<Point3<f64>> = entity
            .mesh
            .vertices
            .par_iter()
            .map(|vertex| {
                let homogenous_vertex = vertex.to_homogeneous();
                let world_vertex = transform_matrix * homogenous_vertex;
                let view_vertex = view_matrix * world_vertex;
                Point3::from_homogeneous(view_vertex).unwrap()
            })
            .collect();

        let screen_coords: Vec<Point2<usize>> = transformed_vertices
            .par_iter()
            .map(|vertex| camera.project_vertex(vertex.coords, &width, &height))
            .collect();

        // Process faces in parallel  NOTE: Might not be best way of doing it
        let local_buffers: Vec<Buffer> = entity
            .mesh
            .faces
            .par_iter()
            .map(|face| {
                // FIX: This is bad, should not be allocating a new buffer for each face
                let mut local_buffer = Buffer::new(width, height);

                for tri in &face.tris {
                    let v0 = screen_coords[tri.vertices.0];
                    let v1 = screen_coords[tri.vertices.1];
                    let v2 = screen_coords[tri.vertices.2];

                    if !is_clockwise(&v0, &v1, &v2) {
                        continue;
                    }

                    if render_mode == RenderMode::Wireframe {
                        draw_line(&mut local_buffer, &v0, &v1, &Pixel::new('#', tri.color));
                        draw_line(&mut local_buffer, &v1, &v2, &Pixel::new('#', tri.color));
                        draw_line(&mut local_buffer, &v2, &v0, &Pixel::new('#', tri.color));
                    } else if render_mode == RenderMode::Solid {
                        draw_filled_triangle_scanline(
                            &mut local_buffer,
                            &v0,
                            &v1,
                            &v2,
                            &Pixel::new_full(face.color),
                        );
                    }
                }

                local_buffer
            })
            .collect();

        // Merge all local buffers into the shared buffer
        {
            let mut shared_buffer = buffer.lock().unwrap();
            for local_buffer in local_buffers {
                merge_buffers(&mut shared_buffer, &local_buffer);
            }
        }
    }

    // Render to terminal
    let _out = buffer.lock().unwrap().render_to_terminal();
    Ok(())
}

fn merge_buffers(shared_buffer: &mut Buffer, local_buffer: &Buffer) {
    for (shared_pixel, local_pixel) in shared_buffer.data.iter_mut().zip(&local_buffer.data) {
        if local_pixel.ch != ' ' {
            *shared_pixel = *local_pixel;
        }
    }
}

// Basic Bresenham's Line Drawing Algorithm for drawing wireframe edges
fn draw_line(buffer: &mut Buffer, v0: &Point2<usize>, v1: &Point2<usize>, pix: &Pixel) {
    let mut v0: Point2<isize> = v0.cast();
    let v1: Point2<isize> = v1.cast();

    let dx = (v1.x - v0.x).abs();
    let dy = -(v1.y - v0.y).abs();
    let sx = if v0.x < v1.x { 1 } else { -1 };
    let sy = if v0.y < v1.y { 1 } else { -1 };
    let mut err = dx + dy;
    while v0.x != v1.x || v0.y != v1.y {
        if v0.x >= 0 && v0.x < buffer.width as isize && v0.y >= 0 && v0.y < buffer.height as isize {
            buffer.set_pixel(v0.x as usize, v0.y as usize, pix.ch, pix.color);
        }

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

//TODO: do this algorithm instead
//https://groups.csail.mit.edu/graphics/classes/6.837/F98/Lecture7/triangles.html
fn cull_face_normal(
    face: &Face,
    transformed_vertices: &[Point3<f64>],
    camera_position: &Point3<f64>,
) -> bool {
    // Compute face center by averaging all vertex positions in the face
    let face_center_coords = face.tris.iter().fold(Vector3::zeros(), |acc, tri| {
        acc + transformed_vertices[tri.vertices.0].coords
            + transformed_vertices[tri.vertices.1].coords
            + transformed_vertices[tri.vertices.2].coords
    }) / (face.tris.len() * 3) as f64;

    // Convert face_center from Vector3 back to Point3
    let face_center = Point3::from(face_center_coords);

    // Calculate vector from camera to face center
    let view_dir = (face_center - camera_position).normalize();

    // Perform backface culling for the whole face using face normal
    face.normal.dot(&view_dir) > 0.0
}

// https://en.wikipedia.org/wiki/Curve_orientation#Orientation_of_a_simple_polygon
// NOTE: Apparently it is generally more efficient to backface cull AFTER screen projection via a
// counterclockwise check (if counterclockwise, cull I think)
// The other method involves surface normals, but might be more costly?
// Calculate the 2D cross product of vectors (v1 - v0) and (v2 - v0)
fn is_clockwise(v0: &Point2<usize>, v1: &Point2<usize>, v2: &Point2<usize>) -> bool {
    let v0: Point2<isize> = v0.cast();
    let v1: Point2<isize> = v1.cast();
    let v2: Point2<isize> = v2.cast();
    let cross_z = (v1.x - v0.x) * (v2.y - v0.y) - (v1.y - v0.y) * (v2.x - v0.x);
    cross_z < 0 // If cross_z is negative, vertices are clockwise (back-facing)
}

fn draw_filled_triangle_scanline(
    buffer: &mut Buffer,
    v0: &Point2<usize>,
    v1: &Point2<usize>,
    v2: &Point2<usize>,
    pix: &Pixel,
) {
    // Sort vertices by y-coordinate (ascending)
    let (v0, v1, v2) = if v0.y <= v1.y && v0.y <= v2.y {
        if v1.y <= v2.y {
            (v0, v1, v2)
        } else {
            (v0, v2, v1)
        }
    } else if v1.y <= v0.y && v1.y <= v2.y {
        if v0.y <= v2.y {
            (v1, v0, v2)
        } else {
            (v1, v2, v0)
        }
    } else if v0.y <= v1.y {
        (v2, v0, v1)
    } else {
        (v2, v1, v0)
    };

    // Prevent overflow with checked arithmetic when calculating split point
    let dy_v1_v0 = v1.y as isize - v0.y as isize;
    let dy_v2_v0 = v2.y as isize - v0.y as isize;
    let dx_v2_v0 = v2.x as isize - v0.x as isize;

    if v1.y == v2.y {
        fill_flat_bottom_triangle(buffer, v0, v1, v2, pix);
    } else if v0.y == v1.y {
        fill_flat_top_triangle(buffer, v0, v1, v2, pix);
    } else {
        if dy_v2_v0 != 0 {
            let v_split_x = v0.x as isize + dy_v1_v0.checked_mul(dx_v2_v0).unwrap_or(0) / dy_v2_v0;
            let v_split_x = v_split_x.clamp(0, buffer.width as isize - 1) as usize;
            let v_split = Point2::new(v_split_x, v1.y);

            fill_flat_bottom_triangle(buffer, v0, v1, &v_split, pix);
            fill_flat_top_triangle(buffer, v1, &v_split, v2, pix);
        }
    }
}

// Helper function to fill a flat-bottom triangle
fn fill_flat_bottom_triangle(
    buffer: &mut Buffer,
    v0: &Point2<usize>,
    v1: &Point2<usize>,
    v2: &Point2<usize>,
    pix: &Pixel,
) {
    // Avoid division by zero by adding a small epsilon when needed
    let dy_v1_v0 = (v1.y as isize - v0.y as isize).max(1);
    let dy_v2_v0 = (v2.y as isize - v0.y as isize).max(1);

    let inv_slope1 = (v1.x as isize - v0.x as isize) as f32 / dy_v1_v0 as f32;
    let inv_slope2 = (v2.x as isize - v0.x as isize) as f32 / dy_v2_v0 as f32;

    let mut cur_x1 = v0.x as f32;
    let mut cur_x2 = v0.x as f32;

    for y in v0.y..=v1.y {
        draw_horizontal_line(buffer, cur_x1 as usize, cur_x2 as usize, y, pix);
        cur_x1 += inv_slope1;
        cur_x2 += inv_slope2;
    }
}

// Helper function to fill a flat-top triangle
fn fill_flat_top_triangle(
    buffer: &mut Buffer,
    v0: &Point2<usize>,
    v1: &Point2<usize>,
    v2: &Point2<usize>,
    pix: &Pixel,
) {
    let dy_v2_v0 = (v2.y as isize - v0.y as isize).max(1);
    let dy_v2_v1 = (v2.y as isize - v1.y as isize).max(1);

    let inv_slope1 = (v2.x as isize - v0.x as isize) as f32 / dy_v2_v0 as f32;
    let inv_slope2 = (v2.x as isize - v1.x as isize) as f32 / dy_v2_v1 as f32;

    let mut cur_x1 = v2.x as f32;
    let mut cur_x2 = v2.x as f32;

    for y in (v0.y..=v2.y).rev() {
        draw_horizontal_line(buffer, cur_x1 as usize, cur_x2 as usize, y, pix);
        cur_x1 -= inv_slope1;
        cur_x2 -= inv_slope2;
    }
}

// Draws a horizontal line between two x coordinates at a given y coordinate
fn draw_horizontal_line(buffer: &mut Buffer, x1: usize, x2: usize, y: usize, pix: &Pixel) {
    let (start_x, end_x) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
    let clamped_y = y.clamp(0, buffer.height - 1);
    for x in start_x..=end_x.min(buffer.width - 1) {
        buffer.set_pixel(x, clamped_y, pix.ch, pix.color);
    }
}
