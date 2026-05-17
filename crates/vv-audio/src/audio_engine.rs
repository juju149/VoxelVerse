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
    diagnostics: AudioDiagnostics,
}

struct AudioOutput {
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

/// Logical channel grouping events whose cooldowns should not interfere with
/// each other. A `BlockBreak` must not be silenced by a recent `ToolSwing`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundChannel {
    /// Per-tick mining-hit feedback.
    Hit,
    /// Block destruction one-shot.
    Break,
    /// Block placement one-shot.
    Place,
    /// Tool swing whoosh.
    Swing,
    /// Reserved for upcoming footstep events.
    #[allow(dead_code)]
    Footstep,
    /// Reserved for UI clicks / inventory feedback.
    #[allow(dead_code)]
    Ui,
    /// Reserved for weather ambience triggers.
    #[allow(dead_code)]
    Weather,
}

impl SoundChannel {
    fn slot(self) -> usize {
        match self {
            SoundChannel::Hit => 0,
            SoundChannel::Break => 1,
            SoundChannel::Place => 2,
            SoundChannel::Swing => 3,
            SoundChannel::Footstep => 4,
            SoundChannel::Ui => 5,
            SoundChannel::Weather => 6,
        }
    }

    pub const COUNT: usize = 7;
}

/// Per-channel min-tick cooldown so a recent `Swing` cannot silence an
/// incoming `Hit` (or vice-versa). One global `tick` counter still advances
/// every `play` call so cooldowns are measured in calls, not wall time.
#[derive(Clone, Copy, Debug, Default)]
struct SoundLimiter {
    tick: u64,
    last_played: [u64; SoundChannel::COUNT],
}

impl SoundLimiter {
    fn allow(&mut self, channel: SoundChannel, min_ticks: u64) -> bool {
        let last = self.last_played[channel.slot()];
        if self.tick.saturating_sub(last) <= min_ticks {
            return false;
        }
        self.last_played[channel.slot()] = self.tick;
        true
    }
}

/// Live audio telemetry. Mirrors the structured-counters pattern used by
/// `WorldgenStats` so the dev overlay can poll without log scraping.
#[derive(Clone, Debug, Default)]
pub struct AudioDiagnostics {
    pub voices_started: u64,
    pub voices_throttled: u64,
    pub file_open_errors: u64,
    pub decode_errors: u64,
    pub play_errors: u64,
    pub output_unavailable_drops: u64,
    pub last_error: Option<String>,
}

#[derive(Clone, Copy, Debug)]
enum AudioErrorKind {
    FileOpen,
    Decode,
    Play,
}

impl AudioDiagnostics {
    fn record_error(&mut self, kind: AudioErrorKind, msg: String) {
        match kind {
            AudioErrorKind::FileOpen => self.file_open_errors += 1,
            AudioErrorKind::Decode => self.decode_errors += 1,
            AudioErrorKind::Play => self.play_errors += 1,
        }
        // Surface the first occurrence to stderr so dev sessions catch it,
        // then keep counting silently if the same error repeats.
        if self.last_error.as_deref() != Some(msg.as_str()) {
            eprintln!("[vv-audio] {msg}");
        }
        self.last_error = Some(msg);
    }
}

#[derive(Clone, Copy, Debug)]
struct AudioRng {
    state: u64,
}

impl AudioEngine {
    pub fn new(core_pack_dir: &Path) -> Self {
        let output = match OutputStream::try_default() {
            Ok((stream, handle)) => Some(AudioOutput {
                _stream: stream,
                handle,
            }),
            Err(err) => {
                eprintln!("[vv-audio] no output device: {err}");
                None
            }
        };
        Self {
            bank: SoundBank::from_core_pack_dir(core_pack_dir),
            output,
            limiter: SoundLimiter::default(),
            rng: AudioRng {
                state: 0x5EED_5EED_1234_5678,
            },
            diagnostics: AudioDiagnostics::default(),
        }
    }

