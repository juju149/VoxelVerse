#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiColor {
    pub rgb: [f32; 3],
    pub alpha: f32,
}

impl UiColor {
    pub const fn rgba(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        Self {
            rgb: [r, g, b],
            alpha,
        }
    }

    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self {
            rgb: [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0],
            alpha: 1.0,
        }
    }

    pub fn scale_rgb(self, factor: f32) -> Self {
        Self {
            rgb: [
                (self.rgb[0] * factor).min(1.0),
                (self.rgb[1] * factor).min(1.0),
                (self.rgb[2] * factor).min(1.0),
            ],
            alpha: self.alpha,
        }
    }

    pub fn as_rgb(self) -> [f32; 3] {
        self.rgb
    }

    pub fn as_rgb8(self) -> [u8; 3] {
        [
            (self.rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        ]
    }
}

/// Visual states every interactive UI element must be able to render.
///
/// `Empty`, `Invalid`, `Success` and `Alert` are gameplay-driven states
/// (no item, can't place, item picked up, low durability...) and must
/// remain visually distinct from `Disabled` (input blocked).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentState {
    Normal,
    Hovered,
    Pressed,
    Selected,
    Disabled,
    Empty,
    Invalid,
    Success,
    Alert,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextRole {
    Title,
    Section,
    Body,
    Muted,
    Badge,
    Notice,
    Control,
}

/// Geometry tokens shared by every gameplay UI screen.
///
/// Sizes are expressed at the 1080p baseline. Use
/// [`UiTheme::scale_for_viewport`] to multiply them when rendering on
/// other resolutions so the interface stays proportional.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiSpacing {
    // Generic spacing scale used for paddings, gaps, and inset offsets.
    pub tiny: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub xlarge: f32,
    pub panel: f32,

    // Slot grid geometry. `slot_size` is the canonical square; the hotbar
    // is allowed to clamp inside [slot_size_hotbar_min, slot_size_hotbar_max].
    pub slot_size: f32,
    pub slot_size_hotbar_min: f32,
    pub slot_size_hotbar_max: f32,
    pub slot_gap: f32,
    pub slot_row_gap: f32,

    // Content inside a slot.
    pub icon_size: f32,
    pub icon_inset: f32,
    pub badge_diameter: f32,

    // Interactive controls.
    pub button_height: f32,
    pub control_height: f32,
    pub chip_height: f32,

    // Stroke widths used everywhere.
    pub border_thin: f32,
    pub border_medium: f32,
    pub border_thick: f32,

    // Corner radii.
    pub radius_slot: f32,
    pub radius_panel: f32,
    pub radius_control: f32,
    pub radius_pill: f32,

    // Hotbar placement clamps. Both are expressed in CSS-like pixels.
    pub hotbar_bottom_margin_min: f32,
    pub hotbar_bottom_margin_max: f32,

    // Tooltip placement.
    pub tooltip_offset: f32,
    pub tooltip_padding: f32,
    pub tooltip_max_width: f32,
}

/// Animation tokens. All durations are expressed in milliseconds and must
/// stay short enough not to delay gameplay actions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMotion {
    pub slot_select_ms: f32,
    pub slot_pickup_ms: f32,
    pub slot_invalid_flash_ms: f32,
    pub slot_success_flash_ms: f32,
    pub panel_open_ms: f32,
    pub panel_close_ms: f32,
    pub filter_swap_ms: f32,
    pub tooltip_delay_ms: f32,
    pub tooltip_fade_ms: f32,
    pub notice_fade_in_ms: f32,
    pub notice_hold_ms: f32,
    pub notice_fade_out_ms: f32,
    /// Cubic-bezier control points (x1, y1, x2, y2) for the default ease-out.
    pub ease_out: [f32; 4],
    pub ease_in: [f32; 4],
    pub ease_in_out: [f32; 4],
}

