pub(crate) fn hash01(seed: u32, a: u32, b: u32, c: u32) -> f32 {
    let mut x = seed
        ^ a.wrapping_mul(0x9E37_79B9)
        ^ b.wrapping_mul(0x85EB_CA6B)
        ^ c.wrapping_mul(0xC2B2_AE35);

    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;

    x as f32 / u32::MAX as f32
}
