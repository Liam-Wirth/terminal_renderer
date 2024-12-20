use crossterm::{
    cursor::Hide,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use glam::Vec3;
use minifb::{Key, Scale, Window, WindowOptions};
use std::io::{self, stdout};
use std::thread;
use std::time::{Duration, Instant};
use terminal_renderer::{
    core::Entity,
    core::{Camera, Scene},
    pipeline::{pipeline::Pipeline, FrameBuffer, TermBuffer},
    DEBUG_PIPELINE,
};

const WIDTH: usize = 2000;
const HEIGHT: usize = 2000;

fn main() -> io::Result<()> {
    let store = DEBUG_PIPELINE.store(false, std::sync::atomic::Ordering::Relaxed);
    let camera = Camera::new(
        Vec3::new(0.0, 0.0, -3.), // Position camera back a bit
        Vec3::ZERO,               // Look at origin
        WIDTH as f32 / HEIGHT as f32,
    );

    let mut scene = Scene::new(camera);

    scene.add_entity(Entity::from_obj("assets/models/suzanne.obj"));
    scene.entities[0].mesh.bake_normals_to_colors();

    // You can choose which one to run
    run_win(scene)
    // or
    // run_term(scene)
}

pub fn run_term(scene: Scene) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        Hide,
        Clear(ClearType::All)
    )?;

    let pipeline = Pipeline::<TermBuffer>::new(WIDTH, HEIGHT, scene);

    loop {
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.code == KeyCode::Esc {
                    break;
                }
            }
        }

        pipeline.render_frame(None)?;
        thread::sleep(Duration::from_millis(16));
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
