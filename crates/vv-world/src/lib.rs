mod block_damage;
mod broken_props;
mod planet;
mod planet_snapshot;
mod runtime;
mod terrain_visual_palette;
mod vox_model;
mod world_time;

// Re-export PlanetProfile from vv-voxel so downstream crates can use vv_world::PlanetProfile.
pub use vv_voxel::PlanetProfile;

pub use block_damage::{BlockDamage, BlockDamageLayer, BlockDamageResult};
pub use broken_props::BrokenPropLayer;
pub use planet::{PlanetData, VoxelEditResult, VoxelRead};
pub use planet_snapshot::PlanetSnapshot;
pub use runtime::VoxelRuntime;
pub use terrain_visual_palette::TerrainVisualPalette;
pub use vox_model::{VoxModel, VoxModelRegistry};
pub use world_time::WorldTime;
