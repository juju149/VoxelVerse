#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VoxelMeshingConfig {
    pub prop_lod_chunk_radius: u32,
    pub cliff_fill_depth: u32,
    pub max_prop_faces_per_stamp: usize,
    pub max_prop_quads_per_chunk: usize,
}

impl Default for VoxelMeshingConfig {
    fn default() -> Self {
        Self {
            prop_lod_chunk_radius: 5,
            cliff_fill_depth: 20,
            max_prop_faces_per_stamp: 256,
            max_prop_quads_per_chunk: 2048,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VoxelMeshingConfig;

    #[test]
    fn default_meshing_config_keeps_v1_budgets_bounded() {
        let config = VoxelMeshingConfig::default();

        assert_eq!(config.prop_lod_chunk_radius, 5);
        assert_eq!(config.cliff_fill_depth, 20);
        assert!(config.max_prop_faces_per_stamp <= 256);
        assert!(config.max_prop_quads_per_chunk <= 2048);
    }
}
