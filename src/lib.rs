pub mod pipeline;
pub mod renderers;

pub mod core;
#[derive(Debug, Clone, Copy)]
pub enum DisplayTarget {
    Terminal,
    Window,
}
