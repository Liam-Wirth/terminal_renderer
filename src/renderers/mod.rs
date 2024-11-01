pub mod terminal {
    pub mod engine;
    pub mod term_pipeline;
    pub mod termbuffer;
    pub use engine::Engine;
    pub use term_pipeline::TermPipeline;
    pub use termbuffer::TermBuffer;
}
pub mod renderer;

pub use renderer::Renderer;
