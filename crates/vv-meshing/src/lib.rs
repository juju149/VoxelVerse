mod ambient_occlusion;
mod cpu_mesh;
mod debug_mesh;
mod greedy_mesher;
mod lod_mesher;
mod primitives;
pub(crate) mod prop_baker;
mod rounded_edges;
pub mod scheduler;
mod voxel_mesher;

pub use cpu_mesh::{CpuMesh, CpuVertex};
pub(crate) use greedy_mesher::GreedyMesher;
pub use rounded_edges::{pack_material_edges, FaceEdgeMask};
pub use scheduler::{MeshScheduler, SchedulerBudget, SchedulerStats, UploadBudgetState};

/// Material sentinel for geometry whose albedo is already baked into vertex color.
pub const VERTEX_COLOR_MATERIAL_SENTINEL: u32 = 0x0000_FFFF;

pub struct MeshGen;
