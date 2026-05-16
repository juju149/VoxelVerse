use crate::SoundKind;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct SoundBank {
    clips: HashMap<SoundClipKey, Vec<PathBuf>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SoundClipKey {
    Hit(SoundKind),
    Break(SoundKind),
    Place(SoundKind),
    Swing,
}

impl SoundBank {
    pub fn from_core_pack_dir(core_pack_dir: &Path) -> Self {
        let audio = core_pack_dir.join("media/audio");
        let mut bank = Self::default();
        bank.add_existing(SoundClipKey::Swing, &audio, &["sfx/abilities/swing.ogg"]);
        for kind in [
            SoundKind::Grass,
            SoundKind::Dirt,
            SoundKind::Sand,
            SoundKind::Snow,
            SoundKind::Stone,
            SoundKind::Wood,
        ] {
            for (key, files) in sound_files(kind) {
                bank.add_existing(key, &audio, files);
            }
        }
        bank
    }

    pub fn clips(&self, key: SoundClipKey) -> &[PathBuf] {
        self.clips.get(&key).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn has_clip(&self, key: SoundClipKey) -> bool {
        !self.clips(key).is_empty()
    }

    fn add_existing(&mut self, key: SoundClipKey, audio_root: &Path, rel_files: &[&str]) {
        let files: Vec<PathBuf> = rel_files
            .iter()
            .map(|rel| audio_root.join(rel))
            .filter(|path| path.exists())
            .collect();
        if !files.is_empty() {
            self.clips.entry(key).or_default().extend(files);
        }
    }
}

fn sound_files(kind: SoundKind) -> Vec<(SoundClipKey, &'static [&'static str])> {
    let step = match kind {
        SoundKind::Grass => &[
            "sfx/footsteps/stepgrass_1.ogg",
            "sfx/footsteps/stepgrass_2.ogg",
            "sfx/footsteps/stepgrass_3.ogg",
        ][..],
        SoundKind::Stone => &[
            "sfx/footsteps/stone_step_1.ogg",
            "sfx/footsteps/stone_step_2.ogg",
            "sfx/footsteps/stone_step_3.ogg",
        ],
        SoundKind::Wood => &[
            "sfx/footsteps/wood_step_1.ogg",
            "sfx/footsteps/wood_step_2.ogg",
            "sfx/footsteps/wood_step_3.ogg",
        ],
        SoundKind::Snow => &[
            "sfx/footsteps/snow_step_1.ogg",
            "sfx/footsteps/snow_step_2.ogg",
            "sfx/footsteps/snow_step_3.ogg",
        ],
        SoundKind::Dirt | SoundKind::Sand => &[
            "sfx/footsteps/stepdirt_1.ogg",
            "sfx/footsteps/stepdirt_2.ogg",
            "sfx/footsteps/stepdirt_3.ogg",
        ],
        SoundKind::None => &[],
    };
    vec![
        (
            SoundClipKey::Hit(kind),
            &["sfx/abilities/pickaxe_damage.ogg"][..],
        ),
        (
            SoundClipKey::Break(kind),
            &["sfx/abilities/pickaxe_damage_broken.ogg"][..],
        ),
        (SoundClipKey::Place(kind), step),
    ]
}

#[cfg(test)]
mod tests {
    use super::{SoundBank, SoundClipKey};
    use crate::SoundKind;
    use std::path::Path;

    #[test]
    fn missing_assets_are_allowed() {
        let bank = SoundBank::from_core_pack_dir(Path::new("__missing_pack__"));

        assert!(!bank.has_clip(SoundClipKey::Swing));
    }

    #[test]
    fn missing_kind_returns_empty_slice() {
        let bank = SoundBank::default();

        assert!(bank.clips(SoundClipKey::Hit(SoundKind::Stone)).is_empty());
    }
}
