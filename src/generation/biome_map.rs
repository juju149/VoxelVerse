use crate::content::BiomeRegistry;
use crate::generation::{
    CoordSystem,
    noise::{NoiseGenerator, NoiseSettings, NoiseType},
};
use crate::world::PlanetProfile;
use glam::Vec3;
use rayon::prelude::*;
use std::sync::Arc;

/// Maximum biome-map resolution per cube-face axis.
/// Matches the terrain heightmap cap so indices align 1:1.
const MAX_BIOME_RES: u32 = 2048;

/// Per-cell biome index map, parallel to the terrain heightmap.
/// Stores a compact `u8` index into `BiomeRegistry` for each surface point.
pub struct BiomeMap {
    indices: Arc<Vec<u8>>,
    heightmap_res: u32,
    voxel_res: u32,
}

impl BiomeMap {
    /// Build the biome map for a planet using a 2D climate model.
    ///
    /// Climate axes:
    /// - **Temperature** (0 = arctic, 1 = tropical) derived from latitude + small jitter.
    /// - **Roughness** (0 = flat, 1 = mountainous) from a large-scale noise field that
    ///   is fully independent of latitude, creating mountain chains that cross climate zones.
    ///
    /// Each cell selects the biome whose (temperature_center, roughness_center) is
    /// nearest in 2D Euclidean distance — a Voronoi-style assignment, no explicit ranges.
    pub(crate) fn new(profile: PlanetProfile, biome_registry: &BiomeRegistry) -> Self {
        let heightmap_res = profile.resolution.min(MAX_BIOME_RES);
        let size = 6 * heightmap_res as usize * heightmap_res as usize;

        let biomes = biome_registry.biomes();

        // Seed offsets prevent the biome boundaries from aligning with terrain ridges.
        let gen = NoiseGenerator::new(profile.seed.wrapping_add(0x00B1_01E0));

        // Small jitter on temperature to create organic coastline-like biome edges.
        let temp_jitter = NoiseSettings {
            noise_type: NoiseType::Perlin,
            frequency: 0.75,
            amplitude: 1.0,
            octaves: 2,
            persistence: 0.5,
            lacunarity: 2.0,
            offset: Vec3::splat(55.5),
        };

        // Large-scale roughness field — drives mountain-chain placement.
        // Low frequency → very broad features spanning a significant fraction of the planet.
        let roughness_gen = NoiseGenerator::new(profile.seed.wrapping_add(0xDEAD_F00D));
        let roughness_settings = NoiseSettings {
            noise_type: NoiseType::Perlin,
            frequency: 0.55,
            amplitude: 1.0,
            octaves: 3,
            persistence: 0.6,
            lacunarity: 2.2,
            offset: Vec3::new(12.4, -7.1, 3.8),
        };

        let mut indices = vec![0u8; size];
        indices.par_iter_mut().enumerate().for_each(|(idx, out)| {
            let hres = heightmap_res as usize;
            let face = (idx / (hres * hres)) as u8;
            let rem = idx % (hres * hres);
            let v = (rem / hres) as u32;
            let u = (rem % hres) as u32;

            let dir = CoordSystem::get_direction(face, u, v, heightmap_res);

            // --- Temperature: latitude-based, with a small noise jitter. ---
            let latitude = dir.y.abs(); // 0 = equator, 1 = pole
            let jitter = gen.compute(dir, &temp_jitter) * 2.0 - 1.0; // −1..1
            let temperature = (1.0 - latitude + jitter * 0.08).clamp(0.0, 1.0);

            // --- Roughness: independent large-scale noise field. ---
            // smoothstep sharpens the transition between flat and rough zones.
            let raw_r = roughness_gen.compute(dir, &roughness_settings); // 0..1
            let roughness = smoothstep(raw_r);

            // --- Biome selection: nearest in (temperature, roughness) 2D space. ---
            *out = biomes
                .iter()
                .min_by(|a, b| {
                    let da = climate_dist(temperature, roughness, a.temperature_center, a.roughness_center);
                    let db = climate_dist(temperature, roughness, b.temperature_center, b.roughness_center);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|b| b.id)
                .unwrap_or(0);
        });

        Self {
            indices: Arc::new(indices),
            heightmap_res,
            voxel_res: profile.resolution,
        }
    }

    /// Get the biome ID for a voxel-space surface coordinate.
    #[inline(always)]
    pub fn get_biome_id(&self, face: u8, u: u32, v: u32) -> u8 {
        let hres = self.heightmap_res as u64;
        let u_h = ((u as u64 * hres) / self.voxel_res as u64).min(hres - 1) as u32;
        let v_h = ((v as u64 * hres) / self.voxel_res as u64).min(hres - 1) as u32;
        let hres = self.heightmap_res as usize;
        let idx = face as usize * hres * hres + v_h as usize * hres + u_h as usize;
        self.indices[idx]
    }
}

impl Clone for BiomeMap {
    fn clone(&self) -> Self {
        Self {
            indices: self.indices.clone(),
            heightmap_res: self.heightmap_res,
            voxel_res: self.voxel_res,
        }
    }
}

/// Weighted 2D Euclidean distance in climate space.
/// Temperature differences are weighted slightly more than roughness to preserve
/// the intuitive hot-to-cold gradient while still allowing mountain bands.
#[inline(always)]
fn climate_dist(t: f32, r: f32, tc: f32, rc: f32) -> f32 {
    let dt = (t - tc) * 1.4;
    let dr = r - rc;
    dt * dt + dr * dr
}

/// Smooth Hermite interpolation — sharpens the mid-range roughness transitions
/// without creating hard cliffs in the biome map.
#[inline(always)]
fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}
