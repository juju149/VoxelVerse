use std::collections::HashMap;

use super::config::{TreeArchetype, TreeGenConfig};
use super::rng::TreeRng;
use super::units::meters_to_voxels;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TreeVoxelKind {
    Log,
    Branch,
    Root,
    Leaf,
    Fruit,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TreeVoxel {
    pub du: i32,
    pub dv: i32,
    pub layer: u32,
    pub kind: TreeVoxelKind,
}

#[derive(Clone, Debug)]
pub(crate) struct TreeBlueprint {
    voxels: Vec<TreeVoxel>,
    scan_radius_layers: i32,
    max_relative_layer: u32,
}

impl TreeBlueprint {
    pub(crate) fn voxels(&self) -> &[TreeVoxel] {
        &self.voxels
    }

    pub(crate) fn scan_radius_layers(&self) -> i32 {
        self.scan_radius_layers
    }

    pub(crate) fn max_relative_layer(&self) -> u32 {
        self.max_relative_layer
    }

    pub(crate) fn has_log_at(&self, du: i32, dv: i32, layer: u32) -> bool {
        self.voxels.iter().any(|voxel| {
            voxel.du == du
                && voxel.dv == dv
                && voxel.layer == layer
                && matches!(
                    voxel.kind,
                    TreeVoxelKind::Log | TreeVoxelKind::Branch | TreeVoxelKind::Root
                )
        })
    }

    pub(crate) fn has_leaf_at(&self, du: i32, dv: i32, layer: u32) -> bool {
        self.voxels.iter().any(|voxel| {
            voxel.du == du
                && voxel.dv == dv
                && voxel.layer == layer
                && voxel.kind == TreeVoxelKind::Leaf
        })
    }
}

pub(crate) struct TreeBlueprintBuilder {
    pub config: TreeGenConfig,
    pub rng: TreeRng,
    pub archetype: TreeArchetype,
    pub height_layers: u32,
    pub crown_radius_layers: i32,
    pub crown_height_layers: u32,
    pub crown_start_layer: u32,
    pub crown_center: (i32, i32, u32),
    voxels: HashMap<(i32, i32, u32), TreeVoxelKind>,
}

impl TreeBlueprintBuilder {
    pub(crate) fn new(config: TreeGenConfig) -> Self {
        let mut rng = TreeRng::new(
            config.face,
            config.u,
            config.v,
            config.world_seed ^ config.variation.seed_salt,
            config.flora_index,
        );

        let height_m = rng.range_f32(config.size.height_min_m, config.size.height_max_m);
        let radius_m = rng.range_f32(config.size.radius_min_m, config.size.radius_max_m);
        let crown_height_m = rng.range_f32(config.crown.height_min_m, config.crown.height_max_m);

        let height_layers = meters_to_voxels(height_m, config.voxel_size_m).max(3);
        let crown_radius_layers = meters_to_voxels(radius_m, config.voxel_size_m).max(1) as i32;
        let crown_height_layers = meters_to_voxels(crown_height_m, config.voxel_size_m).max(1);
        let crown_start_layer =
            ((height_layers as f32 * config.crown.start_t).round() as u32).clamp(1, height_layers);

        let archetype = pick_archetype(&config, &mut rng);

        Self {
            config,
            rng,
            archetype,
            height_layers,
            crown_radius_layers,
            crown_height_layers,
            crown_start_layer,
            crown_center: (0, 0, crown_start_layer + crown_height_layers / 2),
            voxels: HashMap::new(),
        }
    }

    pub(crate) fn set_crown_center(&mut self, du: i32, dv: i32, layer: u32) {
        self.crown_center = (du, dv, layer);
    }

    pub(crate) fn place(&mut self, du: i32, dv: i32, layer: u32, kind: TreeVoxelKind) {
        if layer == 0 {
            return;
        }

        let key = (du, dv, layer);

        match self.voxels.get(&key).copied() {
            None => {
                self.voxels.insert(key, kind);
            }
            Some(existing) => {
                if priority(kind) >= priority(existing) {
                    self.voxels.insert(key, kind);
                }
            }
        }
    }

    pub(crate) fn finish(self) -> TreeBlueprint {
        let mut voxels: Vec<TreeVoxel> = self
            .voxels
            .into_iter()
            .map(|((du, dv, layer), kind)| TreeVoxel {
                du,
                dv,
                layer,
                kind,
            })
            .collect();

        voxels.sort_by_key(|voxel| (voxel.layer, voxel.du, voxel.dv));

        let scan_radius_layers = self.crown_radius_layers.max(meters_to_voxels(
            self.config.size.max_total_radius_m,
            self.config.voxel_size_m,
        ) as i32)
            + 2;

        let max_relative_layer = self
            .height_layers
            .saturating_add(self.crown_height_layers * 2)
            .saturating_add(3);

        TreeBlueprint {
            voxels,
            scan_radius_layers,
            max_relative_layer,
        }
    }
}

fn priority(kind: TreeVoxelKind) -> u8 {
    match kind {
        TreeVoxelKind::Leaf => 1,
        TreeVoxelKind::Fruit => 2,
        TreeVoxelKind::Root => 3,
        TreeVoxelKind::Branch => 4,
        TreeVoxelKind::Log => 5,
    }
}

fn pick_archetype(config: &TreeGenConfig, rng: &mut TreeRng) -> TreeArchetype {
    let total: f32 = config
        .variation
        .archetypes
        .iter()
        .map(|entry| entry.weight.max(0.0))
        .sum();

    if total <= 0.0001 {
        return TreeArchetype::Round;
    }

    let mut roll = rng.next_f32() * total;

    for entry in &config.variation.archetypes {
        let weight = entry.weight.max(0.0);
        if roll <= weight {
            return entry.kind.into();
        }
        roll -= weight;
    }

    TreeArchetype::Round
}
