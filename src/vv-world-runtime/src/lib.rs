use std::collections::{HashMap, HashSet};
use vv_core::{BlockId as VoxelId, ChunkKey, CHUNK_SIZE};
use vv_planet::PlanetGeometry;
use vv_registry::BlockId as ContentBlockId;
use vv_world_gen::PlanetTerrain;

/// Tracks player modifications (mined / placed blocks) for a single chunk.
#[derive(Clone)]
pub struct ChunkMods {
    pub mined: HashSet<VoxelId>,
    pub placed: HashMap<VoxelId, ContentBlockId>,
}

impl ChunkMods {
    pub fn new() -> Self {
        Self {
            mined: HashSet::new(),
            placed: HashMap::new(),
        }
    }
}

impl Default for ChunkMods {
    fn default() -> Self {
        Self::new()
    }
}

/// Mutable runtime state of a planet: terrain + player edits.
///
/// The planet owns a pre-computed `PlanetTerrain` for fast height queries and
/// a sparse map of player-driven block additions / removals.
#[derive(Clone)]
pub struct PlanetData {
    /// Sparse per-chunk edit sets.
    pub chunks: HashMap<ChunkKey, ChunkMods>,
    /// Face grid resolution (equals radial layer count).
    pub resolution: u32,
    /// Physical planet geometry and meter/voxel conversion rules.
    pub geometry: PlanetGeometry,
    /// Whether the planet has an indestructible solid core.
    pub has_core: bool,
    /// Number of radial layers from the centre that cannot be mined.
    pub core_protection_layers: u32,
    /// Pre-computed terrain heightmap.
    pub terrain: PlanetTerrain,
}

impl PlanetData {
    pub fn new(
        geometry: PlanetGeometry,
        terrain: PlanetTerrain,
        core_protection_layers: u32,
    ) -> Self {
        Self {
            chunks: HashMap::new(),
            resolution: geometry.resolution,
            geometry,
            has_core: true,
            core_protection_layers,
            terrain,
        }
    }

    // --- Resize helpers -----------------------------------------------------

    /// Compute the next resolution when resizing (does not apply the change).
    pub fn next_geometry(&self, increase: bool) -> PlanetGeometry {
        let factor = if increase { 1.2 } else { 1.0 / 1.2 };
        let voxel_size_m = (self.geometry.voxel_size_m / factor).clamp(0.01, 10.0);
        PlanetGeometry::new(self.geometry.radius_m, voxel_size_m)
    }

    pub fn next_resolution(&self, increase: bool) -> u32 {
        if increase {
            let r = (self.resolution as f32 * 1.2) as u32;
            r.max(self.resolution + 1).min(16_384)
        } else {
            ((self.resolution as f32 / 1.2) as u32).max(8)
        }
    }

    /// Apply a resize: swap in a freshly-generated terrain and clear all edits.
    pub fn apply_resize(&mut self, new_geometry: PlanetGeometry, new_terrain: PlanetTerrain) {
        self.resolution = new_geometry.resolution;
        self.geometry = new_geometry;
        self.chunks.clear();
        self.terrain = new_terrain;
    }

    // --- Block operations ---------------------------------------------------

    pub fn add_block(&mut self, id: VoxelId, block: ContentBlockId) {
        let key = Self::chunk_key(id);
        let mods = self.chunks.entry(key).or_default();
        mods.mined.remove(&id);
        mods.placed.insert(id, block);
    }

    pub fn remove_block(&mut self, id: VoxelId) {
        if self.has_core && id.layer < self.core_protection_layers {
            return;
        }
        let key = Self::chunk_key(id);
        let mods = self.chunks.entry(key).or_default();
        if mods.placed.contains_key(&id) {
            mods.placed.remove(&id);
        } else if id.layer < self.resolution {
            mods.mined.insert(id);
        }
    }

    /// Returns `true` if a voxel exists at `id` (accounting for player edits).
    pub fn exists(&self, id: VoxelId) -> bool {
        self.block_at(id).is_some()
    }

    pub fn block_at(&self, id: VoxelId) -> Option<ContentBlockId> {
        let key = Self::chunk_key(id);
        if let Some(mods) = self.chunks.get(&key) {
            if let Some(block) = mods.placed.get(&id) {
                return Some(*block);
            }
            if mods.mined.contains(&id) {
                return None;
            }
        }
        let height = self.terrain.get_height(id.face, id.u, id.v);
        if id.layer <= height {
            Some(self.terrain.get_block(id.face, id.u, id.v, id.layer))
        } else {
            None
        }
    }

    pub fn runtime_stats(&self) -> PlanetRuntimeStats {
        let mut mined_blocks = 0usize;
        let mut placed_blocks = 0usize;
        let mut dirty_chunks = 0usize;
        for mods in self.chunks.values() {
            let dirty = !mods.mined.is_empty() || !mods.placed.is_empty();
            if dirty {
                dirty_chunks += 1;
            }
            mined_blocks += mods.mined.len();
            placed_blocks += mods.placed.len();
        }
        PlanetRuntimeStats {
            edited_chunks: self.chunks.len(),
            mined_blocks,
            placed_blocks,
            dirty_chunks,
        }
    }

    // --- Chunk key ----------------------------------------------------------

    #[inline]
    pub fn chunk_key(id: VoxelId) -> ChunkKey {
        ChunkKey {
            face: id.face,
            u_idx: id.u / CHUNK_SIZE,
            v_idx: id.v / CHUNK_SIZE,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlanetRuntimeStats {
    pub edited_chunks: usize,
    pub mined_blocks: usize,
    pub placed_blocks: usize,
    pub dirty_chunks: usize,
}
