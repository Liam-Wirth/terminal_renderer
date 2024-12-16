use crossterm::{
    cursor::{Hide, Show},
    event::DisableMouseCapture,
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use log::{error, LevelFilter};
use simplelog::{Config, WriteLogger};
use std::env;
use std::io::{self, stdout, Write};

use terminal_renderer::{
    core::Scene,
    renderers::{terminal::TerminalRenderer, window::WindowRenderer},
    DisplayTarget,
};

fn main() -> io::Result<()> {
    // Setup logging
    let log_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("terminal_renderer.log")
        .unwrap();

    WriteLogger::init(LevelFilter::Info, Config::default(), log_file).unwrap();

    // Parse command line args
    let target = env::args()
        .nth(1)
        .map(|arg| match arg.as_str() {
            "window" => DisplayTarget::Window,
            _ => DisplayTarget::Terminal,
        })
        .unwrap_or(DisplayTarget::Window);

    match target {
        DisplayTarget::Terminal => run_terminal(),
        DisplayTarget::Window => run_window(),
    }
}

fn cleanup_terminal() -> io::Result<()> {
    let mut stdout = stdout();
    disable_raw_mode()?;
    execute!(stdout, Show, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn run_terminal() -> io::Result<()> {
    let (width, height) = terminal::size()?;
    let mut renderer = TerminalRenderer::new()?;
    let mut scene = Scene::default();

    renderer.run(&mut scene)
}

fn run_window() -> io::Result<()> {
    let width = 1920;
    let height = 1080;
    let mut renderer = WindowRenderer::new(width, height)?;
    let mut scene = Scene::default();

    renderer.run(&mut scene)
}
