use vv_gameplay::{can_craft_hand_recipe, Console, PlayerGameplayState};
use vv_input::Controller;
use vv_registry::{BlockRenderSource, CompiledContent, CompiledItemKind, ItemId};

use vv_ui::{
    UiBorder, UiColor, UiEdgeInsets, UiFrame, UiGradient, UiLayer, UiRect, UiShadow, UiStyle,
    UiTextAlign, UiTheme,
};

use crate::gameplay_ui::{GameplayUiLayout, RectPx};

use super::Renderer;

impl<'a> Renderer<'a> {
    pub(super) fn build_renderer_ui_frame(
        &self,
        controller: &Controller,
        console: &Console,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
    ) -> UiFrame {
        let w = self.config.width as f32;
        let h = self.config.height as f32;

        let theme = UiTheme::default();
        let style = UiStyle::from_theme(&theme);
        let mut frame = UiFrame::new(w, h);

        self.draw_fps(&mut frame, &theme);
        self.draw_crosshair(&mut frame, controller, gameplay, &theme);

        if gameplay.inventory_open {
            self.draw_inventory(&mut frame, gameplay, content, &theme, &style);
        } else {
            self.draw_hotbar(&mut frame, gameplay, content, &theme, &style);
        }

        self.draw_gameplay_notices(&mut frame, gameplay, &theme);
        self.draw_console(&mut frame, console, &theme);

        frame
    }

    fn draw_fps(&self, frame: &mut UiFrame, theme: &UiTheme) {
        frame.text(
            UiLayer::Hud,
            UiRect::new(10.0, 5.0, 140.0, 22.0),
            format!("FPS: {}", self.current_fps),
            16.0,
            theme.text_primary,
        );
    }

    fn draw_console(&self, frame: &mut UiFrame, console: &Console, theme: &UiTheme) {
        if console.height_fraction <= 0.001 {
            return;
        }

        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let console_h = (h / 2.0) * console.height_fraction;

        frame.rounded_rect(
            UiLayer::Popup,
            UiRect::new(0.0, 0.0, w, console_h),
            UiColor::rgba(0.04, 0.045, 0.07, 0.92),
            0.0,
            UiBorder::NONE,
            UiShadow::new(0.0, 14.0, 28.0, 0.0, UiColor::rgba(0.0, 0.0, 0.0, 0.35)),
        );

        let start_y = console_h - 44.0;
        let line_h = 20.0;

        for (i, (line, color)) in console.history.iter().rev().enumerate() {
            let y = start_y - i as f32 * line_h;
            if y < 8.0 {
                break;
            }

            frame.text(
                UiLayer::Popup,
                UiRect::new(12.0, y, w - 24.0, line_h),
                line,
                16.0,
                UiColor::rgb(color[0], color[1], color[2]),
            );
        }

        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let cursor = if (ms / 500) % 2 == 0 { "_" } else { " " };

        frame.text(
            UiLayer::Popup,
            UiRect::new(12.0, console_h - 24.0, w - 24.0, 22.0),
            format!("> {}{}", console.input_buffer, cursor),
            16.0,
            theme.accent,
        );
    }

