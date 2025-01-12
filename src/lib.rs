use std::{path::{Path, PathBuf}, time::{Duration, Instant}};

use clap::{Arg, Command, Subcommand};

pub mod core;
pub mod game;
pub mod pipeline;
pub mod util;

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

    fn to_string(&self) -> String {
        format!(
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
        if crate::DEBUG_PIPELINE.load(std::sync::atomic::Ordering::Relaxed) {
            println!($($arg)*);
        }
    };
}

pub fn create_clap_command() -> Command<> {
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
        let mode = sub_matches.get_one::<String>("mode").map(|s| s.as_str()).unwrap_or("terminal");
        let model = sub_matches.get_one::<String>("model").map(|s| s.as_str()).unwrap_or("assets/models/african_head.obj");

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

