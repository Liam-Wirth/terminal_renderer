use crossterm::{
    cursor::{Hide, Show},
    event::DisableMouseCapture,
    execute,
    style::SetBackgroundColor,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use glam::{Quat, Vec3};
use log::{error, LevelFilter};
use simplelog::{Config, WriteLogger};
use std::env;
use std::panic;
use std::{
    f32::consts::PI,
    io::{self, stdout, Write},
};
use std::{fs::OpenOptions, path::Path};
use terminal_renderer::core::Color;
use terminal_renderer::core::Entity;

#[derive(Debug, Clone, Copy)]
pub enum RenderTarget {
    Term,
    Window,
    Both, // TODO: This could be cool
}

fn cleanup() -> io::Result<()> {
    let mut stdout = stdout();
    disable_raw_mode()?;
    execute!(stdout, Show, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("terminal_renderer.log")
        .unwrap();

    WriteLogger::init(LevelFilter::Info, Config::default(), log_file).unwrap();

    let target = env::args()
        .nth(1)
        .map(|arg| match arg.as_str() {
            "window" => RenderTarget::Window,
            _ => RenderTarget::Term, // Default to terminal if not specified or unknown
        })
        .unwrap_or(RenderTarget::Term);

    match target {
        RenderTarget::Term => run_terminal_version(),
        RenderTarget::Window => run_window_version(),
        RenderTarget::Both => run_both(),
    }
}

fn run_window_version() -> io::Result<()> {
    todo!();
    Ok(())
}

fn run_both() -> io::Result<()> {
    todo!();
    Ok(())
}
pub fn run_terminal_version() -> io::Result<()> {
    // Set up panic hook
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // First cleanup the terminal
        if let Err(e) = cleanup() {
            eprintln!("Failed to cleanup terminal: {}", e);
        }

        eprintln!("\n=== Panic Occurred ===");
        original_hook(panic_info);
        error!("Panic occurred: {:?}", panic_info);
    }));

    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        Hide,
        SetBackgroundColor(Color::BLACK.to_crossterm_color()),
        Hide,
    )?;

    let (width, height) = terminal::size()?;

    // Create and initialize the engine
    let mut engine =
        terminal_renderer::renderers::terminal::engine::Engine::new(width as u32, height as u32);

    // Add a triangle to the scene
    let tri = Entity::create_tri();
    let suzanne = Entity::from_obj(Path::new("../../assets/models/suzanne.obj"));
    let teapot = Entity::from_obj(Path::new("../../assets/models/newell_teaset/teapot.obj"));
    let cube = Entity::create_cube();
    cube.transform.translate(Vec3::new(1., 3., 0.));
    let octa = Entity::create_octahedron();
    let spoon = Entity::from_obj(Path::new("../../assets/models/newell_teaset/spoon.obj"));

    let ico = Entity::from_obj(Path::new("../../assets/models/icosphere.obj"));
    suzanne.transform.rotate_quat(Quat::from_rotation_y(PI));

    spoon.transform.translate(Vec3::new(-8., 5., 0.));
    spoon.transform.scale_uniform(6.);
    engine.scene.entities.push(teapot);
    engine.scene.entities.push(spoon);

    // Run the engine
    engine.run()?;

    // Normal cleanup
    cleanup()?;
    Ok(())
}
