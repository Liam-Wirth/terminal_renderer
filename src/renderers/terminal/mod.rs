// src/renderers/terminal/mod.rs
use crossterm::{
    cursor::{Hide, Show},
    event::DisableMouseCapture,
    execute,
    style::SetBackgroundColor,
    terminal::{self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
mod termbuffer;
pub use termbuffer::TermBuffer;
use log::error;
use std::io::{self, stdout};
use std::panic;
use std::time::Instant;
use crate::pipeline::{Pipeline, TerminalPipeline};
use crate::core::{Color, Scene};
pub struct TerminalRenderer {
    pipeline: TerminalPipeline,
    back_buffer: TermBuffer,
}

impl TerminalRenderer {
    pub fn new() -> io::Result<Self> {
        // Set up panic hook for terminal cleanup
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            let _ = Self::cleanup_terminal();
            eprintln!("\n=== Panic Occurred ===");
            original_hook(panic_info);
            error!("Panic occurred: {:?}", panic_info);
        }));

        // Initialize terminal
        let mut stdout = stdout();
        enable_raw_mode()?;
        execute!(
            stdout,
            EnterAlternateScreen,
            Hide,
            SetBackgroundColor(Color::BLACK.to_crossterm_color()),
        )?;

        let (width, height) = terminal::size()?;
        let pipeline = TerminalPipeline::new(width as usize, height as usize)?;
        let back_buffer = TermBuffer::new(width as usize, height as usize);

        Ok(Self {
            pipeline,
            back_buffer,
        })
    }

    fn cleanup_terminal() -> io::Result<()> {
        let mut stdout = stdout();
        disable_raw_mode()?;
        execute!(stdout, Show, LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn run(&mut self, scene: &mut Scene) -> io::Result<()> {
        loop {
            let now = Instant::now();
            let delta = now - self.pipeline.last_frame;

            if delta >= self.pipeline.frame_time {
                // Process one frame
                let processed = self.pipeline.process_geometry(&scene, &scene.camera);
                let fragments = self.pipeline.rasterize(processed);
                self.pipeline.process_fragments(fragments, &mut self.back_buffer);
                self.pipeline.present(&mut self.back_buffer)?;
                self.pipeline.update_metrics(delta, &scene.camera);
                self.pipeline.last_frame = now;
            }

            if let Ok(true) = self.pipeline.handle_input(&mut scene.camera) {
                break;
            }
        }

        Self::cleanup_terminal()
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        let _ = Self::cleanup_terminal();
    }
}
