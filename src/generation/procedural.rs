#![allow(dead_code)]

use crate::content::{
    CompiledCurve, CompiledNoiseField, CompiledNoiseKind, CompiledProceduralPlanet,
    ProceduralRegistry,
};
use crate::generation::{
    noise::{NoiseGenerator, NoiseSettings, NoiseType},
    CoordSystem,
};
use crate::voxel::{SurfaceChunkKey, VoxelCoord, VoxelId, CHUNK_SIZE};
use crate::world::PlanetProfile;
use glam::Vec3;
use rayon::prelude::*;
use std::sync::Arc;

const MAX_SURFACE_FIELD_RES: u32 = 2048;
const MAX_BIOME_WEIGHTS: usize = 4;

#[derive(Clone, Debug)]
pub struct BiomeWeight {
    pub biome: usize,
    pub weight: f32,
}

#[derive(Clone, Debug)]
pub struct SurfaceSample {
    pub height: u32,
    pub primary_biome: usize,
    pub biome_weights: Vec<BiomeWeight>,
    pub temperature: f32,
    pub humidity: f32,
    pub roughness: f32,
}

#[derive(Clone, Debug)]
pub struct GeneratedVoxelContext {
    pub face: u8,
    pub u: u32,
    pub v: u32,
    pub layer: u32,
    pub dir: Vec3,
    pub depth_from_surface: i32,
    pub surface: SurfaceSample,
}

#[derive(Clone, Debug)]
pub enum FeatureStamp {
    VisualDetail {
        coord: VoxelCoord,
        block: VoxelId,
        priority: i32,
    },
    Tree {
        coord: VoxelCoord,
        trunk: VoxelId,
        leaves: VoxelId,
        height: u32,
        canopy_radius: u32,
        priority: i32,
    },
    Structure {
        coord: VoxelCoord,
        stamp: String,
        priority: i32,
    },
}

#[derive(Clone)]
pub struct ProceduralPlanetTerrain {
    registry: Arc<ProceduralRegistry>,
    planet_index: usize,
    heights: Arc<Vec<i16>>,
    primary_biomes: Arc<Vec<u8>>,
    field_res: u32,
    voxel_res: u32,
    surface_layer: u32,
}

impl ProceduralPlanetTerrain {
    pub fn new(
        profile: PlanetProfile,
        registry: Arc<ProceduralRegistry>,
        planet_index: usize,
    ) -> Self {
        let field_res = profile.resolution.min(MAX_SURFACE_FIELD_RES);
        let size = (6 * field_res * field_res) as usize;
        let planet = &registry.planets[planet_index];

        let mut heights = vec![0i16; size];
        let mut primary_biomes = vec![0u8; size];

        heights
            .par_iter_mut()
            .zip(primary_biomes.par_iter_mut())
            .enumerate()
            .for_each(|(idx, (height_out, biome_out))| {
                let face_area = (field_res * field_res) as usize;
                let face = (idx / face_area) as u8;
                let rem = idx % face_area;
                let v = (rem / field_res as usize) as u32;
                let u = (rem % field_res as usize) as u32;
                let dir = CoordSystem::get_direction(face, u, v, field_res);
                let sample = sample_surface_fields(&registry, planet, dir);
                let (height, primary) = resolve_height(&registry, planet, profile, dir, &sample);
                *height_out = (height as i32 - profile.surface_layer as i32)
                    .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                *biome_out = primary.min(u8::MAX as usize) as u8;
            });

        Self {
            registry,
            planet_index,
            heights: Arc::new(heights),
            primary_biomes: Arc::new(primary_biomes),
            field_res,
            voxel_res: profile.resolution,
            surface_layer: profile.surface_layer,
        }
    }

    pub fn planet(&self) -> &CompiledProceduralPlanet {
        &self.registry.planets[self.planet_index]
    }

