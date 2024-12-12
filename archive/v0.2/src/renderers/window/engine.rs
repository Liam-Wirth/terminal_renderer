use std::time::{Duration, Instant};

use crate::core::{Camera, Scene};

use super::WindowPipeline;

pub struct Engine {
    pub renderer: WindowPipeline,
    pub scene: Scene,
    pub camera: Camera,
    last_frame: Instant,
    frame_time: Duration,
    fps_counter: u32,
    fps_update_timer: Instant,
    current_fps: f32,
    frame_times: Vec<f32>, // Store last N frame times for averaging
    metrics: String,
    tris: u32,
}
