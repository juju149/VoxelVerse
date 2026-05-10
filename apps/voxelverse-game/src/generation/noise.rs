//! Noise generators for terrain and climate generation.
//!
//! # Primary noise: OpenSimplex2S (3D simplex)
//! Tetrahedral lattice — no grid-aligned directional artifacts, smooth C²
//! continuity.  Replaces the old Perlin implementation as the engine default.
//!
//! # Ridged Multifractal (Musgrave 1994)
//! Spectral weighting so fine octave detail appears on ridgelines and fades
//! in valleys.  Produces natural mountain chains and eroded escarpments.
//!
//! All generators sample in 3-D world space — never in face-UV space — so
//! there are no cube-sphere seam artifacts.
use glam::Vec3;

/// 12 gradient directions from the midpoints of a cube's edges.
/// Uniformly distributed on a sphere → no directional bias.
const GRAD3: [(f32, f32, f32); 12] = [
    (1.0, 1.0, 0.0),
    (-1.0, 1.0, 0.0),
    (1.0, -1.0, 0.0),
    (-1.0, -1.0, 0.0),
    (1.0, 0.0, 1.0),
    (-1.0, 0.0, 1.0),
    (1.0, 0.0, -1.0),
    (-1.0, 0.0, -1.0),
    (0.0, 1.0, 1.0),
    (0.0, -1.0, 1.0),
    (0.0, 1.0, -1.0),
    (0.0, -1.0, -1.0),
];

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub enum NoiseType {
    /// Classic Perlin — kept for backward compatibility only.  Grid artifacts possible.
    Perlin,
    /// OpenSimplex2S — 3D simplex on a tetrahedral lattice.
    /// Primary noise for all terrain in VoxelVerse.  No directional artifacts.
    OpenSimplex2S,
    /// Simple ridge fold `1 − |2v−1|` using OpenSimplex2S base.
    Ridged,
    /// Ridged multifractal (Musgrave 1994).
    /// Spectral weights make fine detail appear on ridges only.
    /// Best for mountain chains, cliff faces, and eroded terrain.
    RidgedMulti,
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

/// One generator per noise field, seeded from `planet_seed ^ hash(salt)`.
/// Read-only after construction — safe to share across rayon workers.
#[derive(Clone)]
pub(crate) struct NoiseGenerator {
    perm: [u8; 512],
}

impl NoiseGenerator {
    pub(crate) fn new(seed: u32) -> Self {
        let mut p = [0u8; 512];
        let mut permutation: Vec<u8> = (0..=255u8).collect();
        // Fisher-Yates shuffle with a linear congruential generator.
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

    /// Evaluate fBm or ridged-multifractal noise.  Returns ≈ 0..1.
    pub(crate) fn compute(&self, pos: Vec3, settings: &NoiseSettings) -> f32 {
        if settings.noise_type == NoiseType::RidgedMulti {
            return self.compute_ridged_multi(pos, settings);
        }
        if settings.octaves <= 1 {
            return self.compute_base(pos * settings.frequency + settings.offset, settings.noise_type);
        }
        // Standard fBm accumulation (Musgrave 1988).
        let mut total = 0.0_f32;
        let mut total_amp = 0.0_f32;
        let mut amp = 1.0_f32;
        let mut freq = settings.frequency;
        for _ in 0..settings.octaves {
            let p = pos * freq + settings.offset;
            total += self.compute_base(p, settings.noise_type) * amp;
            total_amp += amp;
            amp *= settings.persistence;
            freq *= settings.lacunarity;
        }
        if total_amp > 0.0 { total / total_amp } else { 0.5 }
    }

    // ── Base kernels ─────────────────────────────────────────────────────────

    fn compute_base(&self, p: Vec3, type_: NoiseType) -> f32 {
        match type_ {
            NoiseType::Perlin => (self.perlin(p) + 1.0) * 0.5,
            NoiseType::OpenSimplex2S => {
                (self.simplex3d(p.x, p.y, p.z).clamp(-1.0, 1.0) + 1.0) * 0.5
            }
            NoiseType::Ridged => {
                // Simple fold using simplex base — fast approximation.
                let v = (self.simplex3d(p.x, p.y, p.z).clamp(-1.0, 1.0) + 1.0) * 0.5;
                1.0 - (2.0 * v - 1.0).abs()
            }
            NoiseType::RidgedMulti => unreachable!("dispatched before compute_base"),
        }
    }

    // ── OpenSimplex2S ─────────────────────────────────────────────────────────
    //
    // 3-D simplex noise (Gustavson 2012).  Tetrahedral simplex lattice so
    // there are no axis-aligned grid seams.  Returns approximately −1..1;
    // callers clamp before normalising to 0..1.
    //
    // Reference: https://weber.itn.liu.se/~stegu/simplexnoise/simplexnoise.pdf

    fn simplex3d(&self, x: f32, y: f32, z: f32) -> f32 {
        // Simplex skew / unskew factors for 3-D.
        const F3: f32 = 1.0 / 3.0;
        const G3: f32 = 1.0 / 6.0;

        // Skew input space to find which simplex cell contains the point.
        let s = (x + y + z) * F3;
        let i = (x + s).floor() as i32;
        let j = (y + s).floor() as i32;
        let k = (z + s).floor() as i32;

        // Unskew the cell origin back to xyz space.
        let t = (i + j + k) as f32 * G3;
        let x0 = x - (i as f32 - t);
        let y0 = y - (j as f32 - t);
        let z0 = z - (k as f32 - t);

        // Determine which of the six 3-D simplex tetrahedra the point is in.
        let (i1, j1, k1, i2, j2, k2): (usize, usize, usize, usize, usize, usize) =
            if x0 >= y0 {
                if y0 >= z0      { (1, 0, 0, 1, 1, 0) } // X Y Z
                else if x0 >= z0 { (1, 0, 0, 1, 0, 1) } // X Z Y
                else             { (0, 0, 1, 1, 0, 1) } // Z X Y
            } else if y0 < z0    { (0, 0, 1, 0, 1, 1) } // Z Y X
            else if x0 < z0      { (0, 1, 0, 0, 1, 1) } // Y Z X
            else                 { (0, 1, 0, 1, 1, 0) }; // Y X Z

        // Displacements from the other three simplex corners.
        let x1 = x0 - i1 as f32 + G3;
        let y1 = y0 - j1 as f32 + G3;
        let z1 = z0 - k1 as f32 + G3;
        let x2 = x0 - i2 as f32 + 2.0 * G3;
        let y2 = y0 - j2 as f32 + 2.0 * G3;
        let z2 = z0 - k2 as f32 + 2.0 * G3;
        let x3 = x0 - 1.0 + 3.0 * G3;
        let y3 = y0 - 1.0 + 3.0 * G3;
        let z3 = z0 - 1.0 + 3.0 * G3;

        // Gradient indices for the four simplex corners.
        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;
        let kk = (k & 255) as usize;
        let gi0 = self.perm3(ii,      jj,      kk     ) % 12;
        let gi1 = self.perm3(ii + i1, jj + j1, kk + k1) % 12;
        let gi2 = self.perm3(ii + i2, jj + j2, kk + k2) % 12;
        let gi3 = self.perm3(ii + 1,  jj + 1,  kk + 1 ) % 12;

        // Sum contributions from all four corners.
        let n0 = Self::simplex_corner3(gi0, x0, y0, z0);
        let n1 = Self::simplex_corner3(gi1, x1, y1, z1);
        let n2 = Self::simplex_corner3(gi2, x2, y2, z2);
        let n3 = Self::simplex_corner3(gi3, x3, y3, z3);

        // Empirical scale factor (Gustavson 2012, 3-D simplex).
        (n0 + n1 + n2 + n3) * 32.0
    }

    /// Chain three permutation table lookups.
    /// Inputs i, j, k are each at most 256; perm[512] guarantees safety.
    #[inline]
    fn perm3(&self, i: usize, j: usize, k: usize) -> usize {
        let a = self.perm[k & 511] as usize;
        let b = self.perm[(a + j) & 511] as usize;
        self.perm[(b + i) & 511] as usize
    }

    /// Radial kernel contribution from a single simplex corner.
    /// Uses a C² (quartic) falloff kernel with radius √0.6 ≈ 0.775.
    #[inline]
    fn simplex_corner3(gi: usize, x: f32, y: f32, z: f32) -> f32 {
        let t = 0.6 - x * x - y * y - z * z;
        if t < 0.0 {
            0.0
        } else {
            let t2 = t * t;
            let (gx, gy, gz) = GRAD3[gi];
            t2 * t2 * (gx * x + gy * y + gz * z)
        }
    }

    // ── Ridged Multifractal ───────────────────────────────────────────────────
    //
    // Musgrave 1994: each octave's weight depends on the previous ridge signal
    // so high-frequency detail accumulates on sharp peaks and fades on flats.
    // Output ≈ 0..1 where 1 = sharp ridge peak, 0 = flat plain / deep valley.

    fn compute_ridged_multi(&self, pos: Vec3, settings: &NoiseSettings) -> f32 {
        let mut result = 0.0_f32;
        let mut weight = 1.0_f32;
        let mut freq = settings.frequency;
        let mut amp = 1.0_f32;
        let mut amp_sum = 0.0_f32;

        for _ in 0..settings.octaves {
            let p = pos * freq + settings.offset;
            // Fold the simplex output to create sharp ridge peaks.
            let raw = self.simplex3d(p.x, p.y, p.z).clamp(-1.0, 1.0);
            let signal = (1.0 - raw.abs()).clamp(0.0, 1.0);
            let signal = signal * signal; // sharpen ridge peaks

            result += signal * weight * amp;
            amp_sum += amp;

            // Spectral weighting: next octave is amplified where ridges are strong.
            weight = (signal * 2.0).clamp(0.0, 1.0);
            freq *= settings.lacunarity;
            amp *= settings.persistence;
        }

        (result / amp_sum.max(0.001)).clamp(0.0, 1.0)
    }

    // ── Legacy Perlin ─────────────────────────────────────────────────────────

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
                lerp(u, grad(self.perm[aa], x, y, z), grad(self.perm[ba], x - 1.0, y, z)),
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

// ── Perlin helpers ────────────────────────────────────────────────────────────

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


