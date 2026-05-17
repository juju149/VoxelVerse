//! Integration tests for the weather solver.
//!
//! Verifies determinism, transition mechanics, biome weighting, and lightning
//! event delivery using a hand-built registry (no pack on disk).

use vv_content_schema::{
    RawPrecipitationKind, RawWeatherLightningDef, RawWeatherPostFxDef, RawWeatherPrecipitationDef,
    RawWeatherProfileDef, RawWeatherTransitionsDef, RawWeatherWindDef,
};
use vv_weather::{PrecipitationKindSample, WeatherRegistry, WeatherSimState};

fn make_profile(
    name: &str,
    rarity: f32,
    precip: Option<RawPrecipitationKind>,
    biome_bias: &[(&str, f32)],
    min_dur: f32,
    max_dur: f32,
) -> RawWeatherProfileDef {
    let bias = biome_bias
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect();
    RawWeatherProfileDef {
        display_name: name.to_string(),
        rarity,
        biome_bias: bias,
        cloud_coverage: 0.5,
        cloud_density_mul: 1.0,
        cloud_speed_mul: 1.0,
        cloud_tint: None,
        fog_multiplier: 1.0,
        fog_tint: None,
        precipitation: precip.map(|kind| RawWeatherPrecipitationDef {
            kind,
            intensity: 0.6,
            wind_drift: 0.4,
            splash_density: 0.5,
            sound: None,
        }),
        wind: RawWeatherWindDef {
            base_speed: 5.0,
            gust_speed: 10.0,
            gust_interval_s: 8.0,
            direction_drift_per_s: 0.02,
        },
        lightning: None,
        post_fx: RawWeatherPostFxDef {
            exposure_mul: 1.0,
            saturation_mul: 1.0,
            contrast_add: 0.0,
        },
        transitions: RawWeatherTransitionsDef {
            fade_in_s: 5.0,
            fade_out_s: 5.0,
            min_duration_s: min_dur,
            max_duration_s: max_dur,
        },
    }
}

fn registry_three() -> WeatherRegistry {
    WeatherRegistry::from_raw(&[
        (
            "core:weather/clear".to_string(),
            make_profile("Clear", 0.6, None, &[], 30.0, 90.0),
        ),
        (
            "core:weather/rain".to_string(),
            make_profile(
                "Rain",
                0.3,
                Some(RawPrecipitationKind::Rain),
                &[("desert", 0.0)],
                30.0,
                90.0,
            ),
        ),
        (
            "core:weather/snow".to_string(),
            make_profile(
                "Snow",
                0.1,
                Some(RawPrecipitationKind::Snow),
                &[("polar", 4.0), ("desert", 0.0)],
                30.0,
                90.0,
            ),
        ),
    ])
}

#[test]
fn registry_indexes_profiles_in_order() {
    let reg = registry_three();
    assert_eq!(reg.len(), 3);
    assert_eq!(reg.id_of("clear").map(|id| id.0), Some(0));
    assert_eq!(reg.id_of("rain").map(|id| id.0), Some(1));
    assert_eq!(reg.id_of("snow").map(|id| id.0), Some(2));
}

#[test]
fn initial_pick_is_highest_weight_in_biome() {
    let reg = registry_three();
    // In a polar biome, snow has rarity 0.1 × bias 4 = 0.4 vs clear 0.6 × 1 = 0.6.
    // Clear still wins.
    let sim = WeatherSimState::new(&reg, 1, "polar");
    assert_eq!(sim.current_id(), reg.id_of("clear").unwrap());
}

#[test]
fn same_seed_is_deterministic() {
    let reg = registry_three();
    let mut a = WeatherSimState::new(&reg, 0xDEAD_BEEF, "plains");
    let mut b = WeatherSimState::new(&reg, 0xDEAD_BEEF, "plains");
    for _ in 0..600 {
        a.tick(0.1, &reg, glam::Vec3::ZERO);
        b.tick(0.1, &reg, glam::Vec3::ZERO);
    }
    let snap_a = a.snapshot(&reg);
    let snap_b = b.snapshot(&reg);
    assert_eq!(snap_a.current, snap_b.current);
    assert_eq!(snap_a.next, snap_b.next);
    assert!((snap_a.blend - snap_b.blend).abs() < 1e-6);
    assert!((snap_a.wind.speed - snap_b.wind.speed).abs() < 1e-4);
}

#[test]
fn weather_changes_within_a_few_minutes() {
    let reg = registry_three();
    let mut sim = WeatherSimState::new(&reg, 42, "plains");
    let initial = sim.current_id();

    // Tick at 60 Hz for 5 simulated minutes — must transition at least once.
    let mut transitioned_or_swapped = false;
    for _ in 0..(60 * 60 * 5) {
        sim.tick(1.0 / 60.0, &reg, glam::Vec3::ZERO);
        if sim.is_transitioning() || sim.current_id() != initial {
            transitioned_or_swapped = true;
            break;
        }
    }
    assert!(
        transitioned_or_swapped,
        "expected a transition within 5 simulated minutes"
    );
}

