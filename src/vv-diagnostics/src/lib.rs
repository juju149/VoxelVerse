use std::{
    collections::VecDeque,
    env, fmt,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    fn from_env_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" | "warning" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticMode {
    Normal,
    Debug,
    Perf,
}

impl DiagnosticMode {
    fn from_env_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "normal" | "off" | "0" => Some(Self::Normal),
            "debug" => Some(Self::Debug),
            "perf" | "performance" | "diagnostic" | "diagnostics" | "1" => Some(Self::Perf),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Debug => "debug",
            Self::Perf => "perf",
        }
    }

    pub fn detailed_snapshots(self) -> bool {
        matches!(self, Self::Debug | Self::Perf)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogDomain {
    Startup,
    Config,
    World,
    Worldgen,
    Streaming,
    Lod,
    Mesh,
    Render,
    Gpu,
    Physics,
    Gameplay,
    Ui,
    Input,
    Memory,
    Perf,
    Warnings,
    Errors,
}

impl LogDomain {
    fn as_str(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::Config => "config",
            Self::World => "world",
            Self::Worldgen => "worldgen",
            Self::Streaming => "streaming",
            Self::Lod => "lod",
            Self::Mesh => "mesh",
            Self::Render => "render",
            Self::Gpu => "gpu",
            Self::Physics => "physics",
            Self::Gameplay => "gameplay",
            Self::Ui => "ui",
            Self::Input => "input",
            Self::Memory => "memory",
            Self::Perf => "perf",
            Self::Warnings => "warnings",
            Self::Errors => "errors",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerfPhase {
    Frame,
    Input,
    Physics,
    Gameplay,
    Ui,
    ViewVisibility,
    LodCoverage,
    ChunkStreaming,
    Worldgen,
    Meshing,
    GpuUpload,
    RenderPrep,
    Render,
}

impl PerfPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Frame => "frame",
            Self::Input => "input",
            Self::Physics => "physics",
            Self::Gameplay => "gameplay",
            Self::Ui => "ui",
            Self::ViewVisibility => "view",
            Self::LodCoverage => "lod",
            Self::ChunkStreaming => "streaming",
            Self::Worldgen => "worldgen",
            Self::Meshing => "mesh",
            Self::GpuUpload => "gpu_upload",
            Self::RenderPrep => "render_prep",
            Self::Render => "render",
        }
    }

    fn domain(self) -> LogDomain {
        match self {
            Self::Frame => LogDomain::Perf,
            Self::Input => LogDomain::Input,
            Self::Physics => LogDomain::Physics,
            Self::Gameplay => LogDomain::Gameplay,
            Self::Ui => LogDomain::Ui,
            Self::ViewVisibility => LogDomain::World,
            Self::LodCoverage => LogDomain::Lod,
            Self::ChunkStreaming => LogDomain::Streaming,
            Self::Worldgen => LogDomain::Worldgen,
            Self::Meshing => LogDomain::Mesh,
            Self::GpuUpload => LogDomain::Gpu,
            Self::RenderPrep | Self::Render => LogDomain::Render,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PerformanceBudgets {
    pub frame_warn: Duration,
    pub frame_freeze: Duration,
    pub input: Duration,
    pub physics: Duration,
    pub gameplay: Duration,
    pub ui: Duration,
    pub view_visibility: Duration,
    pub lod_coverage: Duration,
    pub chunk_streaming: Duration,
    pub worldgen: Duration,
    pub meshing: Duration,
    pub gpu_upload: Duration,
    pub render_prep: Duration,
    pub render: Duration,
}

impl Default for PerformanceBudgets {
    fn default() -> Self {
        Self {
            frame_warn: Duration::from_millis(50),
            frame_freeze: Duration::from_millis(100),
            input: Duration::from_millis(2),
            physics: Duration::from_millis(4),
            gameplay: Duration::from_millis(4),
            ui: Duration::from_millis(4),
            view_visibility: Duration::from_millis(8),
            lod_coverage: Duration::from_millis(6),
            chunk_streaming: Duration::from_millis(8),
            worldgen: Duration::from_millis(12),
            meshing: Duration::from_millis(12),
            gpu_upload: Duration::from_millis(8),
            render_prep: Duration::from_millis(8),
            render: Duration::from_millis(16),
        }
    }
}

impl PerformanceBudgets {
    fn budget_for(self, phase: PerfPhase) -> Duration {
        match phase {
            PerfPhase::Frame => self.frame_warn,
            PerfPhase::Input => self.input,
            PerfPhase::Physics => self.physics,
            PerfPhase::Gameplay => self.gameplay,
            PerfPhase::Ui => self.ui,
            PerfPhase::ViewVisibility => self.view_visibility,
            PerfPhase::LodCoverage => self.lod_coverage,
            PerfPhase::ChunkStreaming => self.chunk_streaming,
            PerfPhase::Worldgen => self.worldgen,
            PerfPhase::Meshing => self.meshing,
            PerfPhase::GpuUpload => self.gpu_upload,
            PerfPhase::RenderPrep => self.render_prep,
            PerfPhase::Render => self.render,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DiagnosticConfig {
    pub mode: DiagnosticMode,
    pub min_level: LogLevel,
    pub snapshot_interval: Duration,
    pub budgets: PerformanceBudgets,
}

impl DiagnosticConfig {
    pub fn from_env() -> Self {
        let mode = env::var("VV_DIAGNOSTICS")
            .ok()
            .and_then(|value| DiagnosticMode::from_env_value(&value))
            .unwrap_or(DiagnosticMode::Normal);
        let min_level = env::var("VV_LOG")
            .ok()
            .and_then(|value| LogLevel::from_env_value(&value))
            .unwrap_or(match mode {
                DiagnosticMode::Normal => LogLevel::Info,
                DiagnosticMode::Debug | DiagnosticMode::Perf => LogLevel::Debug,
            });
        let snapshot_interval = match mode {
            DiagnosticMode::Normal => Duration::from_secs(5),
            DiagnosticMode::Debug => Duration::from_secs(2),
            DiagnosticMode::Perf => Duration::from_secs(1),
        };
        Self {
            mode,
            min_level,
            snapshot_interval,
            budgets: PerformanceBudgets::default(),
        }
    }

    pub fn enabled(self, level: LogLevel) -> bool {
        level >= self.min_level || matches!(level, LogLevel::Warn | LogLevel::Error)
    }
}

pub fn emit_from_env(level: LogLevel, domain: LogDomain, message: impl fmt::Display) {
    emit(DiagnosticConfig::from_env(), level, domain, message);
}

pub fn emit(
    config: DiagnosticConfig,
    level: LogLevel,
    domain: LogDomain,
    message: impl fmt::Display,
) {
    if !config.enabled(level) {
        return;
    }
    if matches!(level, LogLevel::Info) {
        println!("[{}] {}", domain.as_str(), message);
    } else {
        println!("[{}][{}] {}", level.as_str(), domain.as_str(), message);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PhaseSample {
    pub phase: PerfPhase,
    pub duration: Duration,
}

pub struct PhaseTimer {
    phase: PerfPhase,
    start: Instant,
}

impl PhaseTimer {
    pub fn start(phase: PerfPhase) -> Self {
        Self {
            phase,
            start: Instant::now(),
        }
    }

    pub fn finish(self) -> PhaseSample {
        PhaseSample {
            phase: self.phase,
            duration: self.start.elapsed(),
        }
    }
}

pub struct FrameRecord {
    index: u64,
    dt_seconds: f32,
    start: Instant,
    phases: Vec<PhaseSample>,
}

impl FrameRecord {
    fn new(index: u64, dt_seconds: f32) -> Self {
        Self {
            index,
            dt_seconds,
            start: Instant::now(),
            phases: Vec::new(),
        }
    }

    pub fn record(&mut self, sample: PhaseSample) {
        self.phases.push(sample);
    }

    pub fn record_duration(&mut self, phase: PerfPhase, duration: Duration) {
        self.phases.push(PhaseSample { phase, duration });
    }

    pub fn phase_duration(&self, phase: PerfPhase) -> Duration {
        self.phases
            .iter()
            .filter(|sample| sample.phase == phase)
            .map(|sample| sample.duration)
            .sum()
    }
}

#[derive(Clone, Debug, Default)]
pub struct WorldCounters {
    pub edited_chunks: usize,
    pub mined_blocks: usize,
    pub placed_blocks: usize,
    pub dirty_chunks: usize,
}

#[derive(Clone, Debug, Default)]
pub struct WorldgenCounters {
    pub cached_columns: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub compute_time: Duration,
}

#[derive(Clone, Debug, Default)]
pub struct StreamingCounters {
    pub active_chunks: usize,
    pub required_chunks: usize,
    pub missing_chunks: usize,
    pub load_queue: usize,
    pub pending_chunk_jobs: usize,
    pub chunks_uploaded: u32,
    pub chunk_jobs_started: u32,
    pub chunks_invalidated: u32,
    pub empty_chunks: u32,
}

#[derive(Clone, Debug, Default)]
pub struct LodCounters {
    pub active_lods: usize,
    pub required_lods: usize,
    pub covered_lods: usize,
    pub missing_lods: usize,
    pub pending_lod_jobs: usize,
    pub lods_uploaded: u32,
    pub lod_jobs_started: u32,
    pub coverage_percent: f32,
}

#[derive(Clone, Debug, Default)]
pub struct MeshCounters {
    pub chunk_jobs_completed: u32,
    pub lod_jobs_completed: u32,
    pub remeshes: u32,
    pub max_job_time: Duration,
    pub total_job_time: Duration,
    pub mesh_cpu_mb: f32,
    pub active_meshes: usize,
    pub vertices: usize,
    pub indices: usize,
}

#[derive(Clone, Debug, Default)]
pub struct GpuCounters {
    pub uploads: u32,
    pub upload_vertices: usize,
    pub upload_indices: usize,
    pub upload_time: Duration,
    pub draw_calls: u32,
    pub visible_chunks: usize,
    pub visible_lods: usize,
    pub active_buffers: usize,
}

#[derive(Clone, Debug, Default)]
pub struct RuntimeSnapshot {
    pub world: WorldCounters,
    pub worldgen: WorldgenCounters,
    pub streaming: StreamingCounters,
    pub lod: LodCounters,
    pub mesh: MeshCounters,
    pub gpu: GpuCounters,
    pub dropped_items: usize,
    pub inventory_open: bool,
}

pub struct EngineDiagnostics {
    config: DiagnosticConfig,
    frame_index: u64,
    last_snapshot: Instant,
    recent_frames: VecDeque<Duration>,
    recent_spikes: VecDeque<Duration>,
    last_worldgen_compute_time: Duration,
}

impl EngineDiagnostics {
    pub fn from_env() -> Self {
        let config = DiagnosticConfig::from_env();
        Self {
            config,
            frame_index: 0,
            last_snapshot: Instant::now(),
            recent_frames: VecDeque::with_capacity(240),
            recent_spikes: VecDeque::with_capacity(16),
            last_worldgen_compute_time: Duration::ZERO,
        }
    }

    pub fn config(&self) -> DiagnosticConfig {
        self.config
    }

    pub fn log(&self, level: LogLevel, domain: LogDomain, message: impl fmt::Display) {
        emit(self.config, level, domain, message);
    }

    pub fn begin_frame(&mut self, dt_seconds: f32) -> FrameRecord {
        self.frame_index += 1;
        FrameRecord::new(self.frame_index, dt_seconds)
    }

    pub fn finish_frame(&mut self, mut record: FrameRecord, snapshot: RuntimeSnapshot) {
        let worldgen_delta = snapshot
            .worldgen
            .compute_time
            .saturating_sub(self.last_worldgen_compute_time);
        self.last_worldgen_compute_time = snapshot.worldgen.compute_time;
        let total = record.start.elapsed();
        record.record_duration(PerfPhase::Frame, total);
        self.record_frame(total);
        self.warn_on_budget_overruns(&record, &snapshot);
        if worldgen_delta > self.config.budgets.worldgen {
            self.log(
                LogLevel::Warn,
                LogDomain::Worldgen,
                format!(
                    "terrain generation pressure frame={} cpu_total={} budget={} cache_misses={} cached_columns={}",
                    record.index,
                    fmt_duration(worldgen_delta),
                    fmt_duration(self.config.budgets.worldgen),
                    snapshot.worldgen.cache_misses,
                    snapshot.worldgen.cached_columns
                ),
            );
        }

        if self.last_snapshot.elapsed() >= self.config.snapshot_interval {
            self.print_snapshot(&record, &snapshot, total);
            self.last_snapshot = Instant::now();
        }
    }

    pub fn record_startup_phase(&self, phase: PerfPhase, duration: Duration) {
        let budget = self.config.budgets.budget_for(phase);
        if duration > budget {
            self.log(
                LogLevel::Warn,
                phase.domain(),
                format!(
                    "{} startup phase took {} budget={}",
                    phase.as_str(),
                    fmt_duration(duration),
                    fmt_duration(budget)
                ),
            );
        } else if self.config.mode.detailed_snapshots() {
            self.log(
                LogLevel::Debug,
                phase.domain(),
                format!(
                    "{} startup phase took {}",
                    phase.as_str(),
                    fmt_duration(duration)
                ),
            );
        }
    }

    fn record_frame(&mut self, total: Duration) {
        self.recent_frames.push_back(total);
        while self.recent_frames.len() > 240 {
            self.recent_frames.pop_front();
        }
        if total >= self.config.budgets.frame_warn {
            self.recent_spikes.push_back(total);
            while self.recent_spikes.len() > 16 {
                self.recent_spikes.pop_front();
            }
        }
    }

    fn warn_on_budget_overruns(&self, record: &FrameRecord, snapshot: &RuntimeSnapshot) {
        let frame = record.phase_duration(PerfPhase::Frame);
        if frame >= self.config.budgets.frame_freeze {
            self.log(
                LogLevel::Error,
                LogDomain::Perf,
                format!(
                    "freeze frame={} time={} dt={} chunks={} lods={} queue={} pending_chunks={} pending_lods={} gpu_uploads={} mesh_jobs={}",
                    record.index,
                    fmt_duration(frame),
                    ms(record.dt_seconds as f64 * 1000.0),
                    snapshot.streaming.active_chunks,
                    snapshot.lod.active_lods,
                    snapshot.streaming.load_queue,
                    snapshot.streaming.pending_chunk_jobs,
                    snapshot.lod.pending_lod_jobs,
                    snapshot.gpu.uploads,
                    snapshot.mesh.chunk_jobs_completed + snapshot.mesh.lod_jobs_completed,
                ),
            );
        } else if frame >= self.config.budgets.frame_warn {
            self.log(
                LogLevel::Warn,
                LogDomain::Perf,
                format!(
                    "slow frame={} time={} budget={} queue={} missing_chunks={} missing_lods={} draw_calls={}",
                    record.index,
                    fmt_duration(frame),
                    fmt_duration(self.config.budgets.frame_warn),
                    snapshot.streaming.load_queue,
                    snapshot.streaming.missing_chunks,
                    snapshot.lod.missing_lods,
                    snapshot.gpu.draw_calls,
                ),
            );
        }

        for sample in &record.phases {
            if sample.phase == PerfPhase::Frame {
                continue;
            }
            let budget = self.config.budgets.budget_for(sample.phase);
            if sample.duration > budget {
                self.log(
                    LogLevel::Warn,
                    sample.phase.domain(),
                    format!(
                        "phase={} took {} budget={} frame={}",
                        sample.phase.as_str(),
                        fmt_duration(sample.duration),
                        fmt_duration(budget),
                        record.index
                    ),
                );
            }
        }

        if snapshot.lod.required_lods > 0 && snapshot.lod.covered_lods == 0 {
            self.log(
                LogLevel::Warn,
                LogDomain::Lod,
                format!(
                    "coverage dropped to 0 required_lods={} pending_lod_jobs={} active_lods={}",
                    snapshot.lod.required_lods,
                    snapshot.lod.pending_lod_jobs,
                    snapshot.lod.active_lods
                ),
            );
        }
        if snapshot.streaming.load_queue > 128 {
            self.log(
                LogLevel::Warn,
                LogDomain::Streaming,
                format!(
                    "chunk queue high queue={} required={} missing={} pending={}",
                    snapshot.streaming.load_queue,
                    snapshot.streaming.required_chunks,
                    snapshot.streaming.missing_chunks,
                    snapshot.streaming.pending_chunk_jobs
                ),
            );
        }
        if snapshot.gpu.uploads > 20 || snapshot.gpu.upload_time > self.config.budgets.gpu_upload {
            self.log(
                LogLevel::Warn,
                LogDomain::Gpu,
                format!(
                    "upload pressure uploads={} time={} vertices={} indices={}",
                    snapshot.gpu.uploads,
                    fmt_duration(snapshot.gpu.upload_time),
                    snapshot.gpu.upload_vertices,
                    snapshot.gpu.upload_indices
                ),
            );
        }
        if snapshot.mesh.max_job_time > self.config.budgets.meshing {
            self.log(
                LogLevel::Warn,
                LogDomain::Mesh,
                format!(
                    "slow mesh job max_job={} total_completed_cpu={} completed_jobs={} active_meshes={}",
                    fmt_duration(snapshot.mesh.max_job_time),
                    fmt_duration(snapshot.mesh.total_job_time),
                    snapshot.mesh.chunk_jobs_completed + snapshot.mesh.lod_jobs_completed,
                    snapshot.mesh.active_meshes,
                ),
            );
        }
    }

    fn print_snapshot(&self, record: &FrameRecord, snapshot: &RuntimeSnapshot, total: Duration) {
        let avg = average_duration(&self.recent_frames);
        let worst = self.recent_frames.iter().copied().max().unwrap_or_default();
        let fps = if avg.as_secs_f64() > 0.0 {
            1.0 / avg.as_secs_f64()
        } else {
            0.0
        };
        self.log(
            LogLevel::Info,
            LogDomain::Perf,
            format!(
                "fps={} frame={} avg={} worst={} spikes={} physics={} gameplay={} view={} render={}",
                fps.round() as u32,
                fmt_duration(total),
                fmt_duration(avg),
                fmt_duration(worst),
                self.recent_spikes.len(),
                fmt_duration(record.phase_duration(PerfPhase::Physics)),
                fmt_duration(record.phase_duration(PerfPhase::Gameplay)),
                fmt_duration(record.phase_duration(PerfPhase::ViewVisibility)),
                fmt_duration(record.phase_duration(PerfPhase::Render)),
            ),
        );

        if !self.config.mode.detailed_snapshots() {
            return;
        }

        self.log(
            LogLevel::Info,
            LogDomain::Streaming,
            format!(
                "chunks active={} required={} missing={} queue={} pending={} uploaded={} started={} invalidated={} dirty={}",
                snapshot.streaming.active_chunks,
                snapshot.streaming.required_chunks,
                snapshot.streaming.missing_chunks,
                snapshot.streaming.load_queue,
                snapshot.streaming.pending_chunk_jobs,
                snapshot.streaming.chunks_uploaded,
                snapshot.streaming.chunk_jobs_started,
                snapshot.streaming.chunks_invalidated,
                snapshot.world.dirty_chunks,
            ),
        );
        self.log(
            LogLevel::Info,
            LogDomain::Lod,
            format!(
                "active={} required={} covered={} missing={} coverage={:.0}% pending={} uploaded={} started={}",
                snapshot.lod.active_lods,
                snapshot.lod.required_lods,
                snapshot.lod.covered_lods,
                snapshot.lod.missing_lods,
                snapshot.lod.coverage_percent,
                snapshot.lod.pending_lod_jobs,
                snapshot.lod.lods_uploaded,
                snapshot.lod.lod_jobs_started,
            ),
        );
        self.log(
            LogLevel::Info,
            LogDomain::Mesh,
            format!(
                "active={} vertices={} indices={} cpu={} jobs_done={} remeshes={} max_job={} total_job={}",
                snapshot.mesh.active_meshes,
                snapshot.mesh.vertices,
                snapshot.mesh.indices,
                mb(snapshot.mesh.mesh_cpu_mb),
                snapshot.mesh.chunk_jobs_completed + snapshot.mesh.lod_jobs_completed,
                snapshot.mesh.remeshes,
                fmt_duration(snapshot.mesh.max_job_time),
                fmt_duration(snapshot.mesh.total_job_time),
            ),
        );
        self.log(
            LogLevel::Info,
            LogDomain::Gpu,
            format!(
                "uploads={} upload_time={} draw_calls={} visible_chunks={} visible_lods={} buffers={}",
                snapshot.gpu.uploads,
                fmt_duration(snapshot.gpu.upload_time),
                snapshot.gpu.draw_calls,
                snapshot.gpu.visible_chunks,
                snapshot.gpu.visible_lods,
                snapshot.gpu.active_buffers,
            ),
        );
        self.log(
            LogLevel::Info,
            LogDomain::Worldgen,
            format!(
                "columns_cached={} cache_hits={} cache_misses={} compute_total={}",
                snapshot.worldgen.cached_columns,
                snapshot.worldgen.cache_hits,
                snapshot.worldgen.cache_misses,
                fmt_duration(snapshot.worldgen.compute_time),
            ),
        );
        self.log(
            LogLevel::Info,
            LogDomain::Memory,
            format!(
                "edited_chunks={} mined_blocks={} placed_blocks={} mesh_cpu={}",
                snapshot.world.edited_chunks,
                snapshot.world.mined_blocks,
                snapshot.world.placed_blocks,
                mb(snapshot.mesh.mesh_cpu_mb),
            ),
        );
    }
}

fn average_duration(values: &VecDeque<Duration>) -> Duration {
    if values.is_empty() {
        return Duration::ZERO;
    }
    let total: Duration = values.iter().copied().sum();
    total / values.len() as u32
}

pub fn fmt_duration(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros >= 10_000 {
        format!("{:.1}ms", duration.as_secs_f64() * 1000.0)
    } else if micros >= 1_000 {
        format!("{:.2}ms", duration.as_secs_f64() * 1000.0)
    } else {
        format!("{micros}us")
    }
}

fn ms(value: f64) -> String {
    format!("{value:.2}ms")
}

fn mb(value: f32) -> String {
    format!("{value:.2}MB")
}
