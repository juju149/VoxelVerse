use vv_diagnostics::{FrameStats, FrameStatsSnapshot};

#[derive(Default)]
pub(super) struct MeshTiming {
    pub(super) sum_ms: f32,
    pub(super) max_ms: f32,
    pub(super) count: usize,
}

impl MeshTiming {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn record(&mut self, elapsed_ms: f32) {
        self.sum_ms += elapsed_ms;
        self.max_ms = self.max_ms.max(elapsed_ms);
        self.count += 1;
    }

    pub(super) fn average_ms(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_ms / self.count as f32
        }
    }
}

pub(super) struct FrameMetrics {
    pub(super) all_mesh: MeshTiming,
    pub(super) voxel_mesh: MeshTiming,
    pub(super) lod_mesh: MeshTiming,
    pub(super) update_view_ms: f32,
    pub(super) lod_selection_ms: f32,
    pub(super) gpu_upload_ms: f32,
    pub(super) gpu_upload_bytes: u64,
    pub(super) terrain_draw_ms: f32,
    pub(super) render_ms: f32,
    pub(super) draw_calls: usize,
    pub(super) shadow_draw_calls: usize,
    pub(super) visible_chunks: usize,
    pub(super) visible_lods: usize,
    pub(super) frame_stats: FrameStats,
}

impl FrameMetrics {
    pub(super) fn new() -> Self {
        Self {
            all_mesh: MeshTiming::default(),
            voxel_mesh: MeshTiming::default(),
            lod_mesh: MeshTiming::default(),
            update_view_ms: 0.0,
            lod_selection_ms: 0.0,
            gpu_upload_ms: 0.0,
            gpu_upload_bytes: 0,
            terrain_draw_ms: 0.0,
            render_ms: 0.0,
            draw_calls: 0,
            shadow_draw_calls: 0,
            visible_chunks: 0,
            visible_lods: 0,
            frame_stats: FrameStats::new(),
        }
    }

    pub(super) fn reset_streaming(&mut self) {
        self.all_mesh.reset();
        self.voxel_mesh.reset();
        self.lod_mesh.reset();
        self.update_view_ms = 0.0;
        self.lod_selection_ms = 0.0;
        self.gpu_upload_ms = 0.0;
        self.gpu_upload_bytes = 0;
    }

    pub(super) fn record_voxel_mesh_time(&mut self, elapsed_ms: f32) {
        self.all_mesh.record(elapsed_ms);
        self.voxel_mesh.record(elapsed_ms);
    }

    pub(super) fn record_lod_mesh_time(&mut self, elapsed_ms: f32) {
        self.all_mesh.record(elapsed_ms);
        self.lod_mesh.record(elapsed_ms);
    }

    pub(super) fn frame_snapshot(&self) -> FrameStatsSnapshot {
        let mut snapshot = FrameStatsSnapshot::from(&self.frame_stats);
        snapshot.render_ms = self.render_ms;
        snapshot
    }
}
