use glam::Vec3;
use std::sync::Arc;
use vv_planet::CoordSystem;
use vv_config::WorldGenConfig;

// --- Noise configuration types ----------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum NoiseType {
    Perlin,
    Simplex,
    Cellular,
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
    fn from_config(res: u32, cfg: &WorldGenConfig) -> Self {
        Self {
            noise_type: NoiseType::Perlin,
            frequency: res as f32 / 100.0,
            amplitude: cfg.terrain_amplitude,
            octaves: cfg.noise_octaves,
            persistence: cfg.noise_persistence,
            lacunarity: cfg.noise_lacunarity,
            offset: Vec3::ZERO,
        }
    }
}

// --- Planet terrain ---------------------------------------------------------

/// Pre-computed per-face heightmap for a planet.
///
/// Wraps the raw heights in an `Arc` so cloning is cheap (used for
/// threaded mesh generation).
pub struct PlanetTerrain {
    heights: Arc<Vec<u16>>,
    resolution: u32,
}

impl PlanetTerrain {
    /// Build the terrain heightmap for `resolution` from `cfg`.
    pub fn new(resolution: u32, cfg: &WorldGenConfig) -> Self {
        println!("Generating terrain noise map (res {})…", resolution);
        let settings = NoiseSettings::from_config(resolution, cfg);
        let generator = NoiseGenerator::new(cfg.noise_seed);
        let base_radius = resolution as f32 / 2.0;
        let size = (6 * resolution * resolution) as usize;
        let mut heights = vec![0u16; size];

        for face in 0u8..6 {
            for v in 0..resolution {
                for u in 0..resolution {
                    let dir = CoordSystem::get_direction(face, u, v, resolution);
                    let noise_val = generator.compute(dir, &settings);
                    let h = (base_radius + noise_val * settings.amplitude).max(1.0) as u16;
                    heights[Self::index(face, u, v, resolution)] = h;
                }
            }
        }

        println!("Terrain generation complete.");
        Self { heights: Arc::new(heights), resolution }
    }

    #[inline(always)]
    fn index(face: u8, u: u32, v: u32, res: u32) -> usize {
        (face as usize) * (res as usize) * (res as usize)
            + (v as usize) * (res as usize)
            + u as usize
    }

    pub fn get_height(&self, face: u8, u: u32, v: u32) -> u32 {
        let u = u.min(self.resolution - 1);
        let v = v.min(self.resolution - 1);
        self.heights[Self::index(face, u, v, self.resolution)] as u32
    }
}

impl Clone for PlanetTerrain {
    fn clone(&self) -> Self {
        Self { heights: self.heights.clone(), resolution: self.resolution }
    }
}

// --- Noise generator --------------------------------------------------------

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

    fn compute(&self, pos: Vec3, settings: &NoiseSettings) -> f32 {
        if settings.octaves <= 1 {
            let p = pos * settings.frequency + settings.offset;
            return self.compute_base(p, settings.noise_type);
        }
        let mut total = 0.0f32;
        let mut total_amp = 0.0f32;
        let mut amp = 1.0f32;
        let mut freq = settings.frequency;
        for _ in 0..settings.octaves {
            let sp = pos * freq + settings.offset;
            total += self.compute_base(sp, settings.noise_type) * amp;
            total_amp += amp;
            amp *= settings.persistence;
            freq *= settings.lacunarity;
        }
        if total_amp > 0.0 { total / total_amp } else { 0.0 }
    }

    fn compute_base(&self, p: Vec3, kind: NoiseType) -> f32 {
        match kind {
            NoiseType::Perlin => (self.perlin(p) + 1.0) * 0.5,
            NoiseType::Simplex | NoiseType::Cellular => 0.0,
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
        let a  = self.perm[x_int as usize] as usize + y_int as usize;
        let aa = self.perm[a] as usize + z_int as usize;
        let ab = self.perm[a + 1] as usize + z_int as usize;
        let b  = self.perm[x_int as usize + 1] as usize + y_int as usize;
        let ba = self.perm[b] as usize + z_int as usize;
        let bb = self.perm[b + 1] as usize + z_int as usize;
        lerp(w,
            lerp(v,
                lerp(u, grad(self.perm[aa],   x,       y,       z),
                         grad(self.perm[ba],   x - 1.0, y,       z)),
                lerp(u, grad(self.perm[ab],   x,       y - 1.0, z),
                         grad(self.perm[bb],   x - 1.0, y - 1.0, z))),
            lerp(v,
                lerp(u, grad(self.perm[aa+1], x,       y,       z - 1.0),
                         grad(self.perm[ba+1], x - 1.0, y,       z - 1.0)),
                lerp(u, grad(self.perm[ab+1], x,       y - 1.0, z - 1.0),
                         grad(self.perm[bb+1], x - 1.0, y - 1.0, z - 1.0))))
    }
}

// --- Math helpers -----------------------------------------------------------

fn fade(t: f32) -> f32 { t * t * t * (t * (t * 6.0 - 15.0) + 10.0) }
fn lerp(t: f32, a: f32, b: f32) -> f32 { a + t * (b - a) }

fn grad(hash: u8, x: f32, y: f32, z: f32) -> f32 {
    let h = (hash & 15) as i32;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
    (if (h & 1) != 0 { -u } else { u }) + (if (h & 2) != 0 { -v } else { v })
}
