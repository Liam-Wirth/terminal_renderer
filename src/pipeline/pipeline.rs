use std::time::Instant;
pub(crate) use std::{cell::RefCell, io};

use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::MouseEventKind;
use glam::Vec3;
use glam::{Affine3A, Mat4, Vec2, Vec4};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};

use super::{
    buffer::Buffer, rasterizer::Rasterizer, Clipper, Fragment, GBuffer, ProcessedGeometry,
};
use crate::core::LightType;
use crate::core::{BlinnPhongShading, FlatShading, LightMode, LightingModel};
use crate::{
    core::{Color, RenderMode, Scene},
    debug_print,
    pipeline::{ClipTriangle, ClipVertex},
    util::format_mat4,
    Metrics,
};

pub struct States {
    pub draw_wireframe: bool,
    pub bake_normals: bool,
    pub backface_culling: bool,
    pub move_obj: bool,
    pub current_obj: usize,
    pub light_mode: crate::core::LightMode,
    pub is_mouse_look_enabled: bool,
    pub is_mouse_pan_enabled: bool,
    pub last_mouse_pos: Option<(f32, f32)>,
    pub draw_light_marker: bool,
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
    pub scene: Scene,                          // 3D scene with camera and objects
    pub states: RefCell<States>,               // Pipeline state flags
    pub scale_factor: usize,                   // Scale factor, 2 means render at half resolution
    front_buffer: RefCell<B>,                  // Currently displayed buffer
    back_buffer: RefCell<B>,                   // Buffer being rendered to
    geometry: RefCell<Vec<ProcessedGeometry>>, // Transformed geometry ready for rasterization
    rasterizer: RefCell<Rasterizer>,           // Converts triangles to fragments
    clipper: RefCell<Clipper>,                 // Clips triangles against view frustum
    fragments: RefCell<Vec<Fragment>>,         // Output fragments from rasterization
    metrics: Metrics,                          // Performance metrics
    gbuffer: RefCell<GBuffer>,                 // Pre-Lighting pass buffer of fragments
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
                is_mouse_look_enabled: false,
                last_mouse_pos: None,
                is_mouse_pan_enabled: false,
                draw_light_marker: false,
            }),
            gbuffer: RefCell::new(GBuffer::new(width * height)),
            scale_factor: 1,
        }
    }
    pub fn new_with_scale(width: usize, height: usize, scene: Scene, scale_factor: usize) -> Self {
        let width = width / scale_factor;
        let height = height / scale_factor;
        Self::new(width, height, scene)
    }

    /// Main render loop function - processes one frame
    ///
    /// Steps:
    /// 1. Clear back buffer
    /// 2. Process environment geometry
    /// 3. Transform vertices to clip space and clip triangles
    /// 4. Rasterize visible triangles to fragments
    /// 5. Process fragments and write to gbuffer
    /// 6. Do Lighting Pass on Gbuffer, and then write to back buffer
    /// 7. Present back buffer to window or output
    /// 8. Swap front and back buffers
    pub fn render_frame(&self, window: Option<&mut Window>) -> io::Result<()> {
        self.back_buffer.borrow_mut().clear();
        self.gbuffer.borrow_mut().clear();

        // 1. Process vertices to clip space
        self.process_geometry();

        // 2. Clip triangles (already integrated in process_geometry)
        // The clipper operates during geometry processing

        // 3. Rasterize clipped triangles
        self.rasterize();

        // 4. Process fragments into gbuffer
        self.process_fragments(&self.fragments.borrow());
        // 5. Lighting pass (will automatically skip if lighting is disabled
        self.lighting_pass();

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
                let material_color = tri.material.and_then(|mat_id| {
                    entity
                        .mesh
                        .materials
                        .get(mat_id)
                        .map(|mat| mat.get_base_color())
                });
                // println!("MATERIAL COLOR: {:?}", material_color);

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
                        uv: v0.uv.unwrap_or(Vec2::ZERO),
                    },
                    ClipVertex {
                        position: mvp_matrix * Vec4::from((v1.pos, 1.0)),
                        color: v1_color,
                        uv: v1.uv.unwrap_or(Vec2::ZERO),
                    },
                    ClipVertex {
                        position: mvp_matrix * Vec4::from((v2.pos, 1.0)),
                        color: v2_color,
                        uv: v2.uv.unwrap_or(Vec2::ZERO),
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
        );
    }
    pub fn process_fragments(&self, fragments: &[Fragment]) {
        // IMPORTANT, THIS NOW PROCESSES INTO A GBUFFER, WHICH THEN PROCESSES INTO THE BACK BUFFER
        let mut gbuffer = self.gbuffer.borrow_mut();
        for (idx, fragment) in fragments.iter().enumerate() {
            let x = fragment.screen_pos.x as usize;
            let y = fragment.screen_pos.y as usize;
            if x >= self.width || y >= self.height {
                continue;
            } // duh
            let idx = y * self.width + x;
            if fragment.depth < gbuffer.depth[idx] {
                // depth test
                gbuffer.albedo[idx] = fragment.albedo;
                gbuffer.normal[idx] = fragment.normal;
                gbuffer.depth[idx] = fragment.depth;
                gbuffer.specular[idx] = fragment.specular;
                gbuffer.shininess[idx] = fragment.shininess;
                gbuffer.matid[idx] = fragment.mat_id;
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

    pub fn lighting_pass(&self) {
        // Early Exit (cases include drawing wireframes for debugging, or just not doing any lighting)
        {
            // cheeky scope so the value gets dropped
            let states = self.states.borrow();
            if states.draw_wireframe || states.light_mode == LightMode::None {
                // Just populate the back buffer as is (copying old code directly over)
                let mut buffer = self.back_buffer.borrow_mut();
                for fragment in self.fragments.borrow().iter() {
                    let pixel = B::create_pixel(fragment.albedo);
                    let pos = (
                        fragment.screen_pos.x as usize,
                        fragment.screen_pos.y as usize,
                    );
                    buffer.set_pixel(pos, &fragment.depth, pixel);
                }
                return;
            }
        }
        // Obtain inverse view_proj Matrix  (helps us reconstruct world space positions, by applying the inverse dot to the vector we basically "un project" but after doing/applying clipping and a depth buffer pass and stuff. This way we ultimately minimize the amount of things we have to shade
        let view = self.scene.camera.view_matrix();
        let proj = self.scene.camera.projection_matrix();
        let inv_viewproj = (proj * view).inverse();

        let gbuffer = self.gbuffer.borrow_mut();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if gbuffer.depth[idx] == f32::INFINITY {
                    continue;
                }

                let ndc_x = (x as f32 / self.width as f32) * 2.0 - 1.0;
                let ndc_y = 1.0 - (y as f32 / self.height as f32) * 2.0;
                let ndc_z = gbuffer.depth[idx] * 2.0 - 1.0;

                let ndc_pos = Vec4::new(ndc_x, ndc_y, ndc_z, 1.0);
                let world_homog = inv_viewproj * ndc_pos;
                let world_pos = world_homog.truncate() / world_homog.w;

                let albedo = gbuffer.albedo[idx];
                let normal = gbuffer.normal[idx].normalize();
                let specular = gbuffer.specular[idx];
                let shininess = gbuffer.shininess[idx];
                let view_dir = (self.scene.camera.position() - world_pos).normalize();
                let matid = gbuffer.matid[idx];
                let mut mat = None;
                if let Some(matid) = matid {
                    let (entid, matid) = matid;
                    mat = Some(&self.scene.entities[entid].mesh.materials[matid]);
                }

                let final_color = Color::BLACK;
                let final_color = match self.states.borrow().light_mode {
                    LightMode::None => final_color,
                    LightMode::BlinnPhong => BlinnPhongShading.shade(
                        albedo,
                        normal,
                        specular,
                        shininess,
                        world_pos,
                        view_dir,
                        &self.scene.lights,
                        mat,
                    ),
                    LightMode::Flat => FlatShading.shade(
                        albedo,
                        normal,
                        specular,
                        shininess,
                        world_pos,
                        view_dir,
                        &self.scene.lights,
                        mat,
                    ),
                };
                let pixel = B::create_pixel(final_color); //FUUUUUUUUUUUUUUU
                self.back_buffer
                    .borrow_mut()
                    .set_pixel((x, y), &gbuffer.depth[idx], pixel)
            }
        }
    }

    // TODO: This
    fn draw_light_markers(&self) {
        if !self.states.borrow().draw_light_marker {
            return;
        }
        let view_proj = self.scene.camera.view_matrix() * self.scene.camera.projection_matrix();
        let buff = self.back_buffer.borrow_mut();

        for light in &self.scene.lights {
            match &light.light_type {
                &LightType::Point { position, .. } => {}
                &LightType::Directional(direction) => {}
                &LightType::Spot {
                    position,
                    direction,
                    ..
                } => {}
            }
        }
    }

    // TODO: Move this to a separate file along witht the input handling for the terminal environment
    pub fn window_handle_input(&mut self, input: &minifb::Window, last_frame: std::time::Instant) {
        let delta = 0.1;
        let move_speed = 1.0;
        let rotate_speed = 1.0;
        let orbit_speed = 1.0;
        let orbit_amount = orbit_speed * delta;
        let move_amount = move_speed * delta;
        let rotate_amount = rotate_speed * delta;

        if input.get_mouse_down(MouseButton::Left) {
            if !self.states.borrow().is_mouse_pan_enabled {
                self.states.borrow_mut().is_mouse_pan_enabled = true;
                self.states.borrow_mut().last_mouse_pos = input.get_mouse_pos(MouseMode::Clamp);
            }
        } else if self.states.borrow().is_mouse_pan_enabled
            && !input.get_mouse_down(MouseButton::Right)
        {
            self.states.borrow_mut().is_mouse_pan_enabled = false;
            self.states.borrow_mut().last_mouse_pos = None;
        }

        // Mouse Look Rotation (if enabled)
        if self.states.borrow().is_mouse_pan_enabled {
            if let Some(current_mouse_pos) = input.get_mouse_pos(minifb::MouseMode::Clamp) {
                if let Some(last_mouse_pos) = self
                    .states
                    .borrow_mut()
                    .last_mouse_pos
                    .replace(current_mouse_pos)
                {
                    let current_mouse_pos = Vec2::new(current_mouse_pos.0, current_mouse_pos.1);
                    let last_mouse_pos = Vec2::new(last_mouse_pos.0, last_mouse_pos.1);
                    let mouse_delta = current_mouse_pos - last_mouse_pos;
                    for ent in self.scene.entities.iter_mut() {
                        let mut t = *ent.transform();
                        t *= Affine3A::from_rotation_y(mouse_delta.x * rotate_speed * 0.005);
                        t *= Affine3A::from_rotation_x(mouse_delta.y * rotate_speed * 0.005);
                        ent.set_transform(t);
                    }
                }
            }
        }

        // Mouse Look Toggle (Right Mouse Button)
        if input.get_mouse_down(MouseButton::Right) {
            if !self.states.borrow().is_mouse_look_enabled {
                self.states.borrow_mut().is_mouse_look_enabled = true;
                self.states.borrow_mut().last_mouse_pos =
                    input.get_mouse_pos(minifb::MouseMode::Clamp)
            }
        } else if self.states.borrow().is_mouse_look_enabled
            && !input.get_mouse_down(MouseButton::Right)
        {
            self.states.borrow_mut().is_mouse_look_enabled = false;
            self.states.borrow_mut().last_mouse_pos = None;
        }

        // Mouse Look Rotation (if enabled)
        if self.states.borrow().is_mouse_look_enabled {
            if let Some(current_mouse_pos) = input.get_mouse_pos(minifb::MouseMode::Clamp) {
                if let Some(last_mouse_pos) = self
                    .states
                    .borrow_mut()
                    .last_mouse_pos
                    .replace(current_mouse_pos)
                {
                    let current_mouse_pos = Vec2::new(current_mouse_pos.0, current_mouse_pos.1);
                    let last_mouse_pos = Vec2::new(last_mouse_pos.0, last_mouse_pos.1);
                    let mouse_delta = current_mouse_pos - last_mouse_pos;
                    self.scene.camera.yaw(mouse_delta.x * rotate_speed * 0.005); // Adjust sensitivity as needed
                    self.scene
                        .camera
                        .pitch(mouse_delta.y * rotate_speed * 0.005); // Adjust sensitivity, invert Y if needed
                }
            }
        }

        // Mouse Wheel Zoom
        //if let Some(wheel_delta) = input.get_scroll_wheel() {
        //
        //    self.scene.camera.move_forward(wheel_delta as f32 * zoom_speed); // Zoom by moving camera forward/backward
        //    input.reset_wheel_delta(); // Important to reset delta each frame
        //}

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
                    minifb::Key::Key3 => {
                       self.scene.lights[0].orbit(Vec3::ZERO, 5.0, 0.01, 0.0001);
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
                        let t = *ent.transform();
                        for entity in &mut self.scene.entities {
                            let mut t = *entity.transform();
                            t *= Affine3A::from_rotation_y(0.1);
                            entity.set_transform(t);
                        }
                        //self.scene.entities[current_obj].set_transform(t);
                    }
                    minifb::Key::Key1 => {
                        for entity in &mut self.scene.entities {
                            let mut t = *entity.transform();
                            t *= Affine3A::from_rotation_x(0.05);
                            t *= Affine3A::from_rotation_y(0.1);
                            t *= Affine3A::from_rotation_z(0.07);
                            entity.set_transform(t);
                        }
                    }
                    _ => {}
                }
            }
        }

        // getting rid of the big ass match statement? maybe?
    }
    pub fn handle_crossterm_input(&mut self, event: Event, _last_frame: Instant) -> bool {
        // Constants (adjust as needed)
        let delta = 0.1;
        let move_speed = 1.0;
        let rotate_speed = 1.0;
        let orbit_speed = 1.0;
        let orbit_amount = orbit_speed * delta;
        let move_amount = move_speed * delta;
        let rotate_amount = rotate_speed * delta;

        // This flag will be set true if the user presses the exit key.
        let mut should_break = false;

        match event {
            event::Event::Key(KeyEvent {
                code, modifiers, ..
            }) => {
                match code {
                    // Toggle wireframe
                    KeyCode::Char('p') => {
                        let current = self.states.borrow().draw_wireframe;
                        self.states.borrow_mut().draw_wireframe = !current;
                        // println!("Draw wireframe: {}", !current);
                    }
                    // Toggle move_obj
                    KeyCode::Char('j') => {
                        let cur = self.states.borrow().move_obj;
                        self.states.borrow_mut().move_obj = !cur;
                        // println!("Move obj: {}", !cur);
                    }
                    // Decrement current object index
                    KeyCode::Char('[') => {
                        let mut current = self.states.borrow().current_obj;
                        current = current.saturating_sub(1);
                        if current > self.scene.entities.len() - 1 {
                            current = self.scene.entities.len() - 1;
                        }
                        self.states.borrow_mut().current_obj = current;
                        //println!("Current object: {}", current);
                    }
                    // Increment current object index
                    KeyCode::Char(']') => {
                        let mut current = self.states.borrow().current_obj;
                        current += 1;
                        if current > self.scene.entities.len() - 1 {
                            current %= self.scene.entities.len();
                        }
                        self.states.borrow_mut().current_obj = current;
                        //println!("Current object: {}", current);
                    }
                    // Reset camera (if not moving an object)
                    KeyCode::Char('u') => {
                        if self.states.borrow().move_obj {
                            // do nothing
                        } else {
                            self.scene.camera.reset();
                        }
                    }
                    // Print matrices debug info
                    KeyCode::Char('/') => {
                        //println!("Printing out Matrices:");
                        //println!(
                        //    "{}",
                        //    format_mat4("Camera View Matrix", &self.scene.camera.view_matrix())
                        //);
                        //println!(
                        //    "{}",
                        //    format_mat4(
                        //        "Camera Projection Matrix",
                        //        &self.scene.camera.projection_matrix()
                        //    )
                        //);
                        //println!(
                        //    "{}",
                        //    format_mat4(
                        //        "Model Matrix (first entity)",
                        //        &Mat4::from(*self.scene.entities[0].transform())
                        //    )
                        //);
                        //println!(
                        //    "{}",
                        //    format_mat4(
                        //        "MVP matrix of first entity",
                        //        &(self.scene.camera.projection_matrix()
                        //            * self.scene.camera.view_matrix()
                        //            * Mat4::from(*self.scene.entities[0].transform()))
                        //    )
                        //);
                    }
                    // Print camera debug info
                    KeyCode::Char('e') => {
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
                    // Cycle render mode on selected object
                    KeyCode::Char('r') => {
                        println!("Updating Render Mode of selected object");
                        let obj = &self.scene.entities[self.states.borrow().current_obj];
                        print!("Selected Object: {:?} -> ", obj.name);
                        if let Ok(mut mode) = obj.render_mode().lock() {
                            *mode = match *mode {
                                RenderMode::Solid => RenderMode::Wireframe,
                                RenderMode::Wireframe => RenderMode::Solid,
                                _ => RenderMode::Solid,
                            };
                            println!("New render mode: {:?}", *mode);
                        }
                    }
                    // Rotate all entities (example for key '0')
                    KeyCode::Char('0') => {
                        for entity in &mut self.scene.entities {
                            let mut t = *entity.transform();
                            t *= Affine3A::from_rotation_y(0.1);
                            entity.set_transform(t);
                        }
                    }
                    // Rotate all entities (example for key '1')
                    KeyCode::Char('1') => {
                        for entity in &mut self.scene.entities {
                            let mut t = *entity.transform();
                            t *= Affine3A::from_rotation_x(0.05);
                            t *= Affine3A::from_rotation_y(0.1);
                            t *= Affine3A::from_rotation_z(0.07);
                            entity.set_transform(t);
                        }
                    }
                    // Movement and object translation keys
                    KeyCode::Char('w') => {
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
                    KeyCode::Char('s') => {
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
                    KeyCode::Char('a') => {
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
                    KeyCode::Char('d') => {
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
                    // Orbit the first light in the light array around the origin
                    KeyCode::Char('3') => {
                        let mut light = self.scene.lights[0].clone();
                        light.orbit(Vec3::ZERO, 5.0, 0.1, 0.1);
                    }
                    // Orbit the camera
                    KeyCode::Char('o') => {
                        self.scene.camera.orbit(orbit_amount);
                    }
                    // Use SPACE for upward movement. If SHIFT is held, move downward.
                    KeyCode::Char(' ') => {
                        if modifiers.contains(event::KeyModifiers::SHIFT) {
                            // SHIFT+SPACE: move down
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
                        } else {
                            // SPACE: move up
                            let move_obj = self.states.borrow().move_obj;
                            let current_obj = self.states.borrow().current_obj;
                            if move_obj {
                                let ent = &self.scene.entities[current_obj];
                                let mut t = *ent.transform();
                                t.translation.y += move_amount;
                                self.scene.entities[current_obj].set_transform(t);
                            } else {
                                self.scene.camera.move_up(move_amount);
                            }
                        }
                    }
                    // Rotate the camera with the arrow keys
                    KeyCode::Up => {
                        self.scene.camera.rotate(rotate_amount, 0.0);
                    }
                    KeyCode::Down => {
                        self.scene.camera.rotate(-rotate_amount, 0.0);
                    }
                    // Exit on Esc or q (or Q)
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                        should_break = true;
                    }
                    _ => {}
                }
            }
            crossterm::event::Event::Mouse(mouse_event) => {
                match mouse_event.kind {
                    // On mouse button down events...
                    MouseEventKind::Down(btn) => match btn {
                        crossterm::event::MouseButton::Left => {
                            if !self.states.borrow().is_mouse_pan_enabled {
                                self.states.borrow_mut().is_mouse_pan_enabled = true;
                                self.states.borrow_mut().last_mouse_pos =
                                    Some((mouse_event.column.into(), mouse_event.row.into()));
                            }
                            println!(
                                "Left click at ({}, {})",
                                mouse_event.column, mouse_event.row
                            );
                        }
                        crossterm::event::MouseButton::Right => {
                            if !self.states.borrow().is_mouse_look_enabled {
                                self.states.borrow_mut().is_mouse_look_enabled = true;
                                self.states.borrow_mut().last_mouse_pos =
                                    Some((mouse_event.column.into(), mouse_event.row.into()));
                            }
                            println!(
                                "Right click at ({}, {})",
                                mouse_event.column, mouse_event.row
                            );
                        }
                        crossterm::event::MouseButton::Middle => {
                            println!(
                                "Middle click at ({}, {})",
                                mouse_event.column, mouse_event.row
                            );
                        }
                        _ => {}
                    },
                    // On mouse button up events...
                    MouseEventKind::Up(btn) => {
                        match btn {
                            crossterm::event::MouseButton::Left => {
                                self.states.borrow_mut().is_mouse_pan_enabled = false;
                                self.states.borrow_mut().last_mouse_pos = None;
                                //println!("Left button released");
                            }
                            crossterm::event::MouseButton::Right => {
                                self.states.borrow_mut().is_mouse_look_enabled = false;
                                self.states.borrow_mut().last_mouse_pos = None;
                                //println!("Right button released");
                            }
                            crossterm::event::MouseButton::Middle => {}
                            _ => {}
                        }
                    }
                    // On dragging events, update the camera or entity rotations accordingly.
                    MouseEventKind::Drag(btn) => {
                        match btn {
                            crossterm::event::MouseButton::Left => {
                                if self.states.borrow().is_mouse_pan_enabled {
                                    match self.states.borrow_mut().last_mouse_pos.take()
                                    { Some(last_pos) => {
                                        let current_pos = (mouse_event.column, mouse_event.row);
                                        let current_mouse =
                                            Vec2::new(current_pos.0 as f32, current_pos.1 as f32);
                                        let last_mouse =
                                            Vec2::new(last_pos.0 as f32, last_pos.1 as f32);
                                        let mouse_delta = current_mouse - last_mouse;
                                        for ent in self.scene.entities.iter_mut() {
                                            let mut t = *ent.transform();
                                            t *= Affine3A::from_rotation_y(
                                                mouse_delta.x * rotate_speed * 0.005,
                                            );
                                            t *= Affine3A::from_rotation_x(
                                                mouse_delta.y * rotate_speed * 0.005,
                                            );
                                            ent.set_transform(t);
                                        }
                                        let (a, b) = current_pos;

                                        self.states.borrow_mut().last_mouse_pos =
                                            Some((a.into(), b.into()));
                                    } _ => {
                                        self.states.borrow_mut().last_mouse_pos = Some((
                                            mouse_event.column.into(),
                                            mouse_event.row.into(),
                                        ));
                                    }}
                                }
                            }
                            crossterm::event::MouseButton::Right => {
                                if self.states.borrow().is_mouse_look_enabled {
                                    match self.states.borrow_mut().last_mouse_pos.take()
                                    { Some(last_pos) => {
                                        let current_pos = (mouse_event.column, mouse_event.row);
                                        let current_mouse =
                                            Vec2::new(current_pos.0 as f32, current_pos.1 as f32);
                                        let last_mouse =
                                            Vec2::new(last_pos.0 as f32, last_pos.1 as f32);
                                        let mouse_delta = current_mouse - last_mouse;
                                        self.scene.camera.yaw(mouse_delta.x * rotate_speed * 0.005);
                                        self.scene
                                            .camera
                                            .pitch(mouse_delta.y * rotate_speed * 0.005);
                                        let (a, b) = current_pos;
                                        self.states.borrow_mut().last_mouse_pos =
                                            Some((a.into(), b.into()));
                                    } _ => {
                                        self.states.borrow_mut().last_mouse_pos = Some((
                                            mouse_event.column.into(),
                                            mouse_event.row.into(),
                                        ));
                                    }}
                                }
                            }
                            _ => {}
                        }
                        //println!(
                        //    "Dragging with {:?} button at ({}, {})",
                        //    btn, mouse_event.column, mouse_event.row
                        //);
                    }
                    MouseEventKind::Moved => {
                        //println!(
                        //    "Mouse moved to ({}, {})",
                        //    mouse_event.column, mouse_event.row
                        //);
                    }
                    MouseEventKind::ScrollUp => {
                        //println!(
                        //    "Scrolled up at ({}, {})",
                        //    mouse_event.column, mouse_event.row
                        //);
                        // Optionally implement zoom in
                    }
                    MouseEventKind::ScrollDown => {
                        //println!(
                        //    "Scrolled down at ({}, {})",
                        //    mouse_event.column, mouse_event.row
                        //);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        should_break
    }
}
