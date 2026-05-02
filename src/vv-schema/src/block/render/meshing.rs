use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockMeshingDef {
    pub render_mode: RenderMode,
    pub occludes: bool,
    pub greedy_merge: bool,
    pub casts_shadow: bool,
    pub receives_ao: bool,
}

pub type RawBlockMeshingDef = BlockMeshingDef;

impl Default for BlockMeshingDef {
    fn default() -> Self {
        Self {
            render_mode: RenderMode::Opaque,
            occludes: true,
            greedy_merge: true,
            casts_shadow: true,
            receives_ao: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    Opaque,
    Cutout,
    Transparent,
    Additive,
}

impl Default for RenderMode {
    fn default() -> Self {
        Self::Opaque
    }
}
