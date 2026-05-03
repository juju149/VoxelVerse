use vv_ui::{UiFrame, UiGradient, UiLayer, UiRect};

use crate::{
    components::{surface, text},
    design::VvDesignTokens,
    GameplayUiContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VvTabSpec<'a> {
    pub label: &'a str,
}

impl<'a> VvTabSpec<'a> {
    pub const fn text(label: &'a str) -> Self {
        Self { label }
    }
}

pub fn segmented(
    frame: &mut UiFrame,
    bounds: UiRect,
    tabs: &[VvTabSpec<'_>],
    active_index: usize,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    if tabs.is_empty() || bounds.width <= 0.0 || bounds.height <= 0.0 {
        return;
    }

    let tokens = VvDesignTokens::current();
    let tab_tokens = tokens.inventory_tabs;

    let gap = tab_tokens.gap * scale;
    let mut widths = tabs
        .iter()
        .map(|tab| ideal_tab_width(tab, scale, &tokens))
        .collect::<Vec<_>>();

    fit_widths(&mut widths, bounds.width, gap, tab_tokens.min_width * scale);

    let total_width = widths.iter().sum::<f32>() + gap * widths.len().saturating_sub(1) as f32;
    let mut x = bounds.x + (bounds.width - total_width).max(0.0) * 0.5;

    for (index, tab) in tabs.iter().enumerate() {
        let width = widths[index];

        if width <= 0.0 {
            continue;
        }

        let rect = UiRect::new(x, bounds.y, width, bounds.height);
        draw_tab(
            frame,
            rect,
            *tab,
            index == active_index,
            ctx,
            scale,
            &tokens,
        );

        x += width + gap;

        if x > bounds.right() + 1.0 {
            break;
        }
    }
}

fn draw_tab(
    frame: &mut UiFrame,
    rect: UiRect,
    tab: VvTabSpec<'_>,
    active: bool,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
    tokens: &VvDesignTokens,
) {
    let tab_tokens = tokens.inventory_tabs;
    let radius = (rect.height * tab_tokens.radius_factor)
        .clamp(tab_tokens.radius_min * scale, tab_tokens.radius_max * scale);

    let border = if active {
        tokens.colors.button_border_active
    } else {
        tokens.colors.button_border
    };

    let gradient = if active {
        UiGradient::vertical(
            tokens.colors.button_active_top,
            tokens.colors.button_active_bottom,
        )
    } else {
        UiGradient::vertical(tokens.colors.button_top, tokens.colors.button_bottom)
    };

    surface::gradient(
        frame,
        UiLayer::Menu,
        rect,
        gradient,
        border,
        if active {
            tokens.button.active_border_width * scale
        } else {
            tokens.button.border_width * scale
        },
        radius,
    );

    text::centered(
        frame,
        UiLayer::Menu,
        rect,
        tab.label,
        tokens.text.tab_size * scale,
        if active {
            ctx.theme.text_primary
        } else {
            ctx.theme.text_muted
        },
    );
}

fn ideal_tab_width(tab: &VvTabSpec<'_>, scale: f32, tokens: &VvDesignTokens) -> f32 {
    let t = tokens.inventory_tabs;
    let label_size = tokens.text.tab_size * scale;

    let raw = t.padding_x * scale * 2.0 + estimate_label_width(tab.label, label_size);

    raw.clamp(t.min_width * scale, t.max_width * scale)
}

fn fit_widths(widths: &mut [f32], available_width: f32, gap: f32, min_width: f32) {
    if widths.is_empty() {
        return;
    }

    let total_gap = gap * widths.len().saturating_sub(1) as f32;
    let total_width = widths.iter().sum::<f32>() + total_gap;

    if total_width <= available_width {
        return;
    }

    let overflow = total_width - available_width;
    let shrinkable = widths
        .iter()
        .map(|width| (*width - min_width).max(0.0))
        .sum::<f32>();

    if shrinkable <= f32::EPSILON {
        let equal = ((available_width - total_gap) / widths.len() as f32).max(0.0);

        for width in widths {
            *width = equal;
        }

        return;
    }

    for width in widths {
        let share = (*width - min_width).max(0.0) / shrinkable;
        *width = (*width - overflow * share).max(min_width);
    }
}

fn estimate_label_width(label: &str, size: f32) -> f32 {
    label.chars().map(|ch| glyph_width(ch, size)).sum()
}

fn glyph_width(ch: char, size: f32) -> f32 {
    match ch {
        ' ' => size * 0.35,
        'i' | 'l' | 'I' | '!' | '|' | '.' | ',' | ':' | ';' => size * 0.36,
        'm' | 'w' | 'M' | 'W' => size * 0.82,
        c if c.is_ascii_digit() => size * 0.56,
        c if c.is_ascii_uppercase() => size * 0.68,
        _ => size * 0.58,
    }
}
