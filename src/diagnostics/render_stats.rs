const VERTEX_BYTES: usize = 36;
const INDEX_BYTES: usize = 4;

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderStats {
    pub visible_chunks: usize,
    pub active_chunks: usize,
    pub visible_lods: usize,
    pub active_lods: usize,
    pub queued_chunks: usize,
    pub pending_chunks: usize,
    pub pending_lods: usize,
    pub gpu_vertices: usize,
    pub gpu_indices: usize,
}

impl RenderStats {
    pub fn estimated_gpu_bytes(self) -> usize {
        self.gpu_vertices * VERTEX_BYTES + self.gpu_indices * INDEX_BYTES
    }

    pub fn estimated_gpu_mib(self) -> f32 {
        self.estimated_gpu_bytes() as f32 / (1024.0 * 1024.0)
    }

    pub fn gpu_memory_label(self) -> String {
        let mib = self.estimated_gpu_mib();
        if mib > 1024.0 {
            format!("{:.2} GB", mib / 1024.0)
        } else {
            format!("{:.2} MB", mib)
        }
    }

    pub fn debug_overlay(self, culling_status: &str, frame_time_ms: f32) -> String {
        format!(
            "Culling: {}\nFrame:   {:.2} ms\nChunks:  {} / {}\nLODs:    {} / {}\nQueue:   {}\nPending: {} chunks / {} LODs\nGPU:     {}",
            culling_status,
            frame_time_ms,
            self.visible_chunks,
            self.active_chunks,
            self.visible_lods,
            self.active_lods,
            self.queued_chunks,
            self.pending_chunks,
            self.pending_lods,
            self.gpu_memory_label()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::RenderStats;

    #[test]
    fn gpu_memory_estimate_uses_vertex_and_index_counts() {
        let stats = RenderStats {
            gpu_vertices: 10,
            gpu_indices: 6,
            ..RenderStats::default()
        };

        assert_eq!(stats.estimated_gpu_bytes(), 10 * 36 + 6 * 4);
    }
}