    pub fn get_height(&self, face: u8, u: u32, v: u32) -> u32 {
        let hres = self.field_res as f32;
        let vres = self.voxel_res as f32;
        let hu = (u as f32 * hres / vres).min(hres - 1.001);
        let hv = (v as f32 * hres / vres).min(hres - 1.001);
        let u0 = hu as u32;
        let v0 = hv as u32;
        let u1 = (u0 + 1).min(self.field_res - 1);
        let v1 = (v0 + 1).min(self.field_res - 1);
        let fu = hu - u0 as f32;
        let fv = hv - v0 as f32;

        let h00 = self.heights[self.index(face, u0, v0)] as f32;
        let h10 = self.heights[self.index(face, u1, v0)] as f32;
        let h01 = self.heights[self.index(face, u0, v1)] as f32;
        let h11 = self.heights[self.index(face, u1, v1)] as f32;
        let h = h00 * (1.0 - fu) * (1.0 - fv)
            + h10 * fu * (1.0 - fv)
            + h01 * (1.0 - fu) * fv
            + h11 * fu * fv;

        (self.surface_layer as i32 + h.round() as i32).max(0) as u32
    }

    pub fn get_biome_id(&self, face: u8, u: u32, v: u32) -> u8 {
        let (u_h, v_h) = self.surface_coords(u, v);
        self.primary_biomes[self.index(face, u_h, v_h)]
    }

    pub fn surface_sample(&self, face: u8, u: u32, v: u32) -> SurfaceSample {
        let dir = CoordSystem::get_direction(face, u, v, self.voxel_res);
        let planet = self.planet();
        let fields = sample_surface_fields(&self.registry, planet, dir);
        let height = self.get_height(face, u, v);
        let primary_biome = self.get_biome_id(face, u, v) as usize;
        let biome_weights = resolve_biome_weights(&self.registry, planet, fields).0;
        SurfaceSample {
            height,
            primary_biome,
            biome_weights,
            temperature: fields.temperature,
            humidity: fields.humidity,
            roughness: fields.roughness,
        }
    }

    pub fn voxel_at(&self, coord: VoxelCoord, profile: PlanetProfile) -> VoxelId {
        if coord.layer >= self.voxel_res || coord.u >= self.voxel_res || coord.v >= self.voxel_res {
            return VoxelId::AIR;
        }
        let surface = self.surface_sample(coord.face, coord.u, coord.v);
        let depth_from_surface = surface.height as i32 - coord.layer as i32;
        let dir = CoordSystem::get_direction(coord.face, coord.u, coord.v, self.voxel_res);
        let ctx = GeneratedVoxelContext {
            face: coord.face,
            u: coord.u,
            v: coord.v,
            layer: coord.layer,
            dir,
            depth_from_surface,
            surface,
        };
        if depth_from_surface < 0 {
            return self.resolve_above_surface_voxel(&ctx);
        }
        self.resolve_voxel(&ctx, profile)
    }

    pub fn features_for_chunk(&self, key: SurfaceChunkKey) -> Vec<FeatureStamp> {
        let planet = self.planet();
        let mut stamps = Vec::new();
        let u0 = key.u_idx * CHUNK_SIZE;
        let v0 = key.v_idx * CHUNK_SIZE;
        let u1 = (u0 + CHUNK_SIZE).min(self.voxel_res);
        let v1 = (v0 + CHUNK_SIZE).min(self.voxel_res);

        for u in u0..u1 {
            for v in v0..v1 {
                let surface = self.surface_sample(key.face, u, v);
                let coord = VoxelCoord {
                    face: key.face,
                    layer: surface.height.saturating_add(1),
                    u,
                    v,
                };
                self.push_visual_details(planet, coord, &surface, &mut stamps);
                self.push_vegetation(planet, coord, &surface, &mut stamps);
            }
        }

        stamps
    }

    pub fn lod_surface_blocks(&self, face: u8, u: u32, v: u32) -> (VoxelId, VoxelId) {
        let biome = self.registry.biome(self.get_biome_id(face, u, v) as usize);
        (biome.surface.top, biome.surface.under)
    }

