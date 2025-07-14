// TODO: Re-implement the floating point thing I did a while ago that like produced that really
// cool "wiggly" effect from old n64, PS2 and ds 3d games (banjo kazooie, crash bandicoot)
// To do the above, we might need to implement a separate fixed-point pipeline that enforces fixed point math on ALL stages of the pipeline
// that will take, alot of code (maybe)
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
// TODO: pre-baking renders?
//
// TODO: Egui for debug console?
// TODO: Live debug log with egui?
//
//
// FIX: Need to re-implement movement for camera, as the camera crosses the origin, culling needs
// to be flipped, among other things, as well as movement values (from positive to negative)
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use glam::{Affine3A, Vec3};
use minifb::{Key, Scale, Window, WindowOptions};
use std::io::{self};
use std::time::{Duration, Instant};
use terminal_renderer::{
    core::{Camera, Entity, Light, Scene},
    pipeline::{pipeline::Pipeline, FrameBuffer, TermBuffer},
    Color, DEBUG_PIPELINE, TINY_DIMENSIONS,
};

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

fn main() -> io::Result<()> {
    DEBUG_PIPELINE.store(false, std::sync::atomic::Ordering::Relaxed);
    let camera = Camera::new(
        Vec3::new(0.1, 2.0, 8.), // Position camera back a bit
        Vec3::new(0.0, 2.0, 0.),
        WIDTH as f32 / HEIGHT as f32,
    );

    let mut scene = Scene::new(camera);
    // scene.add_entity(floor);
    let point = Light::easy_point(Vec3::new(0., 3., 4.)); // FIX: All lighting calculations are backwards
                                                          //let mut point2 = Light::easy_point(Vec3::new(3., -1., 0.)); // FIX: All lighting calculations are backwards
                                                          //point2.color = Color::from_hex("#6bcaf2").unwrap();

    scene.add_light(point);
    //scene.add_light(point2);

    println!("LOADING PENGUIN MODEL...");
    let mut ent = Entity::new_penguin();
    println!("PENGUIN MODEL LOADED, {} entities created", ent.len());
    ent[0].set_transform(Affine3A::from_rotation_x(0.4));
    for e in ent.iter() {
        scene.add_entity(e.clone());
    }

    println!("STARTING WINDOW RENDERER...");
    // run_term(scene)
    run_win(scene)
}

fn run_term(scene: Scene) -> io::Result<()> {
    // 1) Setup crossterm
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        Hide,
        Clear(ClearType::All)
    )?;

    // 2) Create pipeline
    let (tw, th) = crossterm::terminal::size()?;
    let mut pipeline = Pipeline::<TermBuffer>::new(tw as usize, th as usize, scene);

    // 3) For timing
    let mut last_frame = Instant::now();
    let frame_duration = Duration::from_millis(16); // ~60 FPS

    // 4) Main loop
    'mainloop: loop {
        // (a) Check for input
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if pipeline.handle_crossterm_input(crossterm::event::Event::Key(key), last_frame) {
                    break;
                }
            }
        }

        // (b) Check if enough time has passed
        let now = Instant::now();
        if now - last_frame >= frame_duration {
            // (c) Possibly do any scene updates
            // e.g. pipeline.scene.spin(0);

            // (d) Check if terminal size changed
            let (nw, nh) = crossterm::terminal::size()?;
            if nw as usize != pipeline.width || nh as usize != pipeline.height {
                pipeline =
                    Pipeline::<TermBuffer>::new(nw as usize, nh as usize, pipeline.scene.clone());
            }

            // (e) Render
            pipeline.render_frame(None)?;

            last_frame = now;
        }
    }

    // 5) Cleanup
    cleanup_terminal()?;
    Ok(())
}

fn cleanup_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, Show, terminal::LeaveAlternateScreen)?;
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
    #[cfg(debug_assertions)] // In debug mode (just running like plain cargo r, it will be a lower
    // resolution)
    {
        let (w, h) = TINY_DIMENSIONS;
        window = Window::new(
            "TerminalRasterizer - (DEBUG VERSION)",
            w,
            h,
            WindowOptions {
                resize: false,
                scale: Scale::X4,
                ..WindowOptions::default()
            },
        )
        .expect("Unable to open window :(")
    }

    let mut pipeline = Pipeline::<FrameBuffer>::new(WIDTH, HEIGHT, scene);
    while window.is_open() {
        if window.is_key_down(Key::Escape) || window.is_key_down(Key::Q) {
            break;
        }
        pipeline.render_frame(Some(&mut window))?;
        pipeline.window_handle_input(&window, Instant::now());
    }

    Ok(())
}
