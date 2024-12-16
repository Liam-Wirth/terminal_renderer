#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colorf32 {
    pub r: f32, // Red component (0.0 - 1.0)
    pub g: f32, // Green component (0.0 - 1.0)
    pub b: f32, // Blue component (0.0 - 1.0)
}

impl Colorf32 {
    /// Create a new color with RGB components normalized.
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// Create a color from RGBA components by blending the alpha channel into RGB.
    /// Alpha value should be in the range 0.0 - 1.0.
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        let r = r * a;
        let g = g * a;
        let b = b * a;
        Self::new(r, g, b)
    }

    /// Create a color from a hexadecimal string.
    /// Accepts formats like "#RRGGBB" or "RRGGBB".
    pub fn from_hex(hex: &str) -> Result<Self, &'static str> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err("Hex string should be 6 characters long (RRGGBB).");
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component in hex")? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green component in hex")? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component in hex")? as f32 / 255.0;

        Ok(Self::new(r, g, b))
    }
    pub fn to_crossterm_color(&self) -> crossterm::style::Color {
        crossterm::style::Color::Rgb {
            r: (self.r * 255.0) as u8,
            g: (self.g * 255.0) as u8,
            b: (self.b * 255.0) as u8,
        }
    }

    /// Convert the color to a terminal-compatible ANSI escape sequence.
    pub fn to_ansii_escape(&self) -> String {
        format!(
            "\x1b[38;2;{};{};{}m",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8
        )
    }
    pub fn to_u32(&self) -> u32 {
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        (r << 16) | (g << 8) | b
    }
}

// Example Predefined Colors
impl Colorf32 {
    pub const RED: Colorf32 = Colorf32 { r: 1.0, g: 0.0, b: 0.0 };
    pub const GREEN: Colorf32 = Colorf32 { r: 0.0, g: 1.0, b: 0.0 };
    pub const BLUE: Colorf32 = Colorf32 { r: 0.0, g: 0.0, b: 1.0 };
    pub const WHITE: Colorf32 = Colorf32 { r: 1.0, g: 1.0, b: 1.0 };
    pub const BLACK: Colorf32 = Colorf32 { r: 0.0, g: 0.0, b: 0.0 };
}

impl Default for Colorf32 {
    fn default() -> Self {
        Colorf32::WHITE // Default to white color
    }
}
