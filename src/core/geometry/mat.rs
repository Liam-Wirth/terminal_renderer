use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::core::color::Color;
use crate::core::texture::Texture;
// TODO: setup method to be able to have an alternate material holding the baked color normals
#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    // Base colors converted to our Color type
    pub ambient: Option<Color>,
    pub diffuse: Option<Color>,
    pub specular: Option<Color>,

    // Material properties
    pub shininess: Option<f32>,
    pub dissolve: Option<f32>,
    pub optical_density: Option<f32>,

    // Texture paths
    pub ambient_texture: Option<String>,
    pub diffuse_texture: Option<String>,
    pub specular_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub shininess_texture: Option<String>,
    pub dissolve_texture: Option<String>,

    // Loaded textures
    pub ambient_texture_data: Option<Arc<Texture>>,
    pub diffuse_texture_data: Option<Arc<Texture>>,
    pub specular_texture_data: Option<Arc<Texture>>,
    pub normal_texture_data: Option<Arc<Texture>>,
    pub shininess_texture_data: Option<Arc<Texture>>,
    pub dissolve_texture_data: Option<Arc<Texture>>,

    pub illumination_model: Option<u8>,
    //pub unknown_params: std::collections::HashMap<std::string::String, std::string::String>,
}

impl Material {
    pub fn from_tobj(mat: tobj::Material) -> Self {
        let out = Self {
            name: mat.name,

            // Convert [f32; 3] arrays to our Color type
            ambient: mat.ambient.map(|a| Color::new(a[0], a[1], a[2])),
            diffuse: mat.diffuse.map(|d| Color::new(d[0], d[1], d[2])),
            specular: mat.specular.map(|s| Color::new(s[0], s[1], s[2])),

            shininess: mat.shininess,
            dissolve: mat.dissolve,
            optical_density: mat.optical_density,

            // Clone the texture paths
            ambient_texture: mat.ambient_texture,
            diffuse_texture: mat.diffuse_texture,
            specular_texture: mat.specular_texture,
            normal_texture: mat.normal_texture,
            shininess_texture: mat.shininess_texture,
            dissolve_texture: mat.dissolve_texture,

            illumination_model: mat.illumination_model,
            
            // Initialize texture data as None - will be loaded later
            ambient_texture_data: None,
            diffuse_texture_data: None,
            specular_texture_data: None,
            normal_texture_data: None,
            shininess_texture_data: None,
            dissolve_texture_data: None,
            //unknown_params: mat.unknown_param.to_ha
        };
        out
    }

    pub fn get_base_color(&self) -> Color {
        // Use diffuse color if available, otherwise ambient, otherwise default white
        self.diffuse
            .map(|c| c)
            .or(self.ambient)
            .unwrap_or(Color::WHITE)
    }

    /// Load textures for this material using a texture manager
    pub fn load_textures(&mut self, texture_manager: &mut crate::core::TextureManager) {
        // Load diffuse texture if path is available
        if let Some(ref path) = self.diffuse_texture {
            self.diffuse_texture_data = Some(texture_manager.get_texture(path));
        }
        
        // Load ambient texture
        if let Some(ref path) = self.ambient_texture {
            self.ambient_texture_data = Some(texture_manager.get_texture(path));
        }
        
        // Load specular texture
        if let Some(ref path) = self.specular_texture {
            self.specular_texture_data = Some(texture_manager.get_texture(path));
        }
        
        // Load normal texture
        if let Some(ref path) = self.normal_texture {
            self.normal_texture_data = Some(texture_manager.get_texture(path));
        }
        
        // Load shininess texture
        if let Some(ref path) = self.shininess_texture {
            self.shininess_texture_data = Some(texture_manager.get_texture(path));
        }
        
        // Load dissolve texture
        if let Some(ref path) = self.dissolve_texture {
            self.dissolve_texture_data = Some(texture_manager.get_texture(path));
        }
    }

    /// Sample the diffuse color at UV coordinates
    pub fn sample_diffuse(&self, uv: glam::Vec2) -> Color {
        if let Some(ref texture) = self.diffuse_texture_data {
            // Blend texture with material color if available
            let tex_color = texture.sample(uv);
            if let Some(mat_color) = self.diffuse {
                // Multiply texture color with material color
                Color::new(
                    tex_color.r * mat_color.r,
                    tex_color.g * mat_color.g,
                    tex_color.b * mat_color.b,
                )
            } else {
                tex_color
            }
        } else {
            // No texture, return material color or white
            self.diffuse.unwrap_or(Color::WHITE)
        }
    }

    /// Sample the specular color at UV coordinates
    pub fn sample_specular(&self, uv: glam::Vec2) -> Color {
        if let Some(ref texture) = self.specular_texture_data {
            let tex_color = texture.sample(uv);
            if let Some(mat_color) = self.specular {
                Color::new(
                    tex_color.r * mat_color.r,
                    tex_color.g * mat_color.g,
                    tex_color.b * mat_color.b,
                )
            } else {
                tex_color
            }
        } else {
            self.specular.unwrap_or(Color::BLACK)
        }
    }

    /// Sample normal map at UV coordinates and return world space normal
    pub fn sample_normal(&self, uv: glam::Vec2, world_normal: glam::Vec3, tangent: glam::Vec3, bitangent: glam::Vec3) -> glam::Vec3 {
        if let Some(ref texture) = self.normal_texture_data {
            let normal_color = texture.sample(uv);
            
            // Convert from [0,1] to [-1,1] range
            let tangent_normal = glam::Vec3::new(
                normal_color.r * 2.0 - 1.0,
                normal_color.g * 2.0 - 1.0,
                normal_color.b * 2.0 - 1.0,
            );
            
            // Transform from tangent space to world space
            let world_normal_from_map = tangent * tangent_normal.x
                + bitangent * tangent_normal.y
                + world_normal * tangent_normal.z;
                
            world_normal_from_map.normalize()
        } else {
            // No normal map, return original normal
            world_normal
        }
    }
}

impl Display for Material {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: Better pretty printing here
        write!(
            f,
            "Material, Name: {}, \n Diffuse: {:?}",
            self.name, self.diffuse
        )
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            ambient: None,
            diffuse: None,
            specular: None,
            shininess: None,
            dissolve: None,
            optical_density: None,
            ambient_texture: None,
            diffuse_texture: None,
            specular_texture: None,
            normal_texture: None,
            shininess_texture: None,
            dissolve_texture: None,
            illumination_model: Some(2), // Blinn-Phong by default
            
            // Initialize texture data as None
            ambient_texture_data: None,
            diffuse_texture_data: None,
            specular_texture_data: None,
            normal_texture_data: None,
            shininess_texture_data: None,
            dissolve_texture_data: None,
            //unknown_params: HashMap::new(),
        }
    }
}