    fn resolve_voxel(&self, ctx: &GeneratedVoxelContext, profile: PlanetProfile) -> VoxelId {
        if self.is_cave(ctx) {
            return VoxelId::AIR;
        }

        let biome = self.registry.biome(ctx.surface.primary_biome);
        let mut block = if ctx.depth_from_surface == 0 {
            biome.surface.top
        } else if ctx.depth_from_surface as u32 <= biome.surface.depth.1 {
            biome.surface.under
        } else {
            self.layer_block(ctx, profile)
                .unwrap_or(biome.surface.under)
        };

        if let Some(ore) = self.ore_block(ctx, block) {
            block = ore;
        }

        block
    }

    fn resolve_above_surface_voxel(&self, ctx: &GeneratedVoxelContext) -> VoxelId {
        let above = (-ctx.depth_from_surface) as u32;
        let top = self.registry.biome(ctx.surface.primary_biome).surface.top;

        if above == 1 {
            for detail_idx in &self.planet().visual_detail_sets {
                let detail = &self.registry.visual_details[*detail_idx];
                if detail.placement.surface_blocks.contains(&top)
                    && self.feature_hit(
                        detail.placement.field,
                        ctx.face,
                        ctx.u,
                        ctx.v,
                        detail.placement.density,
                    )
                {
                    if let Some(block) =
                        weighted_detail(&detail.details, hash4(ctx.face, ctx.u, ctx.v, 17))
                    {
                        return block;
                    }
                }
            }
        }

        for veg_idx in &self.planet().vegetation_sets {
            let veg = &self.registry.vegetation[*veg_idx];
            if !veg.placement.surface_blocks.contains(&top)
                || !self.feature_hit(
                    veg.placement.field,
                    ctx.face,
                    ctx.u,
                    ctx.v,
                    veg.placement.density,
                )
            {
                continue;
            }
            let height = range_pick(veg.height, hash4(ctx.face, ctx.u, ctx.v, 33));
            if (1..=height).contains(&above) {
                return veg.trunk;
            }
            if above <= height + veg.canopy_radius.1 {
                return veg.leaves;
            }
        }

        VoxelId::AIR
    }

    fn layer_block(&self, ctx: &GeneratedVoxelContext, profile: PlanetProfile) -> Option<VoxelId> {
        let layers = &self.registry.terrain_layers[self.planet().terrain_layers];
        for layer in &layers.layers {
            let biome_ok = layer.all_biomes || layer.biomes.contains(&ctx.surface.primary_biome);
            if !biome_ok {
                continue;
            }
            let depth_ok = layer.depth.is_some_and(|(min, max)| {
                (ctx.depth_from_surface as u32) >= min && (ctx.depth_from_surface as u32) <= max
            });
            let center_depth = profile.core_layers.saturating_sub(ctx.layer);
            let center_ok = layer
                .depth_from_center
                .is_some_and(|(min, max)| center_depth >= min && center_depth <= max);
            if depth_ok || center_ok {
                return Some(layer.block);
            }
        }
        None
    }

    fn ore_block(&self, ctx: &GeneratedVoxelContext, current: VoxelId) -> Option<VoxelId> {
        let planet = self.planet();
        let biome = self.registry.biome(ctx.surface.primary_biome);
        let depth = ctx.depth_from_surface.max(0) as u32;
        for ore_idx in &planet.ore_sets {
            let ore = &self.registry.ores[*ore_idx];
            if depth < ore.depth.0 || depth > ore.depth.1 || !ore.replace.contains(&current) {
                continue;
            }
            let tag_ok = ore.biome_tags.iter().any(|t| t == "*")
                || ore
                    .biome_tags
                    .iter()
                    .any(|t| biome.vegetation_tags.contains(t) || biome.fauna_tags.contains(t));
            if !tag_ok {
                continue;
            }
            let n = self.sample_field(ore.field, ctx.dir + Vec3::splat(depth as f32 * 0.013));
            let threshold = 1.0 - ore.density.clamp(0.0, 0.95);
            if n >= threshold {
                return Some(ore.block);
            }
        }
        None
    }