/// Readability guarantees the renderer must honor regardless of the world
/// behind the UI (snow, forest, cave, sky).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiReadability {
    /// Subtle darkening drawn behind floating text (notices, badges) so it
    /// stays legible over bright biomes.
    pub text_scrim: UiColor,
    /// Color of the 1px outline applied to small floating text.
    pub text_outline: UiColor,
    pub text_outline_width: f32,
    /// Minimum opacity any non-empty slot is allowed to drop to.
    pub min_slot_alpha: f32,
    /// Alpha multiplier applied to the world-blocking scrim shown behind
    /// modal panels (inventory, craft, chest...).
    pub modal_scrim: UiColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelStyle {
    pub fill: UiColor,
    pub border: UiColor,
    pub border_strong: UiColor,
    pub shadow: UiColor,
    pub radius: f32,
    pub border_width: f32,
    pub padding: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlotStyle {
    pub fill: UiColor,
    pub fill_hovered: UiColor,
    pub fill_pressed: UiColor,
    pub fill_selected: UiColor,
    pub fill_empty: UiColor,
    pub fill_disabled: UiColor,
    pub fill_invalid: UiColor,
    pub fill_success: UiColor,
    pub fill_alert: UiColor,

    pub inner_fill: UiColor,

    pub border: UiColor,
    pub border_hovered: UiColor,
    pub border_pressed: UiColor,
    pub border_selected: UiColor,
    pub border_empty: UiColor,
    pub border_disabled: UiColor,
    pub border_invalid: UiColor,
    pub border_success: UiColor,
    pub border_alert: UiColor,

    pub highlight: UiColor,
    pub content_glint: UiColor,

    pub radius: f32,
    pub border_width: f32,
    pub selected_border_width: f32,
    pub inner_inset: f32,
    pub icon_inset: f32,
}

impl SlotStyle {
    pub fn fill_for(self, state: ComponentState) -> UiColor {
        match state {
            ComponentState::Normal => self.fill,
            ComponentState::Hovered => self.fill_hovered,
            ComponentState::Pressed => self.fill_pressed,
            ComponentState::Selected => self.fill_selected,
            ComponentState::Empty => self.fill_empty,
            ComponentState::Disabled => self.fill_disabled,
            ComponentState::Invalid => self.fill_invalid,
            ComponentState::Success => self.fill_success,
            ComponentState::Alert => self.fill_alert,
        }
    }

    pub fn border_for(self, state: ComponentState) -> UiColor {
        match state {
            ComponentState::Normal => self.border,
            ComponentState::Hovered => self.border_hovered,
            ComponentState::Pressed => self.border_pressed,
            ComponentState::Selected => self.border_selected,
            ComponentState::Empty => self.border_empty,
            ComponentState::Disabled => self.border_disabled,
            ComponentState::Invalid => self.border_invalid,
            ComponentState::Success => self.border_success,
            ComponentState::Alert => self.border_alert,
        }
    }

    pub fn border_width_for(self, state: ComponentState) -> f32 {
        match state {
            ComponentState::Selected | ComponentState::Invalid | ComponentState::Success => {
                self.selected_border_width
            }
            _ => self.border_width,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonStyle {
    pub fill: UiColor,
    pub fill_hovered: UiColor,
    pub fill_pressed: UiColor,
    pub fill_selected: UiColor,
    pub fill_disabled: UiColor,
    pub fill_alert: UiColor,
    pub fill_success: UiColor,
    pub border: UiColor,
    pub border_hovered: UiColor,
    pub border_disabled: UiColor,
    pub text: UiColor,
    pub text_disabled: UiColor,
    pub text_alert: UiColor,
    pub text_success: UiColor,
    pub height: f32,
    pub padding_x: f32,
    pub border_width: f32,
    pub radius: f32,
}

impl ButtonStyle {
    pub fn fill_for(self, state: ComponentState) -> UiColor {
        match state {
            ComponentState::Normal | ComponentState::Empty => self.fill,
            ComponentState::Hovered => self.fill_hovered,
            ComponentState::Pressed => self.fill_pressed,
            ComponentState::Selected => self.fill_selected,
            ComponentState::Disabled => self.fill_disabled,
            ComponentState::Invalid | ComponentState::Alert => self.fill_alert,
            ComponentState::Success => self.fill_success,
        }
    }

    pub fn text_for(self, state: ComponentState) -> UiColor {
        match state {
            ComponentState::Disabled => self.text_disabled,
            ComponentState::Invalid | ComponentState::Alert => self.text_alert,
            ComponentState::Success => self.text_success,
            _ => self.text,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchBarStyle {
    pub fill: UiColor,
    pub fill_focused: UiColor,
    pub fill_disabled: UiColor,
    pub border: UiColor,
    pub border_focused: UiColor,
    pub border_disabled: UiColor,
    pub placeholder: UiColor,
    pub text: UiColor,
    pub text_disabled: UiColor,
    pub icon: UiColor,
    pub height: f32,
    pub padding_x: f32,
    pub border_width: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilterChipStyle {
    pub fill: UiColor,
    pub fill_hovered: UiColor,
    pub fill_selected: UiColor,
    pub fill_disabled: UiColor,
    pub border: UiColor,
    pub border_hovered: UiColor,
    pub border_selected: UiColor,
    pub border_disabled: UiColor,
    pub text: UiColor,
    pub text_selected: UiColor,
    pub text_disabled: UiColor,
    pub height: f32,
    pub padding_x: f32,
    pub border_width: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InventoryGridStyle {
    pub columns: usize,
    pub slot_size: f32,
    pub gap: f32,
    pub row_gap: f32,
    pub empty_slot_fill: UiColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HotbarStyle {
    pub slots: usize,
    pub slot_size_min: f32,
    pub slot_size_max: f32,
    pub slot_height_ratio: f32,
    pub panel_padding: f32,
    pub slot_gap: f32,
    pub notice_offset_y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuantityBadgeStyle {
    pub text: UiColor,
    pub text_alert: UiColor,
    pub text_full: UiColor,
    pub shadow: UiColor,
    pub font_size: f32,
    pub line_height: f32,
    pub right_inset: f32,
    pub bottom_inset: f32,
}

/// Style for transient player-facing notices (e.g. "Item picked up",
/// "Inventory full", "Cannot place here"). Distinct from `QuantityBadgeStyle`
/// even though both render small floating text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerNoticeStyle {
    pub info_text: UiColor,
    pub success_text: UiColor,
    pub alert_text: UiColor,
    pub invalid_text: UiColor,
    pub muted_text: UiColor,
    pub shadow: UiColor,
    pub font_size: f32,
    pub line_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TooltipStyle {
    pub fill: UiColor,
    pub border: UiColor,
    pub shadow: UiColor,
    pub title_text: UiColor,
    pub body_text: UiColor,
    pub muted_text: UiColor,
    pub success_text: UiColor,
    pub alert_text: UiColor,
    pub radius: f32,
    pub border_width: f32,
    pub padding: f32,
    pub max_width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub title: UiColor,
    pub section: UiColor,
    pub body: UiColor,
    pub muted: UiColor,
    pub badge: UiColor,
    pub notice: UiColor,
    pub control: UiColor,
    pub title_size: f32,
    pub section_size: f32,
    pub body_size: f32,
    pub muted_size: f32,
    pub badge_size: f32,
    pub control_size: f32,
}

impl TextStyle {
    pub fn color_for(self, role: TextRole) -> UiColor {
        match role {
            TextRole::Title => self.title,
            TextRole::Section => self.section,
            TextRole::Body => self.body,
            TextRole::Muted => self.muted,
            TextRole::Badge => self.badge,
            TextRole::Notice => self.notice,
            TextRole::Control => self.control,
        }
    }

    pub fn size_for(self, role: TextRole) -> f32 {
        match role {
            TextRole::Title => self.title_size,
            TextRole::Section => self.section_size,
            TextRole::Body | TextRole::Notice => self.body_size,
            TextRole::Muted => self.muted_size,
            TextRole::Badge => self.badge_size,
            TextRole::Control => self.control_size,
        }
    }
}

/// Reference design resolution. Every size in `UiSpacing`, `ButtonStyle`,
/// `SlotStyle`, `TextStyle`, etc. is expressed in **UI units** at this
/// resolution. To convert UI units to physical pixels, multiply by the
/// effective scale returned by [`UiTheme::effective_scale`].
pub const REFERENCE_WIDTH: f32 = 1920.0;
pub const REFERENCE_HEIGHT: f32 = 1080.0;

/// Player-visible viewport (window client area, post-DPI).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiViewport {
    pub width: f32,
    pub height: f32,
}

impl UiViewport {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Player-controlled zoom preset. The design system ships five steps so
/// players can match their monitor without exotic intermediate values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserZoom {
    Small,   // 90%
    Normal,  // 100%
    Large,   // 110%
    XLarge,  // 125%
    XXLarge, // 150%
}

impl UserZoom {
    pub const fn factor(self) -> f32 {
        match self {
            UserZoom::Small => 0.90,
            UserZoom::Normal => 1.00,
            UserZoom::Large => 1.10,
            UserZoom::XLarge => 1.25,
            UserZoom::XXLarge => 1.50,
        }
    }

    pub const ALL: [UserZoom; 5] = [
        UserZoom::Small,
        UserZoom::Normal,
        UserZoom::Large,
        UserZoom::XLarge,
        UserZoom::XXLarge,
    ];
}

/// Anchors are how screens declare *where* a UI block lives on the viewport.
/// A panel says "I want to be `BottomCenter`-anchored with margin 40 UI",
/// and the renderer derives the actual pixel rect from the viewport + scale.
/// This is the only acceptable way to position gameplay UI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// Hard limits applied to the combined auto + user zoom scale. Keeps small
/// windows readable and stops 4K from making the hotbar fill the screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResponsiveLimits {
    pub min_scale: f32,
    pub max_scale: f32,
    /// Lower bound for the auto component only, before user zoom is applied.
    pub min_auto_scale: f32,
    /// Upper bound for the auto component only, before user zoom is applied.
    pub max_auto_scale: f32,
}

/// Geometry rules for large panels (inventory, craft, chest). Expressed
/// in UI units and / or fractions of the viewport.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelConstraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
    /// Hard ceiling expressed as a fraction of the viewport, applied AFTER
    /// scaling. Keeps panels from ever spilling off the screen.
    pub max_width_ratio: f32,
    pub max_height_ratio: f32,
    /// Margin (in UI units) between the panel edge and the viewport edge
    /// when the panel touches one of them.
    pub viewport_margin: f32,
}

impl PanelConstraints {
    /// Resolve the actual panel size in physical pixels for the given
    /// viewport and effective scale. Guarantees the panel never overflows
    /// the screen — desired size is shrunk to fit `max_*_ratio` first,
    /// then to fit the viewport minus margins.
    pub fn resolve(
        self,
        viewport: UiViewport,
        scale: f32,
        desired_w: f32,
        desired_h: f32,
    ) -> (f32, f32) {
        let scaled = |v: f32| v * scale;
        let desired_w = desired_w.clamp(self.min_width, self.max_width);
        let desired_h = desired_h.clamp(self.min_height, self.max_height);
        let margin = scaled(self.viewport_margin);
        let max_w = (viewport.width * self.max_width_ratio).min(viewport.width - margin * 2.0);
        let max_h = (viewport.height * self.max_height_ratio).min(viewport.height - margin * 2.0);
        (scaled(desired_w).min(max_w), scaled(desired_h).min(max_h))
    }
}

/// Rules for grids that adapt their column count to the available width.
/// The renderer picks the largest column count that fits the available
/// width, clamped to `[min_columns, max_columns]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdaptiveGrid {
    pub min_columns: usize,
    pub preferred_columns: usize,
    pub max_columns: usize,
}

impl AdaptiveGrid {
    /// Compute the column count for a given available width, using the
    /// already-scaled slot + gap sizes. Guarantees at least `min_columns`
    /// — if even that doesn't fit, the caller must switch to a scrollable
    /// layout (and the returned value is still `min_columns`).
    pub fn columns_for(self, available_width: f32, slot_size: f32, gap: f32) -> usize {
        if slot_size <= 0.0 {
            return self.preferred_columns;
        }
        let step = slot_size + gap;
        let n = ((available_width + gap) / step).floor() as i64;
        n.clamp(self.min_columns as i64, self.max_columns as i64) as usize
    }

    /// Convenience: clamp a desired column count into the configured bounds.
    pub fn clamp(self, columns: usize) -> usize {
        columns.clamp(self.min_columns, self.max_columns)
    }
}

/// All responsive policy in one place: how the auto scale is computed,
/// what the user zoom defaults to, what limits apply, and the shared
/// constraints for large panels and adaptive grids.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiResponsive {
    pub reference_width: f32,
    pub reference_height: f32,
    pub limits: ResponsiveLimits,
    pub default_user_zoom: UserZoom,
    pub inventory_panel: PanelConstraints,
    pub craft_panel: PanelConstraints,
    pub chest_panel: PanelConstraints,
    pub inventory_grid: AdaptiveGrid,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTheme {
    pub name: &'static str,
    pub spacing: UiSpacing,
    pub motion: UiMotion,
    pub readability: UiReadability,
    pub responsive: UiResponsive,
    pub panel: PanelStyle,
    pub slot: SlotStyle,
    pub button: ButtonStyle,
    pub search_bar: SearchBarStyle,
    pub filter_chip: FilterChipStyle,
    pub inventory_grid: InventoryGridStyle,
    pub hotbar: HotbarStyle,
    pub quantity_badge: QuantityBadgeStyle,
    pub player_notice: PlayerNoticeStyle,
    pub tooltip: TooltipStyle,
    pub text: TextStyle,
}

impl UiTheme {
    /// Auto-scale derived from the viewport against the 1920x1080 reference.
    /// Uses the *smaller* of width-ratio and height-ratio so the UI fits
    /// both ultrawide (height-bound) and portrait (width-bound) windows.
    /// Clamped by [`ResponsiveLimits::min_auto_scale`] / `max_auto_scale`.
    pub fn auto_scale(&self, viewport: UiViewport) -> f32 {
        let w_ratio = viewport.width / self.responsive.reference_width;
        let h_ratio = viewport.height / self.responsive.reference_height;
        w_ratio.min(h_ratio).clamp(
            self.responsive.limits.min_auto_scale,
            self.responsive.limits.max_auto_scale,
        )
    }

    /// Effective scale = auto scale × user zoom, clamped by the global
    /// `min_scale` / `max_scale`. This is the single multiplier every
    /// component must apply to convert UI units into physical pixels.
    pub fn effective_scale(&self, viewport: UiViewport, user_zoom: UserZoom) -> f32 {
        let combined = self.auto_scale(viewport) * user_zoom.factor();
        combined.clamp(
            self.responsive.limits.min_scale,
            self.responsive.limits.max_scale,
        )
    }

    /// Convert a value in UI units into physical pixels for the given
    /// viewport + user zoom.
    pub fn scale_units(&self, units: f32, viewport: UiViewport, user_zoom: UserZoom) -> f32 {
        units * self.effective_scale(viewport, user_zoom)
    }

    /// Resolve the actual hotbar slot size in physical pixels for a given
    /// viewport + user zoom. The slot is always square: this same value is
    /// used for both width and height. The hotbar's min/max bounds (in UI
    /// units) are scaled by the effective scale so the clamp stays
    /// proportional across resolutions.
    pub fn hotbar_slot_size(&self, viewport: UiViewport, user_zoom: UserZoom) -> f32 {
        let scale = self.effective_scale(viewport, user_zoom);
        let target = (self.spacing.slot_size * scale).round();
        target.clamp(
            self.hotbar.slot_size_min * scale,
            self.hotbar.slot_size_max * scale,
        )
    }

    /// Pixel-space origin for a UI block of size `size` anchored to the
    /// viewport with the given margin (in physical pixels).
    pub fn anchor_origin(
        &self,
        viewport: UiViewport,
        anchor: UiAnchor,
        size: (f32, f32),
        margin: f32,
    ) -> (f32, f32) {
        let (w, h) = size;
        let vw = viewport.width;
        let vh = viewport.height;
        let cx = (vw - w) * 0.5;
        let cy = (vh - h) * 0.5;
        match anchor {
            UiAnchor::TopLeft => (margin, margin),
            UiAnchor::TopCenter => (cx, margin),
            UiAnchor::TopRight => (vw - w - margin, margin),
            UiAnchor::CenterLeft => (margin, cy),
            UiAnchor::Center => (cx, cy),
            UiAnchor::CenterRight => (vw - w - margin, cy),
            UiAnchor::BottomLeft => (margin, vh - h - margin),
            UiAnchor::BottomCenter => (cx, vh - h - margin),
            UiAnchor::BottomRight => (vw - w - margin, vh - h - margin),
        }
    }

    pub const VOXELVERSE: Self = Self {
        name: "voxelverse_adventure",
        spacing: UiSpacing {
            tiny: 4.0,
            small: 8.0,
            medium: 12.0,
            large: 18.0,
            xlarge: 24.0,
            panel: 24.0,
            slot_size: 56.0,
            slot_size_hotbar_min: 56.0,
            slot_size_hotbar_max: 78.0,
            slot_gap: 9.0,
            slot_row_gap: 12.0,
            icon_size: 32.0,
            icon_inset: 12.0,
            badge_diameter: 20.0,
            button_height: 44.0,
            control_height: 40.0,
            chip_height: 38.0,
            border_thin: 1.0,
            border_medium: 2.0,
            border_thick: 4.0,
            radius_slot: 4.0,
            radius_panel: 8.0,
            radius_control: 6.0,
            radius_pill: 999.0,
            hotbar_bottom_margin_min: 24.0,
            hotbar_bottom_margin_max: 120.0,
            tooltip_offset: 12.0,
            tooltip_padding: 12.0,
            tooltip_max_width: 320.0,
        },
        motion: UiMotion {
            slot_select_ms: 90.0,
            slot_pickup_ms: 180.0,
            slot_invalid_flash_ms: 140.0,
            slot_success_flash_ms: 160.0,
            panel_open_ms: 150.0,
            panel_close_ms: 110.0,
            filter_swap_ms: 90.0,
            tooltip_delay_ms: 250.0,
            tooltip_fade_ms: 90.0,
            notice_fade_in_ms: 90.0,
            notice_hold_ms: 900.0,
            notice_fade_out_ms: 220.0,
            ease_out: [0.22, 0.61, 0.36, 1.00],
            ease_in: [0.55, 0.05, 0.68, 0.19],
            ease_in_out: [0.65, 0.05, 0.36, 1.00],
        },
        readability: UiReadability {
            text_scrim: UiColor::rgba(0.0, 0.0, 0.0, 0.45),
            text_outline: UiColor::rgba(0.0, 0.0, 0.0, 0.80),
            text_outline_width: 1.0,
            min_slot_alpha: 0.78,
            modal_scrim: UiColor::rgba(0.0, 0.0, 0.0, 1.0),
        },
        responsive: UiResponsive {
            reference_width: REFERENCE_WIDTH,
            reference_height: REFERENCE_HEIGHT,
            limits: ResponsiveLimits {
                min_scale: 0.70,
                max_scale: 2.20,
                min_auto_scale: 0.75,
                max_auto_scale: 1.60,
            },
            default_user_zoom: UserZoom::Normal,
            // Inventory: ~720 UI wide at 1080p, capped at 60% of the
            // viewport width so it never spills past the screen edges.
            inventory_panel: PanelConstraints {
                min_width: 560.0,
                max_width: 880.0,
                min_height: 420.0,
                max_height: 720.0,
                max_width_ratio: 0.60,
                max_height_ratio: 0.80,
                viewport_margin: 32.0,
            },
            // Craft: narrower than inventory by design.
            craft_panel: PanelConstraints {
                min_width: 480.0,
                max_width: 720.0,
                min_height: 360.0,
                max_height: 640.0,
                max_width_ratio: 0.50,
                max_height_ratio: 0.78,
                viewport_margin: 32.0,
            },
            // Chest: matches inventory size so they sit side by side.
            chest_panel: PanelConstraints {
                min_width: 560.0,
                max_width: 880.0,
                min_height: 360.0,
                max_height: 640.0,
                max_width_ratio: 0.55,
                max_height_ratio: 0.70,
                viewport_margin: 32.0,
            },
            inventory_grid: AdaptiveGrid {
                min_columns: 5,
                preferred_columns: 8,
                max_columns: 12,
            },
        },
        panel: PanelStyle {
            fill: UiColor::rgba(0.095, 0.108, 0.115, 1.0),
            border: UiColor::rgba(0.72, 0.45, 0.17, 0.82),
            border_strong: UiColor::rgba(1.00, 0.66, 0.22, 0.95),
            shadow: UiColor::rgba(0.0, 0.0, 0.0, 0.58),
            radius: 8.0,
            border_width: 1.0,
            padding: 18.0,
        },
        slot: SlotStyle {
            fill: UiColor::rgba(0.105, 0.085, 0.055, 0.86),
            fill_hovered: UiColor::rgba(0.18, 0.13, 0.070, 0.91),
            fill_pressed: UiColor::rgba(0.22, 0.14, 0.060, 0.95),
            fill_selected: UiColor::rgba(0.24, 0.15, 0.045, 0.94),
            fill_empty: UiColor::rgba(0.060, 0.060, 0.050, 0.72),
            fill_disabled: UiColor::rgba(0.035, 0.040, 0.040, 0.60),
            fill_invalid: UiColor::rgba(0.32, 0.08, 0.06, 0.92),
            fill_success: UiColor::rgba(0.10, 0.22, 0.10, 0.92),
            fill_alert: UiColor::rgba(0.30, 0.15, 0.05, 0.92),
            inner_fill: UiColor::rgba(0.040, 0.057, 0.055, 0.84),
            border: UiColor::rgba(0.55, 0.34, 0.13, 0.86),
            border_hovered: UiColor::rgba(0.86, 0.55, 0.19, 0.95),
            border_pressed: UiColor::rgba(0.94, 0.62, 0.22, 1.0),
            border_selected: UiColor::rgba(1.00, 0.68, 0.26, 1.0),
            border_empty: UiColor::rgba(0.26, 0.20, 0.13, 0.62),
            border_disabled: UiColor::rgba(0.18, 0.16, 0.13, 0.55),
            border_invalid: UiColor::rgba(0.96, 0.32, 0.22, 1.0),
            border_success: UiColor::rgba(0.46, 0.94, 0.42, 1.0),
            border_alert: UiColor::rgba(0.96, 0.62, 0.18, 1.0),
            highlight: UiColor::rgba(1.00, 0.80, 0.43, 0.88),
            content_glint: UiColor::rgba(1.00, 0.92, 0.72, 0.70),
            radius: 4.0,
            border_width: 1.0,
            selected_border_width: 4.0,
            inner_inset: 3.0,
            icon_inset: 12.0,
        },
        button: ButtonStyle {
            fill: UiColor::rgba(0.46, 0.29, 0.08, 0.88),
            fill_hovered: UiColor::rgba(0.58, 0.37, 0.11, 0.94),
            fill_pressed: UiColor::rgba(0.40, 0.25, 0.07, 0.96),
            fill_selected: UiColor::rgba(0.72, 0.46, 0.13, 0.96),
            fill_disabled: UiColor::rgba(0.08, 0.08, 0.07, 0.55),
            fill_alert: UiColor::rgba(0.42, 0.12, 0.08, 0.92),
            fill_success: UiColor::rgba(0.12, 0.28, 0.12, 0.92),
            border: UiColor::rgba(0.94, 0.61, 0.20, 0.86),
            border_hovered: UiColor::rgba(1.00, 0.70, 0.26, 1.0),
            border_disabled: UiColor::rgba(0.18, 0.16, 0.13, 0.55),
            text: UiColor::rgb8(255, 238, 208),
            text_disabled: UiColor::rgb8(118, 111, 98),
            text_alert: UiColor::rgb8(255, 188, 168),
            text_success: UiColor::rgb8(196, 240, 188),
            height: 44.0,
            padding_x: 18.0,
            border_width: 1.0,
            radius: 6.0,
        },
        search_bar: SearchBarStyle {
            fill: UiColor::rgba(0.025, 0.036, 0.038, 0.82),
            fill_focused: UiColor::rgba(0.035, 0.050, 0.052, 0.92),
            fill_disabled: UiColor::rgba(0.020, 0.024, 0.024, 0.55),
            border: UiColor::rgba(0.50, 0.31, 0.12, 0.82),
            border_focused: UiColor::rgba(0.95, 0.62, 0.22, 0.95),
            border_disabled: UiColor::rgba(0.18, 0.16, 0.13, 0.55),
            placeholder: UiColor::rgb8(178, 166, 148),
            text: UiColor::rgb8(255, 238, 208),
            text_disabled: UiColor::rgb8(118, 111, 98),
            icon: UiColor::rgb8(255, 197, 112),
            height: 40.0,
            padding_x: 14.0,
            border_width: 1.0,
            radius: 6.0,
        },
        filter_chip: FilterChipStyle {
            fill: UiColor::rgba(0.040, 0.052, 0.052, 0.78),
            fill_hovered: UiColor::rgba(0.10, 0.080, 0.045, 0.88),
            fill_selected: UiColor::rgba(0.50, 0.32, 0.09, 0.92),
            fill_disabled: UiColor::rgba(0.030, 0.034, 0.034, 0.50),
            border: UiColor::rgba(0.28, 0.22, 0.15, 0.70),
            border_hovered: UiColor::rgba(0.72, 0.46, 0.18, 0.90),
            border_selected: UiColor::rgba(0.95, 0.62, 0.22, 0.95),
            border_disabled: UiColor::rgba(0.16, 0.14, 0.12, 0.55),
            text: UiColor::rgb8(232, 222, 205),
            text_selected: UiColor::rgb8(255, 229, 178),
            text_disabled: UiColor::rgb8(118, 111, 98),
            height: 38.0,
            padding_x: 16.0,
            border_width: 1.0,
            radius: 999.0,
        },
        inventory_grid: InventoryGridStyle {
            columns: 8,
            slot_size: 58.0,
            gap: 10.0,
            row_gap: 12.0,
            empty_slot_fill: UiColor::rgba(0.025, 0.035, 0.035, 0.64),
        },
        hotbar: HotbarStyle {
            slots: 9,
            slot_size_min: 48.0,
            slot_size_max: 160.0,
            slot_height_ratio: 0.062,
            panel_padding: 16.0,
            slot_gap: 8.0,
            notice_offset_y: 30.0,
        },
        quantity_badge: QuantityBadgeStyle {
            text: UiColor::rgb8(255, 235, 190),
            text_alert: UiColor::rgb8(255, 168, 130),
            text_full: UiColor::rgb8(196, 240, 188),
            shadow: UiColor::rgba(0.0, 0.0, 0.0, 0.85),
            font_size: 13.0,
            line_height: 16.0,
            right_inset: 6.0,
            bottom_inset: 4.0,
        },
        player_notice: PlayerNoticeStyle {
            info_text: UiColor::rgb8(255, 235, 190),
            success_text: UiColor::rgb8(196, 240, 188),
            alert_text: UiColor::rgb8(255, 168, 130),
            invalid_text: UiColor::rgb8(255, 120, 110),
            muted_text: UiColor::rgb8(196, 184, 162),
            shadow: UiColor::rgba(0.0, 0.0, 0.0, 0.72),
            font_size: 17.0,
            line_height: 22.0,
        },
        tooltip: TooltipStyle {
            fill: UiColor::rgba(0.022, 0.030, 0.034, 0.94),
            border: UiColor::rgba(0.92, 0.60, 0.20, 0.92),
            shadow: UiColor::rgba(0.0, 0.0, 0.0, 0.60),
            title_text: UiColor::rgb8(255, 229, 178),
            body_text: UiColor::rgb8(240, 231, 214),
            muted_text: UiColor::rgb8(176, 164, 144),
            success_text: UiColor::rgb8(196, 240, 188),
            alert_text: UiColor::rgb8(255, 168, 130),
            radius: 6.0,
            border_width: 1.0,
            padding: 12.0,
            max_width: 320.0,
        },
        text: TextStyle {
            title: UiColor::rgb8(255, 237, 210),
            section: UiColor::rgb8(255, 176, 47),
            body: UiColor::rgb8(240, 231, 214),
            muted: UiColor::rgb8(176, 164, 144),
            badge: UiColor::rgb8(255, 235, 190),
            notice: UiColor::rgb8(255, 205, 140),
            control: UiColor::rgb8(255, 238, 208),
            title_size: 30.0,
            section_size: 18.0,
            body_size: 16.0,
            muted_size: 14.0,
            badge_size: 16.0,
            control_size: 16.0,
        },
    };
}
