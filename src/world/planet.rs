use crate::content::{BiomeRegistry, BlockRegistry};
use crate::generation::{terrain::PlanetTerrain, CoordSystem};
use crate::voxel::{SurfaceChunkKey, VoxelCoord, VoxelId};
use crate::world::{PlanetProfile, VoxelRuntime};
use std::sync::Arc;

/// Cached runtime block ID for the planet core (deep underground).
/// Surface/subsurface blocks come from the biome registry.
#[derive(Clone, Copy)]
struct PlanetBlockIds {
    core: VoxelId,
}

impl PlanetBlockIds {
    fn from_registry(registry: &BlockRegistry) -> Self {
        Self {
            core: registry.lookup("core:core").unwrap_or(VoxelId::AIR),
        }
    }
}

#[derive(Clone)]
pub struct PlanetData {
    pub voxels: VoxelRuntime,
    pub content: Arc<BlockRegistry>,
    pub biomes: Arc<BiomeRegistry>,
    pub profile: PlanetProfile,
    pub resolution: u32,
    pub has_core: bool,
    pub terrain: PlanetTerrain,
    block_ids: PlanetBlockIds,
    /// Seed stored so resize can regenerate terrain with the same seed.
    seed: u32,
}

impl PlanetData {
    pub fn new(
        resolution: u32,
        seed: u32,
        registry: Arc<BlockRegistry>,
        biome_registry: Arc<BiomeRegistry>,
    ) -> Self {
        let profile = PlanetProfile::with_seed(resolution, seed);
        println!(
            "Generating terrain for resolution {}  (radius ≈ {})…",
            profile.resolution,
            profile.resolution / 2
        );
        let terrain = PlanetTerrain::new(profile, &biome_registry);
        println!("Terrain generation complete.");

        let block_ids = PlanetBlockIds::from_registry(&registry);

        Self {
            voxels: VoxelRuntime::new(),
            block_ids,
            content: registry,
            biomes: biome_registry,
            profile,
            resolution: profile.resolution,
            has_core: true,
            terrain,
            seed,
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
        self.profile = PlanetProfile::with_seed(self.resolution, self.seed);
        self.resolution = self.profile.resolution;

        println!("Regenerating terrain for resolution {}…", self.resolution);
        self.terrain = PlanetTerrain::new(self.profile, &self.biomes);
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
        key: SurfaceChunkKey,
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
        } else {
            // Surface and subsurface blocks come from the biome at this location.
            let biome_id = self.terrain.get_biome_id(coord.face, coord.u, coord.v);
            let biome = self.biomes.biome(biome_id);
            if coord.layer == height {
                biome.surface_block
            } else {
                biome.subsurface_block
            }
        }
    }

    pub fn surface_radius(&self, face: u8, u: u32, v: u32) -> f32 {
        let h = self.terrain.get_height(face, u, v);
        self.profile.layer_radius(h + 1)
    }

    pub fn spawn_position(&self) -> glam::Vec3 {
        // Face 4 = equatorial +Z face.  At center: dir.y ≈ 0 → latitude ≈ 0
        // → temperature ≈ 1.0 → tropical biome.  Face 0 is the +Y pole.
        let u = self.resolution / 2;
        let v = self.resolution / 2;
        let dir = CoordSystem::get_direction(4, u, v, self.resolution);
        dir * (self.surface_radius(4, u, v) + self.profile.spawn_clearance())
    }
}
