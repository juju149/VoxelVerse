use vv_ui::{UiFrame, UiLayer, UiRect};

use crate::{components::slot, item_visual, GameplayUiContext, InventoryUiLayout};

pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, layout: &InventoryUiLayout) {
    for hotbar_slot in &layout.hotbar_slots {
        let stack = ctx.gameplay.inventory.slots()[hotbar_slot.index].stack;
        let visual = stack.map(|stack| item_visual(ctx.content, stack.item, stack.count));

        slot::item_slot(
            frame,
            hotbar_slot.rect,
            hotbar_slot.index == ctx.gameplay.selected_hotbar_slot,
            visual.as_ref(),
            ctx.gameplay.inventory_drag.source_slot == Some(hotbar_slot.index),
            ctx.theme,
            UiLayer::Menu,
        );

        frame.text(
            UiLayer::Menu,
            UiRect::new(
                hotbar_slot.rect.x + 6.0,
                hotbar_slot.rect.y + 4.0,
                22.0,
                18.0,
            ),
            (hotbar_slot.index + 1).to_string(),
            12.0 * layout.scale,
            ctx.theme.text_primary,
        );
    }
}
