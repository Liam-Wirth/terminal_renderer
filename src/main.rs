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

use terminal_renderer::DisplayTarget;

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
        .unwrap_or(DisplayTarget::Terminal);

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
    // Set up panic hook for terminal cleanup
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = cleanup_terminal();
        eprintln!("\n=== Panic Occurred ===");
        original_hook(panic_info);
        error!("Panic occurred: {:?}", panic_info);
    }));

    // Initialize terminal
    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide,)?;

    let (width, height) = terminal::size()?;
    println!("Terminal mode initialized: {}x{}", width, height);

    // TODO: Initialize and run terminal renderer

    cleanup_terminal()
}

fn run_window() -> io::Result<()> {
    use minifb::{Window, WindowOptions};

    let mut window = Window::new(
        "3D Terminal Renderer - Window Mode",
        800,
        600,
        WindowOptions {
            resize: true,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Set a minimum refresh rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() {
        // TODO: Initialize and run window renderer

        // Temporary: Just show a blank window
        window.update();
    }

    Ok(())
}
