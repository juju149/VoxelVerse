use serde::{Deserialize, Serialize};

use super::{
    BlockEnvironmentDef, BlockFaceOverrides, BlockLightingDef, BlockMaterialDef, BlockMeshingDef,
    BlockShapeDef, BlockSurfaceProgramDef, BlockVariationDef,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockRenderPatchDef {
    pub material: Option<BlockMaterialPatch>,
    pub lighting: Option<BlockLightingPatch>,
    pub shape: Option<BlockShapePatch>,
    pub program: Option<BlockSurfaceProgramDef>,
    pub variation: Option<BlockVariationPatch>,
    pub environment: Option<BlockEnvironmentPatch>,
    pub faces: Option<BlockFaceOverridesPatch>,
    pub meshing: Option<BlockMeshingPatch>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockMaterialPatch {
    pub kind: Option<super::BlockMaterialKind>,
    pub base_color: Option<crate::common::HexColor>,
    pub palette: Option<Vec<crate::common::HexColor>>,
    pub roughness: Option<f32>,
    pub metallic: Option<f32>,
    pub alpha: Option<f32>,
    pub tint: Option<super::TintMode>,
}

pub type RawBlockSurfacePatch = BlockMaterialPatch;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockLightingPatch {
    pub emission: Option<Option<crate::common::HexColor>>,
    pub emits_light: Option<u8>,
}

pub type RawBlockLightingPatch = BlockLightingPatch;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockShapePatch {
    pub shape: Option<super::BlockShape>,
    pub profile: Option<super::BlockShapeProfile>,
    pub bevel: Option<f32>,
    pub roundness: Option<f32>,
    pub face_depth: Option<f32>,
    pub normal_strength: Option<f32>,
}

pub type RawBlockGeometryPatch = BlockShapePatch;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockVariationPatch {
    pub per_voxel_tint: Option<f32>,
    pub per_face_tint: Option<f32>,
    pub macro_noise_scale: Option<f32>,
    pub macro_noise_strength: Option<f32>,
    pub micro_noise_scale: Option<f32>,
    pub micro_noise_strength: Option<f32>,
    pub edge_darkening: Option<f32>,
    pub ao_influence: Option<f32>,
}

pub type RawBlockVisualVariationPatch = BlockVariationPatch;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockEnvironmentPatch {
    pub biome_tint_strength: Option<f32>,
    pub wetness_response: Option<f32>,
    pub snow_response: Option<f32>,
    pub dust_response: Option<f32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockFaceOverridesPatch {
    pub top: Option<Option<super::BlockFaceOverride>>,
    pub side: Option<Option<super::BlockFaceOverride>>,
    pub bottom: Option<Option<super::BlockFaceOverride>>,
    pub north: Option<Option<super::BlockFaceOverride>>,
    pub south: Option<Option<super::BlockFaceOverride>>,
    pub east: Option<Option<super::BlockFaceOverride>>,
    pub west: Option<Option<super::BlockFaceOverride>>,
}

pub type RawBlockFaceVisualsPatch = BlockFaceOverridesPatch;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockMeshingPatch {
    pub render_mode: Option<super::RenderMode>,
    pub occludes: Option<bool>,
    pub greedy_merge: Option<bool>,
    pub casts_shadow: Option<bool>,
    pub receives_ao: Option<bool>,
}

pub type RawBlockMeshingPatch = BlockMeshingPatch;

#[allow(dead_code)]
fn _keep_patch_imports_alive(
    _: Option<BlockMaterialDef>,
    _: Option<BlockLightingDef>,
    _: Option<BlockShapeDef>,
    _: Option<BlockSurfaceProgramDef>,
    _: Option<BlockVariationDef>,
    _: Option<BlockEnvironmentDef>,
    _: Option<BlockFaceOverrides>,
    _: Option<BlockMeshingDef>,
) {
}
