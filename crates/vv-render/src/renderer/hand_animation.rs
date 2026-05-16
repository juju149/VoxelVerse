#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct HandPose {
    pub offset: [f32; 2],
    pub rotation: f32,
    pub scale: f32,
}

#[derive(Clone, Debug)]
pub(super) struct HandAnimation {
    idle_seconds: f32,
    swing_seconds: f32,
    recoil_seconds: f32,
    break_accent_seconds: f32,
    swing_strength: f32,
    recoil_strength: f32,
}

impl HandAnimation {
    const SWING_DURATION: f32 = 0.24;
    const RECOIL_DURATION: f32 = 0.16;
    const BREAK_ACCENT_DURATION: f32 = 0.28;

    pub fn new() -> Self {
        Self {
            idle_seconds: 0.0,
            swing_seconds: 0.0,
            recoil_seconds: 0.0,
            break_accent_seconds: 0.0,
            swing_strength: 0.0,
            recoil_strength: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let dt = dt.max(0.0);
        self.idle_seconds += dt;
        self.swing_seconds = (self.swing_seconds - dt).max(0.0);
        self.recoil_seconds = (self.recoil_seconds - dt).max(0.0);
        self.break_accent_seconds = (self.break_accent_seconds - dt).max(0.0);
    }

    pub fn swing(&mut self, strength: f32) {
        self.swing_seconds = Self::SWING_DURATION;
        self.swing_strength = strength.clamp(0.1, 1.4);
    }

    pub fn hit_recoil(&mut self, strength: f32) {
        self.recoil_seconds = Self::RECOIL_DURATION;
        self.recoil_strength = strength.clamp(0.1, 1.4);
    }

    pub fn break_accent(&mut self, strength: f32) {
        self.swing(strength.max(1.0));
        self.hit_recoil(strength.max(1.0));
        self.break_accent_seconds = Self::BREAK_ACCENT_DURATION;
    }

    pub fn pose(&self) -> HandPose {
        let idle = (self.idle_seconds * 2.4).sin() * 0.012;
        let swing_t = normalized_remaining(self.swing_seconds, Self::SWING_DURATION);
        let recoil_t = normalized_remaining(self.recoil_seconds, Self::RECOIL_DURATION);
        let accent_t = normalized_remaining(self.break_accent_seconds, Self::BREAK_ACCENT_DURATION);

        let swing_arc = (std::f32::consts::PI * swing_t).sin() * self.swing_strength;
        let recoil = (std::f32::consts::PI * recoil_t).sin() * self.recoil_strength;
        let accent = (std::f32::consts::PI * accent_t).sin();

        HandPose {
            offset: [
                -0.02 * swing_arc + 0.015 * accent,
                idle - 0.12 * swing_arc + 0.06 * recoil - 0.025 * accent,
            ],
            rotation: -0.55 * swing_arc + 0.16 * recoil,
            scale: 1.0 + 0.035 * accent,
        }
    }
}

impl Default for HandAnimation {
    fn default() -> Self {
        Self::new()
    }
}

fn normalized_remaining(remaining: f32, duration: f32) -> f32 {
    if duration <= 0.0 || remaining <= 0.0 {
        0.0
    } else {
        (remaining / duration).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::HandAnimation;

    #[test]
    fn idle_pose_is_stable() {
        let mut animation = HandAnimation::new();
        animation.update(0.016);
        let pose = animation.pose();

        assert!(pose.scale > 0.9 && pose.scale < 1.1);
    }

    #[test]
    fn swing_changes_pose_then_returns_to_idle() {
        let mut animation = HandAnimation::new();
        animation.swing(1.0);
        let swinging = animation.pose();
        animation.update(1.0);
        let idle = animation.pose();

        assert_ne!(swinging.rotation, idle.rotation);
        assert!(idle.rotation.abs() < 0.001);
    }

    #[test]
    fn hit_recoil_affects_pose() {
        let mut animation = HandAnimation::new();
        animation.hit_recoil(1.0);

        assert!(animation.pose().offset[1] > 0.0);
    }

    #[test]
    fn break_accent_scales_hand() {
        let mut animation = HandAnimation::new();
        animation.break_accent(1.0);

        assert!(animation.pose().scale >= 1.0);
    }
}
