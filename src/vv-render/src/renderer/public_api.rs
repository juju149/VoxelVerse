use glam::{Vec2, Vec3};

use vv_diagnostics::{emit, LogDomain, LogLevel};
use vv_gameplay::PlayerGameplayState;
use vv_registry::{CompiledContent, RecipeId};
use vv_ui::UiPoint;
use vv_voxel::BlockId;
use vv_world_runtime::PlanetData;

use crate::block_feedback::{selection_outline_mesh, SelectionOutlineStyle};

use super::Renderer;

impl<'a> Renderer<'a> {
    pub fn advance_time(&mut self, dt: f32) {
        self.sky_state.advance(dt);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth = Self::mk_depth(&self.device, &self.config);

        emit(
            self.diagnostic_config,
            LogLevel::Info,
            LogDomain::Render,
            format!("surface resized width={} height={}", width, height),
        );
    }

    pub fn begin_diagnostic_frame(&mut self) {
        self.frame_telemetry = Default::default();
    }

    pub fn update_cursor(&mut self, planet: &PlanetData, id: Option<BlockId>) {
        if let Some(id) = id {
            let mesh = selection_outline_mesh(planet, id, SelectionOutlineStyle::default());

            self.queue
                .write_buffer(&self.cursor_v_buf, 0, bytemuck::cast_slice(&mesh.vertices));
            self.queue
                .write_buffer(&self.cursor_i_buf, 0, bytemuck::cast_slice(&mesh.indices));

            self.cursor_inds = mesh.indices.len() as u32;
        } else {
            self.cursor_inds = 0;
        }
    }

    pub fn inventory_slot_at(
        &self,
        gameplay: &PlayerGameplayState,
        mouse_pos: Vec2,
    ) -> Option<usize> {
        vv_interface::InventoryUiLayout::new(
            self.config.width as f32,
            self.config.height as f32,
            &gameplay.inventory,
            gameplay.inventory_open,
        )
        .inventory_slot_at(UiPoint::new(mouse_pos.x, mouse_pos.y))
    }

    pub fn inventory_recipe_at(
        &self,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
        mouse_pos: Vec2,
    ) -> Option<RecipeId> {
        let mut layout = vv_interface::InventoryUiLayout::new(
            self.config.width as f32,
            self.config.height as f32,
            &gameplay.inventory,
            gameplay.inventory_open,
        );

        layout.add_hand_recipes(content.recipes.recipes_for_station(None));
        layout.recipe_at(UiPoint::new(mouse_pos.x, mouse_pos.y))
    }

    pub fn force_reload_all(&mut self, planet: &PlanetData, player_pos: Vec3) {
        self.chunks.clear();
        self.lod_chunks.clear();
        self.load_queue.clear();
        self.pending_chunks.clear();
        self.pending_lods.clear();
        self.player_chunk_pos = None;

        self.update_view(player_pos, planet);
    }
}
