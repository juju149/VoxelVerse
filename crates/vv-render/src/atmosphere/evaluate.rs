use super::config::{AtmosphereConfig, WaterConfig};
use crate::quality::QualitySettings;
use vv_world::WorldTime;

/// Vertical band the observer currently occupies. Drives sky/atmosphere
/// blending toward space rendering (Phase 7 of the weather/cosmos roadmap).
/// Phase 0 always reports [`AltitudeBand::Ground`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AltitudeBand {
    #[default]
    Ground,
    Strato,
    Meso,
    Space,
}

#[derive(Clone, Copy, Debug)]
pub struct EvaluatedAtmosphere {
    pub elapsed_seconds: f32,
    pub time_of_day: f32,
    pub sun_dir: glam::Vec3,
    pub sky_horizon: glam::Vec3,
    pub sky_zenith: glam::Vec3,
    pub sun_intensity: f32,
    pub fog_density: f32,
    pub height_fog_strength: f32,
    pub volumetric_fog_strength: f32,
    pub exposure: f32,
    pub cloud_steps: f32,
    pub cloud_density: f32,
    pub cloud_speed: f32,
    pub cloud_coverage: f32,
    pub water: WaterConfig,
    pub altitude_band: AltitudeBand,
}

impl AtmosphereConfig {
    pub fn evaluate(
        self,
        surface_radius: f32,
        time: WorldTime,
        quality: QualitySettings,
    ) -> EvaluatedAtmosphere {
        let elapsed_seconds = time.elapsed_seconds();
        let day_phase = time.day_phase();
        let sun_angle = day_phase * std::f32::consts::TAU;
        let sun_dir = glam::Vec3::new(sun_angle.sin() * 0.55, sun_angle.cos(), 0.30).normalize();
        let sun_elevation = sun_dir.y.clamp(-1.0, 1.0);
        let above_horizon = sun_elevation.max(0.0);
        let dawn_factor = {
            let abs_elev = sun_elevation.abs();
            let ramp_up = (sun_elevation * 6.0 + 0.8).clamp(0.0, 1.0);
            (1.0 - (abs_elev * 5.0).min(1.0)).powi(2) * ramp_up
        };

        let dawn_col = if day_phase < 0.5 {
            self.sky.horizon_dawn
        } else {
            self.sky.horizon_dusk
        };
        let sky_horizon = if sun_elevation >= 0.0 {
            self.sky.horizon_noon.lerp(dawn_col, dawn_factor)
        } else {
            let night_t = ((-sun_elevation - 0.05) * 5.0).clamp(0.0, 1.0);
            dawn_col.lerp(self.sky.horizon_night, night_t)
        };

        let day_t = above_horizon.powf(0.4);
        let dawn_z = dawn_factor * 0.55;
        let sky_zenith = self
            .sky
            .zenith_night
            .lerp(self.sky.zenith_day, day_t)
            .lerp(self.sky.zenith_dawn, dawn_z);

        let cloud_density = if quality.volumetric_clouds {
            self.clouds.volumetric_density
        } else {
            self.clouds.clear_density
        };
        let volumetric_fog_strength = if quality.volumetric_fog {
            self.fog.volumetric_strength
        } else {
            0.0
        };

        EvaluatedAtmosphere {
            elapsed_seconds,
            time_of_day: day_phase,
            sun_dir,
            sky_horizon,
            sky_zenith,
            sun_intensity: above_horizon.powf(0.42).min(1.0),
            fog_density: self.fog.density_scale * self.weather.fog_multiplier
                / surface_radius.max(1.0),
            height_fog_strength: self.fog.height_strength,
            volumetric_fog_strength,
            exposure: self.post_process.exposure,
            cloud_steps: quality.cloud_steps as f32,
            cloud_density,
            cloud_speed: self.clouds.speed,
            cloud_coverage: self.weather.cloud_coverage,
            water: self.water,
            altitude_band: AltitudeBand::Ground,
        }
    }
}
