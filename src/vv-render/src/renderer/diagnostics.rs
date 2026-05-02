use std::time::Duration;

use vv_diagnostics::{emit, LogDomain, LogLevel, RuntimeSnapshot, WorldCounters, WorldgenCounters};
use vv_gameplay::PlayerGameplayState;
use vv_mesh::Vertex;
use vv_world_runtime::PlanetData;

use super::Renderer;

impl<'a> Renderer<'a> {
    pub fn log_memory(&self, planet: &PlanetData) {
        let (tv, ti) = self.chunks.values().fold((0usize, 0usize), |(v, i), c| {
            (v + c.num_verts, i + c.num_inds as usize)
        });
        let mb = ((tv * std::mem::size_of::<Vertex>()) + (ti * 4)) as f32 / (1024.0 * 1024.0);
        emit(
            self.diagnostic_config,
            LogLevel::Info,
            LogDomain::Memory,
            format!(
                "resolution={} chunks={} mesh_cpu={:.2}MB",
                planet.resolution,
                self.chunks.len(),
                mb
            ),
        );
    }

    pub fn render_prep_time(&self) -> Duration {
        self.frame_telemetry.render_prep_time
    }

    pub fn lod_coverage_time(&self) -> Duration {
        self.frame_telemetry.lod_coverage_time
    }

    pub fn chunk_streaming_time(&self) -> Duration {
        self.frame_telemetry.chunk_streaming_time
    }

    pub fn diagnostic_snapshot(
        &self,
        planet: &PlanetData,
        gameplay: &PlayerGameplayState,
    ) -> RuntimeSnapshot {
        let planet_stats = planet.runtime_stats();
        let terrain_stats = planet.terrain.cache_stats();
        let mut snapshot = RuntimeSnapshot {
            world: WorldCounters {
                edited_chunks: planet_stats.edited_chunks,
                mined_blocks: planet_stats.mined_blocks,
                placed_blocks: planet_stats.placed_blocks,
                dirty_chunks: planet_stats.dirty_chunks,
            },
            worldgen: WorldgenCounters {
                cached_columns: terrain_stats.cached_columns,
                cache_hits: terrain_stats.cache_hits,
                cache_misses: terrain_stats.cache_misses,
                compute_time: Duration::from_micros(terrain_stats.compute_micros),
            },
            streaming: self.frame_telemetry.streaming.clone(),
            lod: self.frame_telemetry.lod.clone(),
            mesh: self.frame_telemetry.mesh.clone(),
            gpu: self.frame_telemetry.gpu.clone(),
            dropped_items: gameplay.dropped_items.len(),
            inventory_open: gameplay.inventory_open,
        };

        snapshot.streaming.active_chunks = self.chunks.len();
        snapshot.streaming.load_queue = self.load_queue.len();
        snapshot.streaming.pending_chunk_jobs = self.pending_chunks.len();
        snapshot.lod.active_lods = self.lod_chunks.len();
        snapshot.lod.pending_lod_jobs = self.pending_lods.len();

        let mesh_totals = self.mesh_totals();
        snapshot.mesh.active_meshes = self.chunks.len() + self.lod_chunks.len();
        snapshot.mesh.vertices = mesh_totals.0;
        snapshot.mesh.indices = mesh_totals.1;
        snapshot.mesh.mesh_cpu_mb = mesh_totals.2;
        snapshot.gpu.active_buffers =
            (self.chunks.len() + self.lod_chunks.len() + self.animator.dying_chunks.len()) * 3;
        snapshot
    }

    fn mesh_totals(&self) -> (usize, usize, f32) {
        let mut vertices = 0usize;
        let mut indices = 0usize;
        for mesh in self
            .chunks
            .values()
            .chain(self.lod_chunks.values())
            .chain(self.animator.dying_chunks.values().map(|state| &state.mesh))
        {
            vertices += mesh.num_verts;
            indices += mesh.num_inds as usize;
        }
        let mb =
            ((vertices * std::mem::size_of::<Vertex>()) + (indices * 4)) as f32 / (1024.0 * 1024.0);
        (vertices, indices, mb)
    }
}
