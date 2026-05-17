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

impl Default for AtmosphereConfig {
    fn default() -> Self {
        Self::preset(PlanetAtmospherePreset::Tropical)
    }
}
