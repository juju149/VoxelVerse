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
    /// Enable per-fragment triplanar grain noise (3 sin() calls per fragment).
    pub triplanar_grain: bool,
    /// Shadow PCF kernel size.
    pub pcf: PcfQuality,
}

impl Default for QualitySettings {
    /// Conservative default — favours frame-rate over visual fidelity.
    fn default() -> Self {
        Self {
            triplanar_grain: false,
            pcf: PcfQuality::Low,
        }
    }
}

impl QualitySettings {
    /// Pack settings into a single f32 written to `global.cam_pos.w`.
    /// Bit 0 = triplanar; bits 1-2 = pcf level.
    pub fn pack(self) -> f32 {
        let bits = (self.triplanar_grain as u32) | (self.pcf.level_bits() << 1);
        bits as f32
    }
}
