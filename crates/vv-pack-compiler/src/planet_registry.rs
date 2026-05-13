/// A compiled planet definition ready for runtime use.
#[derive(Clone, Debug)]
pub struct CompiledPlanet {
    pub key: String,
    pub display_name: String,
    pub seed: u32,
    pub resolution: u32,
    pub surface_layer: u32,
    pub voxel_size_meters: f32,
    pub edge_rounding_radius_voxels: f32,
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

    /// Build a [`vv_voxel::PlanetProfile`] from this compiled planet definition.
    pub fn to_planet_profile(&self) -> vv_voxel::PlanetProfile {
        debug_assert!(self.voxel_size_meters.is_finite() && self.voxel_size_meters > 0.0);
        let voxel_size_meters = self.voxel_size_meters;
        let scale = (1.0_f32 / voxel_size_meters.max(0.0001)).max(1.0);
        let resolution = ((self.resolution as f32) * scale).round() as u32;
        let resolution = resolution.max(8);
        let surface_layer = ((self.surface_layer as f32) * scale).round() as u32;
        let surface_layer = surface_layer.clamp(4, resolution - 1);
        let core_layers_raw = ((self.core_layers as f32) * scale).round() as u32;
        let core_layers = core_layers_raw.min(surface_layer.saturating_sub(1)).max(1);
        let max_terrain_offset = ((self.max_terrain_offset as f32) * scale).round() as i32;

        let fraction = self.inner_radius_fraction.clamp(0.02, 0.95);
        let shell_depth = surface_layer.max(1) as f32 * voxel_size_meters;
        let surface_radius = shell_depth / (1.0 - fraction);
        let inner_radius = surface_radius * fraction;
        let layer_height = voxel_size_meters;

        vv_voxel::PlanetProfile {
            resolution,
            surface_layer,
            core_layers,
            voxel_size_meters,
            edge_rounding_radius_voxels: self.edge_rounding_radius_voxels,
            inner_radius,
            surface_radius,
            layer_height,
            max_terrain_offset,
            spawn_clearance_layers: self.spawn_clearance_layers,
            seed: self.seed,
        }
    }
}
