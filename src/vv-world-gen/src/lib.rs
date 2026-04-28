use glam::Vec3;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use vv_config::WorldGenConfig;
use vv_planet::CoordSystem;
use vv_registry::{
    BiomeId, BlockId as ContentBlockId, CompiledBiome, CompiledClimateCurves, CompiledIdealRange,
    CompiledPlanetType, PlanetTypeSource, WorldgenContentView, WorldgenSettingsSource,
};

#[derive(Clone, Copy, Debug)]
pub struct TerrainColumn {
    pub height: u16,
    biome_index: u16,
}

#[derive(Clone, Debug)]
struct TerrainBiome {
    id: BiomeId,
    data: CompiledBiome,
}

#[derive(Clone, Copy, Debug)]
struct TerrainNoiseConfig {
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
}

/// Deterministic lazy terrain for a planet.
///
/// Content data is captured from runtime registries at construction time, then
/// columns are computed on demand and cached as chunks or LOD tiles request them.
pub struct PlanetTerrain {
    columns: Arc<Mutex<HashMap<u64, TerrainColumn>>>,
    biomes: Arc<Vec<TerrainBiome>>,
    planet: CompiledPlanetType,
    climate_curves: CompiledClimateCurves,
    generator: Arc<NoiseGenerator>,
    noise: TerrainNoiseConfig,
    resolution: u32,
    voxel_size_m: f32,
}

impl PlanetTerrain {
    pub fn generate(
        resolution: u32,
        cfg: &WorldGenConfig,
        content: &WorldgenContentView<'_>,
        voxel_size_m: f32,
    ) -> Result<Self, TerrainGenerationError> {
        println!(
            "Generating registry-driven terrain map (res {})...",
            resolution
        );
        let default_planet = content
            .default_planet_type()
            .ok_or(TerrainGenerationError::MissingDefaultPlanetType)?;
        let planet = content
            .planet_type(default_planet)
            .ok_or(TerrainGenerationError::MissingPlanetType(default_planet))?;
        let biome_views: Vec<_> = content.biomes().collect();
        if biome_views.is_empty() {
            return Err(TerrainGenerationError::NoBiomes);
        }
        if biome_views
            .iter()
            .any(|biome| biome.data.surface_layers.is_empty())
        {
            return Err(TerrainGenerationError::BiomeWithoutSurfaceLayer);
        }

        let terrain_biomes: Vec<_> = biome_views
            .iter()
            .map(|biome| TerrainBiome {
                id: biome.id,
                data: biome.data.clone(),
            })
            .collect();
        println!("Terrain generation ready; columns will be generated lazily.");
        Ok(Self {
            columns: Arc::new(Mutex::new(HashMap::new())),
            biomes: Arc::new(terrain_biomes),
            planet: planet.data.clone(),
            climate_curves: *content.climate_curves(),
            generator: Arc::new(NoiseGenerator::new(cfg.noise_seed)),
            noise: TerrainNoiseConfig {
                octaves: cfg.noise_octaves,
                persistence: cfg.noise_persistence,
                lacunarity: cfg.noise_lacunarity,
            },
            resolution,
            voxel_size_m,
        })
    }

    #[inline(always)]
    fn cache_key(face: u8, u: u32, v: u32) -> u64 {
        ((face as u64) << 56) | ((u as u64) << 28) | v as u64
    }

    pub fn get_height(&self, face: u8, u: u32, v: u32) -> u32 {
        self.column(face, u, v).height as u32
    }

    pub fn get_surface_block(&self, face: u8, u: u32, v: u32) -> ContentBlockId {
        let column = self.column(face, u, v);
        self.get_block(face, u, v, column.height as u32)
    }

    pub fn get_biome(&self, face: u8, u: u32, v: u32) -> BiomeId {
        let column = self.column(face, u, v);
        self.biomes[column.biome_index as usize].id
    }

    pub fn get_block(&self, face: u8, u: u32, v: u32, layer: u32) -> ContentBlockId {
        let column = self.column(face, u, v);
        let biome = &self.biomes[column.biome_index as usize];
        let depth_m = column.height.saturating_sub(layer as u16) as f32 * self.voxel_size_m;
        let mut accumulated_depth = 0.0;
        for surface_layer in &biome.data.surface_layers {
            match surface_layer.depth_m {
                Some(layer_depth) => {
                    accumulated_depth += layer_depth.max(0.0);
                    if depth_m <= accumulated_depth {
                        return surface_layer.block;
                    }
                }
                None => return surface_layer.block,
            }
        }
        biome
            .data
            .surface_layers
            .last()
            .expect("terrain biome should have surface layers")
            .block
    }

