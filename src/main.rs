use crossterm::{
    cursor::{Hide, Show},
    event::DisableMouseCapture,
    execute,
    style::SetBackgroundColor,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use glam::Vec3;
use log::{error, LevelFilter};
use simplelog::{Config, WriteLogger};
use std::{fs::OpenOptions, path::Path};
use std::io::{self, stdout, Write};
use std::panic;
use terminal_renderer::core::Color;
use terminal_renderer::core::Entity;

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
    let mut suzanne = Entity::from_obj(Path::new("assets/models/suzanne.obj"));
    let mut teapot = Entity::from_obj(Path::new("assets/models/teapot.obj"));
    let cube = Entity::create_cube();
    cube.transform.translate(Vec3::new(1., 3., 0.));
    let octa = Entity::create_octahedron();
    let spoon = Entity::from_obj(Path::new("assets/models/newell_teaset/spoon.obj"));
    //let mut teacup = Entity::from_obj(Path::new("assets/models/newell_teaset/teacup.obj"));
    //teacup.transform.scale_uniform(1.);
    //for tri in &mut teacup.mesh.tris {
        //tri.update(&teacup.mesh.verts);
    //}
    engine.scene.entities.push(teapot);
    //engine.scene.entities.push(teacup);



    // Run the engine
    engine.run()?;

    // Normal cleanup
    cleanup()?;
    Ok(())
}
