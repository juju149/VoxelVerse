#[derive(Clone, Debug)]
pub(crate) struct TreeGenConfig {
    pub face: u8,
    pub u: u32,
    pub v: u32,
    pub flora_index: u32,
    pub world_seed: u32,
    pub voxel_size_m: f32,

    pub blocks: TreeBlocksDef,
    pub size: TreeSizeDef,
    pub trunk: TreeTrunkDef,
    pub branches: TreeBranchDef,
    pub crown: TreeCrownDef,
    pub roots: TreeRootDef,
    pub variation: TreeVariationDef,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TreeBlocksDef;

#[derive(Clone, Copy, Debug)]
pub(crate) struct TreeSizeDef {
    pub height_min_m: f32,
    pub height_max_m: f32,
    pub radius_min_m: f32,
    pub radius_max_m: f32,
    pub max_total_radius_m: f32,
}

impl Default for TreeSizeDef {
    fn default() -> Self {
        Self {
            height_min_m: 3.5,
            height_max_m: 7.5,
            radius_min_m: 1.4,
            radius_max_m: 2.4,
            max_total_radius_m: 4.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TreeTrunkDef {
    pub base_radius_m: f32,
    pub top_radius_m: f32,
    pub flare_radius_m: f32,
    pub flare_height_m: f32,
    pub bend_strength_m: f32,
    pub bend_frequency: f32,
    pub lean_max_m: f32,
    pub taper: f32,
}

impl Default for TreeTrunkDef {
    fn default() -> Self {
        Self {
            base_radius_m: 0.25,
            top_radius_m: 0.18,
            flare_radius_m: 0.5,
            flare_height_m: 0.8,
            bend_strength_m: 0.35,
            bend_frequency: 0.5,
            lean_max_m: 0.35,
            taper: 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TreeBranchDef {
    pub enabled: bool,
    pub count_min: u32,
    pub count_max: u32,
    pub start_min_t: f32,
    pub start_max_t: f32,
    pub length_min_m: f32,
    pub length_max_m: f32,
    pub upward_tilt: f32,
    pub droop: f32,
    pub fork_chance: f32,
    pub leaf_lobe_radius_m: f32,
}

impl Default for TreeBranchDef {
    fn default() -> Self {
        Self {
            enabled: true,
            count_min: 2,
            count_max: 5,
            start_min_t: 0.45,
            start_max_t: 0.82,
            length_min_m: 0.8,
            length_max_m: 2.0,
            upward_tilt: 0.25,
            droop: 0.05,
            fork_chance: 0.10,
            leaf_lobe_radius_m: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TreeCrownDef {
    pub shape: TreeCrownShape,
    pub start_t: f32,
    pub height_min_m: f32,
    pub height_max_m: f32,
    pub density: f32,
    pub surface_noise: f32,
    pub hollow_core: f32,
    pub lobe_count_min: u32,
    pub lobe_count_max: u32,
    pub lobe_spread_m: f32,
    pub bottom_trim: f32,
}

impl Default for TreeCrownDef {
    fn default() -> Self {
        Self {
            shape: TreeCrownShape::LobedEllipsoid,
            start_t: 0.60,
            height_min_m: 1.8,
            height_max_m: 3.0,
            density: 0.84,
            surface_noise: 0.25,
            hollow_core: 0.16,
            lobe_count_min: 3,
            lobe_count_max: 6,
            lobe_spread_m: 1.0,
            bottom_trim: 0.15,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TreeCrownShape {
    Ellipsoid,
    LobedEllipsoid,
    Cone,
    Layered,
    Columnar,
    Palm,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TreeRootDef {
    pub enabled: bool,
    pub count_min: u32,
    pub count_max: u32,
    pub length_min_m: f32,
    pub length_max_m: f32,
    pub surface_only: bool,
}

impl Default for TreeRootDef {
    fn default() -> Self {
        Self {
            enabled: true,
            count_min: 2,
            count_max: 5,
            length_min_m: 0.6,
            length_max_m: 1.4,
            surface_only: true,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TreeVariationDef {
    pub archetypes: Vec<TreeArchetypeWeight>,
    pub leaf_airiness: f32,
    pub asymmetry: f32,
    pub seed_salt: u32,
}

impl Default for TreeVariationDef {
    fn default() -> Self {
        Self {
            archetypes: vec![TreeArchetypeWeight {
                kind: TreeArchetypeKind::Round,
                weight: 1.0,
            }],
            leaf_airiness: 0.10,
            asymmetry: 0.25,
            seed_salt: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TreeArchetypeWeight {
    pub kind: TreeArchetypeKind,
    pub weight: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TreeArchetypeKind {
    Round,
    Spreading,
    Columnar,
    Layered,
    Irregular,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TreeArchetype {
    Round,
    Spreading,
    Columnar,
    Layered,
    Irregular,
}

impl From<TreeArchetypeKind> for TreeArchetype {
    fn from(value: TreeArchetypeKind) -> Self {
        match value {
            TreeArchetypeKind::Round => Self::Round,
            TreeArchetypeKind::Spreading => Self::Spreading,
            TreeArchetypeKind::Columnar => Self::Columnar,
            TreeArchetypeKind::Layered => Self::Layered,
            TreeArchetypeKind::Irregular => Self::Irregular,
        }
    }
}
