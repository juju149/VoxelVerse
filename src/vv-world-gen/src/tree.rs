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
    /// Fraction of trunk height at which the crown is centered (data-driven, default 0.75).
    pub canopy_start_t: f32,
    /// Trunk girth factor: 0.0 = single-voxel trunk, 1.0 = widest supported trunk.
    pub trunk_girth: f32,
    /// Crown silhouette bias: negative = columnar, positive = spreading. Clamped to [-1.0, 1.0].
    pub crown_bias: f32,
}

/// Overall silhouette archetype for a tree. Chosen deterministically from the tree's seed.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Archetype {
    /// Standard rounded crown -- the baseline.
    Round,
    /// Wide, flat crown with generous horizontal spread.
    Spreading,
    /// Tall and narrow, lobes stacked vertically close to the trunk axis.
    Columnar,
    /// Crown composed of distinct horizontal disk-like layers.
    Layered,
    /// Asymmetric crown with one dominant lobe pulling to a side.
    Irregular,
}

#[derive(Clone, Copy, Debug)]
struct Branch {
    start_layer: u32,
    length: i32,
    du: i32,
    dv: i32,
    /// When 1 the branch gently rises as it extends; 0 means purely horizontal.
    climb: i32,
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
    /// 0 = single-voxel trunk, 1 = radius-1 cross (5 voxels).
    trunk_radius: u32,
    bend_du: i32,
    bend_dv: i32,
    bend_layers: i32,
    sway_du: i32,
    sway_dv: i32,
    crown_du: i32,
    crown_dv: i32,
    /// Layer index at which the crown is centered vertically.
    canopy_layer: u32,
    canopy_radius_u: f32,
    canopy_radius_v: f32,
    canopy_radius_y: f32,
    /// Radius of the interior void ellipsoid that hollows out the crown core.
    void_radius: f32,
    /// Per-archetype base threshold for the main canopy ellipsoid.
    canopy_solid_threshold: f32,
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

        // --- Trunk height ---
        let height_t = (unit(config, salt, 1) * 0.55 + smoothstep(unit(config, salt, 2)) * 0.45)
            .clamp(0.0, 1.0);
        let trunk_height_m = if config.trunk_height_min_m >= config.trunk_height_max_m {
            config.trunk_height_min_m
        } else {
            config.trunk_height_min_m
                + (config.trunk_height_max_m - config.trunk_height_min_m) * height_t
        };
        let trunk_height = meters_to_layers(voxel_size_m, trunk_height_m).max(2);

        // --- Archetype selection ---
        let archetype = pick_archetype(unit(config, salt, 13), config.crown_bias.clamp(-1.0, 1.0));

        let (radius_mult, height_mult, canopy_start_adjust, canopy_solid_threshold) =
            match archetype {
                Archetype::Round => (1.00_f32, 1.00_f32, 0.00_f32, 1.00_f32),
                Archetype::Spreading => (1.35, 0.78, -0.12, 1.00),
                Archetype::Columnar => (0.52, 1.48, 0.08, 0.92),
                Archetype::Layered => (1.12, 0.65, 0.00, 0.82),
                Archetype::Irregular => (1.00, 1.00, 0.00, 0.95),
            };

        // --- Canopy radii ---
        let radius_jitter = 0.82 + unit(config, salt, 3) * 0.42;
        let compactness = 1.12 - height_t * 0.22;
        let effective_radius_m =
            (config.canopy_radius_m * radius_jitter * compactness * radius_mult)
                .max(voxel_size_m * 1.5);
        let effective_height_m =
            (config.canopy_height_m * (0.82 + unit(config, salt, 4) * 0.50) * height_mult)
                .max(voxel_size_m);
        let canopy_radius_layers = (effective_radius_m / voxel_size_m).ceil().max(1.0);
        let canopy_radius_y = (effective_height_m / voxel_size_m).ceil().max(1.0);
        let asymmetry = 0.78 + unit(config, salt, 5) * 0.48;
        let canopy_radius_u = canopy_radius_layers * asymmetry;
        let canopy_radius_v = canopy_radius_layers * (1.34 - asymmetry * 0.36);

        // --- Trunk bend & sway ---
        let bend_dir = direction(unit(config, salt, 6));
        let sway_dir = direction(unit(config, salt, 7));
        let bend_cap = ((trunk_height as f32 * 0.18).min(canopy_radius_layers * 0.55)) as i32;
        let bend_layers = if bend_cap <= 0 {
            0
        } else {
            (unit(config, salt, 8) * (bend_cap as f32 + 1.0)).floor() as i32
        };

