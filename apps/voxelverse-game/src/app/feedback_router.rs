use vv_audio::{AudioEngine, SoundEvent, SoundKind};
use vv_pack_compiler::CompiledSoundKind;
use vv_render::{PlayerActionFeedback, Renderer};

pub(super) enum AppFeedback {
    ToolSwing {
        strength: f32,
    },
    BlockHit {
        sound_kind: SoundKind,
        strength: f32,
    },
    BlockBreak {
        sound_kind: SoundKind,
        strength: f32,
    },
    BlockPlace {
        sound_kind: SoundKind,
    },
}

pub(super) fn route_feedback(
    renderer: &mut Renderer<'_>,
    audio: &mut AudioEngine,
    feedback: AppFeedback,
) {
    match feedback {
        AppFeedback::ToolSwing { strength } => {
            renderer.notify_player_action(PlayerActionFeedback::Swing { strength });
            audio.play(SoundEvent::ToolSwing { strength });
        }
        AppFeedback::BlockHit {
            sound_kind,
            strength,
        } => {
            renderer.notify_player_action(PlayerActionFeedback::Hit { strength });
            audio.play(SoundEvent::MineHit {
                sound_kind,
                strength,
            });
        }
        AppFeedback::BlockBreak {
            sound_kind,
            strength,
        } => {
            renderer.notify_player_action(PlayerActionFeedback::Break { strength });
            audio.play(SoundEvent::BlockBreak {
                sound_kind,
                strength,
            });
        }
        AppFeedback::BlockPlace { sound_kind } => {
            renderer.notify_player_action(PlayerActionFeedback::Place);
            audio.play(SoundEvent::BlockPlace { sound_kind });
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
