#[derive(Clone, Copy, Debug)]
pub struct PlanetProfile {
    pub resolution: u32,
    pub surface_layer: u32,
    pub core_layers: u32,
    pub voxel_size_meters: f32,
    pub edge_rounding_radius_voxels: f32,
    pub inner_radius: f32,
    pub surface_radius: f32,
    pub layer_height: f32,
    /// Maximum terrain height offset in layers.  Stored as `i32` and used as
    /// `f32` amplitude — supports the i16 height storage (range up to 32767).
    pub max_terrain_offset: i32,
    pub spawn_clearance_layers: f32,
    pub seed: u32,
}

impl PlanetProfile {
    #[cfg(test)]
    const DEFAULT_SEED: u32 = 1;
    #[cfg(test)]
    const DEFAULT_VOXEL_SIZE_METERS: f32 = 1.0;
    #[cfg(test)]
    const DEFAULT_INNER_RADIUS_FRACTION: f32 = 0.18;

    #[cfg(test)]
    pub fn new(resolution: u32) -> Self {
        Self::with_seed(resolution, Self::DEFAULT_SEED)
    }

    #[cfg(test)]
    pub fn with_seed(resolution: u32, seed: u32) -> Self {
        Self::with_seed_and_voxel_size(resolution, seed, Self::DEFAULT_VOXEL_SIZE_METERS)
    }

    #[cfg(test)]
    pub fn with_seed_and_voxel_size(resolution: u32, seed: u32, voxel_size_meters: f32) -> Self {
        let resolution = resolution.max(8);
        let surface_layer = resolution / 2;
        let core_layers = 6.min(surface_layer.saturating_sub(2)).max(2);
        let voxel_size_meters = if voxel_size_meters.is_finite() && voxel_size_meters > 0.0 {
            voxel_size_meters
        } else {
            Self::DEFAULT_VOXEL_SIZE_METERS
        };
        let (inner_radius, surface_radius, layer_height) =
            radii_from_layers(surface_layer, voxel_size_meters, Self::DEFAULT_INNER_RADIUS_FRACTION);

        let max_terrain_offset = ((resolution as f32 * 0.005).round() as i32).clamp(6, 800);

        Self {
            resolution,
            surface_layer,
            core_layers,
            voxel_size_meters,
            edge_rounding_radius_voxels: 0.16,
            inner_radius,
            surface_radius,
            layer_height,
            max_terrain_offset,
            spawn_clearance_layers: 8.0,
            seed,
        }
    }

    #[allow(dead_code)]
    pub fn procedural_resolution(seed: u32) -> u32 {
        let s = seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223)
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let u = s as f64 / (u32::MAX as f64 + 1.0);
        let t = (-(1.0_f64 - u).max(1e-12_f64).ln() / 22.0).min(1.0);
        let radius = 5_000.0 + 995_000.0 * t;
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
        self.layer_height * self.spawn_clearance_layers
    }
}

pub(crate) fn radii_from_layers(
    surface_layer: u32,
    voxel_size_meters: f32,
    inner_radius_fraction: f32,
) -> (f32, f32, f32) {
    let fraction = inner_radius_fraction.clamp(0.02, 0.95);
    let shell_depth = surface_layer.max(1) as f32 * voxel_size_meters;
    let surface_radius = shell_depth / (1.0 - fraction);
    let inner_radius = surface_radius * fraction;
    (inner_radius, surface_radius, voxel_size_meters)
}

#[cfg(test)]
mod tests {
    use super::PlanetProfile;

    #[test]
    fn layer_radius_is_monotonic_and_surface_is_stable() {
        let profile = PlanetProfile::new(49);
        assert!(profile.inner_radius > 0.0);
        assert!(profile.layer_height > 0.0);
        assert_eq!(profile.layer_height, profile.voxel_size_meters);
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
            let radius = res / 2;
            assert!(radius >= 5_000, "seed {seed}: radius {radius} < 5000");
            assert!(radius <= 1_000_000, "seed {seed}: radius {radius} > 1_000_000");
        }
    }

    #[test]
    fn configured_voxel_size_controls_layer_spacing() {
        let profile = PlanetProfile::with_seed_and_voxel_size(64, 1, 0.5);
        assert_eq!(profile.voxel_size_meters, 0.5);
        assert_eq!(profile.layer_height, 0.5);
        assert!((profile.layer_radius(11) - profile.layer_radius(10) - 0.5).abs() < 0.0001);
    }
}
