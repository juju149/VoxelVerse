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
    use vv_celestial::{AltitudeBand as CelestialAltitudeBand, CelestialState};
    use vv_weather::{
        LightningStrike, PrecipitationKindSample, PrecipitationSample, WeatherProfileId,
        WeatherState, WindVector,
    };
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

    fn storm_state() -> WeatherState {
        WeatherState {
            current: WeatherProfileId(0),
            next: None,
            blend: 0.0,
            cloud_coverage: 0.95,
            fog_multiplier: 1.30,
            cloud_density_mul: 1.5,
            cloud_speed_mul: 2.5,
            wind: WindVector::default(),
            precipitation: PrecipitationSample {
                kind: PrecipitationKindSample::Rain,
                intensity: 0.85,
                wind_drift: 0.6,
                splash_density: 0.7,
            },
            lightning_events: Vec::new(),
        }
    }

    #[test]
    fn apply_weather_overrides_coverage_and_scales_density_and_fog() {
        let config = AtmosphereConfig::default();
        let mut time = WorldTime::new(config.day_length_seconds, 0.5);
        time.tick(0.0);
        let base = config.evaluate(500.0, time, QualitySettings::default());

        let mut evaluated = base;
        evaluated.apply_weather(&storm_state());

        // Coverage replaced.
        assert!((evaluated.cloud_coverage - 0.95).abs() < 1e-5);
        // Density multiplied.
        assert!((evaluated.cloud_density - base.cloud_density * 1.5).abs() < 1e-5);
        // Speed multiplied.
        assert!((evaluated.cloud_speed - base.cloud_speed * 2.5).abs() < 1e-5);
        // Fog multiplied.
        assert!((evaluated.fog_density - base.fog_density * 1.30).abs() < 1e-5);
        // Sun untouched without strikes.
        assert!((evaluated.sun_intensity - base.sun_intensity).abs() < 1e-5);
    }

    #[test]
    fn apply_weather_lightning_boosts_sun_intensity_within_cap() {
        let config = AtmosphereConfig::default();
        let mut time = WorldTime::new(config.day_length_seconds, 0.5);
        time.tick(0.0);
        let base = config.evaluate(500.0, time, QualitySettings::default());

        let mut state = storm_state();
        state.lightning_events.push(LightningStrike {
            position: glam::Vec3::ZERO,
            distance_m: 100.0,
            flash_intensity: 4.0,
            thunder_delay_s: 0.3,
        });
        let mut evaluated = base;
        evaluated.apply_weather(&state);
        assert!(
            evaluated.sun_intensity > base.sun_intensity,
            "lightning must boost sun_intensity"
        );

        // Cap test: spamming strikes can never exceed 1.5.
        let mut state = storm_state();
        for _ in 0..50 {
            state.lightning_events.push(LightningStrike {
                position: glam::Vec3::ZERO,
                distance_m: 100.0,
                flash_intensity: 10.0,
                thunder_delay_s: 0.3,
            });
        }
        let mut evaluated = base;
        evaluated.apply_weather(&state);
        assert!(evaluated.sun_intensity <= 1.5 + 1e-5);
    }

    fn celestial_state(sun_dir: glam::Vec3, eclipse: f32, band: CelestialAltitudeBand) -> CelestialState {
        CelestialState {
            sun_dir_world: sun_dir,
            sun_disc_color: glam::Vec3::new(1.0, 0.95, 0.85),
            sun_disc_angular_radius: 0.0046,
            sun_distance_m: 1.496e11,
            moons: Vec::new(),
            stars_visibility: 0.0,
            aurora_intensity: 0.0,
            eclipse_factor: eclipse,
            altitude_band: band,
        }
    }

    #[test]
    fn apply_celestial_overrides_sun_dir() {
        let config = AtmosphereConfig::default();
        let mut time = WorldTime::new(config.day_length_seconds, 0.25);
        time.tick(0.0);
        let mut evaluated = config.evaluate(500.0, time, QualitySettings::default());
        let new_dir = glam::Vec3::new(0.3, 0.9, 0.1).normalize();
        evaluated.apply_celestial(&celestial_state(new_dir, 0.0, CelestialAltitudeBand::Ground));
        assert!((evaluated.sun_dir - new_dir).length() < 1e-5);
    }

    #[test]
    fn apply_celestial_eclipse_dims_sun() {
        let config = AtmosphereConfig::default();
        let mut time = WorldTime::new(config.day_length_seconds, 0.25);
        time.tick(0.0);
        let mut evaluated = config.evaluate(500.0, time, QualitySettings::default());
        let new_dir = glam::Vec3::new(0.0, 1.0, 0.0);
        let before = {
            let mut clear = evaluated;
            clear.apply_celestial(&celestial_state(new_dir, 0.0, CelestialAltitudeBand::Ground));
            clear.sun_intensity
        };
        evaluated.apply_celestial(&celestial_state(new_dir, 1.0, CelestialAltitudeBand::Ground));
        assert!(
            evaluated.sun_intensity < before * 0.2,
            "totality must crush sun_intensity (got {} vs base {before})",
            evaluated.sun_intensity
        );
    }

    #[test]
    fn apply_celestial_propagates_altitude_band() {
        let config = AtmosphereConfig::default();
        let mut time = WorldTime::new(config.day_length_seconds, 0.25);
        time.tick(0.0);
        let mut evaluated = config.evaluate(500.0, time, QualitySettings::default());
        evaluated.apply_celestial(&celestial_state(
            glam::Vec3::Y,
            0.0,
            CelestialAltitudeBand::Space,
        ));
        assert_eq!(evaluated.altitude_band, AltitudeBand::Space);
    }
}
