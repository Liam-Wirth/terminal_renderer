use std::{
    fmt::Display,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use clap::{Arg, Command};

pub mod core;
pub mod game;
pub mod pipeline;
pub mod util;

pub use core::geometry;
pub use core::Camera;
pub use core::Color;
pub use core::Entity;
pub use core::Scene;
pub use util::format_mat4;

#[derive(Debug, Clone, Copy)]
pub enum DisplayTarget {
    Terminal,
    Window,
}

pub struct Metrics {
    pub last_frame: Instant,
    pub frame_time: Duration,
    pub fps_counter: u32,
    pub fps_update_timer: Instant,
    pub current_fps: f32,
    pub frame_times: Vec<f32>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
impl Metrics {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            frame_time: Duration::from_secs_f32(1.0 / 60.0),
            fps_counter: 0,
            fps_update_timer: Instant::now(),
            current_fps: 0.0,
            frame_times: Vec::with_capacity(120),
        }
    }

    pub fn update(&mut self, frame_delta: Duration) {
        self.fps_counter += 1;
        self.frame_times.push(frame_delta.as_secs_f32() * 1000.0);
    }
}
impl Display for Metrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FPS: {:.2} | Avg: {:.2}ms | Min: {:.2}ms | Max: {:.2}ms",
            self.current_fps,
            self.frame_time.as_secs_f32() * 1000.0,
            self.frame_times
                .iter()
                .copied()
                .reduce(f32::min)
                .unwrap_or(0.0),
            self.frame_times
                .iter()
                .copied()
                .reduce(f32::max)
                .unwrap_or(0.0)
        )
    }
}

pub static DEBUG_PIPELINE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

// Helper macro for debug printing
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        if $crate::DEBUG_PIPELINE.load(std::sync::atomic::Ordering::Relaxed) {
            println!($($arg)*);
        }
    };
}

pub fn create_clap_command() -> Command {
    Command::new("terminal_renderer")
        .about("3D Software Renderer")
        .version("0.1")
        .author("Liam Wirth")
        .subcommand(
            Command::new("render")
                .about("Render a 3D model in the terminal or a window (using minifb)")
                .arg(
                    Arg::new("mode")
                        .short('m')
                        .long("mode")
                        .value_name("MODE")
                        .help("Specify the mode ('terminal', 'video', 't', or 'v')")
                        .required(false)
                        .value_parser(["terminal", "video", "t", "v"]), // Accept both long and shorthand
                )
                .arg(
                    Arg::new("model")
                        .short('f')
                        .long("model")
                        .value_name("FILE")
                        .help("Specify the absolute path to the .obj model you want to render. If not supplied, a debug model is used.")
                        .required(false),
                ),
        )
}

pub fn handle_clap_matches(matches: &clap::ArgMatches) -> (DisplayTarget, Option<PathBuf>) {
    if let Some(("render", sub_matches)) = matches.subcommand() {
        let mode = sub_matches
            .get_one::<String>("mode")
            .map(|s| s.as_str())
            .unwrap_or("terminal");
        let model = sub_matches
            .get_one::<String>("model")
            .map(|s| s.as_str())
            .unwrap_or("assets/models/african_head.obj");

        let target = match mode {
            "terminal" | "t" => DisplayTarget::Terminal,
            "video" | "v" => DisplayTarget::Window,
            _ => {
                eprintln!("Invalid mode: {}. Defaulting to terminal.", mode);
                DisplayTarget::Terminal
            }
        };
        let model = Path::new(model).to_owned().canonicalize().unwrap();
        return (target, Some(model));
    }

    // Default behavior when no subcommand is provided
    (DisplayTarget::Terminal, None)
}

