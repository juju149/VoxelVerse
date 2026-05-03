use vv_ui::UiFrame;

use crate::{components::key_hint, GameplayUiContext, InventoryUiLayout};

pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, layout: &InventoryUiLayout) {
    let s = layout.scale;
    let key_w = 70.0 * s;
    let label_w = 84.0 * s;
    let gap = 8.0 * s;
    let total_w = key_w + label_w + gap;

    let x = layout.title_bar.right() - total_w;
    let y = ctx.screen_height - 64.0 * s;

    key_hint::draw_pair(frame, x, y, "ÉCHAP", "Fermer", ctx, s);
}
