use crate::core::{Color, Colorf32, Pixel};
use fontdue::{Font, FontSettings};
use std::sync::Arc;
use glam::Vec2;

pub const MAX_DIMS: (usize, usize) = (3840, 2160); // 4K resolution as max
const MAX_BUFFER_SIZE: usize = MAX_DIMS.0 * MAX_DIMS.1;

pub struct WinBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u32>,
    pub depth: Vec<f32>,
    font: Arc<Font>,
}

impl WinBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        // Load font data - you can embed it in the binary
        let font_data = std::fs::read("assets/IBMPlexMono-Medium.ttf").unwrap();
        let font = Font::from_bytes(
            font_data,
            FontSettings {
                scale: 14.0,
                ..FontSettings::default()
            },
        )
        .unwrap();

        WinBuffer {
            width,
            height,
            data: vec![0; MAX_BUFFER_SIZE],
            depth: vec![f32::INFINITY; MAX_BUFFER_SIZE],
            font: Arc::new(font),
        }
    }

    #[inline]
    fn get_active_size(&self) -> usize {
        self.width * self.height
    }

    pub fn clear(&mut self) {
        let active_size = self.get_active_size();
        self.data[..active_size].fill(0);
        self.depth[..active_size].fill(f32::INFINITY);
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        assert!(
            new_width <= MAX_DIMS.0 && new_height <= MAX_DIMS.1,
            "Attempted to resize beyond maximum dimensions"
        );
        self.width = new_width;
        self.height = new_height;
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, color: Colorf32) {
        let mut cursor_x = x;
        let font_size = 14.0;

        for c in text.chars() {
            let (metrics, bitmap) = self.font.rasterize(c, font_size);

            // Draw the bitmap for this character
            for (i, &alpha) in bitmap.iter().enumerate() {
                let bx = i % metrics.width;
                let by = i / metrics.width;

                let px = cursor_x + bx as i32 + metrics.xmin;
                let py = y + by as i32 + metrics.ymin;

                if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                    let idx = (py as usize) * self.width + (px as usize);
                    if idx < self.get_active_size() {
                        let alpha_f = alpha as f32 / 255.0;
                        let text_color = Colorf32::new(
                            color.r * alpha_f,
                            color.g * alpha_f,
                            color.b * alpha_f,
                        );
                        self.data[idx] = text_color.to_u32();
                    }
                }
            }

            cursor_x += metrics.advance_width as i32;
        }
    }

    pub fn get_active_buffer(&self) -> &[u32] {
        &self.data[..self.get_active_size()]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, depth: f32, pixel: Pixel) {
        if x < self.width && y < self.height {
            let index = x + y * self.width;
            if depth < self.depth[index] {
                let color = pixel.get_color();
                self.data[index] = color.to_u32();
                self.depth[index] = depth;
            }
        }
    }
    pub fn draw_line(&mut self, start: Vec2, end: Vec2, color: Colorf32) {
        use crate::pipeline::rasterizer::bresenham;
        let pixel = Pixel::new_framebuffer(color);

        bresenham(start, end, pixel, |pos, depth, p| {
            self.set_pixel(pos.x as usize, pos.y as usize, depth, p);
        });
    }
}
