use std::sync::{Arc, Mutex};

use crate::core::{Color, RenderMode, Scene};
use crate::debug_print;
use crate::pipeline::{to_fixed, Fragment, ProcessedGeometry, FP_ONE, FP_SHIFT};
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

        // Separate environment geometry from regular geometry
        let (env_geo, scene_geo): (Vec<_>, Vec<_>) =
            geometry.iter().partition(|geo| geo.entity_id == usize::MAX); // what the hell is that second check?

        // Process environment first (it should be behind everything)
        let mut env_fragments: Vec<_> = env_geo
            .par_iter()
            .flat_map(|geo: &&ProcessedGeometry| self.process_environment_triangles(geo))
            .collect();

        // Then process regular scene geometry
        let scene_fragments: Vec<_> = scene_geo
            .par_iter()
            .flat_map(|geo: &&ProcessedGeometry| {
                let id = geo.entity_id;
                self.process_mesh_triangles(geo, scene.entities[id].render_mode())
            })
            .collect();

        // Combine fragments, environment first
        frags.append(&mut env_fragments);
        frags.extend(scene_fragments);

        debug_print!("Generated {} fragments", frags.len());
    }

    fn process_environment_triangles(&self, geo: &ProcessedGeometry) -> Vec<Fragment> {
        let vertices = [
            geo.vertices[0].position,
            geo.vertices[1].position,
            geo.vertices[2].position,
        ];

        let screen_verts = self.project_to_screen(&vertices);
        let colors = [
            geo.vertices[0].color,
            geo.vertices[1].color,
            geo.vertices[2].color,
        ];

        // Use maximum depth for environment to ensure it's behind everything
        self.rasterize_environment_triangle(screen_verts, colors, &vertices)
    }

    fn rasterize_environment_triangle(
        &self,
        screen_verts: [Vec2; 3],
        colors: [Color; 3],
        clip_verts: &[Vec4; 3],
    ) -> Vec<Fragment> {
        let mut fragments = Vec::new();
        let mut bbox_min = Vec2::new(self.width as f32 - 1.0, self.height as f32 - 1.0);
        let mut bbox_max = Vec2::new(0.0, 0.0);

        for v in &screen_verts {
            bbox_min.x = bbox_min.x.min(v.x);
            bbox_min.y = bbox_min.y.min(v.y);
            bbox_max.x = bbox_max.x.max(v.x);
            bbox_max.y = bbox_max.y.max(v.y);
        }

        bbox_min.x = bbox_min.x.max(0.0);
        bbox_min.y = bbox_min.y.max(0.0);
        bbox_max.x = bbox_max.x.min((self.width - 1) as f32);
        bbox_max.y = bbox_max.y.min((self.height - 1) as f32);

        let (v0, v1, v2) = (screen_verts[0], screen_verts[1], screen_verts[2]);

        // Environment-specific depth handling
        let w0 = clip_verts[0].w;
        let w1 = clip_verts[1].w;
        let w2 = clip_verts[2].w;

        for y in bbox_min.y as i32..=bbox_max.y as i32 {
            for x in bbox_min.x as i32..=bbox_max.x as i32 {
                let p = Vec2::new(x as f32, y as f32);
                if let Some((b0, b1, b2)) = barycentric(p, v0, v1, v2) {
                    if b0 >= 0.0 && b1 >= 0.0 && b2 >= 0.0 {
                        // Use perspective-correct interpolation for colors
                        let w = 1.0 / (b0 / w0 + b1 / w1 + b2 / w2);
                        let b0_c = (b0 / w0) * w;
                        let b1_c = (b1 / w1) * w;
                        let b2_c = (b2 / w2) * w;

                        // Environment always uses maximum depth
                        let depth = 0.99; // Just before the far plane

                        let color = Color {
                            r: colors[0].r * b0_c + colors[1].r * b1_c + colors[2].r * b2_c,
                            g: colors[0].g * b0_c + colors[1].g * b1_c + colors[2].g * b2_c,
                            b: colors[0].b * b0_c + colors[1].b * b1_c + colors[2].b * b2_c,
                        };

                        fragments.push(Fragment {
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

    fn process_mesh_triangles(
        &self,
        geo: &ProcessedGeometry,
        mode: Arc<Mutex<RenderMode>>,
    ) -> Vec<Fragment> {
        let vertices = [
            geo.vertices[0].position,
            geo.vertices[1].position,
            geo.vertices[2].position,
        ];

        // Get the render mode ONCE and release it's lock immediately
        let render_mode = *mode.lock().unwrap();

        let screen_verts = self.project_to_screen(&vertices);
        let colors = [
            geo.vertices[0].color,
            geo.vertices[1].color,
            geo.vertices[2].color,
        ];

        // MATCHING RENDER MODE TO DETERMINE HOW TO DRAW

        match render_mode {
            RenderMode::Solid => {
                self.rasterize_triangle_barycentric(screen_verts, colors, &vertices)
            }
            RenderMode::FixedPoint => self.rasterize_fixed_point(screen_verts, colors, &vertices),
            RenderMode::Wireframe => self.rasterize_triangle(&screen_verts, &colors),
        }
    }

    fn transform_vertices(
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

    fn vertex_colors(
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
        clip_verts: &[Vec4; 3], // Add this parameter
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

        // Get z and w values for perspective-correct interpolation
        let w0 = clip_verts[0].w;
        let w1 = clip_verts[1].w;
        let w2 = clip_verts[2].w;

        let z0 = clip_verts[0].z / w0;
        let z1 = clip_verts[1].z / w1;
        let z2 = clip_verts[2].z / w2;

        for y in bbox_min.y as i32..=bbox_max.y as i32 {
            for x in bbox_min.x as i32..=bbox_max.x as i32 {
                let p = glam::Vec2::new(x as f32, y as f32);
                if let Some((b0, b1, b2)) = barycentric(p, v0, v1, v2) {
                    if b0 >= 0.0 && b1 >= 0.0 && b2 >= 0.0 {
                        // Perspective-correct interpolation
                        let w = 1.0 / (b0 / w0 + b1 / w1 + b2 / w2);
                        let b0_c = (b0 / w0) * w;
                        let b1_c = (b1 / w1) * w;
                        let b2_c = (b2 / w2) * w;

                        // Interpolate z
                        let mut depth = z0 * b0_c + z1 * b1_c + z2 * b2_c; // In [-1, 1] range
                        depth = (depth + 1.0) * 0.5; // In [0, 1] range
                        depth = depth.clamp(0.1, 1.0);

                        // Interpolate color
                        let color = crate::core::Color {
                            r: colors[0].r * b0_c + colors[1].r * b1_c + colors[2].r * b2_c,
                            g: colors[0].g * b0_c + colors[1].g * b1_c + colors[2].g * b2_c,
                            b: colors[0].b * b0_c + colors[1].b * b1_c + colors[2].b * b2_c,
                        };

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

    /// This was purely implemented for fun, no real use case
    // BUG: will panic on overflow, need to handle that
    fn rasterize_fixed_point(
        &self,
        screen_verts: [glam::Vec2; 3],
        colors: [Color; 3],
        clip_verts: &[Vec4; 3],
    ) -> Vec<Fragment> {
        let mut fragments = Vec::new();

        // 1) Precompute reciprocal of w for each vertex (in float).
        //    (PS2-era hardware often used fixed-point for these too, but
        //     let's keep it simpler by storing them as floats.)
        let w0_recip = 1.0 / clip_verts[0].w;
        let w1_recip = 1.0 / clip_verts[1].w;
        let w2_recip = 1.0 / clip_verts[2].w;

        // Likewise, if you want the final depth in [0,1], you might store z/w:
        let z0_ndc = clip_verts[0].z * w0_recip;
        let z1_ndc = clip_verts[1].z * w1_recip;
        let z2_ndc = clip_verts[2].z * w2_recip;

        // 2) Convert screen-space coords to fixed point
        let v0 = (to_fixed(screen_verts[0].x), to_fixed(screen_verts[0].y));
        let v1 = (to_fixed(screen_verts[1].x), to_fixed(screen_verts[1].y));
        let v2 = (to_fixed(screen_verts[2].x), to_fixed(screen_verts[2].y));

        // 3) Triangle area in fixed point
        let area = edge_function_fixed(v0, v1, v2);
        if area <= 0 {
            return fragments; // Degenerate or back-facing
        }

        // 4) Compute bounding box in integer screen coords
        let min_x = (v0.0.min(v1.0).min(v2.0) >> FP_SHIFT).max(0);
        let max_x = (v0.0.max(v1.0).max(v2.0) >> FP_SHIFT).min(self.width as i32 - 1);
        let min_y = (v0.1.min(v1.1).min(v2.1) >> FP_SHIFT).max(0);
        let max_y = (v0.1.max(v1.1).max(v2.1) >> FP_SHIFT).min(self.height as i32 - 1);

        // 5) We'll still do the “shifting by (FP_SHIFT - 4)” trick for partial subpixel coverage,
        //    plus a float area scale to keep barycentric weighting in a manageable range
        let inv_area = FP_ONE as f32 / (area >> FP_SHIFT) as f32;

        // We'll collect each scanline in parallel (same as you had)
        let scanlines: Vec<_> = (min_y..=max_y).collect();
        let fragment_lists: Vec<_> = scanlines
            .par_iter()
            .map(|&y| {
                let mut line_fragments = Vec::new();
                let y_fixed = y << FP_SHIFT;

                for x in min_x..=max_x {
                    let x_fixed = x << FP_SHIFT;

                    // Edge function in fixed point
                    let w0 = edge_function_fixed(v1, v2, (x_fixed, y_fixed));
                    let w1 = edge_function_fixed(v2, v0, (x_fixed, y_fixed));
                    let w2 = edge_function_fixed(v0, v1, (x_fixed, y_fixed));

                    if w0 >= 0 && w1 >= 0 && w2 >= 0 {
                        // Convert to float, including subpixel shift:
                        let w0f = (w0 >> (FP_SHIFT - 4)) as f32 * inv_area;
                        let w1f = (w1 >> (FP_SHIFT - 4)) as f32 * inv_area;
                        let w2f = (w2 >> (FP_SHIFT - 4)) as f32 * inv_area;

                        // 6) **Perspective**: multiply each w#f by the vertex's reciprocal-w
                        //    so that far-away vertices contribute less to the final color.
                        let p0 = w0f * w0_recip;
                        let p1 = w1f * w1_recip;
                        let p2 = w2f * w2_recip;

                        let sum = p0 + p1 + p2;
                        if sum < 1e-8 {
                            // near-degenerate, skip
                            continue;
                        }
                        // normalized barycentric
                        let b0_c = p0 / sum;
                        let b1_c = p1 / sum;
                        let b2_c = p2 / sum;

                        // 7) Interpolate color with some “retro” quantization
                        //    (You could do your * 32.0 .floor() / 32.0 thing here)
                        let mut r = colors[0].r * b0_c + colors[1].r * b1_c + colors[2].r * b2_c;
                        let mut g = colors[0].g * b0_c + colors[1].g * b1_c + colors[2].g * b2_c;
                        let mut b = colors[0].b * b0_c + colors[1].b * b1_c + colors[2].b * b2_c;

                        // e.g. use *32.0 if you like heavier banding
                        r = (r * 32.0).floor() / 32.0;
                        g = (g * 32.0).floor() / 32.0;
                        b = (b * 32.0).floor() / 32.0;

                        // 8) Interpolate depth from the z/w values
                        let mut depth = z0_ndc * b0_c + z1_ndc * b1_c + z2_ndc * b2_c;
                        // Transform NDC z in [-1,1] to [0,1]
                        depth = (depth + 1.0) * 0.5;
                        depth = depth.clamp(0.0, 1.0);

                        line_fragments.push(Fragment {
                            screen_pos: glam::Vec2::new(x as f32, y as f32),
                            depth,
                            color: Color { r, g, b },
                        });
                    }
                }
                line_fragments
            })
            .collect();

        // 9) Combine all fragments
        for mut line_fragments in fragment_lists {
            fragments.append(&mut line_fragments);
        }

        fragments
    }
}

/// Same edge function you already have, but left as is.
#[inline(always)]
fn edge_function_fixed(v0: (i32, i32), v1: (i32, i32), p: (i32, i32)) -> i32 {
    let dx = v1.0 - v0.0;
    let dy = v1.1 - v0.1;
    ((p.0 - v0.0) * dy - (p.1 - v0.1) * dx) >> (FP_SHIFT - 4)
}
/// Barycentric coordinates for a point in a triangle
/// Returns None if the triangle is degenerate
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

///  Bresenham's line algorithm
/// https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm
/// 
/// This function is used to draw lines in the rasterizer
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
