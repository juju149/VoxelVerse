use vv_registry::CompiledFloraFeature;

use super::types::TerrainFlora;

pub(crate) fn max_feature_height_m(flora: &[TerrainFlora]) -> f32 {
    flora
        .iter()
        .map(|flora| match flora.data.feature {
            CompiledFloraFeature::Plant { height_max_m, .. } => height_max_m,
            CompiledFloraFeature::Tree {
                trunk_height_max_m,
                canopy_height_m,
                ..
            } => trunk_height_max_m + canopy_height_m * 2.0 + 1.0,
            CompiledFloraFeature::Cluster { radius_max_m, .. } => radius_max_m,
        })
        .fold(1.0, f32::max)
}

pub(crate) fn max_feature_radius_m(flora: &[TerrainFlora]) -> f32 {
    flora
        .iter()
        .map(|flora| match flora.data.feature {
            CompiledFloraFeature::Plant { .. } => 0.0,
            CompiledFloraFeature::Tree {
                canopy_radius_m, ..
            } => canopy_radius_m * 1.6 + 1.0,
            CompiledFloraFeature::Cluster { radius_max_m, .. } => radius_max_m,
        })
        .fold(0.0, f32::max)
}
