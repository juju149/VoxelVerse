#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledMaterialPhase {
    Solid,
    Liquid,
    Passable,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledBlockPhysics {
    pub phase: CompiledMaterialPhase,
    pub density: f32,
    pub friction: f32,
    pub drag: f32,
}
