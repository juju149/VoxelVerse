/// Chunk size: number of voxels per chunk edge on a face.
/// Changing this value alters chunk granularity and streaming overhead.
pub const CHUNK_SIZE: u32 = 32;

/// Identifies a single voxel on a planet face.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct BlockId {
    /// Cube-sphere face index (0–5).
    pub face: u8,
    /// Radial layer from planet centre outward.
    pub layer: u32,
    /// U coordinate on the face grid.
    pub u: u32,
    /// V coordinate on the face grid.
    pub v: u32,
}

/// Identifies a streaming chunk on a planet face.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct ChunkKey {
    pub face: u8,
    pub u_idx: u32,
    pub v_idx: u32,
}

/// Identifies an LOD tile on a planet face.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct LodKey {
    pub face: u8,
    /// Top-left U coordinate of the tile in face-voxel space.
    pub x: u32,
    /// Top-left V coordinate of the tile in face-voxel space.
    pub y: u32,
    /// Side length of the tile in face-voxel units.
    pub size: u32,
}
