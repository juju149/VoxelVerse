mod ambient_occlusion;
mod cpu_mesh;
mod debug_mesh;
mod lod_mesher;
mod primitives;
pub(crate) mod prop_baker;
mod rounded_edges;
mod voxel_mesher;

pub use cpu_mesh::{CpuMesh, CpuVertex};
pub use rounded_edges::{pack_material_edges, FaceEdgeMask};

pub struct MeshGen;
