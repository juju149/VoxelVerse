#include "include/camera/globals.wgsl"

// Procedural screen-space precipitation overlay.
//
// Drawn once per frame between the fog pass and post-process. Stays cheap
// (no instance buffers, no depth read) by generating streaks/flakes in screen
// space from `weather_params`. Phase 3.B of the weather/cosmos roadmap.
//
// kind values (see `vv_precip_kind`):
//   1 = rain      — vertical streaks, slight wind shear
//   2 = snow      — soft flakes, low velocity, large wind drift
//   3 = sleet     — rendered as fast rain
//   anything else — skipped (sand/ash/toxic_mist get dedicated passes in 3.B+)

struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn hash21(p: vec2<f32>) -> f32 {
    var x = fract(p * vec2<f32>(123.34, 456.21));
    x = x + dot(x, x + 34.345);
    return fract(x.x * x.y);
}

// Rain streak intensity at uv with given speed and tilt. Cells advance
// downward (with wind tilt) over time. Each cell shows a thin vertical
// line whose alpha falls off near the cell boundary.
fn rain_layer(uv: vec2<f32>, cell: vec2<f32>, speed: f32, tilt: f32, time: f32) -> f32 {
    let aspect = vec2<f32>(cell.x, cell.y);
    // Tilt by shifting x with y to give an angled streak.
    let q = vec2<f32>(uv.x + uv.y * tilt, uv.y + time * speed) * aspect;
    let cell_id = floor(q);
    let local = fract(q);
    let jitter = hash21(cell_id);
    // Streak position inside the cell.
    let center_x = 0.20 + 0.60 * jitter;
    let dx = abs(local.x - center_x);
    // Sharp horizontal mask, smooth vertical falloff.
    let streak = smoothstep(0.04, 0.0, dx) * (1.0 - local.y);
    // Spawn probability per cell.
    let alive = step(0.35, jitter);
    return streak * alive;
}

fn snow_layer(uv: vec2<f32>, cell: vec2<f32>, speed: f32, drift: f32, time: f32) -> f32 {
    let aspect = vec2<f32>(cell.x, cell.y);
    // Snow drifts more horizontally and falls slower.
    let q = vec2<f32>(
        uv.x + sin((uv.y + time * 0.2) * 6.0) * drift,
        uv.y + time * speed,
    ) * aspect;
    let cell_id = floor(q);
    let local = fract(q) - vec2<f32>(0.5);
    let jitter = hash21(cell_id);
    let radius = 0.18 + 0.10 * jitter;
    let d = length(local);
    let flake = smoothstep(radius, radius - 0.12, d);
    let alive = step(0.45, jitter);
    return flake * alive;
}

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let intensity = vv_precip_intensity();
    if intensity <= 0.001 {
        return vec4<f32>(0.0);
    }

    let kind = vv_precip_kind();
    let time = vv_time_seconds();
    let wind = vv_wind_dir_xz();

    // Slight viewport-aspect correction: keep cells visually square.
    let viewport = vv_viewport_size();
    let aspect = viewport.x / max(viewport.y, 1.0);
    let uv = vec2<f32>(in.uv.x * aspect, in.uv.y);

    var density = 0.0;
    var tint = vec3<f32>(0.0);
    var alpha_cap = 0.0;

    if kind == 1u || kind == 3u {
        // Rain (and fast sleet). Two layers @ different speeds give parallax.
        // Tilt scales with wind X component.
        let tilt = -wind.x * 0.25;
        let speed_a = select(1.6, 2.4, kind == 3u);
        let speed_b = select(2.4, 3.4, kind == 3u);
        let d0 = rain_layer(uv, vec2<f32>(40.0, 120.0), speed_a, tilt, time);
        let d1 = rain_layer(uv, vec2<f32>(60.0, 180.0), speed_b, tilt * 0.6, time + 13.7);
        density = max(d0, d1 * 0.7);
        tint = mix(vec3<f32>(0.65, 0.70, 0.78), vec3<f32>(0.78, 0.82, 0.90), density);
        alpha_cap = 0.55;
    } else if kind == 2u {
        // Snow. Two flake sizes.
        let drift = 0.02 + abs(wind.x) * 0.03;
        let d0 = snow_layer(uv, vec2<f32>(28.0, 28.0), 0.18, drift, time);
        let d1 = snow_layer(uv, vec2<f32>(48.0, 48.0), 0.28, drift * 0.6, time + 7.3);
        density = max(d0, d1 * 0.65);
        tint = vec3<f32>(0.95, 0.97, 1.0);
        alpha_cap = 0.85;
    } else {
        return vec4<f32>(0.0);
    }

    let alpha = clamp(density * intensity, 0.0, alpha_cap);
    return vec4<f32>(tint, alpha);
}
