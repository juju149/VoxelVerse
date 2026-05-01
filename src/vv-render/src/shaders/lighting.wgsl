fn apply_fog(lit: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let dist = distance(global.camera_pos.xyz, world_pos);

    let fog_density = global.atmosphere.fog_color_density.w;
    let fog_base = global.atmosphere.fog_color_density.xyz;
    let fog_start = max(global.atmosphere.sky_params.x, 0.0);

    let fog_range = max(dist - fog_start, 0.0);
    let fog_factor = 1.0 - exp(-(fog_range * fog_density) * (fog_range * fog_density * 0.45));

    return mix(lit, fog_base, saturate(fog_factor));
}

fn shade_voxel_surface(
    in: VertexOut,
    visual: BlockVisual,
    albedo: vec3<f32>,
    alpha: f32,
) -> vec3<f32> {
    let N = safe_normalize(in.world_normal);
    let L = safe_normalize(global.atmosphere.sun_direction.xyz);
    let V = safe_normalize(global.camera_pos.xyz - in.world_pos);
    let R = reflect(-L, N);

    let radial_up = safe_normalize(in.world_pos);

    let ao_direct = mix(1.0, in.ao, visual.variation_b.w);

    // Phase 1 tweak:
    // Less aggressive than ao^2, keeps shadows readable.
    let ao_ambient = mix(ao_direct, ao_direct * ao_direct, 0.5);

    let n_dot_l = saturate(dot(N, L));
    let n_dot_v = saturate(dot(N, V));
    let shadow = shadow_visibility(in.shadow_pos, n_dot_l);

    let hemi = dot(N, radial_up) * 0.5 + 0.5;

    let ambient_color = mix(
        global.atmosphere.ground_ambient_color.xyz,
        global.atmosphere.sky_color.xyz,
        hemi,
    );

    // Slightly stronger ambient for stylized readability.
    let ambient_strength = 0.72 + visual.surface.x * 0.20;
    let ambient = ambient_color * ambient_strength * ao_ambient;

    let wrapped_light = mix(
        n_dot_l,
        saturate((n_dot_l + 0.25) / 1.25),
        0.24,
    );

    let diffuse = global.atmosphere.sun_color.xyz * shadow * wrapped_light;

    let shadow_fill = global.atmosphere.shadow_tint_color.xyz
        * (1.0 - shadow)
        * (0.26 + n_dot_l * 0.42);

    // Stronger cinematic rim, especially on backlit silhouettes.
    let back_lighting = saturate(-dot(N, L));
    let rim = global.atmosphere.sky_color.xyz
        * pow(1.0 - n_dot_v, 2.5)
        * mix(0.08, 0.20, back_lighting);

    let gloss = pow(1.0 - visual.surface.x, 2.0);
    let specular = global.atmosphere.sun_color.xyz
        * shadow
        * pow(saturate(dot(R, V)), mix(8.0, 64.0, gloss))
        * gloss
        * 0.10;

    var lit = albedo * (diffuse * ao_direct + ambient + shadow_fill + rim)
        + specular
        + visual.emission.rgb;

    lit = apply_fog(lit, in.world_pos);

    return lit;
}