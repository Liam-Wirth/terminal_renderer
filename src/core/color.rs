// TODO: implement like a "Colors" enum to be able to more quickly access the predefined colors or
// something
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8, // Red component (0-255)
    pub g: u8, // Green component (0-255)
    pub b: u8, // Blue component (0-255)
}

impl Color {
    /// Create a new color with RGB components.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create a color from RGBA components by blending the alpha channel into RGB.
    /// Alpha value should be in the range 0-255.
    /// The resulting RGB values are scaled by the alpha channel.
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        let alpha = a as f64 / 255.0; // Normalize alpha to range [0, 1]

        let r = ((r as f64) * alpha).round() as u8;
        let g = ((g as f64) * alpha).round() as u8;
        let b = ((b as f64) * alpha).round() as u8;

        Self { r, g, b }
    }

    /// Create a color from a hexadecimal string.
    ///
    /// Accepts formats like "#RRGGBB" or "RRGGBB".
    pub fn from_hex(hex: &str) -> Result<Self, &'static str> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err("Hex string should be 6 characters long (RRGGBB).");
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component in hex")?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green component in hex")?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component in hex")?;

        Ok(Self::new(r, g, b))
    }

    pub fn to_crossterm_color(&self) -> crossterm::style::Color {
        crossterm::style::Color::Rgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
    ///chat-gpt generated colors
    // Primary Colors
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };

    // Secondary Colors
    pub const CYAN: Color = Color {
        r: 0,
        g: 255,
        b: 255,
    };
    pub const MAGENTA: Color = Color {
        r: 255,
        g: 0,
        b: 255,
    };
    pub const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
    };

    // Tertiary Colors
    pub const ORANGE: Color = Color {
        r: 255,
        g: 165,
        b: 0,
    };
    pub const LIME: Color = Color {
        r: 191,
        g: 255,
        b: 0,
    };
    pub const PURPLE: Color = Color {
        r: 128,
        g: 0,
        b: 128,
    };
    pub const TEAL: Color = Color {
        r: 0,
        g: 128,
        b: 128,
    };
    pub const VIOLET: Color = Color {
        r: 238,
        g: 130,
        b: 238,
    };
    pub const INDIGO: Color = Color {
        r: 75,
        g: 0,
        b: 130,
    };

    // Neutral Colors
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
    };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const DARK_GRAY: Color = Color {
        r: 64,
        g: 64,
        b: 64,
    };
    pub const GRAY: Color = Color {
        r: 128,
        g: 128,
        b: 128,
    };
    pub const LIGHT_GRAY: Color = Color {
        r: 192,
        g: 192,
        b: 192,
    };

    // Pastel Colors
    pub const PASTEL_PINK: Color = Color {
        r: 255,
        g: 182,
        b: 193,
    };
    pub const PASTEL_GREEN: Color = Color {
        r: 119,
        g: 221,
        b: 119,
    };
    pub const PASTEL_BLUE: Color = Color {
        r: 174,
        g: 198,
        b: 207,
    };
    pub const PASTEL_YELLOW: Color = Color {
        r: 253,
        g: 253,
        b: 150,
    };
    pub const PASTEL_PURPLE: Color = Color {
        r: 179,
        g: 158,
        b: 181,
    };

    // Dark Shades
    pub const DARK_RED: Color = Color { r: 139, g: 0, b: 0 };
    pub const DARK_GREEN: Color = Color { r: 0, g: 100, b: 0 };
    pub const DARK_BLUE: Color = Color { r: 0, g: 0, b: 139 };
    pub const DARK_MAGENTA: Color = Color {
        r: 139,
        g: 0,
        b: 139,
    };
    pub const DARK_CYAN: Color = Color {
        r: 0,
        g: 139,
        b: 139,
    };
    pub const DARK_ORANGE: Color = Color {
        r: 255,
        g: 140,
        b: 0,
    };

    // Bright Colors
    /// #ff453a
    pub const BRIGHT_RED: Color = Color {
        r: 255,
        g: 69,
        b: 0,
    };
    pub const BRIGHT_GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 127,
    };
    pub const BRIGHT_BLUE: Color = Color {
        r: 0,
        g: 191,
        b: 255,
    };
    pub const BRIGHT_YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 102,
    };
    pub const BRIGHT_PINK: Color = Color {
        r: 255,
        g: 20,
        b: 147,
    };

    // Earth Tones
    pub const BROWN: Color = Color {
        r: 139,
        g: 69,
        b: 19,
    };
    pub const TAN: Color = Color {
        r: 210,
        g: 180,
        b: 140,
    };
    pub const SAND: Color = Color {
        r: 194,
        g: 178,
        b: 128,
    };
    pub const OLIVE: Color = Color {
        r: 128,
        g: 128,
        b: 0,
    };
    pub const FOREST_GREEN: Color = Color {
        r: 34,
        g: 139,
        b: 34,
    };

    // Warm Colors
    pub const CRIMSON: Color = Color {
        r: 220,
        g: 20,
        b: 60,
    };
    pub const GOLD: Color = Color {
        r: 255,
        g: 215,
        b: 0,
    };
    pub const SALMON: Color = Color {
        r: 250,
        g: 128,
        b: 114,
    };
    pub const CORAL: Color = Color {
        r: 255,
        g: 127,
        b: 80,
    };
    pub const FIREBRICK: Color = Color {
        r: 178,
        g: 34,
        b: 34,
    };

    // Cool Colors
    pub const SKY_BLUE: Color = Color {
        r: 135,
        g: 206,
        b: 235,
    };
    pub const SLATE_BLUE: Color = Color {
        r: 106,
        g: 90,
        b: 205,
    };
    pub const STEEL_BLUE: Color = Color {
        r: 70,
        g: 130,
        b: 180,
    };
    pub const TURQUOISE: Color = Color {
        r: 64,
        g: 224,
        b: 208,
    };
    pub const AQUAMARINE: Color = Color {
        r: 127,
        g: 255,
        b: 212,
    };

    // Metallic Colors
    pub const SILVER: Color = Color {
        r: 192,
        g: 192,
        b: 192,
    };
    pub const GOLDENROD: Color = Color {
        r: 218,
        g: 165,
        b: 32,
    };
    pub const BRONZE: Color = Color {
        r: 205,
        g: 127,
        b: 50,
    };

    pub const MAROON: Color = Color { r: 128, g: 0, b: 0 };
    pub const NAVY: Color = Color { r: 0, g: 0, b: 128 };
    pub const BEIGE: Color = Color {
        r: 245,
        g: 245,
        b: 220,
    };
    pub const LAVENDER: Color = Color {
        r: 230,
        g: 230,
        b: 250,
    };
    pub const MINT: Color = Color {
        r: 189,
        g: 252,
        b: 201,
    };
}

impl Default for Color {
    fn default() -> Self {
        Color::WHITE // Default to white color
    }
}
