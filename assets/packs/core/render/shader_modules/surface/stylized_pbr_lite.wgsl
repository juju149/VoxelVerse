// VoxelVerse — Stylized PBR-Lite terrain fragment shader
//
// Features (all inline — each WGSL module is a standalone file):
//   • ACES tonemapping + sRGB encode
//   • Shadow sampling with quality-scaled PCF (1 / 5-tap cross / 13-tap Poisson)
//   • Ambient sky lighting derived from sky_horizon / sky_zenith uniforms
//   • Atmospheric fog with directional sun-glow scattering
//   • Face variation: per-face random albedo micro-tint
//   • Triplanar grain: optional surface micro-detail (quality bit 0)
//
// GlobalUniform layout (must match renderer.rs / GlobalUniform):
//   view_proj        mat4   (bytes   0–63)
//   light_view_proj  mat4   (bytes  64–127)
//   camera_pos       vec4   xyz=cam_pos,  w=quality_bits   (bytes 128–143)
//   sun_dir          vec4   xyz=sun_dir,  w=fog_density    (bytes 144–159)
//   sky_horizon      vec4   xyz=horizon,  w=time_of_day    (bytes 160–175)
//   sky_zenith       vec4   xyz=zenith,   w=sun_intensity  (bytes 176–191)

struct Global {
    view_proj:       mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    camera_pos:      vec4<f32>,   // xyz=cam_pos, w=quality_bits
    sun_dir:         vec4<f32>,   // xyz=sun_dir, w=fog_density
    sky_horizon:     vec4<f32>,   // xyz=horizon sky color, w=time_of_day
    sky_zenith:      vec4<f32>,   // xyz=zenith  sky color, w=sun_intensity
}

@group(0) @binding(0) var<uniform> global: Global;
@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;

@group(2) @binding(0) var t_albedo:    texture_2d_array<f32>;
@group(2) @binding(1) var t_normal:    texture_2d_array<f32>;
@group(2) @binding(2) var t_roughness: texture_2d_array<f32>;
@group(2) @binding(3) var s_material:  sampler;
@group(2) @binding(4) var<storage, read> material_colors: array<vec4<f32>>;

const MATERIAL_INDEX_MASK:   u32 = 0x0000FFFFu;
const VERTEX_COLOR_ONLY: u32 = 0xFFFFu;

// Quality bit layout (packed into camera_pos.w as f32):
//   bit 0      : triplanar grain enabled
//   bits 1–2   : PCF level  0=Low(1 tap)  1=Medium(5-tap)  2=High(13-tap Poisson)
//   bit 3      : color-only debug mode

struct VertexOut {
    @builtin(position)              clip_pos:         vec4<f32>,
    @location(0)                    uv:               vec2<f32>,
    @location(1)                    world_normal:     vec3<f32>,
    @location(2)                    world_pos:        vec3<f32>,
    @location(3)                    view_pos:         vec3<f32>,
    @location(4)                    shadow_pos:       vec3<f32>,
    @location(5)                    color:            vec3<f32>,
    @location(6) @interpolate(flat) packed_tex_index: u32,
}

