use super::config::TreeCrownShape;

use super::blueprint::{TreeBlueprintBuilder, TreeVoxelKind};
use super::config::TreeArchetype;
use super::units::ellipsoid_score;

pub(crate) fn generate_crown(builder: &mut TreeBlueprintBuilder) {
    match builder.config.crown.shape {
        TreeCrownShape::Cone => generate_cone(builder),
        TreeCrownShape::Layered => generate_layered(builder),
        TreeCrownShape::Columnar => generate_columnar(builder),
        TreeCrownShape::Palm => generate_palm(builder),
        TreeCrownShape::Ellipsoid | TreeCrownShape::LobedEllipsoid => generate_lobed(builder),
    }
}

fn generate_lobed(builder: &mut TreeBlueprintBuilder) {
    let count = builder.rng.range_u32(
        builder.config.crown.lobe_count_min,
        builder.config.crown.lobe_count_max,
    );

    place_ellipsoid(
        builder,
        builder.crown_center,
        builder.crown_radius_layers as f32,
        builder.crown_radius_layers as f32 * 0.92,
        builder.crown_height_layers as f32,
        1.0,
    );

    for _ in 0..count {
        let dir = builder.rng.direction8();
        let distance = builder
            .rng
            .range_f32(0.25, builder.config.crown.lobe_spread_m)
            * builder.crown_radius_layers as f32;

        let layer_offset =
            ((builder.rng.next_f32() - 0.45) * builder.crown_height_layers as f32).round() as i32;

        let center = (
            builder.crown_center.0 + (dir.0 as f32 * distance).round() as i32,
            builder.crown_center.1 + (dir.1 as f32 * distance).round() as i32,
            (builder.crown_center.2 as i32 + layer_offset).max(1) as u32,
        );

        let scale = match builder.archetype {
            TreeArchetype::Spreading => 0.78,
            TreeArchetype::Columnar => 0.48,
            TreeArchetype::Layered => 0.68,
            TreeArchetype::Irregular => builder.rng.range_f32(0.45, 0.9),
            TreeArchetype::Round => 0.65,
        };
        let radius_u = builder.crown_radius_layers as f32 * scale;
        let radius_v = builder.crown_radius_layers as f32 * builder.rng.range_f32(0.48, 0.82);
        let radius_y = builder.crown_height_layers as f32 * builder.rng.range_f32(0.42, 0.75);

        place_ellipsoid(builder, center, radius_u, radius_v, radius_y, 0.95);
    }
}

fn generate_cone(builder: &mut TreeBlueprintBuilder) {
    let height = builder.crown_height_layers.max(2);

    for y in 0..=height {
        let t = y as f32 / height as f32;
        let radius = builder.crown_radius_layers as f32 * (1.0 - t).powf(0.75);
        let layer = builder.crown_start_layer + y;

        place_flat_disk(
            builder,
            builder.crown_center.0,
            builder.crown_center.1,
            layer,
            radius,
        );
    }
}

fn generate_layered(builder: &mut TreeBlueprintBuilder) {
    let layers = 4.max(builder.crown_height_layers / 2);

    for i in 0..layers {
        let t = i as f32 / layers as f32;
        let layer = builder.crown_start_layer + i * 2;
        let radius = builder.crown_radius_layers as f32 * (1.0 - t * 0.28);

        place_flat_disk(
            builder,
            builder.crown_center.0,
            builder.crown_center.1,
            layer,
            radius,
        );
    }
}

fn generate_columnar(builder: &mut TreeBlueprintBuilder) {
    place_ellipsoid(
        builder,
        builder.crown_center,
        builder.crown_radius_layers as f32 * 0.55,
        builder.crown_radius_layers as f32 * 0.55,
        builder.crown_height_layers as f32 * 1.45,
        1.0,
    );
}

fn generate_palm(builder: &mut TreeBlueprintBuilder) {
    let center = builder.crown_center;

    for _ in 0..8 {
        let dir = builder.rng.direction8();
        let length = builder.crown_radius_layers.max(2);

        for step in 1..=length {
            let t = step as f32 / length as f32;
            let layer = (center.2 as f32 - t * 1.4).max(1.0) as u32;

            builder.place(
                center.0 + dir.0 * step,
                center.1 + dir.1 * step,
                layer,
                TreeVoxelKind::Leaf,
            );
        }
    }
}

fn place_ellipsoid(
    builder: &mut TreeBlueprintBuilder,
    center: (i32, i32, u32),
    radius_u: f32,
    radius_v: f32,
    radius_y: f32,
    threshold: f32,
) {
    let ru = radius_u.ceil() as i32;
    let rv = radius_v.ceil() as i32;
    let ry = radius_y.ceil() as i32;

    for du in -ru..=ru {
        for dv in -rv..=rv {
            for dy in -ry..=ry {
                let score = ellipsoid_score(
                    du as f32, dv as f32, dy as f32, radius_u, radius_v, radius_y,
                );

                if score > threshold {
                    continue;
                }

                if dy < 0 && score > 1.0 - builder.config.crown.bottom_trim {
                    continue;
                }

                let noise = builder.rng.next_f32();
                let hollow = builder.config.crown.hollow_core;
                let airiness = builder.config.variation.leaf_airiness;

                if score < hollow && noise > 0.18 {
                    continue;
                }

                let edge_noise = builder.config.crown.surface_noise * (score - 0.55).max(0.0);
                let required_density = builder.config.crown.density - edge_noise - airiness;

                if noise <= required_density {
                    builder.place(
                        center.0 + du,
                        center.1 + dv,
                        (center.2 as i32 + dy).max(1) as u32,
                        TreeVoxelKind::Leaf,
                    );
                }
            }
        }
    }
}

fn place_flat_disk(
    builder: &mut TreeBlueprintBuilder,
    center_u: i32,
    center_v: i32,
    layer: u32,
    radius: f32,
) {
    let r = radius.ceil() as i32;

    for du in -r..=r {
        for dv in -r..=r {
            let d = ((du * du + dv * dv) as f32).sqrt();
            if d <= radius && builder.rng.next_f32() < builder.config.crown.density {
                builder.place(center_u + du, center_v + dv, layer, TreeVoxelKind::Leaf);
            }
        }
    }
}
