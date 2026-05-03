use vv_ui::{
    UiBorder, UiColor, UiEdgeInsets, UiFrame, UiGradient, UiLayer, UiRect, UiShadow, UiTextAlign,
};

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

        for slot in &layout.hotbar_slots {
            let stack = ctx.gameplay.inventory.slots()[slot.index].stack;
            let visual = stack.map(|stack| item_visual(ctx.content, stack.item, stack.count));

            draw_item_slot(
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

pub(crate) fn draw_item_slot(
    frame: &mut UiFrame,
    rect: UiRect,
    selected: bool,
    visual: Option<&crate::ItemVisual>,
    hidden_by_drag: bool,
    theme: &vv_ui::UiTheme,
    layer: UiLayer,
) {
    let border = if selected {
        UiBorder::new(2.0, theme.accent)
    } else {
        UiBorder::new(1.0, theme.border_soft)
    };

    frame.rounded_rect(
        layer,
        rect,
        if selected {
            theme.panel_active.multiply_alpha(0.42)
        } else {
            theme.panel_subtle
        },
        7.0,
        border,
        UiShadow::new(0.0, 7.0, 14.0, 0.0, UiColor::rgba(0.0, 0.0, 0.0, 0.24)),
    );

    frame.rounded_rect(
        layer,
        rect.inset(UiEdgeInsets::all(rect.width * 0.10)),
        theme.panel.multiply_alpha(0.46),
        5.0,
        UiBorder::NONE,
        UiShadow::NONE,
    );

    if hidden_by_drag {
        return;
    }

    let Some(visual) = visual else {
        return;
    };

    let item_rect = rect.inset(UiEdgeInsets::all(rect.width * 0.26));

    frame.gradient_rect(
        layer,
        item_rect,
        UiGradient::vertical(visual.color.lighten(0.20), visual.color.darken(0.18)),
        5.0,
        UiBorder::new(1.0, UiColor::rgba(1.0, 1.0, 1.0, 0.12)),
        UiShadow::new(0.0, 4.0, 10.0, 0.0, UiColor::rgba(0.0, 0.0, 0.0, 0.25)),
    );

    if visual.count > 1 {
        frame.text_aligned(
            layer,
            UiRect::new(
                rect.x + rect.width * 0.44,
                rect.y + rect.height * 0.62,
                rect.width * 0.50,
                rect.height * 0.30,
            ),
            visual.count.to_string(),
            (rect.height * 0.22).clamp(10.0, 16.0),
            theme.text_primary,
            UiTextAlign::Right,
        );
    }
}
