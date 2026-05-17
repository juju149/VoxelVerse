//! Weather simulation crate (Phase 2 of the weather/cosmos roadmap).
//!
//! Owns the data-driven Markov solver that drives the [`WeatherState`]
//! snapshot consumed by the renderer, audio, and HUD. The renderer never
//! reads solver internals — only the snapshot.
//!
//! Module layout follows `docs/v1/13_WEATHER_AND_COSMOS.md` §4.1:
//! - [`profile`]: runtime registry built from `vv-content-schema`.
//! - [`sim`]: Markov state machine + transitions + tick.
//! - [`wind`]: gust process and slow direction drift.
//! - [`lightning`]: Poisson strike sampler.
//! - [`snapshot`]: the public read-only frame state.
//! - [`rng`]: tiny deterministic PCG.

mod lightning;
mod profile;
mod rng;
mod sim;
mod snapshot;
mod wind;

pub use profile::{ResolvedPrecipitation, ResolvedProfile, WeatherProfileId, WeatherRegistry};
pub use sim::WeatherSimState;
pub use snapshot::{
    LightningStrike, PrecipitationKindSample, PrecipitationSample, WeatherState, WindVector,
};
