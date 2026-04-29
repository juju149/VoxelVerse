use crate::ChunkMesh;
use std::collections::HashMap;
use std::time::Instant;
use vv_core::{ChunkKey, LodKey};

/// Discriminated key used by `LodAnimator` to track either a voxel chunk
/// or an LOD tile.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum AnyKey {
    Voxel(ChunkKey),
    Lod(LodKey),
}

/// Fade animation state for a retiring mesh.
pub struct FadeState {
    pub mesh: ChunkMesh,
    pub start_time: Instant,
    pub start_alpha: f32,
    pub target_alpha: f32,
    pub duration: f32,
}

/// Manages cross-fade transitions when LOD or voxel chunks are swapped.
pub struct LodAnimator {
    pub dying_chunks: HashMap<AnyKey, FadeState>,
    pub spawning_chunks: HashMap<AnyKey, Instant>,
    fade_duration: f32,
}

impl LodAnimator {
    pub fn new(fade_duration: f32) -> Self {
        Self {
            dying_chunks: HashMap::new(),
            spawning_chunks: HashMap::new(),
            fade_duration,
        }
    }

    fn smoothstep(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    pub fn start_spawn(&mut self, key: AnyKey) {
        self.dying_chunks.remove(&key);
        self.spawning_chunks.insert(key, Instant::now());
    }

    pub fn retire(&mut self, key: AnyKey, mesh: ChunkMesh) {
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
            let t = (now - *start).as_secs_f32() / self.fade_duration.max(0.001);
            return Self::smoothstep(t);
        }
        1.0
    }

    pub fn limit_retained(&mut self, max_dying: usize, max_spawning: usize) {
        while self.dying_chunks.len() > max_dying {
            let Some(key) = self.dying_chunks.keys().next().copied() else {
                break;
            };
            self.dying_chunks.remove(&key);
        }
        while self.spawning_chunks.len() > max_spawning {
            let Some(key) = self.spawning_chunks.keys().next().copied() else {
                break;
            };
            self.spawning_chunks.remove(&key);
        }
    }

    /// Advance dying animations; returns `(key, current_alpha)` for each
    /// still-active dying chunk.
    pub fn update_dying(&mut self, now: Instant) -> Vec<(AnyKey, f32)> {
        let mut results = Vec::new();
        let mut to_remove = Vec::new();
        for (key, state) in &self.dying_chunks {
            let t = (now - state.start_time).as_secs_f32() / state.duration.max(0.001);
            if t >= 1.0 {
                to_remove.push(*key);
            } else {
                results.push((*key, 1.0 - Self::smoothstep(t)));
            }
        }
        for k in to_remove {
            self.dying_chunks.remove(&k);
        }
        results
    }
}
