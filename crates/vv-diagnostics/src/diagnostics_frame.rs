//! Central diagnostics aggregator.
//!
//! `DiagnosticsFrame` is the single struct the dev overlay reads from. Each
//! subsystem pushes its snapshot in once per frame; the renderer no longer
//! has to own everyone else's counters.
//!
//! Build order is intentional but flexible: callers can fill the fields they
//! have and leave the rest at `Default`. The overlay handles partial frames.

use crate::frame_stats::FrameStats;
use crate::metric::{
    CounterSample, CpuScopeTiming, DiagnosticCategory, DiagnosticWarning, DiagnosticsProfile,
    GaugeSample, PlayerCameraSnapshot, RenderPassTiming, WorkloadSnapshot,
};
use crate::render_stats::RenderStats;
use crate::subsystem_stats::{
    AudioStatsSnapshot, GameplayStatsSnapshot, StreamingStatsSnapshot, WorldgenStatsSnapshot,
};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize)]
pub struct DiagnosticsFrame {
    pub timestamp_ms: u128,
    pub profile: DiagnosticsProfile,
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
    /// Static-name event counters emitted during this frame.
    pub counters: Vec<CounterSample>,
    /// Static-name gauges emitted during this frame.
    pub gauges: Vec<GaugeSample>,
    /// CPU scopes captured through `Diagnostics::time_scope`.
    pub cpu_scopes: Vec<CpuScopeTiming>,
    /// Per-pass render timings. GPU timestamp queries can map here once wired.
    pub render_pass_timings: Vec<RenderPassTiming>,
    pub camera: PlayerCameraSnapshot,
    pub workload: WorkloadSnapshot,
    pub warnings: Vec<DiagnosticWarning>,
}

/// Pure-data view of `FrameStats`. The live counter type carries an `Instant`
/// clock so it isn't `Copy`; this is what consumers actually need.
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct FrameStatsSnapshot {
    pub frame_index: u64,
    pub fps: u32,
    pub fps_rolling_1s: f32,
    pub fps_rolling_5s: f32,
    pub frame_time_ms: f32,
    pub dt_seconds: f32,
    pub update_ms: f32,
    pub render_ms: f32,
    pub idle_wait_ms: f32,
}

impl From<&FrameStats> for FrameStatsSnapshot {
    fn from(stats: &FrameStats) -> Self {
        Self {
            frame_index: stats.frame_index(),
            fps: stats.fps(),
            fps_rolling_1s: stats.fps_rolling_1s(),
            fps_rolling_5s: stats.fps_rolling_5s(),
            frame_time_ms: stats.frame_time_ms(),
            dt_seconds: stats.dt_seconds(),
            update_ms: 0.0,
            render_ms: 0.0,
            idle_wait_ms: 0.0,
        }
    }
}

impl Default for DiagnosticsFrame {
    fn default() -> Self {
        Self {
            timestamp_ms: current_time_ms(),
            profile: DiagnosticsProfile::Normal,
            frame: FrameStatsSnapshot::default(),
            render: RenderStats::default(),
            streaming: StreamingStatsSnapshot::default(),
            worldgen: WorldgenStatsSnapshot::default(),
            audio: AudioStatsSnapshot::default(),
            gameplay: GameplayStatsSnapshot::default(),
            counters: Vec::new(),
            gauges: Vec::new(),
            cpu_scopes: Vec::new(),
            render_pass_timings: Vec::new(),
            camera: PlayerCameraSnapshot::default(),
            workload: WorkloadSnapshot::default(),
            warnings: Vec::new(),
        }
    }
}

impl DiagnosticsFrame {
    /// Empty frame. Consumers can call `with_*` builder methods to fill in
    /// pieces as they become available within a frame.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&mut self, category: DiagnosticCategory, name: &'static str, value: u64) {
        if let Some(sample) = self
            .counters
            .iter_mut()
            .find(|sample| sample.category == category && sample.name == name)
        {
            sample.value = sample.value.saturating_add(value);
            return;
        }
        self.counters.push(CounterSample {
            name,
            category,
            value,
        });
    }

    pub fn gauge(&mut self, category: DiagnosticCategory, name: &'static str, value: f64) {
        if let Some(sample) = self
            .gauges
            .iter_mut()
            .find(|sample| sample.category == category && sample.name == name)
        {
            sample.value = value;
            return;
        }
        self.gauges.push(GaugeSample {
            name,
            category,
            value,
        });
    }

    pub fn record_cpu_scope(
        &mut self,
        category: DiagnosticCategory,
        name: &'static str,
        elapsed_ms: f32,
    ) {
        self.cpu_scopes.push(CpuScopeTiming {
            name,
            category,
            elapsed_ms,
        });
    }

    pub fn record_render_pass(&mut self, name: &'static str, elapsed_ms: f32) {
        self.render_pass_timings
            .push(RenderPassTiming { name, elapsed_ms });
    }

    pub fn warn(&mut self, category: DiagnosticCategory, message: impl Into<String>) {
        self.warnings.push(DiagnosticWarning {
            category,
            message: message.into(),
        });
    }

    pub fn with_frame(mut self, frame: &FrameStats) -> Self {
        self.frame = frame.into();
        self
    }

    pub fn with_render(mut self, render: RenderStats) -> Self {
        self.workload.pending_jobs =
            (render.mesh_jobs_in_flight + render.lod_jobs_in_flight).min(u32::MAX as usize) as u32;
        self.workload.pending_chunks = render.pending_chunks.min(u32::MAX as usize) as u32;
        self.workload.pending_lods = render.pending_lods.min(u32::MAX as usize) as u32;
        self.workload.uploaded_meshes = render.uploads_this_frame.min(u32::MAX as usize) as u32;
        self.workload.draw_calls =
            (render.draw_calls + render.shadow_draw_calls).min(u32::MAX as usize) as u32;
        self.workload.gpu_memory_bytes = render.estimated_gpu_bytes() as u64;
        self.render = render;
        self
    }

    pub fn with_streaming(mut self, streaming: StreamingStatsSnapshot) -> Self {
        self.workload.pending_jobs = streaming
            .pending_mesh_jobs
            .saturating_add(streaming.pending_lod_jobs);
        self.workload.uploaded_meshes = streaming.uploads_this_frame;
        self.workload.upload_bytes = streaming.upload_bytes_this_frame;
        self.workload.pending_chunks = streaming.queued_chunks;
        self.streaming = streaming;
        self
    }

    pub fn with_worldgen(mut self, worldgen: WorldgenStatsSnapshot) -> Self {
        self.workload.worldgen_samples = worldgen
            .cell_hits
            .saturating_add(worldgen.cell_misses)
            .min(u32::MAX as u64) as u32;
        self.workload.baked_props = worldgen.props_emitted.min(u32::MAX as u64) as u32;
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

fn current_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
