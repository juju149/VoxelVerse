use super::{config::PatternedMeshConfig, kinds, PatternedCell};

#[derive(Debug, Clone)]
pub(crate) struct PatternedLayout {
    pub cells: Vec<PatternedCell>,
}

impl PatternedLayout {
    pub(crate) fn build(config: PatternedMeshConfig, face_seed: u32) -> Self {
        Self {
            cells: kinds::build_cells(config, face_seed),
        }
    }
}
