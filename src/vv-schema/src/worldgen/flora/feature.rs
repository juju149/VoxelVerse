use crate::common::BlockRef;
use serde::{Deserialize, Serialize};

use super::tree::TreeFeatureDef;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum FloraFeature {
    Plant {
        block: BlockRef,
        height_min_m: f32,
        height_max_m: f32,
    },

    Tree(TreeFeatureDef),

    Cluster {
        block: BlockRef,
        radius_min_m: f32,
        radius_max_m: f32,
        density: f32,
    },
}
