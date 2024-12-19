use crate::{
    core::geometry::Mesh,
    core::{Camera, Color, Scene},
    Metrics,
};
use buffer::Buffer;
use glam::{Mat4, Vec2, Vec3, Vec4};
use rasterizer::Rasterizer;
use std::cell::RefCell;

pub mod terminal_pipeline;
pub use terminal_pipeline::TerminalPipeline;
pub mod rasterizer;
pub mod window_pipeline;
use crate::core::Pixel;
pub use window_pipeline::WindowPipeline;

mod buffer;

pub trait OldPipeline {
    type Scene;
    type Camera;
    type Buffer;

    fn init(&mut self, width: usize, height: usize);
    // Transform and project meshes into Clip Space
    fn process_geometry(
        &mut self,
        scene: &Self::Scene,
        camera: &Self::Camera,
    ) -> Vec<ProcessedGeometry>;
    // Convert the triangles, into fragments
    fn rasterize(&mut self, geometry: Vec<ProcessedGeometry>, scene: &Self::Scene)
        -> Vec<Fragment>;
    //Apply shading using lighting, textures, or mats
    fn process_fragments(&mut self, fragments: Vec<Fragment>, buffer: &mut Self::Buffer);
    // Draw
    fn present(&mut self, back: &mut Self::Buffer) -> std::io::Result<()>;
    fn cleanup(&mut self) -> std::io::Result<()>;
}

pub struct Pipeline<B: Buffer> {
    pub width: usize,
    pub height: usize,
    front_buffer: RefCell<B>,
    back_buffer: RefCell<B>,
    pub scene: Scene,
    pub camera: Camera,
    geometry: RefCell<Vec<ProcessedGeometry>>,
    rasterizer: RefCell<Rasterizer>,
    fragments: RefCell<Vec<Fragment>>,
    metrics: Metrics,
}

impl<B: Buffer> Pipeline<B> {
    pub fn new(width: usize, height: usize, scene: Scene, camera: Camera) -> Self {
        Self {
            width,
            height,
            front_buffer: RefCell::new(B::new(width, height)),
            back_buffer: RefCell::new(B::new(width, height)),
            scene,
            camera,
            metrics: Metrics::new(),
            geometry: RefCell::new(Vec::with_capacity(1024)),
            rasterizer: RefCell::new(Rasterizer::new(width, height)),
            fragments: RefCell::new(Vec::with_capacity(1024)), // Note: Capacity probably too small cause fragments might be like a per face thing idk
        }
    }

    pub fn render_frame(&self) -> std::io::Result<()> {
        // Clear back buffer
        self.back_buffer.borrow_mut().clear();

        // Process geometry
        self.process_geometry();

        // Rasterize
        self.rasterize();

        // attempt to process the fragments in place
        self.process_fragments(&self.fragments.borrow_mut());

        // Present and swap
        self.front_buffer.borrow().present()?;
        self.swap_buffers();

        Ok(())
    }

    pub fn process_fragments(&self, fragments: &[Fragment]) {
        let mut back = self.back_buffer.borrow_mut();
        for fragment in fragments {
            let x = fragment.screen_pos.x as usize;
            let y = fragment.screen_pos.y as usize;
            // back.set_pixel((x, y), &fragment.depth);
        }
    }

    pub fn swap_buffers(&self) {
        std::mem::swap(
            &mut *self.front_buffer.borrow_mut(),
            &mut *self.back_buffer.borrow_mut(),
        );
    }

    pub fn process_geometry(&self) {
        let view_proj = self.camera.get_projection_matrix() * self.camera.get_view_matrix();
        let mut geo = self.geometry.borrow_mut();

        // Ensure capacity once
        if geo.len() < self.scene.entities.len() {
            geo.resize(
                self.scene.entities.len(),
                ProcessedGeometry {
                    transform: Mat4::IDENTITY,
                    entity_id: 0,
                },
            );
        }

        // Update in place
        for (i, entity) in self.scene.entities.iter().enumerate() {
            geo[i].transform = view_proj * Mat4::from(entity.transform);
            geo[i].entity_id = i;
        }
    }

    pub fn rasterize(&self) {
        let mut rasterizer = self.rasterizer.borrow_mut();
        let geo = self.geometry.borrow();
        rasterizer.rasterize(&geo, &self.scene, &mut self.fragments.borrow_mut());
    }

    pub fn update_metrics(&mut self, frame_delta: std::time::Duration) {
        self.metrics.update(frame_delta);
    }
}

#[derive(Clone, Debug)]
pub struct ProcessedGeometry {
    /// Transformed vertices in Clip Space (MVP)
    pub transform: Mat4,
    /// Index into the Scene's entity buffer
    pub entity_id: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectedVertex {
    pub position: Vec2,
    pub depth: f32,
    pub color: Color,
}

#[derive(Clone)]
pub struct Fragment {
    pub screen_pos: Vec2,
    pub depth: f32,
    pub color: Color,
}

impl Default for Fragment {
    fn default() -> Self {
        Self {
            screen_pos: Vec2::ZERO,
            depth: f32::INFINITY,
            color: Color::WHITE,
        }
    }
}
