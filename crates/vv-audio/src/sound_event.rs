#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SoundKind {
    #[default]
    None,
    Grass,
    Stone,
    Wood,
    Sand,
    Snow,
    Dirt,
}

impl From<vv_pack_compiler::CompiledSoundKind> for SoundKind {
    fn from(kind: vv_pack_compiler::CompiledSoundKind) -> Self {
        use vv_pack_compiler::CompiledSoundKind;
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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SoundEvent {
    MineHit {
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
    ToolSwing {
        strength: f32,
    },
}
