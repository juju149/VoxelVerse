use super::inventory::{selected_recipe_index, InventoryTextSpec};
use super::inventory_geometry::equip_slot_rects;
use crate::snapshot::{
    RenderCraftSnapshot, RenderHotbarSnapshot, RenderInventorySnapshot, RenderInventoryUiSnapshot,
};
use crate::ui::{InventoryLayout, UiTheme};

// =============================================================================
// Text spec helpers
// =============================================================================

pub(super) fn push_header_text(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
) {
    specs.push(InventoryTextSpec {
        text: "INVENTAIRE".to_string(),
        left: layout.title_origin.0,
        top: layout.title_origin.1,
        size: theme.text.title_size * layout.scale,
        color: theme.text.title.as_rgb8(),
    });
    specs.push(InventoryTextSpec {
        text: "Sac, equipement et ressources".to_string(),
        left: layout.subtitle_origin.0,
        top: layout.subtitle_origin.1,
        size: theme.text.muted_size * layout.scale,
        color: theme.text.muted.as_rgb8(),
    });
}

pub(super) fn push_column_titles(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
) {
    let size = theme.text.section_size * layout.scale;
    specs.push(InventoryTextSpec {
        text: "EQUIPEMENT".to_string(),
        left: layout.left_title_origin.0,
        top: layout.left_title_origin.1,
        size,
        color: theme.text.section.as_rgb8(),
    });
    specs.push(InventoryTextSpec {
        text: "SAC A DOS".to_string(),
        left: layout.center_title_origin.0,
        top: layout.center_title_origin.1,
        size,
        color: theme.text.section.as_rgb8(),
    });
    specs.push(InventoryTextSpec {
        text: "ARTISANAT RAPIDE".to_string(),
        left: layout.right_title_origin.0,
        top: layout.right_title_origin.1,
        size,
        color: theme.text.section.as_rgb8(),
    });
    specs.push(InventoryTextSpec {
        text: "Recettes issues du pack actif".to_string(),
        left: layout.right_subtitle_origin.0,
        top: layout.right_subtitle_origin.1,
        size: theme.text.muted_size * layout.scale,
        color: theme.text.muted.as_rgb8(),
    });
}

pub(super) fn push_search_text(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    ui: &RenderInventoryUiSnapshot,
) {
    let (text, color) = if ui.search_query.is_empty() {
        (
            "Rechercher un objet...".to_string(),
            theme.search_bar.placeholder.as_rgb8(),
        )
    } else {
        let suffix = if ui.search_focused { "_" } else { "" };
        (
            format!("{}{}", ui.search_query, suffix),
            theme.search_bar.text.as_rgb8(),
        )
    };
    specs.push(InventoryTextSpec {
        text,
        left: layout.search_text_origin.0,
        top: layout.search_text_origin.1,
        size: theme.text.body_size * layout.scale,
        color,
    });
}

pub(super) fn push_filter_text(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    ui: &RenderInventoryUiSnapshot,
) {
    let size = theme.text.control_size * layout.scale;
    for (filter, rect) in &layout.filter_chips {
        let selected = *filter == ui.active_filter;
        let color = if selected {
            theme.filter_chip.text_selected
        } else {
            theme.filter_chip.text
        };
        // Center label horizontally inside the chip.
        let label = filter.label();
        let est_w = label.chars().count() as f32 * size * 0.62;
        let left = rect.x + (rect.w - est_w) * 0.5;
        let top = rect.y + (rect.h - size) * 0.5 - 2.0;
        specs.push(InventoryTextSpec {
            text: label.to_string(),
            left,
            top,
            size,
            color: color.as_rgb8(),
        });
    }
}

pub(super) fn push_quantity_badges(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    inventory: &RenderInventorySnapshot,
) {
    let font = theme.quantity_badge.font_size * layout.scale;
    for (i, rect) in layout.inventory_slots.iter().enumerate() {
        let Some(slot) = inventory.slots[i] else {
            continue;
        };
        if slot.quantity <= 1 {
            continue;
        }
        let digits = slot.quantity.to_string().chars().count() as f32;
        let badge_w = (digits * font * 0.6 + 10.0 * layout.scale).max(font * 1.4);
        let badge_h = font + 6.0 * layout.scale;
        let badge_x = rect.x + rect.w - 4.0 * layout.scale - badge_w;
        let badge_y = rect.y + rect.h - 3.0 * layout.scale - badge_h;
        specs.push(InventoryTextSpec {
            text: slot.quantity.to_string(),
            left: badge_x + 5.0 * layout.scale,
            top: badge_y + 2.0 * layout.scale,
            size: font,
            color: theme.quantity_badge.text.as_rgb8(),
        });
    }
}

