use crate::core::geometry::Material;
use crate::core::Color;
use glam::Vec3;

/// The lights I'll support (can be extended in the future)
#[derive(Clone, Debug)]
pub enum LightType {
    /// A directional light has a constant direction and does not attenuate with distance.
    /// The direction of the light (the vector the light travels along).
    /// (For example, (0, -1, 0) for light coming from above.)
    Directional(Vec3),
    Point {
        /// The position of the light in world space.
        position: Vec3,
        /// Constant attenuation factor.
        constant: f32,
        /// Linear attenuation factor.
        linear: f32,
        /// Quadratic attenuation factor.
        quadratic: f32,
    },
    Spot {
        /// The position of the light.
        position: Vec3,
        /// The direction the spot light is pointing.
        direction: Vec3,
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

#[derive(Clone, Debug)]
pub struct Light {
    pub light_type: LightType,
    pub color: Color,
    /// A scalar multiplier for the light’s strength. (clamped 0. -> 1.)
    pub intensity: f32,
}

impl Light {
    pub fn dir_above(color: Color, intensity: f32) -> Self {
        Light {
            light_type: LightType::Directional(Vec3::new(0.0, -1.0, 0.0)),
            color,
            intensity,
        }
    }

    pub fn dir_below(color: Color, intensity: f32) -> Self {
        Light {
            light_type: LightType::Directional(Vec3::new(0.0, 1.0, 0.0)),
            color,
            intensity,
        }
    }
    pub fn dir_infront(color: Color, intensity: f32) -> Self {
        Light {
            light_type: LightType::Directional(Vec3::new(0.0, 0.0, -1.0)),
            color,
            intensity,
        }
    }
    pub fn dir_behind(color: Color, intensity: f32) -> Self {
        Light {
            light_type: LightType::Directional(Vec3::new(0.0, 0.0, 1.0)),
            color,
            intensity,
        }
    }
    pub fn dir_right(color: Color, intensity: f32) -> Self {
        Light {
            light_type: LightType::Directional(Vec3::new(1.0, 0.0, 0.0)),
            color,
            intensity,
        }
    }
    pub fn dir_left(color: Color, intensity: f32) -> Self {
        Light {
            light_type: LightType::Directional(Vec3::new(-1.0, 0.0, 0.0)),
            color,
            intensity,
        }
    }
    pub fn default_directional() -> Self {
        Light {
            light_type: LightType::Directional(Vec3::new(0.0, -1.0, 0.0)),
            color: Color::WHITE,
            intensity: 1.0,
        }
    }

    pub fn default_point() -> Self {
        Light {
            light_type: LightType::Point {
                position: Vec3::new(0.0, 0.0, 0.0),
                constant: 1.0,
                linear: 0.09,
                quadratic: 0.032,
            },
            color: Color::WHITE,
            intensity: 1.0,
        }
    }
    pub fn point(position: Vec3, constant: f32, linear: f32, quadratic: f32) -> Self {
        Light {
            light_type: LightType::Point {
                position,
                constant,
                linear,
                quadratic,
            },
            color: Color::WHITE,
            intensity: 1.0,
        }
    }

    pub fn easy_point(position: Vec3) -> Self {
        Light {
            light_type: LightType::Point {
                position,
                constant: 1.0,
                linear: 0.009,
                quadratic: 0.032,
            },
            color: Color::WHITE,
            intensity: 1.0,
        }
    }

    pub fn change_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn default_spot() -> Self {
        Light {
            light_type: LightType::Spot {
                position: Vec3::new(0.0, 0.0, 0.0),
                direction: Vec3::new(0.0, -1.0, 0.0),
                inner_cutoff: 0.9,
                outer_cutoff: 0.85,
                constant: 1.0,
                linear: 0.09,
                quadratic: 0.032,
            },
            color: Color::WHITE,
            intensity: 1.0,
        }
    }
    pub fn spot(position: Vec3, direction: Vec3, inner_cutoff: f32, outer_cutoff: f32) -> Self {
        Light {
            light_type: LightType::Spot {
                position,
                direction,
                inner_cutoff,
                outer_cutoff,
                constant: 1.0,
                linear: 0.08,
                quadratic: 0.032,
            },
            color: Color::WHITE,
            intensity: 1.0,
        }
    }
    pub fn spot_with_pos(position: Vec3, direction: Vec3) -> Self {
        Light {
            light_type: LightType::Spot {
                position,
                direction,
                inner_cutoff: 0.9,
                outer_cutoff: 0.85,
                constant: 1.0,
                linear: 0.08,
                quadratic: 0.032,
            },
            color: Color::WHITE,
            intensity: 1.0,
        }
    }
    pub fn is_directional(&self) -> bool {
        matches!(self.light_type, LightType::Directional(_))
    }
    pub fn is_point(&self) -> bool {
        matches!(self.light_type, LightType::Point { .. })
    }
    pub fn is_spot(&self) -> bool {
        matches!(self.light_type, LightType::Spot { .. })
    }
    pub fn get_position(&self) -> Vec3 {
        match self.light_type {
            LightType::Directional(_) => Vec3::ZERO,
            LightType::Point { position, .. } => position,
            LightType::Spot { position, .. } => position,
        }
    }
    pub fn set_position(&mut self, pos: Vec3) {
        match self.light_type {
            LightType::Directional(_) => {}
            LightType::Point {
                ref mut position, ..
            } => *position = pos,
            LightType::Spot {
                ref mut position, ..
            } => *position = pos,
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Light::default_directional()
    }
}

impl Light {
    pub fn orbit(&mut self, center: Vec3, radius: f32, speed: f32, delta: f32) {
        if let LightType::Point {
            ref mut position, ..
        } = self.light_type
        {
            let current_angle = (position.z - center.z).atan2(position.x - center.x);
            let new_angle = current_angle + speed * delta;
            position.x = center.x + radius * new_angle.cos();
            position.z = center.z + radius * new_angle.sin();
        }
    }
}

pub trait LightingModel {
    /// Computes the final color for a pixel given the GBuffer data and scene lighting.
    ///
    /// - `albedo`: The albedo color from the GBuffer.
    /// - `normal`: The world-space normal from the GBuffer.
    /// - `specular_color`: The specular color from the GBuffer.
    /// - `shininess`: The shininess value from the GBuffer.
    /// - `frag_pos`: the world-space position of the fragment.
    /// - `view_dir`: the normalized direction from the fragment to the camera.
    /// - `lights`: a slice of lights in the scene.
    /// - `material`: material properties (can be defaulted or fetched from GBuffer indices if material IDs are stored).
    fn shade(
        &self,
        albedo: Color,
        normal: Vec3,
        specular_color: Color,
        shininess: f32,
        frag_pos: Vec3,
        view_dir: Vec3,
        lights: &[Light],
        material: Option<&Material>, // Still keeping material for potential complex materials
    ) -> Color;
}

#[derive(Clone, Debug, PartialEq)]
pub enum LightMode {
    Flat,
    BlinnPhong,
    None,
}
pub struct FlatShading;

impl LightingModel for FlatShading {
    fn shade(
        &self,
        albedo: Color,
        normal: Vec3,
        _specular_color: Color, // Not used in flat shading
        _shininess: f32,        // Not used in flat shading
        frag_pos: Vec3,
        _view_dir: Vec3, // Unused in flat shading
        lights: &[Light],
        material: Option<&Material>,
    ) -> Color {
        let mut final_color = Color::BLACK;

        // Prefer the albedo coming from the GBuffer (which already contains
        // sampled textures). Only fall back to material colors when a texture
        // is missing.
        let ambient = material
            .and_then(|m| m.ambient)
            .unwrap_or(Color::DARK_GRAY);
        let diffuse = material
            .map(|m| {
                if m.diffuse_texture_data.is_some() {
                    // Texture already baked into albedo; keep it as-is
                    albedo
                } else {
                    m.diffuse.unwrap_or(albedo)
                }
            })
            .unwrap_or(albedo);

        for light in lights {
            // Accumulate ambient component - it's constant for flat shading typically
            final_color += ambient * light.color * light.intensity; // Ambient is affected by light color & intensity

            match light.light_type {
                LightType::Directional(direction) => {
                    // Diffuse only for directional light
                    let light_dir = -direction.normalize();
                    let diff_factor = normal.dot(light_dir).max(0.0);
                    final_color += diffuse * light.color * diff_factor * light.intensity;
                }
                LightType::Point {
                    position,
                    constant,
                    linear,
                    quadratic,
                } => {
                    // Diffuse for point light
                    let light_vec = position - frag_pos;
                    let light_dir = light_vec.normalize();
                    let distance = light_vec.length();
                    let attenuation = 1.0
                        / (constant + linear * distance + quadratic * distance * distance).max(1.0); // Ensure no divide by zero
                    let diff_factor = normal.dot(light_dir).max(0.0);
                    final_color +=
                        diffuse * light.color * diff_factor * attenuation * light.intensity;
                }
                LightType::Spot {
                    position,
                    direction,
                    inner_cutoff,
                    outer_cutoff,
                    constant,
                    linear,
                    quadratic,
                } => {
                    let light_vec = position - frag_pos;
                    let light_dir = light_vec.normalize();
                    let distance = light_vec.length();
                    let attenuation = 1.0
                        / (constant + linear * distance + quadratic * distance * distance).max(1.0);

                    let spot_dir = (-direction).normalize(); // Spotlight direction
                    let spot_factor = light_dir.dot(spot_dir);

                    if spot_factor > outer_cutoff {
                        // In spotlight cone
                        let intensity_factor = if spot_factor >= inner_cutoff {
                            1.0 // Full intensity
                        } else {
                            // Smooth falloff
                            let smooth_factor =
                                (spot_factor - outer_cutoff) / (inner_cutoff - outer_cutoff);
                            smooth_factor.clamp(0.0, 1.0) // Ensure it's within [0, 1]
                        };
                        let diff_factor = normal.dot(light_dir).max(0.0);
                        final_color += diffuse
                            * light.color
                            * intensity_factor
                            * diff_factor
                            * attenuation
                            * light.intensity;
                    }
                }
            }
        }
        final_color.clamped()
    }
}

pub struct BlinnPhongShading;
impl LightingModel for BlinnPhongShading {
    fn shade(
        &self,
        albedo: Color,
        normal: Vec3,
        specular_color: Color,
        shininess: f32,
        frag_pos: Vec3,
        view_dir: Vec3,
        lights: &[Light],
        material: Option<&Material>,
    ) -> Color {
        let mut final_color: Color = (0.0, 0.0, 0.0).into();
        // Keep the albedo/specular coming from the rasterizer (includes textures).
        let ambient = material
            .and_then(|m| m.ambient)
            .unwrap_or(Color::DARK_GRAY);
        let diffuse = material
            .map(|m| {
                if m.diffuse_texture_data.is_some() {
                    // Texture already baked into albedo; keep it as-is
                    albedo
                } else {
                    m.diffuse.unwrap_or(albedo)
                }
            })
            .unwrap_or(albedo);
        let specular = material
            .map(|m| {
                if m.specular_texture_data.is_some() {
                    // Already sampled from texture; leave as-is
                    specular_color
                } else {
                    m.specular.unwrap_or(specular_color)
                }
            })
            .unwrap_or(specular_color);
        // Shininess is already from GBuffer input

        for light in lights {
            // Accumulate ambient component
            final_color += ambient * light.color * light.intensity; // Ambient unaffected by light type usually

            match light.light_type {
                LightType::Directional(direction) => {
                    let light_dir = -direction.normalize(); // Direction to fragment from light source (opposite of light direction)

                    // Diffuse
                    let diff_factor = normal.dot(light_dir).max(0.0);
                    let diffuse_contrib = diffuse * light.color * diff_factor;

                    // Specular - Blinn-Phong
                    let halfway_dir = (light_dir + view_dir).normalize(); // Halfway vector between light and view
                                                                          //let spec_factor = normal.dot(halfway_dir).max(0.0).powf(shininess); <-- this might have caused problems?
                    let spec_factor =
                        (normal.dot(halfway_dir).max(0.0) * diff_factor).powf(shininess);
                    let specular_contrib = specular * light.color * spec_factor;

                    final_color += (diffuse_contrib + specular_contrib) * light.intensity;
                }
                LightType::Point {
                    position,
                    constant,
                    linear,
                    quadratic,
                } => {
                    let light_vec = position - frag_pos;
                    let light_dir = light_vec.normalize();
                    let distance = light_vec.length();
                    let attenuation = 1.0
                        / (constant + linear * distance + quadratic * distance * distance).max(1.0);

                    // Diffuse
                    let diff_factor = normal.dot(light_dir).max(0.0);
                    let diffuse_contrib = diffuse * light.color * diff_factor;

                    // Specular - Blinn-Phong
                    let halfway_dir = (light_dir + view_dir).normalize();
                    //let spec_factor = normal.dot(halfway_dir).max(0.0).powf(shininess);
                    let spec_factor: f32 =
                        (normal.dot(halfway_dir).max(0.0) * diff_factor).powf(shininess);
                    let specular_contrib = specular * light.color * spec_factor;

                    final_color +=
                        (diffuse_contrib + specular_contrib) * attenuation * light.intensity;
                }
                LightType::Spot {
                    position,
                    direction,
                    inner_cutoff,
                    outer_cutoff,
                    constant,
                    linear,
                    quadratic,
                } => {
                    let light_vec = position - frag_pos;
                    let light_dir = light_vec.normalize();
                    let distance = light_vec.length();
                    let attenuation = 1.0
                        / (constant + linear * distance + quadratic * distance * distance).max(1.0);

                    let spot_dir = (-direction).normalize();
                    let spot_factor = light_dir.dot(spot_dir);

                    if spot_factor > outer_cutoff {
                        let intensity_factor = if spot_factor >= inner_cutoff {
                            1.0
                        } else {
                            let smooth_factor =
                                (spot_factor - outer_cutoff) / (inner_cutoff - outer_cutoff);
                            smooth_factor.clamp(0.0, 1.0)
                        };

                        // Diffuse
                        let diff_factor = normal.dot(light_dir).max(0.0);
                        let diffuse_contrib = diffuse * light.color * diff_factor;

                        // Specular - Blinn-Phong
                        let halfway_dir = (light_dir + view_dir).normalize();
                        //   let spec_factor = normal.dot(halfway_dir).max(0.0).powf(shininess);
                        let spec_factor =
                            (normal.dot(halfway_dir).max(0.0) * diff_factor).powf(shininess);
                        let specular_contrib = specular * light.color * spec_factor;

                        final_color += (diffuse_contrib + specular_contrib)
                            * intensity_factor
                            * attenuation
                            * light.intensity;
                    }
                }
            }
        }

        final_color.clamped()
    }
}
