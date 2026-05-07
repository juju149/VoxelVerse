#[derive(Clone, Copy, Debug)]
pub struct PlanetProfile {
    pub resolution: u32,
    pub surface_layer: u32,
    pub core_layers: u32,
    pub inner_radius: f32,
    pub surface_radius: f32,
    pub layer_height: f32,
    /// Maximum terrain height offset in layers.  Stored as `i32` and used as
    /// `f32` amplitude — supports the i16 height storage (range up to 32767).
    pub max_terrain_offset: i32,
    pub seed: u32,
}

impl PlanetProfile {
    const DEFAULT_SEED: u32 = 1;

    pub fn new(resolution: u32) -> Self {
        Self::with_seed(resolution, Self::DEFAULT_SEED)
    }

    /// Create a profile for the given resolution and explicit seed.
    pub fn with_seed(resolution: u32, seed: u32) -> Self {
        let resolution = resolution.max(8);
        let surface_layer = resolution / 2;
        let core_layers = 6.min(surface_layer.saturating_sub(2)).max(2);
        let surface_radius = resolution as f32 * 0.5;
        let inner_radius = (surface_radius * 0.18).max(2.0);
        let layer_height = (surface_radius - inner_radius) / surface_layer.max(1) as f32;

        // Scale max_terrain_offset with planet size so large planets have
        // proper mountain relief.  0.008 × res gives ~160 for a 20k-radius
        // planet (≈80 blocks for mountains, ≈10 for plains) — Minecraft-scale.
        let max_terrain_offset =
            ((resolution as f32 * 0.005).round() as i32).clamp(6, 800);

        Self {
            resolution,
            surface_layer,
            core_layers,
            inner_radius,
            surface_radius,
            layer_height,
            max_terrain_offset,
            seed,
        }
    }

    /// Generates a procedural planet radius (in world units = resolution/2) from a seed.
    ///
    /// Distribution: exponential tail, range 5_000..=1_000_000, mean ≈ 50_000.
    /// Returns the **resolution** (diameter in voxels) to pass to `PlanetData::new`.
    ///
    /// ```
    /// // Most planets will be small/medium; occasional giants up to 1M radius.
    /// let resolution = PlanetProfile::procedural_resolution(0x4242_1234);
    /// assert!(resolution >= 10_000 && resolution <= 2_000_000);
    /// ```
    pub fn procedural_resolution(seed: u32) -> u32 {
        // Two LCG rounds to spread seed bits.
        let s = seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223)
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);

        // Map 32-bit integer to [0, 1).
        let u = s as f64 / (u32::MAX as f64 + 1.0);

        // Exponential transform: t ~ Exp(22), E[t] ≈ 0.0455.
        // E[radius] ≈ 5_000 + 995_000 × 0.0455 ≈ 50_270.
        let t = (-(1.0_f64 - u).max(1e-12_f64).ln() / 22.0).min(1.0);
        let radius = 5_000.0 + 995_000.0 * t;

        // Round to nearest 500 for clean numbers.  resolution = radius * 2.
        let radius_rounded = ((radius / 500.0).round() as u32) * 500;
        let radius_clamped = radius_rounded.clamp(5_000, 1_000_000);
        radius_clamped * 2
    }

    pub fn layer_radius(self, layer: u32) -> f32 {
        self.inner_radius + self.layer_height * layer as f32
    }

    pub fn layer_center_radius(self, layer: u32) -> f32 {
        self.layer_radius(layer) + self.layer_height * 0.5
    }

    pub fn radius_to_layer(self, radius: f32) -> Option<(u32, f32)> {
        if radius < self.inner_radius || radius.is_nan() {
            return None;
        }

        let layer_f = (radius - self.inner_radius) / self.layer_height;
        let layer = layer_f.floor() as i32;
        if layer < 0 || layer >= self.resolution as i32 {
            return None;
        }

        Some((layer as u32, layer_f.fract()))
    }

    pub fn spawn_clearance(self) -> f32 {
        self.layer_height * 8.0
    }
}

#[cfg(test)]
mod tests {
    use super::PlanetProfile;

    #[test]
    fn layer_radius_is_monotonic_and_surface_is_stable() {
        let profile = PlanetProfile::new(49);

        assert!(profile.inner_radius > 0.0);
        assert!(profile.layer_height > 0.0);
        assert!(
            (profile.layer_radius(profile.surface_layer) - profile.surface_radius).abs() < 0.001
        );

        for layer in 1..profile.resolution {
            assert!(profile.layer_radius(layer) > profile.layer_radius(layer - 1));
        }
    }

    #[test]
    fn procedural_resolution_in_range() {
        for seed in [0u32, 1, 42, 0xDEAD_BEEF, 0xFFFF_FFFF] {
            let res = PlanetProfile::procedural_resolution(seed);
            // radius = res/2 must be in [5_000, 1_000_000]
            let radius = res / 2;
            assert!(radius >= 5_000, "seed {seed}: radius {radius} < 5000");
            assert!(radius <= 1_000_000, "seed {seed}: radius {radius} > 1_000_000");
        }
    }
}
