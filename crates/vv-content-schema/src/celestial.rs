//! Celestial body and star catalog schemas.
//!
//! Three file kinds live under `defs/world/celestial/`:
//! - `*.celestial.ron`    — `CelestialBody` (sun, moon, planet, ring, belt)
//! - `*.star_catalog.ron` — `StarCatalog` (background star field + galaxy)
//!
//! All distances are in metres so the orbital solver can stay in `f64`.
//!
//! See `docs/v1/13_WEATHER_AND_COSMOS.md` §3.3 and §3.4.

use crate::ContentRef;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawCelestialKind {
    Star,
    Moon,
    Planet,
    Belt,
    Ring,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCelestialOrbitDef {
    /// Parent body id. `None` means the system barycentre.
    #[serde(default)]
    pub parent: Option<ContentRef>,
    /// Orbit semi-major axis in metres.
    pub semi_major_axis_m: f64,
    /// `[0, 1)` eccentricity. V1 supports only `0.0` (circular orbits).
    #[serde(default)]
    pub eccentricity: f64,
    /// Orbital period in seconds.
    pub period_s: f64,
    /// Mean-anomaly phase offset in radians.
    #[serde(default)]
    pub phase_rad: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RawCelestialSpinDef {
    /// Rotation axis (unit vector, normalised at load).
    pub axis: (f32, f32, f32),
    /// Sidereal rotation period in seconds.
    pub period_s: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RawCelestialCoronaDef {
    pub inner: (f32, f32, f32),
    pub outer: (f32, f32, f32),
    /// Outer radius as a multiplier of the body's angular radius.
    pub radius_mul: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCelestialSurfaceDef {
    /// HDR emissive colour (linear).
    pub emissive_color: (f32, f32, f32),
    /// HDR emissive intensity. Stars usually `> 1`, moons usually `0`.
    #[serde(default)]
    pub emissive_intensity: f32,
    /// Optional corona (sun-only typically).
    #[serde(default)]
    pub corona: Option<RawCelestialCoronaDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCelestialBodyDef {
    #[serde(alias = "name")]
    pub display_name: String,
    pub kind: RawCelestialKind,
    /// Optional voxel model for ground-close or in-space rendering.
    #[serde(default)]
    pub voxel_model: Option<ContentRef>,
    /// Physical body radius in metres.
    pub radius_m: f64,
    /// Orbit. `None` means the body is the system barycentre.
    #[serde(default)]
    pub orbit: Option<RawCelestialOrbitDef>,
    pub spin: RawCelestialSpinDef,
    pub surface: RawCelestialSurfaceDef,
    /// Whether this body is rendered in the sky pass from a planet surface.
    #[serde(default = "default_true")]
    pub visible_from_surface: bool,
    /// Distance threshold under which the body switches from billboard
    /// impostor to a true voxel mesh.
    #[serde(default = "default_billboard_distance")]
    pub lod_billboard_distance_m: f64,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
pub enum RawStarSpectralClass {
    O,
    B,
    A,
    F,
    G,
    K,
    M,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RawStarSpectralWeight {
    pub class: RawStarSpectralClass,
    pub weight: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawMilkyWayDef {
    /// 2D density texture content ref (KTX2 or PNG).
    pub density_texture: ContentRef,
    pub tint: (f32, f32, f32),
    #[serde(default = "default_milky_intensity")]
    pub intensity: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawNebulaDef {
    pub name: String,
    /// Celestial-coordinate centre in radians `(longitude, latitude)`.
    pub center_lonlat: (f32, f32),
    /// Visual radius in radians.
    pub radius_rad: f32,
    pub color: (f32, f32, f32),
    pub intensity: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawStarCatalogDef {
    #[serde(alias = "name")]
    pub display_name: String,
    /// Catalog seed for procedural star placement.
    pub seed: u64,
    /// Number of dynamic stars instanced at runtime. V1 cap ≈ 8 000.
    pub star_count: u32,
    /// `(min, max)` apparent magnitudes, brightest to dimmest.
    pub magnitude_range: (f32, f32),
    /// Spectral-class weights, used to colour the field.
    pub spectral_distribution: Vec<RawStarSpectralWeight>,
    #[serde(default)]
    pub milky_way: Option<RawMilkyWayDef>,
    #[serde(default)]
    pub nebulae: Vec<RawNebulaDef>,
}

fn default_true() -> bool {
    true
}

fn default_billboard_distance() -> f64 {
    1.0e8
}

fn default_milky_intensity() -> f32 {
    0.6
}
