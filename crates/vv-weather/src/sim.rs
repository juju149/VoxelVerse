//! Markov weather solver.
//!
//! Holds a `current → next` transition pair, a seeded RNG, the wind solver,
//! and lightning sampler. Once per [`WeatherSimState::tick`], it advances all
//! sub-systems and produces a [`WeatherState`] snapshot.
//!
//! Picking rules:
//! - while a transition is in flight, no new transition is started;
//! - once `current` has been active longer than `transitions.min_duration_s`,
//!   each tick we draw a Bernoulli with probability proportional to
//!   `1/max_duration_s` to attempt a switch;
//! - the next profile is sampled from the registry weighted by
//!   `rarity × biome_bias[current_biome]`. Profiles with weight 0 in the
//!   current biome are excluded.

use crate::lightning;
use crate::profile::{ResolvedProfile, WeatherProfileId, WeatherRegistry};
use crate::rng::PcgRng;
use crate::snapshot::{PrecipitationKindSample, PrecipitationSample, WeatherState, WindVector};
use crate::wind::WindState;

#[derive(Clone, Debug)]
pub struct WeatherSimState {
    rng: PcgRng,
    current: WeatherProfileId,
    next: Option<WeatherProfileId>,
    elapsed_in_current_s: f32,
    transition_progress: f32,
    transition_duration_s: f32,
    biome_short_id: String,
    wind: WindState,
    pending_strikes: Vec<crate::snapshot::LightningStrike>,
}

impl WeatherSimState {
    /// Initialise the sim. The first weather is picked deterministically from
    /// the registry using the supplied seed — the highest-rarity profile in
    /// the current biome wins, ties broken by registry order.
    pub fn new(registry: &WeatherRegistry, seed: u64, biome_short_id: impl Into<String>) -> Self {
        assert!(!registry.is_empty(), "WeatherRegistry must contain at least one profile");
        let biome_short_id = biome_short_id.into();
        let mut rng = PcgRng::new(seed);
        let current = pick_initial(registry, &biome_short_id).unwrap_or(WeatherProfileId(0));
        let heading = rng.next_unit() * std::f32::consts::TAU;
        Self {
            rng,
            current,
            next: None,
            elapsed_in_current_s: 0.0,
            transition_progress: 0.0,
            transition_duration_s: 0.0,
            biome_short_id,
            wind: WindState::new(heading),
            pending_strikes: Vec::new(),
        }
    }

    pub fn current_id(&self) -> WeatherProfileId {
        self.current
    }

    pub fn next_id(&self) -> Option<WeatherProfileId> {
        self.next
    }

    pub fn is_transitioning(&self) -> bool {
        self.next.is_some()
    }

    pub fn set_biome(&mut self, biome_short_id: impl Into<String>) {
        self.biome_short_id = biome_short_id.into();
    }

    /// Force a switch to a specific profile, completing instantly. Useful for
    /// debug commands ("/weather thunderstorm").
    pub fn force(&mut self, id: WeatherProfileId) {
        self.current = id;
        self.next = None;
        self.elapsed_in_current_s = 0.0;
        self.transition_progress = 0.0;
        self.transition_duration_s = 0.0;
    }

    /// Advance the solver one frame.
    ///
    /// - `dt` is wall-clock seconds since the last tick.
    /// - `observer_pos` lets lightning sample positions near the player.
    pub fn tick(&mut self, dt: f32, registry: &WeatherRegistry, observer_pos: glam::Vec3) {
        let dt = dt.max(0.0);

        if let Some(next_id) = self.next {
            // Advance transition.
            self.transition_progress += if self.transition_duration_s > 0.0 {
                dt / self.transition_duration_s
            } else {
                1.0
            };
            if self.transition_progress >= 1.0 {
                self.current = next_id;
                self.next = None;
                self.transition_progress = 0.0;
                self.transition_duration_s = 0.0;
                self.elapsed_in_current_s = 0.0;
            }
        } else {
            self.elapsed_in_current_s += dt;
            let profile = registry.get(self.current);
            if self.elapsed_in_current_s >= profile.transitions.min_duration_s
                && self.attempt_switch_now(profile, dt)
            {
                if let Some(next_id) = self.pick_next(registry) {
                    self.next = Some(next_id);
                    self.transition_progress = 0.0;
                    self.transition_duration_s =
                        registry.get(next_id).transitions.fade_in_s.max(0.05);
                }
            }
        }

        // Wind always advances using the *current* profile (the next profile
        // bleeds in via the snapshot's blend, not the wind state itself).
        let current_profile = registry.get(self.current);
        self.wind.tick(dt, current_profile, &mut self.rng);

        // Lightning: blend between current and next strike rates.
        if let Some(strike) = self.sample_lightning(registry, dt, observer_pos) {
            self.pending_strikes.push(strike);
        }
    }

