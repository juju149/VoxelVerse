use crate::{BrokenPropLayer, PlanetProfile, TerrainVisualPalette, VoxModelRegistry, VoxelRuntime};
use std::sync::Arc;
use vv_math::CoordSystem;
use vv_pack_compiler::{BlockRegistry, CompiledPlanet, ItemRegistry, ProceduralRegistry};
use vv_voxel::{SurfaceChunkKey, VoxelCoord, VoxelId, CHUNK_SIZE};
use vv_worldgen::{bake_for_chunk, ChunkFeatureMap, ProceduralPlanetTerrain};

/// Cached runtime block ID for the planet core (deep underground).
/// Surface/subsurface blocks come from the biome registry.
#[derive(Clone, Copy)]
struct PlanetBlockIds {
    core: VoxelId,
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

#[allow(dead_code)]
pub trait VoxelRead {
    fn resolution(&self) -> u32;
    fn profile(&self) -> PlanetProfile;
    fn get_voxel(&self, coord: VoxelCoord) -> VoxelId;
    fn exists(&self, coord: VoxelCoord) -> bool;
}

#[derive(Clone)]
pub struct PlanetData {
    pub voxels: VoxelRuntime,
    pub content: Arc<BlockRegistry>,
    pub items: Arc<ItemRegistry>,
    pub terrain_visuals: Arc<TerrainVisualPalette>,
    pub procedural: Arc<ProceduralRegistry>,
    pub procedural_planet_index: usize,
    pub profile: PlanetProfile,
    pub resolution: u32,
    pub has_core: bool,
    pub terrain: ProceduralPlanetTerrain,
    /// Read-only registry of pre-loaded .vox prop models.
    pub prop_models: Arc<VoxModelRegistry>,
    /// Mutable set of prop columns the player has explicitly destroyed.
    pub broken_props: BrokenPropLayer,
    /// Surface-chunk key of the player's current position (updated each frame
    /// before rayon workers start).  Used by the mesher's prop LOD gate to skip
    /// prop geometry in chunks beyond `PROP_LOD_CHUNK_RADIUS`.
    pub player_surface_key: Option<SurfaceChunkKey>,
    block_ids: PlanetBlockIds,
    planet_def: CompiledPlanet,
    /// Seed stored so resize can regenerate terrain with the same seed.
    seed: u32,
}

impl PlanetData {
    #[allow(dead_code)]
    pub fn new(
        planet_def: CompiledPlanet,
        registry: Arc<BlockRegistry>,
        items: Arc<ItemRegistry>,
        procedural: Arc<ProceduralRegistry>,
        procedural_planet_index: usize,
    ) -> Self {
        Self::new_with_progress(
            planet_def,
            registry.clone(),
            items,
            Arc::new(TerrainVisualPalette::fallback_from_blocks(&registry)),
            procedural,
            procedural_planet_index,
            Arc::new(VoxModelRegistry::default()),
            |_, _| {},
        )
    }

    pub fn new_with_progress(
        planet_def: CompiledPlanet,
        registry: Arc<BlockRegistry>,
        items: Arc<ItemRegistry>,
        terrain_visuals: Arc<TerrainVisualPalette>,
        procedural: Arc<ProceduralRegistry>,
        procedural_planet_index: usize,
        prop_models: Arc<VoxModelRegistry>,
        progress: impl FnMut(f32, &str),
    ) -> Self {
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

        Self {
            voxels: VoxelRuntime::new(),
            block_ids,
            content: registry,
            items,
            terrain_visuals,
            procedural,
            procedural_planet_index,
            profile,
            resolution: profile.resolution,
            has_core: true,
            terrain,
            prop_models,
            broken_props: BrokenPropLayer::new(),
            player_surface_key: None,
            seed: profile.seed,
            planet_def,
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
        self.planet_def = self.planet_def.with_resolution(self.resolution);
        self.planet_def.seed = self.seed;
        self.profile = self.planet_def.to_planet_profile();
        self.resolution = self.profile.resolution;

        println!("Regenerating terrain for resolution {}…", self.resolution);
        self.terrain = ProceduralPlanetTerrain::new(
            self.profile,
            self.procedural.clone(),
            self.procedural_planet_index,
        );
    }

    pub fn place_block(&mut self, coord: VoxelCoord, voxel: VoxelId) -> VoxelEditResult {
        self.set_voxel(coord, voxel)
    }

    pub fn remove_block(&mut self, coord: VoxelCoord) -> VoxelEditResult {
        if self.has_core && coord.layer < self.profile.core_layers {
            return VoxelEditResult {
                changed: coord,
                dirty_chunks: Vec::new(),
            };
        }

        // If a prop is sitting directly above the broken block, destroy it too.
        // Props sit at `surface_layer + 1`, so a prop whose surface_layer == coord.layer
        // is supported by this block.
        self.broken_props.break_prop(coord.face, coord.u, coord.v);

        self.set_voxel(coord, VoxelId::AIR)
    }

    pub fn get_voxel(&self, coord: VoxelCoord) -> VoxelId {
        self.voxels
            .get_override(coord)
            .unwrap_or_else(|| self.generated_voxel(coord))
    }

    /// Resolve an `ItemId` to the `VoxelId` it places, if the item is a
    /// block-placement item. Returns `None` for tools, food, weapons, etc.
    ///
    /// This is the canonical bridge between the item inventory and the voxel
    /// world: the renderer and placement logic use this to display or place
    /// block-item stacks.
    pub fn resolve_item_voxel(&self, item_id: vv_pack_compiler::ItemId) -> Option<VoxelId> {
        use vv_pack_compiler::CompiledItemGameplay;
        let item = self.items.get(item_id)?;
        match &item.gameplay {
            CompiledItemGameplay::PlaceBlock { block_key } => self.content.lookup(block_key),
            _ => None,
        }
    }

    pub fn set_voxel(&mut self, coord: VoxelCoord, voxel: VoxelId) -> VoxelEditResult {
        let generated = self.generated_voxel(coord);
        let override_voxel = (voxel != generated).then_some(voxel);
        self.voxels.set_override(coord, override_voxel);
        VoxelEditResult {
            changed: coord,
            dirty_chunks: Self::dirty_chunks_for_coord(self.resolution, coord),
        }
    }

    pub fn exists(&self, coord: VoxelCoord) -> bool {
        self.content.is_solid(self.get_voxel(coord))
    }

    /// Bake a full chunk's tree + visual-detail voxels into a sparse map.
    /// The mesher uses this so it never has to re-scan tree neighbourhoods
    /// at the per-voxel level.
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

    fn generated_voxel(&self, coord: VoxelCoord) -> VoxelId {
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

    fn dirty_chunks_for_coord(resolution: u32, coord: VoxelCoord) -> Vec<SurfaceChunkKey> {
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
