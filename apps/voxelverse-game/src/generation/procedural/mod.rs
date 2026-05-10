//! Procedural planet terrain generation.
//!
//! `ProceduralPlanetTerrain` orchestrates the climate → biome → height pipeline
//! at construction time, then resolves single voxels and per-chunk feature
//! stamps on demand.  The implementation is split across focused submodules
//! along single-responsibility lines:
//!
//! * [`climate`]: 5-axis climate sampling (temperature, humidity,
//!   continentality, erosion, weirdness).
//! * [`biome_select`]: weighted-blend biome resolution in normalized 6-D
//!   climate space.
//! * [`height`]: per-biome terrain height composition.
//! * [`noise_sampler`]: shared noise-field evaluator used by every other
//!   submodule.
//! * [`voxel_resolver`]: surface, layer, ore, and cave voxel resolution.
//! * [`feature_eval`]: slow-path tree query (mirror of the chunk bakery's
//!   tree stamping for collision / raycast lookups).
//!
//! Anything outside this module sees only the public API on
//! `ProceduralPlanetTerrain` plus the helper types re-exported below.
#![allow(dead_code)]

mod biome_select;
mod climate;
mod feature_eval;
mod height;
mod noise_sampler;
mod voxel_resolver;

use crate::content::{CompiledProceduralPlanet, ProceduralRegistry};
use crate::generation::{noise::NoiseGenerator, CoordSystem};
use crate::voxel::{SurfaceChunkKey, VoxelCoord, VoxelId, CHUNK_SIZE};
use crate::world::PlanetProfile;
use glam::Vec3;
use rayon::prelude::*;
use std::sync::Arc;

pub(super) const MAX_SURFACE_FIELD_RES: u32 = 1024;
pub(super) const MAX_BIOME_WEIGHTS: usize = 4;

/// Reference voxel size (in metres) the procedural RON pack is authored
/// against.  Every voxel-count field assumes this baseline; runtime scales
/// relative to it.
pub(crate) const WORLD_SCALE_BASELINE_METERS: f32 = 1.0;

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

/// A vox prop instance to be rendered above the terrain surface.
/// Props are not in the voxel grid — they are rendered separately.
#[derive(Clone, Debug)]
pub struct PropStamp {
    pub face: u8,
    pub u: u32,
    pub v: u32,
    /// Layer index of the surface block (prop sits at layer + 1).
    pub surface_layer: u32,
    /// Content ref to a .vox asset, e.g. `"core:voxel/vegetation/flowers/flower_blue_1"`.
    pub model_key: String,
    /// Quarter-turn rotation around the radial axis (0–3).
    pub rotation: u8,
}

#[derive(Clone)]
pub struct ProceduralPlanetTerrain {
    pub(super) registry: Arc<ProceduralRegistry>,
    pub(super) planet_index: usize,
    pub(super) heights: Arc<Vec<i16>>,
    pub(super) primary_biomes: Arc<Vec<u8>>,
    pub(super) noise_generators: Arc<Vec<NoiseGenerator>>,
    pub(super) field_res: u32,
    pub(super) voxel_res: u32,
    pub(super) surface_layer: u32,
}

impl ProceduralPlanetTerrain {
    pub fn new(
        profile: PlanetProfile,
        registry: Arc<ProceduralRegistry>,
        planet_index: usize,
    ) -> Self {
        Self::new_with_progress(profile, registry, planet_index, |_, _| {})
    }

    pub fn new_with_progress(
        profile: PlanetProfile,
        registry: Arc<ProceduralRegistry>,
        planet_index: usize,
        mut progress: impl FnMut(f32, &str),
    ) -> Self {
        let field_res = profile.resolution.min(MAX_SURFACE_FIELD_RES);
        let size = (6 * field_res * field_res) as usize;
        let planet = &registry.planets[planet_index];
        let noise_generators: Arc<Vec<NoiseGenerator>> = Arc::new(
            registry
                .fields
                .iter()
                .map(|field| NoiseGenerator::new(planet.base.seed.wrapping_add(field.seed_salt)))
                .collect(),
        );

        let mut heights = vec![0i16; size];
        let mut primary_biomes = vec![0u8; size];

        progress(0.05, "Préparation des champs procéduraux");
        let face_area = (field_res * field_res) as usize;
        for face in 0..6usize {
            let start = face * face_area;
            let end = start + face_area;
            heights[start..end]
                .par_iter_mut()
                .zip(primary_biomes[start..end].par_iter_mut())
                .enumerate()
                .for_each(|(rem, (height_out, biome_out))| {
                    let v = (rem / field_res as usize) as u32;
                    let u = (rem % field_res as usize) as u32;
                    let dir = CoordSystem::get_direction(face as u8, u, v, field_res);
                    let sample =
                        climate::sample_surface_fields(&registry, &noise_generators, planet, dir);
                    let (height, primary) = height::resolve_height(
                        &registry,
                        &noise_generators,
                        planet,
                        profile,
                        dir,
                        &sample,
                    );
                    *height_out = (height as i32 - profile.surface_layer as i32)
                        .clamp(i16::MIN as i32, i16::MAX as i32)
                        as i16;
                    *biome_out = primary.min(u8::MAX as usize) as u8;
                });
            let pct = 0.05 + ((face + 1) as f32 / 6.0) * 0.90;
            progress(pct, "Génération terrain, climat et biomes");
        }
        progress(0.98, "Finalisation planète");

        Self {
            registry,
            planet_index,
            heights: Arc::new(heights),
            primary_biomes: Arc::new(primary_biomes),
            noise_generators,
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
        let height = self.get_height(face, u, v);
        let primary_biome = self.get_biome_id(face, u, v) as usize;
        SurfaceSample {
            height,
            primary_biome,
            biome_weights: Vec::new(),
            temperature: 0.0,
            humidity: 0.0,
            roughness: 0.0,
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
                self.push_vegetation(planet, coord, &surface, &mut stamps);
            }
        }

        stamps
    }

