use serde::{Deserialize, Serialize};

/// Tool kind. Single source of truth, shared by BlockMiningDef and ItemKind::Tool.
/// Used in BlockMiningDef.tool (required tool to mine) and ItemKind::Tool.tool_type (tool item type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    Hand,
    Pickaxe,
    Axe,
    Shovel,
    Sword,
    Shears,
    Hoe,
}

impl Default for ToolKind {
    fn default() -> Self {
        ToolKind::Hand
    }
}