    fn draw_crosshair(
        &self,
        frame: &mut UiFrame,
        controller: &Controller,
        gameplay: &PlayerGameplayState,
        theme: &UiTheme,
    ) {
        if !controller.first_person || gameplay.inventory_open {
            return;
        }

        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let scale = (w.min(h) / 720.0).clamp(0.75, 1.35);
        let cx = w * 0.5;
        let cy = h * 0.5;
        let thickness = (2.0 * scale).max(1.5);
        let gap = 6.0 * scale;
        let arm = 10.0 * scale;

        let active = gameplay.target.is_some();
        let mining = gameplay.mining.progress > 0.0;

        let color = if mining {
            theme.accent
        } else if active {
            theme.text_primary
        } else {
            theme.text_muted
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

    fn draw_hotbar(
        &self,
        frame: &mut UiFrame,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
        theme: &UiTheme,
        _style: &UiStyle,
    ) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let layout = GameplayUiLayout::new(w, h, &gameplay.inventory, false);

        for slot in &layout.hotbar_slots {
            let selected = slot.index == gameplay.selected_hotbar_slot;
            self.draw_inventory_slot(
                frame,
                slot.rect,
                selected,
                gameplay.inventory.slots()[slot.index].stack.map(|stack| {
                    (
                        self.item_color(stack.item, content),
                        stack.count,
                        gameplay.inventory_drag.source_slot == Some(slot.index),
                    )
                }),
                theme,
                UiLayer::Hud,
            );
        }
    }

    fn draw_inventory(
        &self,
        frame: &mut UiFrame,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
        theme: &UiTheme,
        _style: &UiStyle,
    ) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;

        frame.rect(
            UiLayer::Menu,
            UiRect::new(0.0, 0.0, w, h),
            theme.background_dim,
        );

        let mut layout = GameplayUiLayout::new(w, h, &gameplay.inventory, true);
        layout.add_hand_recipes(content.recipes.recipes_for_station(None));

        if let Some(panel) = layout.inventory_panel {
            self.draw_glass_panel(frame, panel, theme);

            frame.text(
                UiLayer::Menu,
                px_rect(
                    panel.x + 16.0 * layout.scale,
                    panel.y + 9.0 * layout.scale,
                    220.0,
                    28.0,
                ),
                "Backpack",
                16.0 * layout.scale,
                theme.text_primary,
            );

            frame.text(
                UiLayer::Menu,
                px_rect(
                    panel.x + 16.0 * layout.scale,
                    panel.y + panel.h - layout.slot - 18.0 * layout.scale,
                    160.0,
                    24.0,
                ),
                "Hotbar",
                13.0 * layout.scale,
                theme.text_muted,
            );
        }

        for slot in &layout.inventory_slots {
            self.draw_inventory_slot(
                frame,
                slot.rect,
                slot.index == gameplay.selected_hotbar_slot,
                gameplay.inventory.slots()[slot.index].stack.map(|stack| {
                    (
                        self.item_color(stack.item, content),
                        stack.count,
                        gameplay.inventory_drag.source_slot == Some(slot.index),
                    )
                }),
                theme,
                UiLayer::Menu,
            );
        }

        for slot in &layout.recipe_slots {
            let enabled = can_craft_hand_recipe(&gameplay.inventory, slot.recipe, content);
            let color = content
                .recipes
                .get(slot.recipe)
                .map(|recipe| self.item_color(recipe.result_item, content))
                .unwrap_or(theme.text_disabled);

            let bg = if enabled {
                theme.success.multiply_alpha(0.35)
            } else {
                theme.panel_subtle
            };

            frame.rounded_rect(
                UiLayer::Menu,
                px_rect(slot.rect.x, slot.rect.y, slot.rect.w, slot.rect.h),
                bg,
                6.0 * layout.scale,
                UiBorder::new(1.0, theme.border_soft),
                UiShadow::NONE,
            );

            frame.rounded_rect(
                UiLayer::Menu,
                px_rect(
                    slot.rect.x + slot.rect.w * 0.28,
                    slot.rect.y + slot.rect.h * 0.22,
                    slot.rect.w * 0.44,
                    slot.rect.h * 0.56,
                ),
                if enabled { color } else { color.darken(0.55) },
                4.0 * layout.scale,
                UiBorder::NONE,
                UiShadow::NONE,
            );
        }

        if let Some(stack) = gameplay.inventory_drag.stack {
            let color = self.item_color(stack.item, content);
            let size = layout.slot * 0.78;
            let rect = RectPx {
                x: 0.0,
                y: 0.0,
                w: size,
                h: size,
            };

            let drag_rect = px_rect(
                rect.x + 0.0 + 0.0 + 0.0 + 0.0 + 0.0 + 0.0 + 0.0 + 0.0 + 0.0,
                rect.y,
                rect.w,
                rect.h,
            );

            let mouse_x = 0.0;
            let mouse_y = 0.0;
            let _ = (drag_rect, mouse_x, mouse_y, color);
        }
    }

