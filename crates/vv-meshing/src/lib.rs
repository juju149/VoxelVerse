mod ambient_occlusion;
mod config;
mod cpu_mesh;
mod greedy_mesher;
mod lod_input;
mod lod_mesher;
mod primitives;
pub mod scheduler;
pub mod voxel_mesher;

pub use config::VoxelMeshingConfig;
pub use cpu_mesh::{CpuMesh, CpuVertex};
pub use lod_input::{LodCellColors, LodMeshInput};
pub use scheduler::{MeshScheduler, SchedulerBudget, SchedulerStats, UploadBudgetState};
pub use voxel_mesher::chunk_input::{ChunkBorderSamples, ChunkMeshInput, ChunkVoxelView};
pub use voxel_mesher::material_packing::{
    FaceEdgeMask, MeshMaterialEntry, MeshMaterialTable, VoxelMeshClass, VoxelMeshKind,
    VoxelVisual, VoxelVisualLayers, pack_material_edges,
};
pub use voxel_mesher::prop_integration::{
    BakedPropFace, PropMeshInstance, PropMeshModel, PropSurfaceOrientation,
};
pub use voxel_mesher::MeshGen;

/// Material sentinel for geometry whose albedo is already baked into vertex color.
pub const VERTEX_COLOR_MATERIAL_SENTINEL: u32 = 0x0000_FFFF;
