use std::{cell::RefCell, io};

use glam::{Mat4, Vec4};
use minifb::Window;

use crate::{
    core::{Color, Scene},
    debug_print,
    pipeline::{ClipTriangle, ClipVertex},
    Metrics,
};

use super::{buffer::Buffer, rasterizer::Rasterizer, Clipper, Fragment, ProcessedGeometry};

pub struct Pipeline<B: Buffer> {
    pub width: usize,
    pub height: usize,
    front_buffer: RefCell<B>,
    back_buffer: RefCell<B>,
    pub scene: Scene,
    geometry: RefCell<Vec<ProcessedGeometry>>,
    rasterizer: RefCell<Rasterizer>,
    clipper: RefCell<Clipper>, // Add this
    fragments: RefCell<Vec<Fragment>>,
    metrics: Metrics,
}

impl<B: Buffer> Pipeline<B> {
    pub fn front_buffer(&self) -> &RefCell<B> {
        &self.front_buffer
    }

    pub fn back_buffer(&self) -> &RefCell<B> {
        &self.back_buffer
    }

    pub fn geometry(&self) -> &RefCell<Vec<ProcessedGeometry>> {
        &self.geometry
    }

    pub fn rasterizer(&self) -> &RefCell<Rasterizer> {
        &self.rasterizer
    }

    pub fn fragments(&self) -> &RefCell<Vec<Fragment>> {
        &self.fragments
    }

    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}

impl<B: Buffer> Pipeline<B> {
    pub fn new(width: usize, height: usize, scene: Scene) -> Self {
        Self {
            width,
            height,
            front_buffer: RefCell::new(B::new(width, height)),
            back_buffer: RefCell::new(B::new(width, height)),
            scene,
            metrics: Metrics::new(),
            geometry: RefCell::new(Vec::with_capacity(1024)),
            rasterizer: RefCell::new(Rasterizer::new(width, height)),
            clipper: RefCell::new(Clipper::new()), // Add this
            fragments: RefCell::new(Vec::with_capacity(1024)),
        }
    }

    /// Consider this function to be like the function that gets run every frame, like the main loop
    pub fn render_frame(&self, window: Option<&mut Window>) -> io::Result<()> {
        self.back_buffer.borrow_mut().clear();

        // 1. Process vertices to clip space
        self.process_geometry();

        // 2. Clip triangles (already integrated in process_geometry)
        // The clipper operates during geometry processing

        // 3. Rasterize clipped triangles
        self.rasterize();

        // 4. Process fragments
        self.process_fragments(&self.fragments.borrow());

        // Present
        if let Some(window) = window {
            self.front_buffer.borrow().present_window(window)?;
        } else {
            self.front_buffer.borrow().present()?;
        }

        self.swap_buffers();
        Ok(())
    }

    pub fn process_geometry(&self) {
        let view_matrix = self.scene.camera.get_view_matrix();
        let projection_matrix = self.scene.camera.get_projection_matrix();

        // Update clipper with current frustum planes
        self.clipper
            .borrow_mut()
            .update_frustum_planes(&self.scene.camera.get_frustum_planes());

        self.geometry.borrow_mut().clear();
        debug_print!(
            "Processing geometry for {} entities",
            self.scene.entities.len()
        );

        for (i, entity) in self.scene.entities.iter().enumerate() {
            let model_matrix = Mat4::from(entity.transform);
            let mvp_matrix = projection_matrix * view_matrix * model_matrix;

            // Process each triangle
            for tri in &entity.mesh.tris {
                // Create clip vertices
                let clip_verts = [
                    ClipVertex {
                        position: mvp_matrix
                            * Vec4::from((entity.mesh.vertices[tri.vertices[0]].pos, 1.0)),
                        color: entity.mesh.vertices[tri.vertices[0]]
                            .color
                            .unwrap_or(Color::WHITE),
                    },
                    ClipVertex {
                        position: mvp_matrix
                            * Vec4::from((entity.mesh.vertices[tri.vertices[1]].pos, 1.0)),
                        color: entity.mesh.vertices[tri.vertices[1]]
                            .color
                            .unwrap_or(Color::WHITE),
                    },
                    ClipVertex {
                        position: mvp_matrix
                            * Vec4::from((entity.mesh.vertices[tri.vertices[2]].pos, 1.0)),
                        color: entity.mesh.vertices[tri.vertices[2]]
                            .color
                            .unwrap_or(Color::WHITE),
                    },
                ];

                let clip_triangle = ClipTriangle {
                    vertices: clip_verts,
                };

                // Clip the triangle
                let clipped_triangles = self.clipper.borrow().clip_triangle(&clip_triangle);

                // Add resulting triangles to geometry buffer
                for triangle in clipped_triangles {
                    self.geometry.borrow_mut().push(ProcessedGeometry {
                        transform: mvp_matrix,
                        entity_id: i,
                        vertices: triangle.vertices, // Add this field to ProcessedGeometry
                    });
                }
            }
        }
    }

    pub fn rasterize(&self) {
        self.rasterizer.borrow_mut().rasterize(
            &self.geometry.borrow(),
            &self.scene,
            &mut self.fragments.borrow_mut(),
        );
    }
    pub fn process_fragments(&self, fragments: &[Fragment]) {
        let mut buffer = self.back_buffer.borrow_mut();
        for fragment in fragments {
            let pixel = B::create_pixel(fragment.color);
            let pos = (
                fragment.screen_pos.x as usize,
                fragment.screen_pos.y as usize,
            );
            buffer.set_pixel(pos, &fragment.depth, pixel);
        }
    }

    pub fn swap_buffers(&self) {
        std::mem::swap(
            &mut *self.front_buffer.borrow_mut(),
            &mut *self.back_buffer.borrow_mut(),
        );
    }

    pub fn update_metrics(&mut self, frame_delta: std::time::Duration) {
        self.metrics.update(frame_delta);
    }
    pub fn get_front_buffer(&self) -> &RefCell<B> {
        &self.front_buffer
    }
    pub fn get_back_buffer(&self) -> &RefCell<B> {
        &self.back_buffer
    }

    pub fn window_handle_input(&mut self, input: &minifb::Window) {
        let delta = 0.1;
        // Base speeds
        let move_speed = 2.0; // Units per second
        let rotate_speed = 1.0; // Radians per second
        let orbit_speed = 1.0; // Radians per second

        let orbit_amount = orbit_speed * delta;

        // Calculate frame-adjusted movements
        let move_amount = move_speed * delta;
        let rotate_amount = rotate_speed * delta;
        if let Some(keys) = Some(input.get_keys()) {
            // LMFAO
            for key in keys {
                match key {
                    minifb::Key::W => self.scene.camera.move_forward(move_amount),
                    minifb::Key::S => self.scene.camera.move_forward(-move_amount),
                    minifb::Key::A => self.scene.camera.rotate(0.0, rotate_amount),
                    minifb::Key::D => self.scene.camera.rotate(0.0, -rotate_amount),
                    minifb::Key::Up => self.scene.camera.rotate(rotate_amount, 0.0),
                    minifb::Key::Down => self.scene.camera.rotate(-rotate_amount, 0.0),
                    minifb::Key::E => {
                        // Orbit clockwise
                        let current_angle = self.scene.camera.get_orbital_angle();
                        self.scene.camera.orbit(current_angle - orbit_amount);
                    }
                    _ => {}
                }
            }
        }
    }
}
