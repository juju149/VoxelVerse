use crate::quality::QualitySettings;
use vv_world::WorldTime;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanetAtmospherePreset {
    Tropical,
    Desert,
    Frozen,
    Lunar,
    Toxic,
    Alien,
    Oceanic,
}

#[derive(Clone, Copy, Debug)]
pub struct SkyPalette {
    pub horizon_noon: glam::Vec3,
    pub horizon_dawn: glam::Vec3,
    pub horizon_dusk: glam::Vec3,
    pub horizon_night: glam::Vec3,
    pub zenith_day: glam::Vec3,
    pub zenith_dawn: glam::Vec3,
    pub zenith_night: glam::Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct FogConfig {
    pub density_scale: f32,
    pub height_strength: f32,
    pub volumetric_strength: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct CloudConfig {
    pub clear_density: f32,
    pub volumetric_density: f32,
    pub speed: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PostProcessConfig {
    pub exposure: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct WeatherConfig {
    pub cloud_coverage: f32,
    pub fog_multiplier: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct WaterConfig {
    pub fresnel: f32,
    pub specular: f32,
    pub alpha: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct AtmosphereConfig {
    pub preset: PlanetAtmospherePreset,
    pub day_length_seconds: f32,
    pub start_phase: f32,
    pub sky: SkyPalette,
    pub fog: FogConfig,
    pub clouds: CloudConfig,
    pub post_process: PostProcessConfig,
    pub weather: WeatherConfig,
    pub water: WaterConfig,
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
}

impl Default for AtmosphereConfig {
    fn default() -> Self {
        Self::preset(PlanetAtmospherePreset::Tropical)
    }
}

impl AtmosphereConfig {
    pub fn preset(preset: PlanetAtmospherePreset) -> Self {
        match preset {
            PlanetAtmospherePreset::Tropical => Self {
                preset,
                day_length_seconds: 1_200.0,
                start_phase: 0.15,
                sky: SkyPalette {
                    horizon_noon: glam::Vec3::new(0.50, 0.70, 1.00),
                    horizon_dawn: glam::Vec3::new(1.00, 0.52, 0.18),
                    horizon_dusk: glam::Vec3::new(0.88, 0.36, 0.28),
                    horizon_night: glam::Vec3::new(0.055, 0.064, 0.105),
                    zenith_day: glam::Vec3::new(0.08, 0.28, 0.86),
                    zenith_dawn: glam::Vec3::new(0.22, 0.20, 0.48),
                    zenith_night: glam::Vec3::new(0.028, 0.034, 0.072),
                },
                fog: FogConfig {
                    density_scale: 0.75,
                    height_strength: 0.75,
                    volumetric_strength: 1.0,
                },
                clouds: CloudConfig {
                    clear_density: 0.48,
                    volumetric_density: 0.74,
                    speed: 0.018,
                },
                post_process: PostProcessConfig { exposure: 0.82 },
                weather: WeatherConfig {
                    cloud_coverage: 0.58,
                    fog_multiplier: 1.0,
                },
                water: WaterConfig {
                    fresnel: 0.48,
                    specular: 0.72,
                    alpha: 0.72,
                },
            },
            PlanetAtmospherePreset::Desert => Self {
                preset,
                day_length_seconds: 1_000.0,
                start_phase: 0.13,
                sky: SkyPalette {
                    horizon_noon: glam::Vec3::new(0.78, 0.82, 0.96),
                    horizon_dawn: glam::Vec3::new(1.00, 0.58, 0.24),
                    horizon_dusk: glam::Vec3::new(0.95, 0.42, 0.22),
                    horizon_night: glam::Vec3::new(0.070, 0.060, 0.085),
                    zenith_day: glam::Vec3::new(0.20, 0.42, 0.90),
                    zenith_dawn: glam::Vec3::new(0.42, 0.26, 0.40),
                    zenith_night: glam::Vec3::new(0.034, 0.030, 0.060),
                },
                fog: FogConfig {
                    density_scale: 0.52,
                    height_strength: 0.42,
                    volumetric_strength: 0.45,
                },
                clouds: CloudConfig {
                    clear_density: 0.20,
                    volumetric_density: 0.34,
                    speed: 0.014,
                },
                post_process: PostProcessConfig { exposure: 0.88 },
                weather: WeatherConfig {
                    cloud_coverage: 0.24,
                    fog_multiplier: 0.86,
                },
                water: WaterConfig {
                    fresnel: 0.50,
                    specular: 0.78,
                    alpha: 0.68,
                },
            },
            PlanetAtmospherePreset::Frozen => Self {
                preset,
                day_length_seconds: 1_500.0,
                start_phase: 0.18,
                sky: SkyPalette {
                    horizon_noon: glam::Vec3::new(0.66, 0.82, 1.00),
                    horizon_dawn: glam::Vec3::new(0.88, 0.56, 0.50),
                    horizon_dusk: glam::Vec3::new(0.58, 0.42, 0.72),
                    horizon_night: glam::Vec3::new(0.040, 0.060, 0.110),
                    zenith_day: glam::Vec3::new(0.12, 0.32, 0.78),
                    zenith_dawn: glam::Vec3::new(0.24, 0.24, 0.52),
                    zenith_night: glam::Vec3::new(0.020, 0.036, 0.085),
                },
                fog: FogConfig {
                    density_scale: 0.95,
                    height_strength: 0.90,
                    volumetric_strength: 0.88,
                },
                clouds: CloudConfig {
                    clear_density: 0.56,
                    volumetric_density: 0.80,
                    speed: 0.010,
                },
                post_process: PostProcessConfig { exposure: 0.76 },
                weather: WeatherConfig {
                    cloud_coverage: 0.72,
                    fog_multiplier: 1.15,
                },
                water: WaterConfig {
                    fresnel: 0.56,
                    specular: 0.82,
                    alpha: 0.62,
                },
            },
            PlanetAtmospherePreset::Lunar => Self {
                preset,
                day_length_seconds: 2_400.0,
                start_phase: 0.20,
                sky: SkyPalette {
                    horizon_noon: glam::Vec3::new(0.045, 0.050, 0.070),
                    horizon_dawn: glam::Vec3::new(0.16, 0.13, 0.12),
                    horizon_dusk: glam::Vec3::new(0.12, 0.10, 0.16),
                    horizon_night: glam::Vec3::new(0.010, 0.012, 0.020),
                    zenith_day: glam::Vec3::new(0.020, 0.024, 0.040),
                    zenith_dawn: glam::Vec3::new(0.060, 0.052, 0.078),
                    zenith_night: glam::Vec3::new(0.006, 0.008, 0.016),
                },
                fog: FogConfig {
                    density_scale: 0.06,
                    height_strength: 0.08,
                    volumetric_strength: 0.0,
                },
                clouds: CloudConfig {
                    clear_density: 0.0,
                    volumetric_density: 0.0,
                    speed: 0.0,
                },
                post_process: PostProcessConfig { exposure: 1.05 },
                weather: WeatherConfig {
                    cloud_coverage: 0.0,
                    fog_multiplier: 0.2,
                },
                water: WaterConfig {
                    fresnel: 0.40,
                    specular: 0.45,
                    alpha: 0.0,
                },
            },
            PlanetAtmospherePreset::Toxic => Self {
                preset,
                day_length_seconds: 1_100.0,
                start_phase: 0.11,
                sky: SkyPalette {
                    horizon_noon: glam::Vec3::new(0.58, 0.78, 0.46),
                    horizon_dawn: glam::Vec3::new(0.86, 0.70, 0.20),
                    horizon_dusk: glam::Vec3::new(0.72, 0.34, 0.20),
                    horizon_night: glam::Vec3::new(0.035, 0.070, 0.040),
                    zenith_day: glam::Vec3::new(0.16, 0.42, 0.20),
                    zenith_dawn: glam::Vec3::new(0.34, 0.35, 0.16),
                    zenith_night: glam::Vec3::new(0.015, 0.040, 0.026),
                },
                fog: FogConfig {
                    density_scale: 1.10,
                    height_strength: 1.05,
                    volumetric_strength: 1.0,
                },
                clouds: CloudConfig {
                    clear_density: 0.68,
                    volumetric_density: 0.92,
                    speed: 0.020,
                },
                post_process: PostProcessConfig { exposure: 0.70 },
                weather: WeatherConfig {
                    cloud_coverage: 0.86,
                    fog_multiplier: 1.30,
                },
                water: WaterConfig {
                    fresnel: 0.62,
                    specular: 0.48,
                    alpha: 0.76,
                },
            },
            PlanetAtmospherePreset::Alien => Self {
                preset,
                day_length_seconds: 900.0,
                start_phase: 0.08,
                sky: SkyPalette {
                    horizon_noon: glam::Vec3::new(0.58, 0.54, 0.98),
                    horizon_dawn: glam::Vec3::new(0.98, 0.38, 0.70),
                    horizon_dusk: glam::Vec3::new(0.44, 0.28, 0.92),
                    horizon_night: glam::Vec3::new(0.035, 0.030, 0.090),
                    zenith_day: glam::Vec3::new(0.18, 0.12, 0.64),
                    zenith_dawn: glam::Vec3::new(0.40, 0.18, 0.54),
                    zenith_night: glam::Vec3::new(0.020, 0.012, 0.065),
                },
                fog: FogConfig {
                    density_scale: 0.82,
                    height_strength: 0.74,
                    volumetric_strength: 0.92,
                },
                clouds: CloudConfig {
                    clear_density: 0.52,
                    volumetric_density: 0.70,
                    speed: 0.024,
                },
                post_process: PostProcessConfig { exposure: 0.78 },
                weather: WeatherConfig {
                    cloud_coverage: 0.46,
                    fog_multiplier: 1.05,
                },
                water: WaterConfig {
                    fresnel: 0.60,
                    specular: 0.68,
                    alpha: 0.70,
                },
            },
            PlanetAtmospherePreset::Oceanic => Self {
                preset,
                day_length_seconds: 1_300.0,
                start_phase: 0.14,
                sky: SkyPalette {
                    horizon_noon: glam::Vec3::new(0.46, 0.78, 1.00),
                    horizon_dawn: glam::Vec3::new(0.92, 0.54, 0.32),
                    horizon_dusk: glam::Vec3::new(0.56, 0.40, 0.66),
                    horizon_night: glam::Vec3::new(0.030, 0.060, 0.105),
                    zenith_day: glam::Vec3::new(0.04, 0.30, 0.78),
                    zenith_dawn: glam::Vec3::new(0.18, 0.26, 0.50),
                    zenith_night: glam::Vec3::new(0.012, 0.036, 0.075),
                },
                fog: FogConfig {
                    density_scale: 0.88,
                    height_strength: 0.82,
                    volumetric_strength: 0.86,
                },
                clouds: CloudConfig {
                    clear_density: 0.60,
                    volumetric_density: 0.82,
                    speed: 0.022,
                },
                post_process: PostProcessConfig { exposure: 0.80 },
                weather: WeatherConfig {
                    cloud_coverage: 0.68,
                    fog_multiplier: 1.10,
                },
                water: WaterConfig {
                    fresnel: 0.58,
                    specular: 0.86,
                    alpha: 0.74,
                },
            },
        }
    }

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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AtmosphereConfig, PlanetAtmospherePreset};
    use crate::quality::QualitySettings;
    use vv_world::WorldTime;

    #[test]
    fn presets_have_distinct_sky_signatures() {
        let tropical = AtmosphereConfig::preset(PlanetAtmospherePreset::Tropical);
        let lunar = AtmosphereConfig::preset(PlanetAtmospherePreset::Lunar);

        assert_ne!(tropical.sky.zenith_day, lunar.sky.zenith_day);
        assert!(lunar.weather.cloud_coverage < tropical.weather.cloud_coverage);
    }

    #[test]
    fn evaluation_uses_world_time_phase() {
        let config = AtmosphereConfig::default();
        let mut time = WorldTime::new(config.day_length_seconds, 0.25);
        time.tick(0.0);

        let evaluated = config.evaluate(500.0, time, QualitySettings::default());

        assert!((evaluated.time_of_day - 0.25).abs() < 0.0001);
        assert!(evaluated.sun_dir.x > 0.0);
        assert!(evaluated.fog_density > 0.0);
    }
}
