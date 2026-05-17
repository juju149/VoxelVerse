use super::config::{
    AtmosphereConfig, CloudConfig, FogConfig, PlanetAtmospherePreset, PostProcessConfig,
    SkyPalette, WaterConfig, WeatherConfig,
};

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
}
