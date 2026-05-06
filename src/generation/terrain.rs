use crate::generation::CoordSystem;
use crate::world::PlanetProfile;
use glam::Vec3;
use rayon::prelude::*;
use std::sync::Arc;

/// Maximum heightmap resolution per cube-face axis.
/// Caps memory at 6 × 2048² × 1 byte ≈ 25 MB regardless of planet resolution.
const MAX_HEIGHTMAP_RES: u32 = 2048;

// --- SETTINGS & ENUMS ---

#[derive(Clone, Copy, Debug)]
pub enum NoiseType {
    Perlin,
}

#[derive(Clone, Copy, Debug)]
pub struct NoiseSettings {
    pub noise_type: NoiseType,
    pub frequency: f32,
    pub amplitude: f32,
    pub octaves: u32,
    pub persistence: f32,
    pub lacunarity: f32,
    pub offset: Vec3,
}

impl NoiseSettings {
    pub fn default_terrain(profile: PlanetProfile) -> Self {
        Self {
            noise_type: NoiseType::Perlin,
            frequency: 1.85,
            amplitude: profile.max_terrain_offset as f32,
            octaves: 5,
            persistence: 0.48,
            lacunarity: 2.0,
            offset: Vec3::ZERO,
        }
    }
}

// --- PLANET TERRAIN DATA ---

pub struct PlanetTerrain {
    /// Terrain height stored as i8 offset from `surface_layer`.
    /// Range [-127, 127] layers — compact and valid for any planet resolution.
    heights: Arc<Vec<i8>>,
    /// Resolution of the heightmap grid (capped at MAX_HEIGHTMAP_RES).
    heightmap_res: u32,
    /// Surface layer in voxel space (= profile.surface_layer).
    surface_layer: u32,
    /// Full voxel resolution of the planet (= profile.resolution).
    voxel_res: u32,
}

impl PlanetTerrain {
    pub fn new(profile: PlanetProfile) -> Self {
        let heightmap_res = profile.resolution.min(MAX_HEIGHTMAP_RES);
        let surface_layer = profile.surface_layer;
        let size = (6 * heightmap_res * heightmap_res) as usize;

        let generator = NoiseGenerator::new(profile.seed);
        let settings = NoiseSettings::default_terrain(profile);
        let detail_settings = NoiseSettings {
            frequency: settings.frequency * 3.7,
            amplitude: settings.amplitude,
            octaves: 3,
            persistence: 0.42,
            lacunarity: 2.15,
            offset: Vec3::splat(17.0),
            ..settings
        };

        let min_layer = (profile.core_layers as i32).saturating_add(2);
        let max_layer = (profile.resolution as i32).saturating_sub(3);

        let mut heights = vec![0i8; size];
        heights.par_iter_mut().enumerate().for_each(|(idx, h)| {
            let face_area = (heightmap_res * heightmap_res) as usize;
            let face = (idx / face_area) as u8;
            let rem = idx % face_area;
            let v = (rem / heightmap_res as usize) as u32;
            let u = (rem % heightmap_res as usize) as u32;

            let dir = CoordSystem::get_direction(face, u, v, heightmap_res);
            let rolling = generator.compute(dir, &settings) * 2.0 - 1.0;
            let detail = generator.compute(dir, &detail_settings) * 2.0 - 1.0;
            let latitude = dir.y.abs();
            let roundness_bias = (1.0 - latitude * 0.18).clamp(0.82, 1.0);
            let h_offset =
                (rolling * 0.78 + detail * 0.22) * settings.amplitude * roundness_bias;
            let final_layer = (surface_layer as f32 + h_offset).round() as i32;
            let final_layer = final_layer.clamp(min_layer, max_layer);
            *h = (final_layer - surface_layer as i32).clamp(-127, 127) as i8;
        });

        Self {
            heights: Arc::new(heights),
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
        // Scale voxel UV coords down to heightmap resolution (no-op when equal).
        let u_h = ((u as u64 * self.heightmap_res as u64) / self.voxel_res as u64)
            .min(self.heightmap_res as u64 - 1) as u32;
        let v_h = ((v as u64 * self.heightmap_res as u64) / self.voxel_res as u64)
            .min(self.heightmap_res as u64 - 1) as u32;

        let idx = Self::get_index(face, u_h, v_h, self.heightmap_res);
        let offset = self.heights[idx] as i32;
        (self.surface_layer as i32 + offset).max(0) as u32
    }
}

impl Clone for PlanetTerrain {
    fn clone(&self) -> Self {
        Self {
            heights: self.heights.clone(),
            heightmap_res: self.heightmap_res,
            surface_layer: self.surface_layer,
            voxel_res: self.voxel_res,
        }
    }
}

// --- NOISE GENERATOR ---

struct NoiseGenerator {
    perm: [u8; 512],
}

impl NoiseGenerator {
    fn new(seed: u32) -> Self {
        let mut p = [0u8; 512];
        let mut permutation: Vec<u8> = (0..=255).collect();
        let mut state = seed;
        for i in (1..256).rev() {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            let j = (state as usize) % (i + 1);
            permutation.swap(i, j);
        }

        p[..256].copy_from_slice(&permutation[..256]);
        p[256..512].copy_from_slice(&permutation[..256]);
        Self { perm: p }
    }

    fn compute(&self, pos: Vec3, settings: &NoiseSettings) -> f32 {
        if settings.octaves <= 1 {
            let p = pos * settings.frequency + settings.offset;
            return self.compute_base(p, settings.noise_type); // Returns 0..1
        }

        let mut total_val = 0.0;
        let mut total_amp = 0.0;

        let mut amp = 1.0;
        let mut freq = settings.frequency;
        for _ in 0..settings.octaves {
            let sample_pos = pos * freq + settings.offset;
            total_val += self.compute_base(sample_pos, settings.noise_type) * amp;
            total_amp += amp;

            amp *= settings.persistence;
            freq *= settings.lacunarity;
        }

        // normalize result to 0..1 range
        if total_amp > 0.0 {
            total_val / total_amp
        } else {
            0.0
        }
    }

    fn compute_base(&self, p: Vec3, type_: NoiseType) -> f32 {
        match type_ {
            NoiseType::Perlin => (self.perlin(p) + 1.0) * 0.5,
        }
    }

    // --- PERLIN MATH ---

    fn perlin(&self, pos: Vec3) -> f32 {
        let x = pos.x.floor();
        let y = pos.y.floor();
        let z = pos.z.floor();

        let xi = x as i32 & 255;
        let yi = y as i32 & 255;
        let zi = z as i32 & 255;

        let x = pos.x - x;
        let y = pos.y - y;
        let z = pos.z - z;

        let u = fade(x);
        let v = fade(y);
        let w = fade(z);

        let a = self.perm[xi as usize] as usize + yi as usize;
        let aa = self.perm[a] as usize + zi as usize;
        let ab = self.perm[a + 1] as usize + zi as usize;
        let b = self.perm[xi as usize + 1] as usize + yi as usize;
        let ba = self.perm[b] as usize + zi as usize;
        let bb = self.perm[b + 1] as usize + zi as usize;

        lerp(
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
        )
    }
}

// ---MATH-HELPERS---

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}
fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}
fn grad(hash: u8, x: f32, y: f32, z: f32) -> f32 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else {
        if h == 12 || h == 14 {
            x
        } else {
            z
        }
    };
    (if (h & 1) == 0 { u } else { -u }) + (if (h & 2) == 0 { v } else { -v })
}
