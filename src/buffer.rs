use crossterm::{cursor::MoveTo, style::Print, QueueableCommand};
use std::io::{stdout, Write};

pub struct Buffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<char>, // TODO:  Make this be a bespoke struct of sort that holds color data and
                         // stuff like that instead of just being a char, like a vec of Point or something that holds
                         // the char it draws, and then the color of it
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Buffer {
            width,
            height,
            data: vec![' '; width * height], // Fill buffer with spaces initially
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(' ');
    }

    pub fn set_char(&mut self, x: usize, y: usize, ch: char) {
        if x < self.width && y < self.height {
            self.data[x + y * self.width] = ch;
        }
    }

    pub fn render_to_terminal(&self) -> std::io::Result<()> {
        let mut stdout = stdout();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = x + y * self.width;
                let ch = self.data[idx];

                // Move the cursor to the appropriate position and print the character
                stdout.queue(MoveTo(x as u16, y as u16))?;
                stdout.queue(Print(ch))?;
            }
        }

        // Flush the commands to the terminal
        stdout.flush()?;
        Ok(())
    }
}
