use super::Renderer;
use crate::atmosphere::{AtmosphereConfig, PlanetAtmospherePreset};
use vv_diagnostics::RenderStats;

impl<'a> Renderer<'a> {
    pub fn toggle_engine_debug_page(&mut self) {
        self.engine_debug_page = !self.engine_debug_page;
        println!(
            "[engine] debug page = {}",
            if self.engine_debug_page { "on" } else { "off" }
        );
    }

    pub fn set_engine_debug_page(&mut self, enabled: bool) {
        self.engine_debug_page = enabled;
    }

    pub fn set_atmosphere_preset(&mut self, preset: PlanetAtmospherePreset) {
        self.atmosphere = AtmosphereConfig::preset(preset);
    }

    pub fn has_active_scene_chunks(&self) -> bool {
        !self.chunks.is_empty() || !self.lod_chunks.is_empty()
    }

    pub fn log_engine_snapshot(&self, label: &str, planet: &vv_world::PlanetData) {
        let stats = self.render_stats(0, 0);
        println!(
            "[engine/{label}] profile={:?} planet_resolution={} chunks={} lods={} pending={}/{} uploads={} draw_calls={} frame_ms={:.2} gpu_est={}",
            self.quality.profile,
            planet.resolution,
            stats.active_chunks,
            stats.active_lods,
            stats.pending_chunks,
            stats.pending_lods,
            stats.uploads_this_frame,
            stats.draw_calls + stats.shadow_draw_calls,
            self.frame_stats.frame_time_ms(),
            stats.gpu_memory_label(),
        );
    }

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
        let voxel_meshing_avg_ms = if self.completed_voxel_mesh_count == 0 {
            0.0
        } else {
            self.completed_voxel_mesh_time_sum_ms / self.completed_voxel_mesh_count as f32
        };
        let lod_meshing_avg_ms = if self.completed_lod_mesh_count == 0 {
            0.0
        } else {
            self.completed_lod_mesh_time_sum_ms / self.completed_lod_mesh_count as f32
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
            draw_calls: self.last_draw_calls,
            shadow_draw_calls: self.last_shadow_draw_calls,
            // job/timing fields are zero-filled here; the caller may override them
            mesh_jobs_in_flight: self.pending_chunks.len(),
            lod_jobs_in_flight: self.pending_lods.len(),
            uploads_this_frame: self.scheduler_stats.uploaded_voxel
                + self.scheduler_stats.uploaded_lod,
            update_view_ms: self.update_view_ms,
            lod_selection_ms: self.lod_selection_ms,
            meshing_avg_ms,
            meshing_max_ms: self.completed_mesh_time_max_ms,
            voxel_meshing_avg_ms,
            voxel_meshing_max_ms: self.completed_voxel_mesh_time_max_ms,
            lod_meshing_avg_ms,
            lod_meshing_max_ms: self.completed_lod_mesh_time_max_ms,
            gpu_upload_ms: self.gpu_upload_ms,
            terrain_draw_ms: self.last_terrain_draw_ms,
            render_world_ms: self.last_render_ms,
            render_ui_ms: 0.0,
        }
    }
}
