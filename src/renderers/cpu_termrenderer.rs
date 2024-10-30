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

// TODO: would be cool to make this more abstract/ abstract a lot of the logic such that
// I can just have alot of these structs exist as the "backend" for whatever type of output
// rendering I end up trying to do (ppm file -> png, ppm gif or smth, I dunno)
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

pub fn render_scene<W: Write>(
    stdout: &mut W,
    scene: &mut Scene,
    camera: &Camera,
) -> std::io::Result<()> {
    let (width, height) = terminal::size().unwrap();
    let width = width as usize;
    let height = height as usize;

    let buffer = Arc::new(Mutex::new(Buffer::new(width, height)));

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

        // Project the vertices into screen space
        let projected_vertices: Vec<ProjectedVertex> = transformed_vertices
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
                    let v0 = &projected_vertices[tri.vertices.0];
                    let v1 = &projected_vertices[tri.vertices.1];
                    let v2 = &projected_vertices[tri.vertices.2];

                    if !is_clockwise(&v0.position, &v1.position, &v2.position) {
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

pub fn render_scene_par<W: Write>(
    stdout: &mut W,
    scene: &mut Scene,
    cam: &mut Camera,
    bufs: &mut Vec<Buffer>,
) -> std::io::Result<()> {
    todo!();
}
fn merge_buffers(shared_buffer: &mut Buffer, local_buffer: &Buffer) {
    for (shared_pixel, local_pixel) in shared_buffer.data.iter_mut().zip(&local_buffer.data) {
        if local_pixel.ch != ' ' {
            *shared_pixel = *local_pixel;
        }
    }
}

// Basic Bresenham's Line Drawing Algorithm for drawing wireframe edges
fn draw_line(buffer: &mut Buffer, v0: &ProjectedVertex, v1: &ProjectedVertex, pix: &Pixel) {
    let mut v0_screen = v0.position.as_ivec2();
    let v1_screen = v1.position.as_ivec2();
    let dx = (v1_screen.x - v0_screen.x).abs();
    let dy = -(v1_screen.y - v0_screen.y).abs();
    let sx = if v0_screen.x < v1_screen.x { 1 } else { -1 };
    let sy = if v0_screen.y < v1_screen.y { 1 } else { -1 };
    let mut err = dx + dy;

    while v0_screen.x != v1_screen.x || v0_screen.y != v1_screen.y {
        if v0_screen.x >= 0
            && v0_screen.x < buffer.width as i32
            && v0_screen.y >= 0
            && v0_screen.y < buffer.height as i32
        {
            // Interpolate depth here if needed
            buffer.set_pixel(
                v0_screen.x as usize,
                v0_screen.y as usize,
                v0,
                *pix,
            );
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            v0_screen.x += sx;
        }
        if e2 <= dx {
            err += dx;
            v0_screen.y += sy;
        }
    }
}

// Adjust `draw_filled_triangle_scanline` to take `ProjectedVertex`s and use `set_pixel` similarly

//TODO: do this algorithm instead
//https://groups.csail.mit.edu/graphics/classes/6.837/F98/Lecture7/triangles.html
fn cull_face_normal(face: &Face, transformed_vertices: &[Vec3], camera_position: Vec3) -> bool {
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
    v0: &ProjectedVertex,
    v1: &ProjectedVertex,
    v2: &ProjectedVertex,
    pix: &Pixel,
) {
    let (v0, v1, v2) = if v0.position.y <= v1.position.y && v0.position.y <= v2.position.y {
        if v1.position.y <= v2.position.y {
            (v0, v1, v2)
        } else {
            (v0, v2, v1)
        }
    } else if v1.position.y <= v0.position.y && v1.position.y <= v2.position.y {
        if v0.position.y <= v2.position.y {
            (v1, v0, v2)
        } else {
            (v1, v2, v0)
        }
    } else if v0.position.y <= v1.position.y {
        (v2, v0, v1)
    } else {
        (v2, v1, v0)
    };

    if v1.position.y == v2.position.y {
        fill_flat_bottom_triangle(buffer, v0, v1, v2, pix);
    } else if v0.position.y == v1.position.y {
        fill_flat_top_triangle(buffer, v0, v1, v2, pix);
    } else {
        let dy_v2_v0 = v2.position.y - v0.position.y;
        let dy_v1_v0 = v1.position.y - v0.position.y;
        let dx_v2_v0 = v2.position.x - v0.position.x;
        let v_split_x = v0.position.x + dx_v2_v0 * (dy_v1_v0 / dy_v2_v0);
        let v_split_depth = v0.depth + (v2.depth - v0.depth) * (dy_v1_v0 / dy_v2_v0);

        let v_split = ProjectedVertex {
            position: Vec2::new(v_split_x, v1.position.y),
            depth: v_split_depth,
        };

        fill_flat_bottom_triangle(buffer, v0, v1, &v_split, pix);
        fill_flat_top_triangle(buffer, v1, &v_split, v2, pix);
    }
}

fn fill_flat_bottom_triangle(
    buffer: &mut Buffer,
    v0: &ProjectedVertex,
    v1: &ProjectedVertex,
    v2: &ProjectedVertex,
    pix: &Pixel,
) {
    let dy_v1_v0 = (v1.position.y - v0.position.y).max(1.0);
    let dy_v2_v0 = (v2.position.y - v0.position.y).max(1.0);

    let inv_slope1 = (v1.position.x - v0.position.x) / dy_v1_v0;
    let inv_slope2 = (v2.position.x - v0.position.x) / dy_v2_v0;

    let depth_slope1 = (v1.depth - v0.depth) / dy_v1_v0;
    let depth_slope2 = (v2.depth - v0.depth) / dy_v2_v0;

    let mut cur_x1 = v0.position.x;
    let mut cur_x2 = v0.position.x;
    let mut cur_depth1 = v0.depth;
    let mut cur_depth2 = v0.depth;

    for y in v0.position.y as usize..=v1.position.y as usize {
        draw_horizontal_line(buffer, cur_x1, cur_depth1, cur_x2, cur_depth2, y, pix);
        cur_x1 += inv_slope1;
        cur_x2 += inv_slope2;
        cur_depth1 += depth_slope1;
        cur_depth2 += depth_slope2;
    }
}

fn fill_flat_top_triangle(
    buffer: &mut Buffer,
    v0: &ProjectedVertex,
    v1: &ProjectedVertex,
    v2: &ProjectedVertex,
    pix: &Pixel,
) {
    let dy_v2_v0 = (v2.position.y - v0.position.y).max(1.0);
    let dy_v2_v1 = (v2.position.y - v1.position.y).max(1.0);

    let inv_slope1 = (v2.position.x - v0.position.x) / dy_v2_v0;
    let inv_slope2 = (v2.position.x - v1.position.x) / dy_v2_v1;

    let depth_slope1 = (v2.depth - v0.depth) / dy_v2_v0;
    let depth_slope2 = (v2.depth - v1.depth) / dy_v2_v1;

    let mut cur_x1 = v2.position.x;
    let mut cur_x2 = v2.position.x;
    let mut cur_depth1 = v2.depth;
    let mut cur_depth2 = v2.depth;

    for y in (v0.position.y as usize..=v2.position.y as usize).rev() {
        draw_horizontal_line(buffer, cur_x1, cur_depth1, cur_x2, cur_depth2, y, pix);
        cur_x1 -= inv_slope1;
        cur_x2 -= inv_slope2;
        cur_depth1 -= depth_slope1;
        cur_depth2 -= depth_slope2;
    }
}

fn draw_horizontal_line(
    buffer: &mut Buffer,
    x1: f32,
    depth1: f32,
    x2: f32,
    depth2: f32,
    y: usize,
    pix: &Pixel,
) {
    let (start_x, end_x) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
    let (start_depth, end_depth) = if x1 < x2 {
        (depth1, depth2)
    } else {
        (depth2, depth1)
    };

    let clamped_y = y.clamp(0, buffer.height - 1);

    let dx = (end_x - start_x).max(1.0);
    let depth_slope = (end_depth - start_depth) / dx;

    let mut cur_depth = start_depth;

    for x in start_x as usize..=end_x as usize {
        if x < buffer.width {
            buffer.set_pixel(
                x,
                clamped_y,
                &ProjectedVertex {
                    position: Vec2::new(x as f32, y as f32),
                    depth: cur_depth,
                },
                *pix
            );
            cur_depth += depth_slope;
        }
    }
}
