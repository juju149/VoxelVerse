pub(crate) fn hash01(face: u8, u: u32, v: u32, layer: u32, salt: u32) -> f32 {
    let mut x = face as u64;
    x = x.wrapping_mul(0x9E37_79B1_85EB_CA87) ^ u as u64;
    x = x.wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ v as u64;
    x = x.wrapping_mul(0x1656_67B1_9E37_79F9) ^ layer as u64;
    x = x.wrapping_mul(0x85EB_CA77_C2B2_AE63) ^ salt as u64;
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;

    ((x & 0xFFFF_FFFF) as f32) / u32::MAX as f32
}
