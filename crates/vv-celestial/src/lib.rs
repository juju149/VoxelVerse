//! Celestial mechanics crate (Phase 4 of the weather/cosmos roadmap).
//!
//! Drives the [`CelestialState`] snapshot consumed by the renderer's sky
//! pass. Like `vv-weather`, the solver is stateless w.r.t. its output and
//! deterministic given the same `WorldTime` and registry — no RNG involved.
//!
//! Module layout follows `docs/v1/13_WEATHER_AND_COSMOS.md` §4.2:
//! - [`body`]: runtime registry built from `vv-content-schema`.
//! - [`orbit`]: f64 circular orbit math (recursive parent chain walk).
//! - [`eclipse`]: angular eclipse factor.
//! - [`sim`]: `CelestialSimState` + per-frame snapshot.
//! - [`snapshot`]: the public read-only frame state.

mod body;
mod eclipse;
mod orbit;
mod sim;
mod snapshot;

pub use body::{
    CelestialBodyId, CelestialRegistry, RegistryError, ResolvedBody, ResolvedOrbit,
    ResolvedSpin, ResolvedSurface,
};
pub use eclipse::solar_eclipse_factor;
pub use orbit::{body_position, SystemPos};
pub use sim::CelestialSimState;
pub use snapshot::{AltitudeBand, CelestialState, MoonSample};
