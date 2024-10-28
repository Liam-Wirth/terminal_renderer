use crate::core::color::Color;
use crate::core::face::Face;
use crossterm::terminal;
use glam::{Vec2, Vec3};
use std::io::Write;
use std::sync::{Arc, Mutex};
use crate::core::{camera::Camera, scene::Scene};

use super::buffer::Buffer;
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

        let transformed_vertices: Vec<Vec3> = entity
            .mesh
            .vertices
            .par_iter()
            .map(|vertex| {
                let homogenous_vertex = vertex.extend(1.0);
                let world_vertex = transform_matrix * homogenous_vertex;
                let view_vertex = view_matrix * world_vertex;
                view_vertex.truncate()
            })
            .collect();

        let screen_coords: Vec<Vec2> = transformed_vertices
            .par_iter()
            .map(|vertex| camera.project_vertex(*vertex, width, height))
            .collect();

        // Process faces in parallel
        let local_buffers: Vec<Buffer> = entity
            .mesh
            .faces
            .par_iter()
            .map(|face| {
                let mut local_buffer = Buffer::new(width, height);

                for tri in &face.tris {
                    let v0 = screen_coords[tri.vertices.0];
                    let v1 = screen_coords[tri.vertices.1];
                    let v2 = screen_coords[tri.vertices.2];

                    if !is_clockwise(&v0, &v1, &v2) {
                        continue;
                    }

                    if render_mode == RenderMode::Wireframe {
                        draw_line(&mut local_buffer, v0, v1, &Pixel::new('#', tri.color));
                        draw_line(&mut local_buffer, v1, v2, &Pixel::new('#', tri.color));
                        draw_line(&mut local_buffer, v2, v0, &Pixel::new('#', tri.color));
                    } else if render_mode == RenderMode::Solid {
                        draw_filled_triangle_scanline(
                            &mut local_buffer,
                            v0,
                            v1,
                            v2,
                            &Pixel::new_full(face.color),
                        );
                    }
                }

                local_buffer
            })
            .collect();

        {
            let mut shared_buffer = buffer.lock().unwrap();
            for local_buffer in local_buffers {
                merge_buffers(&mut shared_buffer, &local_buffer);
            }
        }
    }

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
fn draw_line(buffer: &mut Buffer, v0: Vec2, v1: Vec2, pix: &Pixel) {
    let mut v0 = v0.as_ivec2();
    let v1 = v1.as_ivec2();

    let dx = (v1.x - v0.x).abs();
    let dy = -(v1.y - v0.y).abs();
    let sx = if v0.x < v1.x { 1 } else { -1 };
    let sy = if v0.y < v1.y { 1 } else { -1 };
    let mut err = dx + dy;
    while v0.x != v1.x || v0.y != v1.y {
        if v0.x >= 0 && v0.x < buffer.width as i32 && v0.y >= 0 && v0.y < buffer.height as i32 {
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
    transformed_vertices: &[Vec3],
    camera_position: Vec3,
) -> bool {
    // Compute face center by averaging all vertex positions in the face
    let face_center_coords = face.tris.iter().fold(Vec3::ZERO, |acc, tri| {
        acc + transformed_vertices[tri.vertices.0]
            + transformed_vertices[tri.vertices.1]
            + transformed_vertices[tri.vertices.2]
    }) / (face.tris.len() * 3) as f32;

    let view_dir = (face_center_coords - camera_position).normalize();

    // Perform backface culling for the whole face using face normal
    face.normal.dot(view_dir) > 0.0
}

// https://en.wikipedia.org/wiki/Curve_orientation#Orientation_of_a_simple_polygon
// NOTE: Apparently it is generally more efficient to backface cull AFTER screen projection via a
// counterclockwise check (if counterclockwise, cull I think)
// The other method involves surface normals, but might be more costly?
// Calculate the 2D cross product of vectors (v1 - v0) and (v2 - v0)

fn is_clockwise(v0: &Vec2, v1: &Vec2, v2: &Vec2) -> bool {
    let v0 = v0.as_ivec2();
    let v1 = v1.as_ivec2();
    let v2 = v2.as_ivec2();
    let cross_z = (v1.x - v0.x) * (v2.y - v0.y) - (v1.y - v0.y) * (v2.x - v0.x);
    cross_z < 0 // If cross_z is negative, vertices are clockwise (back-facing)
}

fn draw_filled_triangle_scanline(
    buffer: &mut Buffer,
    v0: Vec2,
    v1: Vec2,
    v2: Vec2,
    pix: &Pixel,
) {
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

    let dy_v1_v0 = v1.y as i32 - v0.y as i32;
    let dy_v2_v0 = v2.y as i32 - v0.y as i32;
    let dx_v2_v0 = v2.x as i32 - v0.x as i32;

    if v1.y == v2.y {
        fill_flat_bottom_triangle(buffer, v0, v1, v2, pix);
    } else if v0.y == v1.y {
        fill_flat_top_triangle(buffer, v0, v1, v2, pix);
    } else if dy_v2_v0 != 0 {
        let v_split_x = v0.x as i32 + dy_v1_v0.checked_mul(dx_v2_v0).unwrap_or(0) / dy_v2_v0;
        let v_split_x = v_split_x.clamp(0, buffer.width as i32 - 1) as f32;
        let v_split = Vec2::new(v_split_x, v1.y);

        fill_flat_bottom_triangle(buffer, v0, v1, v_split, pix);
        fill_flat_top_triangle(buffer, v1, v_split, v2, pix);
    }
}

fn fill_flat_bottom_triangle(
    buffer: &mut Buffer,
    v0: Vec2,
    v1: Vec2,
    v2: Vec2,
    pix: &Pixel,
) {
    let dy_v1_v0 = (v1.y as i32 - v0.y as i32).max(1);
    let dy_v2_v0 = (v2.y as i32 - v0.y as i32).max(1);

    let inv_slope1 = (v1.x as i32 - v0.x as i32) as f32 / dy_v1_v0 as f32;
    let inv_slope2 = (v2.x as i32 - v0.x as i32) as f32 / dy_v2_v0 as f32;

    let mut cur_x1 = v0.x;
    let mut cur_x2 = v0.x;

    for y in v0.y as usize..=v1.y as usize {
        draw_horizontal_line(buffer, cur_x1 as usize, cur_x2 as usize, y, pix);
        cur_x1 += inv_slope1;
        cur_x2 += inv_slope2;
    }
}

fn fill_flat_top_triangle(
    buffer: &mut Buffer,
    v0: Vec2,
    v1: Vec2,
    v2: Vec2,
    pix: &Pixel,
) {
    let dy_v2_v0 = (v2.y as i32 - v0.y as i32).max(1);
    let dy_v2_v1 = (v2.y as i32 - v1.y as i32).max(1);

    let inv_slope1 = (v2.x as i32 - v0.x as i32) as f32 / dy_v2_v0 as f32;
    let inv_slope2 = (v2.x as i32 - v1.x as i32) as f32 / dy_v2_v1 as f32;

    let mut cur_x1 = v2.x;
    let mut cur_x2 = v2.x;

    for y in (v0.y as usize..=v2.y as usize).rev() {
        draw_horizontal_line(buffer, cur_x1 as usize, cur_x2 as usize, y, pix);
        cur_x1 -= inv_slope1;
        cur_x2 -= inv_slope2;
    }
}

fn draw_horizontal_line(buffer: &mut Buffer, x1: usize, x2: usize, y: usize, pix: &Pixel) {
    let (start_x, end_x) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
    let clamped_y = y.clamp(0, buffer.height - 1);
    for x in start_x..=end_x.min(buffer.width - 1) {
        buffer.set_pixel(x, clamped_y, pix.ch, pix.color);
    }
}
