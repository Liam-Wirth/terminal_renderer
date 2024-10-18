use crate::{scene::Scene, buffer::Buffer, camera::Camera};
use nalgebra::Vector4;
use std::io::Write;

pub fn render_scene<W: Write>(_stdout: &mut W, scene: &Scene, camera: &Camera) -> std::io::Result<()> {
    let (width, height) = (80, 40); // Terminal size
    let mut buffer = Buffer::new(width, height);

    buffer.clear();

    let view_matrix = camera.get_view_matrix();

    for entity in &scene.entities {
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

            buffer.set_char(screen_x, screen_y, '@');
        }
    }

    // Render the buffer to the terminal using Crossterm
    buffer.render_to_terminal()?;
    Ok(())
}

