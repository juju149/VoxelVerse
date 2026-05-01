mod blueprint;
mod branches;
mod config;
mod crown;
mod rng;
mod roots;
mod trunk;
mod units;

pub(crate) use config::TreeGenConfig;

use blueprint::{TreeBlueprint, TreeBlueprintBuilder};

use self::config::{
    TreeArchetypeKind, TreeArchetypeWeight, TreeBlocksDef, TreeBranchDef, TreeCrownDef,
    TreeCrownShape, TreeRootDef, TreeSizeDef, TreeTrunkDef, TreeVariationDef,
};

pub(crate) struct TreeGenerator;

impl TreeGenerator {
    pub(crate) fn generate(config: TreeGenConfig) -> TreeBlueprint {
        let mut builder = TreeBlueprintBuilder::new(config);

        trunk::generate_trunk(&mut builder);
        branches::generate_branches(&mut builder);
        roots::generate_roots(&mut builder);
        crown::generate_crown(&mut builder);

        builder.finish()
    }
}

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
    pub canopy_start_t: f32,
    pub trunk_girth: f32,
    pub crown_bias: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct TreeShape {
    blueprint: TreeBlueprint,
}

impl TreeShape {
    pub(crate) fn new(config: TreeShapeConfig) -> Self {
        Self {
            blueprint: TreeGenerator::generate(legacy_config_to_tree_gen(config)),
        }
    }

    pub(crate) fn expanded_scan_radius_layers(
        voxel_size_m: f32,
        canopy_radius_m: f32,
        trunk_height_max_m: f32,
    ) -> i32 {
        let voxel_size_m = voxel_size_m.max(0.01);
        let canopy = (canopy_radius_m / voxel_size_m).ceil() as i32;
        let bend = ((trunk_height_max_m / voxel_size_m) * 0.20).ceil() as i32;

        (canopy + bend + 3).max(1)
    }

    pub(crate) fn scan_radius_layers(&self) -> i32 {
        self.blueprint.scan_radius_layers()
    }

    pub(crate) fn max_relative_layer(&self) -> u32 {
        self.blueprint.max_relative_layer()
    }

    pub(crate) fn has_log_at(&self, du: i32, dv: i32, rel_layer: u32) -> bool {
        self.blueprint.has_log_at(du, dv, rel_layer)
    }

    pub(crate) fn has_leaf_at(&self, du: i32, dv: i32, rel_layer: u32) -> bool {
        self.blueprint.has_leaf_at(du, dv, rel_layer)
    }
}

fn legacy_config_to_tree_gen(config: TreeShapeConfig) -> TreeGenConfig {
    let crown_bias = config.crown_bias.clamp(-1.0, 1.0);
    let spreading_weight = 0.20 + crown_bias.max(0.0) * 0.35;
    let columnar_weight = 0.12 + (-crown_bias).max(0.0) * 0.30;
    let irregular_weight = 0.22;

    let trunk_radius = if config.trunk_girth > 0.50 {
        0.55
    } else {
        0.28
    };
    let flare_radius = if config.trunk_girth > 0.50 {
        0.95
    } else {
        0.62
    };

    TreeGenConfig {
        face: config.face,
        u: config.u,
        v: config.v,
        flora_index: config.flora_index,
        world_seed: config.world_seed,
        voxel_size_m: config.voxel_size_m,

        blocks: TreeBlocksDef::default(),

        size: TreeSizeDef {
            height_min_m: config.trunk_height_min_m,
            height_max_m: config.trunk_height_max_m,
            radius_min_m: (config.canopy_radius_m * 0.72).max(config.voxel_size_m),
            radius_max_m: config.canopy_radius_m.max(config.voxel_size_m),
            max_total_radius_m: config.canopy_radius_m * 1.85 + 1.0,
        },

        trunk: TreeTrunkDef {
            base_radius_m: trunk_radius,
            top_radius_m: 0.20,
            flare_radius_m: flare_radius,
            flare_height_m: 0.95,
            bend_strength_m: 0.45 + config.canopy_radius_m * 0.10,
            bend_frequency: 0.70,
            lean_max_m: 0.55 + config.canopy_radius_m * 0.12,
            taper: 0.58,
        },

        branches: TreeBranchDef {
            enabled: true,
            count_min: 2,
            count_max: 6,
            start_min_t: 0.42,
            start_max_t: 0.80,
            length_min_m: (config.canopy_radius_m * 0.38).max(0.6),
            length_max_m: (config.canopy_radius_m * 0.95).max(1.0),
            upward_tilt: 0.30,
            droop: 0.08,
            fork_chance: 0.16,
            leaf_lobe_radius_m: (config.canopy_radius_m * 0.45).max(0.75),
        },

        crown: TreeCrownDef {
            shape: TreeCrownShape::LobedEllipsoid,
            start_t: config.canopy_start_t.clamp(0.45, 0.95),
            height_min_m: (config.canopy_height_m * 0.85).max(config.voxel_size_m),
            height_max_m: (config.canopy_height_m * 1.35).max(config.voxel_size_m),
            density: 0.78,
            surface_noise: 0.35,
            hollow_core: 0.22,
            lobe_count_min: 4,
            lobe_count_max: 8,
            lobe_spread_m: (config.canopy_radius_m * 0.70).max(0.5),
            bottom_trim: 0.28,
        },

        roots: TreeRootDef {
            enabled: true,
            count_min: 3,
            count_max: 6,
            length_min_m: 0.55,
            length_max_m: (config.canopy_radius_m * 0.65).max(1.0),
            surface_only: true,
        },

        variation: TreeVariationDef {
            archetypes: vec![
                TreeArchetypeWeight {
                    kind: TreeArchetypeKind::Round,
                    weight: 0.34,
                },
                TreeArchetypeWeight {
                    kind: TreeArchetypeKind::Spreading,
                    weight: spreading_weight,
                },
                TreeArchetypeWeight {
                    kind: TreeArchetypeKind::Columnar,
                    weight: columnar_weight,
                },
                TreeArchetypeWeight {
                    kind: TreeArchetypeKind::Layered,
                    weight: 0.12,
                },
                TreeArchetypeWeight {
                    kind: TreeArchetypeKind::Irregular,
                    weight: irregular_weight,
                },
            ],
            leaf_airiness: 0.16,
            asymmetry: 0.38,
            seed_salt: 0,
        },
    }
}
