use crate::app::action_result::FeedbackEvent;
use vv_audio::{AudioEngine, SoundEvent, SoundKind};
use vv_pack_compiler::CompiledSoundKind;
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
                    sound_kind: *sound_kind,
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
                    sound_kind: *sound_kind,
                    strength: *strength,
                });
            }
            FeedbackEvent::BlockPlace { sound_kind } => {
                renderer.notify_player_action(PlayerActionFeedback::Place);
                audio.play(SoundEvent::BlockPlace {
                    sound_kind: *sound_kind,
                });
            }
        }
    }
}

pub(super) fn sound_kind(kind: CompiledSoundKind) -> SoundKind {
    match kind {
        CompiledSoundKind::None => SoundKind::None,
        CompiledSoundKind::Grass => SoundKind::Grass,
        CompiledSoundKind::Stone => SoundKind::Stone,
        CompiledSoundKind::Wood => SoundKind::Wood,
        CompiledSoundKind::Sand => SoundKind::Sand,
        CompiledSoundKind::Snow => SoundKind::Snow,
        CompiledSoundKind::Dirt => SoundKind::Dirt,
    }
}
