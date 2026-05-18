use crate::diagnostics_frame::DiagnosticsFrame;
use crate::export::{
    append_spike_report, export_latest_frame, export_rolling_summary, export_timeline,
};
use crate::metric::{DiagnosticCategory, DiagnosticsProfile};
use crate::ring_buffer::{DiagnosticsRingBuffer, RollingDiagnosticsSummary};
use crate::spike::{SpikeReport, SpikeThresholds};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct DiagnosticsConfig {
    pub profile: DiagnosticsProfile,
    pub ring_capacity_frames: usize,
    pub export_dir: PathBuf,
    pub spike_thresholds: SpikeThresholds,
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            profile: DiagnosticsProfile::Normal,
            ring_capacity_frames: 1800,
            export_dir: PathBuf::from("logs/diagnostics"),
            spike_thresholds: SpikeThresholds::default(),
        }
    }
}

pub struct Diagnostics {
    state: Mutex<DiagnosticsState>,
}

struct DiagnosticsState {
    config: DiagnosticsConfig,
    current: DiagnosticsFrame,
    ring: DiagnosticsRingBuffer,
    spikes: Vec<SpikeReport>,
}

pub struct CpuScopeGuard<'a> {
    diagnostics: &'a Diagnostics,
    category: DiagnosticCategory,
    name: &'static str,
    start: Option<Instant>,
}

impl Diagnostics {
    pub fn new(config: DiagnosticsConfig) -> Self {
        let mut current = DiagnosticsFrame::new();
        current.profile = config.profile;
        let ring = DiagnosticsRingBuffer::new(config.ring_capacity_frames);
        Self {
            state: Mutex::new(DiagnosticsState {
                config,
                current,
                ring,
                spikes: Vec::new(),
            }),
        }
    }

    pub fn disabled() -> Self {
        Self::new(DiagnosticsConfig {
            profile: DiagnosticsProfile::Off,
            ..DiagnosticsConfig::default()
        })
    }

    pub fn profile(&self) -> DiagnosticsProfile {
        self.state.lock().unwrap().config.profile
    }

    pub fn set_profile(&self, profile: DiagnosticsProfile) {
        let mut state = self.state.lock().unwrap();
        state.config.profile = profile;
        state.current.profile = profile;
    }

    pub fn begin_frame(&self, mut frame: DiagnosticsFrame) {
        let mut state = self.state.lock().unwrap();
        frame.profile = state.config.profile;
        state.current = frame;
    }

    pub fn time_scope(&self, name: &'static str) -> CpuScopeGuard<'_> {
        self.time_scope_in(DiagnosticCategory::App, name)
    }

    pub fn time_scope_in(
        &self,
        category: DiagnosticCategory,
        name: &'static str,
    ) -> CpuScopeGuard<'_> {
        let enabled = self.state.lock().unwrap().config.profile.captures_scopes();
        CpuScopeGuard {
            diagnostics: self,
            category,
            name,
            start: enabled.then(Instant::now),
        }
    }

    pub fn increment(&self, category: DiagnosticCategory, name: &'static str, value: u64) {
        let mut state = self.state.lock().unwrap();
        if state.config.profile != DiagnosticsProfile::Off {
            state.current.increment(category, name, value);
        }
    }

    pub fn gauge(&self, category: DiagnosticCategory, name: &'static str, value: f64) {
        let mut state = self.state.lock().unwrap();
        if state.config.profile != DiagnosticsProfile::Off {
            state.current.gauge(category, name, value);
        }
    }

    pub fn record_render_pass(&self, name: &'static str, elapsed_ms: f32) {
        let mut state = self.state.lock().unwrap();
        if state.config.profile != DiagnosticsProfile::Off {
            state.current.record_render_pass(name, elapsed_ms);
        }
    }

    pub fn warn(&self, category: DiagnosticCategory, message: impl Into<String>) {
        let mut state = self.state.lock().unwrap();
        if state.config.profile != DiagnosticsProfile::Off {
            state.current.warn(category, message);
        }
    }

