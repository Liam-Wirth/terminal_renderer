use crossterm::{
    cursor::MoveTo,
    style::{Color, Print, SetForegroundColor},
    terminal, QueueableCommand,
};
use nalgebra::{Vector2, Vector4};
use std::io::{stdout, Write};

use crate::core::{camera::Camera, scene::Scene};

struct Tri {
    v1: Vector2<usize>,
    v2: Vector2<usize>,
    v3: Vector2<usize>,
    pixel: Pixel,
}

impl Tri {
    pub fn new(v1: Vector2<usize>, v2: Vector2<usize>, v3: Vector2<usize>, pixel: Pixel) -> Self {
        Tri { v1, v2, v3, pixel }
    }
}

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
        let mut stdout = stdout();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = x + y * self.width;
                let pixel = &self.data[idx];

                // Move the cursor and set the color before printing the character
                stdout.queue(MoveTo(x as u16, y as u16))?;
                stdout.queue(SetForegroundColor(pixel.color))?;
                stdout.queue(Print(pixel.ch))?;
            }
        }

        // Flush the commands to the terminal
        stdout.flush()?;
        Ok(())
    }
}

fn is_backface(v1: &Vector2<f64>, v2: &Vector2<f64>, v3: &Vector2<f64>) -> bool {
    let edge1 = *v2 - *v1;
    let edge2 = *v3 - *v1;

    // Cross product (2D)
    let cross_product = edge1.x * edge2.y - edge1.y * edge2.x;

    // If the cross product is negative, it's a backface
    cross_product < 0.0
}

