use crate::RawContentSet;

#[derive(Debug, Clone)]
pub struct LoadedPack {
    pub content: RawContentSet,
}

#[derive(Debug, Clone, Default)]
pub struct PackLoadOrder {
    packs: Vec<LoadedPack>,
}

impl PackLoadOrder {
    pub fn new(packs: Vec<LoadedPack>) -> Self {
        Self { packs }
    }

    pub fn packs(&self) -> &[LoadedPack] {
        &self.packs
    }

    pub fn into_packs(self) -> Vec<LoadedPack> {
        self.packs
    }
}
