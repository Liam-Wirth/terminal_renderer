pub mod core;
pub mod renderers;

#[derive(Debug, Default, Clone)]
pub enum RENDERMODE {
    #[default]
    Wireframe,
    Solid,
    WireframeTris, // TODO: logic for this
}
