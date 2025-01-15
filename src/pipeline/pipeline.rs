pub(crate) use std::{cell::RefCell, io};

use glam::{Affine3A, Mat4, Vec4};
use minifb::Window;

use crate::{
    core::{Color, RenderMode, Scene},
    debug_print,
    pipeline::{ClipTriangle, ClipVertex},
    util::format_mat4,
    Metrics,
};

use super::{buffer::Buffer, rasterizer::Rasterizer, Clipper, Fragment, ProcessedGeometry};

pub struct States {
    pub draw_wireframe: bool,
    pub bake_normals: bool,
    pub backface_culling: bool,
    pub move_obj: bool,
    pub current_obj: usize,
}



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
    states: RefCell<States>,
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
            states: RefCell::new(States {
                draw_wireframe: false,
                bake_normals: false,
                backface_culling: true,
                move_obj: false,
                current_obj: 0, // kinda dumb but I'll make it work trust
            }),
        }
    }

    /// Consider this function to be like the function that gets run every frame, like the main loop
    pub fn render_frame(&self, window: Option<&mut Window>) -> io::Result<()> {
        self.back_buffer.borrow_mut().clear();

        self.process_environment();
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
        let view_matrix = self.scene.camera.view_matrix();
        let projection_matrix = self.scene.camera.projection_matrix();

        // Update clipper with current frustum planes
        self.clipper
            .borrow_mut()
            .update_frustum_planes(&self.scene.camera.frustum_planes());

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
                // Get material color if available
                let material_color = tri
                    .material
                    .map(|mat_id| entity.mesh.materials[mat_id].diffuse_color)
                    .unwrap_or(Color::WHITE);

                // Create clip vertices
                let clip_verts = [
                    ClipVertex {
                        position: mvp_matrix
                            * Vec4::from((entity.mesh.vertices[tri.vertices[0]].pos, 1.0)),
                        color: entity.mesh.vertices[tri.vertices[0]]
                            .color
                            .unwrap_or(material_color), // Use material color if vertex color not present
                    },
                    ClipVertex {
                        position: mvp_matrix
                            * Vec4::from((entity.mesh.vertices[tri.vertices[1]].pos, 1.0)),
                        color: entity.mesh.vertices[tri.vertices[1]]
                            .color
                            .unwrap_or(material_color), // Use material color if vertex color not present
                    },
                    ClipVertex {
                        position: mvp_matrix
                            * Vec4::from((entity.mesh.vertices[tri.vertices[2]].pos, 1.0)),
                        color: entity.mesh.vertices[tri.vertices[2]]
                            .color
                            .unwrap_or(material_color), // Use material color if vertex color not present
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
                        vertices: triangle.vertices,
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

    fn process_environment(&self) {
        if let Some(env_mesh) = self.scene.environment.get_mesh() {
            let view_matrix = self.scene.camera.view_matrix();
            let projection_matrix = self.scene.camera.projection_matrix();
            let mvp_matrix = projection_matrix * view_matrix; // No model matrix needed for environment

            // Process environment mesh triangles
            for tri in &env_mesh.tris {
                let clip_verts = [
                    ClipVertex {
                        position: mvp_matrix
                            * Vec4::from((env_mesh.vertices[tri.vertices[0]].pos, 1.0)),
                        color: env_mesh.vertices[tri.vertices[0]]
                            .color
                            .unwrap_or(Color::WHITE),
                    },
                    ClipVertex {
                        position: mvp_matrix
                            * Vec4::from((env_mesh.vertices[tri.vertices[1]].pos, 1.0)),
                        color: env_mesh.vertices[tri.vertices[1]]
                            .color
                            .unwrap_or(Color::WHITE),
                    },
                    ClipVertex {
                        position: mvp_matrix
                            * Vec4::from((env_mesh.vertices[tri.vertices[2]].pos, 1.0)),
                        color: env_mesh.vertices[tri.vertices[2]]
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
                        entity_id: usize::MAX, // Special ID for environment
                        vertices: triangle.vertices,
                    });
                }
            }
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
        let move_speed = 1.0;
        let rotate_speed = 1.0;
        let orbit_speed = 1.0;
        let orbit_amount = orbit_speed * delta;
        let move_amount = move_speed * delta;
        let rotate_amount = rotate_speed * delta;

        // FIX: Update input handling to be less "fast" like if I try and just tap a button it
        // seems to register that I hit it like 4 times (due to fast framerate) need to slow down
        // polling I presume, or handle it using the key_pressed instead of some other thing
        if let Some(keys) = Some(input.get_keys()) {
            for key in keys {
                match key {
                    minifb::Key::P => {
                        let current = self.states.borrow().draw_wireframe;
                        self.states.borrow_mut().draw_wireframe = !current;
                    }
                    minifb::Key::B => {
                        let current = self.states.borrow().bake_normals;
                        self.states.borrow_mut().bake_normals = !current;
                    }
                    minifb::Key::J => {
                        let current = self.states.borrow().move_obj;
                        self.states.borrow_mut().move_obj = !current;
                        println!("Move obj: {}", !current);
                    }
                    minifb::Key::LeftBracket => {
                        let mut current = self.states.borrow().current_obj;
                        current = current.saturating_sub(1);
                        if current > self.scene.entities.len() - 1 {
                            current = self.scene.entities.len() - 1;
                        }
                        self.states.borrow_mut().current_obj = current;
                        println!("Current object: {}", current);
                    }
                    minifb::Key::RightBracket => {
                        let mut current = self.states.borrow().current_obj;
                        current += 1;
                        if current > self.scene.entities.len() - 1 {
                            current %= self.scene.entities.len();
                        }
                        self.states.borrow_mut().current_obj = current;
                        println!("Current object: {}", current);
                    }
                    minifb::Key::W => {
                        let move_obj = self.states.borrow().move_obj;
                        let current_obj = self.states.borrow().current_obj;
                        if move_obj {
                            self.scene.entities[current_obj].transform.translation.z += move_amount;
                        } else {
                            self.scene.camera.move_forward(move_amount);
                        }
                    }
                    minifb::Key::S => {
                        let move_obj = self.states.borrow().move_obj;
                        let current_obj = self.states.borrow().current_obj;
                        if move_obj {
                            self.scene.entities[current_obj].transform.translation.z -= move_amount;
                        } else {
                            self.scene.camera.move_forward(-move_amount);
                        }
                    }
                    minifb::Key::A => {
                        let move_obj = self.states.borrow().move_obj;
                        let current_obj = self.states.borrow().current_obj;
                        if move_obj {
                            self.scene.entities[current_obj].transform.translation.x -= move_amount;
                        } else {
                            self.scene.camera.move_right(-move_amount);
                        }
                    }
                    minifb::Key::U => {
                        // reset cam or obj
                        if self.states.borrow().move_obj {
                            self.scene.entities[self.states.borrow().current_obj]
                                .transform
                                .translation = glam::Vec3::ZERO.into();
                        } else {
                            self.scene.camera.reset();
                        }
                    }
                    minifb::Key::D => {
                        let move_obj = self.states.borrow().move_obj;
                        let current_obj = self.states.borrow().current_obj;
                        if move_obj {
                            self.scene.entities[current_obj].transform.translation.x += move_amount;
                        } else {
                            self.scene.camera.move_right(move_amount);
                        }
                    }
                    minifb::Key::Space => {
                        let move_obj = self.states.borrow().move_obj;
                        let current_obj = self.states.borrow().current_obj;
                        if move_obj {
                            self.scene.entities[current_obj].transform.translation.y += move_amount;
                        } else {
                            self.scene.camera.move_up(move_amount);
                        }
                    }
                    minifb::Key::Slash => {
                        // Treat this as a question mark to print out debug info
                        println!("Printing out Matrices:");
                        println!(
                            "{}",
                            format_mat4("Camera View Matrix", &self.scene.camera.view_matrix())
                        );
                        println!(
                            "{}",
                            format_mat4(
                                "Camera Projection Matrix",
                                &self.scene.camera.projection_matrix()
                            )
                        );
                        println!(
                            "{}",
                            format_mat4(
                                "Model Matrix (first entity)",
                                &Mat4::from(self.scene.entities[0].transform)
                            )
                        );
                        println!(
                            "{}",
                            format_mat4(
                                "MVP matrix of first entity",
                                &(self.scene.camera.projection_matrix()
                                    * self.scene.camera.view_matrix()
                                    * Mat4::from(self.scene.entities[0].transform))
                            )
                        );
                    }
                    minifb::Key::LeftShift => {
                        let move_obj = self.states.borrow().move_obj;
                        let current_obj = self.states.borrow().current_obj;
                        if move_obj {
                            self.scene.entities[current_obj].transform.translation.y -= move_amount;
                        } else {
                            self.scene.camera.move_up(-move_amount);
                        }
                    }
                    minifb::Key::Up => self.scene.camera.rotate(rotate_amount, 0.0),
                    minifb::Key::Down => self.scene.camera.rotate(-rotate_amount, 0.0),
                    minifb::Key::E => {
                        println!("Camera Debug Info:");
                        println!("Camera position: {:?}", self.scene.camera.position());
                        println!("Camera target: {:?}\n", self.scene.camera.target());

                        println!("{:?}", self.scene.camera.orientation());

                        println!("Camera forward: {:?}", self.scene.camera.forward());
                        println!("Camera right: {:?}\n", self.scene.camera.right());
                        println!("Camera up: {:?}\n", self.scene.camera.up());

                        println!(
                            "{}",
                            format_mat4("Camera View Matrix", &self.scene.camera.view_matrix())
                        );
                        println!(
                            "{}",
                            format_mat4(
                                "Camera Projection Matrix",
                                &self.scene.camera.projection_matrix()
                            )
                        );
                        println!("\n\n");
                    }

                    minifb::Key::Key0 => {
                        let current_obj = self.states.borrow().current_obj;
                        self.scene.entities[current_obj].transform *=
                            Affine3A::from_rotation_x(0.1);
                    }
                    minifb::Key::R => {
                        println!("Updating Render Mode of selected object");
                        // We'll do it cyclicly for now
                        let obj = &self.scene.entities[self.states.borrow().current_obj];

                        print!("Selected Object: {:?} -> ", obj.name);

                        if let Ok(mut mode) = obj.render_mode().lock() {
                            // Cycle through the render modes
                            *mode = match *mode {
                                RenderMode::Solid => RenderMode::Wireframe,
                                RenderMode::Wireframe => RenderMode::FixedPoint,
                                RenderMode::FixedPoint => RenderMode::Solid,
                            };
                            println!("New render mode: {:?}", *mode);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
