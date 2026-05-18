use crate::app::content_bootstrap::load_core_content;
use crate::app::cursor::{grab_cursor, sync_cursor_mode};
use crate::app::diagnostics_export::create_diagnostics;
use crate::app::event_router::{route_device_event, route_window_event};
use crate::app::frame_driver::tick_game_frame;
use crate::app::golden_scene::{golden_scene_enabled, GoldenScene};
use crate::app::input_accumulator::InputAccumulator;
use crate::app::runtime_state::GameRuntime;
use std::sync::Arc;
use std::time::Instant;
use vv_audio::AudioEngine;
use vv_diagnostics::{Diagnostics, DiagnosticsFileSink};
use vv_gameplay::{Console, Player};
use vv_render::{Renderer, StreamingView};
use vv_world::{PlanetData, PlanetDataSources, VoxModelRegistry};
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowId};

pub(super) struct GameApp<'a> {
    pub(super) renderer: Renderer<'a>,
    pub(super) audio: AudioEngine,
    pub(super) runtime: GameRuntime,
    pub(super) diagnostics: Diagnostics,
    pub(super) diagnostics_sink: Option<DiagnosticsFileSink>,
    pub(super) input_accum: InputAccumulator,
    cursor_grabbed: bool,
    last_time: Instant,
}

impl<'a> GameApp<'a> {
    pub(super) fn new(window: &'a Window) -> Self {
        let mut content = match load_core_content() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[voxelverse] Content load failed:\n{e}");
                eprintln!("[voxelverse] Run `cargo run -p vv-pack-doctor -- assets/packs/core` for details.");
                std::process::exit(1);
            }
        };
        let golden_scene = golden_scene_enabled().then_some(GoldenScene::DEFAULT);
        if let Some(scene) = golden_scene {
            content.planet = scene.apply_planet(content.planet);
            println!(
                "[engine/golden] enabled seed={} resolution={} fixed_time_s={:.1}",
                scene.seed, scene.resolution, scene.fixed_elapsed_secs
            );
        }

        grab_cursor(window);

        let pack_stack = vec![vv_render::PackShaderRoot::new(
            "core",
            content.core_pack_dir.clone(),
        )];
        let mut renderer =
            pollster::block_on(Renderer::new(window, &content.textures, &pack_stack));
        let audio = AudioEngine::new(&content.core_pack_dir);
        if let Some(scene) = golden_scene {
            renderer.quality = scene.quality;
            renderer.set_engine_debug_page(true);
        }
        renderer.render_loading(0.0, "Initialisation planète");

        renderer.render_loading(0.05, "Chargement des modèles vox…");
        let prop_models = Arc::new(VoxModelRegistry::load_all(
            &content.core_pack_dir,
            &content.vox_asset_paths,
            &content.needed_vox_keys,
        ));

        let loot = Arc::clone(&content.loot);
        let tags = Arc::clone(&content.tags);
        let recipes = Arc::clone(&content.recipes);

        let mut planet = PlanetData::new_with_progress(
            content.planet,
            PlanetDataSources {
                registry: content.blocks,
                items: content.items,
                terrain_visuals: content.terrain_visuals,
                procedural: content.procedural,
                procedural_planet_index: content.procedural_planet_index,
                prop_models,
            },
            |progress, message| renderer.render_loading(progress, message),
        );
        planet.set_day_length_seconds(renderer.atmosphere.day_length_seconds);
        planet.set_day_phase(renderer.atmosphere.start_phase);
        if let Some(scene) = golden_scene {
            scene.apply_time(&mut planet);
        }
        renderer.render_loading(1.0, "Monde prêt");
        renderer.window.set_title("voxelverse");

        let mut player = Player::new();
        if let Some(scene) = golden_scene {
            scene.spawn_player(&mut player, &planet);
        } else {
            player.spawn(planet.spawn_position());
        }

        let warmup_view = StreamingView {
            player_pos: player.position,
            camera_pos: player.position,
            view_dir: player.position.normalize_or_zero(),
            cursor_id: None,
        };
        renderer.prewarm_until_idle(&planet, warmup_view, |r, progress, message| {
            r.render_loading(progress, message);
        });
        renderer.log_engine_snapshot("startup", &planet);

        let (diagnostics, diagnostics_sink) = create_diagnostics();

        Self {
            renderer,
            audio,
            runtime: GameRuntime::new(player, planet, loot, tags, recipes, create_console()),
            diagnostics,
            diagnostics_sink,
            input_accum: InputAccumulator::new(),
            cursor_grabbed: false,
            last_time: Instant::now(),
        }
    }

    pub(super) fn window_id(&self) -> WindowId {
        self.renderer.window.id()
    }

    pub(super) fn handle_device_event(&mut self, event: DeviceEvent) {
        route_device_event(self, event);
    }

    pub(super) fn handle_window_event(
        &mut self,
        event: WindowEvent,
        target: &EventLoopWindowTarget<()>,
    ) {
        route_window_event(self, event, target);
    }

    pub(super) fn tick(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_time).as_secs_f32();
        self.last_time = now;

        sync_cursor_mode(
            self.renderer.window,
            self.runtime.first_person(),
            self.runtime.ui_captures_input(),
            &mut self.cursor_grabbed,
        );
        tick_game_frame(self, dt);
    }
}

fn create_console() -> Console {
    let mut console = Console::new();
    console.log("Welcome to voxelverse.", [0.0, 1.0, 0.0]);
    console.log("Press ` to open console.", [1.0, 1.0, 1.0]);
    console
}
