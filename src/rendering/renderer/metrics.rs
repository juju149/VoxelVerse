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
        }
    }
}
