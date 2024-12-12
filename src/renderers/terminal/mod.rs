use crossterm::terminal;
use std::io;

pub struct TerminalRenderer {
    width: usize,
    height: usize,
}

impl TerminalRenderer {
    pub fn new() -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self {
            width: width as usize,
            height: height as usize,
        })
    }
}
