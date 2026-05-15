use super::theme::{
    AdaptiveGrid, ComponentState, TextRole, UiAnchor, UiTheme, UiViewport, UserZoom,
};

#[test]
fn voxelverse_theme_declares_shared_hotbar_shape() {
    let theme = UiTheme::VOXELVERSE;

    assert_eq!(theme.hotbar.slots, 9);
    assert_eq!(theme.inventory_grid.gap, theme.spacing.slot_gap + 1.0);
    assert!(theme.hotbar.slot_size_min <= theme.spacing.slot_size);
    assert!(theme.spacing.slot_size <= theme.hotbar.slot_size_max);
}

#[test]
fn selected_slot_uses_stronger_border_than_normal_slot() {
    let slot = UiTheme::VOXELVERSE.slot;

    assert_ne!(
        slot.border_for(ComponentState::Normal),
        slot.border_for(ComponentState::Selected)
    );
    assert!(slot.selected_border_width > slot.border_width);
}

#[test]
fn slot_distinguishes_every_gameplay_state() {
    let slot = UiTheme::VOXELVERSE.slot;
    let states = [
        ComponentState::Normal,
        ComponentState::Hovered,
        ComponentState::Pressed,
        ComponentState::Selected,
        ComponentState::Empty,
        ComponentState::Disabled,
        ComponentState::Invalid,
        ComponentState::Success,
        ComponentState::Alert,
    ];
    for (i, a) in states.iter().enumerate() {
        for b in states.iter().skip(i + 1) {
            assert_ne!(
                (slot.fill_for(*a), slot.border_for(*a)),
                (slot.fill_for(*b), slot.border_for(*b)),
                "slot states {:?} and {:?} look identical",
                a,
                b
            );
        }
    }
}

#[test]
fn button_supplies_a_color_for_every_state() {
    let button = UiTheme::VOXELVERSE.button;
    for state in [
        ComponentState::Normal,
        ComponentState::Hovered,
        ComponentState::Pressed,
        ComponentState::Selected,
        ComponentState::Disabled,
        ComponentState::Empty,
        ComponentState::Invalid,
        ComponentState::Success,
        ComponentState::Alert,
    ] {
        let _ = button.fill_for(state);
        let _ = button.text_for(state);
    }
}

#[test]
fn viewport_scaling_clamps_in_sensible_range() {
    let theme = UiTheme::VOXELVERSE;
    let baseline = theme.auto_scale(UiViewport::new(1920.0, 1080.0));
    assert!((baseline - 1.0).abs() < f32::EPSILON);
    assert!(theme.auto_scale(UiViewport::new(1280.0, 720.0)) >= 0.75);
    assert!(theme.auto_scale(UiViewport::new(3840.0, 2160.0)) <= 1.60);
}

#[test]
fn effective_scale_combines_auto_and_user_zoom() {
    let theme = UiTheme::VOXELVERSE;
    let vp = UiViewport::new(1920.0, 1080.0);
    assert!((theme.effective_scale(vp, UserZoom::Normal) - 1.0).abs() < 1e-4);
    assert!((theme.effective_scale(vp, UserZoom::XXLarge) - 1.5).abs() < 1e-4);
    let big = UiViewport::new(3840.0, 2160.0);
    assert!(theme.effective_scale(big, UserZoom::XXLarge) <= 2.20 + 1e-4);
}

#[test]
fn user_zoom_presets_cover_required_steps() {
    let factors: Vec<f32> = UserZoom::ALL.iter().map(|z| z.factor()).collect();
    assert_eq!(factors, vec![0.90, 1.00, 1.10, 1.25, 1.50]);
}

#[test]
fn hotbar_slot_size_obeys_clamps() {
    let theme = UiTheme::VOXELVERSE;
    let tiny = theme.hotbar_slot_size(UiViewport::new(1280.0, 720.0), UserZoom::Small);
    let huge = theme.hotbar_slot_size(UiViewport::new(3840.0, 2160.0), UserZoom::XXLarge);
    assert!(tiny > 0.0);
    assert!(huge * 9.0 < 3840.0 * 0.95);
}

#[test]
fn panel_constraints_never_overflow_viewport() {
    let theme = UiTheme::VOXELVERSE;
    let vp = UiViewport::new(1024.0, 600.0);
    let (w, h) = theme.responsive.inventory_panel.resolve(
        vp,
        theme.effective_scale(vp, UserZoom::Normal),
        2000.0,
        2000.0,
    );
    assert!(w <= vp.width * theme.responsive.inventory_panel.max_width_ratio + 1.0);
    assert!(h <= vp.height * theme.responsive.inventory_panel.max_height_ratio + 1.0);
}

#[test]
fn adaptive_grid_adapts_columns_to_available_width() {
    let grid = AdaptiveGrid {
        min_columns: 5,
        preferred_columns: 8,
        max_columns: 12,
    };
    assert_eq!(grid.columns_for(2000.0, 58.0, 8.0), 12);
    assert_eq!(grid.columns_for(8.0 * 58.0 + 7.0 * 8.0, 58.0, 8.0), 8);
    assert_eq!(grid.columns_for(100.0, 58.0, 8.0), 5);
}

#[test]
fn anchor_origin_keeps_blocks_inside_viewport() {
    let theme = UiTheme::VOXELVERSE;
    let vp = UiViewport::new(1920.0, 1080.0);
    let (x, y) = theme.anchor_origin(vp, UiAnchor::BottomCenter, (400.0, 80.0), 30.0);
    assert!((x - (1920.0 - 400.0) * 0.5).abs() < 1.0);
    assert!((y - (1080.0 - 80.0 - 30.0)).abs() < 1.0);
    let (x, y) = theme.anchor_origin(vp, UiAnchor::Center, (600.0, 400.0), 0.0);
    assert!((x - 660.0).abs() < 1.0);
    assert!((y - 340.0).abs() < 1.0);
}

#[test]
fn text_roles_all_resolve_to_a_size_and_color() {
    let text = UiTheme::VOXELVERSE.text;
    for role in [
        TextRole::Title,
        TextRole::Section,
        TextRole::Body,
        TextRole::Muted,
        TextRole::Badge,
        TextRole::Notice,
        TextRole::Control,
    ] {
        assert!(text.size_for(role) > 0.0);
        let _ = text.color_for(role);
    }
}

#[test]
fn motion_durations_stay_short_enough_to_not_delay_gameplay() {
    let m = UiTheme::VOXELVERSE.motion;
    assert!(m.slot_select_ms <= 150.0);
    assert!(m.slot_invalid_flash_ms <= 200.0);
    assert!(m.panel_open_ms <= 200.0);
    assert!(m.panel_close_ms <= m.panel_open_ms);
    assert!(m.tooltip_delay_ms >= 150.0);
}
