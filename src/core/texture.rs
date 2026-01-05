use crate::core::Color;
use glam::Vec2;
use image::{ImageBuffer, Rgb, RgbImage};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Represents a loaded texture with sampling capabilities
#[derive(Debug, Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: Arc<Vec<Color>>,
    pub path: String,
}

impl Texture {
    /// Load a texture from a file path
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        // Try to load the image
        let img = image::open(path)
            .map_err(|e| format!("Failed to load texture '{}': {}", path, e))?;
        
        // Convert to RGB8 format
        let rgb_img = img.to_rgb8();
        let (width, height) = rgb_img.dimensions();
        
        // Convert pixel data to our Color format
        let mut color_data = Vec::with_capacity((width * height) as usize);
        for pixel in rgb_img.pixels() {
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            color_data.push(Color::new(r, g, b));
        }
        
        Ok(Texture {
            width,
            height,
            data: Arc::new(color_data),
            path: path.to_string(),
        })
    }
    
    /// Create a default 1x1 white texture
    pub fn default_white() -> Self {
        Texture {
            width: 1,
            height: 1,
            data: Arc::new(vec![Color::WHITE]),
            path: "default_white".to_string(),
        }
    }
    
    /// Create a default 1x1 normal map (RGB: 128, 128, 255 = normal pointing up)
    pub fn default_normal() -> Self {
        Texture {
            width: 1,
            height: 1,
            data: Arc::new(vec![Color::new(0.5, 0.5, 1.0)]), // Default normal map color
            path: "default_normal".to_string(),
        }
    }
    
    /// Sample the texture at UV coordinates using bilinear filtering
    pub fn sample(&self, uv: Vec2) -> Color {
        self.sample_filtered(uv, TextureFilter::Bilinear)
    }

    pub fn sample_direct(&self, uv: Vec2) -> Color {
        self.sample_filtered(uv, TextureFilter::Nearest)
    }
    
    /// Sample the texture with specified filtering
    pub fn sample_filtered(&self, uv: Vec2, filter: TextureFilter) -> Color {
        // Wrap UV coordinates to [0, 1] using euclidean modulo (handles negatives).
        let u = wrap_uv(uv.x);
        let v = wrap_uv(1.0 - uv.y);
        
        match filter {
            TextureFilter::Nearest => self.sample_nearest(u, v),
            TextureFilter::Bilinear => self.sample_bilinear(u, v),
        }
    }
    
    /// Sample using nearest neighbor filtering
    fn sample_nearest(&self, u: f32, v: f32) -> Color {
        let x = ((u * self.width as f32) as u32).min(self.width - 1);
        let y = ((v * self.height as f32) as u32).min(self.height - 1);
        let index = (y * self.width + x) as usize;
        self.data[index]
    }
    
    /// Sample using bilinear filtering
    fn sample_bilinear(&self, u: f32, v: f32) -> Color {
        let x_f = u * (self.width - 1) as f32;
        let y_f = v * (self.height - 1) as f32;
        
        let x0 = x_f.floor() as u32;
        let y0 = y_f.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);
        
        let dx = x_f - x0 as f32;
        let dy = y_f - y0 as f32;
        
        // Get the four neighboring pixels
        let c00 = self.data[(y0 * self.width + x0) as usize];
        let c10 = self.data[(y0 * self.width + x1) as usize];
        let c01 = self.data[(y1 * self.width + x0) as usize];
        let c11 = self.data[(y1 * self.width + x1) as usize];
        
        // Bilinear interpolation
        let c0 = c00.lerp(&c10, dx);
        let c1 = c01.lerp(&c11, dx);
        c0.lerp(&c1, dy)
    }
}

fn wrap_uv(value: f32) -> f32 {
    value.rem_euclid(1.0)
}

/// Texture filtering modes
#[derive(Debug, Clone, Copy)]
pub enum TextureFilter {
    Nearest,
    Bilinear,
}

/// Texture manager for loading and caching textures
pub struct TextureManager {
    textures: HashMap<String, Arc<Texture>>,
    base_path: String,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            base_path: normalize_base_path("assets/"),
        }
    }
    
    pub fn with_base_path(base_path: &str) -> Self {
        Self {
            textures: HashMap::new(),
            base_path: normalize_base_path(base_path),
        }
    }
    
    /// Load a texture and cache it
    pub fn load_texture(&mut self, path: &str) -> Result<Arc<Texture>, String> {
        let normalized_path = normalize_texture_path(path);

        // Check if already loaded
        if let Some(texture) = self.textures.get(&normalized_path) {
            return Ok(texture.clone());
        }

        let full_path = self
            .resolve_texture_path(&normalized_path)
            .ok_or_else(|| {
                format!(
                    "Texture '{}' not found (base path: '{}')",
                    normalized_path, self.base_path
                )
            })?;

        let texture = Texture::load_from_file(full_path.to_string_lossy().as_ref())?;
        let arc_texture = Arc::new(texture);
        
        // Cache the loaded texture
        self.textures
            .insert(normalized_path, arc_texture.clone());
        Ok(arc_texture)
    }
    
    /// Get a texture from cache, loading it if necessary
    pub fn get_texture(&mut self, path: &str) -> Arc<Texture> {
        match self.load_texture(path) {
            Ok(texture) => texture,
            Err(e) => {
                println!("Warning: Failed to load texture '{}': {}", path, e);
                Arc::new(Texture::default_white())
            }
        }
    }
    
    /// Get default white texture
    pub fn get_default_white(&self) -> Arc<Texture> {
        Arc::new(Texture::default_white())
    }
    
    /// Get default normal map texture
    pub fn get_default_normal(&self) -> Arc<Texture> {
        Arc::new(Texture::default_normal())
    }

    pub fn print_about(&self) {
        println!("TextureManager: Loaded {} textures.", self.textures.len());
        println!("Base path: {}", self.base_path);
        println!("Textures:");
        for (path, texture) in &self.textures {
            println!(" - {} ({}x{}), colorlen: {}", path, texture.width, texture.height, texture.data.len());
        }

    }


}

fn normalize_base_path(base_path: &str) -> String {
    let mut base = base_path.to_string();
    if !base.ends_with('/') && !base.ends_with('\\') {
        base.push(std::path::MAIN_SEPARATOR);
    }
    base
}

fn normalize_texture_path(path: &str) -> String {
    if std::path::MAIN_SEPARATOR == '/' {
        path.replace('\\', "/")
    } else {
        path.to_string()
    }
}

impl TextureManager {
    fn resolve_texture_path(&self, path: &str) -> Option<PathBuf> {
        let base = Path::new(&self.base_path);
        let path_obj = Path::new(path);

        if path_obj.is_absolute() {
            return path_obj.exists().then(|| path_obj.to_path_buf());
        }

        let candidate = base.join(path_obj);
        if candidate.exists() {
            return Some(candidate);
        }

        if let Some(file_name) = path_obj.file_name() {
            let candidate = base.join(file_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }

        None
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
}
