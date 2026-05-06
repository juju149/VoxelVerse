use crate::generation::terrain::PlanetTerrain;
use crate::voxel::{BlockId, ChunkKey, CHUNK_SIZE};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct ChunkMods {
    pub mined: HashSet<BlockId>,
    pub placed: HashSet<BlockId>,
}

impl ChunkMods {
    pub fn new() -> Self {
        Self {
            mined: HashSet::new(),
            placed: HashSet::new(),
        }
    }
}

#[derive(Clone)]
pub struct PlanetData {
    pub chunks: HashMap<ChunkKey, ChunkMods>,
    pub resolution: u32,
    pub has_core: bool,
    pub terrain: PlanetTerrain,
}

impl PlanetData {
    pub fn new(resolution: u32) -> Self {
        println!("Generating Terrain Noise Map for res {}...", resolution);
        let terrain = PlanetTerrain::new(resolution); // calculate once
        println!("Terrain Generation Complete.");

        Self {
            chunks: HashMap::new(),
            resolution,
            has_core: true,
            terrain, // <--- Store it
        }
    }

    pub fn resize(&mut self, increase: bool) {
        if increase {
            // multiply by 1.2
            // i use .max(self.resolution + 1) to ensure it always grows by at least 1 block
            let new_res = (self.resolution as f32 * 1.2) as u32;
            self.resolution = new_res.max(self.resolution + 1).min(16384);
        } else {
            // divide by 1.2
            let new_res = (self.resolution as f32 / 1.2) as u32;
            self.resolution = new_res.max(8);
        }

        self.chunks.clear();

        // regenerate noise map for new resolution
        println!("Regenerating Terrain for new res {}...", self.resolution);
        self.terrain = PlanetTerrain::new(self.resolution);
    }

    fn get_chunk_key(id: BlockId) -> ChunkKey {
        ChunkKey {
            face: id.face,
            u_idx: id.u / CHUNK_SIZE,
            v_idx: id.v / CHUNK_SIZE,
        }
    }

    pub fn add_block(&mut self, id: BlockId) {
        let key = Self::get_chunk_key(id);
        let mods = self.chunks.entry(key).or_insert_with(ChunkMods::new);

        if mods.mined.contains(&id) {
            mods.mined.remove(&id);
        } else {
            mods.placed.insert(id);
        }
    }

    pub fn remove_block(&mut self, id: BlockId) {
        // protect the bottom 4 layers as the unbreakable core
        if self.has_core && id.layer < 6 {
            return;
        }

        let key = Self::get_chunk_key(id);
        let mods = self.chunks.entry(key).or_insert_with(ChunkMods::new);

        if mods.placed.contains(&id) {
            mods.placed.remove(&id);
        } else {
            if id.layer < self.resolution {
                mods.mined.insert(id);
            }
        }
    }

    pub fn exists(&self, id: BlockId) -> bool {
        let key = Self::get_chunk_key(id);
        if let Some(mods) = self.chunks.get(&key) {
            if mods.placed.contains(&id) {
                return true;
            }
            if mods.mined.contains(&id) {
                return false;
            }
        }

        // instead of a flat floor, we check the pre-calculated noise map
        let height = self.terrain.get_height(id.face, id.u, id.v);
        id.layer <= height
    }
}
