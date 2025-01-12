use crate::core::{Color, Pixel};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    style::{Print, SetForegroundColor},
    QueueableCommand,
};
use glam::UVec2;
use minifb::Window;
use rayon::iter::ParallelIterator;
use rayon::slice::ParallelSliceMut;
use std::io::{self, Write};
pub const MAX_DIMS: UVec2 = UVec2::new(1920, 1080);

// Note might be worth going back to refcells for interior mutability

pub trait Buffer {
    type Pixel: Clone + Send + Sync;

    fn new(width: usize, height: usize) -> Self
    where
        Self: Sized;
    fn clear(&mut self);
    fn create_pixel(color: Color) -> Self::Pixel;
    fn set_pixel(&mut self, pos: (usize, usize), depth: &f32, pixel: Self::Pixel);
    fn draw_line(&mut self, start: UVec2, end: UVec2, pixel: Self::Pixel);
    fn present(&self) -> io::Result<()> {
        Ok(()) // Default does nothin
    }
    fn present_window(&self, _window: &mut Window) -> io::Result<()> {
        Ok(()) // Default implementation does nothing
    }
}

pub struct TermBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Pixel>,
    pub depth: Vec<f32>,
}

impl Buffer for TermBuffer {
    type Pixel = Pixel;

    fn new(width: usize, height: usize) -> Self {
        let _max = MAX_DIMS.x as usize * MAX_DIMS.y as usize;
        TermBuffer {
            width,
            height,
            data: vec![Pixel::default_term(); width * height], // Initialize with default pixels
            depth: vec![f32::INFINITY; width * height],        // Initialize depth buffer
        }
    }

    fn clear(&mut self) {
        let buf_size = self.width * self.height;
        self.data[..buf_size]
            .par_chunks_mut(1024)
            .for_each(|chunk| {
                for point in chunk {
                    point.reset();
                }
            });
        self.depth[..buf_size]
            .par_chunks_mut(1024)
            .for_each(|chunk| {
                for depth in chunk {
                    *depth = f32::INFINITY;
                }
            });
    }

    fn set_pixel(&mut self, pos: (usize, usize), depth: &f32, pixel: Self::Pixel) {
        if pos.0 < self.width && pos.1 < self.height {
            let index = pos.0 + pos.1 * self.width;
            if *depth < self.depth[index] {
                self.data[index] = pixel;
                self.depth[index] = *depth;
            }
        }
    }

    fn create_pixel(color: Color) -> Self::Pixel {
        Pixel::new_terminal('█', color)
    }

    fn draw_line(&mut self, start: UVec2, end: UVec2, pixel: Self::Pixel) {
        let start = (start.x as f32, start.y as f32);
        let end = (end.x as f32, end.y as f32);

        crate::pipeline::rasterizer::bresenham(
            start.into(),
            end.into(),
            pixel,
            |pos, depth, pixel| {
                self.set_pixel((pos.x as usize, pos.y as usize), &depth, pixel);
            },
        );
    }

    fn present(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Use Crossterm's queueing system to minimize syscalls
        stdout.queue(MoveTo(0, 0))?;
        stdout.queue(Hide)?;

        for y in 0..self.height {
            for x in 0..self.width {
                let index = x + y * self.width;
                if let Pixel::Terminal { ch, color } = self.data[index] {
                    stdout.queue(SetForegroundColor(color.to_crossterm_color()))?;
                    stdout.queue(Print(ch))?;
                }
            }
            // Print a newline but avoid scrolling on last line:
            if y < self.height - 1 {
                stdout.queue(Print("\r\n"))?;
            }
        }

        stdout.queue(Show)?;
        stdout.flush()?;
        Ok(())
    }
}

pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u32>,
    pub depth: Vec<f32>,
    pub window: *mut Window, // Babies first Unsafe code
}

impl Buffer for FrameBuffer {
    type Pixel = Pixel;

    fn new(width: usize, height: usize) -> Self {
        let buf_size = width * height;
        FrameBuffer {
            width,
            height,
            data: vec![0; buf_size], // White background initially
            depth: vec![f32::INFINITY; buf_size],
            window: std::ptr::null_mut(), // Will be set externally
        }
    }

    fn clear(&mut self) {
        let _buf_size = self.width * self.height;
        self.data.par_chunks_mut(1024).for_each(|chunk| {
            for point in chunk {
                *point = 0; // Reset to white
            }
        });
        self.depth.par_chunks_mut(1024).for_each(|chunk| {
            for d in chunk {
                *d = f32::INFINITY;
            }
        });
    }

    fn set_pixel(&mut self, pos: (usize, usize), depth: &f32, pixel: Self::Pixel) {
        if pos.0 < self.width && pos.1 < self.height {
            let index = pos.0 + pos.1 * self.width;
            if *depth < self.depth[index] {
                // Pixel::Framebuffer(u32) is what we expect
                match pixel {
                    Pixel::Framebuffer(color) => {
                        self.data[index] = color;
                        self.depth[index] = *depth;
                    }
                    Pixel::Terminal { color, .. } => {
                        // Just convert terminal color to u32 for uniformity
                        self.data[index] = color.to_u32();
                        self.depth[index] = *depth;
                    }
                }
            }
        }
    }

    fn create_pixel(color: Color) -> Self::Pixel {
        Pixel::new_framebuffer(color)
    }

    fn draw_line(&mut self, start: glam::UVec2, end: glam::UVec2, pixel: Self::Pixel) {
        let start = (start.x as f32, start.y as f32);
        let end = (end.x as f32, end.y as f32);

        crate::pipeline::rasterizer::bresenham(
            start.into(),
            end.into(),
            pixel,
            |pos, depth, pixel| {
                self.set_pixel((pos.x as usize, pos.y as usize), &depth, pixel);
            },
        );
    }

    fn present_window(&self, window: &mut Window) -> io::Result<()> {
        window
            .update_with_buffer(&self.data, self.width, self.height)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    }
}

impl FrameBuffer {
    pub fn attach_window(&mut self, window: &mut Window) {
        self.window = window as *mut Window;
    }
}
