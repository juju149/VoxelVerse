use crate::content::BiomeRegistry;
use crate::generation::{
    biome_map::BiomeMap,
    noise::{NoiseGenerator, NoiseSettings, NoiseType},
    CoordSystem,
};
use crate::world::PlanetProfile;
use glam::Vec3;
use rayon::prelude::*;
use std::sync::Arc;

/// Maximum heightmap resolution per cube-face axis.
/// Caps memory at 6 × 2048² × 2 bytes ≈ 50 MB regardless of planet resolution.
const MAX_HEIGHTMAP_RES: u32 = 2048;

// --- PLANET TERRAIN DATA ---

pub struct PlanetTerrain {
    /// Terrain height stored as i16 offset from `surface_layer`.
    /// Range [-32767, 32767] layers — supports large max_terrain_offset values.
    heights: Arc<Vec<i16>>,
    /// Per-cell biome index map, same resolution as the heightmap.
    biome_map: BiomeMap,
    /// Resolution of the heightmap grid (capped at MAX_HEIGHTMAP_RES).
    heightmap_res: u32,
    /// Surface layer in voxel space (= profile.surface_layer).
    surface_layer: u32,
    /// Full voxel resolution of the planet (= profile.resolution).
    voxel_res: u32,
}

impl PlanetTerrain {
    pub fn new(profile: PlanetProfile, biome_registry: &BiomeRegistry) -> Self {
        let heightmap_res = profile.resolution.min(MAX_HEIGHTMAP_RES);
        let surface_layer = profile.surface_layer;
        let size = (6 * heightmap_res * heightmap_res) as usize;

        // --- 1. Build the biome map first. ---
        let biome_map = BiomeMap::new(profile, biome_registry);

        // --- 2. Extract per-biome terrain params. ---
        let biome_params: Vec<(f32, f32)> = biome_registry
            .biomes()
            .iter()
            .map(|b| (b.terrain_amplitude, b.terrain_flatness))
            .collect();

        // --- 3. Noise generators and settings. ---
        let gen = NoiseGenerator::new(profile.seed);
        let amplitude = profile.max_terrain_offset as f32;

        // Frequency scaling: normalises noise frequencies to physical cell count.
        // freq_scale = hres/256 → one rolling cycle ≈ 50 cells regardless of resolution.
        let freq_scale = (heightmap_res as f32 / 256.0).max(1.0);

        // Continental base — large-scale regions (highlands vs basins).
        // No domain warp — we want clean, smooth continental shapes.
        let continental = NoiseSettings {
            noise_type: NoiseType::Perlin,
            frequency: 0.70,
            amplitude,
            octaves: 4,
            persistence: 0.50,
            lacunarity: 2.0,
            offset: Vec3::ZERO,
        };

        // Hills layer — domain-warped to break rectilinear grid alignment.
        // Dominant mid-scale layer: gives rolling hills their organic, curved shapes.
        let hills = NoiseSettings {
            noise_type: NoiseType::Perlin,
            frequency: 2.2 * freq_scale,
            amplitude,
            octaves: 6,
            persistence: 0.52,
            lacunarity: 2.0,
            offset: Vec3::new(7.3, 3.1, 9.8),
        };

        // Micro-relief — domain-warped fine bumps that break any remaining linearity.
        // 4× finer than hills; contributes fractal roughness visible at walking scale.
        let micro = NoiseSettings {
            noise_type: NoiseType::Perlin,
            frequency: 7.0 * freq_scale,
            amplitude,
            octaves: 4,
            persistence: 0.50,
            lacunarity: 2.0,
            offset: Vec3::new(17.5, 22.1, 5.7),
        };

        // Domain-warped ridged noise for mountain chains.
        let ridge = NoiseSettings {
            noise_type: NoiseType::Ridged,
            frequency: 1.5 * freq_scale,
            amplitude,
            octaves: 5,
            persistence: 0.50,
            lacunarity: 2.0,
            offset: Vec3::splat(33.0),
        };

        let min_layer = (profile.core_layers as i32).saturating_add(2);
        let max_layer = (profile.resolution as i32).saturating_sub(3);

        // --- 4. Generate heightmap in parallel. ---
        let mut heights = vec![0i16; size];
        heights.par_iter_mut().enumerate().for_each(|(idx, h)| {
            let face_area = (heightmap_res * heightmap_res) as usize;
            let face = (idx / face_area) as u8;
            let rem = idx % face_area;
            let v_coord = (rem / heightmap_res as usize) as u32;
            let u_coord = (rem % heightmap_res as usize) as u32;

            let dir = CoordSystem::get_direction(face, u_coord, v_coord, heightmap_res);

            // --- Noise samples. ---
            // Continental: no warp — clean broad regional shapes.
            let cont_v = gen.compute(dir, &continental) * 2.0 - 1.0; // −1..1

            // Hills: domain-warped → curved, organic contours (eliminates straight lines).
            let hill_v = gen.domain_warp(dir, &hills, 0.30) * 2.0 - 1.0;

            // Micro-relief: domain-warped fine detail → fractal roughness at ground level.
            let micro_v = gen.domain_warp(dir, &micro, 0.18) * 2.0 - 1.0;

            // Domain-warped ridged noise for mountain chains.
            let ridge_v = gen.domain_warp(dir, &ridge, 0.40) * 2.0 - 1.0;

            // --- Biome-driven modulation. ---
            let biome_id = biome_map.get_biome_id(face, u_coord, v_coord) as usize;
            let (amp_scale, flatness) = biome_params.get(biome_id).copied().unwrap_or((1.0, 0.0));

            let flat_factor = 1.0 - flatness;

            // Multi-scale blend: continental shapes + hills + micro roughness.
            // Domain-warped hills & micro ensure no rectilinear patterns.
            let base = (cont_v * 0.25 + hill_v * 0.55 + micro_v * 0.20) * flat_factor;

            // Ridge: only meaningful in mountainous biomes (high amp_scale, low flatness).
            let mountain_char = (amp_scale * flat_factor).min(0.55);
            let ridge_contrib = ridge_v * mountain_char * 0.40;

            // Latitude polar smoothing — poles are naturally flatter.
            let lat = dir.y.abs();
            let polar_damp = (1.0 - lat * 0.20).clamp(0.80, 1.0);

            let h_offset = (base + ridge_contrib)
                * amplitude
                * polar_damp
                * amp_scale;

            let final_layer = (surface_layer as f32 + h_offset).round() as i32;
            let final_layer = final_layer.clamp(min_layer, max_layer);
            *h = (final_layer - surface_layer as i32).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        });

        Self {
            heights: Arc::new(heights),
            biome_map,
            heightmap_res,
            surface_layer,
            voxel_res: profile.resolution,
        }
    }