pub(super) fn push_weight_text(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    ui: &RenderInventoryUiSnapshot,
    inventory: &RenderInventorySnapshot,
    _hotbar: &RenderHotbarSnapshot,
) {
    let total_kg = total_weight_kg(inventory, ui);
    let label = format!("{:.1} / {:.0} kg", total_kg, ui.capacity_kg);
    specs.push(InventoryTextSpec {
        text: label,
        left: layout.weight_bar_label_origin.0,
        top: layout.weight_bar_label_origin.1,
        size: theme.text.muted_size * layout.scale,
        color: theme.text.muted.as_rgb8(),
    });
}

pub(super) fn push_sort_text(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
) {
    let size = theme.text.control_size * layout.scale;
    let label = "Trier";
    let est_w = label.chars().count() as f32 * size * 0.6;
    specs.push(InventoryTextSpec {
        text: label.to_string(),
        left: layout.sort_button.x + (layout.sort_button.w - est_w) * 0.5,
        top: layout.sort_text_origin.1,
        size,
        color: theme.button.text.as_rgb8(),
    });
}

pub(super) fn push_equipment_placeholder(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
) {
    // Label each equipment slot to its right: Casque / Plastron / Bottes.
    let size = theme.text.muted_size * layout.scale;
    let slots = equip_slot_rects(layout.left_panel, layout.scale);
    for (rect, label) in &slots {
        let lx = rect.x + rect.w + 8.0 * layout.scale;
        let ly = rect.y + (rect.h - size) * 0.5;
        specs.push(InventoryTextSpec {
            text: label.to_string(),
            left: lx,
            top: ly,
            size,
            color: theme.text.muted.as_rgb8(),
        });
    }
    // Suppress unused-field warning.
    let _ = layout.equipment_placeholder_origin;
}

pub(super) fn push_craft_text(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    ui: &RenderInventoryUiSnapshot,
    craft: &RenderCraftSnapshot,
) {
    push_recipe_rows(specs, theme, layout, ui, craft);
    push_recipe_detail(specs, theme, layout, ui, craft);
}

fn push_recipe_rows(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    ui: &RenderInventoryUiSnapshot,
    craft: &RenderCraftSnapshot,
) {
    if craft.recipes.is_empty() {
        specs.push(InventoryTextSpec {
            text: "Aucune recette chargee".to_string(),
            left: layout.right_panel.x + 18.0 * layout.scale,
            top: layout.right_panel.y + 78.0 * layout.scale,
            size: theme.text.body_size * layout.scale,
            color: theme.text.muted.as_rgb8(),
        });
        return;
    }

    let selected = selected_recipe_index(ui, craft);
    let size = theme.text.control_size * layout.scale;
    for (row, recipe) in layout.craft_recipe_rows.iter().zip(craft.recipes.iter()) {
        let color = if Some(recipe.index) == selected {
            theme.filter_chip.text_selected
        } else {
            theme.filter_chip.text
        };
        specs.push(InventoryTextSpec {
            text: recipe.output_name.clone(),
            left: row.x + row.h + 12.0 * layout.scale,
            top: row.y + (row.h - size) * 0.5 - 2.0 * layout.scale,
            size,
            color: color.as_rgb8(),
        });
    }
}

