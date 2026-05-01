#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainGenerationError {
    MissingDefaultPlanetType,
    MissingPlanetType(vv_registry::PlanetTypeId),
    NoBiomes,
    BiomeWithoutSurfaceLayer,
}

impl std::fmt::Display for TerrainGenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDefaultPlanetType => {
                write!(f, "worldgen content has no default planet type")
            }
            Self::MissingPlanetType(id) => {
                write!(
                    f,
                    "worldgen content references missing planet type {:?}",
                    id
                )
            }
            Self::NoBiomes => write!(f, "worldgen content has no biomes"),
            Self::BiomeWithoutSurfaceLayer => {
                write!(f, "worldgen biome has no surface layers")
            }
        }
    }
}

impl std::error::Error for TerrainGenerationError {}
