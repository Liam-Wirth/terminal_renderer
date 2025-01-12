// TODO: Re-implement the floating point thing I did a while ago that like produced that really
// cool "wiggly" effect from old n64, PS2 and ds 3d games (banjo kazooie)
//
// TODO: re-implement Clap stuff for command line parsing/ mode selection
//  TODO: In name of that specifically I'd like
//  - Ability to choose model by passing a flag
// TODO: Add a background to scenes
// TODO: Re-Implement the debug menu thing
//
// TODO: Would be cool to try and see if I can get this rust engine to compile to WASM and interact
//with the javascript canvas to make draw calls 
//
// TODO: Materials
// TODO: Lighting
// TODO: Difuse
// TODO: pre-baking renders?
//
//
use crossterm::{
    cursor::Hide,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use glam::Vec3;
use minifb::{Key, Scale, Window, WindowOptions};
use std::{io::{self, stdout}, path::PathBuf};
use std::time::{Duration, Instant};
use terminal_renderer::{
    core::Entity,
    core::{Camera, Scene},
    pipeline::{pipeline::Pipeline, FrameBuffer, TermBuffer},
    DEBUG_PIPELINE,
};

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

fn main() -> io::Result<()> {
    let store = DEBUG_PIPELINE.store(false, std::sync::atomic::Ordering::Relaxed);
    let camera = Camera::new(
        Vec3::new(0.0, 0.0, -3.), // Position camera back a bit
        Vec3::ZERO,               // Look at origin
        WIDTH as f32 / HEIGHT as f32,
    );

    let mut scene = Scene::new(camera);

    let model_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("models")
        .join("newell_teaset/spoon.obj");

    scene.add_entity(Entity::from_obj(&model_path.to_str().unwrap()));
    scene.entities[0].mesh.bake_normals_to_colors();
    let mod2_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("models")
        .join("african_head.obj");

    let mod3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("models")
        .join("suzanne.mtl");

    //scene.add_entity(Entity::from_obj(&mod2_path.to_str().unwrap()));
    //scene.entities[1].mesh.bake_normals_to_colors();
    scene.add_entity(Entity::from_obj(&mod3_path.to_str().unwrap()));
    scene.entities[1].mesh.bake_normals_to_colors();
    scene.entities[1].mesh.calculate_normals();
    scene.add_entity(Entity::from_obj(&mod2_path.to_str().unwrap()));
    scene.entities[2].mesh.calculate_normals();
    scene.entities[2].mesh.bake_normals_to_colors();




    scene.entities[0].mesh.calculate_normals();
    // scene.entities[0].mesh.bake_normals_to_colors();

    // You can choose which one to run
    // let _ = run_win(scene.clone());
    // or
     run_term(scene)
    //run_win(scene)
}

pub fn run_term(mut scene: Scene) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        Hide,
        Clear(ClearType::All)
    )?;

    let (tw, th) = crossterm::terminal::size()?;

    let mut pipeline = Pipeline::<TermBuffer>::new(tw as usize, th as usize, scene);

    // 4) Main loop
    loop {
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => cleanup_terminal()?,
                    KeyCode::Char('w') => pipeline.scene.camera.move_forward(0.1),
                    KeyCode::Char('s') => pipeline.scene.camera.move_backward(0.1),
                    KeyCode::Char('a') => pipeline.scene.camera.move_left(0.1),
                    KeyCode::Char('d') => pipeline.scene.camera.move_right(0.1),
                    KeyCode::Up => pipeline.scene.camera.rotate(0.05, 0.0),
                    KeyCode::Down => pipeline.scene.camera.rotate(-0.05, 0.0),
                    KeyCode::Left => pipeline.scene.camera.rotate(0.0, 0.05),
                    KeyCode::Right => pipeline.scene.camera.rotate(0.0, -0.05),
                    KeyCode::Char('p') => pipeline.scene.spin(0),
                    // etc...
                    _ => {}
                }
            }
        }

        let (nw, nh) = crossterm::terminal::size()?;
        if nw as usize != pipeline.width || nh as usize != pipeline.height {
            pipeline =
                Pipeline::<TermBuffer>::new(nw as usize, nh as usize, pipeline.scene.clone());
        }

        pipeline.render_frame(None)?;

        // 4D) Sleep ~16 ms or so
        //thread::sleep(Duration::from_millis(16));
    }

    cleanup_terminal()?;
    Ok(())
}

pub fn run_win(scene: Scene) -> io::Result<()> {
    let mut window = Window::new(
        "Terminal Renderer - Window Mode",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            scale: Scale::X1,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to open window");

    let mut pipeline = Pipeline::<FrameBuffer>::new(WIDTH, HEIGHT, scene);
    while (window.is_open() && (!window.is_key_down(Key::Escape) || !window.is_key_down(Key::Q))) {
        pipeline.render_frame(Some(&mut window))?;
        pipeline.window_handle_input(&window);
    }

    Ok(())
}

fn cleanup_terminal() -> io::Result<()> {
    let mut stdout = stdout();
    disable_raw_mode()?;
    execute!(stdout, terminal::LeaveAlternateScreen,)?;
    Ok(())
}
