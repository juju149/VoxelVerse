use crate::sound_bank::SoundClipKey;
use crate::{SoundBank, SoundEvent, SoundKind};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct AudioEngine {
    bank: SoundBank,
    output: Option<AudioOutput>,
    limiter: SoundLimiter,
    rng: AudioRng,
}

struct AudioOutput {
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

#[derive(Clone, Copy, Debug)]
struct SoundLimiter {
    last_tick: u64,
    tick: u64,
}

#[derive(Clone, Copy, Debug)]
struct AudioRng {
    state: u64,
}

impl AudioEngine {
    pub fn new(core_pack_dir: &Path) -> Self {
        let output = OutputStream::try_default()
            .ok()
            .map(|(_stream, handle)| AudioOutput { _stream, handle });
        Self {
            bank: SoundBank::from_core_pack_dir(core_pack_dir),
            output,
            limiter: SoundLimiter {
                last_tick: 0,
                tick: 0,
            },
            rng: AudioRng {
                state: 0x5EED_5EED_1234_5678,
            },
        }
    }

    pub fn play(&mut self, event: SoundEvent) {
        self.limiter.tick = self.limiter.tick.saturating_add(1);
        if self.output.is_none() || !self.limiter.allow(event) {
            return;
        }

        let (key, strength) = clip_key(event);
        if key == SoundClipKey::Hit(SoundKind::None) {
            return;
        }
        let clips = self.bank.clips(key);
        if clips.is_empty() {
            return;
        }
        let index = (self.rng.next_u32() as usize) % clips.len();
        let path = clips[index].clone();
        let volume = (0.18 + 0.42 * strength.clamp(0.0, 1.5)) * self.rng.range(0.88, 1.08);
        let pitch = self.rng.range(0.92, 1.08);
        self.play_path(&path, volume, pitch);
    }

    fn play_path(&self, path: &Path, volume: f32, pitch: f32) {
        let Some(output) = &self.output else {
            return;
        };
        let Ok(file) = File::open(path) else {
            return;
        };
        let Ok(source) = Decoder::new(BufReader::new(file)) else {
            return;
        };
        let _ = output
            .handle
            .play_raw(source.speed(pitch).amplify(volume).convert_samples());
    }
}

fn clip_key(event: SoundEvent) -> (SoundClipKey, f32) {
    match event {
        SoundEvent::MineHit {
            sound_kind,
            strength,
        } => (SoundClipKey::Hit(sound_kind), strength),
        SoundEvent::BlockBreak {
            sound_kind,
            strength,
        } => (SoundClipKey::Break(sound_kind), strength),
        SoundEvent::BlockPlace { sound_kind } => (SoundClipKey::Place(sound_kind), 0.65),
        SoundEvent::ToolSwing { strength } => (SoundClipKey::Swing, strength),
    }
}

impl SoundLimiter {
    fn allow(&mut self, event: SoundEvent) -> bool {
        let min_ticks = match event {
            SoundEvent::ToolSwing { .. } => 2,
            SoundEvent::MineHit { .. } => 1,
            SoundEvent::BlockBreak { .. } | SoundEvent::BlockPlace { .. } => 0,
        };
        if self.tick.saturating_sub(self.last_tick) <= min_ticks {
            return false;
        }
        self.last_tick = self.tick;
        true
    }
}

impl AudioRng {
    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        let unit = self.next_u32() as f32 / u32::MAX as f32;
        min + (max - min) * unit
    }
}

#[cfg(test)]
mod tests {
    use super::clip_key;
    use crate::sound_bank::SoundClipKey;
    use crate::{SoundEvent, SoundKind};

    #[test]
    fn hit_event_maps_to_hit_clip() {
        assert_eq!(
            clip_key(SoundEvent::MineHit {
                sound_kind: SoundKind::Stone,
                strength: 0.5
            })
            .0,
            SoundClipKey::Hit(SoundKind::Stone)
        );
    }
}
