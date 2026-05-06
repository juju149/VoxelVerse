use crate::content::BlockRegistry;
use crate::generation::{terrain::PlanetTerrain, CoordSystem};
use crate::voxel::{ChunkKey, VoxelCoord, VoxelId};
use crate::world::{PlanetProfile, VoxelRuntime};
use std::sync::Arc;

/// Cached runtime block IDs for world generation.
/// Looked up once from the registry at planet creation — avoids per-voxel string lookups.
struct PlanetBlockIds {
    core: VoxelId,
    dirt: VoxelId,
    grass: VoxelId,
}

impl PlanetBlockIds {
    fn from_registry(registry: &BlockRegistry) -> Self {
        Self {
            core: registry.lookup("core:core").unwrap_or(VoxelId::AIR),
            dirt: registry.lookup("core:dirt").unwrap_or(VoxelId::AIR),
            grass: registry.lookup("core:grass").unwrap_or(VoxelId::AIR),
        }
    }
}

#[derive(Clone)]
pub struct PlanetData {
    pub voxels: VoxelRuntime,
    pub content: Arc<BlockRegistry>,
    pub profile: PlanetProfile,
    pub resolution: u32,
    pub has_core: bool,
    pub terrain: PlanetTerrain,
    block_ids: PlanetBlockIds,
}

impl Clone for PlanetBlockIds {
    fn clone(&self) -> Self {
        Self {
            core: self.core,
            dirt: self.dirt,
            grass: self.grass,
        }
    }
}

impl PlanetData {
    pub fn new(resolution: u32, registry: Arc<BlockRegistry>) -> Self {
        let profile = PlanetProfile::new(resolution);
        println!(
            "Generating terrain for resolution {}...",
            profile.resolution
        );
        let terrain = PlanetTerrain::new(profile);
        println!("Terrain generation complete.");

        let block_ids = PlanetBlockIds::from_registry(&registry);

        Self {
            voxels: VoxelRuntime::new(),
            block_ids,
            content: registry,
            profile,
            resolution: profile.resolution,
            has_core: true,
            terrain,
        }
    }

    pub fn resize(&mut self, increase: bool) {
        if increase {
            let new_res = (self.resolution as f32 * 1.2) as u32;
            self.resolution = new_res.max(self.resolution + 1).min(16384);
        } else {
            let new_res = (self.resolution as f32 / 1.2) as u32;
            self.resolution = new_res.max(8);
        }

        self.voxels.clear();
        self.profile = PlanetProfile::new(self.resolution);
        self.resolution = self.profile.resolution;

        println!("Regenerating terrain for resolution {}...", self.resolution);
        self.terrain = PlanetTerrain::new(self.profile);
    }

    pub fn add_block(&mut self, coord: VoxelCoord) {
        let voxel = self.content.default_place_voxel();
        self.set_voxel(coord, voxel);
    }

    pub fn remove_block(&mut self, coord: VoxelCoord) {
        if self.has_core && coord.layer < self.profile.core_layers {
            return;
        }

        self.set_voxel(coord, VoxelId::AIR);
    }

    pub fn get_voxel(&self, coord: VoxelCoord) -> VoxelId {
        self.voxels
            .get_override(coord)
            .unwrap_or_else(|| self.generated_voxel(coord))
    }

    pub fn set_voxel(&mut self, coord: VoxelCoord, voxel: VoxelId) {
        let generated = self.generated_voxel(coord);
        let override_voxel = (voxel != generated).then_some(voxel);
        self.voxels.set_override(coord, override_voxel);
    }

    pub fn exists(&self, coord: VoxelCoord) -> bool {
        self.content.is_solid(self.get_voxel(coord))
    }

    pub fn modified_voxels_in_chunk_column(
        &self,
        key: ChunkKey,
    ) -> impl Iterator<Item = (VoxelCoord, VoxelId)> + '_ {
        self.voxels
            .iter_column_overrides(key.face, key.u_idx, key.v_idx)
            .filter(move |(coord, _)| coord.u < self.resolution && coord.v < self.resolution)
    }

    fn generated_voxel(&self, coord: VoxelCoord) -> VoxelId {
        if coord.layer >= self.resolution
            || coord.u >= self.resolution
            || coord.v >= self.resolution
        {
            return VoxelId::AIR;
        }

        let height = self.terrain.get_height(coord.face, coord.u, coord.v);
        if coord.layer > height {
            VoxelId::AIR
        } else if self.has_core && coord.layer < self.profile.core_layers {
            self.block_ids.core
        } else if coord.layer == height {
            self.block_ids.grass
        } else {
            self.block_ids.dirt
        }
    }

    pub fn surface_radius(&self, face: u8, u: u32, v: u32) -> f32 {
        let h = self.terrain.get_height(face, u, v);
        self.profile.layer_radius(h + 1)
    }

    pub fn spawn_position(&self) -> glam::Vec3 {
        let u = self.resolution / 2;
        let v = self.resolution / 2;
        let dir = CoordSystem::get_direction(0, u, v, self.resolution);
        dir * (self.surface_radius(0, u, v) + self.profile.spawn_clearance())
    }
}

