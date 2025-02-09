use crate::core::geometry::Material;
use crate::core::Color;
use glam::Vec3;

/// The three kinds of lights we support.
#[derive(Clone, Debug)]
pub enum Light {
    /// A directional light has a constant direction and does not attenuate with distance.
    Directional {
        /// The direction of the light (the vector the light travels along).
        /// (For example, (0, -1, 0) for light coming from above.)
        direction: Vec3,
        /// The light’s color.
        color: Color,
        /// A scalar multiplier for the light’s strength.
        intensity: f32,
    },
    /// A point light emits light in all directions from a given position.
    Point {
        /// The position of the light in world space.
        position: Vec3,
        /// The light’s color.
        color: Color,
        /// A scalar multiplier for the light’s strength.
        intensity: f32,
        /// Constant attenuation factor.
        constant: f32,
        /// Linear attenuation factor.
        linear: f32,
        /// Quadratic attenuation factor.
        quadratic: f32,
    },
    /// A spot light emits light in a cone.
    Spot {
        /// The position of the light.
        position: Vec3,
        /// The direction the spot light is pointing.
        direction: Vec3,
        /// The light’s color.
        color: Color,
        /// A scalar multiplier for the light’s strength.
        intensity: f32,
        /// The cosine of the inner cutoff angle (full intensity within this cone).
        inner_cutoff: f32,
        /// The cosine of the outer cutoff angle (beyond this cone, the light is off).
        outer_cutoff: f32,
        /// Attenuation factors (see point light).
        constant: f32,
        linear: f32,
        quadratic: f32,
    },
}

impl Light {
    pub fn default_directional() -> Self {
        Light::Directional {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Color::WHITE,
            intensity: 1.0,
        }
    }

    pub fn default_point() -> Self {
        Light::Point {
            position: Vec3::new(0.0, 0.0, 0.0),
            color: Color::WHITE,
            intensity: 1.0,
            constant: 1.0,
            linear: 0.09,
            quadratic: 0.032,
        }
    }