    fn is_cave(&self, ctx: &GeneratedVoxelContext) -> bool {
        if ctx.depth_from_surface <= 4 {
            return false;
        }
        for cave_idx in &self.planet().caves {
            let cave = &self.registry.caves[*cave_idx];
            for carver in &cave.carvers {
                let depth = ctx.depth_from_surface as u32;
                if depth < carver.depth.0 || depth > carver.depth.1 {
                    continue;
                }
                let n = self.sample_field(
                    carver.field,
                    ctx.dir + Vec3::new(ctx.layer as f32 * 0.017, depth as f32 * 0.011, 0.0),
                );
                if n >= carver.threshold {
                    return true;
                }
            }
        }
        false
    }

    fn push_visual_details(
        &self,
        planet: &CompiledProceduralPlanet,
        coord: VoxelCoord,
        surface: &SurfaceSample,
        stamps: &mut Vec<FeatureStamp>,
    ) {
        let top = self.registry.biome(surface.primary_biome).surface.top;
        for detail_idx in &planet.visual_detail_sets {
            let detail = &self.registry.visual_details[*detail_idx];
            if !detail.placement.surface_blocks.contains(&top) {
                continue;
            }
            if self.feature_hit(
                detail.placement.field,
                coord.face,
                coord.u,
                coord.v,
                detail.placement.density,
            ) {
                if let Some(block) =
                    weighted_detail(&detail.details, hash4(coord.face, coord.u, coord.v, 17))
                {
                    stamps.push(FeatureStamp::VisualDetail {
                        coord,
                        block,
                        priority: 10,
                    });
                }
            }
        }
    }

    fn push_vegetation(
        &self,
        planet: &CompiledProceduralPlanet,
        coord: VoxelCoord,
        surface: &SurfaceSample,
        stamps: &mut Vec<FeatureStamp>,
    ) {
        let top = self.registry.biome(surface.primary_biome).surface.top;
        for veg_idx in &planet.vegetation_sets {
            let veg = &self.registry.vegetation[*veg_idx];
            if !veg.placement.surface_blocks.contains(&top) {
                continue;
            }
            if self.feature_hit(
                veg.placement.field,
                coord.face,
                coord.u,
                coord.v,
                veg.placement.density,
            ) {
                let h = range_pick(veg.height, hash4(coord.face, coord.u, coord.v, 33));
                let r = range_pick(veg.canopy_radius, hash4(coord.face, coord.u, coord.v, 34));
                stamps.push(FeatureStamp::Tree {
                    coord,
                    trunk: veg.trunk,
                    leaves: veg.leaves,
                    height: h,
                    canopy_radius: r,
                    priority: 30,
                });
            }
        }
    }

    fn feature_hit(&self, field: usize, face: u8, u: u32, v: u32, density: f32) -> bool {
        let dir = CoordSystem::get_direction(face, u, v, self.voxel_res);
        let field_value = self.sample_field(field, dir);
        let jitter = hash4(face, u, v, field as u32) as f32 / u32::MAX as f32;
        field_value * density.clamp(0.0, 1.0) > jitter
    }

    fn sample_field(&self, field: usize, pos: Vec3) -> f32 {
        sample_noise_field(&self.registry, self.planet().base.seed, field, pos, 0)
    }

    fn index(&self, face: u8, u: u32, v: u32) -> usize {
        face as usize * self.field_res as usize * self.field_res as usize
            + v as usize * self.field_res as usize
            + u as usize
    }

    fn surface_coords(&self, u: u32, v: u32) -> (u32, u32) {
        let hres = self.field_res as u64;
        (
            ((u as u64 * hres) / self.voxel_res as u64).min(hres - 1) as u32,
            ((v as u64 * hres) / self.voxel_res as u64).min(hres - 1) as u32,
        )
    }
}

#[derive(Clone, Copy)]
struct SurfaceFields {
    temperature: f32,
    humidity: f32,
    roughness: f32,
    continentality: f32,
    erosion: f32,
    weirdness: f32,
}

