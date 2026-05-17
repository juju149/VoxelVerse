//! End-to-end tests for the celestial simulator.

use vv_celestial::{
    body_position, AltitudeBand, CelestialBodyId, CelestialRegistry, CelestialSimState, SystemPos,
};
use vv_content_schema::{
    ContentRef, RawCelestialBodyDef, RawCelestialKind, RawCelestialOrbitDef, RawCelestialSpinDef,
    RawCelestialSurfaceDef,
};
use vv_world::WorldTime;

fn star(name: &str, radius_m: f64, spin_period_s: f64) -> (String, RawCelestialBodyDef) {
    (
        format!("core:celestial/{name}"),
        RawCelestialBodyDef {
            display_name: name.to_string(),
            kind: RawCelestialKind::Star,
            voxel_model: None,
            radius_m,
            orbit: None,
            spin: RawCelestialSpinDef {
                axis: (0.0, 1.0, 0.0),
                period_s: spin_period_s,
            },
            surface: RawCelestialSurfaceDef {
                emissive_color: (1.0, 0.95, 0.85),
                emissive_intensity: 8.0,
                corona: None,
            },
            visible_from_surface: true,
            lod_billboard_distance_m: 1.0e8,
        },
    )
}

fn planet(
    name: &str,
    parent: Option<&str>,
    semi_major_m: f64,
    period_s: f64,
    spin_period_s: f64,
    radius_m: f64,
) -> (String, RawCelestialBodyDef) {
    (
        format!("core:celestial/{name}"),
        RawCelestialBodyDef {
            display_name: name.to_string(),
            kind: RawCelestialKind::Planet,
            voxel_model: None,
            radius_m,
            orbit: Some(RawCelestialOrbitDef {
                parent: parent.map(|p| ContentRef(format!("core:celestial/{p}"))),
                semi_major_axis_m: semi_major_m,
                eccentricity: 0.0,
                period_s,
                phase_rad: 0.0,
            }),
            spin: RawCelestialSpinDef {
                // Earth-like axial tilt (~23.5° from the orbit normal).
                // Without a tilt the sun rides the horizon forever — the
                // observer's equator would lie in the orbit plane.
                axis: (0.3978, 0.9175, 0.0),
                period_s: spin_period_s,
            },
            surface: RawCelestialSurfaceDef {
                emissive_color: (0.4, 0.6, 0.8),
                emissive_intensity: 0.0,
                corona: None,
            },
            visible_from_surface: true,
            lod_billboard_distance_m: 1.0e7,
        },
    )
}

fn moon(
    name: &str,
    parent: &str,
    semi_major_m: f64,
    period_s: f64,
    radius_m: f64,
) -> (String, RawCelestialBodyDef) {
    (
        format!("core:celestial/{name}"),
        RawCelestialBodyDef {
            display_name: name.to_string(),
            kind: RawCelestialKind::Moon,
            voxel_model: None,
            radius_m,
            orbit: Some(RawCelestialOrbitDef {
                parent: Some(ContentRef(format!("core:celestial/{parent}"))),
                semi_major_axis_m: semi_major_m,
                eccentricity: 0.0,
                period_s,
                phase_rad: 0.0,
            }),
            spin: RawCelestialSpinDef {
                axis: (0.0, 1.0, 0.0),
                period_s, // tidally locked
            },
            surface: RawCelestialSurfaceDef {
                emissive_color: (0.6, 0.6, 0.62),
                emissive_intensity: 0.0,
                corona: None,
            },
            visible_from_surface: true,
            lod_billboard_distance_m: 1.0e7,
        },
    )
}

fn earth_like_registry() -> CelestialRegistry {
    let items = vec![
        star("sol", 6.96e8, 2.16e6),
        planet(
            "terra",
            Some("sol"),
            1.496e11,
            365.25 * 86_400.0,
            86_400.0,
            6.371e6,
        ),
        moon("luna", "terra", 3.844e8, 27.3 * 86_400.0, 1.737e6),
    ];
    CelestialRegistry::from_raw(&items).expect("registry must build")
}

#[test]
fn registry_resolves_parents_and_finds_kinds() {
    let reg = earth_like_registry();
    assert_eq!(reg.len(), 3);
    let sun = reg.id_of("sol").expect("sun");
    let terra = reg.id_of("terra").expect("terra");
    let luna = reg.id_of("luna").expect("luna");
    assert_eq!(reg.get(sun).orbit.as_ref().and_then(|o| o.parent), None);
    assert_eq!(
        reg.get(terra).orbit.as_ref().and_then(|o| o.parent),
        Some(sun)
    );
    assert_eq!(
        reg.get(luna).orbit.as_ref().and_then(|o| o.parent),
        Some(terra)
    );
}

#[test]
fn unknown_parent_is_rejected() {
    let items = vec![planet("orphan", Some("ghost"), 1.0, 1.0, 1.0, 1.0)];
    assert!(CelestialRegistry::from_raw(&items).is_err());
}

