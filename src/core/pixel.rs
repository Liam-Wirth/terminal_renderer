use crate::renderers::terminal::TermBuffer;
use crate::renderers::window::WinBuffer;
use crate::core::color::Color;
use crate::core::colorf32::Colorf32;
#[derive(Clone, Copy, Debug)]
pub enum Pixel {
    Terminal {
        ch: char,
        color: Colorf32,
    },
    Framebuffer {
        color: Colorf32,
    }
}

impl Pixel {
    pub fn new_terminal(ch: char, color: Colorf32) -> Self {
        Pixel::Terminal { ch, color }
    }

    pub fn new_framebuffer(color: Colorf32) -> Self {
        Pixel::Framebuffer { color }
    }

    pub fn get_color(&self) -> Colorf32 {
        match self {
            Pixel::Terminal { color, .. } => *color,
            Pixel::Framebuffer { color } => *color,
        }
    }
}
