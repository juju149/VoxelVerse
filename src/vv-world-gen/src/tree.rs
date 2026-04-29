use super::{hash01, smoothstep};

const BRANCH_CAPACITY: usize = 4;
const LOBE_CAPACITY: usize = 5;
const DIRECTIONS: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

#[derive(Clone, Copy, Debug)]
pub(crate) struct TreeShapeConfig {
    pub face: u8,
    pub u: u32,
    pub v: u32,
    pub flora_index: u32,
    pub world_seed: u32,
    pub voxel_size_m: f32,
    pub trunk_height_min_m: f32,
    pub trunk_height_max_m: f32,
    pub canopy_radius_m: f32,
    pub canopy_height_m: f32,
}

#[derive(Clone, Copy, Debug)]
struct Branch {
    start_layer: u32,
    length: i32,
    du: i32,
    dv: i32,
}

#[derive(Clone, Copy, Debug)]
struct LeafLobe {
    du: i32,
    dv: i32,
    layer: u32,
    radius_u: f32,
    radius_v: f32,
    radius_y: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct TreeShape {
    face: u8,
    u: u32,
    v: u32,
    salt: u32,
    trunk_height: u32,
    bend_du: i32,
    bend_dv: i32,
    bend_layers: i32,
    sway_du: i32,
    sway_dv: i32,
    crown_du: i32,
    crown_dv: i32,
    canopy_layer: u32,
    canopy_radius_u: f32,
    canopy_radius_v: f32,
    canopy_radius_y: f32,
    scan_radius_layers: i32,
    max_relative_layer: u32,
    branches: [Branch; BRANCH_CAPACITY],
    branch_count: usize,
    lobes: [LeafLobe; LOBE_CAPACITY],
    lobe_count: usize,
}

impl TreeShape {
    pub(crate) fn new(config: TreeShapeConfig) -> Self {
        let salt = config
            .world_seed
            .rotate_left(11)
            .wrapping_add(config.flora_index.wrapping_mul(0x9E37_79B9));
        let voxel_size_m = config.voxel_size_m.max(0.01);
        let height_t = (unit(config, salt, 1) * 0.55 + smoothstep(unit(config, salt, 2)) * 0.45)
            .clamp(0.0, 1.0);
        let trunk_height_m = if config.trunk_height_min_m >= config.trunk_height_max_m {
            config.trunk_height_min_m
        } else {
            config.trunk_height_min_m
                + (config.trunk_height_max_m - config.trunk_height_min_m) * height_t
        };
        let trunk_height = meters_to_layers(voxel_size_m, trunk_height_m).max(2);

        let radius_jitter = 0.82 + unit(config, salt, 3) * 0.42;
        let compactness = 1.12 - height_t * 0.22;
        let canopy_radius_m =
            (config.canopy_radius_m * radius_jitter * compactness).max(voxel_size_m * 1.5);
        let canopy_height_m =
            (config.canopy_height_m * (0.82 + unit(config, salt, 4) * 0.5)).max(voxel_size_m);
        let canopy_radius_layers = (canopy_radius_m / voxel_size_m).ceil().max(1.0);
        let canopy_radius_y = (canopy_height_m / voxel_size_m).ceil().max(1.0);
        let asymmetry = 0.78 + unit(config, salt, 5) * 0.48;
        let canopy_radius_u = canopy_radius_layers * asymmetry;
        let canopy_radius_v = canopy_radius_layers * (1.34 - asymmetry * 0.36);

        let bend_dir = direction(unit(config, salt, 6));
        let sway_dir = direction(unit(config, salt, 7));
        let bend_cap = ((trunk_height as f32 * 0.18).min(canopy_radius_layers * 0.55)) as i32;
        let bend_layers = if bend_cap <= 0 {
            0
        } else {
            (unit(config, salt, 8) * (bend_cap as f32 + 1.0)).floor() as i32
        };
        let crown_side = direction(unit(config, salt, 9));
        let crown_push = if canopy_radius_layers >= 3.0 && unit(config, salt, 10) > 0.35 {
            1
        } else {
            0
        };
        let top = trunk_offset(trunk_height, trunk_height, bend_dir, bend_layers, sway_dir);
        let crown_du = top.0 + crown_side.0 * crown_push;
        let crown_dv = top.1 + crown_side.1 * crown_push;
        let canopy_layer = trunk_height.saturating_sub((canopy_radius_y * 0.25) as u32);

        let mut branches = [Branch {
            start_layer: 0,
            length: 0,
            du: 0,
            dv: 0,
        }; BRANCH_CAPACITY];
        let branch_count = branch_count(canopy_radius_layers, trunk_height, unit(config, salt, 11));
        for (index, branch) in branches.iter_mut().take(branch_count).enumerate() {
            let branch_dir = direction(unit(config, salt, 20 + index as u32));
            let start_t = 0.48 + unit(config, salt, 30 + index as u32) * 0.36;
            let start_layer = ((trunk_height as f32 * start_t).round() as u32)
                .clamp(2, trunk_height.saturating_sub(1).max(2));
            let max_len = canopy_radius_layers.round().max(1.0) as i32;
            let length = 1 + (unit(config, salt, 40 + index as u32) * max_len as f32) as i32;
            *branch = Branch {
                start_layer,
                length: length.min(max_len.max(1)),
                du: branch_dir.0,
                dv: branch_dir.1,
            };
        }

        let mut lobes = [LeafLobe {
            du: 0,
            dv: 0,
            layer: 0,
            radius_u: 1.0,
            radius_v: 1.0,
            radius_y: 1.0,
        }; LOBE_CAPACITY];
        let lobe_count = (3 + (unit(config, salt, 12) * 3.0) as usize).min(LOBE_CAPACITY);
        for (index, lobe) in lobes.iter_mut().take(lobe_count).enumerate() {
            let dir = direction(unit(config, salt, 50 + index as u32));
            let distance =
                (0.25 + unit(config, salt, 60 + index as u32) * 0.65) * canopy_radius_layers;
            let layer_offset =
                ((unit(config, salt, 70 + index as u32) - 0.42) * canopy_radius_y).round() as i32;
            *lobe = LeafLobe {
                du: crown_du + (dir.0 as f32 * distance).round() as i32,
                dv: crown_dv + (dir.1 as f32 * distance).round() as i32,
                layer: offset_layer(canopy_layer, layer_offset),
                radius_u: (canopy_radius_layers
                    * (0.45 + unit(config, salt, 80 + index as u32) * 0.28))
                    .max(1.0),
                radius_v: (canopy_radius_layers
                    * (0.42 + unit(config, salt, 90 + index as u32) * 0.3))
                    .max(1.0),
                radius_y: (canopy_radius_y
                    * (0.55 + unit(config, salt, 100 + index as u32) * 0.32))
                    .max(1.0),
            };
        }

        let scan_radius_layers = Self::expanded_scan_radius_layers(
            voxel_size_m,
            config.canopy_radius_m,
            config.trunk_height_max_m,
        );
        let max_relative_layer = trunk_height
            .saturating_add(((canopy_radius_y * 2.0).ceil() as u32).max(1))
            .saturating_add(2);

        Self {
            face: config.face,
            u: config.u,
            v: config.v,
            salt,
            trunk_height,
            bend_du: bend_dir.0,
            bend_dv: bend_dir.1,
            bend_layers,
            sway_du: sway_dir.0,
            sway_dv: sway_dir.1,
            crown_du,
            crown_dv,
            canopy_layer,
            canopy_radius_u,
            canopy_radius_v,
            canopy_radius_y,
            scan_radius_layers,
            max_relative_layer,
            branches,
            branch_count,
            lobes,
            lobe_count,
        }
    }

