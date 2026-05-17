/// Precomputed camera data passed to the renderer each frame.
/// Computed in the app layer from Controller + Player — no gameplay types in render.
pub struct RenderCamera {
    /// View-projection matrix (precomputed from controller.get_matrix).
    pub view_proj: glam::Mat4,
    /// Camera eye position in world space (precomputed from controller.get_camera_pos).
    pub camera_pos: glam::Vec3,
    /// Player body position (feet/centre).
    pub player_pos: glam::Vec3,
    /// Player model matrix for 3rd-person body rendering.
    pub model_matrix: glam::Mat4,
    /// True in first-person mode: no player body drawn, crosshair visible.
    pub is_first_person: bool,
    /// Voxel coordinate targeted by the cursor ray, if any.
    pub cursor_id: Option<vv_voxel::VoxelCoord>,
}

/// Debug visibility flags; all false when dev mode is disabled.
#[derive(Default)]
pub struct RenderDebugFlags {
    pub show_collisions: bool,
    pub freeze_culling: bool,
    pub is_wireframe: bool,
    /// True when the engine diagnostic overlay should be shown.
    /// Set by `player.debug_mode` or the F2 dev page toggle (now dev_mode in app).
    pub debug_mode: bool,
}

/// Read-only console state extracted for the renderer.
pub struct RenderConsoleSnapshot<'a> {
    pub height_fraction: f32,
    pub history: &'a [(String, [f32; 3])],
    pub input_buffer: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderItemStack {
    pub item_id: vv_pack_compiler::ItemId,
    pub quantity: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderSlotRef {
    Hotbar(usize),
    Inventory(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderHeldStack {
    pub stack: RenderItemStack,
    pub source: RenderSlotRef,
}

pub const RENDER_HOTBAR_SLOT_COUNT: usize = 9;
pub const RENDER_INVENTORY_COLS: usize = 9;
pub const RENDER_INVENTORY_ROWS: usize = 4;
pub const RENDER_INVENTORY_SIZE: usize = RENDER_INVENTORY_COLS * RENDER_INVENTORY_ROWS;

#[derive(Clone, Debug)]
pub struct RenderHotbarSnapshot {
    pub slots: [Option<RenderItemStack>; RENDER_HOTBAR_SLOT_COUNT],
    pub selected_index: usize,
    pub revision: u64,
    pub notice_text: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct RenderInventorySnapshot {
    pub slots: [Option<RenderItemStack>; RENDER_INVENTORY_SIZE],
    pub total_count: u32,
}

#[derive(Clone, Debug)]
pub struct RenderInventoryUiSnapshot {
    pub is_open: bool,
    pub search_query: String,
    pub held: Option<RenderHeldStack>,
    pub cursor: (f32, f32),
    pub hovered_slot: Option<RenderSlotRef>,
    pub hovered_button: Option<crate::ui::InventoryButton>,
    pub hovered_search: bool,
    pub hovered_filter: Option<crate::ui::InventoryFilter>,
    pub hovered_recipe: Option<usize>,
    pub active_filter: crate::ui::InventoryFilter,
    pub selected_recipe: Option<usize>,
    pub craft_quantity: u32,
    pub search_focused: bool,
    pub user_zoom: crate::ui::UserZoom,
    pub capacity_kg: f32,
}

impl RenderInventoryUiSnapshot {
    pub fn matches_search(&self, name: &str) -> bool {
        let q = self.search_query.to_lowercase();
        if q.is_empty() {
            return true;
        }
        name.to_lowercase().contains(&q)
    }

    pub fn matches_filter(&self, category: &str) -> bool {
        match self.active_filter {
            crate::ui::InventoryFilter::All => true,
            crate::ui::InventoryFilter::Resources => {
                matches!(
                    category,
                    "resource" | "ore" | "terrain" | "natural/log" | "natural/leaves" | "flora"
                )
            }
            crate::ui::InventoryFilter::Tools => matches!(category, "tool" | "weapon"),
            crate::ui::InventoryFilter::Food => matches!(category, "food" | "consumable"),
            crate::ui::InventoryFilter::Misc => !matches!(
                category,
                "resource"
                    | "ore"
                    | "terrain"
                    | "natural/log"
                    | "natural/leaves"
                    | "flora"
                    | "tool"
                    | "weapon"
                    | "food"
                    | "consumable"
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderCraftIngredient {
    pub icon: Option<RenderItemStack>,
    pub label: String,
    pub count: u32,
}

#[derive(Clone, Debug)]
pub struct RenderCraftRecipe {
    pub index: usize,
    pub output: RenderItemStack,
    pub output_name: String,
    pub station_label: String,
    pub ingredients: Vec<RenderCraftIngredient>,
}

#[derive(Clone, Debug, Default)]
pub struct RenderCraftSnapshot {
    pub recipes: Vec<RenderCraftRecipe>,
    pub selected_recipe: Option<RenderCraftRecipe>,
}

#[derive(Clone, Debug)]
pub struct RenderUiSnapshot {
    pub inventory: RenderInventoryUiSnapshot,
}

/// All data the renderer needs for one frame.
/// Built in the app layer before each `Renderer::render` call.
pub struct RenderFrameSnapshot<'a> {
    pub camera: RenderCamera,
    pub planet: &'a vv_world::PlanetData,
    pub hotbar: RenderHotbarSnapshot,
    pub inventory: RenderInventorySnapshot,
    pub ui: RenderUiSnapshot,
    pub craft: RenderCraftSnapshot,
    pub console: RenderConsoleSnapshot<'a>,
    pub debug: RenderDebugFlags,
    /// Optional per-frame weather snapshot (Phase 2 of the weather/cosmos
    /// roadmap). When `None` the renderer uses the planet preset as-is.
    pub weather: Option<&'a vv_weather::WeatherState>,
    /// Optional per-frame celestial snapshot (Phase 4 of the weather/cosmos
    /// roadmap). When `None` the renderer falls back to the preset-driven
    /// sun direction and skips stars/moons/aurora.
    pub celestial: Option<&'a vv_celestial::CelestialState>,
}