    pub fn diagnostics(&self) -> &AudioDiagnostics {
        &self.diagnostics
    }

    pub fn play(&mut self, event: SoundEvent) {
        self.limiter.tick = self.limiter.tick.saturating_add(1);

        if self.output.is_none() {
            self.diagnostics.output_unavailable_drops += 1;
            return;
        }

        let (channel, min_ticks) = channel_for(event);
        if !self.limiter.allow(channel, min_ticks) {
            self.diagnostics.voices_throttled += 1;
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

    fn play_path(&mut self, path: &Path, volume: f32, pitch: f32) {
        let Some(output) = &self.output else {
            self.diagnostics.output_unavailable_drops += 1;
            return;
        };
        let file = match File::open(path) {
            Ok(f) => f,
            Err(err) => {
                let msg = format!("open {} failed: {err}", path.display());
                self.diagnostics.record_error(AudioErrorKind::FileOpen, msg);
                return;
            }
        };
        let source = match Decoder::new(BufReader::new(file)) {
            Ok(s) => s,
            Err(err) => {
                let msg = format!("decode {} failed: {err}", path.display());
                self.diagnostics.record_error(AudioErrorKind::Decode, msg);
                return;
            }
        };
        match output
            .handle
            .play_raw(source.speed(pitch).amplify(volume).convert_samples())
        {
            Ok(()) => {
                self.diagnostics.voices_started += 1;
            }
            Err(err) => {
                let msg = format!("play_raw failed: {err}");
                self.diagnostics.record_error(AudioErrorKind::Play, msg);
            }
        }
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

/// Map a `SoundEvent` to its channel + per-channel cooldown in ticks.
/// `min_ticks = 0` means every call is allowed; higher values throttle.
fn channel_for(event: SoundEvent) -> (SoundChannel, u64) {
    match event {
        SoundEvent::MineHit { .. } => (SoundChannel::Hit, 1),
        SoundEvent::BlockBreak { .. } => (SoundChannel::Break, 0),
        SoundEvent::BlockPlace { .. } => (SoundChannel::Place, 0),
        SoundEvent::ToolSwing { .. } => (SoundChannel::Swing, 2),
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
    use super::{channel_for, clip_key, SoundChannel, SoundLimiter};
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

    #[test]
    fn swing_and_hit_do_not_share_cooldown() {
        let mut lim = SoundLimiter { tick: 3, ..Default::default() };
        assert!(lim.allow(SoundChannel::Swing, 2));
        // Tick = 4 — hit must not be throttled by the recent swing.
        lim.tick = 4;
        assert!(lim.allow(SoundChannel::Hit, 1));
    }

    #[test]
    fn within_channel_cooldown_throttles() {
        let mut lim = SoundLimiter { tick: 5, ..Default::default() };
        assert!(lim.allow(SoundChannel::Swing, 2));
        // Same channel, tick within cooldown → blocked.
        lim.tick = 6;
        assert!(!lim.allow(SoundChannel::Swing, 2));
        // Outside cooldown → allowed again.
        lim.tick = 9;
        assert!(lim.allow(SoundChannel::Swing, 2));
    }

    #[test]
    fn channel_for_routes_events_to_expected_channels() {
        assert_eq!(
            channel_for(SoundEvent::ToolSwing { strength: 1.0 }).0,
            SoundChannel::Swing
        );
        assert_eq!(
            channel_for(SoundEvent::MineHit {
                sound_kind: SoundKind::Stone,
                strength: 0.5
            })
            .0,
            SoundChannel::Hit
        );
        assert_eq!(
            channel_for(SoundEvent::BlockBreak {
                sound_kind: SoundKind::Stone,
                strength: 1.0
            })
            .0,
            SoundChannel::Break
        );
        assert_eq!(
            channel_for(SoundEvent::BlockPlace {
                sound_kind: SoundKind::Stone
            })
            .0,
            SoundChannel::Place
        );
    }
}