    pub(crate) fn expanded_scan_radius_layers(
        voxel_size_m: f32,
        canopy_radius_m: f32,
        trunk_height_max_m: f32,
    ) -> i32 {
        let voxel_size_m = voxel_size_m.max(0.01);
        let canopy = (canopy_radius_m / voxel_size_m).ceil() as i32;
        let bend = ((trunk_height_max_m / voxel_size_m) * 0.2).ceil() as i32;
        (canopy + bend + 2).max(1)
    }

    pub(crate) fn scan_radius_layers(&self) -> i32 {
        self.scan_radius_layers
    }

    pub(crate) fn max_relative_layer(&self) -> u32 {
        self.max_relative_layer
    }

    pub(crate) fn has_log_at(&self, du: i32, dv: i32, rel_layer: u32) -> bool {
        if rel_layer == 0 || rel_layer > self.trunk_height {
            return self.has_branch_log_at(du, dv, rel_layer);
        }

        let trunk = self.trunk_offset_at(rel_layer);
        if (du, dv) == trunk {
            return true;
        }

        let base_flare = rel_layer <= 2
            && (du - trunk.0).abs() + (dv - trunk.1).abs() == 1
            && self.voxel_hash(du, dv, rel_layer, 200) < 0.34;
        base_flare || self.has_branch_log_at(du, dv, rel_layer)
    }

