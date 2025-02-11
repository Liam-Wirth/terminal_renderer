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
    pub fn dir_above(color: Color, intensity: f32) -> Self {
        Light::Directional {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color,
            intensity,
        }
    }

    pub fn dir_below(color: Color, intensity: f32) -> Self {
        Light::Directional {
            direction: Vec3::new(0.0, 1.0, 0.0),
            color,
            intensity,
        }
    }
    pub fn dir_infront(color: Color, intensity: f32) -> Self {
        Light::Directional {
            direction: Vec3::new(0.0, 0.0, -1.0),
            color,
            intensity,
        }
    }
    pub fn dir_behind(color: Color, intensity: f32) -> Self {
        Light::Directional {
            direction: Vec3::new(0.0, 0.0, 1.0),
            color,
            intensity,
        }
    }
    pub fn dir_right(color: Color, intensity: f32) -> Self {
        Light::Directional {
            direction: Vec3::new(1.0, 0.0, 0.0),
            color,
            intensity,
        }
    }
    pub fn dir_left(color: Color, intensity: f32) -> Self {
        Light::Directional {
            direction: Vec3::new(-1.0, 0.0, 0.0),
            color,
            intensity,
        }
    }
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
        // Get material properties (with defaults)
        let ambient = material.ambient.unwrap_or(Color::new(0.2, 0.2, 0.2));
        let diffuse = material.diffuse.unwrap_or(Color::WHITE);
        let specular = material.specular.unwrap_or(Color::WHITE);
        let shininess = material.shininess.unwrap_or(32.0);

        // Initialize separate accumulators
        let mut ambient_acc = Color::new(0.0, 0.0, 0.0);
        let mut diffuse_acc = Color::new(0.0, 0.0, 0.0);
        let mut specular_acc = Color::new(0.0, 0.0, 0.0);

        for light in lights {
            match light {
                Light::Directional {
                    direction,
                    color,
                    intensity,
                } => {
                    // --- Ambient Contribution ---
                    let ambient_term = Color::new(
                        ambient.r * color.r,
                        ambient.g * color.g,
                        ambient.b * color.b,
                    );
                    ambient_acc.r += ambient_term.r;
                    ambient_acc.g += ambient_term.g;
                    ambient_acc.b += ambient_term.b;

                    // --- Diffuse Contribution ---
                    let light_dir = direction.normalize();
                    let diff_factor = normal.dot(light_dir).max(0.0);
                    let diffuse_term = Color::new(
                        diffuse.r * color.r * diff_factor * intensity,
                        diffuse.g * color.g * diff_factor * intensity,
                        diffuse.b * color.b * diff_factor * intensity,
                    );
                    diffuse_acc.r += diffuse_term.r;
                    diffuse_acc.g += diffuse_term.g;
                    diffuse_acc.b += diffuse_term.b;

                    // --- Specular Contribution ---
                    let halfway_dir = (light_dir + view_dir).normalize();
                    let spec_factor = normal.dot(halfway_dir).max(0.0).powf(shininess);
                    let specular_term = Color::new(
                        specular.r * color.r * spec_factor * intensity,
                        specular.g * color.g * spec_factor * intensity,
                        specular.b * color.b * spec_factor * intensity,
                    );
                    specular_acc.r += specular_term.r;
                    specular_acc.g += specular_term.g;
                    specular_acc.b += specular_term.b;
                }
                // (Handle Point and Spot lights similarly.)
                _ => {}
            }
        }

        // Option 1: Clamp each accumulator separately then sum
        ambient_acc.r = ambient_acc.r.clamp(0.0, 1.0);
        ambient_acc.g = ambient_acc.g.clamp(0.0, 1.0);
        ambient_acc.b = ambient_acc.b.clamp(0.0, 1.0);

        diffuse_acc.r = diffuse_acc.r.clamp(0.0, 1.0);
        diffuse_acc.g = diffuse_acc.g.clamp(0.0, 1.0);
        diffuse_acc.b = diffuse_acc.b.clamp(0.0, 1.0);

        specular_acc.r = specular_acc.r.clamp(0.0, 1.0);
        specular_acc.g = specular_acc.g.clamp(0.0, 1.0);
        specular_acc.b = specular_acc.b.clamp(0.0, 1.0);

        let final_r = (ambient_acc.r + diffuse_acc.r + specular_acc.r).clamp(0.0, 1.0);
        let final_g = (ambient_acc.g + diffuse_acc.g + specular_acc.g).clamp(0.0, 1.0);
        let final_b = (ambient_acc.b + diffuse_acc.b + specular_acc.b).clamp(0.0, 1.0);

        Color::new(final_r, final_g, final_b)
    }
}
