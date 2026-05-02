use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockPlacementDef {
    pub allowed_faces: PlacementFaces,
    pub replaceable: bool,
    pub requires_support: SupportRequirement,
    pub orientation: OrientationMode,
    pub auto_connect: bool,
}

impl Default for BlockPlacementDef {
    fn default() -> Self {
        Self {
            allowed_faces: PlacementFaces::OnSolid,
            replaceable: false,
            requires_support: SupportRequirement::None,
            orientation: OrientationMode::None,
            auto_connect: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementFaces {
    Any,
    OnSolid,
    OnlyFloor,
    OnlyCeiling,
    OnlyWall,
}

impl Default for PlacementFaces {
    fn default() -> Self {
        Self::OnSolid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportRequirement {
    None,
    SolidFloor,
    SolidWall,
    SolidCeiling,
}

impl Default for SupportRequirement {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrientationMode {
    None,
    Facing,
    Cardinal4,
    Cardinal6,
}

impl Default for OrientationMode {
    fn default() -> Self {
        Self::None
    }
}
