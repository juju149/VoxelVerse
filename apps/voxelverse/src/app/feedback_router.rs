use vv_audio::{AudioEngine, SoundEvent};
use vv_gameplay::GameFeedbackEvent;
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
                    sound_kind: (*sound_kind).into(),
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
                    sound_kind: (*sound_kind).into(),
                    strength: *strength,
                });
            }
            GameFeedbackEvent::BlockPlace { sound_kind } => {
                renderer.notify_player_action(PlayerActionFeedback::Place);
                audio.play(SoundEvent::BlockPlace {
                    sound_kind: (*sound_kind).into(),
                });
            }
        }
    }
}
