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
