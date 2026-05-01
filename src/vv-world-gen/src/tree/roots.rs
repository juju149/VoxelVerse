use super::blueprint::{TreeBlueprintBuilder, TreeVoxelKind};
use super::units::meters_to_voxels;

pub(crate) fn generate_roots(builder: &mut TreeBlueprintBuilder) {
    if !builder.config.roots.enabled {
        return;
    }

    let count = builder.rng.range_u32(
        builder.config.roots.count_min,
        builder.config.roots.count_max,
    );

    for _ in 0..count {
        let dir = builder.rng.direction8();

        let length = meters_to_voxels(
            builder.rng.range_f32(
                builder.config.roots.length_min_m,
                builder.config.roots.length_max_m,
            ),
            builder.config.voxel_size_m,
        )
        .max(1);

        for step in 1..=length {
            let layer = if builder.config.roots.surface_only {
                1
            } else {
                1 + step / 4
            };

            builder.place(
                dir.0 * step as i32,
                dir.1 * step as i32,
                layer,
                TreeVoxelKind::Root,
            );
        }
    }
}
