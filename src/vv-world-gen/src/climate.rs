use glam::Vec3;
use vv_registry::{
    BiomeId, CompiledBiome, CompiledClimateCurves, CompiledIdealRange, CompiledPlanetType,
};

use crate::{centered, smoothstep, NoiseGenerator};

#[derive(Clone, Debug)]
pub(crate) struct TerrainBiome {
    pub id: BiomeId,
    pub data: CompiledBiome,
}

#[derive(Clone, Copy)]
pub(crate) struct ClimateSample {
    temperature: f32,
    humidity: f32,
    altitude: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct BiomeBlend {
    pub dominant_index: usize,
    pub weights: Vec<BiomeWeight>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BiomeWeight {
    pub index: usize,
    pub weight: f32,
}

impl ClimateSample {
    pub(crate) fn sample(
        dir: Vec3,
        generator: &NoiseGenerator,
        curves: CompiledClimateCurves,
        planet: &CompiledPlanetType,
    ) -> Self {
        let latitude = dir.y.abs();

        let temperature_noise = generator.fractal(dir, curves.temperature_noise_scale, 3, 0.5, 2.0);

        let humidity_noise = generator.fractal(dir, curves.humidity_noise_scale, 3, 0.5, 2.0);

        Self {
            temperature: (temperature_noise + planet.temperature_bias - latitude * 0.35)
                .clamp(0.0, 1.0),
            humidity: (humidity_noise + planet.humidity_bias).clamp(0.0, 1.0),
            altitude: centered(generator.fractal(
                dir,
                curves.minimum_biome_transition_m,
                2,
                0.5,
                2.0,
            ))
            .abs()
            .clamp(0.0, 1.0),
        }
    }
}

pub(crate) fn choose_biome_blend(biomes: &[TerrainBiome], climate: ClimateSample) -> BiomeBlend {
    let mut scored = Vec::with_capacity(biomes.len());
    let mut dominant_index = 0usize;
    let mut max_score = 0.0;

    for (index, biome) in biomes.iter().enumerate() {
        let score = score_biome(&biome.data, climate);

        if score > max_score {
            max_score = score;
            dominant_index = index;
        }

        scored.push((index, score));
    }

    if max_score <= f32::EPSILON {
        dominant_index = nearest_biome_index(biomes, climate);

        return BiomeBlend {
            dominant_index,
            weights: vec![BiomeWeight {
                index: dominant_index,
                weight: 1.0,
            }],
        };
    }

    let mut weights = Vec::with_capacity(scored.len());
    let mut total = 0.0;

    for (index, score) in scored {
        if score <= f32::EPSILON {
            continue;
        }

        weights.push(BiomeWeight {
            index,
            weight: score,
        });

        total += score;
    }

    if total <= f32::EPSILON {
        return BiomeBlend {
            dominant_index,
            weights: vec![BiomeWeight {
                index: dominant_index,
                weight: 1.0,
            }],
        };
    }

    for entry in &mut weights {
        entry.weight /= total;
    }

    BiomeBlend {
        dominant_index,
        weights,
    }
}

fn score_biome(biome: &CompiledBiome, climate: ClimateSample) -> f32 {
    const MAX_CLIMATE_BLEND_DISTANCE: f32 = 0.46;

    let distance = biome_climate_distance(biome, climate).sqrt();
    let compatibility = smoothstep((1.0 - distance / MAX_CLIMATE_BLEND_DISTANCE).clamp(0.0, 1.0));

    biome.weight.max(0.0) * compatibility
}

fn nearest_biome_index(biomes: &[TerrainBiome], climate: ClimateSample) -> usize {
    biomes
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            biome_climate_distance(&a.data, climate)
                .partial_cmp(&biome_climate_distance(&b.data, climate))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(index, _)| index)
        .expect("biomes should not be empty")
}

fn biome_climate_distance(biome: &CompiledBiome, climate: ClimateSample) -> f32 {
    ideal_distance(biome.climate.temperature, climate.temperature).powi(2)
        + ideal_distance(biome.climate.humidity, climate.humidity).powi(2)
        + ideal_distance(biome.climate.altitude, climate.altitude).powi(2)
}

fn ideal_distance(range: CompiledIdealRange, value: f32) -> f32 {
    if value < range.ideal_min {
        range.ideal_min - value
    } else if value > range.ideal_max {
        value - range.ideal_max
    } else {
        0.0
    }
}
