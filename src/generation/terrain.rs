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

        // Continental base — very low frequency, sets broad highland vs lowland zones.
        let continental = NoiseSettings {
            noise_type: NoiseType::Perlin,
            frequency: 0.75,
            amplitude,
            octaves: 3,
            persistence: 0.55,
            lacunarity: 2.0,
            offset: Vec3::ZERO,
        };

        // Rolling terrain — medium frequency, the main visible terrain shape.
        let rolling = NoiseSettings {
            noise_type: NoiseType::Perlin,
            frequency: 1.85,
            amplitude,
            octaves: 5,
            persistence: 0.48,
            lacunarity: 2.0,
            offset: Vec3::splat(7.3),
        };

        // Domain-warped ridged noise for mountain chains.
        // The domain warp gives the chains their organic, curved appearance.
        let ridge = NoiseSettings {
            noise_type: NoiseType::Ridged,
            frequency: 1.4,
            amplitude,
            octaves: 6,
            persistence: 0.52,
            lacunarity: 2.1,
            offset: Vec3::splat(33.0),
        };

        // Fine detail — high frequency, adds micro-roughness.
        let detail = NoiseSettings {
            noise_type: NoiseType::Perlin,
            frequency: rolling.frequency * 3.7,
            amplitude,
            octaves: 3,
            persistence: 0.42,
            lacunarity: 2.15,
            offset: Vec3::splat(17.0),
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
            // Continental base drives whether this cell is a highland or basin.
            let cont_v = gen.compute(dir, &continental) * 2.0 - 1.0; // −1..1

            // Rolling terrain blended on top.
            let roll_v = gen.compute(dir, &rolling) * 2.0 - 1.0;

            // Domain-warped ridged noise: the warp_scale of 0.45 creates strongly
            // curved chains.  The result is in 0..1 (ridged) → shifted to −1..1.
            let ridge_v = gen.domain_warp(dir, &ridge, 0.45) * 2.0 - 1.0;

            // Detail noise.
            let detail_v = gen.compute(dir, &detail) * 2.0 - 1.0;

            // --- Biome-driven modulation. ---
            let biome_id = biome_map.get_biome_id(face, u_coord, v_coord) as usize;
            let (amp_scale, flatness) = biome_params.get(biome_id).copied().unwrap_or((1.0, 0.0));

            // Mountain character: high amp + low flatness → strong ridge mixing.
            let mountain_char = amp_scale * (1.0 - flatness).max(0.0);
            let flat_factor = 1.0 - flatness;

            // Base terrain: continental + rolling blend.
            let base = (cont_v * 0.35 + roll_v * 0.65) * flat_factor;

            // Ridge contribution: domain-warped ridged noise, scaled by mountain character.
            // Blended smoothly so plains biomes feel genuinely flat.
            let ridge_contrib = ridge_v * mountain_char * mountain_char;

            // Fine detail (small, present everywhere but dampened in flat zones).
            let detail_contrib = detail_v * 0.12 * flat_factor;

            // Latitude polar smoothing — poles are naturally gentler.
            let lat = dir.y.abs();
            let polar_damp = (1.0 - lat * 0.15).clamp(0.85, 1.0);

            let h_offset = (base * 0.75 + ridge_contrib * 0.95 + detail_contrib)
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
        let u_h = ((u as u64 * self.heightmap_res as u64) / self.voxel_res as u64)
            .min(self.heightmap_res as u64 - 1) as u32;
        let v_h = ((v as u64 * self.heightmap_res as u64) / self.voxel_res as u64)
            .min(self.heightmap_res as u64 - 1) as u32;

        let idx = Self::get_index(face, u_h, v_h, self.heightmap_res);
        let offset = self.heights[idx] as i32;
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
