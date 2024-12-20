#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32, // Red component (0.0 - 1.0)
    pub g: f32, // Green component (0.0 - 1.0)
    pub b: f32, // Blue component (0.0 - 1.0)
}

impl Color {
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

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component in hex")?
            as f32
            / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green component in hex")?
            as f32
            / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component in hex")?
            as f32
            / 255.0;

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

    pub fn lerp(&self, end: &Color, t: f32) -> Color {
        Color {
            r: self.r + (end.r - self.r) * t,
            g: self.g + (end.g - self.g) * t,
            b: self.b + (end.b - self.b) * t,
        }
    }
    // I whent overboard on optimizations here
    // fast af boiiiii
    // this works cause:
    /*
    u32 RGB is just 0xXXRRGGBB (in the context of minifb)

    mask out red and blue chanels (no overlap, so you can operate on em at the same time)

    mask out green chanel

    apply lerp to rb then mask out possible garbage bits: (& 0xFF00FF)
    apply lerp to g then mask out possible garbage bits: (& 0x00FF00)

    bitwise or rb and g together to get rgb
    rb | g

    mask out potential garbage bits in 0xXXRRGGBB (possibly uneccessary)

    rb | g & 0x00FFFFFF;

    */
    pub fn lerp_u32(start: u32, end: u32, t: f32) -> u32 {
        let inv_t = 1.0 - t;
        // separate out rgbs:
        // let sr = (start >> 16) & 0xFF;
        // let sg = (start >> 8) & 0xFF;
        // let sb = start & 0xFF;
        //
        // let er = (end >> 16) & 0xFF;
        // let eg = (end >> 8) & 0xFF;
        // let eb = end & 0xFF;

        // MASKED AS FREAK!!!
        let srb = start & 0xFF00FF;
        let sg = start & 0x00FF00;

        let erb = end & 0x0FF00;
        let eg = end & 0x00FF00;

        let rb = ((srb as f32) * inv_t + erb as f32 * t) as u32 & 0xFF00FF;
        let g = ((sg as f32 * inv_t + eg as f32 * t) as u32) & 0x00FF00;

        (rb | g) & 0x00FFFFFF
    }
}

// Example Predefined Colors
impl Color {
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
}

impl Default for Color {
    fn default() -> Self {
        Color::WHITE // Default to white color
    }
}