    pub fn end_frame(&self) -> Option<SpikeReport> {
        let mut state = self.state.lock().unwrap();
        if state.config.profile == DiagnosticsProfile::Off {
            return None;
        }

        let frame = state.current.clone();
        let spike = SpikeReport::detect(&frame, state.config.spike_thresholds);
        state.ring.push(frame);
        if let Some(spike) = spike.clone() {
            state.spikes.push(spike);
        }
        spike
    }

    pub fn dump(&self) -> std::io::Result<()> {
        let state = self.state.lock().unwrap();
        let Some(latest) = state.ring.latest() else {
            return Ok(());
        };
        let summary = state.ring.rolling_summary(&state.spikes);
        write_exports(
            &state.config.export_dir,
            latest,
            &summary,
            state
                .ring
                .latest()
                .and_then(|frame| SpikeReport::detect(frame, state.config.spike_thresholds)),
            &state.ring,
        )
    }

    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        state.ring.clear();
        state.spikes.clear();
        state.current = DiagnosticsFrame::new();
        state.current.profile = state.config.profile;
    }

    pub fn spikes(&self) -> Vec<SpikeReport> {
        self.state.lock().unwrap().spikes.clone()
    }

    pub fn rolling_summary(&self) -> RollingDiagnosticsSummary {
        let state = self.state.lock().unwrap();
        state.ring.rolling_summary(&state.spikes)
    }

    pub fn latest_frame(&self) -> Option<DiagnosticsFrame> {
        self.state.lock().unwrap().ring.latest().cloned()
    }

    fn record_scope(&self, category: DiagnosticCategory, name: &'static str, elapsed_ms: f32) {
        let mut state = self.state.lock().unwrap();
        if state.config.profile.captures_scopes() {
            state.current.record_cpu_scope(category, name, elapsed_ms);
        }
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new(DiagnosticsConfig::default())
    }
}

impl Drop for CpuScopeGuard<'_> {
    fn drop(&mut self) {
        let Some(start) = self.start else {
            return;
        };
        let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
        self.diagnostics
            .record_scope(self.category, self.name, elapsed_ms);
    }
}

fn write_exports(
    root: &Path,
    latest: &DiagnosticsFrame,
    summary: &RollingDiagnosticsSummary,
    spike: Option<SpikeReport>,
    ring: &DiagnosticsRingBuffer,
) -> std::io::Result<()> {
    export_latest_frame(root, latest)?;
    export_rolling_summary(root, summary)?;
    if let Some(spike) = spike {
        append_spike_report(root, &spike)?;
    }
    export_timeline(root, ring)
}

#[cfg(test)]
mod tests {
    use super::{Diagnostics, DiagnosticsConfig};
    use crate::{DiagnosticCategory, DiagnosticsFrame, DiagnosticsProfile, FrameStatsSnapshot};

    #[test]
    fn scope_guard_records_elapsed_scope_when_profile_high() {
        let diagnostics = Diagnostics::new(DiagnosticsConfig {
            profile: DiagnosticsProfile::High,
            ..DiagnosticsConfig::default()
        });
        diagnostics.begin_frame(DiagnosticsFrame::new());
        {
            let _guard = diagnostics.time_scope_in(DiagnosticCategory::Worldgen, "worldgen.height");
        }
        diagnostics.end_frame();
        let latest = diagnostics.latest_frame().unwrap();
        assert_eq!(latest.cpu_scopes.len(), 1);
        assert_eq!(latest.cpu_scopes[0].name, "worldgen.height");
    }

    #[test]
    fn end_frame_stores_spike_report() {
        let diagnostics = Diagnostics::default();
        let mut frame = DiagnosticsFrame::new();
        frame.frame = FrameStatsSnapshot {
            frame_index: 42,
            frame_time_ms: 101.0,
            ..FrameStatsSnapshot::default()
        };
        diagnostics.begin_frame(frame);
        let spike = diagnostics.end_frame().unwrap();
        assert_eq!(spike.frame_index, 42);
        assert_eq!(diagnostics.spikes().len(), 1);
    }
}
