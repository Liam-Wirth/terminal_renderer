use std::sync::{Arc, Mutex};

use crate::core::{Color, RenderMode, Scene};
use crate::debug_print;
use crate::geometry::Material;
use crate::pipeline::{to_fixed, Fragment, ProcessedGeometry, FP_ONE, FP_SHIFT};
use glam::{Mat4, Vec2, Vec3, Vec4};
use rayon::prelude::*;

use super::ClipVertex;

pub struct Rasterizer {
    width: usize,
    height: usize,
}

#[derive(Clone, Copy)]
struct LineVertex {
    screen_pos: Vec2,
    color: Color,
    uv: Vec2,
    inv_w: f32,
    z_over_w: f32,
    uv_over_w: Vec2,
}

impl LineVertex {
    fn new(screen_pos: Vec2, clip: &ClipVertex) -> Self {
        let inv_w = 1.0 / clip.position.w;
        Self {
            screen_pos,
            color: clip.color,
            uv: clip.uv,
            inv_w,
            z_over_w: clip.position.z * inv_w,
            uv_over_w: clip.uv * inv_w,
        }
    }
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

        // Then process regular scene geometry
        let scene_fragments: Vec<_> = geometry
            .par_iter()
            .flat_map(|geo: &ProcessedGeometry| {
                let id = geo.entity_id;
                self.process_mesh_triangles(geo, scene.entities[id].render_mode(), &scene)
            })
            .collect();

        frags.extend(scene_fragments);

