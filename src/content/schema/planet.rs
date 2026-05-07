use serde::Deserialize;

/// Raw planet definition as loaded from a `.ron` data file.
/// Path-as-identity: `packs/core/planets/default.ron` -> key `"core:default"`.
#[derive(Debug, Clone, Deserialize)]
pub struct RawPlanetDef {
    pub display_name: String,
    pub seed: u32,
    /// Full cube-face resolution in voxels per axis.
    pub resolution: u32,
    /// Surface layer in voxel space. If omitted, defaults to `resolution / 2`.
    #[serde(default)]
    pub surface_layer: Option<u32>,
    /// Number of protected core layers.
    pub core_layers: u32,
    /// Inner radius as a fraction of surface radius.
    pub inner_radius_fraction: f32,
    /// Maximum terrain offset in voxel layers.
    pub max_terrain_offset: i32,
    /// Spawn clearance in voxel layers above terrain.
    pub spawn_clearance_layers: f32,
}
