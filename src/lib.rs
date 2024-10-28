
use nalgebra::Vector3;

pub mod core {
    pub mod camera;
    pub mod entity;
    pub mod mesh;
    pub mod scene;
    pub mod transform;
    pub mod face;
    pub mod tri;
    pub mod color;
}

pub mod renderers {
    pub mod cpu_termrenderer;
    pub mod renderer;
    pub mod buffer;
}

const GLOBAL_UP: Vector3<f64> = Vector3::new(0.0, 1.0, 0.0);

// TODO: Decouple or smth from the global render mode
#[derive(Debug, Default, Clone)]
pub enum RENDERMODE {
    #[default]
    Wireframe,

    Solid,
    WireframeTris, // TODO: logic for this
}



