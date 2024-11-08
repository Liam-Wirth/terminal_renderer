use std::cell::RefCell;

use crate::core::{camera::Camera, scene::Scene};
use crate::core::{Color, ProjectedVertex};
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
        let _r_mode = get_render_mode();
        let _view_proj = cam.get_view_projection_matrix();

        // Render to back buffer
        for entity in &scene.entities {
            let model_mat = entity.transform.model_mat();
            entity
                .mesh
                .update_projected_vertices(&model_mat, &screen_dims, cam);

            let proj_verts = entity.mesh.projected_verts.borrow();
            for tri in entity.mesh.indices.chunks(3) {
                if let &[i1, i2, i3] = tri {
                    let v1 = &proj_verts[i1 as usize];
                    let v2 = &proj_verts[i2 as usize];
                    let v3 = &proj_verts[i3 as usize];
                    self.draw_line_to_back(v1, v2, Color::RED);
                    self.draw_line_to_back(v2, v3, Color::GREEN);
                    self.draw_line_to_back(v3, v1, Color::BLUE);
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
        v0: &ProjectedVertex,
        v1: &ProjectedVertex,
        v2: &ProjectedVertex,
        pix: &Pixel,
    ) {
        // TODO: Eventually add functionality for coloring based on the vertex colors (ie the
        // hello world rgb triangle)

        // Find some way for the profiler to know whether or not this part of the function is
        // bottle-necking
        let (v0, v1, v2) = if v0.pos.y <= v1.pos.y && v0.pos.y <= v2.pos.y {
            if v1.pos.y <= v2.pos.y {
                (v0, v1, v2)
            } else {
                (v1, v2, v0)
            }
        } else if v1.pos.y <= v0.pos.y && v1.pos.y <= v2.pos.y {
            if v0.pos.y <= v2.pos.y {
                (v1, v0, v2)
            } else {
                (v1, v2, v0)
            }
        } else if v0.pos.y <= v1.pos.y {
            (v2, v0, v1)
        } else {
            (v2, v1, v0)
        };
        if v1.pos.y == v2.pos.y {
            self.fill_flat_bot_tri(v0, v1, v2, pix);
        } else if v0.pos.y == v1.pos.y {
            self.fill_flat_top_tri(v0, v1, v2, pix);
        } else {
            let dy_v2_v0 = v2.pos.y - v0.pos.y;
            let dy_v1_v0 = v1.pos.y - v0.pos.y;
            let dx_v2_v0 = v2.pos.x - v0.pos.x;
            let v_split_x = v0.pos.x + dx_v2_v0 * (dy_v1_v0 / dy_v2_v0);
            let v_split_depth = v0.depth + (v2.depth - v0.depth) * (dy_v1_v0 / dy_v2_v0);

            let v_split = ProjectedVertex {
                pos: Vec2::new(v_split_x, v1.pos.y),
                depth: v_split_depth,
                color: pix.color,
            };
            self.fill_flat_bot_tri(v0, v1, &v_split, pix);
            self.fill_flat_top_tri(v0, v1, &v_split, pix);
        }
    }
    fn fill_flat_bot_tri(
        &mut self,
        v0: &ProjectedVertex,
        v1: &ProjectedVertex,
        v2: &ProjectedVertex,
        pix: &Pixel,
    ) {
        let dy_v1_v0 = (v1.pos.y - v0.pos.y).max(1.0);
        let dy_v2_v0 = (v2.pos.y - v0.pos.y).max(1.0);

        let inv_slope1 = (v1.pos.x - v0.pos.x) / dy_v1_v0;
        let inv_slope2 = (v2.pos.x - v0.pos.x) / dy_v2_v0;

        let depth_slope1 = (v1.depth - v0.depth) / dy_v1_v0;
        let depth_slope2 = (v2.depth - v0.depth) / dy_v2_v0;

        let mut cur_x1 = v0.pos.x;
        let mut cur_x2 = v0.pos.x;
        let mut cur_depth1 = v0.depth;
        let mut cur_depth2 = v0.depth;

        for y in v0.pos.y as usize..=v1.pos.y as usize {}
        todo!();
    }
    fn fill_flat_top_tri(
        &mut self,
        v0: &ProjectedVertex,
        v1: &ProjectedVertex,
        v2: &ProjectedVertex,
        pix: &Pixel,
    ) {
        todo!();
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
