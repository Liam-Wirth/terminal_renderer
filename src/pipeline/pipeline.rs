pub(crate) use std::{cell::RefCell, io};

use crossterm::cursor;
use glam::{Affine3A, Mat4, Vec4};
use minifb::{Key, KeyRepeat, Window};

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
    pub light_mode: crate::core::LightMode,
}

/// A graphics rendering pipeline that processes 3D geometry into 2D screen output
///
/// The pipeline handles:
/// - Vertex processing and transformation to clip space
/// - Triangle clipping against view frustum
/// - Rasterization of triangles to fragments
/// - Fragment processing and writing to framebuffer
pub struct Pipeline<B: Buffer> {
    pub width: usize,                          // Screen width in pixels
    pub height: usize,                         // Screen height in pixels
    front_buffer: RefCell<B>,                  // Currently displayed buffer
    back_buffer: RefCell<B>,                   // Buffer being rendered to
    pub scene: Scene,                          // 3D scene with camera and objects
    geometry: RefCell<Vec<ProcessedGeometry>>, // Transformed geometry ready for rasterization
    rasterizer: RefCell<Rasterizer>,           // Converts triangles to fragments
    clipper: RefCell<Clipper>,                 // Clips triangles against view frustum
    fragments: RefCell<Vec<Fragment>>,         // Output fragments from rasterization
    metrics: Metrics,                          // Performance metrics
    pub states: RefCell<States>,               // Pipeline state flags
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
                light_mode: crate::core::LightMode::BlinnPhong,
            }),
        }
    }

    /// Main render loop function - processes one frame
    ///
    /// Steps:
    /// 1. Clear back buffer
    /// 2. Process environment geometry
    /// 3. Transform vertices to clip space and clip triangles
    /// 4. Rasterize visible triangles to fragments
    /// 5. Process fragments and write to back buffer
    /// 6. Present back buffer to window or output
    /// 7. Swap front and back buffers
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

    /// Process scene geometry through vertex transformation and clipping
    ///
    /// For each mesh:
    /// 1. Calculate model-view-projection matrix
    /// 2. Transform vertices to clip space
    /// 3. Clip triangles against view frustum
    /// 4. Store processed geometry for rasterization
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
            entity.update();
            let model_matrix = Mat4::from(*entity.transform());
            let mvp_matrix = projection_matrix * view_matrix * model_matrix;

            // Process each triangle
            for tri in &entity.mesh.tris {
                // Get the material’s base color (if available)
                let material_color = tri
                    .material
                    .map(|mat_id| entity.mesh.materials[mat_id].get_base_color());

                // For each vertex, if no per-vertex color is provided then use the material's base color (or white)
                let v0 = &entity.mesh.vertices[tri.vertices[0]];
                let v1 = &entity.mesh.vertices[tri.vertices[1]];
                let v2 = &entity.mesh.vertices[tri.vertices[2]];

                let v0_color = v0
                    .color
                    .unwrap_or_else(|| material_color.unwrap_or(Color::WHITE));
                let v1_color = v1
                    .color
                    .unwrap_or_else(|| material_color.unwrap_or(Color::WHITE));
                let v2_color = v2
                    .color
                    .unwrap_or_else(|| material_color.unwrap_or(Color::WHITE));

                // Create clip vertices using the vertex positions and chosen colors
                let clip_verts = [
                    ClipVertex {
                        position: mvp_matrix * Vec4::from((v0.pos, 1.0)),
                        color: v0_color,
                    },
                    ClipVertex {
                        position: mvp_matrix * Vec4::from((v1.pos, 1.0)),
                        color: v1_color,
                    },
                    ClipVertex {
                        position: mvp_matrix * Vec4::from((v2.pos, 1.0)),
                        color: v2_color,
                    },
                ];

                let clip_triangle = ClipTriangle {
                    vertices: clip_verts,
                };

                // Clip the triangle (using the clipper)
                let clipped_triangles = self.clipper.borrow().clip_triangle(&clip_triangle);

                // Add resulting triangles to the geometry buffer for rasterization
                for triangle in clipped_triangles {
                    self.geometry.borrow_mut().push(ProcessedGeometry {
                        transform: mvp_matrix,
                        entity_id: i,
                        vertices: triangle.vertices,
                        material_id: tri.material,
                        world_pos: tri.vertices,
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
            &self.states.borrow().light_mode,
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

    pub fn window_handle_input(&mut self, input: &minifb::Window, last_frame: std::time::Instant) {
        let delta = 0.1;
        let move_speed = 1.0;
        let rotate_speed = 1.0;
        let orbit_speed = 1.0;
        let orbit_amount = orbit_speed * delta;
        let move_amount = move_speed * delta;
        let rotate_amount = rotate_speed * delta;

        if input.is_key_pressed(minifb::Key::P, KeyRepeat::No) {
            let current = self.states.borrow().draw_wireframe;
            self.states.borrow_mut().draw_wireframe = !current;
            println!("Draw wireframe: {}", !current);
        }
        if input.is_key_pressed(minifb::Key::J, KeyRepeat::No) {
            let cur = self.states.borrow().move_obj;
            self.states.borrow_mut().move_obj = !cur;
            println!("Move obj: {}", !cur);
        }
        if input.is_key_pressed(minifb::Key::B, KeyRepeat::No) {
            println!("Re baking the normals of the selected object");
            let current_obj = self.states.borrow().current_obj;
            self.scene.entities[current_obj]
                .mesh
                .bake_normals_to_colors();
        }
        if input.is_key_pressed(minifb::Key::LeftBracket, KeyRepeat::No) {
            let mut current = self.states.borrow().current_obj;
            current = current.saturating_sub(1);
            if current > self.scene.entities.len() - 1 {
                current = self.scene.entities.len() - 1;
            }
            self.states.borrow_mut().current_obj = current;
            println!("Current object: {}", current);
        }
        if input.is_key_pressed(minifb::Key::RightBracket, KeyRepeat::No) {
            let mut current = self.states.borrow().current_obj;
            current += 1;
            if current > self.scene.entities.len() - 1 {
                current %= self.scene.entities.len();
            }
            self.states.borrow_mut().current_obj = current;
            println!("Current object: {}", current);
        }

        if input.is_key_pressed(minifb::Key::U, KeyRepeat::No) {
            if self.states.borrow().move_obj {
                return;
            } else {
                self.scene.camera.reset();
            }
        }
        if input.is_key_pressed(minifb::Key::Slash, KeyRepeat::No) {
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
                    &Mat4::from(*self.scene.entities[0].transform())
                )
            );
            println!(
                "{}",
                format_mat4(
                    "MVP matrix of first entity",
                    &(self.scene.camera.projection_matrix()
                        * self.scene.camera.view_matrix()
                        * Mat4::from(*self.scene.entities[0].transform()))
                )
            );
        }
        if input.is_key_pressed(minifb::Key::E, KeyRepeat::No) {
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
        if input.is_key_pressed(Key::R, KeyRepeat::No) {
            println!("Updating Render Mode of selected object");
            // We'll do it cyclicly for now
            let obj = &self.scene.entities[self.states.borrow().current_obj];

            print!("Selected Object: {:?} -> ", obj.name);

            if let Ok(mut mode) = obj.render_mode().lock() {
                // Cycle through the render modes
                *mode = match *mode {
                    RenderMode::Solid => RenderMode::Wireframe,
                    RenderMode::Wireframe => RenderMode::Solid,
                    //RenderMode::FixedPoint => RenderMode::Solid,
                    _ => RenderMode::Solid,
                };
                println!("New render mode: {:?}", *mode);
            }
        }

        if input.is_key_pressed(Key::NumPad0, KeyRepeat::No) {
            let obj = &self.scene.entities[self.states.borrow().current_obj];
            println!("Material Info of selected object {:?}", obj.name);
            for (i, mat) in obj.mesh.materials.iter().enumerate() {
                println!("Material {}: {}", i, mat);
            }
        }

        // FIX: Update input handling to be less "fast" like if I try and just tap a button it
        // seems to register that I hit it like 4 times (due to fast framerate) need to slow down
        // polling I presume, or handle it using the key_pressed instead of some other thing

        if let Some(keys) = Some(input.get_keys()) {
            for key in keys.iter() {
                match key {
                    minifb::Key::W => {
                        let move_obj = self.states.borrow().move_obj;
                        let current_obj = self.states.borrow().current_obj;
                        if move_obj {
                            let ent = &self.scene.entities[current_obj];
                            let mut t = *ent.transform();
                            t.translation.z += move_amount;
                            self.scene.entities[current_obj].set_transform(t);
                        } else {
                            self.scene.camera.move_forward(move_amount);
                        }
                    }
                    minifb::Key::S => {
                        let move_obj = self.states.borrow().move_obj;
                        let current_obj = self.states.borrow().current_obj;
                        if move_obj {
                            let ent = &self.scene.entities[current_obj];
                            let mut t = *ent.transform();
                            t.translation.z -= move_amount;
                            self.scene.entities[current_obj].set_transform(t);
                        } else {
                            self.scene.camera.move_forward(-move_amount);
                        }
                    }
                    minifb::Key::A => {
                        let move_obj = self.states.borrow().move_obj;
                        let current_obj = self.states.borrow().current_obj;
                        if move_obj {
                            let ent = &self.scene.entities[current_obj];
                            let mut t = *ent.transform();
                            t.translation.x -= move_amount;
                            self.scene.entities[current_obj].set_transform(t);
                        } else {
                            self.scene.camera.move_right(-move_amount);
                        }
                    }
                    minifb::Key::D => {
                        let move_obj = self.states.borrow().move_obj;
                        let current_obj = self.states.borrow().current_obj;
                        if move_obj {
                            let ent = &self.scene.entities[current_obj];
                            let mut t = *ent.transform();
                            t.translation.x += move_amount;
                            self.scene.entities[current_obj].set_transform(t);
                        } else {
                            self.scene.camera.move_right(move_amount);
                        }
                    }
                    // orbit
                    minifb::Key::O => {
                        self.scene.camera.orbit(orbit_amount);
                    }
                    minifb::Key::Space => {
                        let move_obj = self.states.borrow().move_obj;
                        let current_obj = self.states.borrow().current_obj;
                        if move_obj {
                            return;
                        } else {
                            self.scene.camera.move_up(move_amount);
                        }
                    }
                    minifb::Key::LeftShift => {
                        let move_obj = self.states.borrow().move_obj;
                        let current_obj = self.states.borrow().current_obj;
                        if move_obj {
                            let ent = &self.scene.entities[current_obj];
                            let mut t = *ent.transform();
                            t.translation.y -= move_amount;
                            self.scene.entities[current_obj].set_transform(t);
                        } else {
                            self.scene.camera.move_up(-move_amount);
                        }
                    }
                    minifb::Key::Up => self.scene.camera.rotate(rotate_amount, 0.0),
                    minifb::Key::Down => self.scene.camera.rotate(-rotate_amount, 0.0),
                    minifb::Key::Key0 => {
                        let current_obj = self.states.borrow().current_obj;
                        let ent = &self.scene.entities[current_obj];
                        let mut t = *ent.transform();
                        t *= Affine3A::from_rotation_x(0.1);
                        self.scene.entities[current_obj].set_transform(t);
                    }
                    _ => {}
                }
            }
        }

        // getting rid of the big ass match statement? maybe?
    }
}
