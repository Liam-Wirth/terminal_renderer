use crate::core::{Color, Pixel, Scene};
use crate::pipeline::{Fragment, ProcessedGeometry};
use glam::{Mat4, Vec2, Vec3, Vec4};
use rayon::prelude::*;

pub struct Rasterizer {
    width: usize,
    height: usize,
}

impl Rasterizer {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
    /// Modifies the fragment buffer in place
    pub fn rasterize(
        &mut self,
        geometry: &[ProcessedGeometry],
        scene: &Scene,
        frags: &mut Vec<Fragment>,
    ) {
        frags.clear();

        // Process each piece of geometry in parallel
        *frags = geometry
            .par_iter()
            .flat_map(|geo| {
                let entity = &scene.entities[geo.entity_id];
                self.process_mesh_triangles(entity, geo.transform)
            })
            .collect();
    }

    fn process_mesh_triangles(
        &self,
        entity: &crate::core::Entity,
        transform: Mat4,
    ) -> Vec<Fragment> {
        entity
            .mesh
            .tris
            .par_iter()
            .flat_map(|tri| {
                // Get vertex positions and transform them
                let vertices = self.get_transformed_vertices(tri, entity, transform);

                // Skip if triangle is outside view
                if !self.is_triangle_visible(&vertices) {
                    return Vec::new();
                }

                // Project to screen space
                let screen_verts = self.project_to_screen(&vertices);

                // Get vertex colors
                let colors = self.get_vertex_colors(tri, entity);

                // Rasterize the triangle
                self.rasterize_triangle(&screen_verts, &colors)
            })
            .collect()
    }

    fn get_transformed_vertices(
        &self,
        tri: &crate::core::geometry::Tri,
        entity: &crate::core::Entity,
        transform: Mat4,
    ) -> [Vec4; 3] {
        let mut vertices = [Vec4::ZERO; 3];
        for i in 0..3 {
            let pos = entity.mesh.vertices[tri.vertices[i]].pos;
            vertices[i] = transform * Vec4::new(pos.x, pos.y, pos.z, 1.0);
        }
        vertices
    }

    fn is_triangle_visible(&self, vertices: &[Vec4; 3]) -> bool {
        // Basic frustum culling
        for v in vertices {
            if v.w <= 0.0 {
                return false;
            }
        }
        true
    }

    fn project_to_screen(&self, vertices: &[Vec4; 3]) -> [Vec2; 3] {
        let mut screen_verts = [Vec2::ZERO; 3];
        for i in 0..3 {
            let ndc = Vec2::new(vertices[i].x / vertices[i].w, vertices[i].y / vertices[i].w);
            screen_verts[i] = Vec2::new(
                (ndc.x + 1.0) * 0.5 * self.width as f32,
                (ndc.y + 1.0) * 0.5 * self.height as f32,
            );
        }
        screen_verts
    }

    fn get_vertex_colors(
        &self,
        tri: &crate::core::geometry::Tri,
        entity: &crate::core::Entity,
    ) -> [Color; 3] {
        let mut colors = [Color::WHITE; 3];
        for i in 0..3 {
            colors[i] = entity.mesh.vertices[tri.vertices[i]]
                .color
                .unwrap_or(Color::WHITE);
        }
        colors
    }

    fn rasterize_triangle(&self, vertices: &[Vec2; 3], colors: &[Color; 3]) -> Vec<Fragment> {
        let mut fragments = Vec::new();

        // Draw edges
        for i in 0..3 {
            let start = vertices[i];
            let end = vertices[(i + 1) % 3];
            let color = colors[i];

            self.draw_line(start, end, color, &mut fragments);
        }

        fragments
    }

    fn draw_line(&self, start: Vec2, end: Vec2, color: Color, fragments: &mut Vec<Fragment>) {
        let mut steep = false;

        let mut x0 = start.x as i32;
        let mut y0 = start.y as i32;
        let mut x1 = end.x as i32;
        let mut y1 = end.y as i32;

        if (x0 - x1).abs() < (y0 - y1).abs() {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
            steep = true;
        }

        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        let dx = x1 - x0;
        let dy = y1 - y0;
        let derror2 = dy.abs() * 2;
        let mut error2 = 0;
        let mut y = y0;

        for x in x0..=x1 {
            let pos = if steep {
                Vec2::new(y as f32, x as f32)
            } else {
                Vec2::new(x as f32, y as f32)
            };

            // Only add fragments within screen bounds
            if pos.x >= 0.0
                && pos.x < self.width as f32
                && pos.y >= 0.0
                && pos.y < self.height as f32
            {
                fragments.push(Fragment {
                    screen_pos: pos,
                    depth: 0.0, // TODO: Implement proper depth calculation
                    color,
                });
            }

            error2 += derror2;
            if error2 > dx {
                y += if y1 > y0 { 1 } else { -1 };
                error2 -= dx * 2;
            }
        }
    }
}
