use vv_audio::{AudioEngine, SoundEvent, SoundKind};
use vv_gameplay::{BlockSoundKind, GameFeedbackEvent};
use vv_render::{PlayerActionFeedback, Renderer};

/// Forward a slice of feedback events produced by an action to the renderer and audio engine.
pub(super) fn route_feedback_events(
    renderer: &mut Renderer<'_>,
    audio: &mut AudioEngine,
    events: &[GameFeedbackEvent],
) {
    for event in events {
        match event {
            GameFeedbackEvent::ToolSwing { strength } => {
                renderer.notify_player_action(PlayerActionFeedback::Swing {
                    strength: *strength,
                });
                audio.play(SoundEvent::ToolSwing {
                    strength: *strength,
                });
            }
            GameFeedbackEvent::BlockHit {
                sound_kind,
                strength,
            } => {
                renderer.notify_player_action(PlayerActionFeedback::Hit {
                    strength: *strength,
                });
                audio.play(SoundEvent::MineHit {
                    sound_kind: to_audio_kind(*sound_kind),
                    strength: *strength,
                });
            }
            GameFeedbackEvent::BlockBreak {
                sound_kind,
                strength,
            } => {
                renderer.notify_player_action(PlayerActionFeedback::Break {
                    strength: *strength,
                });
                audio.play(SoundEvent::BlockBreak {
                    sound_kind: to_audio_kind(*sound_kind),
                    strength: *strength,
                });
            }
            GameFeedbackEvent::BlockPlace { sound_kind } => {
                renderer.notify_player_action(PlayerActionFeedback::Place);
                audio.play(SoundEvent::BlockPlace {
                    sound_kind: to_audio_kind(*sound_kind),
                });
            }
        }
    }
}

/// Convert the gameplay `BlockSoundKind` to the audio-crate `SoundKind`.
/// This conversion is the only place in the app that depends on both types.
fn to_audio_kind(kind: BlockSoundKind) -> SoundKind {
    match kind {
        BlockSoundKind::None => SoundKind::None,
        BlockSoundKind::Grass => SoundKind::Grass,
        BlockSoundKind::Stone => SoundKind::Stone,
        BlockSoundKind::Wood => SoundKind::Wood,
        BlockSoundKind::Sand => SoundKind::Sand,
        BlockSoundKind::Snow => SoundKind::Snow,
        BlockSoundKind::Dirt => SoundKind::Dirt,
    }
}