#[test]
fn biome_bias_zero_excludes_profile() {
    // In a desert, snow has bias 0 and rain has bias 0 → only clear remains.
    let reg = WeatherRegistry::from_raw(&[
        (
            "core:weather/clear".to_string(),
            make_profile("Clear", 0.6, None, &[], 5.0, 10.0),
        ),
        (
            "core:weather/rain".to_string(),
            make_profile(
                "Rain",
                0.3,
                Some(RawPrecipitationKind::Rain),
                &[("desert", 0.0)],
                5.0,
                10.0,
            ),
        ),
        (
            "core:weather/snow".to_string(),
            make_profile(
                "Snow",
                0.1,
                Some(RawPrecipitationKind::Snow),
                &[("desert", 0.0)],
                5.0,
                10.0,
            ),
        ),
    ]);
    let mut sim = WeatherSimState::new(&reg, 7, "desert");
    // Drive long enough that the solver had many chances to pick.
    for _ in 0..(60 * 60 * 2) {
        sim.tick(1.0 / 60.0, &reg, glam::Vec3::ZERO);
        // The only candidates with non-zero weight in desert biomes other
        // than `clear` itself are rain (0) and snow (0). So no transition
        // should ever be initiated.
        assert!(
            !sim.is_transitioning(),
            "desert biome must never transition to rain/snow"
        );
        assert_eq!(sim.current_id(), reg.id_of("clear").unwrap());
    }
}

#[test]
fn transition_fades_blend_zero_to_one() {
    let reg = registry_three();
    let mut sim = WeatherSimState::new(&reg, 99, "plains");
    // Force the second profile so we deterministically observe a fade.
    let rain_id = reg.id_of("rain").unwrap();
    sim.force(rain_id);

    // Manually trigger a switch by ticking long enough to exit min_duration,
    // then poll until a transition is in flight.
    for _ in 0..(60 * 120) {
        sim.tick(1.0 / 60.0, &reg, glam::Vec3::ZERO);
        if sim.is_transitioning() {
            break;
        }
    }
    assert!(sim.is_transitioning(), "expected a transition by now");

    // Sample the blend while it climbs.
    let snap0 = sim.snapshot(&reg);
    assert!((0.0..=1.0).contains(&snap0.blend));
    let blend0 = snap0.blend;
    for _ in 0..30 {
        sim.tick(1.0 / 60.0, &reg, glam::Vec3::ZERO);
    }
    let snap1 = sim.snapshot(&reg);
    assert!(
        snap1.blend > blend0 || !sim.is_transitioning(),
        "blend must increase during a transition (was {blend0}, now {})",
        snap1.blend
    );
}

#[test]
fn lightning_strikes_are_emitted_when_configured() {
    // A profile with an extreme strike rate so we hit at least one within
    // a 10-second window.
    let mut profile = make_profile(
        "Storm",
        1.0,
        Some(RawPrecipitationKind::Rain),
        &[],
        1.0,
        2.0,
    );
    profile.lightning = Some(RawWeatherLightningDef {
        strikes_per_minute: 120.0, // 2 / s
        flash_intensity: 3.0,
        thunder_delay_per_km: 3.0,
        sound: None,
    });
    let reg = WeatherRegistry::from_raw(&[("core:weather/storm".to_string(), profile)]);
    let mut sim = WeatherSimState::new(&reg, 0xAA55, "plains");

    let mut got_strike = false;
    for _ in 0..(60 * 10) {
        sim.tick(1.0 / 60.0, &reg, glam::Vec3::ZERO);
        let snap = sim.snapshot(&reg);
        if !snap.lightning_events.is_empty() {
            for s in &snap.lightning_events {
                assert!(s.distance_m >= 0.0);
                assert!(s.thunder_delay_s >= 0.0);
                assert!(s.flash_intensity > 0.0);
            }
            got_strike = true;
            break;
        }
    }
    assert!(got_strike, "expected at least one strike in 10s at 120 spm");
}

#[test]
fn snapshot_precipitation_matches_current_profile() {
    let reg = registry_three();
    let mut sim = WeatherSimState::new(&reg, 1, "plains");
    sim.force(reg.id_of("rain").unwrap());
    sim.tick(0.0, &reg, glam::Vec3::ZERO);
    let snap = sim.snapshot(&reg);
    assert_eq!(snap.precipitation.kind, PrecipitationKindSample::Rain);
    assert!(snap.precipitation.intensity > 0.0);

    sim.force(reg.id_of("clear").unwrap());
    sim.tick(0.0, &reg, glam::Vec3::ZERO);
    let snap = sim.snapshot(&reg);
    assert_eq!(snap.precipitation.kind, PrecipitationKindSample::None);
    assert_eq!(snap.precipitation.intensity, 0.0);
}
