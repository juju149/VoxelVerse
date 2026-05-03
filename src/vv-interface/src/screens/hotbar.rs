use vv_ui::{UiFrame, UiLayer};

use crate::{components::slot::item_slot, item_visual, GameplayUiContext, InventoryUiLayout};

#[derive(Debug, Default, Clone, Copy)]
pub struct HotbarScreen;

impl HotbarScreen {
    pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        let layout = InventoryUiLayout::hotbar_only(
            ctx.screen_width,
            ctx.screen_height,
            &ctx.gameplay.inventory,
        );

        for slot in &layout.hotbar_slots {
            let stack = ctx.gameplay.inventory.slots()[slot.index].stack;
            let visual = stack.map(|stack| item_visual(ctx.content, stack.item, stack.count));

            item_slot(
                frame,
                slot.rect,
                slot.index == ctx.gameplay.selected_hotbar_slot,
                visual.as_ref(),
                ctx.gameplay.inventory_drag.source_slot == Some(slot.index),
                ctx.theme,
                UiLayer::Hud,
            );
        }
    }
}