    fn column(&self, face: u8, u: u32, v: u32) -> TerrainColumn {
        let u = u.min(self.resolution - 1);
        let v = v.min(self.resolution - 1);
        let key = Self::cache_key(face, u, v);
        if let Some(column) = self
            .columns
            .lock()
            .expect("terrain cache should not be poisoned")
            .get(&key)
            .copied()
        {
            return column;
        }

        let column = self.compute_column(face, u, v);
        self.columns
            .lock()
            .expect("terrain cache should not be poisoned")
            .insert(key, column);
        column
    }

    fn compute_column(&self, face: u8, u: u32, v: u32) -> TerrainColumn {
        let dir = CoordSystem::get_direction(face, u, v, self.resolution);
        let climate =
            ClimateSample::sample(dir, &self.generator, self.climate_curves, &self.planet);
        let (biome_index, biome) = choose_biome(&self.biomes, climate);
        let relief_noise = self.generator.fractal(
            dir,
            self.resolution as f32 / self.climate_curves.minimum_biome_transition_m,
            self.noise.octaves,
            self.noise.persistence,
            self.noise.lacunarity,
        );
        let relief = biome.data.relief;
        let base_radius = self.resolution as f32 / 2.0;
        let height_delta = relief.base_height_m
            + centered(relief_noise)
                * relief.height_variance_m
                * relief.roughness.max(0.0)
                * self.planet.altitude_variance_multiplier;
        TerrainColumn {
            height: (base_radius + height_delta).max(1.0) as u16,
            biome_index: biome_index as u16,
        }
    }
}

impl Clone for PlanetTerrain {
    fn clone(&self) -> Self {
        Self {
            columns: self.columns.clone(),
            biomes: self.biomes.clone(),
            planet: self.planet.clone(),
            climate_curves: self.climate_curves,
            generator: self.generator.clone(),
            noise: self.noise,
            resolution: self.resolution,
            voxel_size_m: self.voxel_size_m,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainGenerationError {
    MissingDefaultPlanetType,
    MissingPlanetType(vv_registry::PlanetTypeId),
    NoBiomes,
    BiomeWithoutSurfaceLayer,
}

impl std::fmt::Display for TerrainGenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerrainGenerationError::MissingDefaultPlanetType => {
                write!(f, "worldgen content has no default planet type")
            }
            TerrainGenerationError::MissingPlanetType(id) => {
                write!(
                    f,
                    "worldgen content references missing planet type {:?}",
                    id
                )
            }
            TerrainGenerationError::NoBiomes => write!(f, "worldgen content has no biomes"),
            TerrainGenerationError::BiomeWithoutSurfaceLayer => {
                write!(f, "worldgen biome has no surface layers")
            }
        }
    }
}

impl std::error::Error for TerrainGenerationError {}

#[derive(Clone, Copy)]
struct ClimateSample {
    temperature: f32,
    humidity: f32,
    altitude: f32,
}

