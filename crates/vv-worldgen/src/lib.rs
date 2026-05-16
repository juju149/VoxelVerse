pub mod cave_decoration;
pub mod diagnostics;
pub mod features;
pub mod placement;
pub mod procedural;

pub(crate) mod noise;

pub use diagnostics::{WorldgenStats, WorldgenStatsSnapshot};
pub use features::{bake_for_chunk, ChunkFeatureMap};
pub use procedural::{ProceduralPlanetTerrain, PropOrientation, PropStamp};
