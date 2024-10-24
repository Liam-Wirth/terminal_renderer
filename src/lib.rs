use nalgebra::Vector3;

pub mod core {
    pub mod camera;
    pub mod entity;
    pub mod mesh;
    pub mod scene;
    pub mod transform;
}

pub mod renderers {
    pub mod cpu_termrenderer;
    pub mod renderer;
}

const GLOBAL_UP: Vector3<f64> = Vector3::new(0.0, 1.0, 0.0);
