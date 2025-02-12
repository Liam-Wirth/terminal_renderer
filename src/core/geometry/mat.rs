use std::fmt::{Display, Formatter};

use crate::core::color::Color;
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

    // Texture paths (we'll need a texture system later)
    pub ambient_texture: Option<String>,
    pub diffuse_texture: Option<String>,
    pub specular_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub shininess_texture: Option<String>,
    pub dissolve_texture: Option<String>,

    pub illumination_model: Option<u8>,
    //pub unknown_params: std::collections::HashMap<std::string::String, std::string::String>,
}

impl Material {
    pub fn from_tobj(mat: tobj::Material) -> Self {
        Self {
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
            //unknown_params: mat.unknown_param.to_ha
        }
    }

    pub fn get_base_color(&self) -> Color {
        // Use diffuse color if available, otherwise ambient, otherwise default white
        self.diffuse
            .map(|c| c)
            .or(self.ambient)
            .unwrap_or(Color::WHITE)
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
                                         //unknown_params: HashMap::new(),
        }
    }
}
