use crate::core::geometry::Tri;
use crate::core::{Color, Entity, Scene};
use crate::debug_print;
use crate::pipeline::{Fragment, ProcessedGeometry};
use glam::{Mat4, Vec2, Vec4};
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
        debug_print!("Processing {} geometries", geometry.len());

        *frags = geometry
            .par_iter()
            .flat_map(|geo| self.process_mesh_triangles(geo))
            .collect();

        debug_print!("Generated {} fragments", frags.len());
    }

    fn process_mesh_triangles(&self, geo: &ProcessedGeometry) -> Vec<Fragment> {
        // Now we're working with already clipped vertices
        let vertices = [
            geo.vertices[0].position,
            geo.vertices[1].position,
            geo.vertices[2].position,
        ];

        // Project to screen space
        let screen_verts = self.project_to_screen(&vertices);

        // Get colors from the clipped vertices
        let colors = [
            geo.vertices[0].color,
            geo.vertices[1].color,
            geo.vertices[2].color,
        ];

        // Rasterize the triangle
      self.rasterize_triangle_barycentric(screen_verts, colors)
    }

    // fn project_to_screen(&self, vertices: &[Vec4; 3]) -> [Vec2; 3] {
    //     let mut screen_verts = [Vec2::ZERO; 3];
    //     for i in 0..3 {
    //         // Perspective divide
    //         let ndc = Vec2::new(vertices[i].x / vertices[i].w, vertices[i].y / vertices[i].w);
    //         // Convert to screen space
    //         screen_verts[i] = Vec2::new(
    //             (ndc.x + 1.0) * 0.5 * self.width as f32,
    //             (ndc.y + 1.0) * 0.5 * self.height as f32,
    //         );
    //     }
    //     screen_verts
    // }

    // fn process_mesh_triangles(&self, processed_geo: &ProcessedGeometry) -> Vec<Fragment> {
    //     let vertices = [
    //         processed_geo.vertices[0].position,
    //         processed_geo.vertices[1].position,
    //         processed_geo.vertices[2].position,
    //     ];

    //     let colors = [
    //         processed_geo.vertices[0].color,
    //         processed_geo.vertices[1].color,
    //         processed_geo.vertices[2].color,
    //     ];

    //     let screen_verts = self.project_to_screen(&vertices);
    //     self.rasterize_triangle_barycentric(screen_verts, colors)
    // }

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

    // TODO: parallelize
    fn is_triangle_visible(&self, vertices: &[Vec4; 3]) -> bool {
        for v in vertices {
            let ndc_z = v.z / v.w;
            debug_print!("Vertex NDC z: {}", ndc_z);
            if ndc_z > 1.0 || ndc_z < -1.0 {
                debug_print!("Culling triangle: vertex z/w = {} outside [-1,1]", ndc_z);
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

    // TODO: Implement a switch statement matching the global state to determine if triangle rasterization means:
    // Wireframe tris
    // or
    // filled tris
    fn rasterize_triangle(&self, vertices: &[Vec2; 3], colors: &[Color; 3]) -> Vec<Fragment> {
        let mut fragments = Vec::new();

        for i in 0..3 {
            let next = (i + 1) % 3;
            self.draw_line(
                (vertices[i], colors[i]),
                (vertices[next], colors[next]),
                &mut fragments,
            );
        }
        fragments
    }
    fn draw_line(&self, start: (Vec2, Color), end: (Vec2, Color), fragments: &mut Vec<Fragment>) {
        let mut steep = false;

        let (start_vert, start_col) = start;
        let (end_vert, end_col) = end;

        let mut x0 = start_vert.x as i32;
        let mut y0 = start_vert.y as i32;
        let mut x1 = end_vert.x as i32;
        let mut y1 = end_vert.y as i32;

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

            // Calculate interpolation factor based on x-distance
            let t = if dx != 0 {
                (x - x0) as f32 / dx as f32
            } else {
                1.0
            };

            // Interpolate color using the lerp method
            let color = start_col.lerp(&end_col, t);

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

    fn rasterize_triangle_barycentric(
        &self,
        screen_verts: [glam::Vec2; 3],
        colors: [crate::core::Color; 3],
    ) -> Vec<crate::pipeline::Fragment> {
        let mut fragments = Vec::new();

        // Compute bounding box
        let mut bbox_min = glam::Vec2::new(self.width as f32 - 1.0, self.height as f32 - 1.0);
        let mut bbox_max = glam::Vec2::new(0.0, 0.0);

        for v in &screen_verts {
            bbox_min.x = bbox_min.x.min(v.x);
            bbox_min.y = bbox_min.y.min(v.y);
            bbox_max.x = bbox_max.x.max(v.x);
            bbox_max.y = bbox_max.y.max(v.y);
        }

        // Clamp bounding box to screen
        bbox_min.x = bbox_min.x.max(0.0);
        bbox_min.y = bbox_min.y.max(0.0);
        bbox_max.x = bbox_max.x.min((self.width - 1) as f32);
        bbox_max.y = bbox_max.y.min((self.height - 1) as f32);

        let (v0, v1, v2) = (screen_verts[0], screen_verts[1], screen_verts[2]);
        for y in bbox_min.y as i32..=bbox_max.y as i32 {
            for x in bbox_min.x as i32..=bbox_max.x as i32 {
                let p = glam::Vec2::new(x as f32, y as f32);
                if let Some((w0, w1, w2)) = barycentric(p, v0, v1, v2) {
                    // If inside the triangle
                    if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                        // Interpolate color
                        let color = crate::core::Color {
                            r: colors[0].r * w0 + colors[1].r * w1 + colors[2].r * w2,
                            g: colors[0].g * w0 + colors[1].g * w1 + colors[2].g * w2,
                            b: colors[0].b * w0 + colors[1].b * w1 + colors[2].b * w2,
                        };

                        // Interpolate depth if needed (assuming you have vertex depths)
                        // For now just set a dummy depth
                        let depth = 0.0;

                        fragments.push(crate::pipeline::Fragment {
                            screen_pos: p,
                            depth,
                            color,
                        });
                    }
                }
            }
        }

        fragments
    }

    fn rasterize_fixed_point(
        &self,
        screen_verts: [glam::Vec2; 3],
        colors: [Color; 3],
    ) -> Vec<Fragment> {
        let mut fragments = Vec::new();
        let width = self.width;
        let height = self.height;

        let v0 = screen_verts[0];
        let v1 = screen_verts[1];
        let v2 = screen_verts[2];
        let area = edge_function(v0, v1, v2);

        if area <= 0 {
            return fragments; // Backface culling
        }

        let (min_x, max_x) = (
            v0.x.min(v1.x).min(v2.x).max(0.0) as i32,
            v0.x.max(v1.x).max(v2.x).min(width as f32) as i32,
        );
        let (min_y, max_y) = (
            v0.y.min(v1.y).min(v2.y).max(0.0) as i32,
            v0.y.max(v1.y).max(v2.y).min(height as f32) as i32,
        );

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = glam::Vec2::new(x as f32, y as f32);

                let w0 = edge_function(v1, v2, p);
                let w1 = edge_function(v2, v0, p);
                let w2 = edge_function(v0, v1, p);

                if w0 >= 0 && w1 >= 0 && w2 >= 0 {
                    let inv_area = 1.0 / (area as f32);

                    // Convert weights back to floating-point for interpolation
                    let w0f = w0 as f32 * inv_area;
                    let w1f = w1 as f32 * inv_area;
                    let w2f = w2 as f32 * inv_area;

                    let color = Color {
                        r: colors[0].r * w0f + colors[1].r * w1f + colors[2].r * w2f,
                        g: colors[0].g * w0f + colors[1].g * w1f + colors[2].g * w2f,
                        b: colors[0].b * w0f + colors[1].b * w1f + colors[2].b * w2f,
                    };

                    fragments.push(Fragment {
                        screen_pos: p,
                        depth: 0.0, // Implement depth interpolation here
                        color,
                    });
                }
            }
        }

        fragments
    }
}
pub fn edge_function(v0: glam::Vec2, v1: glam::Vec2, p: glam::Vec2) -> i32 {
    ((p.x - v0.x) as i32 * (v1.y - v0.y) as i32 - (p.y - v0.y) as i32 * (v1.x - v0.x) as i32) << 16
}