    fn draw_inventory_slot(
        &self,
        frame: &mut UiFrame,
        rect: RectPx,
        selected: bool,
        item: Option<(UiColor, u32, bool)>,
        theme: &UiTheme,
        layer: UiLayer,
    ) {
        let outer = px_rect(rect.x, rect.y, rect.w, rect.h);
        let border = if selected {
            UiBorder::new(2.0, theme.accent)
        } else {
            UiBorder::new(1.0, theme.border_soft)
        };

        frame.rounded_rect(
            layer,
            outer,
            if selected {
                theme.panel_active.multiply_alpha(0.42)
            } else {
                theme.panel_subtle
            },
            6.0,
            border,
            UiShadow::NONE,
        );

        frame.rounded_rect(
            layer,
            outer.inset(UiEdgeInsets::all(rect.w * 0.10)),
            theme.panel.multiply_alpha(0.46),
            4.0,
            UiBorder::NONE,
            UiShadow::NONE,
        );

        if let Some((color, count, hidden_by_drag)) = item {
            if hidden_by_drag {
                return;
            }

            let item_rect = outer.inset(UiEdgeInsets::all(rect.w * 0.26));

            frame.gradient_rect(
                layer,
                item_rect,
                UiGradient::vertical(color.lighten(0.20), color.darken(0.18)),
                4.0,
                UiBorder::new(1.0, UiColor::rgba(1.0, 1.0, 1.0, 0.12)),
                UiShadow::new(0.0, 4.0, 10.0, 0.0, UiColor::rgba(0.0, 0.0, 0.0, 0.25)),
            );

            if count > 1 {
                frame.text_aligned(
                    layer,
                    UiRect::new(
                        outer.x + outer.width * 0.44,
                        outer.y + outer.height * 0.62,
                        outer.width * 0.50,
                        outer.height * 0.30,
                    ),
                    count.to_string(),
                    (outer.height * 0.22).clamp(10.0, 16.0),
                    theme.text_primary,
                    UiTextAlign::Right,
                );
            }
        }
    }

    fn draw_glass_panel(&self, frame: &mut UiFrame, rect: RectPx, theme: &UiTheme) {
        frame.rounded_rect(
            UiLayer::Menu,
            px_rect(rect.x, rect.y, rect.w, rect.h),
            theme.panel,
            12.0,
            UiBorder::new(1.0, theme.border_soft),
            UiShadow::new(0.0, 16.0, 34.0, 0.0, UiColor::rgba(0.0, 0.0, 0.0, 0.38)),
        );
    }

    fn draw_gameplay_notices(
        &self,
        frame: &mut UiFrame,
        gameplay: &PlayerGameplayState,
        theme: &UiTheme,
    ) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;

        if gameplay.pickup_notice_timer > 0.0 {
            frame.text_aligned(
                UiLayer::Hud,
                UiRect::new(w * 0.5 - 120.0, h - 96.0, 240.0, 24.0),
                "Picked up",
                16.0,
                theme.text_primary,
                UiTextAlign::Center,
            );
        }

        if gameplay.placement_blocked_timer > 0.0 {
            frame.text_aligned(
                UiLayer::Hud,
                UiRect::new(w * 0.5 - 120.0, h * 0.58, 240.0, 24.0),
                "Cannot place",
                16.0,
                theme.warning,
                UiTextAlign::Center,
            );
        }
    }

    fn item_color(&self, item: ItemId, content: &CompiledContent) -> UiColor {
        let Some(item) = content.items.get(item) else {
            return UiColor::rgb(0.75, 0.75, 0.75);
        };

        let color = match item.kind {
            CompiledItemKind::Block { block } => self
                .block_content
                .block_render(block)
                .map(|render| render.color)
                .unwrap_or([0.75, 0.75, 0.75]),
            CompiledItemKind::Placeable { .. } => [0.95, 0.72, 0.35],
            CompiledItemKind::Tool { .. } => [0.72, 0.78, 0.85],
            CompiledItemKind::Armor => [0.62, 0.72, 0.90],
            CompiledItemKind::Food => [0.72, 0.90, 0.48],
            CompiledItemKind::Resource => [0.72, 0.68, 0.58],
        };

        UiColor::rgb(color[0], color[1], color[2])
    }
}

fn px_rect(x: f32, y: f32, w: f32, h: f32) -> UiRect {
    UiRect::new(x, y, w, h)
}
