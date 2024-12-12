use minifb::{Window, WindowOptions};
use std::io;

pub struct WindowRenderer {
    window: Window,
    buffer: Vec<u32>,
}

impl WindowRenderer {
    pub fn new(width: usize, height: usize) -> io::Result<Self> {
        let window = Window::new("3D Renderer", width, height, WindowOptions::default())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(Self {
            window,
            buffer: vec![0; width * height],
        })
    }
}