fn barycentric(
    p: glam::Vec2,
    v0: glam::Vec2,
    v1: glam::Vec2,
    v2: glam::Vec2,
) -> Option<(f32, f32, f32)> {
    let denom = (v1.y - v2.y) * (v0.x - v2.x) + (v2.x - v1.x) * (v0.y - v2.y);
    if denom.abs() < 1e-10 {
        // Degenerate triangle
        return None;
    }
    let w1 = ((v1.y - v2.y) * (p.x - v2.x) + (v2.x - v1.x) * (p.y - v2.y)) / denom;
    let w2 = ((v2.y - v0.y) * (p.x - v2.x) + (v0.x - v2.x) * (p.y - v2.y)) / denom;
    let w0 = 1.0 - w1 - w2;
    Some((w0, w1, w2))
}

pub fn bresenham<F>(start: glam::Vec2, end: glam::Vec2, p: crate::core::Pixel, mut plot: F)
where
    F: FnMut(glam::Vec2, f32, crate::core::Pixel),
{
    let mut x0 = start.x as i32;
    let mut y0 = start.y as i32;
    let x1 = end.x as i32;
    let y1 = end.y as i32;

    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let depth = 0.0; // You should probably compute actual depth, but for now just 0.0

    loop {
        plot(glam::Vec2::new(x0 as f32, y0 as f32), depth, p);

        if x0 == x1 && y0 == y1 {
            break;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Color;
    use std::time::Instant;

    #[test]
    fn benchmark_rasterization() {
        let iterations = 100;

        let rasterizer = Rasterizer::new(1920, 1080);
        let screen_verts = [
            Vec2::new(100.0, 100.0),
            Vec2::new(400.0, 150.0),
            Vec2::new(300.0, 300.0),
        ];
        let colors = [Color::RED, Color::GREEN, Color::BLUE];

        // Benchmark Float Rasterization
        let start_time = Instant::now();
        for _ in 0..iterations {
            let _ = rasterizer.rasterize_triangle_barycentric(screen_verts, colors);
        }
        let float_time = start_time.elapsed();

        // Benchmark Fixed-Point Rasterization
        let start_time = Instant::now();
        for _ in 0..iterations {
        }
        let fixed_time = start_time.elapsed();

        println!("Float Rasterization took: {:?}", float_time);
        println!("Average per iteration: {:?}", float_time / iterations);
        println!("Fixed-Point Rasterization took: {:?}", fixed_time);
        println!("Average per iteration: {:?}", fixed_time / iterations);
        println!(
            "Speed difference: {:.2}x",
            float_time.as_nanos() as f64 / fixed_time.as_nanos() as f64
        );
    }
}
