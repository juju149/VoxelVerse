#include "include/camera/globals.wgsl"

// Phase 5.B of the weather/cosmos roadmap: stars + moon disc + aurora ribbon
// overlay. Drawn between the sky pass (which paints the base gradient + sun)
// and the clouds pass (which then occludes everything above the horizon).
//
// Inputs (from the global uniform):
//   - vv_stars_visibility()    drives star field brightness
//   - vv_moon_dir/radius()     positions and sizes the primary moon
//   - vv_aurora_intensity()    activates the polar ribbon
//   - vv_eclipse_factor()      lifts star visibility during totality
//
// No textures needed — everything is procedural so the pass survives the
// "data-driven content" rule (no extra .ktx2 in the pack for V1).

struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn hash21(p: vec2<f32>) -> f32 {
    var x = fract(p * vec2<f32>(123.34, 456.21));
    x = x + dot(x, x + 34.345);
    return fract(x.x * x.y);
}

fn hash22(p: vec2<f32>) -> vec2<f32> {
    let a = hash21(p);
    let b = hash21(p + vec2<f32>(1.7, 9.2));
    return vec2<f32>(a, b);
}

// Project a world-space direction into screen NDC. Returns (ndc, in_front)
// where in_front == 0 means the direction is behind the camera.
fn project_dir(dir: vec3<f32>) -> vec3<f32> {
    let world = global.camera_pos.xyz + dir * 9000.0;
    let clip = global.view_proj * vec4<f32>(world, 1.0);
    if clip.w <= 0.001 {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    let ndc = clip.xy / clip.w;
    return vec3<f32>(ndc, 1.0);
}

// Star field: hash-based grid, each cell may host a star with random
// position, brightness, and a slow twinkle.
fn stars_field(uv: vec2<f32>, time: f32) -> vec3<f32> {
    let viewport = vv_viewport_size();
    let aspect = viewport.x / max(viewport.y, 1.0);
    // Grid in NDC-ish coords with aspect correction. 100 cells across at 1080p
    // gives ~8000 candidate cells, of which only ~1/4 are populated → ≈ 2k
    // visible stars. Cheap and uniform.
    let cell_density = 110.0;
    let q = vec2<f32>(uv.x * aspect, uv.y) * cell_density;
    let cell_id = floor(q);
    let local = fract(q);

    let r0 = hash21(cell_id);
    // Only ~30 % of cells host a star, weighted by their hash → variety.
    if r0 < 0.70 {
        return vec3<f32>(0.0);
    }
    let centre = hash22(cell_id + vec2<f32>(0.13, 0.71));
    let d = distance(local, centre);
    let radius = 0.02 + 0.06 * r0;
    let disc = smoothstep(radius, radius * 0.3, d);

    // Twinkle: 0.4 .. 1.0 sinusoid offset by cell id.
    let twinkle = 0.7 + 0.3 * sin(time * 1.7 + r0 * 28.0);

    // Spectral tint via cell id: blueish, white, warm.
    let tint_a = vec3<f32>(0.78, 0.85, 1.0);
    let tint_b = vec3<f32>(1.0, 0.96, 0.84);
    let tint_c = vec3<f32>(1.0, 0.84, 0.66);
    let h = hash21(cell_id + vec2<f32>(7.1, 3.7));
    var tint = tint_a;
    if h > 0.66 { tint = tint_c; } else if h > 0.33 { tint = tint_b; }

    let mag = r0 * r0; // skew toward dim stars
    return tint * disc * twinkle * mag;
}

// Moon disc with a soft phase shade. `phase` here is a derived value from
// `sun_dir · moon_dir`: positive when the moon is on the far side of the
// sun (new), negative when on the same side (full from observer's frame).
fn moon_disc(uv: vec2<f32>, moon_dir: vec3<f32>, radius_rad: f32) -> vec3<f32> {
    if radius_rad <= 0.0 {
        return vec3<f32>(0.0);
    }
    let p = project_dir(moon_dir);
    if p.z <= 0.0 {
        return vec3<f32>(0.0);
    }
    let pixel_ndc = uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0);
    // Convert angular radius (radians) to a rough NDC radius using the
    // projection's vertical FOV approximation. We don't have FOV available
    // directly so we use a heuristic that matches the previously hand-tuned
    // values in sky.frag.wgsl: angular_radius_rad ≈ 0.005 → NDC ≈ 0.020.
    let ndc_radius = radius_rad * 4.0 + 0.005;
    let d = distance(pixel_ndc, p.xy);
    let disc = 1.0 - smoothstep(ndc_radius * 0.9, ndc_radius, d);
    if disc <= 0.0 {
        return vec3<f32>(0.0);
    }
    // Lambertian-ish phase shading. Sun direction tells us where the lit
    // side is; we shade by the dot product of the surface normal at the
    // pixel relative to moon centre.
    let local = (pixel_ndc - p.xy) / max(ndc_radius, 1e-4);
    let normal = vec3<f32>(local.x, -local.y, sqrt(max(0.0, 1.0 - dot(local, local))));
    let sun_dir = vv_sun_direction();
    // Project the sun direction roughly into the moon's local frame: this is
    // an approximation but cheap and visually convincing.
    let lit = clamp(dot(normalize(sun_dir), normalize(vec3<f32>(local.x, local.y, 1.0))), 0.0, 1.0);
    let shade = mix(0.06, 1.0, lit); // ambient floor so the moon is never pure black
    let moon_tint = vec3<f32>(0.92, 0.95, 1.05);
    return moon_tint * disc * shade;
}

// Aurora ribbon: vertical band biased to the upper half of the sky,
// shaped with a slow noise. Active when `vv_aurora_intensity() > 0`.
fn aurora_band(uv: vec2<f32>, time: f32) -> vec3<f32> {
    let intensity = vv_aurora_intensity();
    if intensity <= 0.001 {
        return vec3<f32>(0.0);
    }
    // Vertical mask: peak at roughly uv.y = 0.35 (upper third of screen),
    // fall off elsewhere.
    let height = 1.0 - uv.y;
    let height_mask = smoothstep(0.50, 0.85, height) * (1.0 - smoothstep(0.85, 1.00, height));
    if height_mask <= 0.0 {
        return vec3<f32>(0.0);
    }
    // Horizontal ripples: low-frequency sin sum with time drift.
    let h = sin(uv.x * 6.28 * 1.7 + time * 0.30)
          + 0.6 * sin(uv.x * 6.28 * 3.1 - time * 0.25)
          + 0.4 * sin(uv.x * 6.28 * 5.7 + time * 0.18);
    let ribbon = smoothstep(1.4, 2.4, h + 1.5);
    let col_a = vec3<f32>(0.10, 0.92, 0.55); // green
    let col_b = vec3<f32>(0.38, 0.42, 0.95); // violet
    let tint = mix(col_a, col_b, smoothstep(0.0, 1.0, uv.x));
    return tint * ribbon * height_mask * intensity;
}

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let time = vv_time_seconds();
    var color = vec3<f32>(0.0);

    // Stars: visible when sun is low (handled by stars_visibility) and
    // boosted by eclipse_factor so a daytime totality reveals them.
    let star_strength = clamp(vv_stars_visibility() + vv_eclipse_factor() * 0.8, 0.0, 1.0);
    color += stars_field(in.uv, time) * star_strength;

    // Moon disc.
    color += moon_disc(in.uv, vv_moon_dir(), vv_moon_angular_radius());

    // Aurora.
    color += aurora_band(in.uv, time);

    // Output additive over the sky pass.
    return vec4<f32>(color, 1.0);
}
