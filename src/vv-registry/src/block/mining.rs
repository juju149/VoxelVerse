use crate::CompiledToolKind;

#[derive(Debug, Clone, Copy)]
pub struct CompiledBlockMining {
    pub hardness: f32,
    pub tool: CompiledToolKind,
    pub tool_tier_min: u8,
    pub drop_xp: u8,
}
