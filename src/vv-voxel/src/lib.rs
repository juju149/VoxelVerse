/// Chunk size: number of voxels per chunk edge on a face.
/// This belongs to the voxel runtime layer, not vv-core.
pub const CHUNK_SIZE: u32 = 32;

/// Identifies a single voxel on a planet face.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct VoxelId {
    /// Cube-sphere face index.
    pub face: u8,
    /// Radial layer from planet centre outward.
    pub layer: u32,
    /// U coordinate on the face grid.
    pub u: u32,
    /// V coordinate on the face grid.
    pub v: u32,
}

/// Temporary compatibility alias.
/// Remove once every crate uses VoxelId.
pub type BlockId = VoxelId;

/// Identifies a streaming chunk on a planet face.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct ChunkKey {
    pub face: u8,
    pub u_idx: u32,
    pub v_idx: u32,
}

impl ChunkKey {
    pub fn from_voxel(id: VoxelId) -> Self {
        Self {
            face: id.face,
            u_idx: id.u / CHUNK_SIZE,
            v_idx: id.v / CHUNK_SIZE,
        }
    }
}

/// Identifies an LOD tile on a planet face.
/// Temporary home until LOD ownership is cleaned up.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct LodKey {
    pub face: u8,
    pub x: u32,
    pub y: u32,
    pub size: u32,
}
