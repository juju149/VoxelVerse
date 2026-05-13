// VoxelVerse — Cinematic Atmospheric Sky
//
// Full cinematic sky: gradient, Rayleigh/Mie scattering, twilight purple band,
// sun disc with corona, moon disc, star field, and cirrus cloud layer.
// Called as a fullscreen triangle; vertex comes from post/final_composite.wgsl.
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
    camera_pos:      vec4<f32>,
    sun_dir:         vec4<f32>,
    sky_horizon:     vec4<f32>,
    sky_zenith:      vec4<f32>,
}

@group(0) @binding(0) var<uniform> global: Global;

struct SkyFragmentIn {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0)       uv:       vec2<f32>,
}

// ── Quintic smooth curve ──────────────────────────────────────────────────────
fn smooth5(t: f32) -> f32 {
    let c = clamp(t, 0.0, 1.0);
    return c * c * c * (c * (c * 6.0 - 15.0) + 10.0);
}

// ── Hash / noise utilities ────────────────────────────────────────────────────
fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn hash22(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453),
        fract(sin(dot(p, vec2<f32>(269.5, 183.3))) * 43758.5453),
    );
}

// Bilinear smooth noise on 2D grid
fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash21(i),                    hash21(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

// 2-octave FBM for cirrus clouds
fn fbm_clouds(p: vec2<f32>) -> f32 {
    return noise2d(p) * 0.60 + noise2d(p * 2.3 + vec2<f32>(4.1, 1.7)) * 0.30
         + noise2d(p * 5.1 + vec2<f32>(1.3, 7.2)) * 0.10;
}

// ── Star field ────────────────────────────────────────────────────────────────
// Screen-space hash stars. Offset by sun_dir to give the illusion of
// world-space attachment (acceptable for low-movement voxel game).
fn star_field(uv: vec2<f32>, sun_xz: vec2<f32>) -> f32 {
    let grid = 200.0;
    let suv  = uv + sun_xz * 0.4;           // tie to sun direction
    let cell = floor(suv * grid);
    let frac = fract(suv * grid);
    let star_pos   = hash22(cell);
    let brightness = hash21(cell * 9.1);
    let size       = 0.014 + brightness * 0.016;
    return smoothstep(size, 0.0, length(frac - star_pos)) * brightness;
}

// ── Proper sRGB encode (IEC 61966-2-1) ───────────────────────────────────────
fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) * 1.055 - vec3<f32>(0.055);
    return select(lo, hi, c > vec3<f32>(0.0031308));
}

