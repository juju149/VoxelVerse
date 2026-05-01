use glam::Vec3;

use crate::math::lerp;

pub(crate) struct NoiseGenerator {
    perm: [u8; 512],
}

impl NoiseGenerator {
    pub(crate) fn new(seed: u32) -> Self {
        let mut permutation: Vec<u8> = (0u8..=255).collect();
        let mut state = seed;

        for i in (1..256).rev() {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let j = (state as usize) % (i + 1);
            permutation.swap(i, j);
        }

        let mut perm = [0u8; 512];

        for i in 0..256 {
            perm[i] = permutation[i];
            perm[i + 256] = permutation[i];
        }

        Self { perm }
    }

    pub(crate) fn fractal(
        &self,
        pos: Vec3,
        scale: f32,
        octaves: u32,
        persistence: f32,
        lacunarity: f32,
    ) -> f32 {
        let mut total = 0.0;
        let mut total_amp = 0.0;
        let mut amp = 1.0;
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

        let value = lerp(
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
        );

        (value + 1.0) * 0.5
    }
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
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
