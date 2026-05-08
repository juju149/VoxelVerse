// Shared Perlin noise primitives — used by both terrain and biome generation.
use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub enum NoiseType {
    Perlin,
    /// 1.0 - |2·perlin - 1| per octave → sharp mountain ridges.
    Ridged,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct NoiseSettings {
    pub noise_type: NoiseType,
    pub frequency: f32,
    pub amplitude: f32,
    pub octaves: u32,
    pub persistence: f32,
    pub lacunarity: f32,
    pub offset: Vec3,
}

pub(crate) struct NoiseGenerator {
    perm: [u8; 512],
}

impl NoiseGenerator {
    pub(crate) fn new(seed: u32) -> Self {
        let mut p = [0u8; 512];
        let mut permutation: Vec<u8> = (0..=255).collect();
        let mut state = seed;
        for i in (1..256).rev() {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let j = (state as usize) % (i + 1);
            permutation.swap(i, j);
        }
        p[..256].copy_from_slice(&permutation[..256]);
        p[256..512].copy_from_slice(&permutation[..256]);
        Self { perm: p }
    }

    pub(crate) fn compute(&self, pos: Vec3, settings: &NoiseSettings) -> f32 {
        if settings.octaves <= 1 {
            let p = pos * settings.frequency + settings.offset;
            return self.compute_base(p, settings.noise_type);
        }

        let mut total_val = 0.0_f32;
        let mut total_amp = 0.0_f32;
        let mut amp = 1.0_f32;
        let mut freq = settings.frequency;

        for _ in 0..settings.octaves {
            let sample_pos = pos * freq + settings.offset;
            total_val += self.compute_base(sample_pos, settings.noise_type) * amp;
            total_amp += amp;
            amp *= settings.persistence;
            freq *= settings.lacunarity;
        }

        if total_amp > 0.0 {
            total_val / total_amp
        } else {
            0.0
        }
    }

    fn compute_base(&self, p: Vec3, type_: NoiseType) -> f32 {
        match type_ {
            NoiseType::Perlin => (self.perlin(p) + 1.0) * 0.5,
            // Ridged: fold so valleys become 0, ridges become 1.
            NoiseType::Ridged => {
                let v = (self.perlin(p) + 1.0) * 0.5;
                1.0 - (2.0 * v - 1.0).abs()
            }
        }
    }

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
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };
    (if (h & 1) == 0 { u } else { -u }) + (if (h & 2) == 0 { v } else { -v })
}
