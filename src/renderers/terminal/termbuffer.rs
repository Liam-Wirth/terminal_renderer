use std::io::stdout;
use std::io::Write;

use glam::UVec2;
use glam::Vec2;

use crate::core::{Color, ProjectedVertex};

pub const MAX_DIMS: UVec2 = UVec2::new(1920, 1080);

#[derive(Clone, Copy, Debug)]
pub struct Pixel {
    pub ch: char,
    pub color: Color, // foreground color
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            ch: ' ',
            color: Color::WHITE,
        }
    }
}

impl Pixel {
    pub fn new(ch: char, color: Color) -> Self {
        Pixel { ch, color }
    }

    /// this char will be primarily used for the general rendering mode
    pub fn new_full(color: Color) -> Self {
        Pixel { ch: '█', color }
    }

    pub fn reset(&mut self) {
        self.ch = ' ';
        self.color = Color::WHITE;
    }
}

pub struct TermBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Pixel>,
    pub depth: Vec<f32>,
}

impl TermBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let max = MAX_DIMS.x as usize * MAX_DIMS.y as usize;
        TermBuffer {
            width,
            height,
            data: vec![Pixel::default(); max], // Fill buffer with spaces initially, up to max
            // predicted dimensions, done at startup once to minimize chances of re-allocation
            depth: vec![f32::INFINITY; max], // Fill buffer with spaces initially same with z-buf
        }
    }

    pub fn clear(&mut self) {
        for pixel in &mut self.data {
            pixel.reset();
        }
        for depth in &mut self.depth {
            *depth = f32::INFINITY;
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, depth: &f32 , pix: Pixel) {
        if x < self.width && y < self.height {
            let index = x + y * self.width;
            if *depth < self.depth[index] {
                self.data[index] = pix;
                self.depth[index] = *depth;
            }
        }
    }

    pub fn render_to_terminal(&self) -> std::io::Result<()> {
        let mut stdout = stdout();

        let mut output = String::new();

        // Keep track of the last color to minimize color changes
        let mut last_color = None;

        // Hide the cursor and clear the screen once
        output.push_str("\x1B[?25l"); // Hide cursor
        output.push_str("\x1B[2J"); // Clear screen
        output.push_str("\x1B[H"); // Move cursor to home position

        // For each line
        for y in 0..self.height {
            let mut x = 0;
            // Move cursor to the beginning of the line once
            output.push_str(&format!("\x1B[{};{}H", y + 1, 1));

            while x < self.width {
                let index = x + y * self.width;
                let pixel = &self.data[index];
                let current_color = pixel.color.to_ansii_escape(); // returns the ANSI escape code string

                // Accumulate characters with the same color
                let mut pixel_chars = String::new();
                while x < self.width && self.data[x + y * self.width].color == pixel.color {
                    pixel_chars.push(self.data[x + y * self.width].ch);
                    x += 1;
                }

                // Change color if necessary
                if last_color != Some(current_color.clone()) {
                    output.push_str(&current_color);
                    last_color = Some(current_color.clone());
                }

                // Append the accumulated characters
                output.push_str(&pixel_chars);
            }
        }

        // Show the cursor again
        output.push_str("\x1B[?25h");

        stdout.write_all(output.as_bytes())?;
        stdout.flush()
    }
}

pub struct BufferChunk {
    pub buffer: TermBuffer,
    pub offset: Vec2, // Primarily just do x offsets, maybe we chunk with y offsets in the future
                      // but unlikely I think
}

impl BufferChunk {
    pub fn new(width: usize, height: usize, offset: Vec2) -> Self {
        BufferChunk {
            buffer: TermBuffer::new(width, height),
            offset,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn set_pixel(
        &mut self,
        x: usize,
        y: usize,
        depth: &f32,
        pix: Pixel, // take ownership of it
    ) {
        self.buffer.set_pixel(
            x - self.offset.x as usize,
            y - self.offset.y as usize,
            depth,
            pix,
        );
    }

    pub fn render_to_terminal(&self) -> std::io::Result<()> {
        self.buffer.render_to_terminal()
    }
}