        // --- Crown centre ---
        let crown_side = direction(unit(config, salt, 9));
        let crown_push = if canopy_radius_layers >= 3.0 && unit(config, salt, 10) > 0.35 {
            1
        } else {
            0
        };
        let top = trunk_offset(trunk_height, trunk_height, bend_dir, bend_layers, sway_dir);
        let crown_du = top.0 + crown_side.0 * crown_push;
        let crown_dv = top.1 + crown_side.1 * crown_push;

        // --- Canopy start layer ---
        let effective_canopy_start_t =
            (config.canopy_start_t + canopy_start_adjust).clamp(0.40, 1.10);
        let canopy_layer = ((trunk_height as f32 * effective_canopy_start_t).round() as u32)
            .clamp(1, trunk_height.saturating_add(2));

        // --- Trunk girth ---
        let base_radius: u32 = if config.trunk_girth > 0.50 { 1 } else { 0 };
        let size_bonus: u32 =
            if trunk_height >= 14 && unit(config, salt, 14) < config.trunk_girth + 0.30 {
                1
            } else {
                0
            };
        let trunk_radius = (base_radius + size_bonus).min(1);

        // --- Interior void ---
        let void_radius = canopy_radius_layers * (0.22 + unit(config, salt, 15) * 0.20);

        // --- Branches ---
        let mut branches = [Branch {
            start_layer: 0,
            length: 0,
            du: 0,
            dv: 0,
            climb: 0,
        }; BRANCH_CAPACITY];
        let branch_count = branch_count(canopy_radius_layers, trunk_height, unit(config, salt, 11));
        for (index, branch) in branches.iter_mut().take(branch_count).enumerate() {
            let branch_dir = direction(unit(config, salt, 20 + index as u32));
            let start_t = 0.48 + unit(config, salt, 30 + index as u32) * 0.36;
            let start_layer = ((trunk_height as f32 * start_t).round() as u32)
                .clamp(2, trunk_height.saturating_sub(1).max(2));
            let max_len = canopy_radius_layers.round().max(1.0) as i32;
            let length = 1 + (unit(config, salt, 40 + index as u32) * max_len as f32) as i32;
            let climb = if unit(config, salt, 45 + index as u32) > 0.40 {
                1
            } else {
                0
            };
            *branch = Branch {
                start_layer,
                length: length.min(max_len.max(1)),
                du: branch_dir.0,
                dv: branch_dir.1,
                climb,
            };
        }

        // --- Leaf lobes ---
        let mut lobes = [LeafLobe {
            du: 0,
            dv: 0,
            layer: 0,
            radius_u: 1.0,
            radius_v: 1.0,
            radius_y: 1.0,
        }; LOBE_CAPACITY];
        let lobe_count = compute_lobe_count(archetype, unit(config, salt, 12));
        for (index, lobe) in lobes.iter_mut().take(lobe_count).enumerate() {
            *lobe = compute_lobe(
                archetype,
                index,
                config,
                salt,
                crown_du,
                crown_dv,
                canopy_layer,
                canopy_radius_layers,
                canopy_radius_y,
            );
        }

        let scan_radius_layers = Self::expanded_scan_radius_layers(
            voxel_size_m,
            config.canopy_radius_m * radius_mult.max(1.0),
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
            trunk_radius,
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
            void_radius,
            canopy_solid_threshold,
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

        let on_trunk = if self.trunk_radius == 0 {
            (du, dv) == trunk
        } else {
            (du - trunk.0).pow(2) + (dv - trunk.1).pow(2) <= 1
        };

        if on_trunk {
            return true;
        }

        let base_flare = if self.trunk_radius == 0 {
            rel_layer <= 2
                && (du - trunk.0).abs() + (dv - trunk.1).abs() == 1
                && self.voxel_hash(du, dv, rel_layer, 200) < 0.34
        } else {
            rel_layer <= 2
                && (du - trunk.0).pow(2) + (dv - trunk.1).pow(2) <= 4
                && self.voxel_hash(du, dv, rel_layer, 200) < 0.45
        };

        base_flare || self.has_branch_log_at(du, dv, rel_layer)
    }

