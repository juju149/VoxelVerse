use serde::{Deserialize, Serialize};

// ─── Domain-typed refs ────────────────────────────────────────────────────────
// All use "namespace:name" format.
// Distinct types document intent and enable future compile-time validation
// by vv-compiler (verifies the target exists and is the correct type).

/// Reference to a block definition. Format: "namespace:name". E.g. "voxelverse:stone".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BlockRef(pub String);

/// Reference to an item definition. Format: "namespace:name". E.g. "voxelverse:iron_ingot".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemRef(pub String);

/// Reference to an entity definition. Format: "namespace:name". E.g. "voxelverse:wolf".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityRef(pub String);

/// Reference to a placeable definition. Format: "namespace:name". E.g. "voxelverse:sign".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlaceableRef(pub String);

/// Reference to a named loot table. Format: "namespace:name". E.g. "voxelverse:stone".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LootTableRef(pub String);

/// Reference to a recipe definition. Format: "namespace:name".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RecipeRef(pub String);

/// Reference to a UI theme definition. Format: "namespace:name".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiThemeRef(pub String);

// ─── Polymorphic ref ──────────────────────────────────────────────────────────

/// Generic reference for polymorphic or cross-domain content.
/// Use only when the target domain is genuinely mixed (e.g., tag values).
/// Prefer domain-typed refs above in all other contexts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentRef(pub String);

// ─── System refs ──────────────────────────────────────────────────────────────

/// Reference to a namespaced tag. Format: "namespace:tag_name".
/// Tags are ALWAYS namespaced. Never use bare names like "solid".
/// Example: "voxelverse:solid", "mymod:flammable".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TagRef(pub String);

/// Localization key. Format: "category.namespace.name".
/// Example: "block.voxelverse.stone", "item.voxelverse.iron_ingot".
/// If absent in a schema, the compiler auto-derives it from the file path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LangKey(pub String);

/// Logical reference to a media resource (texture, sound, model).
/// Format: "namespace:name". The loader resolves to the actual file based on context.
/// Example: in BlockRenderDef, "voxelverse:stone" → resources/textures/blocks/stone.png.
/// Never put a raw disk path here.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResourceRef(pub String);

/// Reference to a script resource. Format: "namespace:name".
/// The script runtime and binding model are intentionally outside vv-schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScriptRef(pub String);
