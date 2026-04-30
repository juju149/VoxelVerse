use crate::common::tool::ToolKind;
use crate::common::{LangKey, ResourceRef, RgbColor, ScriptRef, TagRef};
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
    pub render: BlockRenderDef,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockRenderDef {
    /// Fallback color for LOD and degraded rendering.
    pub color: RgbColor,
    pub roughness: f32,
    pub translucent: bool,
    /// Emitted light level (0–15).
    pub emits_light: u8,
    pub emission: Emission,
    pub texture: TextureLayout,
    /// Logical texture resources used by `texture`.
    ///
    /// Empty means the renderer should keep using the fallback color.
    /// References are logical `namespace:name` resources, not filesystem paths.
    #[serde(default)]
    pub textures: BlockTextureRefs,
    /// Dynamic tint (grass, leaves, water based on biome).
    pub tint: TintMode,
    /// Stylized material controls consumed by the renderer to reduce visible
    /// tiling while preserving the block's authored identity.
    pub material: StylizedMaterialDef,
    /// Custom model override. Absent = standard cube.
    #[serde(default)]
    pub model: Option<ResourceRef>,
}

impl Default for BlockRenderDef {
    fn default() -> Self {
        BlockRenderDef {
            color: RgbColor::default(),
            roughness: 0.7,
            translucent: false,
            emits_light: 0,
            emission: Emission::default(),
            texture: TextureLayout::Single,
            textures: BlockTextureRefs::default(),
            tint: TintMode::None,
            material: StylizedMaterialDef::default(),
            model: None,
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Emission {
    pub emits: bool,
    pub intensity: f32,
}

impl Default for Emission {
    fn default() -> Self {
        Emission {
            emits: false,
            intensity: 0.0,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StylizedMaterialDef {
    pub visual_type: VisualMaterialType,
    pub secondary_color: Option<RgbColor>,
    pub texture_influence: f32,
    pub block_variation: f32,
    pub face_variation: f32,
    pub macro_variation: f32,
    pub detail_strength: f32,
}

impl Default for StylizedMaterialDef {
    fn default() -> Self {
        Self {
            visual_type: VisualMaterialType::Generic,
            secondary_color: None,
            texture_influence: 1.0,
            block_variation: 0.08,
            face_variation: 0.04,
            macro_variation: 0.05,
            detail_strength: 0.03,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualMaterialType {
    Generic,
    Grass,
    Dirt,
    Snow,
    Stone,
    Sand,
    Wood,
    Leaves,
    Ice,
    CutStone,
    Planks,
    Ore,
    Water,
}

impl Default for VisualMaterialType {
    fn default() -> Self {
        VisualMaterialType::Generic
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
    pub render: BlockRenderDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateCondition {
    pub property: String,
    pub value: String,
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
