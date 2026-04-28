use glam::Vec3;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use vv_config::WorldGenConfig;
use vv_planet::CoordSystem;
use vv_registry::{
    BiomeId, BlockId as ContentBlockId, CompiledBiome, CompiledClimateCurves, CompiledFlora,
    CompiledFloraFeature, CompiledIdealRange, CompiledOre, CompiledPlanetType, PlanetTypeSource,
    TagId, WorldgenContentView, WorldgenSettingsSource,
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

#[derive(Clone, Debug)]
struct TerrainFlora {
    index: u32,
    data: CompiledFlora,
}

#[derive(Clone, Debug)]
struct TerrainOre {
    index: u32,
    data: CompiledOre,
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
    flora: Arc<Vec<TerrainFlora>>,
    ores: Arc<Vec<TerrainOre>>,
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
        let terrain_flora = content
            .flora()
            .enumerate()
            .map(|(index, flora)| TerrainFlora {
                index: index as u32,
                data: flora.data.clone(),
            })
            .collect();
        let terrain_ores = content
            .ores()
            .enumerate()
            .map(|(index, ore)| TerrainOre {
                index: index as u32,
                data: ore.data.clone(),
            })
            .collect();
        println!("Terrain generation ready; columns will be generated lazily.");
        Ok(Self {
            columns: Arc::new(Mutex::new(HashMap::new())),
            biomes: Arc::new(terrain_biomes),
            flora: Arc::new(terrain_flora),
            ores: Arc::new(terrain_ores),
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
        if let Some(ore) = self.ore_block(face, u, v, layer, depth_m, biome) {
            return ore;
        }
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

    pub fn generated_feature_block(
        &self,
        face: u8,
        u: u32,
        v: u32,
        layer: u32,
    ) -> Option<ContentBlockId> {
        if u >= self.resolution || v >= self.resolution {
            return None;
        }
        let column = self.column(face, u, v);
        if layer <= column.height as u32 {
            return None;
        }
        let biome = &self.biomes[column.biome_index as usize];
        for flora in self.flora.iter() {
            if !tags_match(
                &flora.data.required_tags,
                &flora.data.forbidden_tags,
                &biome.data.provided_tags,
            ) {
                continue;
            }
            if let Some(block) = self.flora_block(face, u, v, layer, biome, flora) {
                return Some(block);
            }
        }
        None
    }

    pub fn feature_candidate_layers(
        &self,
        face: u8,
        u: u32,
        v: u32,
    ) -> std::ops::RangeInclusive<u32> {
        let h = self.get_height(face, u, v);
        h.saturating_add(1)..=h.saturating_add(8)
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

    fn ore_block(
        &self,
        face: u8,
        u: u32,
        v: u32,
        layer: u32,
        depth_m: f32,
        biome: &TerrainBiome,
    ) -> Option<ContentBlockId> {
        if depth_m <= 0.0 {
            return None;
        }
        for ore in self.ores.iter() {
            let vein = ore.data.vein;
            if depth_m < vein.depth_min_m || depth_m > vein.depth_max_m {
                continue;
            }
            if !tags_match(
                &ore.data.required_tags,
                &ore.data.forbidden_tags,
                &biome.data.provided_tags,
            ) {
                continue;
            }
            let chance = (vein.frequency * 0.035).clamp(0.0, 0.35);
            if hash01(face, u, v, layer, ore.index) < chance {
                return Some(ore.data.block);
            }
        }
        None
    }

    fn flora_block(
        &self,
        face: u8,
        u: u32,
        v: u32,
        layer: u32,
        biome: &TerrainBiome,
        flora: &TerrainFlora,
    ) -> Option<ContentBlockId> {
        match flora.data.feature {
            CompiledFloraFeature::Plant {
                block,
                height_min: _,
                height_max,
            } => {
                let surface = self.column(face, u, v).height as u32;
                if layer <= surface + height_max.max(1)
                    && layer == surface + 1
                    && self.flora_origin(face, u, v, biome, flora)
                {
                    return Some(block);
                }
                None
            }
            CompiledFloraFeature::Tree {
                log_block,
                leaf_block,
                trunk_height_min,
                trunk_height_max,
                canopy_radius,
                canopy_height,
            } => {
                if self.flora_origin(face, u, v, biome, flora) {
                    let surface = self.column(face, u, v).height as u32;
                    let trunk_height =
                        ranged_u32(face, u, v, flora.index, trunk_height_min, trunk_height_max);
                    if layer > surface && layer <= surface + trunk_height {
                        return Some(log_block);
                    }
                }

                let radius = canopy_radius.ceil().max(1.0) as i32;
                for du in -radius..=radius {
                    for dv in -radius..=radius {
                        let ou = u as i32 + du;
                        let ov = v as i32 + dv;
                        if ou < 0
                            || ov < 0
                            || ou >= self.resolution as i32
                            || ov >= self.resolution as i32
                        {
                            continue;
                        }
                        let ou = ou as u32;
                        let ov = ov as u32;
                        if !self.flora_origin(face, ou, ov, biome, flora) {
                            continue;
                        }
                        let origin_surface = self.column(face, ou, ov).height as u32;
                        let trunk_height = ranged_u32(
                            face,
                            ou,
                            ov,
                            flora.index,
                            trunk_height_min,
                            trunk_height_max,
                        );
                        let canopy_center = origin_surface + trunk_height;
                        let vertical = layer.abs_diff(canopy_center) as f32;
                        let horizontal = ((du * du + dv * dv) as f32).sqrt();
                        if horizontal <= canopy_radius && vertical <= canopy_height.max(1.0) {
                            return Some(leaf_block);
                        }
                    }
                }
                None
            }
            CompiledFloraFeature::Cluster {
                block, radius_max, ..
            } => {
                let surface = self.column(face, u, v).height as u32;
                if layer != surface + 1 {
                    return None;
                }
                let radius = radius_max.ceil().max(1.0) as i32;
                for du in -radius..=radius {
                    for dv in -radius..=radius {
                        let ou = u as i32 + du;
                        let ov = v as i32 + dv;
                        if ou < 0
                            || ov < 0
                            || ou >= self.resolution as i32
                            || ov >= self.resolution as i32
                        {
                            continue;
                        }
                        if self.flora_origin(face, ou as u32, ov as u32, biome, flora) {
                            return Some(block);
                        }
                    }
                }
                None
            }
        }
    }

    fn flora_origin(
        &self,
        face: u8,
        u: u32,
        v: u32,
        biome: &TerrainBiome,
        flora: &TerrainFlora,
    ) -> bool {
        let placement = flora.data.placement;
        let surface = self.column(face, u, v).height as f32 * self.voxel_size_m;
        if placement
            .altitude_max
            .is_some_and(|altitude_max| surface > altitude_max)
        {
            return false;
        }
        tags_match(
            &flora.data.required_tags,
            &flora.data.forbidden_tags,
            &biome.data.provided_tags,
        ) && hash01(face, u, v, 0, flora.index) < placement.density_base.clamp(0.0, 1.0)
    }
}

impl Clone for PlanetTerrain {
    fn clone(&self) -> Self {
        Self {
            columns: self.columns.clone(),
            biomes: self.biomes.clone(),
            flora: self.flora.clone(),
            ores: self.ores.clone(),
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

fn tags_match(required: &[TagId], forbidden: &[TagId], provided: &[TagId]) -> bool {
    required.iter().all(|tag| provided.contains(tag))
        && forbidden.iter().all(|tag| !provided.contains(tag))
}

fn hash01(face: u8, u: u32, v: u32, layer: u32, salt: u32) -> f32 {
    let mut x = face as u64;
    x = x.wrapping_mul(0x9E37_79B1_85EB_CA87) ^ u as u64;
    x = x.wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ v as u64;
    x = x.wrapping_mul(0x1656_67B1_9E37_79F9) ^ layer as u64;
    x = x.wrapping_mul(0x85EB_CA77_C2B2_AE63) ^ salt as u64;
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    ((x & 0xFFFF_FFFF) as f32) / u32::MAX as f32
}

fn ranged_u32(face: u8, u: u32, v: u32, salt: u32, min: u32, max: u32) -> u32 {
    if min >= max {
        return min;
    }
    let span = max - min + 1;
    min + (hash01(face, u, v, 1, salt) * span as f32).floor() as u32 % span
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

    #[test]
    fn alpha_resources_are_generated_from_content_defs() {
        let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let content = compile_assets_root(&assets).expect("core content should compile");
        let worldgen = content.worldgen_content();
        let resolution = 64;
        let terrain = PlanetTerrain::generate(
            resolution,
            &WorldGenConfig::default(),
            &worldgen,
            content.world.voxel_size_m,
        )
        .expect("terrain should generate");

        let wood_log = content
            .blocks
            .id(&ContentKey::from_str("voxelverse:wood_log").unwrap())
            .expect("wood_log block");
        let coal_ore = content
            .blocks
            .id(&ContentKey::from_str("voxelverse:coal_ore").unwrap())
            .expect("coal_ore block");
        let iron_ore = content
            .blocks
            .id(&ContentKey::from_str("voxelverse:iron_ore").unwrap())
            .expect("iron_ore block");
        let water = content
            .blocks
            .id(&ContentKey::from_str("voxelverse:water").unwrap())
            .expect("water block");

        let mut found_wood = false;
        let mut found_coal = false;
        let mut found_iron = false;
        let mut found_water = false;

        for face in 0..6 {
            for u in (0..resolution).step_by(2) {
                for v in (0..resolution).step_by(2) {
                    for layer in terrain.feature_candidate_layers(face, u, v) {
                        if terrain.generated_feature_block(face, u, v, layer) == Some(wood_log) {
                            found_wood = true;
                        }
                    }
                    if terrain.get_surface_block(face, u, v) == water {
                        found_water = true;
                    }

                    let height = terrain.get_height(face, u, v);
                    for depth in 1..=32 {
                        let Some(layer) = height.checked_sub(depth) else {
                            continue;
                        };
                        match terrain.get_block(face, u, v, layer) {
                            block if block == coal_ore => found_coal = true,
                            block if block == iron_ore => found_iron = true,
                            _ => {}
                        }
                    }
                }
            }
        }

        assert!(found_wood, "wood should be generated by flora defs");
        assert!(found_coal, "coal should be generated by ore defs");
        assert!(found_iron, "iron should be generated by ore defs");
        assert!(
            found_water,
            "water should be generated by biome surface defs"
        );
    }
}
