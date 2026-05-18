use crate::types::ChunkMesh;
use std::collections::HashMap;
use std::time::Instant;
use vv_voxel::{LodKey, SurfaceChunkKey};

/// Hard cap on how many retired meshes may linger in the fade-out queue.
/// Each dying entry holds live GPU buffers; without a ceiling the set grows
/// without bound during fast orbital flight where many tiles retire every frame.
const MAX_DYING_CHUNKS: usize = 48;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum AnyKey {
    Voxel(SurfaceChunkKey),
    Lod(LodKey),
}

pub struct FadeState {
    pub mesh: ChunkMesh,
    pub start_time: Instant,
    pub start_alpha: f32,
    pub target_alpha: f32,
    pub duration: f32,
}

pub struct LodAnimator {
    pub dying_chunks: HashMap<AnyKey, FadeState>,
    pub spawning_chunks: HashMap<AnyKey, Instant>,
    fade_duration: f32,
}

impl LodAnimator {
    pub fn new() -> Self {
        Self {
            dying_chunks: HashMap::new(),
            spawning_chunks: HashMap::new(),
            fade_duration: 2.0,
        }
    }

    pub fn set_fade_duration(&mut self, duration: f32) {
        self.fade_duration = duration.max(0.05);
    }

    // smoothstep Interpolation (t * t * (3 - 2t))
    // creates a sigmoid curve: slow start -> fast middle -> slow end
    fn smoothstep(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    pub fn start_spawn(&mut self, key: AnyKey) {
        if self.dying_chunks.remove(&key).is_some() {
            // if reviving, we just reset.
        }
        self.spawning_chunks.insert(key, Instant::now());
    }

    pub fn retire(&mut self, key: AnyKey, mesh: ChunkMesh) {
        // When at capacity, evict the dying chunk that is already closest to
        // fully transparent.  It is nearly invisible and holds the least visual
        // value while still occupying GPU buffers.
        if self.dying_chunks.len() >= MAX_DYING_CHUNKS {
            let now = Instant::now();
            let to_drop = self
                .dying_chunks
                .iter()
                .max_by(|a, b| {
                    let ta = (now - a.1.start_time).as_secs_f32() / a.1.duration.max(0.001);
                    let tb = (now - b.1.start_time).as_secs_f32() / b.1.duration.max(0.001);
                    ta.partial_cmp(&tb).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(k, _)| *k);
            if let Some(k) = to_drop {
                self.dying_chunks.remove(&k);
            }
        }
        self.dying_chunks.insert(
            key,
            FadeState {
                mesh,
                start_time: Instant::now(),
                start_alpha: 1.0,
                target_alpha: 0.0,
                duration: self.fade_duration,
            },
        );
        self.spawning_chunks.remove(&key);
    }

    pub fn get_opacity(&self, key: AnyKey, now: Instant) -> f32 {
        if let Some(start) = self.spawning_chunks.get(&key) {
            let elapsed = (now - *start).as_secs_f32();
            let linear_t = elapsed / self.fade_duration;
            return Self::smoothstep(linear_t);
        }
        1.0
    }

    pub fn update_dying(&mut self, now: Instant) -> Vec<(AnyKey, f32)> {
        let mut results = Vec::new();
        let mut to_remove = Vec::new();

        for (key, state) in &self.dying_chunks {
            let elapsed = (now - state.start_time).as_secs_f32();
            let linear_t = elapsed / state.duration;

            if linear_t >= 1.0 {
                to_remove.push(*key);
            } else {
                let t = Self::smoothstep(linear_t);
                let alpha = state.start_alpha + (state.target_alpha - state.start_alpha) * t;
                results.push((*key, alpha));
            }
        }

        for k in to_remove {
            self.dying_chunks.remove(&k);
        }
        results
    }
}
