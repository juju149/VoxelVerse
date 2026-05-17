//! Tiny deterministic 64-bit RNG (PCG-XSH-RR variant).
//!
//! Phase-2 of the weather/cosmos roadmap requires same-seed-same-output —
//! pulling in `rand` for two operations would be overkill. This implementation
//! is ~20 lines, zero allocations, perfectly fine for picking weather
//! transitions and gust intervals.
//!
//! Reference: O'Neill, "PCG: A Family of Simple Fast Space-Efficient
//! Statistically Good Algorithms for Random Number Generation", 2014.

const MULTIPLIER: u64 = 6_364_136_223_846_793_005;
const INCREMENT: u64 = 1_442_695_040_888_963_407;

#[derive(Clone, Copy, Debug)]
pub struct PcgRng {
    state: u64,
}

impl PcgRng {
    pub fn new(seed: u64) -> Self {
        // Mix the seed into the state with one step so seed=0 isn't degenerate.
        let mut rng = Self {
            state: seed.wrapping_add(INCREMENT),
        };
        let _ = rng.next_u32();
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let x = self.state;
        self.state = x.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
        let xor = (((x >> 18) ^ x) >> 27) as u32;
        let rot = (x >> 59) as u32;
        xor.rotate_right(rot)
    }

    /// Uniform `f32` in `[0, 1)`.
    pub fn next_unit(&mut self) -> f32 {
        // 24 bits of precision; matches IEEE-754 f32 mantissa.
        const SCALE: f32 = 1.0 / (1u32 << 24) as f32;
        ((self.next_u32() >> 8) as f32) * SCALE
    }

    /// Uniform `f32` in `[lo, hi)`. Order-invariant: `lo` and `hi` may be swapped.
    pub fn next_range(&mut self, lo: f32, hi: f32) -> f32 {
        let (a, b) = if lo <= hi { (lo, hi) } else { (hi, lo) };
        a + self.next_unit() * (b - a)
    }
}

#[cfg(test)]
mod tests {
    use super::PcgRng;

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut a = PcgRng::new(0xC0FFEE);
        let mut b = PcgRng::new(0xC0FFEE);
        for _ in 0..32 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn different_seed_diverges() {
        let mut a = PcgRng::new(1);
        let mut b = PcgRng::new(2);
        let mut differ = false;
        for _ in 0..8 {
            if a.next_u32() != b.next_u32() {
                differ = true;
                break;
            }
        }
        assert!(
            differ,
            "two distinct seeds must not produce identical streams"
        );
    }

    #[test]
    fn unit_stays_in_unit_interval() {
        let mut rng = PcgRng::new(42);
        for _ in 0..1024 {
            let v = rng.next_unit();
            assert!((0.0..1.0).contains(&v), "out of range: {v}");
        }
    }
}