impl ClimateSample {
    fn sample(
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

fn choose_biome(biomes: &[TerrainBiome], climate: ClimateSample) -> (usize, &TerrainBiome) {
    biomes
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            score_biome(&a.data, climate)
                .partial_cmp(&score_biome(&b.data, climate))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("biomes should not be empty")
}

fn score_biome(biome: &CompiledBiome, climate: ClimateSample) -> f32 {
    biome.weight.max(0.0)
        * ideal_score(biome.climate.temperature, climate.temperature)
        * ideal_score(biome.climate.humidity, climate.humidity)
        * ideal_score(biome.climate.altitude, climate.altitude)
}

fn ideal_score(range: CompiledIdealRange, value: f32) -> f32 {
    if value < range.min || value > range.max {
        return 0.0;
    }
    if value >= range.ideal_min && value <= range.ideal_max {
        return 1.0;
    }
    if value < range.ideal_min {
        let width = (range.ideal_min - range.min).max(f32::EPSILON);
        return ((value - range.min) / width).clamp(0.0, 1.0);
    }
    let width = (range.max - range.ideal_max).max(f32::EPSILON);
    ((range.max - value) / width).clamp(0.0, 1.0)
}

fn centered(value: f32) -> f32 {
    value * 2.0 - 1.0
}

struct NoiseGenerator {
    perm: [u8; 512],
}

impl NoiseGenerator {
    fn new(seed: u32) -> Self {
        let mut permutation: Vec<u8> = (0u8..=255).collect();
        let mut state = seed;
        for i in (1..256).rev() {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let j = (state as usize) % (i + 1);
            permutation.swap(i, j);
        }
        let mut p = [0u8; 512];
        for i in 0..256 {
            p[i] = permutation[i];
            p[i + 256] = permutation[i];
        }
        Self { perm: p }
    }

    fn fractal(
        &self,
        pos: Vec3,
        scale: f32,
        octaves: u32,
        persistence: f32,
        lacunarity: f32,
    ) -> f32 {
        let mut total = 0.0f32;
        let mut total_amp = 0.0f32;
        let mut amp = 1.0f32;
        let mut freq = scale.max(f32::EPSILON);
        for _ in 0..octaves.max(1) {
            total += self.perlin(pos * freq) * amp;
            total_amp += amp;
            amp *= persistence;
            freq *= lacunarity;
        }
        if total_amp > 0.0 {
            total / total_amp
        } else {
            0.0
        }
    }

    fn perlin(&self, pos: Vec3) -> f32 {
        let xi = pos.x.floor();
        let yi = pos.y.floor();
        let zi = pos.z.floor();
        let x_int = xi as i32 & 255;
        let y_int = yi as i32 & 255;
        let z_int = zi as i32 & 255;
        let x = pos.x - xi;
        let y = pos.y - yi;
        let z = pos.z - zi;
        let u = fade(x);
        let v = fade(y);
        let w = fade(z);
        let a = self.perm[x_int as usize] as usize + y_int as usize;
        let aa = self.perm[a] as usize + z_int as usize;
        let ab = self.perm[a + 1] as usize + z_int as usize;
        let b = self.perm[x_int as usize + 1] as usize + y_int as usize;
        let ba = self.perm[b] as usize + z_int as usize;
        let bb = self.perm[b + 1] as usize + z_int as usize;
        (lerp(
            w,
            lerp(
                v,
                lerp(
                    u,
                    grad(self.perm[aa], x, y, z),
                    grad(self.perm[ba], x - 1.0, y, z),
                ),
                lerp(
                    u,
                    grad(self.perm[ab], x, y - 1.0, z),
                    grad(self.perm[bb], x - 1.0, y - 1.0, z),
                ),
            ),
            lerp(
                v,
                lerp(
                    u,
                    grad(self.perm[aa + 1], x, y, z - 1.0),
                    grad(self.perm[ba + 1], x - 1.0, y, z - 1.0),
                ),
                lerp(
                    u,
                    grad(self.perm[ab + 1], x, y - 1.0, z - 1.0),
                    grad(self.perm[bb + 1], x - 1.0, y - 1.0, z - 1.0),
                ),
            ),
        ) + 1.0)
            * 0.5
    }
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

fn grad(hash: u8, x: f32, y: f32, z: f32) -> f32 {
    let h = (hash & 15) as i32;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };
    (if (h & 1) != 0 { -u } else { u }) + (if (h & 2) != 0 { -v } else { v })
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::*;
    use vv_compiler::compile_assets_root;
    use vv_registry::ContentKey;

    #[test]
    fn terrain_generation_is_deterministic_and_uses_registry_surface_block() {
        let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let content = compile_assets_root(&assets).expect("core content should compile");
        let worldgen = content.worldgen_content();
        let resolution = 16;
        let cfg = WorldGenConfig::default();
        let a = PlanetTerrain::generate(resolution, &cfg, &worldgen, content.world.voxel_size_m)
            .expect("terrain a");
        let b = PlanetTerrain::generate(resolution, &cfg, &worldgen, content.world.voxel_size_m)
            .expect("terrain b");

        for (face, u, v) in [(0, 4, 4), (1, 7, 3), (5, 9, 12)] {
            assert_eq!(a.get_height(face, u, v), b.get_height(face, u, v));
            assert_eq!(a.get_biome(face, u, v), b.get_biome(face, u, v));
            assert_eq!(
                a.get_surface_block(face, u, v),
                b.get_surface_block(face, u, v)
            );
        }

        let stone = content
            .blocks
            .id(&ContentKey::from_str("voxelverse:stone").unwrap())
            .expect("stone block");
        assert_eq!(a.get_surface_block(0, 4, 4), stone);
    }
}
