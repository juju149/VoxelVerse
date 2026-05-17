mod block_damage;
mod broken_props;
mod mesh_input_builder;
mod planet;
mod planet_edits;
mod planet_geometry;
mod planet_queries;
mod planet_snapshot;
mod runtime;
mod spawn_resolver;
mod terrain_visual_palette;
mod vox_model;
mod world_time;

// Re-export PlanetProfile from vv-voxel so downstream crates can use vv_world::PlanetProfile.
pub use vv_voxel::PlanetProfile;

pub use block_damage::{BlockDamage, BlockDamageLayer, BlockDamageResult};
pub use broken_props::BrokenPropLayer;
pub use planet::{PlanetData, PlanetDataSources, VoxelEditResult, VoxelRead};
pub use planet_geometry::PlanetGeometry;
pub use planet_snapshot::PlanetSnapshot;
pub use runtime::VoxelRuntime;
pub use terrain_visual_palette::TerrainVisualPalette;
pub use vox_model::{VoxModel, VoxModelRegistry};
pub use world_time::WorldTime;
