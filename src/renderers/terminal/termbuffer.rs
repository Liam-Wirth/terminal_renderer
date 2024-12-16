use std::io::stdout;
use std::io::Write;

use glam::{UVec2, Vec2};
use rayon::iter::ParallelIterator;
use rayon::prelude::*;


use crate::core::Colorf32;
use crate::core::Pixel;
pub const MAX_DIMS: UVec2 = UVec2::new(1920, 1080);

pub struct TermBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Pixel>,
    pub depth: Vec<f32>,
}

impl TermBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![Pixel::new_terminal(' ', Colorf32::BLACK); width * height],
            depth: vec![f32::INFINITY; width * height],
        }
    }
    pub fn set_pixel(&mut self, x: usize, y: usize, depth: f32, pixel: Pixel) {
        if x < self.width && y < self.height {
            let index = x + y * self.width;
            if depth < self.depth[index] {
                self.data[index] = pixel;
                self.depth[index] = depth;
            }
        }
    }
    pub fn draw_line(&mut self, start: Vec2, end: Vec2, color: Colorf32) {
        use crate::pipeline::rasterizer::bresenham;
        let pixel = Pixel::new_terminal('█', color);

        bresenham(start, end, pixel, |pos, depth, p| {
            self.set_pixel(pos.x as usize, pos.y as usize, depth, p);
        });
    }

    pub fn render_to_terminal(&self, metrics: &String) -> std::io::Result<()> {
        let mut stdout = stdout();
        let mut output = String::new();
        let mut last_color = None;

        output.push_str(&format!("\x1b]0;{}\x07", metrics));
        output.push_str("\x1B[?25l");
        output.push_str("\x1B[2J");
        output.push_str("\x1B[H");

        for y in 0..self.height {
            let mut x = 0;
            output.push_str(&format!("\x1B[{};{}H", y + 1, 1));

            while x < self.width {
                let index = x + y * self.width;
                if let Pixel::Terminal { ch, color } = self.data[index] {
                    let current_color = color.to_ansii_escape();

                    let mut pixel_chars = String::new();
                    while x < self.width {
                        if let Pixel::Terminal { ch: next_ch, color: next_color } = self.data[x + y * self.width] {
                            if next_color == color {
                                pixel_chars.push(next_ch);
                                x += 1;
                                continue;
                            }
                        }
                        break;
                    }

                    if last_color != Some(current_color.clone()) {
                        output.push_str(&current_color);
                        last_color = Some(current_color);
                    }
                    output.push_str(&pixel_chars);
                }
            }
        }

        output.push_str("\x1B[?25h");
        stdout.write_all(output.as_bytes())?;
        stdout.flush()
    }
}
