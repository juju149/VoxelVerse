use crate::common::{BlockRef, ItemRef, TagRef};
use serde::{Deserialize, Serialize};

/// Raw recipe definition. Station is in the data, not in the file path.
/// Deserialized from defs/recipes/<name>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeDef {
    #[serde(default)]
    pub pattern: RecipePattern,
    pub result: RecipeResult,
    pub ingredients: Vec<RecipeIngredient>,
    /// None = hand-crafting (inventory). Otherwise, reference to the station block.
    #[serde(default)]
    pub station: Option<BlockRef>,
    /// None = instantaneous. Otherwise, duration in seconds (furnace, etc.).
    #[serde(default)]
    pub time_seconds: Option<f32>,
    #[serde(default)]
    pub tags: Vec<TagRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipePattern {
    Shapeless,
    Shaped,
    Processing,
}

impl Default for RecipePattern {
    fn default() -> Self {
        RecipePattern::Shapeless
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeResult {
    pub item: ItemRef,
    #[serde(default = "default_count")]
    pub count: u32,
}

fn default_count() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RecipeIngredient {
    Item {
        item: ItemRef,
        #[serde(default = "default_count")]
        count: u32,
    },
    Tag {
        tag: TagRef,
        #[serde(default = "default_count")]
        count: u32,
    },
}
