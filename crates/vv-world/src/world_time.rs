#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldTime {
    elapsed_seconds: f32,
    day_length_seconds: f32,
    time_scale: f32,
    paused: bool,
}

impl Default for WorldTime {
    fn default() -> Self {
        Self {
            elapsed_seconds: 0.0,
            day_length_seconds: 1_200.0,
            time_scale: 1.0,
            paused: false,
        }
    }
}

impl WorldTime {
    pub fn new(day_length_seconds: f32, start_phase: f32) -> Self {
        let mut time = Self::default();
        time.set_day_length_seconds(day_length_seconds);
        time.set_day_phase(start_phase);
        time
    }

    pub fn tick(&mut self, dt_seconds: f32) {
        if self.paused {
            return;
        }
        self.elapsed_seconds += dt_seconds.max(0.0) * self.time_scale.max(0.0);
    }

    pub fn elapsed_seconds(self) -> f32 {
        self.elapsed_seconds
    }

    pub fn day_length_seconds(self) -> f32 {
        self.day_length_seconds
    }

    pub fn day_phase(self) -> f32 {
        (self.elapsed_seconds / self.day_length_seconds).fract()
    }

    pub fn is_paused(self) -> bool {
        self.paused
    }

    pub fn set_elapsed_seconds(&mut self, elapsed_seconds: f32) {
        self.elapsed_seconds = elapsed_seconds.max(0.0);
    }

    pub fn set_fixed_elapsed_seconds(&mut self, elapsed_seconds: f32) {
        self.set_elapsed_seconds(elapsed_seconds);
        self.paused = true;
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn set_time_scale(&mut self, time_scale: f32) {
        self.time_scale = time_scale.max(0.0);
    }

    pub fn set_day_length_seconds(&mut self, day_length_seconds: f32) {
        self.day_length_seconds = day_length_seconds.max(1.0);
    }

    pub fn set_day_phase(&mut self, phase: f32) {
        self.elapsed_seconds = phase.rem_euclid(1.0) * self.day_length_seconds;
    }
}

#[cfg(test)]
mod tests {
    use super::WorldTime;

    #[test]
    fn tick_advances_scaled_time() {
        let mut time = WorldTime::new(100.0, 0.25);
        time.set_time_scale(2.0);
        time.tick(10.0);

        assert_eq!(time.elapsed_seconds(), 45.0);
        assert!((time.day_phase() - 0.45).abs() < 0.0001);
    }

    #[test]
    fn fixed_time_does_not_tick() {
        let mut time = WorldTime::default();
        time.set_fixed_elapsed_seconds(180.0);
        time.tick(60.0);

        assert_eq!(time.elapsed_seconds(), 180.0);
        assert!(time.is_paused());
    }
}
