use std::cell::RefCell;

use crate::core::{camera::Camera, scene::Scene};
use crate::core::{Color, ProjectedVertex};
use crate::renderers::renderer::RenderMode;
use crate::renderers::renderer::{get_render_mode, Renderer};
use crate::renderers::terminal::termbuffer::{Pixel, TermBuffer};
use crossterm::terminal;
use glam::{UVec2, Vec2};

pub struct TermPipeline {
    // TODO: Add a backbuffer/double renderering
    pub frontbuffer: RefCell<TermBuffer>,
    pub backbuffer: RefCell<TermBuffer>,
}

impl Renderer for TermPipeline {
    type PixelType = Pixel;
    type MetricsType = String;

    fn init(&mut self, width: usize, height: usize) {
        self.frontbuffer = RefCell::new(TermBuffer::new(width, height));
    }

    fn render_frame(&mut self, cam: &Camera, scene: &Scene, metrics: &String) {
        let (width, height) = terminal::size().unwrap();
        {
            // Clear the back buffer for drawing
            let mut back = self.backbuffer.borrow_mut();
            back.clear();
            back.height = height as usize;
            back.width = width as usize;
        }

        let screen_dims = UVec2::new(width as u32, height as u32);

        // Render to back buffer
        for entity in &scene.entities {
            let model_mat = entity.transform.model_mat();
            entity
                .mesh
                .update_projected_vertices(&model_mat, &screen_dims, cam);

            let proj_verts = entity.mesh.projected_verts.borrow();
            for tri in entity.mesh.tris.iter() {
                if !tri.is_facing_cam(&cam.pos.borrow()) {
                    continue;
                }
                match get_render_mode() {
                    RenderMode::Wireframe => {
                        let v1 = &proj_verts[tri.indices[0] as usize];
                        let v2 = &proj_verts[tri.indices[1] as usize];
                        let v3 = &proj_verts[tri.indices[2] as usize];
                        self.draw_line_to_back(v1, v2, Color::RED);
                        self.draw_line_to_back(v2, v3, Color::GREEN);
                        self.draw_line_to_back(v3, v1, Color::BLUE);
                    }
                    RenderMode::Solid => {
                        let v1 = &proj_verts[tri.indices[0] as usize];
                        let v2 = &proj_verts[tri.indices[1] as usize];
                        let v3 = &proj_verts[tri.indices[2] as usize];
                        self.draw_filled_triangle_scan(*v1, *v2, *v3, Color::RED);
                        // TODO: add texture support
                        // TODO: Add face color support
                    }
                }
            }
        }
        // Display back buffer and swap
        self.backbuffer
            .borrow()
            .render_to_terminal(metrics)
            .unwrap();
        self.swap_buffers();
    }

    fn update_res(&mut self, width: usize, height: usize) {
        self.frontbuffer.borrow_mut().width = width;
        self.frontbuffer.borrow_mut().height = height;
        self.backbuffer.borrow_mut().width = width;
        self.backbuffer.borrow_mut().height = height;
    }
}

impl TermPipeline {
    // New method to swap buffers
    fn swap_buffers(&mut self) {
        std::mem::swap(
            &mut *self.frontbuffer.borrow_mut(),
            &mut *self.backbuffer.borrow_mut(),
        );
    }

    // Modified to draw to back buffer
    fn draw_line_to_back(&mut self, start: &ProjectedVertex, end: &ProjectedVertex, color: Color) {
        let dx = end.pos.x - start.pos.x;
        let dy = end.pos.y - start.pos.y;
        let steps = dx.abs().max(dy.abs()) as i32;

        if steps == 0 {
            return;
        }

        let x_inc = dx / steps as f32;
        let y_inc = dy / steps as f32;
        let depth_inc = (end.depth - start.depth) / steps as f32;

        let mut x = start.pos.x;
        let mut y = start.pos.y;
        let mut depth = start.depth;

        for _ in 0..=steps {
            self.backbuffer
                .borrow_mut()
                .set_pixel(x as usize, y as usize, &depth, color, '#');

            x += x_inc;
            y += y_inc;
            depth += depth_inc;
        }
    }
    fn draw_filled_triangle_scan(
        &mut self,
        mut v0: ProjectedVertex,
        mut v1: ProjectedVertex,
        mut v2: ProjectedVertex,
        color: Color, // Pass the solid color
    ) {
        // Sort vertices by y-coordinate ascending (v0.y <= v1.y <= v2.y)
        if v1.pos.y < v0.pos.y {
            std::mem::swap(&mut v0, &mut v1);
        }
        if v2.pos.y < v1.pos.y {
            std::mem::swap(&mut v1, &mut v2);
        }
        if v1.pos.y < v0.pos.y {
            std::mem::swap(&mut v0, &mut v1);
        }

        if (v1.pos.y - v0.pos.y).abs() < std::f32::EPSILON {
            // Flat-top triangle
            self.fill_flat_top_tri(&v0, &v1, &v2, color);
        } else if (v2.pos.y - v1.pos.y).abs() < std::f32::EPSILON {
            // Flat-bottom triangle
            self.fill_flat_bot_tri(&v0, &v1, &v2, color);
        } else {
            // General triangle; split it into a flat-bottom and flat-top triangle
            let alpha = (v1.pos.y - v0.pos.y) / (v2.pos.y - v0.pos.y);
            let v_split_pos = v0.pos + (v2.pos - v0.pos) * alpha;
            let v_split_depth = v0.depth + (v2.depth - v0.depth) * alpha;

            let v_split = ProjectedVertex {
                pos: v_split_pos,
                depth: v_split_depth,
                color,
            };

            self.fill_flat_bot_tri(&v0, &v1, &v_split, color);
            self.fill_flat_top_tri(&v1, &v_split, &v2, color);
        }
    }

