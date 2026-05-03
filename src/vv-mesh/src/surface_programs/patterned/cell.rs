#[derive(Debug, Clone, Copy)]
pub(crate) struct PatternedCell {
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub depth: f32,
    pub bevel: f32,
    pub color_variation: f32,
}

impl PatternedCell {
    pub(crate) fn center(self) -> [f32; 2] {
        [
            (self.uv_min[0] + self.uv_max[0]) * 0.5,
            (self.uv_min[1] + self.uv_max[1]) * 0.5,
        ]
    }
}
