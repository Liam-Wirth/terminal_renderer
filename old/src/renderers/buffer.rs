use crate::core::{camera::ProjectedVertex, color::Color};
use std::io::{stdout, Write};

use super::cpu_termrenderer::Pixel;

pub struct Buffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Pixel>,
    pub depth: Vec<f32>,
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Buffer {
            width,
            height,
            data: vec![Pixel::default(); width * height], // Fill buffer with spaces initially
            depth: vec![f32::INFINITY; width * height],   // Fill buffer with spaces initially
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

        pub fn set_pixel(&mut self, x: usize, y: usize, projected: &ProjectedVertex, ch: char, color: Color) {
        if x < self.width && y < self.height {
            let index = x + y * self.width;
            if projected.depth < self.depth[index] {
                self.data[index] = Pixel::new(ch, color);
                self.depth[index] = projected.depth;
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
