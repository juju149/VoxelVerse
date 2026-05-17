//! Per-subsystem snapshot types.
//!
//! Each subsystem owns its live counters (often `AtomicU64` so workers can
//! increment without locking), but the *snapshot* shape lives here so the
//! central `DiagnosticsFrame` aggregator can be authored without circular
//! dependencies. Producers (`vv-worldgen`, `vv-audio`, …) depend on
//! `vv-diagnostics`; consumers (the dev overlay, `voxelverse` app) only need
//! to depend on `vv-diagnostics` to read them all.
//!
//! Fields are public and intentionally permissive: this is a transport-layer
//! struct, not an invariant-bearing one.

/// Worldgen telemetry snapshot. Produced by `vv-worldgen::WorldgenStats`.
#[derive(Clone, Copy, Debug, Default)]
pub struct WorldgenStatsSnapshot {
    pub cell_hits: u64,
    pub cell_misses: u64,
    pub features_emitted: u64,
    pub props_emitted: u64,
    pub candidates_rejected: u64,
    pub candidates_rejected_spacing: u64,
}

impl WorldgenStatsSnapshot {
    /// Cache hit fraction in `[0, 1]`. Returns `0` for an empty snapshot.
    pub fn cell_hit_ratio(self) -> f32 {
        let total = self.cell_hits + self.cell_misses;
        if total == 0 {
            return 0.0;
        }
        self.cell_hits as f32 / total as f32
    }
}

/// Audio engine telemetry snapshot. Produced by `vv-audio::AudioDiagnostics`.
#[derive(Clone, Debug, Default)]
pub struct AudioStatsSnapshot {
    pub voices_started: u64,
    pub voices_throttled: u64,
    pub file_open_errors: u64,
    pub decode_errors: u64,
    pub play_errors: u64,
    pub output_unavailable_drops: u64,
    pub last_error: Option<String>,
}

/// Chunk streaming telemetry snapshot. Produced by the renderer's chunk
/// pipeline (mesh jobs, GPU uploads). Populated incrementally; fields not
/// yet wired stay at their `Default` value.
#[derive(Clone, Copy, Debug, Default)]
pub struct StreamingStatsSnapshot {
    pub pending_mesh_jobs: u32,
    pub pending_lod_jobs: u32,
    pub uploads_this_frame: u32,
    pub upload_bytes_this_frame: u64,
    pub queued_chunks: u32,
    pub visible_chunks: u32,
}

/// Gameplay-side telemetry snapshot. Captures player-frame state the rest
/// of the engine can't see (current biome, current chunk, etc.). Populated
/// incrementally; empty fields stay at `Default`.
#[derive(Clone, Debug, Default)]
pub struct GameplayStatsSnapshot {
    pub player_pos: [f32; 3],
    pub player_chunk: [i32; 3],
    pub current_biome: Option<String>,
    pub target_voxel_key: Option<String>,
}
