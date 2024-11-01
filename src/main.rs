use crossterm::{
    cursor::{Hide, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    style::SetBackgroundColor,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use std::{
    io::{self, stdout, Write},
    time::{Duration, Instant},
};
use terminal_renderer::{
    core::{camera::Camera, entity::Entity, scene::Scene, Color},
    renderers::terminal::{engine::Engine, term_pipeline::TermPipeline},
};

fn main() -> io::Result<()> {
    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        Hide,
        SetBackgroundColor(Color::BLACK.to_crossterm_color()),
    )?;

    enable_raw_mode()?;

    // Get terminal size
    let (width, height) = terminal::size()?;

    // Create and initialize the engine
    let mut engine = Engine::new(width as u32, height as u32);

    // Add a triangle to the scene
    let tri = Entity::create_tri();
    let cube = Entity::create_cube();
    engine.scene.entities.push(cube);

    // Run the engine
    engine.run()?;

    // Cleanup
    execute!(stdout, Show, LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;

    Ok(())
}
