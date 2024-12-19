use crate::core::color::Color;
#[derive(Clone, Copy, Debug)]
pub enum Pixel {
    Terminal { ch: char, color: Color },
    Framebuffer { color: Color },
}


impl Pixel {
    pub fn new_terminal(ch: char, color: Color) -> Self {
        Pixel::Terminal { ch, color }
    }
    pub fn default_term() -> Self {
        let ch = ' ';
        let color = Color::WHITE;
        Pixel::new_terminal(ch, color)
    }

    pub fn term_full(color: Color) -> Self {
        let ch = '█';
        Pixel::new_terminal(ch, color)
    }

    pub fn reset(&mut self) {
        match self {
            Pixel::Terminal { ch, color } => {
                *ch = ' ';
                *color = Color::WHITE;
            }
            Pixel::Framebuffer { color } => *color = Color::WHITE,
        }
    }

    pub fn new_framebuffer(color: Color) -> Self {
        Pixel::Framebuffer { color }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Pixel::Terminal { color, .. } => *color,
            Pixel::Framebuffer { color } => *color,
        }
    }
}
