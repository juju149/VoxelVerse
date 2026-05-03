mod backpack_panel;
mod crafting_panel;
mod equipment_panel;
mod footer;
mod header;
mod hotbar_strip;

use vv_ui::{UiFrame, UiLayer, UiRect};

use crate::{GameplayUiContext, InventoryUiLayout};

#[derive(Debug, Default, Clone, Copy)]
pub struct InventoryScreen;

impl InventoryScreen {
    pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        let mut layout = InventoryUiLayout::inventory(
            ctx.screen_width,
            ctx.screen_height,
            &ctx.gameplay.inventory,
        );

        layout.add_hand_recipes(ctx.content.recipes.recipes_for_station(None));

        frame.rect(
            UiLayer::Menu,
            UiRect::new(0.0, 0.0, ctx.screen_width, ctx.screen_height),
            ctx.theme.background_dim,
        );

        header::draw(frame, ctx, &layout);
        equipment_panel::draw(frame, ctx, &layout);
        backpack_panel::draw(frame, ctx, &layout);
        crafting_panel::draw(frame, ctx, &layout);
        hotbar_strip::draw(frame, ctx, &layout);
        footer::draw(frame, ctx, &layout);
    }
}
