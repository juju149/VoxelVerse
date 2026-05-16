use crate::app::content_bootstrap::load_core_content;
use crate::app::cursor::{grab_cursor, sync_cursor_mode};
use crate::app::event_router::{route_device_event, route_window_event};
use crate::app::frame_driver::tick_game_frame;
use crate::app::golden_scene::{golden_scene_enabled, GoldenScene};
use crate::ui::InventoryUiState;
use std::sync::Arc;
use std::time::Instant;
use vv_audio::AudioEngine;
use vv_gameplay::{Console, Controller, Hotbar, Inventory, MiningState, Player};
use vv_pack_compiler::{LootRegistry, RecipeRegistry, TagRegistry};
use vv_render::{Renderer, StreamingView};
use vv_world::{PlanetData, PlanetDataSources, VoxModelRegistry};
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowId};

pub(super) struct GameApp<'a> {
    pub(super) renderer: Renderer<'a>,
    pub(super) audio: AudioEngine,
    pub(super) controller: Controller,
    pub(super) player: Player,
    pub(super) hotbar: Hotbar,
    pub(super) inventory: Inventory,
    pub(super) inventory_ui: InventoryUiState,
    pub(super) mining: MiningState,
    pub(super) mining_button_held: bool,
    pub(super) shift_held: bool,
    pub(super) loot: Arc<LootRegistry>,
    pub(super) tags: Arc<TagRegistry>,
    pub(super) recipes: Arc<RecipeRegistry>,
    pub(super) planet: PlanetData,
    pub(super) console: Console,
    cursor_grabbed: bool,
    last_time: Instant,
    pub(super) first_scene_snapshot_logged: bool,
}

impl<'a> GameApp<'a> {
    pub(super) fn new(window: &'a Window) -> Self {
        let mut content = load_core_content();
        let golden_scene = golden_scene_enabled().then_some(GoldenScene::DEFAULT);
        if let Some(scene) = golden_scene {
            content.planet = scene.apply_planet(content.planet);
            println!(
                "[engine/golden] enabled seed={} resolution={} fixed_time_s={:.1}",
                scene.seed, scene.resolution, scene.fixed_elapsed_secs
            );
        }

        grab_cursor(window);

        let mut renderer = pollster::block_on(Renderer::new(
            window,
            &content.textures,
            content.blocks.material_colors(),
            &content.core_pack_dir,
        ));
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
        planet
            .world_time
            .set_day_length_seconds(renderer.atmosphere.day_length_seconds);
        planet
            .world_time
            .set_day_phase(renderer.atmosphere.start_phase);
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

        Self {
            renderer,
            audio,
            controller: Controller::new(),
            player,
            hotbar: Hotbar::new(),
            inventory: Inventory::new(),
            inventory_ui: InventoryUiState::new(),
            mining: MiningState::default(),
            mining_button_held: false,
            shift_held: false,
            loot,
            tags,
            recipes,
            planet,
            console: create_console(),
            cursor_grabbed: false,
            last_time: Instant::now(),
            first_scene_snapshot_logged: false,
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
            self.controller.first_person,
            self.console.is_open || self.inventory_ui.is_open,
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