    pub(crate) fn has_leaf_at(&self, du: i32, dv: i32, rel_layer: u32) -> bool {
        if rel_layer
            < self
                .trunk_height
                .saturating_sub(self.canopy_radius_y as u32 + 1)
        {
            return false;
        }
        if self.has_log_at(du, dv, rel_layer) {
            return false;
        }

        let mut best_score = ellipsoid_score(
            du - self.crown_du,
            dv - self.crown_dv,
            rel_layer as i32 - self.canopy_layer as i32,
            self.canopy_radius_u,
            self.canopy_radius_v,
            self.canopy_radius_y,
        );
        let mut inside = best_score <= canopy_threshold(rel_layer, self.canopy_layer);

        for lobe in self.lobes.iter().take(self.lobe_count) {
            let score = ellipsoid_score(
                du - lobe.du,
                dv - lobe.dv,
                rel_layer as i32 - lobe.layer as i32,
                lobe.radius_u,
                lobe.radius_v,
                lobe.radius_y,
            );
            if score <= 1.0 {
                inside = true;
                best_score = best_score.min(score * 0.9);
            }
        }

        if !inside {
            return false;
        }

        let interior = best_score < 0.36;
        let hole_chance = if interior {
            0.015
        } else {
            0.06 + best_score.clamp(0.0, 1.4) * 0.12
        };
        self.voxel_hash(du, dv, rel_layer, 300) >= hole_chance
    }

    fn trunk_offset_at(&self, rel_layer: u32) -> (i32, i32) {
        trunk_offset(
            rel_layer,
            self.trunk_height,
            (self.bend_du, self.bend_dv),
            self.bend_layers,
            (self.sway_du, self.sway_dv),
        )
    }

    fn has_branch_log_at(&self, du: i32, dv: i32, rel_layer: u32) -> bool {
        for branch in self.branches.iter().take(self.branch_count) {
            let base = self.trunk_offset_at(branch.start_layer);
            for step in 1..=branch.length {
                let branch_layer = branch.start_layer + (step / 2) as u32;
                if rel_layer != branch_layer {
                    continue;
                }
                let pos = (base.0 + branch.du * step, base.1 + branch.dv * step);
                if (du, dv) == pos {
                    return true;
                }
            }
        }
        false
    }

