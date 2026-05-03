use vv_ui::{UiColor, UiFrame, UiLayer, UiRect, UiTextAlign};

use crate::GameplayUiContext;

#[derive(Debug, Default, Clone, Copy)]
pub struct HudScreen;

impl HudScreen {
    pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        Self::draw_fps(frame, ctx);
        Self::draw_crosshair(frame, ctx);
        Self::draw_notices(frame, ctx);
    }

    fn draw_fps(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        frame.text(
            UiLayer::Hud,
            UiRect::new(10.0, 5.0, 140.0, 22.0),
            format!("FPS: {}", ctx.current_fps),
            16.0,
            ctx.theme.text_primary,
        );
    }

    fn draw_crosshair(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        if !ctx.first_person || ctx.gameplay.inventory_open {
            return;
        }

        let scale = (ctx.screen_width.min(ctx.screen_height) / 720.0).clamp(0.75, 1.35);
        let cx = ctx.screen_width * 0.5;
        let cy = ctx.screen_height * 0.5;
        let thickness = (2.0 * scale).max(1.5);
        let gap = 6.0 * scale;
        let arm = 10.0 * scale;

        let active = ctx.gameplay.target.is_some();
        let mining = ctx.gameplay.mining.progress > 0.0;

        let color = if mining {
            ctx.theme.accent
        } else if active {
            ctx.theme.text_primary
        } else {
            ctx.theme.text_muted
        };

        let shadow = UiColor::rgba(0.0, 0.0, 0.0, 0.45);

        for (dx, dy, ww, hh) in [
            (-gap - arm, -thickness * 0.5, arm, thickness),
            (gap, -thickness * 0.5, arm, thickness),
            (-thickness * 0.5, -gap - arm, thickness, arm),
            (-thickness * 0.5, gap, thickness, arm),
        ] {
            frame.rect(
                UiLayer::Hud,
                UiRect::new(cx + dx + scale, cy + dy + scale, ww, hh),
                shadow,
            );
            frame.rect(UiLayer::Hud, UiRect::new(cx + dx, cy + dy, ww, hh), color);
        }
    }

    fn draw_notices(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        if ctx.gameplay.pickup_notice_timer > 0.0 {
            frame.text_aligned(
                UiLayer::Hud,
                UiRect::new(
                    ctx.screen_width * 0.5 - 120.0,
                    ctx.screen_height - 96.0,
                    240.0,
                    24.0,
                ),
                "Picked up",
                16.0,
                ctx.theme.text_primary,
                UiTextAlign::Center,
            );
        }

        if ctx.gameplay.placement_blocked_timer > 0.0 {
            frame.text_aligned(
                UiLayer::Hud,
                UiRect::new(
                    ctx.screen_width * 0.5 - 120.0,
                    ctx.screen_height * 0.58,
                    240.0,
                    24.0,
                ),
                "Cannot place",
                16.0,
                ctx.theme.warning,
                UiTextAlign::Center,
            );
        }
    }
}
