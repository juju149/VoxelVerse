use vv_core::BlockId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InteractionTarget {
    pub block: BlockId,
    pub distance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MiningState {
    pub target: Option<BlockId>,
    pub progress: f32,
}

impl MiningState {
    pub fn idle() -> Self {
        Self {
            target: None,
            progress: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.target = None;
        self.progress = 0.0;
    }

    pub fn advance(&mut self, target: BlockId, hardness: f32, dt: f32, base_speed: f32) -> bool {
        if self.target != Some(target) {
            self.target = Some(target);
            self.progress = 0.0;
        }

        let break_time = (hardness / base_speed.max(0.001)).max(0.05);
        self.progress = (self.progress + dt / break_time).clamp(0.0, 1.0);
        self.progress >= 1.0
    }
}
