use vv_planet::CoordSystem;

use crate::{
    centered,
    climate::{choose_biome_blend, ClimateSample},
};

use super::{PlanetTerrain, TerrainColumn};

impl PlanetTerrain {
    pub(crate) fn compute_column(&self, face: u8, u: u32, v: u32) -> TerrainColumn {
        let dir = CoordSystem::get_direction(face, u, v, self.geometry.resolution);

        let climate =
            ClimateSample::sample(dir, &self.generator, self.climate_curves, &self.planet);

        let blend = choose_biome_blend(&self.biomes, climate);

        let relief_noise = self.generator.fractal(
            dir * self.geometry.radius_m,
            1.0 / self
                .climate_curves
                .minimum_biome_transition_m
                .max(self.geometry.voxel_size_m),
            self.noise.octaves,
            self.noise.persistence,
            self.noise.lacunarity,
        );

        let surface_layer = self.geometry.surface_layer() as f32;

        let height_delta = blend
            .weights
            .iter()
            .map(|entry| {
                let relief = self.biomes[entry.index].data.relief;

                entry.weight
                    * (relief.base_height_m
                        + centered(relief_noise)
                            * relief.height_variance_m
                            * relief.roughness.max(0.0)
                            * self.planet.altitude_variance_multiplier)
            })
            .sum::<f32>();

        let height_delta_layers = height_delta / self.geometry.voxel_size_m;

        TerrainColumn {
            height: (surface_layer + height_delta_layers).max(1.0) as u16,
            biome_index: blend.dominant_index as u16,
        }
    }
}