// ── Fragment entry ────────────────────────────────────────────────────────────
@fragment
fn fs_main(in: SkyFragmentIn) -> @location(0) vec4<f32> {
    let sun_dir       = normalize(global.sun_dir.xyz);
    let sun_intensity = global.sky_zenith.w;
    let horizon_col   = global.sky_horizon.rgb;
    let zenith_col    = global.sky_zenith.rgb;
    let time_of_day   = global.sky_horizon.w;   // 0=midnight 0.5=noon

    // Sun elevation: y component (-1=underground, 0=horizon, 1=overhead)
    let sun_elev  = sun_dir.y;
    // Dawn/dusk factor: 1.0 at sunrise/sunset, 0 at noon and at night
    let dawn_t    = clamp(1.0 - abs(sun_elev) * 5.5, 0.0, 1.0)
                  * clamp(sun_elev * 6.0 + 0.8, 0.0, 1.0);
    // Night factor: 0 at day, 1 deep at night
    let night_t   = clamp((-sun_elev - 0.10) * 7.0, 0.0, 1.0);

    // UV: uv.y=0 → zenith, uv.y=1 → horizon/below
    let horiz_frac  = clamp(1.0 - in.uv.y, 0.0, 1.0);
    let zenith_t    = pow(horiz_frac, 0.55);

    // ── Base sky gradient ─────────────────────────────────────────────────
    var sky = mix(horizon_col, zenith_col, zenith_t);

    // ── Twilight purple-rose band (anti-solar side near horizon at dusk) ──
    let mid_band  = smooth5(clamp((horiz_frac - 0.15) * 3.0, 0.0, 1.0))
                  * (1.0 - smooth5(clamp((horiz_frac - 0.65) * 2.5, 0.0, 1.0)));
    let twilight  = vec3<f32>(0.50, 0.22, 0.55);
    sky = mix(sky, twilight, mid_band * dawn_t * 0.55);

    // ── Horizon haze: bright warm band just above horizon ─────────────────
    let haze_band = pow(clamp((in.uv.y - 0.72) * 5.2, 0.0, 1.0), 2.2) * 0.22;
    let haze_col  = mix(horizon_col * 1.20,
                        vec3<f32>(1.05, 0.78, 0.42),
                        dawn_t * 0.75);
    sky = mix(sky, haze_col, haze_band);

    // ── Pixel NDC for disc projections ────────────────────────────────────
    let pixel_ndc = in.uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0);

    // ── Sun disc + Mie corona ─────────────────────────────────────────────
    let sun_world = global.camera_pos.xyz + sun_dir * 9000.0;
    let sun_clip  = global.view_proj * vec4<f32>(sun_world, 1.0);

    if sun_clip.w > 0.001 && sun_clip.z >= 0.0 {
        let sun_ndc  = sun_clip.xy / sun_clip.w;
        let to_sun   = pixel_ndc - sun_ndc;
        let dist_sq  = dot(to_sun, to_sun);

        // Sun color: deep orange at horizon → warm white at zenith
        let sun_col = mix(
            vec3<f32>(1.05, 0.42, 0.08),    // horizon: deep orange
            vec3<f32>(1.25, 1.15, 0.92),    // zenith:  warm white
            clamp(sun_elev * 2.8 + 0.25, 0.0, 1.0)
        );

        // Wide Rayleigh + Mie atmospheric glow
        let mie_wide   = exp(-dist_sq * 0.55) * 0.72 * sun_intensity;
        let mie_tight  = exp(-dist_sq * 9.0)  * 0.80 * sun_intensity;
        // Hard disc with slight limb darkening
        let disc       = 1.0 - smoothstep(0.0006, 0.0028, dist_sq);

        sky = mix(sky, mix(horizon_col * 1.1, sun_col, 0.65), clamp(mie_wide, 0.0, 0.85));
        sky += sun_col * clamp(mie_tight, 0.0, 0.55);
        sky  = mix(sky, sun_col * 1.5, disc * sun_intensity);
    }

    // ── Moon disc ─────────────────────────────────────────────────────────
    // Simplified: moon orbits roughly opposite sun (offset slightly to avoid
    // perfect overlap at noon which would put both underground simultaneously).
    let moon_dir  = normalize(vec3<f32>(-sun_dir.x, -sun_dir.y + 0.08, -sun_dir.z));
    let moon_world = global.camera_pos.xyz + moon_dir * 9000.0;
    let moon_clip  = global.view_proj * vec4<f32>(moon_world, 1.0);
    let moon_vis   = night_t * clamp(1.0 - sun_intensity * 3.0, 0.0, 1.0);

    if moon_clip.w > 0.001 && moon_clip.z >= 0.0 && moon_vis > 0.01 {
        let moon_ndc  = moon_clip.xy / moon_clip.w;
        let to_moon   = pixel_ndc - moon_ndc;
        let moon_dsq  = dot(to_moon, to_moon);

        let moon_col   = vec3<f32>(0.82, 0.88, 1.00);  // cool silver-blue
        let moon_halo  = exp(-moon_dsq * 18.0) * 0.22;
        let moon_disc  = 1.0 - smoothstep(0.0004, 0.0018, moon_dsq);

        sky  = mix(sky, moon_col * 0.95, clamp(moon_halo, 0.0, 0.5) * moon_vis);
        sky  = mix(sky, moon_col, moon_disc * moon_vis);
    }

    // ── Star field (night only) ───────────────────────────────────────────
    if night_t > 0.02 {
        let stars_raw = star_field(in.uv, sun_dir.xz);
        // Stars appear above the horizon band only
        let star_mask = smooth5(horiz_frac * 1.6 - 0.15);
        sky += vec3<f32>(0.93, 0.95, 1.00) * stars_raw * star_mask * night_t * 1.8;
    }

    // ── Cirrus clouds (medium+ quality) ──────────────────────────────────
    // Quality bit 1-2 = PCF level (0=Low, 1=Med, 2=High)
    let pcf_level = (u32(global.camera_pos.w) >> 1u) & 3u;
    if pcf_level > 0u {
        let scroll    = time_of_day * 0.25;
        let cloud_uv  = in.uv * vec2<f32>(5.0, 2.5) + vec2<f32>(scroll, 0.0);
        let density   = fbm_clouds(cloud_uv);
        let height_m  = smooth5(clamp((horiz_frac - 0.22) * 2.8, 0.0, 1.0));
        let alpha     = clamp(density * 1.6 - 0.62, 0.0, 1.0) * height_m;

        // Cloud color: white in day, warm rose at dawn, dark blue at night
        let cloud_day  = vec3<f32>(0.97, 0.97, 1.00);
        let cloud_dawn = vec3<f32>(0.98, 0.62, 0.38);
        let cloud_night = vec3<f32>(0.06, 0.07, 0.14);
        let day_fac    = clamp(sun_elev * 4.0, 0.0, 1.0);
        let cloud_col  = mix(cloud_night, mix(cloud_dawn, cloud_day, day_fac),
                             clamp(sun_elev * 5.0 + 0.3, 0.0, 1.0));

        sky = mix(sky, cloud_col, alpha * 0.52);
    }

    // ── sRGB encode (IEC 61966-2-1) ───────────────────────────────────────
    return vec4<f32>(linear_to_srgb(max(sky, vec3<f32>(0.0))), 1.0);
}

