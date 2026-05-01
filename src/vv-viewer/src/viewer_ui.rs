/// egui UI panels for the block viewer.
/// This file contains pure egui drawing logic — no GPU state.
use egui::{Context, ScrollArea, TextEdit};
use vv_registry::{CompiledBlock, CompiledContent, ContentKey};

use crate::args::{Scene, ViewerState};
use crate::block_selector::{BlockEntry, BlockSelector};
use crate::debug_mode::DebugMode;

/// Actions signalled by the UI during a frame.
#[derive(Default)]
pub struct UiActions {
    pub new_block: Option<ContentKey>,
    pub scene_changed: bool,
    pub reload_requested: bool,
    pub screenshot_requested: bool,
}

/// Draw all viewer UI panels.
pub fn draw(
    ctx: &Context,
    state: &mut ViewerState,
    selector: &mut BlockSelector,
    content: &CompiledContent,
    actions: &mut UiActions,
) {
    draw_left_panel(ctx, selector, actions);
    draw_right_panel(ctx, state, selector, content, actions);
}

// --------------------------------------------------------------------------

fn draw_left_panel(ctx: &Context, selector: &mut BlockSelector, actions: &mut UiActions) {
    egui::SidePanel::left("block_list")
        .resizable(true)
        .min_width(180.0)
        .default_width(220.0)
        .show(ctx, |ui| {
            ui.strong("Blocks");
            ui.separator();

            // Search bar
            let prev_search = selector.search.clone();
            ui.add(
                TextEdit::singleline(&mut selector.search)
                    .hint_text("Search…")
                    .desired_width(f32::INFINITY),
            );
            if selector.search != prev_search {
                selector.update_filter();
            }

            // Namespace filter
            let namespaces = selector.all_namespaces();
            if namespaces.len() > 1 {
                let prev_ns = selector.namespace_filter.clone();
                egui::ComboBox::from_id_source("ns_filter")
                    .selected_text(if selector.namespace_filter.is_empty() {
                        "All namespaces".to_string()
                    } else {
                        selector.namespace_filter.clone()
                    })
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(selector.namespace_filter.is_empty(), "All")
                            .clicked()
                        {
                            selector.namespace_filter.clear();
                        }
                        for ns in &namespaces {
                            if ui
                                .selectable_label(&selector.namespace_filter == ns, ns)
                                .clicked()
                            {
                                selector.namespace_filter = ns.clone();
                            }
                        }
                    });
                if selector.namespace_filter != prev_ns {
                    selector.update_filter();
                }
            }

            ui.label(
                egui::RichText::new(format!("{} blocks", selector.filtered_indices.len()))
                    .small()
                    .weak(),
            );
            ui.separator();

            let scroll_to = selector.scroll_to_selected;
            selector.scroll_to_selected = false;

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let filtered = selector.filtered_indices.clone();
                    for &global_idx in &filtered {
                        let entry = &selector.all_blocks[global_idx];
                        let is_selected = selector.selected_idx == Some(global_idx);

                        let [r, g, b] = entry.base_color;
                        let swatch = egui::Color32::from_rgb(
                            (r * 255.0) as u8,
                            (g * 255.0) as u8,
                            (b * 255.0) as u8,
                        );

                        let resp = ui
                            .horizontal(|ui| {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(10.0, 10.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(rect, 1.0, swatch);
                                ui.selectable_label(
                                    is_selected,
                                    format!("{}:{}", entry.namespace, entry.key.name()),
                                )
                            })
                            .inner;

                        if resp.clicked() {
                            selector.selected_idx = Some(global_idx);
                            actions.new_block = Some(entry.key.clone());
                        }
                        if scroll_to && is_selected {
                            resp.scroll_to_me(Some(egui::Align::Center));
                        }
                    }
                });
        });
}

