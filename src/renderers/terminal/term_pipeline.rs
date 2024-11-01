use std::cell::RefCell;
use std::fmt::Write;

use crate::core::ProjectedVertex;
use crate::core::{camera::Camera, scene::Scene};
use crate::renderers::renderer::{get_render_mode, Renderer};
use crate::renderers::terminal::termbuffer::{Pixel, TermBuffer};
use crossterm::terminal;
use glam::Vec4Swizzles;
use glam::{UVec2, Vec2};

pub struct TermPipeline {
    // TODO: Add a backbuffer/double renderering
    pub frontbuffer: RefCell<TermBuffer>,
}

impl Renderer for TermPipeline {
    type PixelType = Pixel;

    fn init(&mut self, width: usize, height: usize) {
        self.frontbuffer = RefCell::new(TermBuffer::new(width, height));
    }

    fn render_frame(&mut self, cam: &Camera, scene: &Scene) {
        let (width, height) = terminal::size().unwrap();
        {
            let mut fbuf = self.frontbuffer.borrow_mut();
            fbuf.clear();
            fbuf.height = height as usize;
            fbuf.width = width as usize;
        }
        let screen_dims = UVec2::new(width as u32, height as u32);
        let _r_mode = get_render_mode();
        let _view_proj = cam.get_view_projection_matrix();
        for entity in &scene.entities {
            let model_mat = entity.transform.model_mat();
            entity
                .mesh
                .update_projected_vertices(&model_mat, &screen_dims, &cam);

            let proj_verts = entity.mesh.projected_verts.borrow();
            for tri in entity.mesh.indices.chunks(3) {
                if let &[i1, i2, i3] = tri {
                    let v1 = &proj_verts[i1 as usize];
                    let v2 = &proj_verts[i2 as usize];
                    let v3 = &proj_verts[i3 as usize];
                    // HACK: need to have projected vertices store color
                    self.draw_line(v1, v2, crate::core::Color::RED);
                    self.draw_line(v2, v3, crate::core::Color::GREEN);
                    self.draw_line(v3, v1, crate::core::Color::BLUE);
                }
            }
        }
        // Render the final buffer
        self.frontbuffer.borrow_mut().render_to_terminal().unwrap();
    }

    fn update_res(&mut self, width: usize, height: usize) {}
}

impl TermPipeline {
    fn draw_line(
        &mut self,
        start: &ProjectedVertex,
        end: &ProjectedVertex,
        color: crate::core::Color,
    ) {
        let dx = end.pos.x - start.pos.x;
        let dy = end.pos.y - start.pos.y;
        let steps = dx.abs().max(dy.abs()) as i32;

        if steps == 0 {
            return;
        }

        let x_inc = dx / steps as f32;
        let y_inc = dy / steps as f32;

        // Interpolate depth along the line
        let depth_inc = (end.depth - start.depth) / steps as f32;

        let mut x = start.pos.x;
        let mut y = start.pos.y;
        let mut depth = start.depth;

        for _ in 0..=steps {
            let current_vertex = ProjectedVertex {
                pos: Vec2::new(x, y),
                depth,
            };

            self.frontbuffer.borrow_mut().set_pixel(
                x as usize,
                y as usize,
                &current_vertex,
                Pixel::new_full(color),
            );

            x += x_inc;
            y += y_inc;
            depth += depth_inc;
        }
    }
    pub fn render_frame(&mut self, scene: &Scene, camera: &Camera) -> std::io::Result<()> {
        <Self as Renderer>::render_frame(self, camera, scene);
        Ok(())
    }
    pub fn new(width: usize, height: usize) -> Self {
        TermPipeline {
            frontbuffer: RefCell::new(TermBuffer::new(width, height)),
        }
    }
}
