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