    pub fn default_spot() -> Self {
        Light::Spot {
            position: Vec3::new(0.0, 0.0, 0.0),
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Color::WHITE,
            intensity: 1.0,
            inner_cutoff: 0.9,
            outer_cutoff: 0.85,
            constant: 1.0,
            linear: 0.09,
            quadratic: 0.032,
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Light::default_directional()
    }
}

pub trait LightingModel {
    /// Computes the final color for a fragment given the scene’s lighting and material properties.
    ///
    /// - `frag_pos`: the world-space position of the fragment.
    /// - `normal`: the surface normal (should be normalized).
    /// - `view_dir`: the normalized direction from the fragment to the camera.
    /// - `lights`: a slice of lights in the scene.
    /// - `material`: the material properties of the fragment.
    fn shade(
        &self,
        frag_pos: Vec3,
        normal: Vec3,
        view_dir: Vec3,
        lights: &[Light],
        material: &Material,
    ) -> Color;
}
pub enum LightMode {
    Flat,
    BlinnPhong,
    None,
}
pub struct FlatShading;
impl LightingModel for FlatShading {
    fn shade(
        &self,
        frag_pos: Vec3,
        normal: Vec3,
        _view_dir: Vec3, // Unused in a flat shading model
        lights: &[Light],
        material: &Material,
    ) -> Color {
        let mut final_color = Color::new(0.0, 0.0, 0.0);

        // Get material properties
        let ambient = material.ambient.unwrap_or(Color::new(0.2, 0.2, 0.2));
        let diffuse = material.diffuse.unwrap_or(Color::WHITE);

        for light in lights {
            match light {
                Light::Directional {
                    direction,
                    color,
                    intensity,
                } => {
                    // Ambient
                    final_color.r += ambient.r * color.r;
                    final_color.g += ambient.g * color.g;
                    final_color.b += ambient.b * color.b;

                    // Diffuse
                    let light_dir = -direction.normalize();
                    let diff = normal.dot(light_dir).max(0.0);

                    final_color.r += diffuse.r * color.r * diff * intensity;
                    final_color.g += diffuse.g * color.g * diff * intensity;
                    final_color.b += diffuse.b * color.b * diff * intensity;
                }
                Light::Point {
                    position,
                    color,
                    intensity,
                    constant,
                    linear,
                    quadratic,
                } => {
                    // Ambient
                    final_color.r += ambient.r * color.r;
                    final_color.g += ambient.g * color.g;
                    final_color.b += ambient.b * color.b;

                    // Calculate attenuation
                    let light_dir = (*position - frag_pos).normalize();
                    let distance = (*position - frag_pos).length();
                    let attenuation =
                        1.0 / (constant + linear * distance + quadratic * distance * distance);

                    // Diffuse
                    let diff = normal.dot(light_dir).max(0.0);

                    final_color.r += diffuse.r * color.r * diff * intensity * attenuation;
                    final_color.g += diffuse.g * color.g * diff * intensity * attenuation;
                    final_color.b += diffuse.b * color.b * diff * intensity * attenuation;
                }

                Light::Spot {
                    position,
                    direction,
                    color,
                    intensity,
                    inner_cutoff,
                    outer_cutoff,
                    constant,
                    linear,
                    quadratic,
                } => {
                    // determine the ambient lighting
                    final_color.r += ambient.r * color.r;
                    final_color.g += ambient.g * color.g;
                    final_color.b += ambient.b * color.b;

                    todo!();
                    // NOTE: https://developer.download.nvidia.com/CgTutorial/cg_tutorial_chapter05.html
                }
                _ => {}
            }
        }

        // Clamp colors to [0,1]
        final_color.r = final_color.r.clamp(0.0, 1.0);
        final_color.g = final_color.g.clamp(0.0, 1.0);
        final_color.b = final_color.b.clamp(0.0, 1.0);

        final_color
    }
}

pub struct BlinnPhongShading;
impl LightingModel for BlinnPhongShading {
    fn shade(
        &self,
        frag_pos: Vec3,
        normal: Vec3,
        view_dir: Vec3,
        lights: &[Light],
        material: &Material,
    ) -> Color {
        let mut final_color = Color::new(0.0, 0.0, 0.0);

        // Get material properties
        let ambient = material.ambient.unwrap_or(Color::new(0.2, 0.2, 0.2));
        let diffuse = material.diffuse.unwrap_or(Color::WHITE);
        let specular = material.specular.unwrap_or(Color::WHITE);
        let shininess = material.shininess.unwrap_or(32.0);

        for light in lights {
            match light {
                Light::Directional {
                    direction,
                    color,
                    intensity,
                } => {
                    // Ambient
                    final_color.r += ambient.r * color.r;
                    final_color.g += ambient.g * color.g;
                    final_color.b += ambient.b * color.b;

                    // Diffuse
                    let light_dir = -direction.normalize();
                    let diff = normal.dot(light_dir).max(0.0);

                    final_color.r += diffuse.r * color.r * diff * intensity;
                    final_color.g += diffuse.g * color.g * diff * intensity;
                    final_color.b += diffuse.b * color.b * diff * intensity;

                    // Specular
                    let halfway_dir = (light_dir + view_dir).normalize();
                    let spec = normal.dot(halfway_dir).max(0.0).powf(shininess);

                    final_color.r += specular.r * color.r * spec * intensity;
                    final_color.g += specular.g * color.g * spec * intensity;
                    final_color.b += specular.b * color.b * spec * intensity;
                }
                _ => {}
            }
            final_color.r = final_color.r.clamp(0.0, 1.0);
            final_color.g = final_color.g.clamp(0.0, 1.0);
            final_color.b = final_color.b.clamp(0.0, 1.0);
        }
        final_color
    }
}