fn draw_right_panel(
    ctx: &Context,
    state: &mut ViewerState,
    selector: &BlockSelector,
    content: &CompiledContent,
    actions: &mut UiActions,
) {
    egui::SidePanel::right("controls")
        .resizable(true)
        .min_width(200.0)
        .default_width(260.0)
        .show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // ── Scene ──
                    ui.strong("Scene");
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        for &scene in Scene::ALL {
                            if ui
                                .selectable_label(state.scene == scene, scene.label())
                                .clicked()
                            {
                                state.scene = scene;
                                actions.scene_changed = true;
                            }
                        }
                    });
                    let tt_text = if state.turntable {
                        "⏸ Stop"
                    } else {
                        "▶ Turntable"
                    };
                    if ui.button(tt_text).clicked() {
                        state.turntable = !state.turntable;
                    }

                    // ── Debug Mode ──
                    ui.add_space(6.0);
                    ui.strong("Debug Mode");
                    ui.separator();
                    for &mode in DebugMode::ALL {
                        if ui
                            .selectable_label(state.debug_mode == mode, mode.label())
                            .clicked()
                        {
                            state.debug_mode = mode;
                        }
                    }
                    ui.separator();
                    ui.checkbox(&mut state.show_grid, "Grid  [G]");

                    // ── Preview Sliders ──
                    ui.add_space(6.0);
                    ui.strong("Preview");
                    ui.separator();
                    egui::Grid::new("sliders")
                        .num_columns(2)
                        .spacing([6.0, 3.0])
                        .show(ui, |ui| {
                            slider_row(ui, "Exposure", &mut state.exposure, 0.1, 3.0);
                            slider_row(ui, "Variation", &mut state.variation_scale, 0.0, 2.0);
                            slider_row(ui, "Edge Mult", &mut state.edge_strength_mult, 0.0, 2.0);
                            slider_row(ui, "AO Mult", &mut state.ao_mult, 0.0, 2.0);
                            slider_row(ui, "Bevel Mult", &mut state.bevel_mult, 0.0, 3.0);
                            slider_row(ui, "Macro Str", &mut state.macro_strength_mult, 0.0, 2.0);
                            slider_row(ui, "Micro Str", &mut state.micro_strength_mult, 0.0, 2.0);
                        });
                    if ui.small_button("Reset sliders").clicked() {
                        state.reset_sliders();
                    }

                    // ── Block Info ──
                    if let Some(entry) = selector.selected() {
                        if let Some(block) = content.blocks.get(entry.block_id) {
                            ui.add_space(6.0);
                            ui.strong("Block Info");
                            ui.separator();
                            draw_block_info(ui, entry, block);
                        }
                    }

                    // ── Actions ──
                    ui.add_space(8.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("⟳ Reload  [R]").clicked() {
                            actions.reload_requested = true;
                        }
                        if ui.button("📷 Screenshot  [S]").clicked() {
                            actions.screenshot_requested = true;
                        }
                    });
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "Keys: 1-9 debug · G grid · R reload · S screenshot · F reset cam",
                        )
                        .small()
                        .weak(),
                    );
                });
        });
}

fn slider_row(ui: &mut egui::Ui, label: &str, val: &mut f32, min: f32, max: f32) {
    ui.label(egui::RichText::new(label).small());
    ui.add(egui::Slider::new(val, min..=max).show_value(true));
    ui.end_row();
}

fn draw_block_info(ui: &mut egui::Ui, entry: &BlockEntry, block: &CompiledBlock) {
    egui::Grid::new("block_info")
        .num_columns(2)
        .spacing([6.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            info_row(
                ui,
                "Key",
                &format!("{}:{}", entry.namespace, entry.key.name()),
            );
            if let Some(dn) = &block.display_key {
                info_row(ui, "Display", dn);
            }
            info_row(ui, "Block ID", &entry.block_id.raw().to_string());
            info_row(ui, "Visual ID", &block.render.visual_id.raw().to_string());

            info_row(ui, "Hardness", &format!("{:.2}", block.mining.hardness));
            info_row(ui, "Tool", &format!("{:?}", block.mining.tool));
            info_row(ui, "Tier", &block.mining.tool_tier_min.to_string());

            info_row(ui, "Phase", &format!("{:?}", block.physics.phase));
            info_row(ui, "Density", &format!("{:.2}", block.physics.density));

            let [r, g, b] = block.render.color;
            info_row(
                ui,
                "Color",
                &format!(
                    "#{:02X}{:02X}{:02X}",
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8
                ),
            );
            info_row(ui, "Roughness", &format!("{:.2}", block.render.roughness));
            info_row(ui, "Metallic", &format!("{:.2}", block.render.metallic));
            info_row(ui, "Alpha", &format!("{:.2}", block.render.alpha));
            info_row(
                ui,
                "Render Mode",
                &format!("{:?}", block.render.render_mode),
            );
            info_row(ui, "Shape", &format!("{:?}", block.render.shape));
            info_row(ui, "Tint", &format!("{:?}", block.render.tint));
            info_row(ui, "Occludes", &block.render.meshing.occludes.to_string());
            info_row(ui, "Greedy", &block.render.meshing.greedy_merge.to_string());
            info_row(
                ui,
                "Shadows",
                &block.render.meshing.casts_shadow.to_string(),
            );
            info_row(ui, "Recv AO", &block.render.meshing.receives_ao.to_string());

            if !entry.tags.is_empty() {
                info_row(ui, "Tags", &entry.tags.join(", "));
            }
        });
}

fn info_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.label(egui::RichText::new(label).small().weak());
    ui.label(egui::RichText::new(value).small());
    ui.end_row();
}
