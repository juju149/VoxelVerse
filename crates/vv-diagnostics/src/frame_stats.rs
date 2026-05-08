use std::time::{Duration, Instant};

pub struct FrameStats {
    last_fps_sample: Instant,
    frame_count: u32,
    current_fps: u32,
    last_frame_at: Instant,
    last_frame_time: Duration,
}

impl FrameStats {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            last_fps_sample: now,
            frame_count: 0,
            current_fps: 0,
            last_frame_at: now,
            last_frame_time: Duration::ZERO,
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.last_frame_time = now.duration_since(self.last_frame_at);
        self.last_frame_at = now;
        self.frame_count += 1;

        if now.duration_since(self.last_fps_sample).as_secs_f32() >= 1.0 {
            self.current_fps = self.frame_count;
            self.frame_count = 0;
            self.last_fps_sample = now;
        }
    }

    pub fn fps(&self) -> u32 {
        self.current_fps
    }

    pub fn frame_time_ms(&self) -> f32 {
        self.last_frame_time.as_secs_f32() * 1000.0
    }
}

impl Default for FrameStats {
    fn default() -> Self {
        Self::new()
    }
}
