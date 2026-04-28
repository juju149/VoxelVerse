use crate::common::{LangKey, ResourceRef, ScriptRef, TagRef};
use crate::loot::DropSpec;
use serde::{Deserialize, Serialize};

/// Raw definition of a NON-VOXEL placed object.
/// For objects that occupy world space but are not part of the voxel grid:
/// signs, paintings, item frames, rotationally-unconstrained decorations.
///
/// For functional blocks (workbench, chest, furnace), use BlockDef with an `interaction` component.
/// Deserialized from defs/placeables/<name>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PlaceableDef {
    /// Lang key override. If absent: auto-derived as "placeable.<ns>.<name>".
    pub display_key: Option<LangKey>,
    pub model: Option<ResourceRef>,
    /// Footprint in voxels (width, height, depth).
    pub grid_size: (u32, u32, u32),
    pub tags: Vec<TagRef>,
    pub drops: DropSpec,
    pub light_level: u8,
    pub on_use: Option<PlaceableUseAction>,
}

impl Default for PlaceableDef {
    fn default() -> Self {
        PlaceableDef {
            display_key: None,
            model: None,
            grid_size: (1, 1, 1),
            tags: vec![],
            drops: DropSpec::default(),
            light_level: 0,
            on_use: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum PlaceableUseAction {
    Custom { script: ScriptRef },
}
