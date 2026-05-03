use vv_ui::UiFrame;

use crate::{
    screens::{ConsoleScreen, HotbarScreen, HudScreen, InventoryScreen},
    GameplayUiContext,
};

pub fn build_gameplay_ui_frame(ctx: GameplayUiContext<'_>) -> UiFrame {
    let mut frame = UiFrame::new(ctx.screen_width, ctx.screen_height);

    HudScreen::draw(&mut frame, &ctx);

    if ctx.gameplay.inventory_open {
        InventoryScreen::draw(&mut frame, &ctx);
    } else {
        HotbarScreen::draw(&mut frame, &ctx);
    }

    ConsoleScreen::draw(&mut frame, &ctx);

    frame
}
