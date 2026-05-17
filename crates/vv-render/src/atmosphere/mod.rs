//! Atmosphere configuration, presets, and per-frame evaluation.
//!
//! Splitting follows `docs/v1/13_WEATHER_AND_COSMOS.md` §2.2:
//! - [`config`]: data types (`AtmosphereConfig`, palettes, sub-configs).
//! - [`evaluate`]: per-frame `EvaluatedAtmosphere` output + `AltitudeBand`.
//! - `presets`: built-in `PlanetAtmospherePreset` table (private impl).
//! - `weather_blend` / `celestial_blend`: future overlay stages, populated
//!   in Phases 2–5 of the weather/cosmos roadmap.

pub mod config;
pub mod evaluate;

mod celestial_blend;
mod presets;
mod weather_blend;

pub use config::{AtmosphereConfig, PlanetAtmospherePreset};

#[cfg(test)]
mod tests {
    use super::evaluate::AltitudeBand;
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
        assert_eq!(evaluated.altitude_band, AltitudeBand::Ground);
    }
}
