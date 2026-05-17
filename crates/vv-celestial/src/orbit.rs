//! Circular orbit math in `f64` system frame.
//!
//! V1 supports circular orbits only — eccentricity > 0 is accepted by the
//! schema but ignored by the solver (the pack-doctor warns about it). The
//! orbit plane is XZ with +Y as the system normal.
//!
//! Phase 4.1 of the weather/cosmos roadmap. Phase 4.2 will add Keplerian
//! ellipses + inclination.

use crate::body::{CelestialBodyId, CelestialRegistry};

/// System-frame position in metres. `DVec3` because heliocentric distances
/// overflow `f32` precision well before reaching planet/moon orbital radii.
pub type SystemPos = glam::DVec3;

/// Compute the body's position at simulation time `t_seconds`. Recursively
/// walks the parent chain — depth is bounded by registry size, no allocation.
pub fn body_position(
    registry: &CelestialRegistry,
    id: CelestialBodyId,
    t_seconds: f64,
) -> SystemPos {
    let body = registry.get(id);
    let parent_pos = match body.orbit.as_ref().and_then(|o| o.parent) {
        Some(parent_id) => body_position(registry, parent_id, t_seconds),
        None => SystemPos::ZERO,
    };
    match &body.orbit {
        Some(orbit) => {
            let theta =
                std::f64::consts::TAU * (t_seconds / orbit.period_s) + orbit.phase_rad;
            parent_pos
                + SystemPos::new(orbit.semi_major_axis_m * theta.cos(), 0.0, orbit.semi_major_axis_m * theta.sin())
        }
        None => parent_pos,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{CelestialBodyId, CelestialRegistry};
    use vv_content_schema::{
        RawCelestialBodyDef, RawCelestialKind, RawCelestialOrbitDef, RawCelestialSpinDef,
        RawCelestialSurfaceDef, ContentRef,
    };

    fn surf() -> RawCelestialSurfaceDef {
        RawCelestialSurfaceDef {
            emissive_color: (1.0, 1.0, 1.0),
            emissive_intensity: 0.0,
            corona: None,
        }
    }

    fn spin() -> RawCelestialSpinDef {
        RawCelestialSpinDef {
            axis: (0.0, 1.0, 0.0),
            period_s: 100.0,
        }
    }

    fn body(name: &str, parent: Option<&str>, radius_au: f64, period_s: f64) -> RawCelestialBodyDef {
        RawCelestialBodyDef {
            display_name: name.to_string(),
            kind: RawCelestialKind::Planet,
            voxel_model: None,
            radius_m: 1.0,
            orbit: Some(RawCelestialOrbitDef {
                parent: parent.map(|p| ContentRef(format!("core:celestial/{p}"))),
                semi_major_axis_m: radius_au * 1.496e11,
                eccentricity: 0.0,
                period_s,
                phase_rad: 0.0,
            }),
            spin: spin(),
            surface: surf(),
            visible_from_surface: true,
            lod_billboard_distance_m: 1.0e8,
        }
    }

    #[test]
    fn body_with_no_orbit_sits_at_barycentre() {
        let mut sun = body("sun", None, 0.0, 1.0);
        sun.orbit = None;
        let reg = CelestialRegistry::from_raw(&[("core:celestial/sun".to_string(), sun)]).unwrap();
        let pos = body_position(&reg, CelestialBodyId(0), 12345.0);
        assert_eq!(pos, SystemPos::ZERO);
    }

    #[test]
    fn circular_orbit_returns_to_start_after_one_period() {
        let mut sun = body("sun", None, 0.0, 1.0);
        sun.orbit = None;
        let earth = body("earth", Some("sun"), 1.0, 365.25 * 86_400.0);
        let reg = CelestialRegistry::from_raw(&[
            ("core:celestial/sun".to_string(), sun),
            ("core:celestial/earth".to_string(), earth),
        ])
        .unwrap();
        let earth_id = reg.id_of("earth").unwrap();
        let pos0 = body_position(&reg, earth_id, 0.0);
        let pos1 = body_position(&reg, earth_id, 365.25 * 86_400.0);
        assert!((pos0 - pos1).length() < 1.0); // sub-metre after a year
    }

    #[test]
    fn moon_position_includes_parent_motion() {
        let mut sun = body("sun", None, 0.0, 1.0);
        sun.orbit = None;
        let earth = body("earth", Some("sun"), 1.0, 365.25 * 86_400.0);
        let moon = body("moon", Some("earth"), 0.00257, 27.3 * 86_400.0);
        let reg = CelestialRegistry::from_raw(&[
            ("core:celestial/sun".to_string(), sun),
            ("core:celestial/earth".to_string(), earth),
            ("core:celestial/moon".to_string(), moon),
        ])
        .unwrap();
        let earth_id = reg.id_of("earth").unwrap();
        let moon_id = reg.id_of("moon").unwrap();
        let earth_pos = body_position(&reg, earth_id, 1000.0);
        let moon_pos = body_position(&reg, moon_id, 1000.0);
        let moon_radius_from_earth = (moon_pos - earth_pos).length();
        assert!(moon_radius_from_earth > 3.0e8);
        assert!(moon_radius_from_earth < 4.0e8);
    }
}
