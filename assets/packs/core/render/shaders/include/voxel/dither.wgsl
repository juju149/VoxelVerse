fn vv_dither_threshold(clip_pos: vec4<f32>) -> f32 {
    let p = floor(clip_pos.xy);
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}