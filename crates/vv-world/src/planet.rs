use crate::{
    BlockDamageLayer, BrokenPropLayer, PlanetProfile, TerrainVisualPalette, VoxModelRegistry,
    VoxelRuntime, WorldTime,
};
use std::sync::Arc;
use vv_meshing::MeshMaterialTable;
use vv_pack_compiler::{BlockRegistry, CompiledPlanet, ItemRegistry, ProceduralRegistry};
use vv_voxel::{SurfaceChunkKey, VoxelCoord, VoxelId, CHUNK_SIZE};
use vv_worldgen::{ProceduralPlanetTerrain, WorldgenStatsSnapshot};

/// Cached runtime block ID for the planet core (deep underground).
/// Surface/subsurface blocks come from the biome registry.
#[derive(Clone, Copy)]
pub(crate) struct PlanetBlockIds {
    pub(crate) core: VoxelId,
}

impl PlanetBlockIds {
    fn from_registry(registry: &BlockRegistry) -> Self {
        Self {
            core: registry.planet_core_voxel(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VoxelEditResult {
    pub changed: VoxelCoord,
    pub dirty_chunks: Vec<SurfaceChunkKey>,
}

pub struct PlanetDataSources {
    pub registry: Arc<BlockRegistry>,
    pub items: Arc<ItemRegistry>,
    pub terrain_visuals: Arc<TerrainVisualPalette>,
    pub procedural: Arc<ProceduralRegistry>,
    pub procedural_planet_index: usize,
    pub prop_models: Arc<VoxModelRegistry>,
}

pub trait VoxelRead {
    fn resolution(&self) -> u32;
    fn profile(&self) -> PlanetProfile;
    fn get_voxel(&self, coord: VoxelCoord) -> VoxelId;
    fn exists(&self, coord: VoxelCoord) -> bool;
}

#[derive(Clone)]
pub struct PlanetData {
    pub(crate) voxels: Arc<VoxelRuntime>,
    pub(crate) content: Arc<BlockRegistry>,
    pub(crate) items: Arc<ItemRegistry>,
    pub(crate) terrain_visuals: Arc<TerrainVisualPalette>,
    pub(crate) material_table: Arc<MeshMaterialTable>,
    pub(crate) procedural: Arc<ProceduralRegistry>,
    pub(crate) procedural_planet_index: usize,
    pub(crate) profile: PlanetProfile,
    pub(crate) resolution: u32,
    pub(crate) has_core: bool,
    pub(crate) terrain: ProceduralPlanetTerrain,
    pub(crate) world_time: WorldTime,
    pub(crate) prop_models: Arc<VoxModelRegistry>,
    pub(crate) broken_props: Arc<BrokenPropLayer>,
    pub(crate) block_damage: BlockDamageLayer,
    pub(crate) player_surface_key: Option<SurfaceChunkKey>,
    pub(crate) block_ids: PlanetBlockIds,
    pub(crate) planet_def: CompiledPlanet,
    /// Seed stored so resize can regenerate terrain with the same seed.
    pub(crate) seed: u32,
}

impl PlanetData {
    pub fn new(
        planet_def: CompiledPlanet,
        registry: Arc<BlockRegistry>,
        items: Arc<ItemRegistry>,
        procedural: Arc<ProceduralRegistry>,
        procedural_planet_index: usize,
    ) -> Self {
        Self::new_with_progress(
            planet_def,
            PlanetDataSources {
                registry: registry.clone(),
                items,
                terrain_visuals: Arc::new(TerrainVisualPalette::fallback_from_blocks(&registry)),
                procedural,
                procedural_planet_index,
                prop_models: Arc::new(VoxModelRegistry::default()),
            },
            |_, _| {},
        )
    }

    pub fn new_with_progress(
        planet_def: CompiledPlanet,
        sources: PlanetDataSources,
        progress: impl FnMut(f32, &str),
    ) -> Self {
        let PlanetDataSources {
            registry,
            items,
            terrain_visuals,
            procedural,
            procedural_planet_index,
            prop_models,
        } = sources;
        let profile = planet_def.to_planet_profile();
        println!(
            "Generating terrain for resolution {}  (voxel {} m, radius ≈ {:.1} m)…",
            profile.resolution, profile.voxel_size_meters, profile.surface_radius
        );
        let terrain = ProceduralPlanetTerrain::new_with_progress(
            profile,
            procedural.clone(),
            procedural_planet_index,
            progress,
        );
        println!("Terrain generation complete.");

        let block_ids = PlanetBlockIds::from_registry(&registry);
        let material_table = Arc::new(crate::mesh_input_builder::build_material_table(&registry));

        Self {
            voxels: Arc::new(VoxelRuntime::new()),
            block_ids,
            content: registry,
            items,
            terrain_visuals,
            material_table,
            procedural,
            procedural_planet_index,
            profile,
            resolution: profile.resolution,
            has_core: true,
            terrain,
            world_time: WorldTime::new(1_200.0, 0.15),
            prop_models,
            broken_props: Arc::new(BrokenPropLayer::new()),
            block_damage: BlockDamageLayer::new(),
            player_surface_key: None,
            seed: profile.seed,
            planet_def,
        }
    }

    pub(crate) fn generated_voxel(&self, coord: VoxelCoord) -> VoxelId {
        if coord.layer >= self.resolution
            || coord.u >= self.resolution
            || coord.v >= self.resolution
        {
            return VoxelId::AIR;
        }

        if self.has_core && coord.layer < self.profile.core_layers {
            self.block_ids.core
        } else {
            self.terrain.voxel_at(coord, self.profile)
        }
    }

    pub fn worldgen_stats_snapshot(&self) -> WorldgenStatsSnapshot {
        self.terrain.stats().snapshot()
    }

    pub(crate) fn dirty_chunks_for_coord(
        resolution: u32,
        coord: VoxelCoord,
    ) -> Vec<SurfaceChunkKey> {
        if coord.u >= resolution || coord.v >= resolution {
            return Vec::new();
        }

        let max_chunk = resolution.saturating_sub(1) / CHUNK_SIZE;
        let u_idx = coord.u / CHUNK_SIZE;
        let v_idx = coord.v / CHUNK_SIZE;
        let mut keys = vec![SurfaceChunkKey {
            face: coord.face,
            u_idx,
            v_idx,
        }];

        if coord.u.is_multiple_of(CHUNK_SIZE) && u_idx > 0 {
            keys.push(SurfaceChunkKey {
                face: coord.face,
                u_idx: u_idx - 1,
                v_idx,
            });
        }
        if coord.u % CHUNK_SIZE == CHUNK_SIZE - 1 && u_idx < max_chunk {
            keys.push(SurfaceChunkKey {
                face: coord.face,
                u_idx: u_idx + 1,
                v_idx,
            });
        }
        if coord.v.is_multiple_of(CHUNK_SIZE) && v_idx > 0 {
            keys.push(SurfaceChunkKey {
                face: coord.face,
                u_idx,
                v_idx: v_idx - 1,
            });
        }
        if coord.v % CHUNK_SIZE == CHUNK_SIZE - 1 && v_idx < max_chunk {
            keys.push(SurfaceChunkKey {
                face: coord.face,
                u_idx,
                v_idx: v_idx + 1,
            });
        }

        keys.sort_by_key(|k| (k.face, k.u_idx, k.v_idx));
        keys.dedup_by_key(|k| (k.face, k.u_idx, k.v_idx));
        keys
    }
}

impl VoxelRead for PlanetData {
    fn resolution(&self) -> u32 {
        self.resolution
    }

    fn profile(&self) -> PlanetProfile {
        self.profile
    }

    fn get_voxel(&self, coord: VoxelCoord) -> VoxelId {
        PlanetData::get_voxel(self, coord)
    }

    fn exists(&self, coord: VoxelCoord) -> bool {
        PlanetData::exists(self, coord)
    }
}

#[cfg(test)]
mod tests {
    use super::PlanetData;
    use vv_voxel::{SurfaceChunkKey, VoxelCoord, CHUNK_SIZE};

    fn coord(u: u32, v: u32) -> VoxelCoord {
        VoxelCoord {
            face: 2,
            layer: 8,
            u,
            v,
        }
    }

    #[test]
    fn dirty_chunks_include_only_current_chunk_for_interior_edit() {
        let dirty = PlanetData::dirty_chunks_for_coord(128, coord(3, 4));
        assert_eq!(
            dirty,
            vec![SurfaceChunkKey {
                face: 2,
                u_idx: 0,
                v_idx: 0,
            }]
        );
    }

    #[test]
    fn dirty_chunks_include_neighbor_only_on_chunk_border() {
        let dirty = PlanetData::dirty_chunks_for_coord(128, coord(CHUNK_SIZE, CHUNK_SIZE - 1));
        assert_eq!(
            dirty,
            vec![
                SurfaceChunkKey {
                    face: 2,
                    u_idx: 0,
                    v_idx: 0,
                },
                SurfaceChunkKey {
                    face: 2,
                    u_idx: 1,
                    v_idx: 0,
                },
                SurfaceChunkKey {
                    face: 2,
                    u_idx: 1,
                    v_idx: 1,
                },
            ]
        );
    }
}
