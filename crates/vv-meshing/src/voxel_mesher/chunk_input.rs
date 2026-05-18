use std::collections::HashMap;
use std::sync::Arc;
use vv_voxel::{PlanetProfile, SurfaceChunkKey, VoxelId};

use super::material_packing::MeshMaterialTable;
use super::prop_integration::PropMeshInstance;

/// Pre-resolved voxel data for the chunk plus a 1-voxel UV border.
///
/// The caller (world layer) resolves all sources — terrain heightfield,
/// feature voxels, player overrides — before handing this to the mesher.
/// The mesher never touches `PlanetSnapshot` or `ProceduralPlanetTerrain`.
pub struct ChunkVoxelView {
    pub face: u8,
    pub resolution: u32,
    /// Sparse store: (layer, u, v) → VoxelId.
    /// Only non-AIR voxels and their 6-connected neighbors are stored.
    voxels: HashMap<(u32, u32, u32), VoxelId>,
}

impl ChunkVoxelView {
    pub fn new(face: u8, resolution: u32) -> Self {
        Self {
            face,
            resolution,
            voxels: HashMap::new(),
        }
    }

    pub fn insert(&mut self, layer: u32, u: u32, v: u32, id: VoxelId) {
        if id != VoxelId::AIR {
            self.voxels.insert((layer, u, v), id);
        }
    }

    /// Returns the voxel at the given absolute planet coordinate.
    /// Out-of-bounds UV returns AIR; negative layer is treated as core (solid).
    pub fn get(&self, layer: u32, u: u32, v: u32) -> VoxelId {
        if u >= self.resolution || v >= self.resolution {
            return VoxelId::AIR;
        }
        *self.voxels.get(&(layer, u, v)).unwrap_or(&VoxelId::AIR)
    }

    /// Iterate all stored (non-AIR) voxels as `(layer, u, v, id)`.
    pub fn iter_voxels(&self) -> impl Iterator<Item = (u32, u32, u32, VoxelId)> + '_ {
        self.voxels
            .iter()
            .map(|(&(layer, u, v), &id)| (layer, u, v, id))
    }

    /// Signed-offset version used by face culling.
    pub fn get_signed(&self, layer: i32, u: i32, v: i32) -> Option<VoxelId> {
        if layer < 0 {
            return None; // signals "core is solid"
        }
        if u < 0 || v < 0 || u >= self.resolution as i32 || v >= self.resolution as i32 {
            return Some(VoxelId::AIR);
        }
        Some(self.get(layer as u32, u as u32, v as u32))
    }
}

/// Surface heights and sea-level data for the chunk plus its 1-voxel UV border.
///
/// Used by the mesher to build the candidate list (cliff fill, water layer)
/// and to classify voxels for lighting.
pub struct ChunkBorderSamples {
    pub face: u8,
    /// First u index in `heights` (= chunk_u_start.saturating_sub(1)).
    pub u_start: u32,
    /// First v index in `heights` (= chunk_v_start.saturating_sub(1)).
    pub v_start: u32,
    /// Side length of the height array (= CHUNK_SIZE + 2).
    pub width: u32,
    pub resolution: u32,
    pub sea_level: u32,
    pub water_voxel: VoxelId,
    /// Planet geometry profile — needed by the face emitter.
    pub profile: PlanetProfile,
    /// Surface height at each (u, v) in [u_start, u_start + width).
    /// Indexed as `heights[(u - u_start) * width + (v - v_start)]`.
    heights: Vec<u32>,
}

impl ChunkBorderSamples {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        face: u8,
        u_start: u32,
        v_start: u32,
        width: u32,
        resolution: u32,
        sea_level: u32,
        water_voxel: VoxelId,
        profile: PlanetProfile,
        heights: Vec<u32>,
    ) -> Self {
        debug_assert_eq!(heights.len(), (width * width) as usize);
        Self {
            face,
            u_start,
            v_start,
            width,
            resolution,
            sea_level,
            water_voxel,
            profile,
            heights,
        }
    }

    /// Surface height at absolute (u, v). Returns 0 for out-of-range coords.
    pub fn surface_height(&self, u: u32, v: u32) -> u32 {
        let ou = u.wrapping_sub(self.u_start);
        let ov = v.wrapping_sub(self.v_start);
        if ou >= self.width || ov >= self.width {
            return 0;
        }
        self.heights[(ou * self.width + ov) as usize]
    }
}

/// Everything the voxel mesher needs for one chunk.  No world references.
pub struct ChunkMeshInput {
    pub key: SurfaceChunkKey,
    pub voxels: ChunkVoxelView,
    pub border_samples: ChunkBorderSamples,
    pub material_table: Arc<MeshMaterialTable>,
    pub prop_instances: Vec<PropMeshInstance>,
}