    /// Return the vox prop instances that should appear in the given chunk.
    /// Props are procedurally derived (deterministic) and are NOT stored in the
    /// voxel grid — this is the authoritative placement query.
    ///
    /// # Algorithm — jittered sparse grid
    ///
    /// Instead of testing every `(u, v)` voxel column (which creates visible
    /// grid lines), the chunk is divided into `CELL × CELL` voxel cells.
    /// For each cell a single candidate column is chosen by hashing the cell
    /// coordinates into a 2-D jitter offset.  This gives:
    ///
    /// * **Natural distribution** — candidates are never perfectly aligned.
    /// * **~10× fewer surface_sample calls** (32² → (32/CELL)²).
    /// * **Controllable density** — cell size drives the maximum density cap;
    ///   the scatter's RON density is a 0..1 probability gate on top.
    pub fn props_for_chunk(&self, key: SurfaceChunkKey) -> Vec<PropStamp> {
        let planet = self.planet();

        // Cell size in voxels.  3 → max ~115 candidates per chunk.
        // Larger = sparser but more natural.  Must be ≥ 1.
        const CELL: u32 = 3;

        let u0 = key.u_idx * CHUNK_SIZE;
        let v0 = key.v_idx * CHUNK_SIZE;
        let u1 = (u0 + CHUNK_SIZE).min(self.voxel_res);
        let v1 = (v0 + CHUNK_SIZE).min(self.voxel_res);

        // Number of cells that fit in this chunk (may be less at planet edge).
        let cells_u = (u1 - u0).div_ceil(CELL);
        let cells_v = (v1 - v0).div_ceil(CELL);

        let mut props = Vec::with_capacity((cells_u * cells_v) as usize);

        for cu in 0..cells_u {
            for cv in 0..cells_v {
                // Cell origin in world voxel coords.
                let cell_u0 = u0 + cu * CELL;
                let cell_v0 = v0 + cv * CELL;

                // Hash-driven jitter: pick a random column inside this cell.
                // Scramble with Knuth/Fibonacci primes BEFORE hashing so that
                // cells spaced exactly CELL=3 apart don't produce correlated
                // modulo residues.  XOR-ing U and V inputs prevents diagonal
                // mirroring.
                let su =
                    cell_u0.wrapping_mul(2_654_435_761u32) ^ cell_v0.wrapping_mul(805_459_861u32);
                let sv =
                    cell_v0.wrapping_mul(2_246_822_519u32) ^ cell_u0.wrapping_mul(1_013_904_223u32);
                let ju = hash4(key.face, su, sv, 0x1F2E_3D4C) % CELL.min(u1 - cell_u0);
                let jv = hash4(key.face, su ^ 0xDEAD_C0DE, sv ^ 0xBEEF_1234, 0xABCD_EF01)
                    % CELL.min(v1 - cell_v0);

                let u = cell_u0 + ju;
                let v = cell_v0 + jv;

                // Clamp to valid range (shouldn't be needed but be safe).
                if u >= u1 || v >= v1 {
                    continue;
                }

                let surface = self.surface_sample(key.face, u, v);
                let biome = self.registry.biome(surface.primary_biome);
                let top = biome.surface.top;

                for scatter_idx in &planet.vox_prop_scatters {
                    let scatter = &self.registry.vox_prop_scatters[*scatter_idx];
                    if !scatter.placement.allowed_in_biome(biome) {
                        continue;
                    }
                    if !scatter.placement.surface_blocks.contains(&top) {
                        continue;
                    }
                    // The density gate uses the jittered (u,v) so clusters and
                    // clearings from the noise field are still respected.
                    // Note: density_scale() is NOT applied here — the cell size
                    // already controls physical density independently of voxel
                    // resolution.  RON density values are direct 0..1 per-cell
                    // probabilities (average, before noise modulation).
                    if self.feature_hit(
                        scatter.placement.field,
                        key.face,
                        u,
                        v,
                        scatter.placement.density,
                    ) {
                        let hash = hash4(key.face, u, v, 0xDEAD_BEEF);
                        if let Some(variant) = scatter.pick_variant(hash) {
                            let rotation = (hash4(key.face, u, v, 0xCAFE) & 3) as u8;
                            props.push(PropStamp {
                                face: key.face,
                                u,
                                v,
                                surface_layer: surface.height,
                                model_key: variant.model_key.clone(),
                                rotation,
                            });
                        }
                        // Only one prop per cell — avoids stacking scatter types
                        // on the same column.
                        break;
                    }
                }
            }
        }

        props
    }

