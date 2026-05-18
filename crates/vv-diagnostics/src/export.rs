use crate::diagnostics_frame::DiagnosticsFrame;
use crate::ring_buffer::{DiagnosticsRingBuffer, RollingDiagnosticsSummary};
use crate::spike::SpikeReport;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn export_latest_frame(root: &Path, frame: &DiagnosticsFrame) -> io::Result<()> {
    fs::create_dir_all(root)?;
    write_json(root.join("latest_frame.json"), frame)
}

pub fn export_rolling_summary(root: &Path, summary: &RollingDiagnosticsSummary) -> io::Result<()> {
    fs::create_dir_all(root)?;
    write_json(root.join("rolling_summary.json"), summary)
}

pub fn append_spike_report(root: &Path, spike: &SpikeReport) -> io::Result<()> {
    fs::create_dir_all(root)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("spikes.jsonl"))?;
    serde_json::to_writer(&mut file, spike).map_err(io::Error::other)?;
    file.write_all(b"\n")
}

pub fn export_timeline(root: &Path, ring: &DiagnosticsRingBuffer) -> io::Result<()> {
    fs::create_dir_all(root)?;
    let mut file = fs::File::create(root.join("timeline.csv"))?;
    writeln!(
        file,
        "timestamp_ms,frame_index,frame_ms,fps_1s,render_ms,update_view_ms,lod_selection_ms,upload_ms,upload_bytes,pending_jobs,pending_chunks,pending_lods,draw_calls,warnings"
    )?;
    for frame in ring.frames() {
        writeln!(
            file,
            "{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{},{},{},{},{}",
            frame.timestamp_ms,
            frame.frame.frame_index,
            frame.frame.frame_time_ms,
            frame.frame.fps_rolling_1s,
            frame.render.render_world_ms,
            frame.render.update_view_ms,
            frame.render.lod_selection_ms,
            frame.render.gpu_upload_ms,
            frame.workload.upload_bytes,
            frame.workload.pending_jobs,
            frame.workload.pending_chunks,
            frame.workload.pending_lods,
            frame.workload.draw_calls,
            frame.warnings.len()
        )?;
    }
    Ok(())
}

pub struct DiagnosticsFileSink {
    root: PathBuf,
    timeline: File,
}

impl DiagnosticsFileSink {
    pub fn new(root: impl Into<PathBuf>) -> io::Result<Self> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        let timeline_path = root.join("timeline.csv");
        let timeline_exists = timeline_path.exists();
        let mut timeline = OpenOptions::new()
            .create(true)
            .append(true)
            .open(timeline_path)?;
        if !timeline_exists || timeline.metadata()?.len() == 0 {
            writeln!(
                timeline,
                "timestamp_ms,frame_index,frame_ms,fps_1s,fps_5s,render_ms,terrain_draw_ms,update_view_ms,lod_selection_ms,upload_ms,upload_bytes,active_chunks,active_lods,pending_jobs,pending_chunks,pending_lods,draw_calls,gpu_memory_bytes,worldgen_samples,worldgen_cache_hit_ratio,audio_voices_started,audio_voices_throttled,warnings"
            )?;
        }
        Ok(Self { root, timeline })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn write_frame(
        &mut self,
        frame: &DiagnosticsFrame,
        summary: &RollingDiagnosticsSummary,
        spike: Option<&SpikeReport>,
    ) -> io::Result<()> {
        export_latest_frame(&self.root, frame)?;
        export_rolling_summary(&self.root, summary)?;
        if let Some(spike) = spike {
            append_spike_report(&self.root, spike)?;
        }
        self.append_timeline_row(frame)
    }

    fn append_timeline_row(&mut self, frame: &DiagnosticsFrame) -> io::Result<()> {
        writeln!(
            self.timeline,
            "{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{},{},{},{},{},{},{},{},{:.5},{},{},{}",
            frame.timestamp_ms,
            frame.frame.frame_index,
            frame.frame.frame_time_ms,
            frame.frame.fps_rolling_1s,
            frame.frame.fps_rolling_5s,
            frame.render.render_world_ms,
            frame.render.terrain_draw_ms,
            frame.render.update_view_ms,
            frame.render.lod_selection_ms,
            frame.render.gpu_upload_ms,
            frame.render.gpu_upload_bytes,
            frame.render.active_chunks,
            frame.render.active_lods,
            frame.workload.pending_jobs,
            frame.workload.pending_chunks,
            frame.workload.pending_lods,
            frame.workload.draw_calls,
            frame.workload.gpu_memory_bytes,
            frame.workload.worldgen_samples,
            frame.worldgen.cell_hit_ratio(),
            frame.audio.voices_started,
            frame.audio.voices_throttled,
            frame.warnings.len()
        )?;
        self.timeline.flush()
    }
}

fn write_json(path: impl AsRef<Path>, value: &impl serde::Serialize) -> io::Result<()> {
    let file = fs::File::create(path)?;
    serde_json::to_writer_pretty(file, value).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::{export_latest_frame, export_timeline};
    use crate::{DiagnosticsFrame, DiagnosticsRingBuffer, FrameStatsSnapshot};
    use std::fs;

    #[test]
    fn exports_latest_frame_json_and_timeline_csv() {
        let root = std::env::temp_dir().join(format!(
            "vv_diag_export_{}_{}",
            std::process::id(),
            unique_suffix()
        ));
        let mut frame = DiagnosticsFrame::new();
        frame.frame = FrameStatsSnapshot {
            frame_index: 3,
            frame_time_ms: 16.5,
            ..FrameStatsSnapshot::default()
        };

        export_latest_frame(&root, &frame).unwrap();
        let json = fs::read_to_string(root.join("latest_frame.json")).unwrap();
        assert!(json.contains("\"frame_index\": 3"));

        let mut ring = DiagnosticsRingBuffer::new(4);
        ring.push(frame);
        export_timeline(&root, &ring).unwrap();
        let csv = fs::read_to_string(root.join("timeline.csv")).unwrap();
        assert!(csv.contains("frame_ms"));
        assert!(csv.contains("16.500"));

        fs::remove_dir_all(root).unwrap();
    }

    fn unique_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
}
