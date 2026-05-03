use std::collections::HashMap;

use vv_ui::{UiIconId, UiImageId, UiRect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiAtlasRegion {
    pub uv: UiRect,
}

impl UiAtlasRegion {
    pub fn new(uv: UiRect) -> Self {
        Self { uv }
    }

    pub fn full() -> Self {
        Self {
            uv: UiRect::new(0.0, 0.0, 1.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UiAtlas {
    images: HashMap<UiImageId, UiAtlasRegion>,
    icons: HashMap<UiIconId, UiAtlasRegion>,
}

impl UiAtlas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_image(&mut self, id: UiImageId, region: UiAtlasRegion) {
        self.images.insert(id, region);
    }

    pub fn register_icon(&mut self, id: UiIconId, region: UiAtlasRegion) {
        self.icons.insert(id, region);
    }

    pub fn image(&self, id: UiImageId) -> Option<UiAtlasRegion> {
        self.images.get(&id).copied()
    }

    pub fn icon(&self, id: UiIconId) -> Option<UiAtlasRegion> {
        self.icons.get(&id).copied()
    }
}
