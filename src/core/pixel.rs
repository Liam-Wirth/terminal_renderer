use crate::core::color::Color;
#[derive(Clone, Copy, Debug)]
pub enum Pixel {
    Terminal { ch: char, color: Color },
    Framebuffer(u32), // Store u32 directly instead of Color
}

impl Pixel {
    pub fn new_terminal(ch: char, color: Color) -> Self {
        Pixel::Terminal { ch, color }
    }

    pub fn new_framebuffer(color: Color) -> Self {
        Pixel::Framebuffer(color.to_u32()) // Convert to u32 once, at creation
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
            Pixel::Framebuffer(color) => *color = Color::WHITE.to_u32(),
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Pixel::Terminal { color, .. } => *color,
            Pixel::Framebuffer(color) => {
                // Only convert back to Color when explicitly needed
                Color::new(
                    ((*color >> 16) & 0xFF) as f32 / 255.0,
                    ((*color >> 8) & 0xFF) as f32 / 255.0,
                    (*color & 0xFF) as f32 / 255.0,
                )
            }
        }
    }

    pub fn ch(&self) -> char {
        match self {
            Pixel::Terminal { ch, .. } => *ch,
            Pixel::Framebuffer(_) => ' ',
        }
    }

}
