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
//! * [`placement_density`]: shared density gate + voxel-size scaling used
//!   by every scatter pass.
//! * [`vegetation_scatter`]: per-chunk tree / large-flora stamping.
//! * [`prop_scatter`]: per-chunk vox-prop scattering (surface + cave fold-in).
//! * [`types`]: plain data carried through the pipeline.
//!
//! Anything outside this module sees only the public API on
//! `ProceduralPlanetTerrain` plus the helper types re-exported below.
mod biome_select;
mod climate;
mod feature_eval;
mod height;
mod noise_sampler;
mod placement_density;
mod prop_scatter;
mod types;
mod vegetation_scatter;
mod voxel_resolver;

pub use types::{
    BiomeWeight, FeatureStamp, GeneratedVoxelContext, PropOrientation, PropStamp, SurfaceSample,
};

use crate::diagnostics::WorldgenStats;
use crate::noise::NoiseGenerator;
use glam::Vec3;
use std::sync::atomic::{AtomicI16, AtomicU8, Ordering};
use std::sync::Arc;
use vv_math::CoordSystem;
use vv_pack_compiler::{CompiledProceduralPlanet, ProceduralRegistry};
use vv_voxel::PlanetProfile;
use vv_voxel::{VoxelCoord, VoxelId};

pub(super) const MAX_SURFACE_FIELD_RES: u32 = 1024;
pub(super) const MAX_BIOME_WEIGHTS: usize = 4;

/// Reference voxel size (in metres) the procedural RON pack is authored
/// against.  Every voxel-count field assumes this baseline; runtime scales
/// relative to it.
pub(crate) const WORLD_SCALE_BASELINE_METERS: f32 = 1.0;

/// Sentinel marking "this cell has not been computed yet" in the lazy
/// surface cache.  i16::MAX is well outside any realistic height delta
/// (max ≈ ±800 voxels even on giant planets).
const UNCOMPUTED_HEIGHT: i16 = i16::MAX;

/// Sentinel for the lazy primary-biome cache.  No pack ever reaches 255
/// biomes (compile clamps `id` to u8 with the assumption < 100 biomes).
const UNCOMPUTED_BIOME: u8 = u8::MAX;

/// Procedural terrain — chunks are generated on demand, never pre-baked.
///
/// The surface height and primary biome for every `(face, u_field, v_field)`
/// cell are stored in atomic arrays initialised to sentinel values.  The
/// first lookup of a cell computes the sample and writes it back; subsequent
/// lookups are a single atomic load.  Two threads racing on the same cell
/// will independently compute the deterministic sample and store the same
/// result — no lock needed.
pub struct ProceduralPlanetTerrain {
    pub(super) registry: Arc<ProceduralRegistry>,
    pub(super) planet_index: usize,
    pub(super) heights: Arc<Vec<AtomicI16>>,
    pub(super) primary_biomes: Arc<Vec<AtomicU8>>,
    pub(super) noise_generators: Arc<Vec<NoiseGenerator>>,
    pub(super) field_res: u32,
    pub(super) voxel_res: u32,
    pub(super) surface_layer: u32,
    pub(super) profile: PlanetProfile,
    pub(super) stats: Arc<WorldgenStats>,
}

