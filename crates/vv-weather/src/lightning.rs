//! Poisson lightning sampler.
//!
//! Each tick we sample at most one strike per profile (V1 cap). The rate
//! comes from the profile's `lightning.strikes_per_minute`; strike positions
//! are uniform within a radius around the observer, with a per-strike random
//! flash variation.

use crate::profile::ResolvedProfile;
use crate::rng::PcgRng;
use crate::snapshot::LightningStrike;

/// Default strike spawn radius around the observer in metres.
/// Drives audio delay range too — at 300 m, a strike with the default
/// `thunder_delay_per_km = 3 s/km` lands at ~0.9 s after the flash.
pub const STRIKE_RADIUS_M: f32 = 300.0;

/// Sample at most one strike for the current tick.
pub fn sample(
    dt: f32,
    profile: &ResolvedProfile,
    observer_pos: glam::Vec3,
    rng: &mut PcgRng,
) -> Option<LightningStrike> {
    let cfg = profile.lightning.as_ref()?;
    if cfg.strikes_per_minute <= 0.0 || dt <= 0.0 {
        return None;
    }

    // Bernoulli per-tick approximation of a Poisson process with rate
    // λ = strikes_per_minute / 60. Probability of ≥ 1 event in `dt` is
    // 1 - e^(-λ·dt); for small λ·dt this is ≈ λ·dt, accurate enough at 60 Hz.
    let lambda_per_s = cfg.strikes_per_minute / 60.0;
    let p = 1.0 - (-lambda_per_s * dt).exp();
    if rng.next_unit() >= p {
        return None;
    }

    // Uniform-in-disc placement on the XZ plane.
    let theta = rng.next_unit() * std::f32::consts::TAU;
    let r = rng.next_unit().sqrt() * STRIKE_RADIUS_M;
    let offset = glam::Vec3::new(theta.cos() * r, 0.0, theta.sin() * r);
    let position = observer_pos + offset;
    let distance_m = offset.length();

    // Flash gets a ±10 % jitter so successive strikes don't look identical.
    let flash = cfg.flash_intensity.max(0.0) * rng.next_range(0.9, 1.1);
    let thunder_delay_s = (distance_m / 1000.0) * cfg.thunder_delay_per_km.max(0.0);

    Some(LightningStrike {
        position,
        distance_m,
        flash_intensity: flash,
        thunder_delay_s,
    })
}
