use std::time::{Duration, Instant};

pub mod core;
pub mod game;
pub mod pipeline;

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
