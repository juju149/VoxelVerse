use serde::Serialize;

/// Must match `std::mem::size_of::<rendering::types::Vertex>()`.
/// A compile-time assertion in `rendering/types.rs` guarantees this stays in sync.
const VERTEX_BYTES: usize = 48;
const INDEX_BYTES: usize = 4;

/// Per-frame counters collected by the renderer and exposed to the diagnostics overlay.
/// All fields represent the state at the end of the most recent frame.
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct RenderStats {
    // --- geometry counters ---
    pub visible_chunks: usize,
    pub active_chunks: usize,
    pub visible_lods: usize,
    pub active_lods: usize,
    pub queued_chunks: usize,
    pub pending_chunks: usize,
    pub pending_lods: usize,
    pub gpu_vertices: usize,
    pub gpu_indices: usize,
    pub draw_calls: usize,
    pub shadow_draw_calls: usize,

    // --- job counters (updated each frame) ---
    pub mesh_jobs_in_flight: usize,
    pub lod_jobs_in_flight: usize,
    pub uploads_this_frame: usize,

    // --- timing (milliseconds) ---
    pub update_view_ms: f32,
    pub lod_selection_ms: f32,
    pub meshing_avg_ms: f32,
    pub meshing_max_ms: f32,
    pub voxel_meshing_avg_ms: f32,
    pub voxel_meshing_max_ms: f32,
    pub lod_meshing_avg_ms: f32,
    pub lod_meshing_max_ms: f32,
    pub gpu_upload_ms: f32,
    pub terrain_draw_ms: f32,
    pub render_world_ms: f32,
    pub render_ui_ms: f32,
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

    /// Single-string overlay suitable for the on-screen debug console.
    pub fn debug_overlay(
        self,
        culling_status: &str,
        frame_time_ms: f32,
        player_pos: [f32; 3],
        target_voxel: Option<String>,
    ) -> String {
        let target = target_voxel.unwrap_or_else(|| "none".to_string());
        format!(
            "Culling:  {culling}\nFrame:    {frame:.2} ms\nView:     {view:.2} ms\nLOD sel:  {lod_sel:.2} ms\nTerrain:  draw {terrain:.2} ms / upload {upload_ms:.2} ms\nRender:   {render:.2} ms\nPlayer:   {px:.1}, {py:.1}, {pz:.1}\nTarget:   {target}\nChunks:   {vc} / {ac}\nLODs:     {vl} / {al}\nDraws:    {draws} main / {shadow} shadow\nQueue:    {qc}\nPending:  {pc} chunks / {pl} LODs\nJobs:     {mj} mesh / {lj} LOD\nUploads:  {up} this frame\nMesh CPU: voxel {vma:.2}/{vmx:.2} ms  lod {lma:.2}/{lmx:.2} ms\nGPU:      {gpu}",
            culling = culling_status,
            frame = frame_time_ms,
            view = self.update_view_ms,
            lod_sel = self.lod_selection_ms,
            terrain = self.terrain_draw_ms,
            upload_ms = self.gpu_upload_ms,
            render = self.render_world_ms,
            px = player_pos[0],
            py = player_pos[1],
            pz = player_pos[2],
            target = target,
            vc = self.visible_chunks,
            ac = self.active_chunks,
            vl = self.visible_lods,
            al = self.active_lods,
            draws = self.draw_calls,
            shadow = self.shadow_draw_calls,
            qc = self.queued_chunks,
            pc = self.pending_chunks,
            pl = self.pending_lods,
            mj = self.mesh_jobs_in_flight,
            lj = self.lod_jobs_in_flight,
            up = self.uploads_this_frame,
            vma = self.voxel_meshing_avg_ms,
            vmx = self.voxel_meshing_max_ms,
            lma = self.lod_meshing_avg_ms,
            lmx = self.lod_meshing_max_ms,
            gpu = self.gpu_memory_label(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderStats, VERTEX_BYTES};

    #[test]
    fn vertex_bytes_constant_is_48() {
        assert_eq!(VERTEX_BYTES, 48, "VERTEX_BYTES must match Vertex layout");
    }

    #[test]
    fn gpu_memory_estimate_uses_vertex_and_index_counts() {
        let stats = RenderStats {
            gpu_vertices: 10,
            gpu_indices: 6,
            ..RenderStats::default()
        };
        assert_eq!(stats.estimated_gpu_bytes(), 10 * 48 + 6 * 4);
    }

    #[test]
    fn gpu_memory_label_megabytes() {
        let bytes_per_mib = 1024 * 1024;
        let v = bytes_per_mib / VERTEX_BYTES;
        let stats = RenderStats {
            gpu_vertices: v,
            gpu_indices: 0,
            ..Default::default()
        };
        assert!(stats.gpu_memory_label().ends_with("MB"));
    }
}