fn sample_surface_fields(
    registry: &ProceduralRegistry,
    planet: &CompiledProceduralPlanet,
    dir: Vec3,
) -> SurfaceFields {
    let climate = &registry.climates[planet.climate];
    let latitude = dir.y.abs();
    let temperature = sample_axis(
        registry,
        planet.base.seed,
        &climate.temperature,
        dir,
        latitude,
    );
    let humidity = sample_axis(registry, planet.base.seed, &climate.humidity, dir, latitude);
    let continentality = sample_axis(
        registry,
        planet.base.seed,
        &climate.continentality,
        dir,
        latitude,
    );
    let erosion = sample_axis(registry, planet.base.seed, &climate.erosion, dir, latitude);
    let weirdness = sample_axis(
        registry,
        planet.base.seed,
        &climate.weirdness,
        dir,
        latitude,
    );
    let roughness = ((1.0 - erosion) * 0.65 + weirdness * 0.35).clamp(0.0, 1.0);
    SurfaceFields {
        temperature,
        humidity,
        roughness,
        continentality,
        erosion,
        weirdness,
    }
}

fn sample_axis(
    registry: &ProceduralRegistry,
    seed: u32,
    axis: &crate::content::CompiledClimateAxis,
    dir: Vec3,
    latitude: f32,
) -> f32 {
    let mut value = 0.5 + (1.0 - latitude - 0.5) * axis.latitude_bias + axis.ocean_bias;
    for (field, weight) in &axis.fields {
        value += (sample_noise_field(registry, seed, *field, dir, 0) - 0.5) * *weight;
    }
    value.clamp(0.0, 1.0)
}

fn resolve_height(
    registry: &ProceduralRegistry,
    planet: &CompiledProceduralPlanet,
    profile: PlanetProfile,
    dir: Vec3,
    fields: &SurfaceFields,
) -> (u32, usize) {
    let (weights, primary) = resolve_biome_weights(registry, planet, *fields);
    let mut height_offset = 0.0;
    for weight in &weights {
        let biome = registry.biome(weight.biome);
        let hill = sample_noise_field(registry, planet.base.seed, biome.terrain.hill_field, dir, 0)
            * 2.0
            - 1.0;
        let ridge = biome
            .terrain
            .ridge_field
            .map(|field| sample_noise_field(registry, planet.base.seed, field, dir, 0) * 2.0 - 1.0)
            .unwrap_or(0.0);
        let flat = 1.0 - biome.terrain.flatness;
        let mut local = biome.terrain.base_height
            + hill * biome.terrain.amplitude * flat
            + ridge * biome.terrain.amplitude * 0.35;
        if biome.terrain.terrace_strength > 0.0 {
            let steps = 12.0;
            let terraced = (local * steps).round() / steps;
            local = local + (terraced - local) * biome.terrain.terrace_strength;
        }
        height_offset += local * weight.weight;
    }
    let macro_shape =
        (fields.continentality - 0.5) * 0.40 - fields.erosion * 0.18 + fields.weirdness * 0.08;
    height_offset += macro_shape;
    let layer = profile.surface_layer as i32
        + (height_offset * planet.base.max_terrain_offset as f32).round() as i32;
    let min_layer = profile.core_layers.saturating_add(2) as i32;
    let max_layer = profile.resolution.saturating_sub(3) as i32;
    (layer.clamp(min_layer, max_layer) as u32, primary)
}