    fn voxel_hash(&self, du: i32, dv: i32, rel_layer: u32, salt_offset: u32) -> f32 {
        let hu = self.u.wrapping_add((du + 8_192) as u32);
        let hv = self.v.wrapping_add((dv + 8_192) as u32);
        hash01(
            self.face,
            hu,
            hv,
            rel_layer,
            self.salt.wrapping_add(salt_offset),
        )
    }
}

fn unit(config: TreeShapeConfig, salt: u32, offset: u32) -> f32 {
    hash01(
        config.face,
        config.u,
        config.v,
        offset,
        salt.wrapping_add(offset.wrapping_mul(0x85EB_CA6B)),
    )
}

fn meters_to_layers(voxel_size_m: f32, meters: f32) -> u32 {
    (meters.max(0.0) / voxel_size_m).ceil() as u32
}

fn direction(value: f32) -> (i32, i32) {
    let index = ((value.clamp(0.0, 0.999_999) * DIRECTIONS.len() as f32) as usize)
        .min(DIRECTIONS.len() - 1);
    DIRECTIONS[index]
}

fn trunk_offset(
    rel_layer: u32,
    trunk_height: u32,
    bend_dir: (i32, i32),
    bend_layers: i32,
    sway_dir: (i32, i32),
) -> (i32, i32) {
    if trunk_height <= 1 || bend_layers == 0 {
        return (0, 0);
    }
    let t = rel_layer as f32 / trunk_height as f32;
    let bend = smoothstep(t).clamp(0.0, 1.0) * bend_layers as f32;
    let sway = ((t * std::f32::consts::PI * 1.5).sin() * 0.45).round() as i32;
    (
        (bend_dir.0 as f32 * bend).round() as i32 + sway_dir.0 * sway,
        (bend_dir.1 as f32 * bend).round() as i32 + sway_dir.1 * sway,
    )
}

fn branch_count(canopy_radius_layers: f32, trunk_height: u32, roll: f32) -> usize {
    if canopy_radius_layers < 2.0 || trunk_height < 5 {
        return 1;
    }
    (2 + (roll * 3.0) as usize).min(BRANCH_CAPACITY)
}

fn offset_layer(layer: u32, offset: i32) -> u32 {
    if offset < 0 {
        layer.saturating_sub(offset.unsigned_abs())
    } else {
        layer.saturating_add(offset as u32)
    }
}

fn ellipsoid_score(du: i32, dv: i32, dl: i32, radius_u: f32, radius_v: f32, radius_y: f32) -> f32 {
    let u = du as f32 / radius_u.max(1.0);
    let v = dv as f32 / radius_v.max(1.0);
    let y = dl as f32 / radius_y.max(1.0);
    u * u + v * v + y * y
}

fn canopy_threshold(rel_layer: u32, canopy_layer: u32) -> f32 {
    if rel_layer + 1 < canopy_layer {
        0.82
    } else if rel_layer > canopy_layer + 2 {
        0.9
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_shape_is_deterministic_and_varies_by_origin() {
        let config = TreeShapeConfig {
            face: 2,
            u: 120,
            v: 320,
            flora_index: 1,
            world_seed: 42,
            voxel_size_m: 0.5,
            trunk_height_min_m: 4.0,
            trunk_height_max_m: 6.0,
            canopy_radius_m: 2.0,
            canopy_height_m: 1.5,
        };

        let a = TreeShape::new(config);
        let b = TreeShape::new(config);
        assert_eq!(signature(&a), signature(&b));

        let shifted = TreeShape::new(TreeShapeConfig {
            u: config.u + 17,
            v: config.v + 9,
            ..config
        });
        assert_ne!(signature(&a), signature(&shifted));
    }

    fn signature(shape: &TreeShape) -> (u32, u32, i32, i32, u32) {
        let mut logs = 0;
        let mut leaves = 0;
        let mut u_sum = 0;
        let mut v_sum = 0;
        let radius = shape.scan_radius_layers();
        for du in -radius..=radius {
            for dv in -radius..=radius {
                for layer in 1..=shape.max_relative_layer() {
                    if shape.has_log_at(du, dv, layer) {
                        logs += 1;
                        u_sum += du;
                        v_sum += dv;
                    }
                    if shape.has_leaf_at(du, dv, layer) {
                        leaves += 1;
                        u_sum += du;
                        v_sum += dv;
                    }
                }
            }
        }
        (logs, leaves, u_sum, v_sum, shape.max_relative_layer())
    }
}
