// VoxelVerse — Tonemapping utilities
//
// Functions available for use by any shader module.
// Each function takes a linear-light HDR color and returns a tonemapped
// value that should then be gamma-encoded before output.

// ── ACES filmic (Narkowicz 2015 approximation) ────────────────────────────────
// Classic choice. Tends to push warm/red hues and de-saturate.
// Kept for reference and the post/final_composite pipeline.
fn vv_tonemap_aces(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e),
                 vec3<f32>(0.0), vec3<f32>(1.0));
}

// ── AgX (Troy Sobotka) — simplified approximation ────────────────────────────
// Better hue stability than ACES: greens stay green, no burnt-orange skies.
// Does NOT include gamma encoding — call vv_linear_to_srgb() after this.
fn agx_default_contrast(x: vec3<f32>) -> vec3<f32> {
    let x2 = x * x;
    let x4 = x2 * x2;
    return  15.5     * x4 * x2
          - 40.14    * x4 * x
          + 31.96    * x4
          -  6.868   * x2 * x
          +  0.4298  * x2
          +  0.1191  * x
          -  0.00232;
}

fn vv_tonemap_agx(c: vec3<f32>, exposure: f32) -> vec3<f32> {
    let mat = mat3x3<f32>(
        vec3<f32>(0.842479, 0.042328, 0.042376),
        vec3<f32>(0.078434, 0.878469, 0.079166),
        vec3<f32>(0.079224, 0.079116, 0.879143),
    );
    let min_ev = -12.47393;
    let max_ev =   4.026069;
    var v = mat * max(c * exposure, vec3<f32>(1e-10));
    v = clamp((log2(v) - min_ev) / (max_ev - min_ev), vec3<f32>(0.0), vec3<f32>(1.0));
    return agx_default_contrast(v);
}

// ── Khronos PBR Neutral (2023) ────────────────────────────────────────────────
// Very clean, saturation-preserving, good for stylized looks.
fn vv_tonemap_neutral(c: vec3<f32>) -> vec3<f32> {
    let start_compression = 0.8 - 0.04;
    let d = 1.0 - start_compression;
    let peak = max(max(c.r, c.g), c.b);
    let f = select(0.0, (peak - start_compression) / (d * d) * (2.0 * d - (peak - start_compression) / (2.0 * d)), peak > start_compression);
    return clamp(c + f, vec3<f32>(0.0), vec3<f32>(1.0));
}

