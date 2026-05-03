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
    let radius = (rect.width * 0.13).clamp(7.0, 13.0);
    let border_width = if selected { 2.5 } else { 2.0 };

    let border_color = if selected {
        theme.accent.with_alpha(0.90)
    } else {
        UiColor::rgba(0.48, 0.29, 0.12, 0.78)
    };

    let fill_color = if selected {
        UiColor::rgba(0.018, 0.044, 0.050, 0.98)
    } else {
        UiColor::rgba(0.008, 0.025, 0.030, 0.98)
    };

    // Outer rounded rectangle = real rounded border.
    // Do not use UiBorder here because the current CPU border path draws straight strips.
    frame.rounded_rect(
        layer,
        rect,
        border_color,
        radius,
        UiBorder::NONE,
        UiShadow::NONE,
    );

    // Inner rounded rectangle = clean dark slot surface.
    frame.rounded_rect(
        layer,
        rect.inset(UiEdgeInsets::all(border_width)),
        fill_color,
        (radius - border_width).max(4.0),
        UiBorder::NONE,
        UiShadow::NONE,
    );

    if hidden_by_drag {
        return;
    }

    let Some(visual) = visual else {
        return;
    };

    let item_rect = rect.inset(UiEdgeInsets::all(rect.width * 0.27));

    // Keep item rendering simple too: no glow, no drop shadow.
    frame.gradient_rect(
        layer,
        item_rect,
        UiGradient::vertical(visual.color.lighten(0.20), visual.color.darken(0.18)),
        (radius * 0.42).max(4.0),
        UiBorder::new(1.0, UiColor::rgba(1.0, 0.94, 0.72, 0.12)),
        UiShadow::NONE,
    );

    if visual.count > 1 {
        let badge = UiRect::new(
            rect.x + rect.width * 0.52,
            rect.y + rect.height * 0.66,
            rect.width * 0.38,
            rect.height * 0.24,
        );

        frame.text_aligned(
            layer,
            badge,
            visual.count.to_string(),
            (rect.height * 0.20).clamp(10.0, 15.0),
            theme.text_primary,
            UiTextAlign::Right,
        );
    }
}
