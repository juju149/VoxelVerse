use crate::diagnostics_frame::DiagnosticsFrame;
use crate::ring_buffer::{DiagnosticsRingBuffer, RollingDiagnosticsSummary};
use crate::spike::SpikeReport;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

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
