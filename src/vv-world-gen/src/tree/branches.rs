use super::blueprint::{TreeBlueprintBuilder, TreeVoxelKind};
use super::units::meters_to_voxels;

pub(crate) fn generate_branches(builder: &mut TreeBlueprintBuilder) {
    if !builder.config.branches.enabled {
        return;
    }

    let count = builder.rng.range_u32(
        builder.config.branches.count_min,
        builder.config.branches.count_max,
    );

    for _ in 0..count {
        generate_branch(builder, false);
    }
}

fn generate_branch(builder: &mut TreeBlueprintBuilder, fork: bool) {
    let dir = builder.rng.direction8();

    let start_t = builder.rng.range_f32(
        builder.config.branches.start_min_t,
        builder.config.branches.start_max_t,
    );

    let start_layer = (builder.height_layers as f32 * start_t)
        .round()
        .clamp(2.0, builder.height_layers as f32) as u32;

    let length = meters_to_voxels(
        builder.rng.range_f32(
            builder.config.branches.length_min_m,
            builder.config.branches.length_max_m,
        ),
        builder.config.voxel_size_m,
    )
    .max(1);

    let upward = builder.config.branches.upward_tilt;
    let droop = builder.config.branches.droop;

    let (trunk_u, trunk_v, _) = builder.crown_center;

    let mut end = (trunk_u, trunk_v, start_layer);

    for step in 1..=length {
        let t = step as f32 / length as f32;

        let horizontal = step as i32;
        let vertical = (t * upward * length as f32 - t * t * droop * length as f32).round() as i32;

        let du = trunk_u + dir.0 * horizontal;
        let dv = trunk_v + dir.1 * horizontal;
        let layer = (start_layer as i32 + vertical).max(1) as u32;

        builder.place(du, dv, layer, TreeVoxelKind::Branch);
        end = (du, dv, layer);
    }

    place_leaf_lobe(builder, end);

    if !fork && builder.rng.chance(builder.config.branches.fork_chance) {
        generate_branch(builder, true);
    }
}

fn place_leaf_lobe(builder: &mut TreeBlueprintBuilder, center: (i32, i32, u32)) {
    let radius = meters_to_voxels(
        builder.config.branches.leaf_lobe_radius_m,
        builder.config.voxel_size_m,
    )
    .max(1) as f32;

    for du in -(radius as i32)..=(radius as i32) {
        for dv in -(radius as i32)..=(radius as i32) {
            for dy in -(radius as i32)..=(radius as i32) {
                let score = (du * du + dv * dv + dy * dy) as f32 / (radius * radius);
                if score <= 1.0 && builder.rng.next_f32() > 0.18 {
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
