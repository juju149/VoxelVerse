/// A compiled planet definition ready for runtime use.
#[derive(Clone, Debug)]
pub struct CompiledPlanet {
    pub key: String,
    pub display_name: String,
    pub seed: u32,
    pub resolution: u32,
    pub surface_layer: u32,
    pub voxel_size_meters: f32,
    pub core_layers: u32,
    pub inner_radius_fraction: f32,
    pub max_terrain_offset: i32,
    pub spawn_clearance_layers: f32,
}

impl CompiledPlanet {
    pub fn with_resolution(&self, resolution: u32) -> Self {
        let mut next = self.clone();
        next.resolution = resolution;
        next.surface_layer = resolution / 2;
        next.core_layers = self
            .core_layers
            .min(next.surface_layer.saturating_sub(2))
            .max(2);
        next
    }
}
