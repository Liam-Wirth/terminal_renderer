use crate::core::color::Color;

#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,                     // Material name
    pub diffuse_color: Color,             // Base color if no texture is used
    pub diffuse_texture: Option<usize>,  // Texture ID for diffuse map
    pub normal_texture: Option<usize>,   // Texture ID for normal map
    pub specular_texture: Option<usize>, // Texture ID for specular map
    pub shininess: Option<f32>,           // Specular shininess factor
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            diffuse_color: Color::WHITE,
            diffuse_texture: None,
            normal_texture: None,
            specular_texture: None,
            shininess: None,
        }
    }
}
