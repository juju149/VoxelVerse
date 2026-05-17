#![allow(dead_code)]

//! Stable Rust-side shader interface contract.
//!
//! This module is the single place where the renderer documents GPU-visible
//! binary layouts, attribute locations, material sentinels and quality bits.
//! WGSL files must mirror these values through include/interface/*.wgsl.

use crate::renderer::{GlobalUniform, LocalUniform};
use crate::types::Vertex;

pub(crate) const GLOBAL_UNIFORM_BYTES: usize = 304;
pub(crate) const LOCAL_UNIFORM_BYTES: usize = 80;
pub(crate) const VERTEX_BYTES: usize = 48;

pub(crate) const MATERIAL_INDEX_MASK: u32 = 0x0000_FFFF;
pub(crate) const VERTEX_COLOR_ONLY: u32 = 0x0000_FFFF;

pub(crate) mod vertex_location {
    pub(crate) const POSITION: u32 = 0;
    pub(crate) const UV: u32 = 1;
    pub(crate) const NORMAL: u32 = 2;
    pub(crate) const COLOR: u32 = 3;
    pub(crate) const TEX_INDEX: u32 = 4;
}

pub(crate) mod terrain_location {
    pub(crate) const UV: u32 = 0;
    pub(crate) const WORLD_NORMAL: u32 = 1;
    pub(crate) const WORLD_POS: u32 = 2;
    pub(crate) const VIEW_POS: u32 = 3;
    pub(crate) const SHADOW_POS: u32 = 4;
    pub(crate) const COLOR: u32 = 5;
    pub(crate) const PACKED_TEX_INDEX: u32 = 6;
    pub(crate) const LOD_ALPHA: u32 = 7;
}

pub(crate) mod quality_bit {
    pub(crate) const TRIPLANAR: u32 = 1 << 0;
    pub(crate) const PCF_SHIFT: u32 = 1;
    pub(crate) const PCF_MASK: u32 = 0b11;
    pub(crate) const COLOR_ONLY: u32 = 1 << 3;
    pub(crate) const VOLUMETRIC_FOG: u32 = 1 << 4;
    pub(crate) const VOLUMETRIC_CLOUDS: u32 = 1 << 5;
    pub(crate) const SOFT_AA: u32 = 1 << 6;
    pub(crate) const HIGHLIGHT_LIFT: u32 = 1 << 7;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PcfQuality, QualitySettings, RenderQualityProfile};

    #[test]
    fn shader_contract_matches_runtime_layout() {
        assert_eq!(std::mem::size_of::<GlobalUniform>(), GLOBAL_UNIFORM_BYTES);
        assert_eq!(std::mem::size_of::<LocalUniform>(), LOCAL_UNIFORM_BYTES);
        assert_eq!(std::mem::size_of::<Vertex>(), VERTEX_BYTES);
    }

    #[test]
    fn shader_contract_quality_bits_match_quality_settings() {
        let settings = QualitySettings {
            profile: RenderQualityProfile::High,
            triplanar_grain: true,
            pcf: PcfQuality::Medium,
            color_only_mode: true,
            volumetric_fog: true,
            volumetric_clouds: true,
            soft_aa: true,
            highlight_lift: true,
            cloud_steps: 8,
        };

        let bits = settings.pack() as u32;

        assert_ne!(bits & quality_bit::TRIPLANAR, 0);
        assert_eq!((bits >> quality_bit::PCF_SHIFT) & quality_bit::PCF_MASK, 1);
        assert_ne!(bits & quality_bit::COLOR_ONLY, 0);
        assert_ne!(bits & quality_bit::VOLUMETRIC_FOG, 0);
        assert_ne!(bits & quality_bit::VOLUMETRIC_CLOUDS, 0);
        assert_ne!(bits & quality_bit::SOFT_AA, 0);
        assert_ne!(bits & quality_bit::HIGHLIGHT_LIFT, 0);
    }

    #[test]
    fn shader_contract_material_sentinels_are_stable() {
        assert_eq!(MATERIAL_INDEX_MASK, 0x0000_FFFF);
        assert_eq!(VERTEX_COLOR_ONLY, 0x0000_FFFF);
    }
}
