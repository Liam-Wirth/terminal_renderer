mod winbuffer;
pub use winbuffer::WinBuffer;

use crate::core::Scene;
use crate::pipeline::{Pipeline, WindowPipeline};
use std::io;
use std::time::Instant;

pub struct WindowRenderer {
    pipeline: WindowPipeline,
    back_buffer: WinBuffer,
}

impl WindowRenderer {
    pub fn new(width: usize, height: usize) -> io::Result<Self> {
        let pipeline = WindowPipeline::new(width, height)?;
        Ok(Self { pipeline, back_buffer: WinBuffer::new(width, height) })
    }

    pub fn run(&mut self, scene: &mut Scene) -> io::Result<()> {
        while self.pipeline.window.is_open() {
            let now = Instant::now();
            let delta = now - self.pipeline.last_frame;

            if delta >= self.pipeline.frame_time {
                // Process one frame
                let processed = self.pipeline.process_geometry(&scene, &scene.camera);
                let fragments = self.pipeline.rasterize(processed, &scene);

                // Get mutable reference to back buffer once
                let back_buffer = &mut self.back_buffer;
                self.pipeline.process_fragments(fragments, back_buffer);
                self.pipeline.present(back_buffer)?;
                self.pipeline.update_metrics(delta, &scene.camera);
                self.pipeline.last_frame = now;
                self.pipeline.window.set_title(&self.pipeline.metrics);
            }

            if let Ok(true) = self.pipeline.handle_input(&mut scene.camera) {
                break;
            }
        }
        Ok(())
    }
}
