pub mod core;
pub mod pipeline;
pub mod renderers;
#[derive(Debug, Clone, Copy)]
pub enum DisplayTarget {
    Terminal,
    Window,
}