/// Macro that expands into a match statement handling various key codes.
/// It mirrors the logic from your minifb input handling, but for crossterm.
#[macro_export]
macro_rules! handle_crossterm_keys {
    ($key_code:expr, $states:expr, $scene:expr, $move_amount:expr) => {{
        let mut should_break = false;
        match $key_code {
            // Toggle wireframe
            KeyCode::Char('p') => {
                let current = $states.borrow().draw_wireframe;
                $states.borrow_mut().draw_wireframe = !current;
            }
            // Toggle bake_normals
            KeyCode::Char('b') => {
                let current = $states.borrow().bake_normals;
                $states.borrow_mut().bake_normals = !current;
            }
            // Toggle move_obj
            KeyCode::Char('j') => {
                let current = $states.borrow().move_obj;
                $states.borrow_mut().move_obj = !current;
                println!("Move obj: {}", !current);
            }
            // Decrement current_obj
            KeyCode::Char('[') => {
                let mut current = $states.borrow().current_obj;
                current = current.saturating_sub(1);
                if current > $scene.entities.len() - 1 {
                    current = $scene.entities.len() - 1;
                }
                $states.borrow_mut().current_obj = current;
                println!("Current object: {}", current);
            }
            // Increment current_obj
            KeyCode::Char(']') => {
                let mut current = $states.borrow().current_obj;
                current += 1;
                if current > $scene.entities.len() - 1 {
                    current %= $scene.entities.len();
                }
                $states.borrow_mut().current_obj = current;
                println!("Current object: {}", current);
            }
            // Move forward/back or entity
            KeyCode::Char('w') => {
                let move_obj = $states.borrow().move_obj;
                let current_obj = $states.borrow().current_obj;
                if move_obj {
                    let ent = &$scene.entities[current_obj];
                    let mut t = *ent.transform();
                    t.translation.z += $move_amount;
                    $scene.entities[current_obj].set_transform(t);
                } else {
                    $scene.camera.move_forward($move_amount);
                }
            }
            KeyCode::Char('s') => {
                let move_obj = $states.borrow().move_obj;
                let current_obj = $states.borrow().current_obj;
                if move_obj {
                    let ent = &$scene.entities[current_obj];
                    let mut t = *ent.transform();
                    t.translation.z -= $move_amount;
                    $scene.entities[current_obj].set_transform(t);
                } else {
                    $scene.camera.move_forward(-$move_amount);
                }
            }
            // Move left/right or entity
            KeyCode::Char('a') => {
                let move_obj = $states.borrow().move_obj;
                let current_obj = $states.borrow().current_obj;
                if move_obj {
                    let ent = &$scene.entities[current_obj];
                    let mut t = *ent.transform();
                    t.translation.x -= $move_amount;
                    $scene.entities[current_obj].set_transform(t);
                } else {
                    $scene.camera.move_right(-$move_amount);
                }
            }
            KeyCode::Char('d') => {
                let move_obj = $states.borrow().move_obj;
                let current_obj = $states.borrow().current_obj;
                if move_obj {
                    let ent = &$scene.entities[current_obj];
                    let mut t = *ent.transform();
                    t.translation.x += $move_amount;
                    $scene.entities[current_obj].set_transform(t);
                } else {
                    $scene.camera.move_right($move_amount);
                }
            }
            // Reset entity or camera
            KeyCode::Char('u') => {
                if $states.borrow().move_obj {
                } else {
                    $scene.camera.reset();
                }
            }
            // Move up/down (space/shift in minifb example)
            KeyCode::Char(' ') => {
                let move_obj = $states.borrow().move_obj;
                let current_obj = $states.borrow().current_obj;
                if move_obj {
                    let ent = &$scene.entities[current_obj];
                    let mut t = *ent.transform();
                    t.translation.y += $move_amount;
                    $scene.entities[current_obj].set_transform(t);
                } else {
                    $scene.camera.move_up($move_amount);
                }
            }
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('Q') => {
                should_break = true;
            }
            // Catch-all
            _ => {}
        }
        should_break
    }};
}

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

// TODO: eventually use this to handle mouse input
#[macro_export]
macro_rules! handle_crossterm_mouse {
    ($mouse_event:expr, $states:expr, $scene:expr) => {
        match $mouse_event.kind {
            // When mouse button is pressed down
            MouseEventKind::Down(btn) => {
                match btn {
                    MouseButton::Left => {
                        println!(
                            "Left click at ({}, {})",
                            $mouse_event.column, $mouse_event.row
                        );
                        // Add your "on left click" logic here.
                        // E.g. maybe toggle wireframe, or select an entity, etc.
                        // $states.borrow_mut().some_flag = true;
                    }
                    MouseButton::Right => {
                        println!(
                            "Right click at ({}, {})",
                            $mouse_event.column, $mouse_event.row
                        );
                        // Some other logic for right click
                    }
                    MouseButton::Middle => {
                        println!(
                            "Middle click at ({}, {})",
                            $mouse_event.column, $mouse_event.row
                        );
                    }
                }
            }
            // When mouse button is released
            MouseEventKind::Up(btn) => {
                match btn {
                    MouseButton::Left => {
                        println!("Left button released");
                        // Add custom logic for left release
                    }
                    MouseButton::Right => {
                        println!("Right button released");
                    }
                    MouseButton::Middle => {
                        println!("Middle button released");
                    }
                }
            }
            // When mouse is moved while button is held (drag)
            MouseEventKind::Drag(btn) => {
                println!(
                    "Dragging with {:?} button at ({}, {})",
                    btn, $mouse_event.column, $mouse_event.row
                );
                // If you want, track the last position vs. new position, etc.
            }
            // When mouse simply moves (no buttons held)
            MouseEventKind::Moved => {
                // This fires frequently; you may or may not want to do something
                println!(
                    "Mouse moved to ({}, {})",
                    $mouse_event.column, $mouse_event.row
                );
            }
            // Scroll wheel
            MouseEventKind::ScrollDown => {
                println!(
                    "Scrolled down at ({}, {})",
                    $mouse_event.column, $mouse_event.row
                );
                // Maybe zoom the camera out or do something
            }
            MouseEventKind::ScrollUp => {
                println!(
                    "Scrolled up at ({}, {})",
                    $mouse_event.column, $mouse_event.row
                );
                // Maybe zoom the camera in
            }
        }
    };
}
