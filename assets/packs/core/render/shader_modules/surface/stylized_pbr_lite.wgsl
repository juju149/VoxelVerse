// VoxelVerse — Stylized PBR-Lite terrain fragment shader
//
// Cinematic HDR look: AgX tonemapping, hemisphere ambient, Blinn-Phong
// specular, height-based aerial perspective fog, face variation and
// optional triplanar grain. All features are self-contained (no includes).
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

const MATERIAL_INDEX_MASK: u32 = 0x0000FFFFu;
const VERTEX_COLOR_ONLY:   u32 = 0xFFFFu;

// Quality bit layout (packed into camera_pos.w as f32):
//   bit 0      : triplanar grain enabled
//   bits 1–2   : PCF level  0=Low(1 tap)  1=Medium(5-tap cross)  2=High(13-tap Poisson)
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

// ── AgX Tonemapping ───────────────────────────────────────────────────────────
// Based on Troy Sobotka's AgX. Better hue stability than ACES, especially
// for greens and saturated colors. Does not include gamma encoding.
fn agx_contrast(x: vec3<f32>) -> vec3<f32> {
    let x2 = x * x;
    let x4 = x2 * x2;
    return  15.5      * x4 * x2
          - 40.14     * x4 * x
          + 31.96     * x4
          -  6.868    * x2 * x
          +  0.4298   * x2
          +  0.1191   * x
          -  0.00232;
}

fn vv_tonemap_agx(c: vec3<f32>, exposure: f32) -> vec3<f32> {
    // Input transform: rotate into AgX log space
    let mat = mat3x3<f32>(
        vec3<f32>(0.842479, 0.042328, 0.042376),
        vec3<f32>(0.078434, 0.878469, 0.079166),
        vec3<f32>(0.079224, 0.079116, 0.879143),
    );
    let min_ev = -12.47393;
    let max_ev =   4.026069;
    var v = mat * max(c * exposure, vec3<f32>(1e-10));
    v = clamp((log2(v) - min_ev) / (max_ev - min_ev), vec3<f32>(0.0), vec3<f32>(1.0));
    return agx_contrast(v);
}

// ── Proper sRGB gamma encode (IEC 61966-2-1) ─────────────────────────────────
fn vv_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) * 1.055 - vec3<f32>(0.055);
    return select(lo, hi, c > vec3<f32>(0.0031308));
}

