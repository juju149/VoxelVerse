#[derive(Debug, Clone, Copy)]
pub struct CompiledWorldSettings {
    pub chunk_size: u32,
    pub render_distance_chunks: u32,
    pub max_planet_radius_km: f32,
    pub voxel_size_m: f32,
}

impl Default for CompiledWorldSettings {
    fn default() -> Self {
        Self {
            chunk_size: 32,
            render_distance_chunks: 12,
            max_planet_radius_km: 1000.0,
            voxel_size_m: 0.5,
        }
    }
}
