use serde::{Deserialize, Serialize};

use super::render::BlockRenderPatchDef;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockStatesDef {
    pub properties: Vec<StateProperty>,

    #[serde(default)]
    pub render_overrides: Vec<StateRenderOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateProperty {
    pub name: String,
    pub kind: StatePropertyKind,
    pub default_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatePropertyKind {
    Bool,
    Int { min: i32, max: i32 },
    Enum { values: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateRenderOverride {
    pub when: Vec<StateCondition>,
    pub patch: BlockRenderPatchDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateCondition {
    pub property: String,
    pub value: String,
}
