//! Runtime-tunable rendering quality knobs.
//!
//! Single responsibility: own the values that the shader consumes for its
//! optional cost paths (shadow PCF kernel size, triplanar grain).  Encoded
//! into the unused `cam_pos.w` slot of the global uniform to avoid touching
//! the bind-group layout and wgsl struct size.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PcfQuality {
    /// 3×3 (9 samples) — cheapest, slight banding on shadow edges.
    Low,
    /// 5×5 (25 samples) — balanced default for high-end machines.
    Medium,
    /// 7×7 (49 samples) — softest shadow edges, expensive.
    High,
}

impl PcfQuality {
    fn level_bits(self) -> u32 {
        match self {
            PcfQuality::Low => 0,
            PcfQuality::Medium => 1,
            PcfQuality::High => 2,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QualitySettings {
    pub profile: RenderQualityProfile,
    /// Enable per-fragment triplanar grain noise (3 sin() calls per fragment).
    pub triplanar_grain: bool,
    /// Shadow PCF kernel size.
    pub pcf: PcfQuality,
    /// Bypass texture sampling and shade with each block's flat base color.
    /// Used to A/B-compare textured vs. flat-color rendering performance.
    pub color_only_mode: bool,
    pub volumetric_fog: bool,
    pub volumetric_clouds: bool,
    pub fxaa: bool,
    pub bloom: bool,
    pub cloud_steps: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderQualityProfile {
    Potato,
    Balanced,
    High,
    Ultra,
}

impl Default for QualitySettings {
    /// Conservative default — favours frame-rate over visual fidelity.
    fn default() -> Self {
        Self {
            profile: RenderQualityProfile::Potato,
            triplanar_grain: false,
            pcf: PcfQuality::Low,
            color_only_mode: false,
            volumetric_fog: false,
            volumetric_clouds: false,
            fxaa: false,
            bloom: false,
            cloud_steps: 0,
        }
    }
}

impl QualitySettings {
    /// Pack settings into a single f32 written to `global.cam_pos.w`.
    /// Bit 0 = triplanar; bits 1-2 = pcf level; bit 3 = color-only mode;
    /// bit 4 = volumetric fog; bit 5 = volumetric clouds; bit 6 = FXAA;
    /// bit 7 = bloom.
    pub fn pack(self) -> f32 {
        let bits = (self.triplanar_grain as u32)
            | (self.pcf.level_bits() << 1)
            | ((self.color_only_mode as u32) << 3)
            | ((self.volumetric_fog as u32) << 4)
            | ((self.volumetric_clouds as u32) << 5)
            | ((self.fxaa as u32) << 6)
            | ((self.bloom as u32) << 7);
        bits as f32
    }
}

#[cfg(test)]
mod tests {
    use super::{PcfQuality, QualitySettings, RenderQualityProfile};
    use crate::renderer::GlobalUniform;

    #[test]
    fn quality_flags_pack_profile_features() {
        let settings = QualitySettings {
            profile: RenderQualityProfile::High,
            triplanar_grain: true,
            pcf: PcfQuality::Medium,
            color_only_mode: false,
            volumetric_fog: true,
            volumetric_clouds: true,
            fxaa: true,
            bloom: true,
            cloud_steps: 10,
        };
        let bits = settings.pack() as u32;
        assert_eq!(bits & 1, 1);
        assert_eq!((bits >> 1) & 3, 1);
        assert_ne!(bits & (1 << 4), 0);
        assert_ne!(bits & (1 << 5), 0);
        assert_ne!(bits & (1 << 6), 0);
        assert_ne!(bits & (1 << 7), 0);
    }

    #[test]
    fn global_uniform_layout_size_matches_wgsl_contract() {
        assert_eq!(std::mem::size_of::<GlobalUniform>(), 304);
    }
}
