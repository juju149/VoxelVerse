mod diagnostics_frame;
mod export;
mod frame_stats;
mod metric;
mod recorder;
mod render_stats;
mod ring_buffer;
mod spike;
mod stats;
mod subsystem_stats;
mod system;

pub use diagnostics_frame::{DiagnosticsFrame, FrameStatsSnapshot};
pub use export::{
    append_spike_report, export_latest_frame, export_rolling_summary, export_timeline,
    DiagnosticsFileSink,
};
pub use frame_stats::FrameStats;
pub use metric::{
    CounterSample, CpuScopeTiming, DiagnosticCategory, DiagnosticWarning, DiagnosticsProfile,
    GaugeSample, MetricName, PlayerCameraSnapshot, RenderPassTiming, WorkloadSnapshot,
};
pub use recorder::{CpuScopeGuard, Diagnostics, DiagnosticsConfig};
pub use render_stats::RenderStats;
pub use ring_buffer::{DiagnosticsRingBuffer, RollingDiagnosticsSummary};
pub use spike::{SpikeReport, SpikeThresholds};
pub use stats::{RollingWindowStats, RunningStats};
pub use subsystem_stats::{
    AudioStatsSnapshot, GameplayStatsSnapshot, StreamingStatsSnapshot, WorldgenStatsSnapshot,
};
pub use system::{GpuAdapterSnapshot, SystemDiagnostics};
