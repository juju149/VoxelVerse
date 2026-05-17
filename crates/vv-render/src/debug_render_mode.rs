#![allow(dead_code)]

//! Debug render modes for the terrain/material pipeline.
//!
//! These modes must be implemented before adding visual complexity. They are
//! the renderer's X-ray goggles: if terrain breaks, we can see which contract
//! failed instead of guessing.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DebugRenderMode {
    #[default]
    Disabled,
    VertexColor,
    WorldNormal,
    MaterialLayer,
    Uv,
    LodAlpha,
    ChunkKind,
    ShadowFactor,
    Depth,
    WorldPositionBands,
}

impl DebugRenderMode {
    pub(crate) fn as_u32(self) -> u32 {
        match self {
            DebugRenderMode::Disabled => 0,
            DebugRenderMode::VertexColor => 1,
            DebugRenderMode::WorldNormal => 2,
            DebugRenderMode::MaterialLayer => 3,
            DebugRenderMode::Uv => 4,
            DebugRenderMode::LodAlpha => 5,
            DebugRenderMode::ChunkKind => 6,
            DebugRenderMode::ShadowFactor => 7,
            DebugRenderMode::Depth => 8,
            DebugRenderMode::WorldPositionBands => 9,
        }
    }

    pub(crate) fn is_enabled(self) -> bool {
        self != DebugRenderMode::Disabled
    }
}

#[cfg(test)]
mod tests {
    use super::DebugRenderMode;

    #[test]
    fn disabled_debug_mode_is_zero_for_wgsl_contract() {
        assert_eq!(DebugRenderMode::Disabled.as_u32(), 0);
        assert!(!DebugRenderMode::Disabled.is_enabled());
    }

    #[test]
    fn active_debug_modes_are_non_zero() {
        let modes = [
            DebugRenderMode::VertexColor,
            DebugRenderMode::WorldNormal,
            DebugRenderMode::MaterialLayer,
            DebugRenderMode::Uv,
            DebugRenderMode::LodAlpha,
            DebugRenderMode::ChunkKind,
            DebugRenderMode::ShadowFactor,
            DebugRenderMode::Depth,
            DebugRenderMode::WorldPositionBands,
        ];

        for mode in modes {
            assert!(mode.as_u32() > 0);
            assert!(mode.is_enabled());
        }
    }
}
