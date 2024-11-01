use crossterm::{
    cursor::{Hide, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    style::SetBackgroundColor,
    terminal::{self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, stdout, Write},
    time::{Duration, Instant},
};
use terminal_renderer::core::{camera::Camera, entity::Entity, scene::Scene, Color};
use terminal_renderer::renderers::terminal::term_pipeline::TermPipeline;

fn main() -> io::Result<()> {
    todo!();
}
