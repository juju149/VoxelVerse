//! Read-only query methods on `PlanetData`.

use crate::PlanetData;
use vv_pack_compiler::{CompiledBlock, CompiledBlockVisual, CompiledItem, ItemId, ItemRegistry};
use vv_voxel::{SurfaceChunkKey, VoxelCoord, VoxelId};
use vv_worldgen::{bake_for_chunk, ChunkFeatureMap};

impl PlanetData {
    pub fn resolution(&self) -> u32 {
        self.resolution
    }

    pub fn profile(&self) -> vv_voxel::PlanetProfile {
        self.profile
    }

    pub fn world_time(&self) -> crate::WorldTime {
        self.world_time
    }

    pub fn block(&self, id: VoxelId) -> Option<&CompiledBlock> {
        self.content.block(id)
    }

    pub fn block_color(&self, id: VoxelId) -> [f32; 3] {
        self.content.color(id)
    }

    pub fn block_visual(&self, id: VoxelId) -> &CompiledBlockVisual {
        self.content.visual(id)
    }

    pub fn item(&self, id: ItemId) -> Option<&CompiledItem> {
        self.items.get(id)
    }

    pub fn items(&self) -> &ItemRegistry {
        &self.items
    }

    pub fn edge_rounding_radius_voxels(&self) -> f32 {
        self.profile.edge_rounding_radius_voxels
    }

    pub fn block_damage_revision(&self) -> u64 {
        self.block_damage.revision()
    }

    pub fn damaged_block_coords(&self) -> impl Iterator<Item = VoxelCoord> + '_ {
        self.block_damage.iter().map(|(coord, _)| coord)
    }

    pub fn surface_height(&self, face: u8, u: u32, v: u32) -> u32 {
        self.terrain.get_height(face, u, v)
    }

    pub fn terrain_surface_layer(&self, face: u8, u: u32, v: u32) -> u32 {
        self.terrain.terrain_surface_layer(face, u, v)
    }

    pub fn surface_radius(&self, face: u8, u: u32, v: u32) -> f32 {
        let h = self.terrain.get_height(face, u, v);
        self.profile.layer_radius(h + 1)
    }

    pub fn get_voxel(&self, coord: VoxelCoord) -> VoxelId {
        self.voxels
            .get_override(coord)
            .unwrap_or_else(|| self.generated_voxel(coord))
    }

    pub fn exists(&self, coord: VoxelCoord) -> bool {
        self.content.is_solid(self.get_voxel(coord))
    }

    /// Resolve an `ItemId` to the `VoxelId` it places, if the item is a
    /// block-placement item. Returns `None` for tools, food, weapons, etc.
    pub fn resolve_item_voxel(&self, item_id: ItemId) -> Option<VoxelId> {
        use vv_pack_compiler::CompiledItemGameplay;
        let item = self.items.get(item_id)?;
        match &item.gameplay {
            CompiledItemGameplay::PlaceBlock { block_key } => self.content.lookup(block_key),
            _ => None,
        }
    }

    pub fn block_damage_fraction(&self, coord: VoxelCoord) -> Option<f32> {
        let voxel = self.get_voxel(coord);
        let block = self.content.block(voxel)?;
        self.block_damage
            .damage_fraction_for_voxel(coord, voxel, block.hardness.max(1.0))
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
}