    /// Produce the frame snapshot consumed by the renderer/audio/HUD.
    /// Drains pending lightning strikes — they're delivered exactly once.
    pub fn snapshot(&mut self, registry: &WeatherRegistry) -> WeatherState {
        let current = registry.get(self.current);
        let next = self.next.map(|id| registry.get(id));
        let blend = self.transition_progress.clamp(0.0, 1.0);

        let cloud_coverage = blend_scalar(current.cloud_coverage, next.map(|p| p.cloud_coverage), blend);
        let fog_multiplier = blend_scalar(current.fog_multiplier, next.map(|p| p.fog_multiplier), blend);
        let cloud_density_mul = blend_scalar(current.cloud_density_mul, next.map(|p| p.cloud_density_mul), blend);
        let cloud_speed_mul = blend_scalar(current.cloud_speed_mul, next.map(|p| p.cloud_speed_mul), blend);

        let wind: WindVector = self.wind.sample(current, next, blend);
        let precipitation = blend_precipitation(current, next, blend);
        let lightning_events = std::mem::take(&mut self.pending_strikes);

        WeatherState {
            current: self.current,
            next: self.next,
            blend,
            cloud_coverage,
            fog_multiplier,
            cloud_density_mul,
            cloud_speed_mul,
            wind,
            precipitation,
            lightning_events,
        }
    }

    fn attempt_switch_now(&mut self, profile: &ResolvedProfile, dt: f32) -> bool {
        // Bernoulli with mean-time-to-switch = max_duration_s - min_duration_s
        // (so on average we transition near max_duration_s).
        let window = (profile.transitions.max_duration_s - profile.transitions.min_duration_s).max(0.1);
        let p = 1.0 - (-dt / window).exp();
        self.rng.next_unit() < p
    }

    fn pick_next(&mut self, registry: &WeatherRegistry) -> Option<WeatherProfileId> {
        let total: f32 = registry
            .iter()
            .filter(|p| p.id != self.current)
            .map(|p| profile_weight(p, &self.biome_short_id))
            .sum();
        if total <= 0.0 {
            return None;
        }
        let target = self.rng.next_unit() * total;
        let mut acc = 0.0;
        for p in registry.iter() {
            if p.id == self.current {
                continue;
            }
            acc += profile_weight(p, &self.biome_short_id);
            if target <= acc {
                return Some(p.id);
            }
        }
        None
    }

    fn sample_lightning(
        &mut self,
        registry: &WeatherRegistry,
        dt: f32,
        observer_pos: glam::Vec3,
    ) -> Option<crate::snapshot::LightningStrike> {
        // While transitioning, blend the two rates by `blend`.
        let current = registry.get(self.current);
        if let Some(next_id) = self.next {
            let next = registry.get(next_id);
            // Pick which profile drives this tick's bernoulli, weighted by the
            // transition blend. Keeps the sampler simple while still
            // interpolating the rate over the fade.
            let pick_next = self.rng.next_unit() < self.transition_progress.clamp(0.0, 1.0);
            let p = if pick_next { next } else { current };
            lightning::sample(dt, p, observer_pos, &mut self.rng)
        } else {
            lightning::sample(dt, current, observer_pos, &mut self.rng)
        }
    }
}

fn pick_initial(registry: &WeatherRegistry, biome_short_id: &str) -> Option<WeatherProfileId> {
    registry
        .iter()
        .max_by(|a, b| {
            profile_weight(a, biome_short_id)
                .partial_cmp(&profile_weight(b, biome_short_id))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|p| p.id)
}

fn profile_weight(profile: &ResolvedProfile, biome_short_id: &str) -> f32 {
    let bias = profile
        .biome_bias
        .get(biome_short_id)
        .copied()
        .unwrap_or(1.0);
    (profile.rarity.max(0.0)) * bias.max(0.0)
}

fn blend_scalar(a: f32, b: Option<f32>, t: f32) -> f32 {
    match b {
        Some(b) => a + (b - a) * t.clamp(0.0, 1.0),
        None => a,
    }
}

fn blend_precipitation(
    current: &ResolvedProfile,
    next: Option<&ResolvedProfile>,
    t: f32,
) -> PrecipitationSample {
    let cur = current.precipitation.unwrap_or_default_sample();
    let nxt = next
        .and_then(|p| p.precipitation)
        .unwrap_or_default_sample();
    // Kind transitions: we keep `current.kind` until blend > 0.5, then swap.
    // This avoids cross-fading rain ↔ snow (looks wrong); intensity fades
    // through 0 at the crossover.
    let (kind, intensity_scale) = if t < 0.5 {
        (cur.kind, 1.0 - t * 2.0)
    } else if next.is_some() {
        (nxt.kind, (t - 0.5) * 2.0)
    } else {
        (cur.kind, 1.0)
    };

    let base_intensity = if t < 0.5 { cur.intensity } else { nxt.intensity };
    PrecipitationSample {
        kind,
        intensity: (base_intensity * intensity_scale).clamp(0.0, 1.0),
        wind_drift: if t < 0.5 { cur.wind_drift } else { nxt.wind_drift },
        splash_density: if t < 0.5 { cur.splash_density } else { nxt.splash_density },
    }
}

/// Helper trait used above to coerce `Option<ResolvedPrecipitation>` into a
/// non-None sample with `kind = None, intensity = 0`. Keeping it private to
/// this module so it can't be misused elsewhere.
trait PrecipOptExt {
    fn unwrap_or_default_sample(self) -> PrecipitationSample;
}

impl PrecipOptExt for Option<crate::profile::ResolvedPrecipitation> {
    fn unwrap_or_default_sample(self) -> PrecipitationSample {
        match self {
            Some(p) => PrecipitationSample {
                kind: p.kind,
                intensity: p.intensity,
                wind_drift: p.wind_drift,
                splash_density: p.splash_density,
            },
            None => PrecipitationSample {
                kind: PrecipitationKindSample::None,
                intensity: 0.0,
                wind_drift: 0.0,
                splash_density: 0.0,
            },
        }
    }
}
