use crate::common::tool::ToolKind;
use crate::common::{HexColor, LangKey, ResourceRef, ScriptRef, TagRef};
use crate::loot::DropSpec;
use serde::{Deserialize, Serialize};

/// Raw block definition. ID is derived from the file path: namespace:filename.
/// No `id` field, no `numeric_id`, no `name` field.
/// Deserialized from defs/blocks/<name>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockDef {
    /// Lang key override. If absent: auto-derived as "block.<ns>.<name>".
    pub display_key: Option<LangKey>,
    pub tags: Vec<TagRef>,
    pub mining: BlockMiningDef,
    pub render: RawBlockRenderDef,
    pub physics: BlockPhysicsDef,
    pub drops: DropSpec,
    /// Max stack size as an item. Default 64; use 1 for liquids.
    #[serde(default = "default_stack_max")]
    pub stack_max: u8,
    /// Block states (directions, powered, open/closed…). None = stateless.
    #[serde(default)]
    pub states: Option<BlockStatesDef>,
    /// Placement rules. None = standard full-block placement on any face.
    #[serde(default)]
    pub placement: Option<BlockPlacementDef>,
    /// Behavior when used (right-click). None = no interaction.
    #[serde(default)]
    pub interaction: Option<BlockInteractionDef>,
}

fn default_stack_max() -> u8 {
    64
}

impl Default for BlockDef {
    fn default() -> Self {
        BlockDef {
            display_key: None,
            tags: vec![],
            mining: BlockMiningDef::default(),
            render: BlockRenderDef::default(),
            physics: BlockPhysicsDef::default(),
            drops: DropSpec::default(),
            stack_max: 64,
            states: None,
            placement: None,
            interaction: None,
        }
    }
}

// ─── Mining component ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockMiningDef {
    pub hardness: f32,
    /// Tool kind imported from common::tool — same type as ItemKind::Tool.tool_type.
    pub tool: ToolKind,
    pub tool_tier_min: u8,
    pub sound_material: SoundMaterial,
    pub drop_xp: u8,
}

