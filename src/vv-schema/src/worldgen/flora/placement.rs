use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FloraPlacement {
    pub density_base: f32,

    pub altitude_min_m: Option<f32>,
    pub altitude_max_m: Option<f32>,

    pub slope_max: f32,
    pub near_water_bonus: f32,

    pub cluster_radius_m: f32,
    pub cluster_min: u32,
    pub cluster_max: u32,

    pub min_spacing_m: f32,
    pub surface_offset_m: f32,
}

impl Default for FloraPlacement {
    fn default() -> Self {
        Self {
            density_base: 0.05,
            altitude_min_m: None,
            altitude_max_m: None,
            slope_max: 0.5,
            near_water_bonus: 1.0,
            cluster_radius_m: 3.0,
            cluster_min: 1,
            cluster_max: 1,
            min_spacing_m: 1.0,
            surface_offset_m: 0.0,
        }
    }
}
