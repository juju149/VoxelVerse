use super::Renderer;
use crate::diagnostics::RenderStats;

impl<'a> Renderer<'a> {
    pub(super) fn render_stats(&self, visible_chunks: usize, visible_lods: usize) -> RenderStats {
        let mut gpu_vertices = 0;
        let mut gpu_indices = 0;

        for mesh in self.chunks.values().chain(self.lod_chunks.values()) {
            gpu_vertices += mesh.num_verts;
            gpu_indices += mesh.num_inds as usize;
        }

        let meshing_avg_ms = if self.completed_mesh_count == 0 {
            0.0
        } else {
            self.completed_mesh_time_sum_ms / self.completed_mesh_count as f32
        };

        RenderStats {
            visible_chunks,
            active_chunks: self.chunks.len(),
            visible_lods,
            active_lods: self.lod_chunks.len(),
            queued_chunks: self.load_queue.len(),
            pending_chunks: self.pending_chunks.len(),
            pending_lods: self.pending_lods.len(),
            gpu_vertices,
            gpu_indices,
            // job/timing fields are zero-filled here; the caller may override them
            mesh_jobs_in_flight: self.pending_chunks.len(),
            lod_jobs_in_flight: self.pending_lods.len(),
            uploads_this_frame: self.scheduler_stats.uploaded_voxel
                + self.scheduler_stats.uploaded_lod,
            update_view_ms: self.update_view_ms,
            meshing_avg_ms,
            meshing_max_ms: self.completed_mesh_time_max_ms,
            render_world_ms: self.last_render_ms,
            render_ui_ms: 0.0,
        }
    }
}
