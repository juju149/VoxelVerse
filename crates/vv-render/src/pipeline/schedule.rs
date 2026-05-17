#[derive(Clone, Copy, Debug)]
pub(crate) struct RenderScheduleInputs {
    pub clouds_enabled: bool,
    pub cloud_density: f32,
    pub volumetric_fog_enabled: bool,
    pub volumetric_fog_strength: f32,
    pub precipitation_intensity: f32,
    pub celestial_params: [f32; 4],
    pub celestial_moon: [f32; 4],
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderFramePasses {
    pub shadow_depth: bool,
    pub sky: bool,
    pub celestial: bool,
    pub clouds: bool,
    pub terrain_opaque: bool,
    pub volumetric_fog: bool,
    pub precipitation: bool,
    pub final_composite: bool,
    pub ui: bool,
}

impl RenderFramePasses {
    pub(crate) fn from_inputs(input: RenderScheduleInputs) -> Self {
        let celestial = input.celestial_params[0] > 0.001
            || input.celestial_params[1] > 0.001
            || input.celestial_params[2] > 0.001
            || input.celestial_params[3] > 0.0
            || input.celestial_moon[3] > 0.0;

        Self {
            shadow_depth: true,
            sky: true,
            celestial,
            clouds: input.clouds_enabled && input.cloud_density > 0.001,
            terrain_opaque: true,
            volumetric_fog: input.volumetric_fog_enabled && input.volumetric_fog_strength > 0.001,
            precipitation: input.precipitation_intensity > 0.001,
            final_composite: true,
            ui: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderFramePasses, RenderScheduleInputs};

    fn empty_inputs() -> RenderScheduleInputs {
        RenderScheduleInputs {
            clouds_enabled: false,
            cloud_density: 0.0,
            volumetric_fog_enabled: false,
            volumetric_fog_strength: 0.0,
            precipitation_intensity: 0.0,
            celestial_params: [0.0; 4],
            celestial_moon: [0.0; 4],
        }
    }

    #[test]
    fn baseline_only_draws_core_passes() {
        let passes = RenderFramePasses::from_inputs(empty_inputs());

        assert!(passes.shadow_depth);
        assert!(passes.sky);
        assert!(passes.terrain_opaque);
        assert!(passes.final_composite);
        assert!(passes.ui);

        assert!(!passes.celestial);
        assert!(!passes.clouds);
        assert!(!passes.volumetric_fog);
        assert!(!passes.precipitation);
    }

    #[test]
    fn optional_passes_enable_only_when_inputs_are_active() {
        let passes = RenderFramePasses::from_inputs(RenderScheduleInputs {
            clouds_enabled: true,
            cloud_density: 0.5,
            volumetric_fog_enabled: true,
            volumetric_fog_strength: 0.4,
            precipitation_intensity: 0.8,
            celestial_params: [0.0, 1.0, 0.0, 0.0],
            celestial_moon: [0.0; 4],
        });

        assert!(passes.celestial);
        assert!(passes.clouds);
        assert!(passes.volumetric_fog);
        assert!(passes.precipitation);
    }

    #[test]
    fn clouds_stay_off_when_feature_is_disabled() {
        let passes = RenderFramePasses::from_inputs(RenderScheduleInputs {
            clouds_enabled: false,
            cloud_density: 1.0,
            ..empty_inputs()
        });

        assert!(!passes.clouds);
    }
}
