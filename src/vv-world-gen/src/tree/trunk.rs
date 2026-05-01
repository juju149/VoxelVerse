use super::blueprint::{TreeBlueprintBuilder, TreeVoxelKind};
use super::units::{meters_to_voxels, radius_at_height};

pub(crate) fn generate_trunk(builder: &mut TreeBlueprintBuilder) {
    let base_radius_layers = meters_to_voxels(
        builder.config.trunk.base_radius_m,
        builder.config.voxel_size_m,
    )
    .max(1);

    let top_radius_layers = meters_to_voxels(
        builder.config.trunk.top_radius_m,
        builder.config.voxel_size_m,
    )
    .max(1);

    let flare_radius_layers = meters_to_voxels(
        builder.config.trunk.flare_radius_m,
        builder.config.voxel_size_m,
    )
    .max(1);

    let flare_height_layers = meters_to_voxels(
        builder.config.trunk.flare_height_m,
        builder.config.voxel_size_m,
    )
    .max(1);

    let lean_dir = builder.rng.direction8();
    let lean_layers =
        meters_to_voxels(builder.config.trunk.lean_max_m, builder.config.voxel_size_m) as i32;

    let bend_dir = builder.rng.direction8();
    let bend_layers = meters_to_voxels(
        builder.config.trunk.bend_strength_m,
        builder.config.voxel_size_m,
    ) as i32;

    let mut top_offset = (0, 0);

    for layer in 1..=builder.height_layers {
        let t = layer as f32 / builder.height_layers as f32;

        let lean = (t * lean_layers as f32).round() as i32;
        let bend_wave = (t * std::f32::consts::PI * builder.config.trunk.bend_frequency)
            .sin()
            .max(0.0);
        let bend = (bend_wave * bend_layers as f32).round() as i32;

        let du = lean_dir.0 * lean + bend_dir.0 * bend;
        let dv = lean_dir.1 * lean + bend_dir.1 * bend;

        top_offset = (du, dv);

        let radius = radius_at_height(
            base_radius_layers as f32,
            top_radius_layers as f32,
            t,
            builder.config.trunk.taper,
        );

        let radius = if layer <= flare_height_layers {
            radius.max(flare_radius_layers as f32 * (1.0 - t * 0.35))
        } else {
            radius
        };

        place_disk(builder, du, dv, layer, radius, TreeVoxelKind::Log);
    }

    builder.set_crown_center(
        top_offset.0,
        top_offset.1,
        builder.crown_start_layer + builder.crown_height_layers / 2,
    );
}

pub(crate) fn place_disk(
    builder: &mut TreeBlueprintBuilder,
    center_u: i32,
    center_v: i32,
    layer: u32,
    radius: f32,
    kind: TreeVoxelKind,
) {
    let r = radius.ceil() as i32;

    for du in -r..=r {
        for dv in -r..=r {
            let score = ((du * du + dv * dv) as f32).sqrt();
            if score <= radius + 0.15 {
                builder.place(center_u + du, center_v + dv, layer, kind);
            }
        }
    }
}
