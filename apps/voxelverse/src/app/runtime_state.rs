use crate::ui::InventoryUiState;
use std::sync::Arc;
use vv_gameplay::{Console, Controller, Hotbar, Inventory, MiningState, Player};
use vv_pack_compiler::{LootRegistry, RecipeRegistry, TagRegistry};
use vv_world::PlanetData;

pub(super) struct GameRuntime {
    pub(super) gameplay: GameplayRuntime,
    pub(super) ui: UiRuntime,
    pub(super) content: RuntimeContent,
    pub(super) planet: PlanetData,
    pub(super) first_scene_snapshot_logged: bool,
}

pub(super) struct GameplayRuntime {
    pub(super) controller: Controller,
    pub(super) player: Player,
    pub(super) hotbar: Hotbar,
    pub(super) inventory: Inventory,
    pub(super) mining: MiningState,
    pub(super) mining_button_held: bool,
}

pub(super) struct UiRuntime {
    pub(super) inventory: InventoryUiState,
    pub(super) console: Console,
    pub(super) shift_held: bool,
}

pub(super) struct RuntimeContent {
    pub(super) loot: Arc<LootRegistry>,
    pub(super) tags: Arc<TagRegistry>,
    pub(super) recipes: Arc<RecipeRegistry>,
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
}
