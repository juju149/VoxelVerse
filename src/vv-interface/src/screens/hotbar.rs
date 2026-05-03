use vv_ui::{UiBorder, UiColor, UiFrame, UiGradient, UiLayer, UiRect, UiShadow};

use crate::{design::InventoryUiTokens, item_visual, GameplayUiContext, InventoryUiLayout};

#[derive(Debug, Default, Clone, Copy)]
pub struct HotbarScreen;

impl HotbarScreen {
    pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        let layout = InventoryUiLayout::hotbar_only(
            ctx.screen_width,
            ctx.screen_height,
            &ctx.gameplay.inventory,
        );

        draw_hotbar(frame, ctx, &layout, UiLayer::Hud);
    }
}

/// Draws the single official VoxelVerse hotbar.
///
/// Used by:
/// - gameplay HUD
/// - inventory screen
///
/// Any future polish must happen here only.
pub fn draw_hotbar(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
    layer: UiLayer,
) {
    let tokens = InventoryUiTokens::current();
    let s = layout.scale;

    for hotbar_slot in &layout.hotbar_slots {
        let selected = hotbar_slot.index == ctx.gameplay.selected_hotbar_slot;
        let hidden = ctx.gameplay.inventory_drag.source_slot == Some(hotbar_slot.index);
        let stack = ctx.gameplay.inventory.slots()[hotbar_slot.index].stack;
        let visual = stack.map(|stack| item_visual(ctx.content, stack.item, stack.count));

        draw_hotbar_slot_base(frame, hotbar_slot.rect, selected, tokens, s, layer);

        if let Some(visual) = visual.as_ref().filter(|_| !hidden) {
            draw_hotbar_item(frame, hotbar_slot.rect, visual.color, tokens, s, layer);
            draw_hotbar_count(frame, hotbar_slot.rect, visual.count, tokens, s, layer);
        }

        draw_hotbar_number(frame, hotbar_slot.rect, hotbar_slot.index, tokens, s, layer);
    }
}

fn draw_hotbar_slot_base(
    frame: &mut UiFrame,
    rect: UiRect,
    selected: bool,
    tokens: InventoryUiTokens,
    scale: f32,
    layer: UiLayer,
) {
    if selected {
        let expanded = rect.expand(tokens.hotbar.selected_expand * scale);

        tokens
            .hotbar_selected_surface()
            .draw(frame, layer, expanded);
    }

    tokens.hotbar_slot_surface().draw(frame, layer, rect);
}

fn draw_hotbar_item(
    frame: &mut UiFrame,
    rect: UiRect,
    color: UiColor,
    tokens: InventoryUiTokens,
    scale: f32,
    layer: UiLayer,
) {
    let inset = tokens.hotbar.item_inset * scale;
    let item_rect = rect.inset(vv_ui::UiEdgeInsets::all(inset));

    frame.gradient_rect(
        layer,
        item_rect,
        UiGradient::vertical(color.lighten(0.18), color.darken(0.16)),
        tokens.radius.slot * 0.70 * scale,
        UiBorder::new(1.0 * scale, UiColor::rgba(1.0, 1.0, 1.0, 0.08)),
        UiShadow::NONE,
    );

    let bar_margin = tokens.hotbar.durability_margin * scale;
    let bar_h = tokens.hotbar.durability_height * scale;

    let bar = UiRect::new(
        rect.x + bar_margin,
        rect.bottom() - bar_margin - bar_h,
        rect.width - bar_margin * 2.0,
        bar_h,
    );

    frame.rounded_rect(
        layer,
        bar,
        UiColor::rgba(0.005, 0.020, 0.030, 0.92),
        bar_h * 0.5,
        UiBorder::NONE,
        UiShadow::NONE,
    );

    frame.rounded_rect(
        layer,
        UiRect::new(bar.x, bar.y, bar.width * 0.66, bar.height),
        tokens.colors.hotbar_durability_fill,
        bar_h * 0.5,
        UiBorder::NONE,
        UiShadow::NONE,
    );
}

fn draw_hotbar_number(
    frame: &mut UiFrame,
    rect: UiRect,
    index: usize,
    tokens: InventoryUiTokens,
    scale: f32,
    layer: UiLayer,
) {
    frame.text_left_centered(
        layer,
        UiRect::new(
            rect.x + 7.0 * scale,
            rect.y + 5.0 * scale,
            22.0 * scale,
            18.0 * scale,
        ),
        (index + 1).to_string(),
        tokens.hotbar.number_size * scale,
        tokens.colors.text_primary,
    );
}

fn draw_hotbar_count(
    frame: &mut UiFrame,
    rect: UiRect,
    count: u32,
    tokens: InventoryUiTokens,
    scale: f32,
    layer: UiLayer,
) {
    if count <= 1 {
        return;
    }

    frame.text_right_centered(
        layer,
        UiRect::new(
            rect.right() - 38.0 * scale,
            rect.bottom() - 30.0 * scale,
            32.0 * scale,
            20.0 * scale,
        ),
        count.to_string(),
        tokens.hotbar.count_size * scale,
        tokens.colors.text_active,
    );
}
