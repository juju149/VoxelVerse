use serde::Deserialize;

/// Semantic role hints that allow the engine to find a block by purpose
/// instead of by hardcoded name.  A block declares its own role in its data file.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockRole {
    /// Used as the default block placed by the player before a hotbar system exists.
    DefaultPlace,
}

/// Raw block definition as written in pack data files.
/// The block's key (e.g. "core:dirt") is derived from its file path — not stored here.
#[derive(Debug, Clone, Deserialize)]
pub struct RawBlockDef {
    pub display_name: String,
    pub solid: bool,
    /// RGB color used for close-up voxel rendering (until texture atlas is ready).
    pub color: [f32; 3],
    /// How many tool hits to break this block. 0.0 = unbreakable.
    pub hardness: f32,
    /// Optional semantic role. Lets the engine find this block by purpose without hardcoding its name.
    #[serde(default)]
    pub role: Option<BlockRole>,
}
