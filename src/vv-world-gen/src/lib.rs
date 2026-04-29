use glam::Vec3;
mod tree;

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
    time::Instant,
};
use tree::{TreeShape, TreeShapeConfig};
use vv_config::WorldGenConfig;
use vv_core::BlockId;
use vv_planet::{CoordSystem, PlanetGeometry};
use vv_registry::{
    BiomeId, BlockId as ContentBlockId, CompiledBiome, CompiledClimateCurves, CompiledFlora,
    CompiledFloraFeature, CompiledIdealRange, CompiledOre, CompiledPlanetType,
    CompiledWorldSettings, PlanetTypeSource, TagId, WorldgenContentView, WorldgenSettingsSource,
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

#[derive(Clone, Debug)]
struct BiomeBlend {
    dominant_index: usize,
    weights: Vec<BiomeWeight>,
}

#[derive(Clone, Copy, Debug)]
struct BiomeWeight {
    index: usize,
    weight: f32,
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
    columns: Arc<RwLock<HashMap<u64, TerrainColumn>>>,
    cache_hits: Arc<AtomicU64>,
    cache_misses: Arc<AtomicU64>,
    cache_compute_micros: Arc<AtomicU64>,
    biomes: Arc<Vec<TerrainBiome>>,
    flora: Arc<Vec<TerrainFlora>>,
    ores: Arc<Vec<TerrainOre>>,
    planet: CompiledPlanetType,
    climate_curves: CompiledClimateCurves,
    generator: Arc<NoiseGenerator>,
    noise: TerrainNoiseConfig,
    geometry: PlanetGeometry,
    world_seed: u32,
    max_feature_height_m: f32,
    max_feature_radius_m: f32,
}

impl PlanetTerrain {
    pub fn generate(
        cfg: &WorldGenConfig,
        content: &WorldgenContentView<'_>,
        settings: &CompiledWorldSettings,
    ) -> Result<Self, TerrainGenerationError> {
        let default_planet = content
            .default_planet_type()
            .ok_or(TerrainGenerationError::MissingDefaultPlanetType)?;
        let planet = content
            .planet_type(default_planet)
            .ok_or(TerrainGenerationError::MissingPlanetType(default_planet))?;
        let radius_m = deterministic_planet_radius_m(&planet.data, cfg.noise_seed, settings);
        let geometry = PlanetGeometry::new(radius_m, settings.voxel_size_m);
        Self::generate_for_geometry(geometry, cfg, content)
    }

    pub fn generate_for_geometry(
        geometry: PlanetGeometry,
        cfg: &WorldGenConfig,
        content: &WorldgenContentView<'_>,
    ) -> Result<Self, TerrainGenerationError> {
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
        let terrain_flora: Vec<_> = content
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
        let max_feature_height_m = max_feature_height_m(&terrain_flora);
        let max_feature_radius_m = max_feature_radius_m(&terrain_flora);
        Ok(Self {
            columns: Arc::new(RwLock::new(HashMap::new())),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            cache_compute_micros: Arc::new(AtomicU64::new(0)),
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
            geometry,
            world_seed: cfg.noise_seed,
            max_feature_height_m,
            max_feature_radius_m,
        })
    }

    pub fn geometry(&self) -> PlanetGeometry {
        self.geometry
    }

    pub fn resolution(&self) -> u32 {
        self.geometry.resolution
    }

    pub fn cache_stats(&self) -> TerrainCacheStats {
        TerrainCacheStats {
            cached_columns: self
                .columns
                .read()
                .expect("terrain cache should not be poisoned")
                .len(),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            compute_micros: self.cache_compute_micros.load(Ordering::Relaxed),
        }
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
        let depth_m =
            column.height.saturating_sub(layer as u16) as f32 * self.geometry.voxel_size_m;
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
        if u >= self.geometry.resolution || v >= self.geometry.resolution {
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
        let max_layers = self
            .geometry
            .meters_to_voxels_ceil(self.max_feature_height_m.max(self.geometry.voxel_size_m));
        h.saturating_add(1)..=h.saturating_add(max_layers)
    }

    pub fn feature_blocks_in_region(
        &self,
        face: u8,
        u_start: u32,
        v_start: u32,
        u_end: u32,
        v_end: u32,
    ) -> HashMap<BlockId, ContentBlockId> {
        let u_end = u_end.min(self.geometry.resolution);
        let v_end = v_end.min(self.geometry.resolution);
        if u_start >= u_end || v_start >= v_end {
            return HashMap::new();
        }

        let margin = if self.max_feature_radius_m > 0.0 {
            self.geometry
                .meters_to_voxels_ceil(self.max_feature_radius_m)
        } else {
            0
        };
        let scan_u_start = u_start.saturating_sub(margin);
        let scan_v_start = v_start.saturating_sub(margin);
        let scan_u_end = u_end.saturating_add(margin).min(self.geometry.resolution);
        let scan_v_end = v_end.saturating_add(margin).min(self.geometry.resolution);

        let mut blocks = HashMap::new();
        for u in scan_u_start..scan_u_end {
            for v in scan_v_start..scan_v_end {
                let column = self.column(face, u, v);
                let biome = &self.biomes[column.biome_index as usize];
                for flora in self.flora.iter() {
                    if !tags_match(
                        &flora.data.required_tags,
                        &flora.data.forbidden_tags,
                        &biome.data.provided_tags,
                    ) || !self.flora_origin_for_column(face, u, v, column, flora)
                    {
                        continue;
                    }
                    self.add_flora_feature_blocks(
                        face,
                        u,
                        v,
                        column.height as u32,
                        flora,
                        (u_start, v_start, u_end, v_end),
                        &mut blocks,
                    );
                }
            }
        }
        blocks
    }

    fn column(&self, face: u8, u: u32, v: u32) -> TerrainColumn {
        let u = u.min(self.geometry.resolution - 1);
        let v = v.min(self.geometry.resolution - 1);
        let key = Self::cache_key(face, u, v);
        if let Some(column) = self
            .columns
            .read()
            .expect("terrain cache should not be poisoned")
            .get(&key)
            .copied()
        {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            return column;
        }

        self.cache_misses.fetch_add(1, Ordering::Relaxed);
        let compute_start = Instant::now();
        let column = self.compute_column(face, u, v);
        self.cache_compute_micros.fetch_add(
            compute_start.elapsed().as_micros().min(u64::MAX as u128) as u64,
            Ordering::Relaxed,
        );
        self.columns
            .write()
            .expect("terrain cache should not be poisoned")
            .insert(key, column);
        column
    }

    fn compute_column(&self, face: u8, u: u32, v: u32) -> TerrainColumn {
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
            let voxel_volume_m3 = self.geometry.voxel_size_m.powi(3);
            let chance = (vein.frequency * 0.035 * voxel_volume_m3).clamp(0.0, 0.35);
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
                height_min_m: _,
                height_max_m,
            } => {
                let surface = self.column(face, u, v).height as u32;
                let height_layers = self.geometry.meters_to_voxels_ceil(height_max_m);
                if layer > surface
                    && layer <= surface + height_layers
                    && self.flora_origin(face, u, v, biome, flora)
                {
                    return Some(block);
                }
                None
            }
            CompiledFloraFeature::Tree {
                log_block,
                leaf_block,
                trunk_height_min_m,
                trunk_height_max_m,
                canopy_radius_m,
                canopy_height_m,
            } => {
                let radius = TreeShape::expanded_scan_radius_layers(
                    self.geometry.voxel_size_m,
                    canopy_radius_m,
                    trunk_height_max_m,
                );
                for du in -radius..=radius {
                    for dv in -radius..=radius {
                        let ou = u as i32 + du;
                        let ov = v as i32 + dv;
                        if ou < 0
                            || ov < 0
                            || ou >= self.geometry.resolution as i32
                            || ov >= self.geometry.resolution as i32
                        {
                            continue;
                        }
                        let ou = ou as u32;
                        let ov = ov as u32;
                        if !self.flora_origin(face, ou, ov, biome, flora) {
                            continue;
                        }
                        let origin_surface = self.column(face, ou, ov).height as u32;
                        if layer <= origin_surface {
                            continue;
                        }
                        let shape = TreeShape::new(TreeShapeConfig {
                            face,
                            u: ou,
                            v: ov,
                            flora_index: flora.index,
                            world_seed: self.world_seed,
                            voxel_size_m: self.geometry.voxel_size_m,
                            trunk_height_min_m,
                            trunk_height_max_m,
                            canopy_radius_m,
                            canopy_height_m,
                        });
                        let rel_layer = layer - origin_surface;
                        if shape.has_log_at(-du, -dv, rel_layer) {
                            return Some(log_block);
                        }
                        if shape.has_leaf_at(-du, -dv, rel_layer) {
                            return Some(leaf_block);
                        }
                    }
                }
                None
            }
            CompiledFloraFeature::Cluster {
                block,
                radius_max_m,
                ..
            } => {
                let surface = self.column(face, u, v).height as u32;
                if layer != surface + 1 {
                    return None;
                }
                let radius = self.geometry.meters_to_voxels_ceil(radius_max_m) as i32;
                for du in -radius..=radius {
                    for dv in -radius..=radius {
                        let ou = u as i32 + du;
                        let ov = v as i32 + dv;
                        if ou < 0
                            || ov < 0
                            || ou >= self.geometry.resolution as i32
                            || ov >= self.geometry.resolution as i32
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

    fn add_flora_feature_blocks(
        &self,
        face: u8,
        u: u32,
        v: u32,
        surface: u32,
        flora: &TerrainFlora,
        target: (u32, u32, u32, u32),
        blocks: &mut HashMap<BlockId, ContentBlockId>,
    ) {
        let (u_start, v_start, u_end, v_end) = target;
        let mut add_block = |u: u32, v: u32, layer: u32, block: ContentBlockId| {
            if u >= u_start && u < u_end && v >= v_start && v < v_end {
                blocks.entry(BlockId { face, layer, u, v }).or_insert(block);
            }
        };

        match flora.data.feature {
            CompiledFloraFeature::Plant {
                block,
                height_max_m,
                ..
            } => {
                let height_layers = self.geometry.meters_to_voxels_ceil(height_max_m);
                for layer in surface.saturating_add(1)..=surface.saturating_add(height_layers) {
                    add_block(u, v, layer, block);
                }
            }
            CompiledFloraFeature::Tree {
                log_block,
                leaf_block,
                trunk_height_min_m,
                trunk_height_max_m,
                canopy_radius_m,
                canopy_height_m,
            } => {
                let shape = TreeShape::new(TreeShapeConfig {
                    face,
                    u,
                    v,
                    flora_index: flora.index,
                    world_seed: self.world_seed,
                    voxel_size_m: self.geometry.voxel_size_m,
                    trunk_height_min_m,
                    trunk_height_max_m,
                    canopy_radius_m,
                    canopy_height_m,
                });

                let radius = shape.scan_radius_layers();
                for du in -radius..=radius {
                    for dv in -radius..=radius {
                        let ou = u as i32 + du;
                        let ov = v as i32 + dv;
                        if ou < 0
                            || ov < 0
                            || ou >= self.geometry.resolution as i32
                            || ov >= self.geometry.resolution as i32
                        {
                            continue;
                        }
                        for rel_layer in 1..=shape.max_relative_layer() {
                            let layer = surface.saturating_add(rel_layer);
                            if shape.has_log_at(du, dv, rel_layer) {
                                add_block(ou as u32, ov as u32, layer, log_block);
                            }
                        }
                    }
                }

                for du in -radius..=radius {
                    for dv in -radius..=radius {
                        let ou = u as i32 + du;
                        let ov = v as i32 + dv;
                        if ou < 0
                            || ov < 0
                            || ou >= self.geometry.resolution as i32
                            || ov >= self.geometry.resolution as i32
                        {
                            continue;
                        }
                        for rel_layer in 1..=shape.max_relative_layer() {
                            let layer = surface.saturating_add(rel_layer);
                            if shape.has_leaf_at(du, dv, rel_layer) {
                                add_block(ou as u32, ov as u32, layer, leaf_block);
                            }
                        }
                    }
                }
            }
            CompiledFloraFeature::Cluster {
                block,
                radius_max_m,
                ..
            } => {
                let radius = self.geometry.meters_to_voxels_ceil(radius_max_m) as i32;
                for du in -radius..=radius {
                    for dv in -radius..=radius {
                        let ou = u as i32 + du;
                        let ov = v as i32 + dv;
                        if ou < 0
                            || ov < 0
                            || ou >= self.geometry.resolution as i32
                            || ov >= self.geometry.resolution as i32
                        {
                            continue;
                        }
                        let horizontal_m =
                            ((du * du + dv * dv) as f32).sqrt() * self.geometry.voxel_size_m;
                        if horizontal_m <= radius_max_m {
                            let ou = ou as u32;
                            let ov = ov as u32;
                            let surface = self.column(face, ou, ov).height as u32;
                            add_block(ou, ov, surface.saturating_add(1), block);
                        }
                    }
                }
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
        self.flora_origin_for_column(face, u, v, self.column(face, u, v), flora)
            && tags_match(
                &flora.data.required_tags,
                &flora.data.forbidden_tags,
                &biome.data.provided_tags,
            )
    }

    fn flora_origin_for_column(
        &self,
        face: u8,
        u: u32,
        v: u32,
        column: TerrainColumn,
        flora: &TerrainFlora,
    ) -> bool {
        let placement = flora.data.placement;
        let surface = self.geometry.layer_radius_m(column.height as u32) - self.geometry.radius_m;
        if placement
            .altitude_max_m
            .is_some_and(|altitude_max| surface > altitude_max)
        {
            return false;
        }
        let cell_area_m2 = self.geometry.voxel_size_m * self.geometry.voxel_size_m;
        let origin_chance = (placement.density_base * cell_area_m2).clamp(0.0, 1.0);
        hash01(face, u, v, 0, flora.index) < origin_chance
    }
}

impl Clone for PlanetTerrain {
    fn clone(&self) -> Self {
        Self {
            columns: self.columns.clone(),
            cache_hits: self.cache_hits.clone(),
            cache_misses: self.cache_misses.clone(),
            cache_compute_micros: self.cache_compute_micros.clone(),
            biomes: self.biomes.clone(),
            flora: self.flora.clone(),
            ores: self.ores.clone(),
            planet: self.planet.clone(),
            climate_curves: self.climate_curves,
            generator: self.generator.clone(),
            noise: self.noise,
            geometry: self.geometry,
            world_seed: self.world_seed,
            max_feature_height_m: self.max_feature_height_m,
            max_feature_radius_m: self.max_feature_radius_m,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TerrainCacheStats {
    pub cached_columns: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub compute_micros: u64,
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

fn choose_biome_blend(biomes: &[TerrainBiome], climate: ClimateSample) -> BiomeBlend {
    let mut scored = Vec::with_capacity(biomes.len());
    let mut dominant_index = 0usize;
    let mut max_score = 0.0f32;

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
    let mut total = 0.0f32;
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

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
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

fn deterministic_planet_radius_m(
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

fn max_feature_height_m(flora: &[TerrainFlora]) -> f32 {
    flora
        .iter()
        .map(|flora| match flora.data.feature {
            CompiledFloraFeature::Plant { height_max_m, .. } => height_max_m,
            CompiledFloraFeature::Tree {
                trunk_height_max_m,
                canopy_height_m,
                ..
            } => trunk_height_max_m + canopy_height_m * 2.0 + 1.0,
            CompiledFloraFeature::Cluster { radius_max_m, .. } => radius_max_m,
        })
        .fold(1.0, f32::max)
}

fn max_feature_radius_m(flora: &[TerrainFlora]) -> f32 {
    flora
        .iter()
        .map(|flora| match flora.data.feature {
            CompiledFloraFeature::Plant { .. } => 0.0,
            CompiledFloraFeature::Tree {
                canopy_radius_m, ..
            } => canopy_radius_m * 1.6 + 1.0,
            CompiledFloraFeature::Cluster { radius_max_m, .. } => radius_max_m,
        })
        .fold(0.0, f32::max)
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
        let geometry = PlanetGeometry::with_resolution(32.0, 0.5, 16);
        let cfg = WorldGenConfig::default();
        let a = PlanetTerrain::generate_for_geometry(geometry, &cfg, &worldgen).expect("terrain a");
        let b = PlanetTerrain::generate_for_geometry(geometry, &cfg, &worldgen).expect("terrain b");

        for (face, u, v) in [(0, 4, 4), (1, 7, 3), (5, 9, 12)] {
            assert_eq!(a.get_height(face, u, v), b.get_height(face, u, v));
            assert_eq!(a.get_biome(face, u, v), b.get_biome(face, u, v));
            assert_eq!(
                a.get_surface_block(face, u, v),
                b.get_surface_block(face, u, v)
            );
        }

        assert!(content.blocks.key(a.get_surface_block(0, 4, 4)).is_some());
    }

    #[test]
    fn alpha_resources_are_generated_from_content_defs() {
        let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let content = compile_assets_root(&assets).expect("core content should compile");
        let worldgen = content.worldgen_content();
        let geometry = PlanetGeometry::with_resolution(64.0, 0.5, 64);
        let resolution = geometry.resolution;
        let terrain =
            PlanetTerrain::generate_for_geometry(geometry, &WorldGenConfig::default(), &worldgen)
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

    #[test]
    fn biome_boundaries_do_not_create_cliff_steps() {
        let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let content = compile_assets_root(&assets).expect("core content should compile");
        let geometry = PlanetGeometry::with_resolution(96.0, 0.5, 96);
        let terrain = PlanetTerrain::generate_for_geometry(
            geometry,
            &WorldGenConfig::default(),
            &content.worldgen_content(),
        )
        .expect("terrain should generate");

        let mut checked_boundaries = 0;
        let mut largest_step_m = 0.0f32;

        for face in 0..6 {
            for u in 0..95 {
                for v in 0..95 {
                    for (nu, nv) in [(u + 1, v), (u, v + 1)] {
                        if terrain.get_biome(face, u, v) == terrain.get_biome(face, nu, nv) {
                            continue;
                        }
                        checked_boundaries += 1;
                        let step_layers = terrain
                            .get_height(face, u, v)
                            .abs_diff(terrain.get_height(face, nu, nv));
                        largest_step_m = largest_step_m.max(geometry.voxel_extent_m(step_layers));
                    }
                }
            }
        }

        assert!(
            checked_boundaries > 0,
            "test seed should contain biome boundaries"
        );
        assert!(
            largest_step_m <= 3.0,
            "biome boundary height step should stay walkable, got {largest_step_m} m"
        );
    }

    #[test]
    fn same_seed_with_different_voxel_size_keeps_planet_radius_m() {
        let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let content = compile_assets_root(&assets).expect("core content should compile");
        let mut coarse_settings = content.world;
        coarse_settings.voxel_size_m = 0.5;
        let mut fine_settings = content.world;
        fine_settings.voxel_size_m = 0.05;

        let cfg = WorldGenConfig::default();
        let coarse = PlanetTerrain::generate(&cfg, &content.worldgen_content(), &coarse_settings)
            .expect("coarse terrain");
        let fine = PlanetTerrain::generate(&cfg, &content.worldgen_content(), &fine_settings)
            .expect("fine terrain");

        assert_eq!(coarse.geometry().radius_m, fine.geometry().radius_m);
        assert!(fine.geometry().resolution > coarse.geometry().resolution);
    }

    #[test]
    fn same_world_params_keep_global_terrain_shape_in_meters() {
        let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let content = compile_assets_root(&assets).expect("core content should compile");
        let worldgen = content.worldgen_content();
        let cfg = WorldGenConfig::default();
        let coarse_geometry = PlanetGeometry::new(128.0, 0.5);
        let fine_geometry = PlanetGeometry::new(128.0, 0.25);
        let coarse = PlanetTerrain::generate_for_geometry(coarse_geometry, &cfg, &worldgen)
            .expect("coarse terrain");
        let fine = PlanetTerrain::generate_for_geometry(fine_geometry, &cfg, &worldgen)
            .expect("fine terrain");

        for (face, u_frac, v_frac) in [(0, 0.25, 0.25), (2, 0.5, 0.75), (5, 0.8, 0.4)] {
            let coarse_u = (coarse_geometry.resolution as f32 * u_frac) as u32;
            let coarse_v = (coarse_geometry.resolution as f32 * v_frac) as u32;
            let fine_u = (fine_geometry.resolution as f32 * u_frac) as u32;
            let fine_v = (fine_geometry.resolution as f32 * v_frac) as u32;
            let coarse_height_m = coarse_geometry
                .layer_radius_m(coarse.get_height(face, coarse_u, coarse_v))
                - coarse_geometry.radius_m;
            let fine_height_m = fine_geometry.layer_radius_m(fine.get_height(face, fine_u, fine_v))
                - fine_geometry.radius_m;

            assert!(
                (coarse_height_m - fine_height_m).abs() <= coarse_geometry.voxel_size_m * 2.0,
                "terrain height drifted: coarse {coarse_height_m}, fine {fine_height_m}"
            );
        }
    }

    #[test]
    fn authored_tree_height_is_physical_and_voxelized_by_density() {
        let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let content = compile_assets_root(&assets).expect("core content should compile");
        let oak = content
            .flora
            .id(&ContentKey::from_str("voxelverse:oak_trees").unwrap())
            .and_then(|id| content.flora.get(id))
            .expect("oak flora");

        let CompiledFloraFeature::Tree {
            trunk_height_max_m, ..
        } = oak.feature
        else {
            panic!("oak_trees should be a tree feature");
        };

        let coarse = PlanetGeometry::new(128.0, 0.5);
        let fine = PlanetGeometry::new(128.0, 0.05);
        let coarse_layers = coarse.meters_to_voxels_ceil(trunk_height_max_m);
        let fine_layers = fine.meters_to_voxels_ceil(trunk_height_max_m);

        assert_eq!(trunk_height_max_m, 6.0);
        assert_eq!(coarse_layers, 12);
        assert_eq!(fine_layers, 120);
        assert!((coarse.voxel_extent_m(coarse_layers) - 6.0).abs() <= coarse.voxel_size_m);
        assert!((fine.voxel_extent_m(fine_layers) - 6.0).abs() <= fine.voxel_size_m);
    }
}
