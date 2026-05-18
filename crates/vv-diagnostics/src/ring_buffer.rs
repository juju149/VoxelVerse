use crate::diagnostics_frame::DiagnosticsFrame;
use crate::spike::SpikeReport;
use crate::stats::{RollingWindowStats, RunningStats};
use serde::Serialize;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct DiagnosticsRingBuffer {
    capacity: usize,
    frames: VecDeque<DiagnosticsFrame>,
}

impl DiagnosticsRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            frames: VecDeque::with_capacity(capacity.max(1)),
        }
    }

    pub fn push(&mut self, frame: DiagnosticsFrame) {
        if self.frames.len() == self.capacity {
            self.frames.pop_front();
        }
        self.frames.push_back(frame);
    }

    pub fn latest(&self) -> Option<&DiagnosticsFrame> {
        self.frames.back()
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }

    pub fn rolling_summary(&self, spikes: &[SpikeReport]) -> RollingDiagnosticsSummary {
        RollingDiagnosticsSummary {
            frame_time_ms: self.rolling_frame_stats(),
            fps_1s: self.average_fps_for_window(1_000),
            fps_5s: self.average_fps_for_window(5_000),
            recent_frame_max_ms: self
                .frames
                .iter()
                .map(|frame| frame.frame.frame_time_ms)
                .fold(0.0, f32::max),
            spike_count: spikes.len(),
            latest_frame_index: self.latest().map(|frame| frame.frame.frame_index),
        }
    }

    pub fn frames(&self) -> impl Iterator<Item = &DiagnosticsFrame> {
        self.frames.iter()
    }

    fn rolling_frame_stats(&self) -> RollingWindowStats {
        RollingWindowStats {
            one_second: self.stats_for_window(1_000),
            five_seconds: self.stats_for_window(5_000),
            thirty_seconds: self.stats_for_window(30_000),
        }
    }

    fn stats_for_window(&self, window_ms: u128) -> RunningStats {
        let Some(latest) = self.latest() else {
            return RunningStats::default();
        };
        let min_timestamp = latest.timestamp_ms.saturating_sub(window_ms);
        let samples: Vec<f32> = self
            .frames
            .iter()
            .filter(|frame| frame.timestamp_ms >= min_timestamp)
            .map(|frame| frame.frame.frame_time_ms)
            .collect();
        RunningStats::from_samples(&samples)
    }

    fn average_fps_for_window(&self, window_ms: u128) -> f32 {
        let stats = self.stats_for_window(window_ms);
        if stats.avg <= f32::EPSILON {
            0.0
        } else {
            1000.0 / stats.avg
        }
    }
}

impl Default for DiagnosticsRingBuffer {
    fn default() -> Self {
        Self::new(1800)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct RollingDiagnosticsSummary {
    pub frame_time_ms: RollingWindowStats,
    pub fps_1s: f32,
    pub fps_5s: f32,
    pub recent_frame_max_ms: f32,
    pub spike_count: usize,
    pub latest_frame_index: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::DiagnosticsRingBuffer;
    use crate::{DiagnosticsFrame, FrameStatsSnapshot};

    #[test]
    fn ring_buffer_keeps_capacity() {
        let mut ring = DiagnosticsRingBuffer::new(2);
        for index in 0..3 {
            let mut frame = DiagnosticsFrame::new();
            frame.frame = FrameStatsSnapshot {
                frame_index: index,
                frame_time_ms: 16.0,
                ..FrameStatsSnapshot::default()
            };
            frame.timestamp_ms = index as u128;
            ring.push(frame);
        }
        assert_eq!(ring.len(), 2);
        assert_eq!(ring.latest().unwrap().frame.frame_index, 2);
    }

    #[test]
    fn rolling_summary_uses_recent_samples() {
        let mut ring = DiagnosticsRingBuffer::new(8);
        for index in 0..4 {
            let mut frame = DiagnosticsFrame::new();
            frame.timestamp_ms = index * 100;
            frame.frame.frame_time_ms = 10.0 + index as f32;
            ring.push(frame);
        }
        let summary = ring.rolling_summary(&[]);
        assert_eq!(summary.frame_time_ms.one_second.count, 4);
        assert!(summary.fps_1s > 0.0);
    }
}
