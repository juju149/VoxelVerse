/// vv-core must stay tiny.
/// Voxel-specific concepts moved to vv-voxel.
///
/// Temporary compatibility re-exports.
/// Remove these after all crates import vv-voxel directly.
pub use vv_voxel::{BlockId, ChunkKey, LodKey, VoxelId, CHUNK_SIZE};