// ── ACES filmic tonemapping ───────────────────────────────────────────────────
fn vv_tonemap_aces(c: vec3<f32>) -> vec3<f32> {
    // Fitted ACES (Narkowicz 2015)
    let a = 2.51; let b = 0.03; let c2 = 2.43; let d = 0.59; let e = 0.14;
    return clamp((c * (a * c + b)) / (c * (c2 * c + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// ── sRGB gamma encode ─────────────────────────────────────────────────────────
fn vv_srgb(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
}

// ── Shadow PCF ────────────────────────────────────────────────────────────────
// Level 0 = single sample, level 1 = 5-tap cross, level 2 = 13-tap Poisson disc.
fn vv_shadow(shadow_pos: vec3<f32>, ndotl: f32, pcf_level: u32) -> f32 {
    // Fragments outside the shadow map frustum are unshadowed.
    if shadow_pos.z > 1.0 ||
       shadow_pos.x < 0.0 || shadow_pos.x > 1.0 ||
       shadow_pos.y < 0.0 || shadow_pos.y > 1.0 {
        return 1.0;
    }
    let bias = max(0.0004 * (1.0 - ndotl), 0.00008);
    let uv   = shadow_pos.xy;
    let z    = shadow_pos.z - bias;

    if pcf_level == 0u {
        return textureSampleCompare(t_shadow, s_shadow, uv, z);
    }

    let ts = 1.5 / vec2<f32>(textureDimensions(t_shadow)); // 1.5-texel kernel step

    if pcf_level == 1u {
        // 5-tap cross (+centre)
        var s  = textureSampleCompare(t_shadow, s_shadow, uv,                     z);
        s     += textureSampleCompare(t_shadow, s_shadow, uv + vec2<f32>( ts.x,  0.0), z);
        s     += textureSampleCompare(t_shadow, s_shadow, uv + vec2<f32>(-ts.x,  0.0), z);
        s     += textureSampleCompare(t_shadow, s_shadow, uv + vec2<f32>( 0.0,  ts.y), z);
        s     += textureSampleCompare(t_shadow, s_shadow, uv + vec2<f32>( 0.0, -ts.y), z);
        return s * 0.2;
    }

    // 13-tap Poisson disc (high quality)
    var s  = textureSampleCompare(t_shadow, s_shadow, uv, z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>(-0.94,  0.34), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>( 0.94,  0.34), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>(-0.34,  0.94), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>( 0.34, -0.94), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>(-1.82,  0.60), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>( 1.82,  0.60), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>(-0.60,  1.82), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>( 0.60, -1.82), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>(-2.50,  0.00), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>( 2.50,  0.00), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>( 0.00,  2.50), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>( 0.00, -2.50), z);
    return s / 13.0;
}

// ── Triplanar grain noise ─────────────────────────────────────────────────────
// Cheap organic micro-surface detail: three sin() evaluations blended by normal.
fn vv_triplanar_grain(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let f   = 5.0;
    let gx  = sin(world_pos.y * f + 0.30) * sin(world_pos.z * f + 0.70);
    let gy  = sin(world_pos.x * f + 1.10) * sin(world_pos.z * f + 0.20);
    let gz  = sin(world_pos.x * f + 0.80) * sin(world_pos.y * f + 0.50);
    let w   = abs(normal);
    return (gx * w.x + gy * w.y + gz * w.z) * 0.032;
}

// ── Per-face random tint ──────────────────────────────────────────────────────
// Breaks the monotony of tiled textures with a subtle per-block variation.
fn vv_face_variation(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let seed = dot(floor(world_pos + normal * 0.5), vec3<f32>(12.9898, 78.233, 37.719));
    return fract(sin(seed) * 43758.5453);
}

// ── Atmospheric fog with sun-glow ─────────────────────────────────────────────
// Fog transitions to sky horizon color; when looking toward the sun the fog
// picks up warm scattered light, giving cinematic depth to distant vistas.
fn vv_atmospheric_fog(color: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let dist        = distance(global.camera_pos.xyz, world_pos);
    let fog_density = global.sun_dir.w;
    let fog_factor  = 1.0 - exp(-(dist * fog_density) * (dist * fog_density * 0.5));

    // View direction from camera toward the fragment
    let view_dir  = normalize(world_pos - global.camera_pos.xyz);
    let sun_dir   = normalize(global.sun_dir.xyz);

    // Sun glow: forward scattering makes the fog warmer near the sun
    let sun_align = max(dot(view_dir, sun_dir), 0.0);
    let sun_glow  = pow(sun_align, 5.0) * 0.40;

    let fog_base  = global.sky_horizon.rgb;
    let fog_warm  = vec3<f32>(1.12, 0.82, 0.48); // warm sun-scatter color
    let fog_color = mix(fog_base, fog_warm, sun_glow);

    return mix(color, fog_color, clamp(fog_factor, 0.0, 1.0));
}

// ── Fragment entry point ──────────────────────────────────────────────────────
@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let layer       = in.packed_tex_index & MATERIAL_INDEX_MASK;
    let qbits       = u32(global.camera_pos.w);
    let color_only  = (qbits & 8u) != 0u;
    let triplanar   = (qbits & 1u) != 0u;
    let pcf_level   = (qbits >> 1u) & 3u;

    // ── Albedo & roughness ────────────────────────────────────────────────
    var albedo:    vec3<f32>;
    var roughness: f32;

    if layer == VERTEX_COLOR_ONLY {
        albedo    = in.color;
        roughness = 0.78;
    } else if color_only {
        albedo    = material_colors[layer].rgb * in.color;
        roughness = 0.72;
    } else {
        albedo    = textureSample(t_albedo,    s_material, in.uv, i32(layer)).rgb * in.color;
        roughness = textureSample(t_roughness, s_material, in.uv, i32(layer)).r;

        // Per-face micro-tint breaks texture monotony (cheap: one sin + fract)
        let variation = vv_face_variation(in.world_pos, in.world_normal);
        albedo *= (0.91 + variation * 0.18);

        // Optional triplanar grain (toggled by quality tier)
        if triplanar {
            let grain = vv_triplanar_grain(in.world_pos, in.world_normal);
            albedo    = albedo * (1.0 + grain);
        }
    }

    let normal      = normalize(in.world_normal);
    let sun_dir     = normalize(global.sun_dir.xyz);
    let sun_intensity = global.sky_zenith.w;

    // ── Sun direct lighting ───────────────────────────────────────────────
    // Wrap factor 0.82 + 0.18 gives gentle wrap-around light on shadowed faces.
    let ndotl       = max(dot(normal, sun_dir) * 0.82 + 0.18, 0.0);
    let shadow      = mix(0.28, 1.0, vv_shadow(in.shadow_pos, ndotl, pcf_level));

    // Warm gold sun color, slightly desaturated on rough surfaces.
    let sun_color   = vec3<f32>(1.30, 1.18, 0.90) * sun_intensity;
    let sun         = sun_color * ndotl * shadow * mix(1.12, 0.60, roughness);

    // ── Ambient sky lighting ──────────────────────────────────────────────
    // Top-facing surfaces receive zenith sky color; side faces receive horizon.
    // Using the planet's radial "up" (world_pos normalized) for realism on a
    // curved planet surface.
    let planet_up   = normalize(in.world_pos);
    let sky_up      = max(dot(normal, planet_up), 0.0);      // 0–1, top faces
    let sky_side    = clamp(1.0 - sky_up * 1.4, 0.0, 1.0);  // side/bottom faces

    let amb_zenith  = global.sky_zenith.rgb  * 0.38 * mix(0.90, 1.35, 1.0 - roughness);
    let amb_horizon = global.sky_horizon.rgb * 0.16;
    let amb_bounce  = vec3<f32>(0.10, 0.08, 0.05); // warm ground-reflected light

    let ambient     = amb_zenith * sky_up + amb_horizon * sky_side + amb_bounce * (1.0 - sky_up);

    // ── Compose & post-process ────────────────────────────────────────────
    var color = albedo * (sun + ambient);
    color     = vv_atmospheric_fog(color, in.world_pos);
    color     = vv_srgb(vv_tonemap_aces(color));

    return vec4<f32>(color, 1.0);
}

