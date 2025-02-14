use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// TODO: In the future would be cool to look into SIMD stuff for this, possibly like vectorized accumulation of colors etc
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32, // Red component (0.0 - 1.0)
    pub g: f32, // Green component (0.0 - 1.0)
    pub b: f32, // Blue component (0.0 - 1.0)
}

// NOTE: Might be better to not have it be clamped so it's clear whats being done idk

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

    const fn hex_char_to_u8(c: char) -> u8 {
        match c {
            '0'..='9' => (c as u8) - b'0',
            'a'..='f' => (c as u8) - b'a' + 10,
            'A'..='F' => (c as u8) - b'A' + 10,
            _ => 0, // Default to 0 for invalid characters (can handle better with Result/Option if needed)
        }
    }

    /// Convert two hex characters to a single byte (u8).
    const fn hex_pair_to_u8(high: char, low: char) -> u8 {
        (Self::hex_char_to_u8(high) << 4) | Self::hex_char_to_u8(low)
    }

    /// ONLY FOR INTERNAL USE!!! DO NOT USE OTHERWISE!
    fn hex(hex: &str) -> Self {
        let bytes = hex.as_bytes(); // Convert to bytes for indexing
        let offset = if bytes[0] == b'#' { 1 } else { 0 }; // Handle optional `#`

        let r =
            Self::hex_pair_to_u8(bytes[offset] as char, bytes[offset + 1] as char) as f32 / 255.0;
        let g = Self::hex_pair_to_u8(bytes[offset + 2] as char, bytes[offset + 3] as char) as f32
            / 255.0;
        let b = Self::hex_pair_to_u8(bytes[offset + 4] as char, bytes[offset + 5] as char) as f32
            / 255.0;

        Self { r, g, b }
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

    pub fn lerp(self, end: &Color, t: f32) -> Color {
        self * (1.0 - t) + *end * t
    }

    /// Produces new version of self that is clamped from 0 to 1
    pub fn clamped(&self) -> Color {
        let r = self.r.clamp(0., 1.);
        let g = self.g.clamp(0., 1.);
        let b = self.b.clamp(0., 1.);
        Color { r, g, b }
    }
    pub fn clamp(&mut self) {
        self.r = self.r.clamp(0., 1.);
        self.g = self.g.clamp(0., 1.);
        self.b = self.b.clamp(0., 1.);
    }

    pub fn accumulate(&mut self, colors: &[Color]) {
        for color in colors {
            Color::add_assign(self, *color);
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
        let srb = start & 0xFF00FF;
        let sg = (start >> 8) & 0x00FF00; // Green is in middle 8 bits

        let erb = end & 0xFF00FF; // Corrected mask
        let eg = (end >> 8) & 0x00FF00;

        let rb = ((srb as f32 * inv_t + erb as f32 * t) as u32) & 0xFF00FF;
        let g = ((sg as f32 * inv_t + eg as f32 * t) as u32) & 0x00FF00;

        (rb | g) & 0x00FFFFFF
    }
}

// Predefined colors

impl Default for Color {
    fn default() -> Self {
        Color::WHITE // Default to white color
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from(t: (f32, f32, f32)) -> Self {
        Self {
            r: t.0,
            g: t.1,
            b: t.2,
        }
    }
}

impl From<f32> for Color {
    fn from(t: f32) -> Self {
        Self { r: t, g: t, b: t }.clamped()
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(t: (u8, u8, u8)) -> Self {
        Self {
            r: (t.0 as f32) / 255.,
            g: (t.1 as f32) / 255.,
            b: (t.2 as f32) / 255.,
        }
    }
}

impl Neg for Color {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            r: 1.0 - self.r,
            g: 1.0 - self.g,
            b: 1.0 - self.b,
        }
    }
}

impl Add for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Color {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
        .clamped() // Cause 0-1
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.clamp()
    }
}

impl Sub for Color {
    type Output = Color;
    fn sub(self, rhs: Color) -> Color {
        Self {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
        }
        .clamped()
    }
}

impl SubAssign for Color {
    fn sub_assign(&mut self, rhs: Color) {
        self.r -= rhs.r;
        self.g -= rhs.g;
        self.b -= rhs.b;
        self.clamp()
    }
}

