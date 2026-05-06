#[derive(Clone, Copy, Debug)]
pub struct PlanetProfile {
    pub resolution: u32,
    pub surface_layer: u32,
    pub core_layers: u32,
    pub inner_radius: f32,
    pub surface_radius: f32,
    pub layer_height: f32,
    pub max_terrain_offset: i32,
    pub seed: u32,
}

impl PlanetProfile {
    const DEFAULT_SEED: u32 = 42;

    pub fn new(resolution: u32) -> Self {
        let resolution = resolution.max(8);
        let surface_layer = resolution / 2;
        let core_layers = 6.min(surface_layer.saturating_sub(2)).max(2);
        let surface_radius = resolution as f32 * 0.5;
        let inner_radius = (surface_radius * 0.18).max(2.0);
        let layer_height = (surface_radius - inner_radius) / surface_layer.max(1) as f32;
        let max_terrain_offset = ((resolution as f32 * 0.16).round() as i32).clamp(2, 14);

        Self {
            resolution,
            surface_layer,
            core_layers,
            inner_radius,
            surface_radius,
            layer_height,
            max_terrain_offset,
            seed: Self::DEFAULT_SEED,
        }
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
}