// ── Shadow PCF ────────────────────────────────────────────────────────────────
// Level 0 = single sample, level 1 = 5-tap cross, level 2 = 13-tap Poisson disc.
fn vv_shadow(shadow_pos: vec3<f32>, ndotl: f32, pcf_level: u32) -> f32 {
    if shadow_pos.z > 1.0 ||
       shadow_pos.x < 0.0 || shadow_pos.x > 1.0 ||
       shadow_pos.y < 0.0 || shadow_pos.y > 1.0 {
        return 1.0;
    }
    let bias = max(0.00035 * (1.0 - ndotl), 0.00006);
    let uv   = shadow_pos.xy;
    let z    = shadow_pos.z - bias;

    if pcf_level == 0u {
        return textureSampleCompare(t_shadow, s_shadow, uv, z);
    }

    let ts = 1.5 / vec2<f32>(textureDimensions(t_shadow));

    if pcf_level == 1u {
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
fn vv_triplanar_grain(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let f  = 5.0;
    let gx = sin(world_pos.y * f + 0.30) * sin(world_pos.z * f + 0.70);
    let gy = sin(world_pos.x * f + 1.10) * sin(world_pos.z * f + 0.20);
    let gz = sin(world_pos.x * f + 0.80) * sin(world_pos.y * f + 0.50);
    let w  = abs(normal);
    return (gx * w.x + gy * w.y + gz * w.z) * 0.028;
}

// ── Per-face micro-tint ───────────────────────────────────────────────────────
fn vv_face_variation(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let seed = dot(floor(world_pos + normal * 0.5), vec3<f32>(12.9898, 78.233, 37.719));
    return fract(sin(seed) * 43758.5453);
}

// ── Aerial perspective fog ────────────────────────────────────────────────────
// Exponential-squared base fog with height fade, sun forward-scatter glow,
// and sky-color tinting for realistic atmospheric depth.
fn vv_aerial_fog(color: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let cam_pos     = global.camera_pos.xyz;
    let dist        = distance(cam_pos, world_pos);
    let fog_density = global.sun_dir.w;

    // Height-based density: denser near the planet surface.
    // Approximate height above surface using planet-center assumption (origin = center).
    let frag_r  = length(world_pos);
    let cam_r   = length(cam_pos);
    let avg_r   = mix(frag_r, cam_r, 0.5);
    let surface = max(avg_r - 1.0, 0.0) * 0.008;   // soften with altitude
    let dens    = fog_density * (1.0 + exp(-surface) * 0.6);

    let fog_sq  = dist * dens;
    let fog_f   = clamp(1.0 - exp(-fog_sq * fog_sq * 0.5), 0.0, 1.0);

    // Forward-scatter: fog warms near the sun (Mie lobe)
    let view_dir  = normalize(world_pos - cam_pos);
    let sun_dir   = normalize(global.sun_dir.xyz);
    let sun_align = max(dot(view_dir, sun_dir), 0.0);
    let mie_fwd   = pow(sun_align, 7.0) * 0.55 * global.sky_zenith.w;

    let fog_sky   = global.sky_horizon.rgb;
    let fog_sun   = vec3<f32>(1.05, 0.72, 0.32) * global.sky_zenith.w;
    let fog_col   = mix(fog_sky, fog_sun, mie_fwd);

    return mix(color, fog_col, fog_f);
}

// ── Fragment entry ────────────────────────────────────────────────────────────
@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let layer      = in.packed_tex_index & MATERIAL_INDEX_MASK;
    let qbits      = u32(global.camera_pos.w);
    let color_only = (qbits & 8u) != 0u;
    let triplanar  = (qbits & 1u) != 0u;
    let pcf_level  = (qbits >> 1u) & 3u;

    // ── Albedo & roughness ────────────────────────────────────────────────
    var albedo:    vec3<f32>;
    var roughness: f32;

    if layer == VERTEX_COLOR_ONLY {
        albedo    = in.color;
        roughness = 0.75;
    } else if color_only {
        albedo    = material_colors[layer].rgb * in.color;
        roughness = 0.70;
    } else {
        albedo    = textureSample(t_albedo,    s_material, in.uv, i32(layer)).rgb * in.color;
        roughness = textureSample(t_roughness, s_material, in.uv, i32(layer)).r;

        // Per-face micro-tint: subtle variation breaks texture repetition
        let var_v  = vv_face_variation(in.world_pos, in.world_normal);
        albedo    *= (0.88 + var_v * 0.24);

        // Optional triplanar grain (medium+ quality)
        if triplanar {
            albedo *= 1.0 + vv_triplanar_grain(in.world_pos, in.world_normal);
        }
    }

    let normal        = normalize(in.world_normal);
    let sun_dir       = normalize(global.sun_dir.xyz);
    let sun_intensity = global.sky_zenith.w;

    // ── Direct sun lighting ───────────────────────────────────────────────
    // Wrap factor 0.85+0.15: gentle wrap-around on back-lit faces
    let ndotl    = max(dot(normal, sun_dir) * 0.85 + 0.15, 0.0);
    let shadow   = mix(0.22, 1.0, vv_shadow(in.shadow_pos, ndotl, pcf_level));

    // Warm gold sun: color driven by sun elevation (orange at horizon)
    let sun_elev = sun_dir.y;
    let sun_col  = mix(
        vec3<f32>(1.10, 0.60, 0.22),    // horizon: warm orange
        vec3<f32>(1.28, 1.18, 0.92),    // zenith:  bright warm white
        clamp(sun_elev * 2.5 + 0.3, 0.0, 1.0)
    ) * sun_intensity;

    let direct = sun_col * ndotl * shadow;

    // ── Specular highlight (Blinn-Phong, medium+ quality) ─────────────────
    var specular = vec3<f32>(0.0);
    if pcf_level > 0u {
        let view_dir = normalize(global.camera_pos.xyz - in.world_pos);
        let half_v   = normalize(sun_dir + view_dir);
        let ndoth    = max(dot(normal, half_v), 0.0);
        let gloss    = max(32.0 * (1.0 - roughness * roughness), 1.0);
        let spec_val = pow(ndoth, gloss) * (1.0 - roughness) * 0.35;
        specular = sun_col * spec_val * shadow;
    }

    // ── Hemisphere ambient sky lighting ──────────────────────────────────
    // Uses the planet's radial "up" so curved surfaces catch sky correctly.
    let planet_up = normalize(in.world_pos);
    let sky_up    = max(dot(normal, planet_up), 0.0);          // top faces
    let sky_side  = clamp(1.0 - sky_up * 1.4, 0.0, 1.0);      // side faces
    let sky_bot   = max(dot(normal, -planet_up), 0.0);         // bottom faces

    let amb_sky   = global.sky_zenith.rgb  * 0.44 * mix(0.80, 1.40, sky_up);
    let amb_horiz = global.sky_horizon.rgb * 0.18;
    let amb_fill  = vec3<f32>(0.12, 0.09, 0.05);               // warm ground bounce

    let ambient = amb_sky * sky_up + amb_horiz * sky_side + amb_fill * sky_bot;

    // ── Compose linear HDR color ──────────────────────────────────────────
    var color = albedo * (direct + ambient + specular);

    // ── Aerial perspective fog ────────────────────────────────────────────
    color = vv_aerial_fog(color, in.world_pos);

    // ── AgX tonemap + exposure + sRGB encode ─────────────────────────────
    // Exposure: slightly brighter than 1.0 gives a clean, sunlit look.
    let exposure = 1.15;
    color = vv_srgb(vv_tonemap_agx(color, exposure));

    return vec4<f32>(color, 1.0);
}

