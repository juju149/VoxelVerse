use vv_registry::{CompiledPlanetType, CompiledWorldSettings};

use crate::hash01;

pub(crate) fn deterministic_planet_radius_m(
    planet: &CompiledPlanetType,
    seed: u32,
    settings: &CompiledWorldSettings,
) -> f32 {
    let min_m = planet.min_radius_km.max(0.001) * 1_000.0;
    let max_m = planet.max_radius_km.max(planet.min_radius_km).max(0.001) * 1_000.0;

    let t = hash01(
        (seed & 0xFF) as u8,
        seed.rotate_left(7),
        seed.rotate_right(9),
        0,
        0,
    );

    let radius = min_m + (max_m - min_m) * t;
    radius.min(settings.max_planet_radius_km.max(0.001) * 1_000.0)
}
