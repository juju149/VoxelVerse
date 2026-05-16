//! Read-only snapshot of `PlanetData` for off-thread mesh workers.
//!
//! Replaces the previous `PlanetData::clone()` per dispatched job — which
//! deep-cloned the entire override `HashMap`, broken-prop set and string
//! fields of `CompiledPlanet` — with an `Arc`-backed view that is cheap to
//! produce and cheap to clone (ref-count bumps + a few `Copy` fields).
//!
//! Edits on the main thread go through `Arc::make_mut`, so a writer only
//! pays the copy-on-write cost when a worker is still holding the previous
//! version.

use crate::{BrokenPropLayer, TerrainVisualPalette, VoxModelRegistry, VoxelRuntime};
use std::sync::Arc;
use vv_pack_compiler::BlockRegistry;
use vv_voxel::{PlanetProfile, SurfaceChunkKey, VoxelCoord, VoxelId};
use vv_worldgen::{bake_for_chunk, ChunkFeatureMap, ProceduralPlanetTerrain};

#[derive(Clone)]
pub struct PlanetSnapshot {
    pub voxels: Arc<VoxelRuntime>,
    pub broken_props: Arc<BrokenPropLayer>,
    pub content: Arc<BlockRegistry>,
    pub terrain_visuals: Arc<TerrainVisualPalette>,
    pub prop_models: Arc<VoxModelRegistry>,
    pub terrain: ProceduralPlanetTerrain,
    pub profile: PlanetProfile,
    pub resolution: u32,
    pub has_core: bool,
    pub core_voxel: VoxelId,
    pub player_surface_key: Option<SurfaceChunkKey>,
}

impl PlanetSnapshot {
    pub fn get_voxel(&self, coord: VoxelCoord) -> VoxelId {
        self.voxels
            .get_override(coord)
            .unwrap_or_else(|| self.generated_voxel(coord))
    }

    pub fn generated_voxel(&self, coord: VoxelCoord) -> VoxelId {
        if coord.layer >= self.resolution
            || coord.u >= self.resolution
            || coord.v >= self.resolution
        {
            return VoxelId::AIR;
        }
        if self.has_core && coord.layer < self.profile.core_layers {
            self.core_voxel
        } else {
            self.terrain.voxel_at(coord, self.profile)
        }
    }

    pub fn bake_chunk_features(&self, key: SurfaceChunkKey, margin: u32) -> ChunkFeatureMap {
        bake_for_chunk(&self.terrain, key.face, key.u_idx, key.v_idx, margin)
    }

    pub fn modified_voxels_in_chunk_column(
        &self,
        key: SurfaceChunkKey,
    ) -> impl Iterator<Item = (VoxelCoord, VoxelId)> + '_ {
        self.voxels
            .iter_column_overrides(key.face, key.u_idx, key.v_idx)
            .filter(move |(coord, _)| coord.u < self.resolution && coord.v < self.resolution)
    }
}
