use crate::app::gameplay_actions::{MineBlockContext, PlaceBlockContext};
use crate::ui::InventoryUiState;
use std::sync::Arc;
use vv_gameplay::{
    Console, Controller, Hotbar, Inventory, MiningState, Player, PlayerController, PlayerInput,
};
use vv_pack_compiler::{LootRegistry, RecipeRegistry, TagRegistry};
use vv_voxel::VoxelCoord;
use vv_world::PlanetData;
use winit::event::WindowEvent;

/// Top-level runtime state. Fields are private; use controlled accessor methods.
pub(super) struct GameRuntime {
    gameplay: GameplayRuntime,
    ui: UiRuntime,
    content: RuntimeContent,
    planet: PlanetData,
    first_scene_snapshot_logged: bool,
}

struct GameplayRuntime {
    controller: Controller,
    player: Player,
    hotbar: Hotbar,
    inventory: Inventory,
    mining: MiningState,
    mining_button_held: bool,
}

struct UiRuntime {
    inventory: InventoryUiState,
    console: Console,
    shift_held: bool,
}

struct RuntimeContent {
    loot: Arc<LootRegistry>,
    tags: Arc<TagRegistry>,
    recipes: Arc<RecipeRegistry>,
}

impl GameRuntime {
    pub(super) fn new(
        player: Player,
        planet: PlanetData,
        loot: Arc<LootRegistry>,
        tags: Arc<TagRegistry>,
        recipes: Arc<RecipeRegistry>,
        console: Console,
    ) -> Self {
        Self {
            gameplay: GameplayRuntime {
                controller: Controller::new(),
                player,
                hotbar: Hotbar::new(),
                inventory: Inventory::new(),
                mining: MiningState::default(),
                mining_button_held: false,
            },
            ui: UiRuntime {
                inventory: InventoryUiState::new(),
                console,
                shift_held: false,
            },
            content: RuntimeContent {
                loot,
                tags,
                recipes,
            },
            planet,
            first_scene_snapshot_logged: false,
        }
    }

    // ── Controller ────────────────────────────────────────────────────────────

    pub(super) fn controller(&self) -> &Controller {
        &self.gameplay.controller
    }

    pub(super) fn controller_mut(&mut self) -> &mut Controller {
        &mut self.gameplay.controller
    }

    pub(super) fn cursor_id(&self) -> Option<VoxelCoord> {
        self.gameplay.controller.cursor_id
    }

    pub(super) fn set_cursor_id(&mut self, id: Option<VoxelCoord>) {
        self.gameplay.controller.cursor_id = id;
    }

    pub(super) fn first_person(&self) -> bool {
        self.gameplay.controller.first_person
    }

    // ── Player ────────────────────────────────────────────────────────────────

    pub(super) fn player(&self) -> &Player {
        &self.gameplay.player
    }

    // ── Hotbar ────────────────────────────────────────────────────────────────

    pub(super) fn hotbar(&self) -> &Hotbar {
        &self.gameplay.hotbar
    }

    pub(super) fn hotbar_mut(&mut self) -> &mut Hotbar {
        &mut self.gameplay.hotbar
    }

    // ── Inventory ─────────────────────────────────────────────────────────────

    pub(super) fn inventory(&self) -> &Inventory {
        &self.gameplay.inventory
    }

    // ── Mining ────────────────────────────────────────────────────────────────

    pub(super) fn mining_button_held(&self) -> bool {
        self.gameplay.mining_button_held
    }

    pub(super) fn set_mining_button_held(&mut self, held: bool) {
        self.gameplay.mining_button_held = held;
    }

    // ── Planet ────────────────────────────────────────────────────────────────

    pub(super) fn planet(&self) -> &PlanetData {
        &self.planet
    }

    pub(super) fn planet_mut(&mut self) -> &mut PlanetData {
        &mut self.planet
    }

    // ── UI / Console ──────────────────────────────────────────────────────────

    pub(super) fn console(&self) -> &Console {
        &self.ui.console
    }

    pub(super) fn console_mut(&mut self) -> &mut Console {
        &mut self.ui.console
    }

    // ── UI / Inventory ────────────────────────────────────────────────────────

    pub(super) fn inventory_ui(&self) -> &InventoryUiState {
        &self.ui.inventory
    }