#[test]
fn duplicate_short_id_is_rejected() {
    let mut a = star("dup", 1.0, 1.0);
    let mut b = star("dup", 1.0, 1.0);
    // Force same short id via the path tail.
    a.0 = "pack_a:celestial/dup".to_string();
    b.0 = "pack_b:celestial/dup".to_string();
    let items = vec![a, b];
    assert!(CelestialRegistry::from_raw(&items).is_err());
}

#[test]
fn sun_direction_makes_one_complete_revolution_per_day() {
    let reg = earth_like_registry();
    let sim = CelestialSimState::new(&reg);
    let day_length_s = reg.get(reg.id_of("terra").unwrap()).spin.period_s as f32;
    let cfg_day = WorldTime::new(day_length_s, 0.0);

    let mut time = cfg_day;
    let s_noon = sim.snapshot(&reg, time).sun_dir_world;

    // Advance to phase 0.5 (midnight): sun should be roughly opposite.
    let mut t_mid = WorldTime::new(day_length_s, 0.5);
    t_mid.tick(0.0);
    let s_mid = sim.snapshot(&reg, t_mid).sun_dir_world;
    let dot = s_noon.dot(s_mid);
    assert!(
        dot < -0.5,
        "sun should flip across the sky over half a day (dot={dot})"
    );

    // After a full day at the same phase, the snapshot must match.
    time.tick(day_length_s);
    let s_full = sim.snapshot(&reg, time).sun_dir_world;
    assert!((s_full - s_noon).length() < 0.05);
}

#[test]
fn moons_are_included_in_snapshot() {
    let reg = earth_like_registry();
    let sim = CelestialSimState::new(&reg);
    let snap = sim.snapshot(&reg, WorldTime::default());
    assert_eq!(snap.moons.len(), 1);
    let m = &snap.moons[0];
    assert!(m.distance_m > 3.0e8 && m.distance_m < 5.0e8);
    assert!(m.direction.length() > 0.9);
    assert!((0.0..=1.0).contains(&m.phase));
}

#[test]
fn altitude_band_classification() {
    let reg = earth_like_registry();
    let mut sim = CelestialSimState::new(&reg);
    sim.set_observer_altitude_m(0.0);
    assert_eq!(
        sim.snapshot(&reg, WorldTime::default()).altitude_band,
        AltitudeBand::Ground
    );
    sim.set_observer_altitude_m(10_000.0);
    assert_eq!(
        sim.snapshot(&reg, WorldTime::default()).altitude_band,
        AltitudeBand::Strato
    );
    sim.set_observer_altitude_m(50_000.0);
    assert_eq!(
        sim.snapshot(&reg, WorldTime::default()).altitude_band,
        AltitudeBand::Meso
    );
    sim.set_observer_altitude_m(120_000.0);
    assert_eq!(
        sim.snapshot(&reg, WorldTime::default()).altitude_band,
        AltitudeBand::Space
    );
}

#[test]
fn stars_visible_at_night() {
    let reg = earth_like_registry();
    let sim = CelestialSimState::new(&reg);
    let day_length_s = reg.get(reg.id_of("terra").unwrap()).spin.period_s as f32;

    let noon = WorldTime::new(day_length_s, 0.0);
    let midnight = WorldTime::new(day_length_s, 0.5);

    let snap_noon = sim.snapshot(&reg, noon);
    let snap_mid = sim.snapshot(&reg, midnight);
    assert!(
        snap_mid.stars_visibility > snap_noon.stars_visibility,
        "stars must be more visible at midnight ({} vs {})",
        snap_mid.stars_visibility,
        snap_noon.stars_visibility
    );
}

#[test]
fn determinism_same_time_same_snapshot() {
    let reg = earth_like_registry();
    let sim = CelestialSimState::new(&reg);
    let day_length_s = 86_400.0_f32;
    let mut t = WorldTime::new(day_length_s, 0.13);
    t.tick(73.5);
    let a = sim.snapshot(&reg, t);
    let b = sim.snapshot(&reg, t);
    assert_eq!(a.sun_dir_world, b.sun_dir_world);
    assert_eq!(a.moons.len(), b.moons.len());
    for (ma, mb) in a.moons.iter().zip(b.moons.iter()) {
        assert_eq!(ma.direction, mb.direction);
        assert_eq!(ma.distance_m, mb.distance_m);
    }
}

#[test]
fn body_position_returns_sun_at_origin_when_no_orbit() {
    let reg = earth_like_registry();
    let sun = reg.id_of("sol").unwrap();
    let pos = body_position(&reg, sun, 0.0);
    assert_eq!(pos, SystemPos::ZERO);
}

#[test]
fn first_of_kind_finds_primary() {
    let reg = earth_like_registry();
    let star = reg.first_of_kind(RawCelestialKind::Star);
    assert!(star.is_some());
    assert_eq!(reg.get(star.unwrap()).short_id, "sol");
}

#[test]
fn body_id_is_compact() {
    // Ids fit in 16 bits so packing them with kind into a u32 GPU instance
    // attribute stays free in future passes.
    assert_eq!(std::mem::size_of::<CelestialBodyId>(), 2);
}
