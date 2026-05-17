//! Central diagnostics aggregator.
//!
//! `DiagnosticsFrame` is the single struct the dev overlay reads from. Each
//! subsystem pushes its snapshot in once per frame; the renderer no longer
//! has to own everyone else's counters.
//!
//! Build order is intentional but flexible: callers can fill the fields they
//! have and leave the rest at `Default`. The overlay handles partial frames.

use crate::frame_stats::FrameStats;
use crate::render_stats::RenderStats;
use crate::subsystem_stats::{
    AudioStatsSnapshot, GameplayStatsSnapshot, StreamingStatsSnapshot, WorldgenStatsSnapshot,
};

#[derive(Clone, Debug, Default)]
pub struct DiagnosticsFrame {
    /// FPS / frame-time bookkeeping. Owned by the app's main loop.
    pub frame: FrameStatsSnapshot,
    /// Renderer counters (visible chunks, draw calls, timings…).
    pub render: RenderStats,
    /// Streaming pipeline (mesh jobs, GPU uploads).
    pub streaming: StreamingStatsSnapshot,
    /// Worldgen cache + feature emission.
    pub worldgen: WorldgenStatsSnapshot,
    /// Audio voice count + errors.
    pub audio: AudioStatsSnapshot,
    /// Gameplay-side observer state (biome, chunk, target voxel).
    pub gameplay: GameplayStatsSnapshot,
}

/// Pure-data view of `FrameStats`. The live counter type carries an `Instant`
/// clock so it isn't `Copy`; this is what consumers actually need.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameStatsSnapshot {
    pub fps: u32,
    pub frame_time_ms: f32,
}

impl From<&FrameStats> for FrameStatsSnapshot {
    fn from(stats: &FrameStats) -> Self {
        Self {
            fps: stats.fps(),
            frame_time_ms: stats.frame_time_ms(),
        }
    }
}

impl DiagnosticsFrame {
    /// Empty frame. Consumers can call `with_*` builder methods to fill in
    /// pieces as they become available within a frame.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_frame(mut self, frame: &FrameStats) -> Self {
        self.frame = frame.into();
        self
    }

    pub fn with_render(mut self, render: RenderStats) -> Self {
        self.render = render;
        self
    }

    pub fn with_streaming(mut self, streaming: StreamingStatsSnapshot) -> Self {
        self.streaming = streaming;
        self
    }

    pub fn with_worldgen(mut self, worldgen: WorldgenStatsSnapshot) -> Self {
        self.worldgen = worldgen;
        self
    }

    pub fn with_audio(mut self, audio: AudioStatsSnapshot) -> Self {
        self.audio = audio;
        self
    }

    pub fn with_gameplay(mut self, gameplay: GameplayStatsSnapshot) -> Self {
        self.gameplay = gameplay;
        self
    }
}
