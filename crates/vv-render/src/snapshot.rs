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

/// All data the renderer needs for one frame.
/// Built in the app layer before each `Renderer::render` call.
pub struct RenderFrameSnapshot<'a> {
    pub camera: RenderCamera,
    pub planet: &'a vv_world::PlanetData,
    pub hotbar: &'a vv_gameplay::Hotbar,
    pub inventory: &'a vv_gameplay::Inventory,
    pub inventory_ui: &'a crate::ui::InventoryUiState,
    pub recipes: &'a vv_pack_compiler::RecipeRegistry,
    pub console: RenderConsoleSnapshot<'a>,
    pub debug: RenderDebugFlags,
}
