use glam::Vec3;
use vv_voxel::VoxelCoord;

#[derive(Clone, Copy, Debug)]
pub struct WorldStreamingConfig {
    pub lod_near_radius: f32,
    pub lod_split_curve: LodSplitCurve,
    pub lod_hysteresis: f32,
    pub lod_transition_time: f32,
    pub max_visible_voxel_chunks: usize,
    pub max_visible_lod_tiles: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct LodSplitCurve {
    pub far_factor: f32,
    pub mid_factor: f32,
    pub near_factor: f32,
    pub voxel_factor: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct StreamingView {
    pub player_pos: Vec3,
    pub camera_pos: Vec3,
    pub view_dir: Vec3,
    pub cursor_id: Option<VoxelCoord>,
}

impl WorldStreamingConfig {
    pub fn split_factor(self, node_size_chunks: u32) -> f32 {
        match node_size_chunks {
            0 | 1 => self.lod_split_curve.voxel_factor,
            2 => self.lod_split_curve.near_factor,
            3..=4 => self.lod_split_curve.mid_factor,
            _ => self.lod_split_curve.far_factor,
        }
    }

    pub fn split_distance(self, node_radius_world: f32, node_size_chunks: u32) -> f32 {
        let curve_distance = node_radius_world * self.split_factor(node_size_chunks);
        curve_distance.max(self.lod_near_radius)
    }
}

impl Default for WorldStreamingConfig {
    fn default() -> Self {
        Self {
            lod_near_radius: 96.0,
            lod_split_curve: LodSplitCurve {
                far_factor: 4.0,
                mid_factor: 7.0,
                near_factor: 12.0,
                voxel_factor: 18.0,
            },
            lod_hysteresis: 0.15,
            lod_transition_time: 1.2,
            max_visible_voxel_chunks: 384,
            max_visible_lod_tiles: 768,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LodSplitCurve, WorldStreamingConfig};

    #[test]
    fn split_distance_uses_curve_and_near_floor() {
        let config = WorldStreamingConfig {
            lod_near_radius: 50.0,
            lod_split_curve: LodSplitCurve {
                far_factor: 2.0,
                mid_factor: 4.0,
                near_factor: 6.0,
                voxel_factor: 8.0,
            },
            ..Default::default()
        };

        assert_eq!(config.split_distance(10.0, 8), 50.0);
        assert_eq!(config.split_distance(20.0, 2), 120.0);
        assert_eq!(config.split_factor(1), 8.0);
    }

    #[test]
    fn default_streaming_budget_is_bounded() {
        let config = WorldStreamingConfig::default();
        assert!(config.max_visible_voxel_chunks > 0);
        assert!(config.max_visible_lod_tiles > config.max_visible_voxel_chunks);
        assert!(config.lod_hysteresis > 0.0);
        assert!(config.lod_transition_time > 0.0);
    }
}
