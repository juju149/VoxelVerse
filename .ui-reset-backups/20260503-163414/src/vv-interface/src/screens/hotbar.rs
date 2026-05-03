use vv_ui::{UiFrame, UiInput, UiLayer, UiRect, UiSlot, UiSlotContent, UiStyle, UiWidgetId};

use crate::{item_visual, GameplayUiContext, InventoryUiLayout};

#[derive(Debug, Default, Clone, Copy)]
pub struct HotbarScreen;

impl HotbarScreen {
    pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        let layout = InventoryUiLayout::hotbar_only(
            ctx.screen_width,
            ctx.screen_height,
            &ctx.gameplay.inventory,
        );

        Self::draw_from_layout(frame, ctx, &layout);
    }

    fn draw_from_layout(
        frame: &mut UiFrame,
        ctx: &GameplayUiContext<'_>,
        layout: &InventoryUiLayout,
    ) {
        if layout.hotbar_slots.is_empty() {
            return;
        }

        let styles = UiStyle::from_theme(ctx.theme);
        let input = UiInput::default();
        let layer = if ctx.gameplay.inventory_open {
            UiLayer::Menu
        } else {
            UiLayer::Hud
        };

        for hotbar_slot in &layout.hotbar_slots {
            let stack = ctx.gameplay.inventory.slots()[hotbar_slot.index].stack;
            let visual = stack.map(|stack| item_visual(ctx.content, stack.item, stack.count));
            let hidden = ctx.gameplay.inventory_drag.source_slot == Some(hotbar_slot.index);

            let mut slot = UiSlot::new(
                hotbar_widget_id(layer, hotbar_slot.index),
                hotbar_slot.rect,
                styles.slot,
            )
            .selected(hotbar_slot.index == ctx.gameplay.selected_hotbar_slot)
            .layer(layer);

            if let Some(visual) = visual.as_ref().filter(|_| !hidden) {
                slot = slot
                    .content(UiSlotContent::Color(visual.color))
                    .count(Some(visual.count));
            }

            slot.draw(frame, &input, None);

            draw_slot_number(
                frame,
                hotbar_slot.rect,
                hotbar_slot.index,
                layout.scale,
                ctx,
                layer,
            );
        }
    }
}

fn draw_slot_number(
    frame: &mut UiFrame,
    rect: UiRect,
    index: usize,
    scale: f32,
    ctx: &GameplayUiContext<'_>,
    layer: UiLayer,
) {
    frame.text(
        layer,
        UiRect::new(
            rect.x + 6.0 * scale,
            rect.y + 4.0 * scale,
            22.0 * scale,
            18.0 * scale,
        ),
        (index + 1).to_string(),
        12.0 * scale,
        ctx.theme.text_primary,
    );
}

fn hotbar_widget_id(layer: UiLayer, index: usize) -> UiWidgetId {
    let layer_offset = match layer {
        UiLayer::Background => 10_000,
        UiLayer::SceneOverlay => 20_000,
        UiLayer::Hud => 30_000,
        UiLayer::Menu => 40_000,
        UiLayer::Popup => 50_000,
        UiLayer::Tooltip => 60_000,
        UiLayer::Cursor => 70_000,
    };

    UiWidgetId::new(layer_offset + index as u64)
}
