//! Runtime-tunable rendering quality knobs.
//!
//! Single responsibility: own the values that the shader consumes for its
//! optional cost paths (shadow PCF kernel size, triplanar grain).  Packed
//! as a float bitmask written to `global.render_params.y` in the frame
//! uniform.  The same bits are also stored in `global.cam_pos.w`;
//! `render_params.y` is the canonical source.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PcfQuality {
    /// 1 tap — no filtering, hard shadow edges.
    Low,
    /// 5-tap cross — soft edges at low cost.
    Medium,
    /// 13-tap Poisson — soft shadows, expensive on large terrain views.
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
    /// 5-tap box blur in final_composite — cheaper than FXAA, reduces aliasing
    /// on geometry edges without the full FXAA pass.
    pub soft_aa: bool,
    /// Over-bright glow via single highlight lift in final_composite — not a
    /// real multi-pass bloom, just a one-tap overbright clamp.
    pub highlight_lift: bool,
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
            soft_aa: false,
            highlight_lift: false,
            cloud_steps: 0,
        }
    }
}

impl QualitySettings {
    /// Pack settings into a single f32 written to `global.render_params.y`
    /// (also mirrored to `global.cam_pos.w` for bind-group stability).
    /// Bit 0 = triplanar; bits 1-2 = pcf level (0=1tap,1=5tap,2=13tap);
    /// bit 3 = color-only mode; bit 4 = volumetric fog;
    /// bit 5 = volumetric clouds; bit 6 = soft AA; bit 7 = highlight lift.
    pub fn pack(self) -> f32 {
        let bits = (self.triplanar_grain as u32)
            | (self.pcf.level_bits() << 1)
            | ((self.color_only_mode as u32) << 3)
            | ((self.volumetric_fog as u32) << 4)
            | ((self.volumetric_clouds as u32) << 5)
            | ((self.soft_aa as u32) << 6)
            | ((self.highlight_lift as u32) << 7);
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
            soft_aa: true,
            highlight_lift: true,
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
