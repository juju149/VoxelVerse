use crate::diagnostics_frame::DiagnosticsFrame;
use crate::metric::{CpuScopeTiming, DiagnosticWarning, PlayerCameraSnapshot, WorkloadSnapshot};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct SpikeThresholds {
    pub frame_warn_ms: f32,
    pub frame_bad_ms: f32,
    pub frame_freeze_ms: f32,
    pub upload_bytes_budget: u64,
    pub pending_jobs_budget: u32,
    pub pending_chunks_budget: u32,
    pub pending_lods_budget: u32,
    pub worldgen_scope_budget_ms: f32,
    pub meshing_scope_budget_ms: f32,
    pub render_pass_budget_ms: f32,
}

impl Default for SpikeThresholds {
    fn default() -> Self {
        Self {
            frame_warn_ms: 33.0,
            frame_bad_ms: 50.0,
            frame_freeze_ms: 100.0,
            upload_bytes_budget: 16 * 1024 * 1024,
            pending_jobs_budget: 64,
            pending_chunks_budget: 48,
            pending_lods_budget: 32,
            worldgen_scope_budget_ms: 4.0,
            meshing_scope_budget_ms: 6.0,
            render_pass_budget_ms: 4.0,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SpikeReport {
    pub timestamp_ms: u128,
    pub frame_index: u64,
    pub frame_time_ms: f32,
    pub reasons: Vec<String>,
    pub top_cpu_scopes: Vec<CpuScopeTiming>,
    pub render_pass_timings: Vec<(String, f32)>,
    pub camera: PlayerCameraSnapshot,
    pub workload: WorkloadSnapshot,
    pub warnings: Vec<DiagnosticWarning>,
}

impl SpikeReport {
    pub fn detect(frame: &DiagnosticsFrame, thresholds: SpikeThresholds) -> Option<Self> {
        let mut reasons = Vec::new();
        let frame_ms = frame.frame.frame_time_ms;
        if frame_ms >= thresholds.frame_freeze_ms {
            reasons.push(format!("frame >= {:.0} ms", thresholds.frame_freeze_ms));
        } else if frame_ms >= thresholds.frame_bad_ms {
            reasons.push(format!("frame >= {:.0} ms", thresholds.frame_bad_ms));
        } else if frame_ms >= thresholds.frame_warn_ms {
            reasons.push(format!("frame >= {:.0} ms", thresholds.frame_warn_ms));
        }

        let workload = frame.workload;
        if workload.upload_bytes > thresholds.upload_bytes_budget {
            reasons.push(format!(
                "upload bytes {} > budget {}",
                workload.upload_bytes, thresholds.upload_bytes_budget
            ));
        }
        if workload.pending_jobs > thresholds.pending_jobs_budget {
            reasons.push(format!(
                "pending jobs {} > budget {}",
                workload.pending_jobs, thresholds.pending_jobs_budget
            ));
        }
        if workload.pending_chunks > thresholds.pending_chunks_budget {
            reasons.push(format!(
                "pending chunks {} > budget {}",
                workload.pending_chunks, thresholds.pending_chunks_budget
            ));
        }
        if workload.pending_lods > thresholds.pending_lods_budget {
            reasons.push(format!(
                "pending LODs {} > budget {}",
                workload.pending_lods, thresholds.pending_lods_budget
            ));
        }

        for scope in &frame.cpu_scopes {
            let budget = if scope.name.starts_with("worldgen.") {
                thresholds.worldgen_scope_budget_ms
            } else if scope.name.starts_with("meshing.") {
                thresholds.meshing_scope_budget_ms
            } else {
                f32::INFINITY
            };
            if scope.elapsed_ms > budget {
                reasons.push(format!(
                    "{} {:.2} ms > {:.2} ms",
                    scope.name, scope.elapsed_ms, budget
                ));
            }
        }

        for pass in &frame.render_pass_timings {
            if pass.elapsed_ms > thresholds.render_pass_budget_ms {
                reasons.push(format!(
                    "render pass {} {:.2} ms > {:.2} ms",
                    pass.name, pass.elapsed_ms, thresholds.render_pass_budget_ms
                ));
            }
        }

        if reasons.is_empty() {
            return None;
        }

        let mut top_cpu_scopes = frame.cpu_scopes.clone();
        top_cpu_scopes.sort_by(|a, b| b.elapsed_ms.total_cmp(&a.elapsed_ms));
        top_cpu_scopes.truncate(8);

        Some(Self {
            timestamp_ms: frame.timestamp_ms,
            frame_index: frame.frame.frame_index,
            frame_time_ms: frame_ms,
            reasons,
            top_cpu_scopes,
            render_pass_timings: frame
                .render_pass_timings
                .iter()
                .map(|pass| (pass.name.to_string(), pass.elapsed_ms))
                .collect(),
            camera: frame.camera,
            workload,
            warnings: frame.warnings.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{SpikeReport, SpikeThresholds};
    use crate::{DiagnosticsFrame, FrameStatsSnapshot};

    #[test]
    fn detects_frame_time_spike() {
        let mut frame = DiagnosticsFrame::new();
        frame.frame = FrameStatsSnapshot {
            frame_index: 7,
            frame_time_ms: 51.0,
            ..FrameStatsSnapshot::default()
        };
        let report = SpikeReport::detect(&frame, SpikeThresholds::default()).unwrap();
        assert_eq!(report.frame_index, 7);
        assert!(report.reasons.iter().any(|reason| reason.contains("50")));
    }
}
