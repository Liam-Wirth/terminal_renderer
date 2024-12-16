use crossterm::{
    cursor::{Hide, Show},
    event::DisableMouseCapture,
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use glam::{Affine3A, Vec3};
use log::{error, LevelFilter};
use simplelog::{Config, WriteLogger};
use std::env;
use std::io::{self, stdout, Write};

use terminal_renderer::core::Entity;
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
    let mut me: Entity = Entity::from_obj("assets/models/teapot.obj");
    let mut e: Entity = Entity::from_obj("assets/models/teapot.obj");
    me.transform = Affine3A::from_rotation_y(std::f32::consts::PI);

    let flip_y = Affine3A::from_scale(Vec3::new(1.0, -1.0, 1.0));
    let flip_x = Affine3A::from_scale(Vec3::new(-1.0, 1.0, 1.0));

    e.transform = flip_x * flip_y;

    e.transform = e.transform * Affine3A::from_translation(Vec3::new(0., 4., 0.));

    let mut scene = Scene::new_with_entities(Scene::default().camera, vec![me, e]);

    match target {
        DisplayTarget::Terminal => run_term(scene),
        DisplayTarget::Window => run_win(scene),
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
    // Instead of default scene, use a test scene
    let mut scene = Scene::new_test_scene(Scene::default().camera);

    renderer.run(&mut scene)
}

fn run_term(mut scene: Scene) -> io::Result<()> {
    let (width, height) = terminal::size()?;
    let mut renderer = TerminalRenderer::new()?;
    renderer.run(&mut scene)
}

fn run_win(mut scene: Scene) -> io::Result<()> {
    let width = 1920;
    let height = 1080;
    let mut renderer = WindowRenderer::new(width, height)?;
    renderer.run(&mut scene)
}

fn run_window() -> io::Result<()> {
    let width = 1920;
    let height = 1080;
    let mut renderer = WindowRenderer::new(width, height)?;
    // Instead of default scene, use a test scene
    let mut scene = Scene::new_test_scene(Scene::default().camera);

    renderer.run(&mut scene)
}
