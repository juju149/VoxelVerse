use crate::app::action_result::{BlockSoundKind, FeedbackEvent};
use vv_audio::{AudioEngine, SoundEvent, SoundKind};
use vv_render::{PlayerActionFeedback, Renderer};

/// Forward a slice of feedback events produced by an action to the renderer and audio engine.
pub(super) fn route_feedback_events(
    renderer: &mut Renderer<'_>,
    audio: &mut AudioEngine,
    events: &[FeedbackEvent],
) {
    for event in events {
        match event {
            FeedbackEvent::ToolSwing { strength } => {
                renderer.notify_player_action(PlayerActionFeedback::Swing {
                    strength: *strength,
                });
                audio.play(SoundEvent::ToolSwing {
                    strength: *strength,
                });
            }
            FeedbackEvent::BlockHit {
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
            FeedbackEvent::BlockBreak {
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
            FeedbackEvent::BlockPlace { sound_kind } => {
                renderer.notify_player_action(PlayerActionFeedback::Place);
                audio.play(SoundEvent::BlockPlace {
                    sound_kind: to_audio_kind(*sound_kind),
                });
            }
        }
    }
}

/// Convert the app-layer `BlockSoundKind` to the audio-crate `SoundKind`.
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
