//! Per-frame snapshot produced by [`crate::CelestialSimState::snapshot`].
//!
//! The renderer reads only this struct. Solver internals (system positions,
//! orbital phases) are never exposed — matches the discipline established by
//! `vv-weather`.

use crate::body::CelestialBodyId;

/// Vertical band the observer currently occupies. Drives the sky/space
/// transition (Phase 7 of the weather/cosmos roadmap). Phase 4 already wires
/// the band derivation from altitude, but the renderer hooks land later.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AltitudeBand {
    #[default]
    Ground,
    Strato,
    Meso,
    Space,
}

impl AltitudeBand {
    /// Earth-like classification. Altitude in metres above the surface.
    pub fn from_altitude_m(altitude_m: f32) -> Self {
        if altitude_m < 2_000.0 {
            Self::Ground
        } else if altitude_m < 20_000.0 {
            Self::Strato
        } else if altitude_m < 80_000.0 {
            Self::Meso
        } else {
            Self::Space
        }
    }
}

/// Per-moon visibility sample. `direction` is the unit direction toward the
/// moon in the observer's *local* frame (the planet's surface frame, with
/// +Y up). Magnitude is `1.0` so the renderer can drop it straight into a
/// billboard rotation.
#[derive(Clone, Copy, Debug)]
pub struct MoonSample {
    pub id: CelestialBodyId,
    pub direction: glam::Vec3,
    /// Angular radius in radians (`asin(radius / distance)`).
    pub angular_radius_rad: f32,
    /// Linear distance in metres (drives parallax LOD selection).
    pub distance_m: f64,
    /// `[0, 1]` phase: `0` = new (back lit), `1` = full (sun behind observer).
    pub phase: f32,
}

/// Public snapshot.
#[derive(Clone, Debug)]
pub struct CelestialState {
    pub sun_dir_world: glam::Vec3,
    pub sun_disc_color: glam::Vec3,
    pub sun_disc_angular_radius: f32,
    /// Distance from observer to sun in metres. `0` if no star body resolves.
    pub sun_distance_m: f64,
    pub moons: Vec<MoonSample>,
    /// `0..1` — how many stars are visible. `1` at zenith elevation < 0
    /// (sun under horizon) AND no thick cloud cover; the renderer further
    /// modulates this by weather.
    pub stars_visibility: f32,
    /// `0..1` reserved for the polar aurora overlay (computed by the
    /// biome-ambience layer in Phase 6 but exposed here so the sky pass
    /// can sample everything in one struct).
    pub aurora_intensity: f32,
    /// `0..1`. `0` = full sun, `1` = totally eclipsed.
    pub eclipse_factor: f32,
    pub altitude_band: AltitudeBand,
}
