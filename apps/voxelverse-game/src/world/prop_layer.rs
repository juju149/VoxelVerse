//! Vox prop instance layer.
//!
//! Props are small MagicaVoxel models (.vox) placed procedurally on the
//! terrain surface. They are NOT stored in the voxel grid: they live in a
//! separate layer so they can be rendered, broken, and queried independently.
#![allow(dead_code)]
//!
//! Placement is deterministic (same as terrain generation). The only mutable
//! state is the `broken` set: positions where the prop or its support block
//! was destroyed by the player.

use crate::voxel::SurfaceChunkKey;
use std::collections::{HashMap, HashSet};

/// A single vox prop instance — one small .vox model sitting on the terrain.
#[derive(Clone, Debug)]
pub struct PropInstance {
    /// Planet face index (0-5).
    pub face: u8,
    /// Grid column on the face.
    pub u: u32,
    pub v: u32,
    /// Layer index of the surface block the prop sits on.
    pub surface_layer: u32,
    /// Content ref key to a specific .vox asset,
    /// e.g. `"core:voxel/vegetation/flowers/flower_blue_1"`.
    pub model_key: String,
    /// Quarter-turn rotation around the radial (outward) axis, 0-3.
    pub rotation: u8,
}

/// Compact key for one surface column (face, u, v).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PropSupportKey {
    pub face: u8,
    pub u: u32,
    pub v: u32,
}

impl PropSupportKey {
    pub fn new(face: u8, u: u32, v: u32) -> Self {
        Self { face, u, v }
    }
}

/// The prop state for a planet. Tracks which columns have had their prop
/// (or support block) manually broken by the player.
#[derive(Clone, Debug, Default)]
pub struct PropLayer {
    /// Columns where the prop has been explicitly destroyed.
    /// When a prop appears here it is not rendered, even if the procedural
    /// placement would spawn one there.
    broken: HashSet<PropSupportKey>,
}

impl PropLayer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that the prop at `(face, u, v)` was broken.
    pub fn break_prop(&mut self, face: u8, u: u32, v: u32) {
        self.broken.insert(PropSupportKey::new(face, u, v));
    }

    /// True iff the prop at this column is NOT broken.
    pub fn is_alive(&self, face: u8, u: u32, v: u32) -> bool {
        !self.broken.contains(&PropSupportKey::new(face, u, v))
    }
}

/// Cached list of prop instances for one chunk, after filtering through the
/// `PropLayer` broken set.
#[derive(Clone, Debug, Default)]
pub struct ChunkPropList {
    pub instances: Vec<PropInstance>,
}

/// Per-chunk prop cache. Keyed by `SurfaceChunkKey`. Rebuilt when the chunk
/// is loaded; individual entries are removed when a prop is broken.
#[derive(Default)]
pub struct ChunkPropCache {
    cache: HashMap<SurfaceChunkKey, ChunkPropList>,
}

impl ChunkPropCache {
    pub fn insert(&mut self, key: SurfaceChunkKey, list: ChunkPropList) {
        self.cache.insert(key, list);
    }

    pub fn get(&self, key: &SurfaceChunkKey) -> Option<&ChunkPropList> {
        self.cache.get(key)
    }

    pub fn remove(&mut self, key: &SurfaceChunkKey) {
        self.cache.remove(key);
    }

    /// Remove instances at a specific column across all cached chunks (called
    /// when the player breaks a prop or its support block).
    pub fn invalidate_column(&mut self, face: u8, u: u32, v: u32) {
        for list in self.cache.values_mut() {
            list.instances
                .retain(|inst| !(inst.face == face && inst.u == u && inst.v == v));
        }
    }
}