    #[inline(always)]
    fn get_index(face: u8, u: u32, v: u32, res: u32) -> usize {
        (face as usize) * (res as usize) * (res as usize)
            + (v as usize) * (res as usize)
            + (u as usize)
    }

    pub fn get_height(&self, face: u8, u: u32, v: u32) -> u32 {
        let hres = self.heightmap_res as f32;
        let vres = self.voxel_res as f32;

        // Fractional heightmap coordinate for this voxel.
        let hu = (u as f32 * hres / vres).min(hres - 1.001);
        let hv = (v as f32 * hres / vres).min(hres - 1.001);

        // Bilinear interpolation: eliminates the "flat-top plateau" staircase
        // that nearest-neighbour produces when each cell covers ~10 voxels.
        let u0 = hu as u32;
        let v0 = hv as u32;
        let u1 = (u0 + 1).min(self.heightmap_res - 1);
        let v1 = (v0 + 1).min(self.heightmap_res - 1);
        let fu = hu - u0 as f32;
        let fv = hv - v0 as f32;

        let h00 = self.heights[Self::get_index(face, u0, v0, self.heightmap_res)] as f32;
        let h10 = self.heights[Self::get_index(face, u1, v0, self.heightmap_res)] as f32;
        let h01 = self.heights[Self::get_index(face, u0, v1, self.heightmap_res)] as f32;
        let h11 = self.heights[Self::get_index(face, u1, v1, self.heightmap_res)] as f32;

        let h = h00 * (1.0 - fu) * (1.0 - fv)
              + h10 * fu          * (1.0 - fv)
              + h01 * (1.0 - fu) * fv
              + h11 * fu          * fv;

        let offset = h as i32;
        (self.surface_layer as i32 + offset).max(0) as u32
    }

    #[inline(always)]
    pub fn get_biome_id(&self, face: u8, u: u32, v: u32) -> u8 {
        self.biome_map.get_biome_id(face, u, v)
    }
}

impl Clone for PlanetTerrain {
    fn clone(&self) -> Self {
        Self {
            heights: self.heights.clone(),
            biome_map: self.biome_map.clone(),
            heightmap_res: self.heightmap_res,
            surface_layer: self.surface_layer,
            voxel_res: self.voxel_res,
        }
    }
}