    pub(crate) fn has_leaf_at(&self, du: i32, dv: i32, rel_layer: u32) -> bool {
        if rel_layer
            < self
                .canopy_layer
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
        let threshold = canopy_threshold(rel_layer, self.canopy_layer, self.canopy_solid_threshold);
        let mut inside = best_score <= threshold;

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

        // Interior void: hollows out the dense crown core for a more organic look.
        let void_score = ellipsoid_score(
            du - self.crown_du,
            dv - self.crown_dv,
            rel_layer as i32 - self.canopy_layer as i32,
            self.void_radius,
            self.void_radius,
            self.void_radius * 0.70,
        );
        if void_score < 1.0 && best_score < 0.30 {
            return self.voxel_hash(du, dv, rel_layer, 400) < 0.18;
        }

        let interior = best_score < 0.36;
        let noise_bump = self.voxel_hash(du, dv, rel_layer, 310) * 0.04;
        let hole_chance = if interior {
            0.015 + noise_bump
        } else {
            0.06 + best_score.clamp(0.0, 1.4) * 0.12 + noise_bump
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
                let climb_layers = (step as u32 / 4) * branch.climb as u32;
                let branch_layer = branch
                    .start_layer
                    .saturating_add(step as u32 / 2)
                    .saturating_add(climb_layers);
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

fn pick_archetype(roll: f32, crown_bias: f32) -> Archetype {
    let bias = crown_bias.clamp(-1.0, 1.0);
    let p_round = 0.35_f32;
    let p_spreading = (0.25 + bias.max(0.0) * 0.30).clamp(0.05, 0.55);
    let p_columnar = (0.15 + (-bias).max(0.0) * 0.25).clamp(0.05, 0.40);
    let p_layered = 0.15_f32;
    let total_fixed = p_round + p_spreading + p_columnar + p_layered;
    let p_irregular = (1.0 - total_fixed).max(0.05);
    let total = total_fixed + p_irregular;

    let t = roll * total;
    let mut c = p_round;
    if t < c {
        return Archetype::Round;
    }
    c += p_spreading;
    if t < c {
        return Archetype::Spreading;
    }
    c += p_columnar;
    if t < c {
        return Archetype::Columnar;
    }
    c += p_layered;
    if t < c {
        return Archetype::Layered;
    }
    Archetype::Irregular
}

fn compute_lobe_count(archetype: Archetype, roll: f32) -> usize {
    match archetype {
        Archetype::Round => (3 + (roll * 3.0) as usize).min(LOBE_CAPACITY),
        Archetype::Spreading => (4 + (roll * 1.0) as usize).min(LOBE_CAPACITY),
        Archetype::Columnar => LOBE_CAPACITY,
        Archetype::Layered => 4_usize.min(LOBE_CAPACITY),
        Archetype::Irregular => (2 + (roll * 2.0) as usize).min(LOBE_CAPACITY),
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_lobe(
    archetype: Archetype,
    index: usize,
    config: TreeShapeConfig,
    salt: u32,
    crown_du: i32,
    crown_dv: i32,
    canopy_layer: u32,
    canopy_radius_layers: f32,
    canopy_radius_y: f32,
) -> LeafLobe {
    let i = index as u32;
    match archetype {
        Archetype::Round => {
            let dir = direction(unit(config, salt, 50 + i));
            let distance = (0.25 + unit(config, salt, 60 + i) * 0.65) * canopy_radius_layers;
            let layer_offset =
                ((unit(config, salt, 70 + i) - 0.42) * canopy_radius_y).round() as i32;
            LeafLobe {
                du: crown_du + (dir.0 as f32 * distance).round() as i32,
                dv: crown_dv + (dir.1 as f32 * distance).round() as i32,
                layer: offset_layer(canopy_layer, layer_offset),
                radius_u: (canopy_radius_layers * (0.45 + unit(config, salt, 80 + i) * 0.28))
                    .max(1.0),
                radius_v: (canopy_radius_layers * (0.42 + unit(config, salt, 90 + i) * 0.30))
                    .max(1.0),
                radius_y: (canopy_radius_y * (0.55 + unit(config, salt, 100 + i) * 0.32)).max(1.0),
            }
        }
        Archetype::Spreading => {
            // Lobe distance same as Round; the spread effect comes from the 1.35x radius_mult.
            // Lobes are wider and much flatter to reinforce the horizontal silhouette.
            let dir = direction(unit(config, salt, 50 + i));
            let distance = (0.25 + unit(config, salt, 60 + i) * 0.65) * canopy_radius_layers;
            let layer_offset =
                ((unit(config, salt, 70 + i) - 0.50) * canopy_radius_y * 0.55).round() as i32;
            LeafLobe {
                du: crown_du + (dir.0 as f32 * distance).round() as i32,
                dv: crown_dv + (dir.1 as f32 * distance).round() as i32,
                layer: offset_layer(canopy_layer, layer_offset),
                radius_u: (canopy_radius_layers * (0.50 + unit(config, salt, 80 + i) * 0.30))
                    .max(1.0),
                radius_v: (canopy_radius_layers * (0.48 + unit(config, salt, 90 + i) * 0.30))
                    .max(1.0),
                radius_y: (canopy_radius_y * (0.28 + unit(config, salt, 100 + i) * 0.25)).max(1.0),
            }
        }
        Archetype::Columnar => {
            let stack: [i32; 5] = [0, -2, 2, -4, 4];
            let layer_offset = stack[index.min(4)];
            let small_dist = unit(config, salt, 60 + i) * canopy_radius_layers * 0.28;
            let dir = direction(unit(config, salt, 50 + i));
            LeafLobe {
                du: crown_du + (dir.0 as f32 * small_dist).round() as i32,
                dv: crown_dv + (dir.1 as f32 * small_dist).round() as i32,
                layer: offset_layer(canopy_layer, layer_offset),
                radius_u: (canopy_radius_layers * (0.38 + unit(config, salt, 80 + i) * 0.18))
                    .max(1.0),
                radius_v: (canopy_radius_layers * (0.36 + unit(config, salt, 90 + i) * 0.18))
                    .max(1.0),
                radius_y: (canopy_radius_y * (0.58 + unit(config, salt, 100 + i) * 0.32)).max(1.0),
            }
        }
        Archetype::Layered => {
            let disk: [i32; 5] = [0, -3, 3, -6, 1];
            let layer_offset = disk[index.min(4)];
            let dir = direction(unit(config, salt, 50 + i));
            let distance = unit(config, salt, 60 + i) * canopy_radius_layers * 0.42;
            LeafLobe {
                du: crown_du + (dir.0 as f32 * distance).round() as i32,
                dv: crown_dv + (dir.1 as f32 * distance).round() as i32,
                layer: offset_layer(canopy_layer, layer_offset),
                radius_u: (canopy_radius_layers * (0.55 + unit(config, salt, 80 + i) * 0.22))
                    .max(1.0),
                radius_v: (canopy_radius_layers * (0.52 + unit(config, salt, 90 + i) * 0.22))
                    .max(1.0),
                radius_y: (canopy_radius_y * (0.20 + unit(config, salt, 100 + i) * 0.15)).max(1.0),
            }
        }
        Archetype::Irregular => {
            let dir = direction(unit(config, salt, 50 + i));
            let (distance, layer_offset, ru_scale, rv_scale, ry_scale) = if index == 0 {
                (
                    (0.55 + unit(config, salt, 60) * 0.38) * canopy_radius_layers,
                    ((unit(config, salt, 70) - 0.30) * canopy_radius_y).round() as i32,
                    0.62 + unit(config, salt, 80) * 0.28,
                    0.58 + unit(config, salt, 90) * 0.28,
                    0.60 + unit(config, salt, 100) * 0.30,
                )
            } else {
                (
                    (0.20 + unit(config, salt, 60 + i) * 0.55) * canopy_radius_layers,
                    ((unit(config, salt, 70 + i) - 0.42) * canopy_radius_y).round() as i32,
                    0.40 + unit(config, salt, 80 + i) * 0.22,
                    0.38 + unit(config, salt, 90 + i) * 0.22,
                    0.45 + unit(config, salt, 100 + i) * 0.25,
                )
            };
            LeafLobe {
                du: crown_du + (dir.0 as f32 * distance).round() as i32,
                dv: crown_dv + (dir.1 as f32 * distance).round() as i32,
                layer: offset_layer(canopy_layer, layer_offset),
                radius_u: (canopy_radius_layers * ru_scale).max(1.0),
                radius_v: (canopy_radius_layers * rv_scale).max(1.0),
                radius_y: (canopy_radius_y * ry_scale).max(1.0),
            }
        }
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

fn canopy_threshold(rel_layer: u32, canopy_layer: u32, base: f32) -> f32 {
    if rel_layer + 1 < canopy_layer {
        base * 0.82
    } else if rel_layer > canopy_layer + 2 {
        base * 0.90
    } else {
        base
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
            canopy_start_t: 0.75,
            trunk_girth: 0.0,
            crown_bias: 0.0,
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

    #[test]
    fn archetypes_vary_with_crown_bias() {
        let mut spreading_count = 0u32;
        let mut other_count = 0u32;
        for seed in 0u32..200 {
            let roll = hash01(0, seed * 37, seed * 53, 0, seed.rotate_left(11));
            let arch = pick_archetype(roll, 1.0);
            if arch == Archetype::Spreading {
                spreading_count += 1;
            } else {
                other_count += 1;
            }
        }
        assert!(
            spreading_count > other_count / 2,
            "Spreading should be dominant with bias=1.0, got {spreading_count} vs {other_count}"
        );
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