    fn draw_scanline(&mut self, x1: f32, x2: f32, y: i32, depth1: f32, depth2: f32, color: Color) {
        let x_start = x1.min(x2).ceil() as i32;
        let x_end = x1.max(x2).floor() as i32;

        let delta_x = x_end - x_start;
        if delta_x == 0 {
            return;
        }

        let depth_slope = (depth2 - depth1) / delta_x as f32;

        let mut depth = depth1 + (x_start as f32 - x1) * depth_slope;

        for x in x_start..=x_end + 1 {
            if x >= 0
                && x < self.backbuffer.borrow().width as i32 + 1
                && y >= 0
                && y < self.backbuffer.borrow().height as i32 + 1
            {
                self.backbuffer
                    .borrow_mut()
                    .set_pixel(x as usize, y as usize, &depth, color, '█');
            }
            depth += depth_slope;
        }
    }

    fn fill_flat_top_tri(
        &mut self,
        v0: &ProjectedVertex,
        v1: &ProjectedVertex,
        v2: &ProjectedVertex,
        color: Color,
    ) {
        let dy1 = v2.pos.y - v0.pos.y;
        let dy2 = v2.pos.y - v1.pos.y;

        if dy1 == 0.0 || dy2 == 0.0 {
            return;
        }

        let inv_slope1 = (v2.pos.x - v0.pos.x) / dy1;
        let inv_slope2 = (v2.pos.x - v1.pos.x) / dy2;

        let depth_slope1 = (v2.depth - v0.depth) / dy1;
        let depth_slope2 = (v2.depth - v1.depth) / dy2;

        let mut cur_x1 = v2.pos.x;
        let mut cur_x2 = v2.pos.x;
        let mut cur_depth1 = v2.depth;
        let mut cur_depth2 = v2.depth;

        let y_start = v2.pos.y.floor() as i32;
        let y_end = v0.pos.y.floor() as i32;

        for y in (y_end..y_start).rev() {
            self.draw_scanline(cur_x1, cur_x2, y, cur_depth1, cur_depth2, color);

            cur_x1 -= inv_slope1;
            cur_x2 -= inv_slope2;
            cur_depth1 -= depth_slope1;
            cur_depth2 -= depth_slope2;
        }
    }

    fn fill_flat_bot_tri(
        &mut self,
        v0: &ProjectedVertex,
        v1: &ProjectedVertex,
        v2: &ProjectedVertex,
        color: Color,
    ) {
        let dy1 = v1.pos.y - v0.pos.y;
        let dy2 = v2.pos.y - v0.pos.y;

        if dy1 == 0.0 || dy2 == 0.0 {
            return;
        }

        let inv_slope1 = (v1.pos.x - v0.pos.x) / dy1;
        let inv_slope2 = (v2.pos.x - v0.pos.x) / dy2;

        let depth_slope1 = (v1.depth - v0.depth) / dy1;
        let depth_slope2 = (v2.depth - v0.depth) / dy2;

        let mut cur_x1 = v0.pos.x;
        let mut cur_x2 = v0.pos.x;
        let mut cur_depth1 = v0.depth;
        let mut cur_depth2 = v0.depth;

        let y_start = v0.pos.y.ceil() as i32;
        let y_end = v1.pos.y.ceil() as i32;

        for y in y_start..y_end {
            self.draw_scanline(cur_x1, cur_x2, y, cur_depth1, cur_depth2, color);

            cur_x1 += inv_slope1;
            cur_x2 += inv_slope2;
            cur_depth1 += depth_slope1;
            cur_depth2 += depth_slope2;
        }
    }

    pub fn render_frame(
        &mut self,
        scene: &Scene,
        camera: &Camera,
        metrics: &String,
    ) -> std::io::Result<()> {
        <Self as Renderer>::render_frame(self, camera, scene, metrics);
        Ok(())
    }
    pub fn new(width: usize, height: usize) -> Self {
        TermPipeline {
            frontbuffer: RefCell::new(TermBuffer::new(width, height)),
            backbuffer: RefCell::new(TermBuffer::new(width, height)),
        }
    }
}
