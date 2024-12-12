pub mod terminal;
pub mod window;

pub trait Renderer {
    fn init(&mut self, width: usize, height: usize);
    fn render_frame(&mut self) -> std::io::Result<()>;
    fn cleanup(&mut self) -> std::io::Result<()>;
}
