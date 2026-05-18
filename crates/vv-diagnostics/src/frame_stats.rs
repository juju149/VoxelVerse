use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct FrameStats {
    frame_index: u64,
    last_fps_sample: Instant,
    frame_count: u32,
    current_fps: u32,
    last_frame_at: Instant,
    last_frame_time: Duration,
    recent_frames: VecDeque<(Instant, Duration)>,
}

impl FrameStats {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            frame_index: 0,
            last_fps_sample: now,
            frame_count: 0,
            current_fps: 0,
            last_frame_at: now,
            last_frame_time: Duration::ZERO,
            recent_frames: VecDeque::with_capacity(512),
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.last_frame_time = now.duration_since(self.last_frame_at);
        self.last_frame_at = now;
        self.frame_index = self.frame_index.saturating_add(1);
        self.frame_count += 1;
        self.recent_frames.push_back((now, self.last_frame_time));
        while self
            .recent_frames
            .front()
            .is_some_and(|(sample_at, _)| now.duration_since(*sample_at).as_secs_f32() > 5.0)
        {
            self.recent_frames.pop_front();
        }

        if now.duration_since(self.last_fps_sample).as_secs_f32() >= 1.0 {
            self.current_fps = self.frame_count;
            self.frame_count = 0;
            self.last_fps_sample = now;
        }
    }

    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub fn fps(&self) -> u32 {
        self.current_fps
    }

    pub fn fps_rolling_1s(&self) -> f32 {
        self.rolling_fps(Duration::from_secs(1))
    }

    pub fn fps_rolling_5s(&self) -> f32 {
        self.rolling_fps(Duration::from_secs(5))
    }

    pub fn frame_time_ms(&self) -> f32 {
        self.last_frame_time.as_secs_f32() * 1000.0
    }

    pub fn dt_seconds(&self) -> f32 {
        self.last_frame_time.as_secs_f32()
    }

    fn rolling_fps(&self, window: Duration) -> f32 {
        let Some((latest_at, _)) = self.recent_frames.back() else {
            return 0.0;
        };
        let mut count = 0usize;
        let mut total = Duration::ZERO;
        for (sample_at, elapsed) in self.recent_frames.iter().rev() {
            if latest_at.duration_since(*sample_at) > window {
                break;
            }
            count += 1;
            total += *elapsed;
        }
        if count == 0 || total.is_zero() {
            0.0
        } else {
            count as f32 / total.as_secs_f32()
        }
    }
}

impl Default for FrameStats {
    fn default() -> Self {
        Self::new()
    }
}
