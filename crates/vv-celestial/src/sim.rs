//! Celestial simulation state.
//!
//! Stateless w.r.t. orbital positions — those are derived from `WorldTime` on
//! every `snapshot()` call. The "state" is really just the configuration:
//! which body is the player's planet, which is the primary star, observer
//! altitude, etc. Everything else is computed deterministically from time.

use vv_content_schema::RawCelestialKind;
use vv_world::WorldTime;

use crate::body::{CelestialBodyId, CelestialRegistry};
use crate::eclipse::solar_eclipse_factor;
use crate::orbit::{body_position, SystemPos};
use crate::snapshot::{AltitudeBand, CelestialState, MoonSample};

#[derive(Clone, Debug)]
pub struct CelestialSimState {
    observer_planet: Option<CelestialBodyId>,
    primary_star: Option<CelestialBodyId>,
    observer_altitude_m: f32,
    /// Day-phase value at which the sun crosses local noon. Lets the runtime
    /// align the existing `WorldTime` day cycle to the celestial sun pass
    /// without rebaking content. Default `0.0` ↔ noon at phase 0.
    noon_phase: f32,
}

impl CelestialSimState {
    /// Pick the first `Planet` as observer and the first `Star` as primary.
    /// Either may be overridden later via `set_*` methods.
    pub fn new(registry: &CelestialRegistry) -> Self {
        Self {
            observer_planet: registry.first_of_kind(RawCelestialKind::Planet),
            primary_star: registry.first_of_kind(RawCelestialKind::Star),
            observer_altitude_m: 0.0,
            noon_phase: 0.0,
        }
    }

    pub fn set_observer_planet(&mut self, id: Option<CelestialBodyId>) {
        self.observer_planet = id;
    }

    pub fn set_primary_star(&mut self, id: Option<CelestialBodyId>) {
        self.primary_star = id;
    }

    pub fn set_observer_altitude_m(&mut self, altitude_m: f32) {
        self.observer_altitude_m = altitude_m.max(0.0);
    }

    pub fn observer_planet(&self) -> Option<CelestialBodyId> {
        self.observer_planet
    }

    pub fn primary_star(&self) -> Option<CelestialBodyId> {
        self.primary_star
    }

    pub fn snapshot(&self, registry: &CelestialRegistry, time: WorldTime) -> CelestialState {
        if registry.is_empty() {
            return empty_state(self.observer_altitude_m);
        }

        let t = time.elapsed_seconds() as f64;
        let observer_system_pos = match self.observer_planet {
            Some(id) => body_position(registry, id, t),
            None => SystemPos::ZERO,
        };

        // Build the local-frame rotation: the planet spins around its axis
        // with the cycle described by `WorldTime`. The current day_phase
        // gives the spin angle; we rotate every system-frame direction by
        // the inverse to recover the observer's local sky.
        let spin_axis = self
            .observer_planet
            .map(|id| registry.get(id).spin.axis)
            .unwrap_or(glam::Vec3::Y);
        let spin_angle = -(time.day_phase() - self.noon_phase) * std::f32::consts::TAU;
        let spin_rot = glam::Quat::from_axis_angle(spin_axis, spin_angle);

        // Sun.
        let (sun_dir_world, sun_distance_m, sun_color, sun_alpha) =
            if let Some(star_id) = self.primary_star {
                let star = registry.get(star_id);
                let star_pos = body_position(registry, star_id, t);
                let delta = star_pos - observer_system_pos;
                let distance_m = delta.length();
                let dir_system = if distance_m > 1.0e-6 {
                    (delta / distance_m).as_vec3()
                } else {
                    glam::Vec3::Y
                };
                let dir_world = (spin_rot * dir_system).normalize_or_zero();
                let alpha = if distance_m > 0.0 {
                    ((star.radius_m / distance_m).clamp(-1.0, 1.0)).asin() as f32
                } else {
                    0.0
                };
                (dir_world, distance_m, star.surface.emissive_color, alpha)
            } else {
                (glam::Vec3::Y, 0.0, glam::Vec3::ONE, 0.0)
            };

        // Moons (every body of kind Moon).
        let mut moons = Vec::new();
        for body in registry.iter() {
            if body.kind != RawCelestialKind::Moon {
                continue;
            }
            let pos = body_position(registry, body.id, t);
            let delta = pos - observer_system_pos;
            let distance_m = delta.length();
            if distance_m <= 0.0 {
                continue;
            }
            let dir_system = (delta / distance_m).as_vec3();
            let dir_world = (spin_rot * dir_system).normalize_or_zero();
            let angular_radius_rad = ((body.radius_m / distance_m).clamp(-1.0, 1.0)).asin() as f32;
            // Phase: relative angle between moon and sun in local frame.
            // 0 = new (back lit), 1 = full (sun behind observer).
            let cos_sep = dir_world.dot(sun_dir_world).clamp(-1.0, 1.0);
            let phase = 0.5 * (1.0 - cos_sep);
            moons.push(MoonSample {
                id: body.id,
                direction: dir_world,
                angular_radius_rad,
                distance_m,
                phase,
            });
        }

        // Eclipse: max over moons (only one can be "in front" but stars are
        // also valid occluders for future inner-planet transits).
        let mut eclipse_factor = 0.0_f32;
        if sun_distance_m > 0.0 {
            let sun_radius_m = self
                .primary_star
                .map(|id| registry.get(id).radius_m)
                .unwrap_or(0.0);
            for m in &moons {
                let f = solar_eclipse_factor(
                    sun_dir_world,
                    sun_radius_m,
                    sun_distance_m,
                    m.direction,
                    registry.get(m.id).radius_m,
                    m.distance_m,
                );
                if f > eclipse_factor {
                    eclipse_factor = f;
                }
            }
        }

        // Stars visibility: linear fade from sun_dir.y > 0 (day) to
        // sun_dir.y < 0 (night). Cloud cover and weather modulate this
        // further outside this crate.
        let stars_visibility = (1.0 - sun_dir_world.y).clamp(0.0, 2.0) * 0.5;
        let stars_visibility = stars_visibility.clamp(0.0, 1.0);

        CelestialState {
            sun_dir_world,
            sun_disc_color: sun_color,
            sun_disc_angular_radius: sun_alpha,
            sun_distance_m,
            moons,
            stars_visibility,
            aurora_intensity: 0.0,
            eclipse_factor,
            altitude_band: AltitudeBand::from_altitude_m(self.observer_altitude_m),
        }
    }
}

fn empty_state(altitude_m: f32) -> CelestialState {
    CelestialState {
        sun_dir_world: glam::Vec3::Y,
        sun_disc_color: glam::Vec3::ONE,
        sun_disc_angular_radius: 0.0,
        sun_distance_m: 0.0,
        moons: Vec::new(),
        stars_visibility: 0.0,
        aurora_intensity: 0.0,
        eclipse_factor: 0.0,
        altitude_band: AltitudeBand::from_altitude_m(altitude_m),
    }
}