        debug_print!("Generated {} fragments", frags.len());
    }

    fn process_mesh_triangles(
        &self,
        geo: &ProcessedGeometry,
        mode: Arc<Mutex<RenderMode>>,
        scene: &Scene,
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
        // HACK:  This is fucked
        let world_pos = [
            scene.entities[geo.entity_id].mesh.vertices[geo.world_pos[0]].pos,
            scene.entities[geo.entity_id].mesh.vertices[geo.world_pos[1]].pos,
            scene.entities[geo.entity_id].mesh.vertices[geo.world_pos[2]].pos,
        ];

        let normals = {
            let normal_buffer = scene.entities[geo.entity_id].mesh.normals.lock().unwrap();
            [
                normal_buffer[geo.world_pos[0]],
                normal_buffer[geo.world_pos[1]],
                normal_buffer[geo.world_pos[2]],
            ]
        };
        let material = match geo.material_id {
            Some(mat_id) if mat_id < scene.entities[geo.entity_id].mesh.materials.len() => {
                &scene.entities[geo.entity_id].mesh.materials[mat_id]
            }
            _ => &Material::default(),
        };

        match render_mode {
            RenderMode::Solid => {
                // TODO: cleanup this method signature from hell
                // self.rasterize_triangle_barycentric(screen_verts, colors, &vertices)
                self.rasterize_triangle_barycentric_2(
                    screen_verts,
                    &geo.vertices,
                    &world_pos,
                    &normals,
                    material, // TODO: fix this as well. gonna leave all these bugs in cause I just wanna see shading damnit
                    geo.material_id.map(|mat_id| (geo.entity_id, mat_id)),
                )
            }
            RenderMode::FixedPoint => self.rasterize_fixed_point(screen_verts, colors, &vertices),
            RenderMode::Wireframe => {
                self.rasterize_triangle_wireframe(screen_verts, &geo.vertices, material)
            }
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
                (1.0 - ndc.y) * 0.5 * self.height as f32, // NOTE: FLIPPED THIS FROM (ndc.y +
                                                          // 1.0)
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

    // Wireframe rasterization with depth + material/texture sampling
    fn rasterize_triangle_wireframe(
        &self,
        vertices: [Vec2; 3],
        clip_verts: &[ClipVertex; 3],
        material: &Material,
    ) -> Vec<Fragment> {
        let mut fragments = Vec::new();
        let line_verts = [
            LineVertex::new(vertices[0], &clip_verts[0]),
            LineVertex::new(vertices[1], &clip_verts[1]),
            LineVertex::new(vertices[2], &clip_verts[2]),
        ];

        for i in 0..3 {
            let next = (i + 1) % 3;
            self.draw_line(
                &line_verts[i],
                &line_verts[next],
                material,
                &mut fragments,
                2,
            );
        }
        fragments
    }

    // Bresenham's line algorithm with material-based sampling and depth interpolation
    fn draw_line(
        &self,
        start: &LineVertex,
        end: &LineVertex,
        material: &Material,
        fragments: &mut Vec<Fragment>,
        width: usize,
    ) {
        let mut steep = false;

        let mut x0 = start.screen_pos.x as i32;
        let mut y0 = start.screen_pos.y as i32;
        let mut x1 = end.screen_pos.x as i32;
        let mut y1 = end.screen_pos.y as i32;
        let mut start_v = *start;
        let mut end_v = *end;

        if (x0 - x1).abs() < (y0 - y1).abs() {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
            steep = true;
        }

        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
            std::mem::swap(&mut start_v, &mut end_v);
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
                0.0
            };

            let inv_w = start_v.inv_w + (end_v.inv_w - start_v.inv_w) * t;
            let z_over_w = start_v.z_over_w + (end_v.z_over_w - start_v.z_over_w) * t;
            let mut depth = (z_over_w + 1.0) * 0.5;
            depth = depth.clamp(0.0, 1.0);

            let uv_over_w = start_v.uv_over_w + (end_v.uv_over_w - start_v.uv_over_w) * t;
            let uv = if inv_w.abs() > f32::EPSILON {
                uv_over_w / inv_w
            } else {
                Vec2::ZERO
            };

            let color = if material.diffuse_texture_data.is_some() || material.diffuse.is_some() {
                material.sample_diffuse(uv)
            } else {
                start_v.color.lerp(&end_v.color, t)
            };

            // Only add fragments within screen bounds
            if pos.x >= 0.0
                && pos.x < self.width as f32
                && pos.y >= 0.0
                && pos.y < self.height as f32
            {
                // NOTE: Might need to find a way to override here. Probably best move is on wireframe toggle disable any lighting
                fragments.push(Fragment {
                    screen_pos: pos,
                    depth,
                    albedo: color,
                    normal: Vec3::ZERO,
                    uv,
                    ..Default::default()
                });
            }

            error2 += derror2;
            if error2 > dx {
                y += if y1 > y0 { 1 } else { -1 };
                error2 -= dx * 2;
            }
        }
    }

    // Writing it as a new function temporarily to A-B Compare and see if there is a noticeable difference

    // OPT: This function does generate a new fragment buffer on every run, in a future refactor pass a mutable reference to the old fragment buffer, and edit in place
    // FIXME: Clean up function signature, wayyyy to many arguments
    // TODO: write a test to compare runtimes of this and the original function
    fn rasterize_triangle_barycentric_2(
        &self,
        screen_verts: [glam::Vec2; 3],
        clip_verts: &[ClipVertex; 3],
        world_pos: &[Vec3; 3],
        normals: &[Vec3; 3],
        material: &Material,
        mat_id: Option<(usize, usize)>,
    ) -> Vec<Fragment> {
        let mut frags = Vec::new();
        let mut bbox_min = Vec2::new(self.width as f32 - 1.0, self.height as f32 - 1.0);
        let mut bbox_max = Vec2::new(0.0, 0.0);
        for v in &screen_verts {
            bbox_min = bbox_min.min(*v);
            bbox_max = bbox_max.max(*v);
        }
        // Clamp bounding box to screen
        bbox_min = bbox_min.max(Vec2::ZERO);
        bbox_max = Vec2::new(
            bbox_max.x.min((self.width - 1) as f32),
            bbox_max.y.min((self.height - 1) as f32),
        );
        let (v0, v1, v2) = (screen_verts[0], screen_verts[1], screen_verts[2]);

        //precompute perspective correction factors (w)
        let w0 = clip_verts[0].position.w;
        let w1 = clip_verts[1].position.w;
        let w2 = clip_verts[2].position.w;

        // precompute their inverses for perspective correction
        let inv_w0 = 1.0 / w0;
        let inv_w1 = 1.0 / w1;
        let inv_w2 = 1.0 / w2;

        // precompute z/w for depth interp
        let z0 = clip_verts[0].position.z * inv_w0;
        let z1 = clip_verts[1].position.z * inv_w1;
        let z2 = clip_verts[2].position.z * inv_w2;

        // precompute attribute/w for perspective correction
        let norm0 = normals[0] * inv_w0;
        let norm1 = normals[1] * inv_w1;
        let norm2 = normals[2] * inv_w2;

        let pos0 = world_pos[0] * inv_w0;
        let pos1 = world_pos[1] * inv_w1;
        let pos2 = world_pos[2] * inv_w2;

        // scan bounding box
        for y in bbox_min.y as i32..=bbox_max.y as i32 {
            for x in bbox_min.x as i32..=bbox_max.x as i32 {
                let p = Vec2::new(x as f32, y as f32);
                if let Some((b0, b1, b2)) = barycentric(p, v0, v1, v2) {
                    if b0 >= 0.0 && b1 >= 0.0 && b2 >= 0. {
                        let persp_w = 1.0 / (b0 * inv_w0 + b1 * inv_w1 + b2 * inv_w2);

                        let b0_c = (b0 * inv_w0) * persp_w;
                        let b1_c = (b1 * inv_w1) * persp_w;
                        let b2_c = (b2 * inv_w2) * persp_w;

                        //depth interpolation
                        let mut depth = z0 * b0_c + z1 * b1_c + z2 * b2_c;
                        depth = (depth + 1.0) * 0.5; // In [0, 1] range
                        depth = depth.clamp(0.0, 1.0); // dunno why tbh

                        // world pos interpolation

                        let world_pos = (pos0 * b0_c + pos1 * b1_c + pos2 * b2_c) * persp_w;

                        //interp normalize normal
                        // let normal = (norm0 * b0_c + norm1 * b1_c + norm2 * b2_c) * persp_w;
                        let normal = norm0 * b0_c + norm1 * b1_c + norm2 * b2_c;
                        let normal = normal.normalize();

                        // Interpolate UV coordinates first
                        let uv0 = clip_verts[0].uv;
                        let uv1 = clip_verts[1].uv;
                        let uv2 = clip_verts[2].uv;
                        let uv = uv0 * b0_c + uv1 * b1_c + uv2 * b2_c;

                        // Sample material properties using UV coordinates
                        let albedo = material.sample_diffuse(uv);
                        let specular = material.sample_specular(uv);
                        let shininess = material.shininess.unwrap_or(0.0);

                        // DEBUG: Check if we're actually getting texture data
                        if material.diffuse_texture_data.is_some() {
                            // If texture loading failed, use bright magenta as fallback
                            if albedo == Color::WHITE && material.diffuse.is_none() {
                                frags.push(Fragment {
                                    screen_pos: p,
                                    depth,
                                    albedo: Color::new(1.0, 0.0, 1.0), // Bright magenta
                                    normal,
                                    specular,
                                    shininess,
                                    dissolve: material.dissolve.unwrap_or(1.0),
                                    uv,
                                    mat_id,
                                });
                                continue;
                            }
                        } else if material.diffuse_texture.is_some() {
                            // Texture should exist but wasn't loaded - use bright cyan
                            frags.push(Fragment {
                                screen_pos: p,
                                depth,
                                albedo: Color::new(0.0, 1.0, 1.0), // Bright cyan
                                normal,
                                specular,
                                shininess,
                                dissolve: material.dissolve.unwrap_or(1.0),
                                uv,
                                mat_id,
                            });
                            continue;
                        }

                        frags.push(Fragment {
                            screen_pos: p,
                            depth,
                            albedo,
                            normal,
                            specular,
                            shininess,
                            dissolve: material.dissolve.unwrap_or(1.0),
                            uv,
                            mat_id,
                        });
                    }
                }
            }
        }
        frags
    }

    fn rasterize_triangle_barycentric(
        &self,
        screen_verts: [glam::Vec2; 3],
        clip_verts: &[ClipVertex; 3],
        normals: &[Vec3; 3], // Will optionally compute face normal dependent on lighting model
        material: &Material,
        mat_id: Option<(usize, usize)>,
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
        let w0 = clip_verts[0].position.w;
        let w1 = clip_verts[1].position.w;
        let w2 = clip_verts[2].position.w;

        let z0 = clip_verts[0].position.z / w0;
        let z1 = clip_verts[1].position.z / w1;
        let z2 = clip_verts[2].position.z / w2;

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
                        depth = depth.clamp(0.0, 1.0);

                        let interp_normal =
                            normals[0] * b0_c + normals[1] * b1_c + normals[2] * b2_c; // this is important I've learned
                        let normal = interp_normal.normalize(); // Get it back into 0-1 range

                        // Interpolate UV coordinates first
                        let uv0 = clip_verts[0].uv;
                        let uv1 = clip_verts[1].uv;
                        let uv2 = clip_verts[2].uv;
                        let uv = uv0 * b0_c + uv1 * b1_c + uv2 * b2_c;

                        // Sample material properties using UV coordinates
                        let diff = material.sample_diffuse(uv);
                        let specular = material.sample_specular(uv);

                        fragments.push(crate::pipeline::Fragment {
                            screen_pos: p,
                            depth,
                            albedo: diff,
                            normal,
                            specular,
                            shininess: material.shininess.unwrap_or(0.),
                            dissolve: material.dissolve.unwrap_or(0.),
                            uv,
                            mat_id: mat_id,
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
                            albedo: Color { r, g, b },
                            normal: Vec3::ZERO,
                            uv: Vec2::ZERO,
                            ..Default::default()
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
    let total_area = edge_function(&v0, &v1, &v2); // Returns 2x the signed area
                                                   // Early exit for a degenerate tri:
    if total_area.abs() < 1e-10 {
        return None;
    }

    let w0 = edge_function(&v1, &v2, &p); // Returns 2x A_1
    let w1 = edge_function(&v2, &v0, &p); // Returns 2x A_2
    let w2 = edge_function(&v0, &v1, &p); // returns 2x A_3

    let include_edge = |w: f32, start: &Vec2, end: &Vec2| -> bool {
        if w.abs() < 1e-10 {
            // on the edge
            if start.y == end.y {
                // horizontal edge
                start.y > end.y
            } else {
                start.x > end.x
            }
        } else {
            w >= 0.0 // Include if we're inside
        }
    };

    if include_edge(w0, &v1, &v2) && include_edge(w1, &v2, &v0) && include_edge(w2, &v0, &v1) {
        // convert areas to weights
        let b0 = w0 / total_area;
        let b1 = w1 / total_area;
        let b2 = w2 / total_area;
        Some((b0, b1, b2))
    } else {
        None
    }
}

// https://www.scratchapixel.com/lessons/3d-basic-rendering/rasterization-practical-implementation/rasterization-stage.html
// https://www.cs.drexel.edu/~deb39/Classes/Papers/comp175-06-pineda.pdf
// Based on pineda's edge function to determine if a point is inside a triangle
// NOTE: Could also use this for wireframes by just discarding everything pixel that doesnt return 0
/// Returns Some int N
/// 'N' > 0 if the point is to the right of the line
/// 'N' == 0 if the point is on the line
/// 'N' < 0 if the point is to the left of the line
fn edge_function(a: &Vec2, b: &Vec2, c: &Vec2) -> f32 {
    // NOTE TO SELF this can also be represented as the magnitude of the cross products between (V1 - V0) and (P- V0)
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}

fn is_on_edge(v0: &Vec2, v1: &Vec2, v2: &Vec2, p: &Vec2) -> bool {
    let mut on_edge = false;
    // NOTE: |= is a BitOrAssign operator. So it's doing the bitwise OR operation
    // Basically just does the or operator, and then assigns the result
    on_edge |= edge_function(v0, v1, p).abs() < 1e-10;
    on_edge |= edge_function(v1, v2, p).abs() < 1e-10;
    on_edge |= edge_function(v2, v0, p).abs() < 1e-10;
    on_edge
}

fn inside(v0: &Vec2, v1: &Vec2, v2: &Vec2, p: &Vec2) -> bool {
    let mut inside = true;
    inside &= edge_function(v0, v1, p) >= 0.0;
    inside &= edge_function(v1, v2, p) >= 0.0;
    inside &= edge_function(v2, v0, p) >= 0.0;
    inside
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