impl Div<f32> for Color {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self {
            r: self.r / scalar,
            g: self.g / scalar,
            b: self.b / scalar,
        }
    }
}

impl DivAssign<f32> for Color {
    fn div_assign(&mut self, scalar: f32) {
        self.r /= scalar;
        self.g /= scalar;
        self.b /= scalar;
    }
}

impl Div for Color {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            r: self.r / other.r,
            g: self.g / other.g,
            b: self.b / other.b,
        }
    }
}

impl DivAssign for Color {
    fn div_assign(&mut self, other: Self) {
        self.r /= other.r;
        self.g /= other.g;
        self.b /= other.b;
    }
}

impl Mul<f32> for Color {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
        }
        .clamped()
    }
}
impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, scalar: f32) {
        self.r *= scalar;
        self.g *= scalar;
        self.b *= scalar;
        self.clamp();
    }
}
impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        (self.r - other.r).abs() < f32::EPSILON
            && (self.g - other.g).abs() < f32::EPSILON
            && (self.b - other.b).abs() < f32::EPSILON
    }
}

impl Mul<&Color> for Color {
    type Output = Color;

    fn mul(self, rhs: &Color) -> Color {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
        .clamped() // Clamping to keep values within valid range
    }
}

impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Color {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
        .clamped()
    }
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    pub const LIGHT_GRAY: Color = Color {
        r: 0.827,
        g: 0.827,
        b: 0.827,
    };
    pub const DARK_GRAY: Color = Color {
        r: 0.211,
        g: 0.215,
        b: 0.215,
    };
    pub const GRAY: Color = Color {
        r: 0.502,
        g: 0.502,
        b: 0.502,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
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
    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
    };
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
    };
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
    };
    pub const ORANGE: Color = Color {
        r: 1.0,
        g: 0.647,
        b: 0.0,
    };
    pub const PURPLE: Color = Color {
        r: 0.502,
        g: 0.0,
        b: 0.502,
    };
    pub const PINK: Color = Color {
        r: 1.0,
        g: 0.753,
        b: 0.796,
    };
    pub const BROWN: Color = Color {
        r: 0.647,
        g: 0.165,
        b: 0.165,
    };
    pub const GOLD: Color = Color {
        r: 1.0,
        g: 0.843,
        b: 0.0,
    };
    pub const SILVER: Color = Color {
        r: 0.753,
        g: 0.753,
        b: 0.753,
    };
    pub const TEAL: Color = Color {
        r: 0.0,
        g: 0.502,
        b: 0.502,
    };
    pub const NAVY: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.502,
    };
    pub const MAROON: Color = Color {
        r: 0.502,
        g: 0.0,
        b: 0.0,
    };
    pub const OLIVE: Color = Color {
        r: 0.502,
        g: 0.502,
        b: 0.0,
    };
    pub const LIME: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
    };
    pub const AQUA: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
    };
    pub const FUCHSIA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
    };
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_lerp() {
        let iterations = 10_000_000;

        // Test float lerp
        let start_color = Color::new(1.0, 0.0, 0.0);
        let end_color = Color::new(0.0, 1.0, 0.0);
        let start_time = Instant::now();
        for t in 0..iterations {
            let t = (t as f32) / (iterations as f32);
            let _result = start_color.lerp(&end_color, t);
        }
        let float_time = start_time.elapsed();

        // Test u32 lerp
        let start_u32 = start_color.to_u32();
        let end_u32 = end_color.to_u32();
        let start_time = Instant::now();
        for t in 0..iterations {
            let t = (t as f32) / (iterations as f32);
            let _result = Color::lerp_u32(start_u32, end_u32, t);
        }
        let u32_time = start_time.elapsed();

        println!("Float lerp took: {:?}", float_time);
        println!("For an average of: {:?}", float_time / iterations);
        println!("U32 lerp took: {:?}", u32_time);
        println!("For an average of: {:?}", u32_time / iterations);
        println!(
            "Speed difference: {:.2}x",
            float_time.as_nanos() as f64 / u32_time.as_nanos() as f64
        );
    }
}