fn resolve_biome_weights(
    registry: &ProceduralRegistry,
    planet: &CompiledProceduralPlanet,
    fields: SurfaceFields,
) -> (Vec<BiomeWeight>, usize) {
    let set = &registry.biome_sets[planet.biome_set];
    let mut weights: Vec<BiomeWeight> = set
        .selectors
        .iter()
        .filter_map(|selector| {
            let dt = range_distance(fields.temperature, selector.temperature);
            let dh = range_distance(fields.humidity, selector.humidity);
            let dr = range_distance(fields.roughness, selector.roughness);
            let d = (dt * dt + dh * dh + dr * dr).sqrt();
            let w = ((set.blend_radius - d) / set.blend_radius).clamp(0.0, 1.0) * selector.weight;
            (w > 0.0).then_some(BiomeWeight {
                biome: selector.biome,
                weight: w,
            })
        })
        .collect();

    if weights.is_empty() {
        let fallback = set
            .selectors
            .iter()
            .min_by(|a, b| {
                let da = range_distance(fields.temperature, a.temperature)
                    + range_distance(fields.humidity, a.humidity)
                    + range_distance(fields.roughness, a.roughness);
                let db = range_distance(fields.temperature, b.temperature)
                    + range_distance(fields.humidity, b.humidity)
                    + range_distance(fields.roughness, b.roughness);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.biome)
            .unwrap_or(0);
        weights.push(BiomeWeight {
            biome: fallback,
            weight: 1.0,
        });
    }

    weights.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    weights.truncate(MAX_BIOME_WEIGHTS);
    let total = weights.iter().map(|w| w.weight).sum::<f32>().max(0.001);
    for weight in &mut weights {
        weight.weight /= total;
    }
    let primary = weights.first().map(|w| w.biome).unwrap_or(0);
    (weights, primary)
}

fn sample_noise_field(
    registry: &ProceduralRegistry,
    seed: u32,
    field_idx: usize,
    pos: Vec3,
    depth: u32,
) -> f32 {
    let field: &CompiledNoiseField = &registry.fields[field_idx];
    if matches!(field.kind, CompiledNoiseKind::Constant) {
        return field.amplitude.clamp(0.0, 1.0);
    }

    let mut sample_pos = pos;
    if let Some((warp_idx, strength)) = field.domain_warp {
        if depth < 4 {
            let warp =
                sample_noise_field(registry, seed, warp_idx, pos + Vec3::splat(13.7), depth + 1)
                    * 2.0
                    - 1.0;
            sample_pos += Vec3::new(warp, warp * 0.73, warp * 1.37) * strength;
        }
    }

    let gen = NoiseGenerator::new(seed.wrapping_add(field.seed_salt));
    let noise_type = match field.kind {
        CompiledNoiseKind::Ridged | CompiledNoiseKind::Cellular => NoiseType::Ridged,
        _ => NoiseType::Perlin,
    };
    let settings = NoiseSettings {
        noise_type,
        frequency: field.frequency,
        amplitude: field.amplitude,
        octaves: field.octaves,
        persistence: field.persistence,
        lacunarity: field.lacunarity,
        offset: Vec3::ZERO,
    };
    let mut value = gen.compute(sample_pos, &settings) * field.amplitude;
    if let Some(remap) = &field.remap {
        let denom = (remap.in_max - remap.in_min).abs().max(0.0001);
        let mut t = ((value - remap.in_min) / denom).clamp(0.0, 1.0);
        if matches!(remap.curve, CompiledCurve::Smoothstep) {
            t = t * t * (3.0 - 2.0 * t);
        }
        value = remap.out_min + (remap.out_max - remap.out_min) * t;
    }
    value.clamp(0.0, 1.0)
}

fn weighted_detail(
    items: &[crate::content::CompiledVisualDetailItem],
    roll: u32,
) -> Option<VoxelId> {
    let total = items.iter().map(|i| i.weight).sum::<u32>();
    if total == 0 {
        return None;
    }
    let mut pick = roll % total;
    for item in items {
        if pick < item.weight {
            return Some(item.block);
        }
        pick -= item.weight;
    }
    None
}

fn range_distance(value: f32, range: (f32, f32)) -> f32 {
    if value < range.0 {
        range.0 - value
    } else if value > range.1 {
        value - range.1
    } else {
        0.0
    }
}

fn range_pick(range: (u32, u32), roll: u32) -> u32 {
    if range.0 == range.1 {
        range.0
    } else {
        range.0 + roll % (range.1 - range.0 + 1)
    }
}

fn hash4(face: u8, u: u32, v: u32, salt: u32) -> u32 {
    let mut x = salt ^ (face as u32).wrapping_mul(0x9E37_79B9);
    x ^= u.wrapping_mul(0x85EB_CA6B).rotate_left(13);
    x ^= v.wrapping_mul(0xC2B2_AE35).rotate_right(7);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^ (x >> 16)
}
