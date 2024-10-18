use crate::{scene::Scene, buffer::Buffer, camera::Camera};
use nalgebra::Vector4;
use std::io::Write;

pub fn render_scene<W: Write>(stdout: &mut W, scene: &Scene, camera: &Camera) -> std::io::Result<()> {
    let (width, height) = (80, 40); // Terminal size
    let mut buffer = Buffer::new(width, height);

    buffer.clear();

    let view_matrix = camera.get_view_matrix();

    for entity in &scene.entities {
        let mut projected_vertices = vec![];

        // Project each vertex to 2D
        for vertex in &entity.vertices {
            let mut v = Vector4::new(vertex.x, vertex.y, vertex.z, 1.0);

            // Transform the vertex by the camera's view matrix
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

        // Draw edges (wireframe)
        for &(start_idx, end_idx) in &entity.edges {
            let (x0, y0) = projected_vertices[start_idx];
            let (x1, y1) = projected_vertices[end_idx];

            draw_line(&mut buffer, x0, y0, x1, y1, '@'); // Draw wireframe using '@' character
        }
    }

    // Render the buffer to the terminal using Crossterm
    buffer.render_to_terminal()?;
    Ok(())
}

// Basic Bresenham's Line Drawing Algorithm for drawing wireframe edges
fn draw_line(buffer: &mut Buffer, x0: usize, y0: usize, x1: usize, y1: usize, ch: char) {
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
        buffer.set_char(x0 as usize, y0 as usize, ch);

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