fn push_recipe_detail(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    ui: &RenderInventoryUiSnapshot,
    craft: &RenderCraftSnapshot,
) {
    let Some(recipe) = craft.selected_recipe.as_ref() else {
        return;
    };

    let title_size = theme.text.section_size * layout.scale;
    let body_size = theme.text.body_size * layout.scale;
    let muted_size = theme.text.muted_size * layout.scale;
    specs.push(InventoryTextSpec {
        text: recipe.output_name.to_uppercase(),
        left: layout.craft_output_slot.x + layout.craft_output_slot.w + 12.0 * layout.scale,
        top: layout.craft_output_slot.y + 2.0 * layout.scale,
        size: title_size,
        color: theme.text.section.as_rgb8(),
    });
    specs.push(InventoryTextSpec {
        text: recipe.station_label.clone(),
        left: layout.craft_output_slot.x + layout.craft_output_slot.w + 12.0 * layout.scale,
        top: layout.craft_output_slot.y + title_size + 8.0 * layout.scale,
        size: muted_size,
        color: theme.text.muted.as_rgb8(),
    });

    for (row, ingredient) in layout.craft_ingredient_rows.iter().zip(&recipe.ingredients) {
        specs.push(InventoryTextSpec {
            text: ingredient.label.clone(),
            left: row.x + row.h + 12.0 * layout.scale,
            top: row.y + (row.h - muted_size) * 0.5 - 2.0 * layout.scale,
            size: muted_size,
            color: theme.text.body.as_rgb8(),
        });
        let qty = format!("{} / craft", ingredient.count);
        let qty_w = qty.chars().count() as f32 * muted_size * 0.58;
        specs.push(InventoryTextSpec {
            text: qty,
            left: row.x + row.w - qty_w - 8.0 * layout.scale,
            top: row.y + (row.h - muted_size) * 0.5 - 2.0 * layout.scale,
            size: muted_size,
            color: theme.text.muted.as_rgb8(),
        });
    }

    push_centered_text(
        specs,
        "-",
        layout.craft_quantity_down,
        body_size,
        theme.button.text.as_rgb8(),
    );
    push_centered_text(
        specs,
        "+",
        layout.craft_quantity_up,
        body_size,
        theme.button.text.as_rgb8(),
    );
    push_centered_text(
        specs,
        "Max",
        layout.craft_max_button,
        body_size,
        theme.button.text.as_rgb8(),
    );
    push_centered_text(
        specs,
        &ui.craft_quantity.to_string(),
        layout.craft_quantity_value,
        body_size,
        theme.text.body.as_rgb8(),
    );
    push_centered_text(
        specs,
        "FABRIQUER",
        layout.craft_button,
        theme.text.section_size * layout.scale,
        theme.button.text.as_rgb8(),
    );
}

fn push_centered_text(
    specs: &mut Vec<InventoryTextSpec>,
    text: &str,
    rect: crate::ui::UiRect,
    size: f32,
    color: [u8; 3],
) {
    let est_w = text.chars().count() as f32 * size * 0.60;
    specs.push(InventoryTextSpec {
        text: text.to_string(),
        left: rect.x + (rect.w - est_w) * 0.5,
        top: rect.y + (rect.h - size) * 0.5 - 2.0,
        size,
        color,
    });
}

pub(super) fn push_held_quantity(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    ui: &RenderInventoryUiSnapshot,
) {
    let Some(held) = ui.held else { return };
    if held.stack.quantity <= 1 {
        return;
    }
    let ghost = layout.inventory_slot_size * 0.85;
    let font = theme.quantity_badge.font_size * layout.scale;
    let digits = held.stack.quantity.to_string().chars().count() as f32;
    let badge_w = (digits * font * 0.6 + 10.0 * layout.scale).max(font * 1.4);
    let badge_h = font + 6.0 * layout.scale;
    let badge_x = ui.cursor.0 + ghost * 0.5 - 4.0 * layout.scale - badge_w;
    let badge_y = ui.cursor.1 + ghost * 0.5 - 3.0 * layout.scale - badge_h;
    specs.push(InventoryTextSpec {
        text: held.stack.quantity.to_string(),
        left: badge_x + 5.0 * layout.scale,
        top: badge_y + 2.0 * layout.scale,
        size: font,
        color: theme.quantity_badge.text.as_rgb8(),
    });
}

pub(super) fn push_footer_keys(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
) {
    let size = theme.text.muted_size * layout.scale;
    specs.push(InventoryTextSpec {
        text: "E/ECHAP fermer  -  Clic: prendre/poser  -  Clic droit: 1/2  -  Maj+clic: deplacer  -  1-9: hotbar  -  Q: jeter".to_string(),
        left: layout.modal.x + 18.0 * layout.scale,
        top: layout.modal.y + layout.modal.h - size - 10.0 * layout.scale,
        size,
        color: theme.text.muted.as_rgb8(),
    });
}

pub(super) fn total_weight_kg(
    inventory: &RenderInventorySnapshot,
    ui: &RenderInventoryUiSnapshot,
) -> f32 {
    let count = inventory.total_count as f32;
    let held = ui.held.map(|h| h.stack.quantity as f32).unwrap_or(0.0);
    (count + held) * crate::ui::InventoryUiState::UNIT_WEIGHT_KG
}
