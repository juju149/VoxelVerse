use vv_gameplay::Player;
use vv_pack_compiler::CompiledPlanet;
use vv_render::{QualitySettings, RenderQualityProfile};
use vv_world::PlanetData;

#[derive(Clone, Copy)]
pub struct GoldenScene {
    pub seed: u32,
    pub resolution: u32,
    pub fixed_elapsed_secs: f32,
    pub quality: QualitySettings,
}

impl GoldenScene {
    pub const DEFAULT: Self = Self {
        seed: 0x51C0_1D01,
        resolution: 512,
        fixed_elapsed_secs: 180.0,
        quality: QualitySettings {
            profile: RenderQualityProfile::Balanced,
            triplanar_grain: false,
            pcf: vv_render::PcfQuality::Low,
            color_only_mode: false,
            volumetric_fog: true,
            volumetric_clouds: false,
            soft_aa: true,
            highlight_lift: false,
            cloud_steps: 6,
        },
    };

    pub fn apply_planet(self, mut planet: CompiledPlanet) -> CompiledPlanet {
        planet.seed = self.seed;
        planet.resolution = self.resolution;
        planet.surface_layer = self.resolution / 2;
        planet
    }

    pub fn spawn_player(self, player: &mut Player, planet: &PlanetData) {
        player.spawn(planet.spawn_position());
        player.cam_pitch = -0.18;
    }

    pub fn apply_time(self, planet: &mut PlanetData) {
        planet.set_fixed_elapsed_seconds(self.fixed_elapsed_secs);
    }
}

pub fn golden_scene_enabled() -> bool {
    std::env::args().any(|arg| arg == "--golden-scene")
        || std::env::var("VV_GOLDEN_SCENE")
            .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

#[cfg(test)]
mod tests {
    use super::GoldenScene;
    use crate::app::content_bootstrap::load_core_content;
    use vv_world::PlanetData;

    #[test]
    fn golden_scene_locks_planet_seed_resolution_and_quality() {
        let content = load_core_content().expect("core pack must load in tests");
        let scene = GoldenScene::DEFAULT;
        let planet = scene.apply_planet(content.planet);

        assert_eq!(planet.seed, 0x51C0_1D01);
        assert_eq!(planet.resolution, 512);
        assert_eq!(planet.surface_layer, 256);
        assert_eq!(scene.fixed_elapsed_secs, 180.0);
        assert_eq!(
            scene.quality.profile,
            vv_render::RenderQualityProfile::Balanced
        );

        let runtime = PlanetData::new(
            planet,
            content.blocks,
            content.items,
            content.procedural,
            content.procedural_planet_index,
        );
        assert_eq!(runtime.profile().seed, scene.seed);
        assert_eq!(runtime.profile().resolution, 1024);
        assert!(runtime.spawn_position().is_finite());

        let mut runtime = runtime;
        scene.apply_time(&mut runtime);
        assert_eq!(runtime.world_time().elapsed_seconds(), 180.0);
        assert!(runtime.world_time().is_paused());
    }
}
