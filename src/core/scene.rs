use crate::core::camera::Camera;

pub struct Scene {
    pub camera: Camera,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self { camera }
    }
}

impl Default for Scene {
    fn default() -> Self {
        let cam = Camera::new(
            glam::Vec3::new(0.0, 2.4, -6.0),
            glam::Vec3::new(0.0, 0.0, 1.0),
            800.0 / 600.0,
        );
        Self {
            camera: cam,
        }
    }
}