    pub(super) fn inventory_ui_mut(&mut self) -> &mut InventoryUiState {
        &mut self.ui.inventory
    }

    // ── UI / Shift ────────────────────────────────────────────────────────────
    // shift_held state is owned by inventory_event_parts; no standalone accessor needed.

    // ── Content registries ────────────────────────────────────────────────────
    // loot and tags are accessed via as_mine_context / inventory_event_parts.

    pub(super) fn recipes(&self) -> &RecipeRegistry {
        &self.content.recipes
    }

    // ── Semantic helpers ──────────────────────────────────────────────────────

    /// Returns `true` when any UI layer is capturing keyboard / mouse input.
    pub(super) fn ui_captures_input(&self) -> bool {
        self.ui.console.is_open || self.ui.inventory.is_open
    }

    pub(super) fn scene_snapshot_logged(&self) -> bool {
        self.first_scene_snapshot_logged
    }

    pub(super) fn mark_scene_snapshot_logged(&mut self) {
        self.first_scene_snapshot_logged = true;
    }

    // ── Multi-field action contexts ───────────────────────────────────────────
    // These methods split-borrow internal fields so callers never need to reach
    // into the private structs directly.

    pub(super) fn as_mine_context(&mut self) -> MineBlockContext<'_> {
        MineBlockContext {
            controller: &self.gameplay.controller,
            planet: &mut self.planet,
            hotbar: &mut self.gameplay.hotbar,
            inventory: &mut self.gameplay.inventory,
            mining: &mut self.gameplay.mining,
            loot: &self.content.loot,
        }
    }

    pub(super) fn as_place_context(
        &mut self,
        view_width: f32,
        view_height: f32,
    ) -> PlaceBlockContext<'_> {
        PlaceBlockContext {
            controller: &mut self.gameplay.controller,
            player: &self.gameplay.player,
            planet: &mut self.planet,
            hotbar: &mut self.gameplay.hotbar,
            view_width,
            view_height,
        }
    }

    // ── Inventory event helpers ───────────────────────────────────────────────
    // Provide structured access for inventory_events without exposing inner types.

    pub(super) fn inventory_event_parts(
        &mut self,
    ) -> (
        &mut Controller,
        &Player,
        &PlanetData,
        &mut Hotbar,
        &mut Inventory,
        &mut InventoryUiState,
        &RecipeRegistry,
        &TagRegistry,
        &mut bool,
        &Console,
    ) {
        (
            &mut self.gameplay.controller,
            &self.gameplay.player,
            &self.planet,
            &mut self.gameplay.hotbar,
            &mut self.gameplay.inventory,
            &mut self.ui.inventory,
            &self.content.recipes,
            &self.content.tags,
            &mut self.ui.shift_held,
            &self.ui.console,
        )
    }

    // ── Console submit ────────────────────────────────────────────────────────

    /// Submit the pending console input, passing the player for command execution.
    /// `ui.console` and `gameplay.player` are disjoint fields — safe to access together.
    pub(super) fn submit_console_command(&mut self) {
        self.ui.console.submit(&mut self.gameplay.player);
    }

    // ── Controller + player combined operations ───────────────────────────────

    /// Route a winit window event to the controller with player context.
    /// Splits the borrow across `gameplay.controller` (mut) and `gameplay.player` (ref).
    pub(super) fn process_controller_event(&mut self, event: &WindowEvent) {
        self.gameplay
            .controller
            .process_events(event, &self.gameplay.player);
    }

    /// Advance player physics using controller-sampled input and planet data.
    /// Splits the borrow across `gameplay.player` (mut) and `planet` (ref).
    pub(super) fn update_player_movement(&mut self, input: PlayerInput, dt: f32) {
        PlayerController::update(&mut self.gameplay.player, &self.planet, input, dt);
    }

    /// Sample player input from the controller (non-mutating).
    pub(super) fn sample_player_input(&mut self) -> PlayerInput {
        self.gameplay.controller.sample_player_input()
    }

    // ── Planet + player combined access ───────────────────────────────────────

    /// Provide simultaneous mutable access to planet and player (e.g. for planet resize).
    pub(super) fn planet_and_player_mut(&mut self) -> (&mut PlanetData, &mut Player) {
        (&mut self.planet, &mut self.gameplay.player)
    }
}
