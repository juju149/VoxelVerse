use serde::Deserialize;

/// Raw biome definition as loaded from a `.ron` data file.
/// Path-as-identity: `packs/core/biomes/plains.ron` → key `"core:plains"`.
#[derive(Debug, Deserialize)]
pub struct RawBiomeDef {
    pub display_name: String,
    /// Namespaced key of the block placed on the top surface layer (e.g. `"core:grass"`).
    pub surface_block: String,
    /// Namespaced key of the block placed below the surface (e.g. `"core:dirt"`).
    pub subsurface_block: String,
    /// Climate temperature center: 0.0 = arctic, 1.0 = tropical.
    /// Derived from latitude (equator = 1.0, poles = 0.0) + noise jitter.
    pub temperature_center: f32,
    /// Terrain roughness center: 0.0 = flat, 1.0 = very mountainous.
    /// Driven by a large-scale noise field independent of latitude.
    pub roughness_center: f32,
    /// Overall terrain amplitude multiplier (0.0 = completely flat, 1.0 = full planet amplitude).
    pub terrain_amplitude: f32,
    /// Flatness bias: 0.0 = natural, 1.0 = forces terrain fully toward sea level.
    pub terrain_flatness: f32,
}