pub fn render_scene<W: Write>(
    stdout: &mut W,
    scene: &Scene,
    camera: &Camera,
    fill_faces: bool,
) -> std::io::Result<()> {
    // Get terminal size dynamically
    let (width, height) = terminal::size().unwrap();
    let width = width as usize;
    let height = height as usize;

    let mut buffer = Buffer::new(width, height);

    buffer.clear();

    let view_matrix = camera.get_view_matrix();

    for entity in &scene.entities {
        let mut projected_vertices = vec![];

        // Project each vertex to 2D
        for vertex in &entity.mesh.vertices {
            let mut v = Vector4::new(vertex.x, vertex.y, vertex.z, 1.0);

            // Apply entity's transformation
            v = entity.transform.apply_to_vertex(v);

            // Transform by the camera's view matrix
            v = view_matrix * v;

            // Perspective projection
            let fov_adjustment = (camera.fov / 2.0).to_radians().tan();
            v.x /= v.z * fov_adjustment;
            v.y /= v.z * fov_adjustment;

            // Convert from normalized device coordinates to screen space
            let screen_x = ((v.x + 1.0) / 2.0 * width as f64) as usize;
            let screen_y = ((1.0 - (v.y + 1.0) / 2.0) * height as f64) as usize;

            projected_vertices.push((screen_x, screen_y));
        }
        if fill_faces {
            // Draw filled faces using the new Tri struct
            for face in &entity.mesh.faces {
                let (x0, y0) = projected_vertices[face.vertices.0];
                let (x1, y1) = projected_vertices[face.vertices.1];
                let (x2, y2) = projected_vertices[face.vertices.2];

                let v1 = Vector2::new(x0 as f64, y0 as f64);
                let v2 = Vector2::new(x1 as f64, y1 as f64);
                let v3 = Vector2::new(x2 as f64, y2 as f64);

                // Skip backfaces
                if is_backface(&v1, &v2, &v3) {
                    continue;
                }

                let triangle = Tri::new(
                    Vector2::new(x0, y0),
                    Vector2::new(x1, y1),
                    Vector2::new(x2, y2),
                    Pixel::new('█', face.color),
                );

                draw_filled_triangle(&mut buffer, triangle);
            }
        } else {
            // Draw edges (wireframe)
            for &(start_idx, end_idx) in &entity.mesh.edges {
                let (x0, y0) = projected_vertices[start_idx];
                let (x1, y1) = projected_vertices[end_idx];

                draw_line(&mut buffer, x0, y0, x1, y1, '#', Color::Green); // Draw wireframe using '@' character in green
            }
        }
    }

    // Render the buffer to the terminal using Crossterm
    buffer.render_to_terminal()?;
    Ok(())
}
// Basic Bresenham's Line Drawing Algorithm for drawing wireframe edges
fn draw_line(
    buffer: &mut Buffer,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    ch: char,
    color: Color,
) {
    let mut x0 = x0 as isize;
    let mut y0 = y0 as isize;
    let x1 = x1 as isize;
    let y1 = y1 as isize;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let mut sx = if x0 < x1 { 1 } else { -1 };
    let mut sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    while x0 != x1 || y0 != y1 {
        buffer.set_pixel(x0 as usize, y0 as usize, ch, color);

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

// TODO: implement face filling
fn draw_filled_triangle(buffer: &mut Buffer, triangle: Tri) {
    let Tri { v1, v2, v3, pixel } = triangle;

    let mut xs = [v1.x as f64, v2.x as f64, v3.x as f64];
    let mut ys = [v1.y as f64, v2.y as f64, v3.y as f64];

    // Sort vertices by y-coordinate (ascending)
    if ys[1] < ys[0] {
        xs.swap(0, 1);
        ys.swap(0, 1);
    }
    if ys[2] < ys[0] {
        xs.swap(0, 2);
        ys.swap(0, 2);
    }
    if ys[2] < ys[1] {
        xs.swap(1, 2);
        ys.swap(1, 2);
    }

    // Draw flat-bottom triangle
    if (ys[1] - ys[2]).abs() < f64::EPSILON {
        let flat_bottom_triangle = Tri {
            v1: Vector2::new(xs[0] as usize, ys[0] as usize),
            v2: Vector2::new(xs[1] as usize, ys[1] as usize),
            v3: Vector2::new(xs[2] as usize, ys[2] as usize),
            pixel,
        };
        fill_flat_bottom_triangle(buffer, flat_bottom_triangle);
    }
    // Draw flat-top triangle
    else if (ys[0] - ys[1]).abs() < f64::EPSILON {
        let flat_top_triangle = Tri {
            v1: Vector2::new(xs[0] as usize, ys[0] as usize),
            v2: Vector2::new(xs[1] as usize, ys[1] as usize),
            v3: Vector2::new(xs[2] as usize, ys[2] as usize),
            pixel,
        };
        fill_flat_top_triangle(buffer, flat_top_triangle);
    }
    // General triangle, split into a flat-bottom and flat-top triangle
    else {
        let x3 = xs[0] + ((ys[1] - ys[0]) * (xs[2] - xs[0])) / (ys[2] - ys[0]); // Interpolated X coordinate
        let flat_bottom_triangle = Tri {
            v1: Vector2::new(xs[0] as usize, ys[0] as usize),
            v2: Vector2::new(xs[1] as usize, ys[1] as usize),
            v3: Vector2::new(x3 as usize, ys[1] as usize),
            pixel,
        };
        fill_flat_bottom_triangle(buffer, flat_bottom_triangle);

        let flat_top_triangle = Tri {
            v1: Vector2::new(xs[1] as usize, ys[1] as usize),
            v2: Vector2::new(x3 as usize, ys[1] as usize),
            v3: Vector2::new(xs[2] as usize, ys[2] as usize),
            pixel,
        };
        fill_flat_top_triangle(buffer, flat_top_triangle);
    }
}

fn fill_flat_bottom_triangle(buffer: &mut Buffer, triangle: Tri) {
    let Tri { v1, v2, v3, pixel } = triangle;

    let inv_slope_1 = (v2.x as f64 - v1.x as f64) / (v2.y as f64 - v1.y as f64);
    let inv_slope_2 = (v3.x as f64 - v1.x as f64) / (v3.y as f64 - v1.y as f64);

    let mut cur_x1 = v1.x as f64;
    let mut cur_x2 = v1.x as f64;

    for scanline_y in v1.y..=v2.y {
        for x in cur_x1 as usize..=cur_x2 as usize {
            if x < buffer.width && scanline_y < buffer.height {
                buffer.set_pixel(x, scanline_y, pixel.ch, pixel.color);
            }
        }
        cur_x1 += inv_slope_1;
        cur_x2 += inv_slope_2;
    }
}

fn fill_flat_top_triangle(buffer: &mut Buffer, triangle: Tri) {
    let Tri { v1, v2, v3, pixel } = triangle;

    let inv_slope_1 = (v3.x as f64 - v1.x as f64) / (v3.y as f64 - v1.y as f64);
    let inv_slope_2 = (v3.x as f64 - v2.x as f64) / (v3.y as f64 - v2.y as f64);

    let mut cur_x1 = v3.x as f64;
    let mut cur_x2 = v3.x as f64;

    for scanline_y in (v1.y..=v3.y).rev() {
        for x in cur_x1 as usize..=cur_x2 as usize {
            if x < buffer.width && scanline_y < buffer.height {
                buffer.set_pixel(x, scanline_y, pixel.ch, pixel.color);
            }
        }
        cur_x1 -= inv_slope_1;
        cur_x2 -= inv_slope_2;
    }
}