impl Clone for ProceduralPlanetTerrain {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            planet_index: self.planet_index,
            heights: self.heights.clone(),
            primary_biomes: self.primary_biomes.clone(),
            noise_generators: self.noise_generators.clone(),
            field_res: self.field_res,
            voxel_res: self.voxel_res,
            surface_layer: self.surface_layer,
            profile: self.profile,
            stats: self.stats.clone(),
        }
    }
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
        // The lazy surface cache means startup does no per-voxel work — chunk
        // generation is now fully on-demand.  We still emit a progress beat
        // so the loading screen draws once.
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

        progress(0.30, "Préparation de la pipeline procédurale paresseuse");

        let heights: Vec<AtomicI16> = (0..size)
            .map(|_| AtomicI16::new(UNCOMPUTED_HEIGHT))
            .collect();
        let primary_biomes: Vec<AtomicU8> =
            (0..size).map(|_| AtomicU8::new(UNCOMPUTED_BIOME)).collect();

        progress(0.95, "Cache surface alloué — génération à la volée");

        Self {
            registry,
            planet_index,
            heights: Arc::new(heights),
            primary_biomes: Arc::new(primary_biomes),
            noise_generators,
            field_res,
            voxel_res: profile.resolution,
            surface_layer: profile.surface_layer,
            profile,
            stats: Arc::new(WorldgenStats::default()),
        }
    }

    /// Read-only handle to the live worldgen telemetry counters.  Used by
    /// the diagnostics overlay; safe to share across threads.
    pub fn stats(&self) -> &Arc<WorldgenStats> {
        &self.stats
    }

    /// Lazily resolve one cell of the surface cache.  Idempotent: races
    /// between threads converge on the same deterministic sample.
    fn ensure_cell(&self, face: u8, u_field: u32, v_field: u32) -> (i16, u8) {
        let idx = self.index(face, u_field, v_field);
        let cached_h = self.heights[idx].load(Ordering::Relaxed);
        if cached_h != UNCOMPUTED_HEIGHT {
            let cached_b = self.primary_biomes[idx].load(Ordering::Relaxed);
            if cached_b != UNCOMPUTED_BIOME {
                self.stats.record_cell_hit();
                return (cached_h, cached_b);
            }
        }

        self.stats.record_cell_miss();
        let planet = &self.registry.planets[self.planet_index];
        let dir = CoordSystem::get_direction(face, u_field, v_field, self.field_res);
        let fields =
            climate::sample_surface_fields(&self.registry, &self.noise_generators, planet, dir);
        let (height, primary) = height::resolve_height(
            &self.registry,
            &self.noise_generators,
            planet,
            self.profile,
            dir,
            &fields,
        );
        let h_i16 = (height as i32 - self.profile.surface_layer as i32)
            .clamp((i16::MIN + 1) as i32, (i16::MAX - 1) as i32) as i16;
        let b_u8 = primary.min((u8::MAX - 1) as usize) as u8;
        self.heights[idx].store(h_i16, Ordering::Relaxed);
        self.primary_biomes[idx].store(b_u8, Ordering::Relaxed);
        (h_i16, b_u8)
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

        let (h00, _) = self.ensure_cell(face, u0, v0);
        let (h10, _) = self.ensure_cell(face, u1, v0);
        let (h01, _) = self.ensure_cell(face, u0, v1);
        let (h11, _) = self.ensure_cell(face, u1, v1);
        let h = h00 as f32 * (1.0 - fu) * (1.0 - fv)
            + h10 as f32 * fu * (1.0 - fv)
            + h01 as f32 * (1.0 - fu) * fv
            + h11 as f32 * fu * fv;

        (self.surface_layer as i32 + h.round() as i32).max(0) as u32
    }

    pub fn get_biome_id(&self, face: u8, u: u32, v: u32) -> u8 {
        // Smooth domain warp for organic biome boundaries.
        // Each field-cell corner has a random offset; we bilinearly interpolate
        // the 4 surrounding corners so the warp is continuous and nearby voxels
        // receive similar displacements.  Maximum displacement ≈ WARP_CELLS
        // field cells (~30 voxels), enough to dissolve the visible grid without
        // making distant biome shapes unrecognisable.
        const WARP_CELLS: f32 = 3.0;

        let hres = self.field_res as f64;
        let vres = self.voxel_res as f64;

        // Continuous field-coordinate of this voxel [0, field_res).
        let uf = (u as f64 * hres / vres) as f32;
        let vf = (v as f64 * hres / vres) as f32;

        let u0 = uf.floor() as u32;
        let v0 = vf.floor() as u32;
        let fu = uf - uf.floor(); // sub-cell fraction [0, 1)
        let fv = vf - vf.floor();

        // Per-corner random offset in field-cell units, deterministic by hash.
        let corner_warp = |cu: u32, cv: u32| -> (f32, f32) {
            let cu = cu.min(self.field_res - 1);
            let cv = cv.min(self.field_res - 1);
            let h0 = hash4(face, cu, cv, 0x9E3779B9) as f32 / u32::MAX as f32;
            let h1 = hash4(face, cu, cv, 0x6C62272E) as f32 / u32::MAX as f32;
            ((h0 * 2.0 - 1.0) * WARP_CELLS, (h1 * 2.0 - 1.0) * WARP_CELLS)
        };

        let (du00, dv00) = corner_warp(u0, v0);
        let (du10, dv10) = corner_warp(u0 + 1, v0);
        let (du01, dv01) = corner_warp(u0, v0 + 1);
        let (du11, dv11) = corner_warp(u0 + 1, v0 + 1);

        // Bilinear interpolation of the four warp vectors.
        let w00 = (1.0 - fu) * (1.0 - fv);
        let w10 = fu * (1.0 - fv);
        let w01 = (1.0 - fu) * fv;
        let w11 = fu * fv;
        let du = du00 * w00 + du10 * w10 + du01 * w01 + du11 * w11;
        let dv = dv00 * w00 + dv10 * w10 + dv01 * w01 + dv11 * w11;

        let u_w = (uf + du).clamp(0.0, (self.field_res - 1) as f32) as u32;
        let v_w = (vf + dv).clamp(0.0, (self.field_res - 1) as f32) as u32;

        let (_, b) = self.ensure_cell(face, u_w, v_w);
        b
    }

    pub fn surface_sample(&self, face: u8, u: u32, v: u32) -> SurfaceSample {
        // Fast path — uses the cached primary biome and bilinear-interpolated
        // height.  Weights are deferred to `surface_sample_with_weights`
        // (callers that need them pay the climate-resampling cost explicitly).
        let height = self.get_height(face, u, v);
        let primary_biome = self.get_biome_id(face, u, v) as usize;
        let mut weights = [BiomeWeight::default(); MAX_BIOME_WEIGHTS];
        weights[0] = BiomeWeight {
            biome: primary_biome as u16,
            weight: 1.0,
        };
        SurfaceSample {
            height,
            primary_biome,
            biome_weights: weights,
            weight_count: 1,
            temperature: 0.0,
            humidity: 0.0,
            roughness: 0.0,
        }
    }

    /// Full surface sample with continuous biome blend weights.  Re-samples
    /// the climate fields at the exact `(u, v)` because the per-cell cache
    /// only stores the primary biome — weighted blends drive feature density
    /// feathering and per-biome height curve mixing.  Skip this path on the
    /// per-voxel hot loop; use the cheap `surface_sample` instead.
    pub fn surface_sample_with_weights(&self, face: u8, u: u32, v: u32) -> SurfaceSample {
        let height = self.get_height(face, u, v);
        let planet = &self.registry.planets[self.planet_index];
        let (u_h, v_h) = self.surface_coords(u, v);
        let dir = CoordSystem::get_direction(face, u_h, v_h, self.field_res);
        let fields =
            climate::sample_surface_fields(&self.registry, &self.noise_generators, planet, dir);
        let mut weights = [BiomeWeight::default(); MAX_BIOME_WEIGHTS];
        let (count, primary) =
            biome_select::resolve_biome_weights_into(&self.registry, planet, fields, &mut weights);
        SurfaceSample {
            height,
            primary_biome: primary,
            biome_weights: weights,
            weight_count: count,
            temperature: fields.temperature,
            humidity: fields.humidity,
            roughness: fields.roughness,
        }
    }

    pub fn terrain_surface_layer(&self, face: u8, u: u32, v: u32) -> u32 {
        let base = self.get_height(face, u, v) as i32;
        let adjusted = base + self.micro_height_offset(face, u, v);
        adjusted.clamp(0, self.voxel_res.saturating_sub(1) as i32) as u32
    }

    pub fn voxel_at(&self, coord: VoxelCoord, profile: PlanetProfile) -> VoxelId {
        if coord.layer >= self.voxel_res || coord.u >= self.voxel_res || coord.v >= self.voxel_res {
            return VoxelId::AIR;
        }
        let surface = self.surface_sample(coord.face, coord.u, coord.v);

        let surface_layer = self.terrain_surface_layer(coord.face, coord.u, coord.v);
        let depth_from_surface = surface_layer as i32 - coord.layer as i32;
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

    // Two-octave height perturbation for organic surface detail. Kept in one
    // function so voxel resolution and mesh candidate generation cannot drift.
    fn micro_height_offset(&self, face: u8, u: u32, v: u32) -> i32 {
        // Octave 1: 4-voxel cells, +/-2 voxels.
        let detail = {
            const S: u32 = 4;
            let cu = u / S;
            let cv = v / S;
            let fu = (u % S) as f32 / S as f32;
            let fv = (v % S) as f32 / S as f32;
            let c = |cu: u32, cv: u32| -> f32 {
                hash4(face, cu, cv, 0xC3A5B1D7) as f32 / u32::MAX as f32 * 4.0 - 2.0
            };
            let su = fu * fu * (3.0 - 2.0 * fu);
            let sv = fv * fv * (3.0 - 2.0 * fv);
            c(cu, cv) * (1.0 - su) * (1.0 - sv)
                + c(cu + 1, cv) * su * (1.0 - sv)
                + c(cu, cv + 1) * (1.0 - su) * sv
                + c(cu + 1, cv + 1) * su * sv
        };
        // Octave 2: 16-voxel cells, +/-3 voxels.
        let broad = {
            const S: u32 = 16;
            let cu = u / S;
            let cv = v / S;
            let fu = (u % S) as f32 / S as f32;
            let fv = (v % S) as f32 / S as f32;
            let c = |cu: u32, cv: u32| -> f32 {
                hash4(face, cu, cv, 0x7B4F2E91) as f32 / u32::MAX as f32 * 6.0 - 3.0
            };
            let su = fu * fu * (3.0 - 2.0 * fu);
            let sv = fv * fv * (3.0 - 2.0 * fv);
            c(cu, cv) * (1.0 - su) * (1.0 - sv)
                + c(cu + 1, cv) * su * (1.0 - sv)
                + c(cu, cv + 1) * (1.0 - su) * sv
                + c(cu + 1, cv + 1) * su * sv
        };
        (detail + broad).round() as i32
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

    pub fn surface_layer(&self) -> u32 {
        self.surface_layer
    }

    /// The planet profile used to build this terrain (voxel resolution,
    /// surface layer, core layers, etc.).  Required by callers that invoke
    /// `voxel_at` without owning a copy of the profile.
    pub fn profile(&self) -> vv_voxel::PlanetProfile {
        self.profile
    }

    pub fn sea_level_layer(&self) -> u32 {
        let level = self.profile.surface_layer as i32 + self.planet().sea_level_offset;
        level.clamp(0, self.profile.resolution.saturating_sub(1) as i32) as u32
    }

    pub fn water_block(&self) -> Option<VoxelId> {
        self.planet().water_block
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
    crate::features::range_pick(range, roll)
}

pub(super) fn hash4(face: u8, u: u32, v: u32, salt: u32) -> u32 {
    crate::features::hash4(face, u, v, salt)
}
