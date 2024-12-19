use crate::core::{Color, Pixel};
use fontdue::{Font, FontSettings};
use glam::UVec2;
use rayon::iter::ParallelIterator;
use rayon::prelude::*;
use rayon::slice::ParallelSliceMut;
use std::io;
use std::sync::Arc;
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
    fn present(&self) -> io::Result<()>;
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
        let max = MAX_DIMS.x as usize * MAX_DIMS.y as usize;
        TermBuffer {
            width,
            height,
            data: vec![Pixel::default_term()],
            depth: vec![f32::INFINITY; max],
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
        // TODO: Implement line drawing
    }

    fn present(&self) -> io::Result<()> {
        // TODO: Implement terminal rendering
        Ok(())
    }
}

pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Pixel>,
    pub depth: Vec<f32>,
}

impl Buffer for FrameBuffer {
    type Pixel = Pixel;

    fn new(width: usize, height: usize) -> Self {
        let max = MAX_DIMS.x as usize * MAX_DIMS.y as usize;
        FrameBuffer {
            width,
            height,
            data: vec![Pixel::new_framebuffer(Color::WHITE); max],
            depth: vec![f32::INFINITY; max],
        }
    }

    fn clear(&mut self) {
        self.data.par_chunks_mut(1024).for_each(|chunk| {
            for point in chunk {
                point.reset();
            }
        });
        self.depth[..self.width * self.height]
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
            self.data[index] = pixel;
            self.depth[index] = *depth;
        }
    }

    fn draw_line(&mut self, start: UVec2, end: UVec2, pixel: Self::Pixel) {
        // TODO: Implement line drawing
    }

    fn present(&self) -> io::Result<()> {
        // TODO: Implement framebuffer rendering
        Ok(())
    }

    fn create_pixel(color: Color) -> Self::Pixel {
        Pixel::new_framebuffer(color)
    }
}