    pub fn lod_surface_blocks(&self, face: u8, u: u32, v: u32) -> (VoxelId, VoxelId) {
        let biome = self.registry.biome(self.get_biome_id(face, u, v) as usize);
        (biome.surface.top, biome.surface.under)
    }

    pub fn registry(&self) -> &ProceduralRegistry {
        &self.registry
    }

    pub fn planet_index(&self) -> usize {
        self.planet_index
    }

    pub fn voxel_res(&self) -> u32 {
        self.voxel_res
    }

    /// Multiplier applied to every voxel-counted quantity authored in the
    /// procedural pack so that physical world size stays constant when
    /// `voxel_size_meters` shrinks.  RON values are written assuming a 1 m
    /// baseline; at 0.5 m voxels this returns 2.0 → trees, soil layers, ore
    /// veins all double in voxel count but keep their physical dimensions.
    pub fn voxel_scale(&self) -> f32 {
        WORLD_SCALE_BASELINE_METERS / self.planet().base.voxel_size_meters.max(0.0001)
    }

    pub(crate) fn feature_hit_pub(
        &self,
        field: usize,
        face: u8,
        u: u32,
        v: u32,
        density: f32,
    ) -> bool {
        self.feature_hit(field, face, u, v, density)
    }

    /// Density compensation: `feature_hit` runs once per `(u, v)` cell.  When
    /// voxels shrink the cell grid densifies quadratically, so authored
    /// per-cell densities must shrink by the same factor to preserve the
    /// physical "trees per m²" the pack writer intended.
    pub fn density_scale(&self) -> f32 {
        let voxel_m = self.planet().base.voxel_size_meters.max(0.0001);
        (voxel_m / WORLD_SCALE_BASELINE_METERS).powi(2)
    }

    // ---- chunk feature stamping (orchestration; per-feature kind logic
    //      is a thin loop here because it's just data-routing) -------------

    fn push_vegetation(
        &self,
        planet: &CompiledProceduralPlanet,
        coord: VoxelCoord,
        surface: &SurfaceSample,
        stamps: &mut Vec<FeatureStamp>,
    ) {
        let biome = self.registry.biome(surface.primary_biome);
        let top = biome.surface.top;
        for veg_idx in &planet.vegetation_sets {
            let veg = &self.registry.vegetation[*veg_idx];
            if !veg.placement.allowed_in_biome(biome) {
                continue;
            }
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

    pub(super) fn feature_hit(&self, field: usize, face: u8, u: u32, v: u32, density: f32) -> bool {
        // Sample the scatter field as a spatial density envelope: high-noise
        // areas become denser clusters (forests), low-noise areas become
        // clearings.
        let dir = CoordSystem::get_direction(face, u, v, self.voxel_res);
        let cluster = noise_sampler::sample_noise_field(
            &self.registry,
            &self.noise_generators,
            field,
            dir,
            0,
        );
        // cluster 0..1 → density × 0..2 so the average density stays on target
        // (average cluster ≈ 0.5 × 2 = 1.0 × density).
        let effective = (density * cluster * 2.0).clamp(0.0, 1.0);
        let roll = hash4(face, u, v, field as u32 ^ 0xA5B6_C7D8) as f32 / u32::MAX as f32;
        roll < effective
    }

    pub(super) fn sample_field(&self, field: usize, pos: Vec3) -> f32 {
        noise_sampler::sample_noise_field(&self.registry, &self.noise_generators, field, pos, 0)
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

// ---- Small shared helpers -------------------------------------------------

pub(super) fn range_pick(range: (u32, u32), roll: u32) -> u32 {
    crate::generation::features::range_pick(range, roll)
}

pub(super) fn hash4(face: u8, u: u32, v: u32, salt: u32) -> u32 {
    crate::generation::features::hash4(face, u, v, salt)
}
