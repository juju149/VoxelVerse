use crate::common::{LangKey, TagRef};
use serde::{Deserialize, Serialize};

mod feature;
mod placement;
mod tree;

pub use feature::FloraFeature;
pub use placement::FloraPlacement;
pub use tree::{
    TreeArchetypeKind, TreeArchetypeWeight, TreeBlocksDef, TreeBranchDef, TreeCrownDef,
    TreeCrownShape, TreeFeatureDef, TreeRootDef, TreeSizeDef, TreeTrunkDef, TreeVariationDef,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FloraDef {
    pub display_key: Option<LangKey>,

    #[serde(default = "one")]
    pub weight: f32,

    #[serde(default)]
    pub required_tags: Vec<TagRef>,

    #[serde(default)]
    pub forbidden_tags: Vec<TagRef>,

    #[serde(default)]
    pub optional_tags: Vec<TagRef>,

    #[serde(default)]
    pub provided_tags: Vec<TagRef>,

    pub placement: FloraPlacement,
    pub feature: FloraFeature,
}

fn one() -> f32 {
    1.0
}