impl Default for BlockMiningDef {
    fn default() -> Self {
        BlockMiningDef {
            hardness: 1.0,
            tool: ToolKind::Hand,
            tool_tier_min: 0,
            sound_material: SoundMaterial::Stone,
            drop_xp: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundMaterial {
    Stone,
    Dirt,
    Gravel,
    Sand,
    Wood,
    Grass,
    Water,
    Lava,
    Glass,
    Metal,
    Cloth,
}

impl Default for SoundMaterial {
    fn default() -> Self {
        SoundMaterial::Stone
    }
}

// ─── Render component ─────────────────────────────────────────────────────────

pub type BlockRenderDef = RawBlockRenderDef;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockRenderDef {
    pub surface: RawBlockSurfaceDef,
    pub lighting: RawBlockLightingDef,
    pub geometry: RawBlockGeometryDef,
    pub surface_program: RawBlockSurfaceProgramDef,
    pub variation: RawBlockVisualVariation,
    pub environment: RawBlockEnvironmentResponseDef,
    pub procedural: RawBlockProceduralDef,
    pub faces: RawBlockFaceVisuals,
    pub details: Vec<RawBlockDetailDef>,
    pub meshing: RawBlockMeshingDef,
}

impl Default for RawBlockRenderDef {
    fn default() -> Self {
        RawBlockRenderDef {
            surface: RawBlockSurfaceDef::default(),
            lighting: RawBlockLightingDef::default(),
            geometry: RawBlockGeometryDef::default(),
            surface_program: RawBlockSurfaceProgramDef::default(),
            variation: RawBlockVisualVariation::default(),
            environment: RawBlockEnvironmentResponseDef::default(),
            procedural: RawBlockProceduralDef::default(),
            faces: RawBlockFaceVisuals::default(),
            details: Vec::new(),
            meshing: RawBlockMeshingDef::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockSurfaceDef {
    pub material: BlockMaterialRef,
    pub base_color: HexColor,
    pub palette: Vec<HexColor>,
    pub roughness: f32,
    pub metallic: f32,
    pub alpha: f32,
    pub tint: TintMode,
    pub texture_layout: TextureLayout,
    pub textures: BlockTextureRefs,
}


impl Default for RawBlockSurfaceDef {
    fn default() -> Self {
        Self {
            material: BlockMaterialRef("voxelverse:procedural_pixel".into()),
            textures: Default::default(),
            texture_layout: Default::default(),
            tint: Default::default(),
            base_color: HexColor("#8A8A8A".into()),
            palette: Vec::new(),
            roughness: 0.85,
            metallic: 0.0,
            alpha: 1.0,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockSurfaceProgramDef {
    pub kind: BlockSurfaceProgram,
    pub variant: Option<String>,
    pub scale: f32,
    pub contrast: f32,
    pub cavity_strength: f32,
    pub edge_highlight: f32,
    pub anisotropy: f32,
}

impl Default for RawBlockSurfaceProgramDef {
    fn default() -> Self {
        Self {
            kind: BlockSurfaceProgram::Flat,
            variant: None,
            scale: 1.0,
            contrast: 1.0,
            cavity_strength: 0.0,
            edge_highlight: 0.0,
            anisotropy: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockSurfaceProgram {
    Flat,
    Grass,
    Dirt,
    Stone,
    StoneBricks,
    WoodLog,
    WoodPlanks,
    Sand,
    Snow,
    Ice,
    Leaves,
    Lava,
    Crystal,
    Ore,
    Mushroom,
    Custom { program: ResourceRef },
}

impl Default for BlockSurfaceProgram {
    fn default() -> Self {
        BlockSurfaceProgram::Flat
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockLightingDef {
    pub emission: Option<HexColor>,
    /// Emitted light level (0-15).
    pub emits_light: u8,
}

impl Default for RawBlockLightingDef {
    fn default() -> Self {
        Self {
            emission: None,
            emits_light: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockGeometryDef {
    pub shape: BlockShape,
    pub profile: BlockGeometryProfile,
    pub bevel: f32,
    pub edge_roundness: f32,
    pub face_pillow: f32,
    pub silhouette_noise: f32,
    pub corner_softness: f32,
    pub normal_strength: f32,
}

impl Default for RawBlockGeometryDef {
    fn default() -> Self {
        Self {
            shape: BlockShape::Cube,
            profile: BlockGeometryProfile::HardCube,
            bevel: 0.0,
            edge_roundness: 0.0,
            face_pillow: 0.0,
            silhouette_noise: 0.0,
            corner_softness: 0.0,
            normal_strength: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockGeometryProfile {
    HardCube,
    SoftCube,
    PillowCube,
    ChunkyStone,
    StoneBrick,
    LayeredSediment,
    LeafCluster,
    OrganicBlob,
    Crystal,
    LiquidCube,
}

impl Default for BlockGeometryProfile {
    fn default() -> Self {
        BlockGeometryProfile::HardCube
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BlockMaterialRef(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BlockDetailRef(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockShape {
    Cube,
    Cross,
    Fluid,
    Custom { model: ResourceRef },
}

impl Default for BlockShape {
    fn default() -> Self {
        BlockShape::Cube
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
        RenderMode::Opaque
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockVisualVariation {
    pub per_voxel_tint: f32,
    pub per_face_tint: f32,
    pub macro_noise_scale: f32,
    pub macro_noise_strength: f32,
    pub micro_noise_scale: f32,
    pub micro_noise_strength: f32,
    pub edge_darkening: f32,
    pub ao_influence: f32,
}

impl Default for RawBlockVisualVariation {
    fn default() -> Self {
        Self {
            per_voxel_tint: 0.0,
            per_face_tint: 0.0,
            macro_noise_scale: 1.0,
            macro_noise_strength: 0.0,
            micro_noise_scale: 1.0,
            micro_noise_strength: 0.0,
            edge_darkening: 0.0,
            ao_influence: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockEnvironmentResponseDef {
    pub biome_tint_strength: f32,
    pub wetness_response: f32,
    pub snow_response: f32,
    pub dust_response: f32,
    pub slope_moss_bias: f32,
    pub cavity_dust_bias: f32,
}

impl Default for RawBlockEnvironmentResponseDef {
    fn default() -> Self {
        Self {
            biome_tint_strength: 0.0,
            wetness_response: 0.0,
            snow_response: 0.0,
            dust_response: 0.0,
            slope_moss_bias: 0.0,
            cavity_dust_bias: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockProceduralDef {
    pub grid_size: u32,
    pub face_blend: bool,
}

impl Default for RawBlockProceduralDef {
    fn default() -> Self {
        Self {
            grid_size: 10,
            face_blend: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockFaceVisuals {
    pub top: Option<RawBlockFaceVisual>,
    pub side: Option<RawBlockFaceVisual>,
    pub bottom: Option<RawBlockFaceVisual>,
    pub north: Option<RawBlockFaceVisual>,
    pub south: Option<RawBlockFaceVisual>,
    pub east: Option<RawBlockFaceVisual>,
    pub west: Option<RawBlockFaceVisual>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockFaceVisual {
    pub color_bias: Option<HexColor>,
    pub detail_bias: Vec<BlockDetailRef>,
}

impl Default for RawBlockFaceVisual {
    fn default() -> Self {
        Self {
            color_bias: None,
            detail_bias: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockDetailDef {
    pub kind: BlockDetailRef,
    pub density: f32,
    pub weight: f32,
    pub scale: f32,
    pub strength: f32,
    pub color: Option<HexColor>,
    pub min_size: f32,
    pub max_size: f32,
    pub slope_bias: f32,
    pub height_bias: f32,
    pub blend: BlockDetailBlend,
    pub target: BlockDetailTarget,
}

impl Default for RawBlockDetailDef {
    fn default() -> Self {
        Self {
            kind: BlockDetailRef("voxelverse:generic_detail".to_owned()),
            density: 0.0,
            weight: 1.0,
            scale: 1.0,
            strength: 1.0,
            color: None,
            min_size: 0.0,
            max_size: 0.0,
            slope_bias: 0.0,
            height_bias: 0.0,
            blend: BlockDetailBlend::Overlay,
            target: BlockDetailTarget::Any,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockDetailBlend {
    Overlay,
    Multiply,
    Add,
    Replace,
    Emissive,
    Cut,
}

impl Default for BlockDetailBlend {
    fn default() -> Self {
        BlockDetailBlend::Overlay
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockDetailTarget {
    Any,
    Top,
    Bottom,
    Sides,
    North,
    South,
    East,
    West,
    UpFacing,
    DownFacing,
    Vertical,
}

impl Default for BlockDetailTarget {
    fn default() -> Self {
        BlockDetailTarget::Any
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockMeshingDef {
    pub render_mode: RenderMode,
    pub occludes: bool,
    pub greedy_merge: bool,
    pub casts_shadow: bool,
    pub receives_ao: bool,
}

impl Default for RawBlockMeshingDef {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureLayout {
    /// One texture applied to all faces.
    Single,
    /// Six textures: top, bottom, north, south, east, west.
    Sides,
    /// Textures referenced via resources/.
    Custom,
}

impl Default for TextureLayout {
    fn default() -> Self {
        TextureLayout::Single
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockTextureRefs {
    /// Texture used by `single`, and fallback for unspecified faces.
    pub single: Option<ResourceRef>,
    /// Shared side texture used by `sides`.
    pub side: Option<ResourceRef>,
    pub top: Option<ResourceRef>,
    pub bottom: Option<ResourceRef>,
    pub north: Option<ResourceRef>,
    pub south: Option<ResourceRef>,
    pub east: Option<ResourceRef>,
    pub west: Option<ResourceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TintMode {
    None,
    GrassColor,
    FoliageColor,
    WaterColor,
}

impl Default for TintMode {
    fn default() -> Self {
        TintMode::None
    }
}

// ─── Physics component ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockPhysicsDef {
    pub phase: MaterialPhase,
    pub density: f32,
    pub collider: ColliderShape,
    /// Selection highlight shape. None = same as collider.
    #[serde(default)]
    pub selection: Option<SelectionBox>,
    /// Friction coefficient (0.0 = ice, 1.0 = standard).
    pub friction: f32,
    /// Drag for liquids (slows entities passing through).
    pub drag: f32,
}

impl Default for BlockPhysicsDef {
    fn default() -> Self {
        BlockPhysicsDef {
            phase: MaterialPhase::Solid,
            density: 1.5,
            collider: ColliderShape::Full,
            selection: None,
            friction: 1.0,
            drag: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialPhase {
    Solid,
    Liquid,
    /// Walkthrough (flowers, tall grass, etc.).
    Passable,
}

impl Default for MaterialPhase {
    fn default() -> Self {
        MaterialPhase::Solid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColliderShape {
    /// Full 1×1×1 voxel collider.
    Full,
    /// No collision.
    None,
    /// Custom AABB (for slabs, stairs, etc.).
    Aabb { min: [f32; 3], max: [f32; 3] },
}

impl Default for ColliderShape {
    fn default() -> Self {
        ColliderShape::Full
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SelectionBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

// ─── States component ─────────────────────────────────────────────────────────

/// Block states (e.g. facing=north, powered=true, open=false).
/// Used for directional blocks, doors, levers, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockStatesDef {
    pub properties: Vec<StateProperty>,
    /// Render overrides per state value combination.
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockRenderPatchDef {
    pub surface: Option<RawBlockSurfacePatch>,
    pub lighting: Option<RawBlockLightingPatch>,
    pub geometry: Option<RawBlockGeometryPatch>,
    pub variation: Option<RawBlockVisualVariationPatch>,
    pub procedural: Option<RawBlockProceduralPatch>,
    pub faces: Option<RawBlockFaceVisualsPatch>,
    pub details: Option<Vec<RawBlockDetailDef>>,
    pub meshing: Option<RawBlockMeshingPatch>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockSurfacePatch {
    pub material: Option<BlockMaterialRef>,
    pub base_color: Option<HexColor>,
    pub palette: Option<Vec<HexColor>>,
    pub roughness: Option<f32>,
    pub metallic: Option<f32>,
    pub alpha: Option<f32>,
    pub tint: Option<TintMode>,
    pub texture_layout: Option<TextureLayout>,
    pub textures: Option<BlockTextureRefs>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockLightingPatch {
    pub emission: Option<Option<HexColor>>,
    pub emits_light: Option<u8>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockGeometryPatch {
    pub shape: Option<BlockShape>,
    pub bevel: Option<f32>,
    pub normal_strength: Option<f32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockVisualVariationPatch {
    pub per_voxel_tint: Option<f32>,
    pub per_face_tint: Option<f32>,
    pub macro_noise_scale: Option<f32>,
    pub macro_noise_strength: Option<f32>,
    pub micro_noise_scale: Option<f32>,
    pub micro_noise_strength: Option<f32>,
    pub edge_darkening: Option<f32>,
    pub ao_influence: Option<f32>,
    pub biome_tint_strength: Option<f32>,
    pub wetness_response: Option<f32>,
    pub snow_response: Option<f32>,
    pub dust_response: Option<f32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockProceduralPatch {
    pub grid_size: Option<u32>,
    pub face_blend: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockFaceVisualsPatch {
    pub top: Option<Option<RawBlockFaceVisual>>,
    pub side: Option<Option<RawBlockFaceVisual>>,
    pub bottom: Option<Option<RawBlockFaceVisual>>,
    pub north: Option<Option<RawBlockFaceVisual>>,
    pub south: Option<Option<RawBlockFaceVisual>>,
    pub east: Option<Option<RawBlockFaceVisual>>,
    pub west: Option<Option<RawBlockFaceVisual>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawBlockMeshingPatch {
    pub render_mode: Option<RenderMode>,
    pub occludes: Option<bool>,
    pub greedy_merge: Option<bool>,
    pub casts_shadow: Option<bool>,
    pub receives_ao: Option<bool>,
}

// ─── Placement component ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockPlacementDef {
    pub allowed_faces: PlacementFaces,
    /// This block can be replaced when placing another block on the same position.
    pub replaceable: bool,
    pub requires_support: SupportRequirement,
    /// Automatic rotation when placing.
    pub orientation: OrientationMode,
    /// Auto-connects to neighboring blocks of the same kind (fences, glass panes, etc.).
    pub auto_connect: bool,
}

impl Default for BlockPlacementDef {
    fn default() -> Self {
        BlockPlacementDef {
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
        PlacementFaces::OnSolid
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
        SupportRequirement::None
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
        OrientationMode::None
    }
}

// ─── Interaction component ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockInteractionDef {
    pub on_use: BlockUseAction,
    /// If true, Shift+right-click ignores the interaction and lets the player place a block.
    #[serde(default)]
    pub sneaking_skips: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BlockUseAction {
    /// Opens a crafting grid (workbench, etc.).
    CraftingGrid { width: u8, height: u8 },
    /// Opens a smelting interface with a fuel slot (furnace, etc.).
    Smelting,
    /// Opens a storage inventory (chest, etc.).
    Storage { slots: u32, rows: u8 },
    /// Calls a script. The script engine and compiled bindings live outside vv-schema.
    Custom { script: ScriptRef },
}



