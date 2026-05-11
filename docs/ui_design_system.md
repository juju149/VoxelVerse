# VoxelVerse UI Design System

The runtime source of truth is [apps/voxelverse-game/src/ui/theme.rs](../apps/voxelverse-game/src/ui/theme.rs).
This document explains the rules behind that file. Code is canonical, this doc
is the manual. If they disagree, fix the doc.

This design system exists so every gameplay interface — hotbar, inventory,
craft, chests, equipment, menus, tooltips — shares one visual language. No
screen is allowed to invent its own palette, slot size, or button style.

---

## 1. Identity

VoxelVerse UI should feel **warm, readable, premium, adventurous, practical**.

- Warm: dark translucent panels with gold accents, never neutral gray chrome.
- Readable: tokens are sized for 1080p and scaled for other viewports, never
  for "looks good in a screenshot".
- Premium: depth comes from layered fills + glints + soft shadows, not from
  decorative noise.
- Adventurous: the gold accent is reserved for selection, focus and call-to-
  action — it is the player's eye-catch.
- Practical: fast play comes first. Animations are short, controls are large
  enough to click without aiming, and the hotbar stays out of the gameplay
  reticle.

Every screen reads its colors, sizes, durations and easings from `UiTheme`.

---

## 2. Visual states

Every interactive UI element must define a visual rendering for every state
in the `ComponentState` enum. The renderer must never fall back to a default
unstyled state.

| State        | When it applies                                    | Visual rule                                                  |
|--------------|----------------------------------------------------|--------------------------------------------------------------|
| `Normal`     | Idle, ready for input.                             | Base fill + thin warm border.                                |
| `Hovered`    | Pointer over the element.                          | Slightly brighter fill, brighter border, no size change.     |
| `Pressed`    | Mouse / key held down.                             | Darker fill, stronger border, optionally inset 1px.          |
| `Selected`   | Element is the active choice (hotbar slot, chip).  | Strong gold border (`selected_border_width`), warmest fill.  |
| `Disabled`   | Input blocked for systemic reasons.                | Dim fill, low-contrast border, never gray-on-gray.           |
| `Empty`      | Container has no content (empty slot, no results). | Subtle fill, muted border, no glint, no badge.               |
| `Invalid`    | Player action was rejected (can't place, full).    | Red border, red-tinted fill, transient flash.                |
| `Success`    | Player action confirmed (picked up, crafted).      | Green border, green-tinted fill, transient flash.            |
| `Alert`      | Persistent warning (low durability, near-full).    | Amber border, amber-tinted fill, **not** a flash.            |

Rules:

- `Disabled` and `Empty` must be visually distinct. An empty slot still
  invites a drop; a disabled slot does not.
- `Invalid` and `Alert` must be visually distinct. Invalid is a short red
  flash that fades; alert is a steady amber warning.
- `Success` and `Selected` must be visually distinct. Selected is gold and
  steady; success is green and transient.
- The slot test `slot_distinguishes_every_gameplay_state` enforces pairwise
  uniqueness — if you add a new state, extend the test.

---

## 3. Sizes and spacing

All values live in `UiSpacing` and are expressed in **UI units** at the
reference resolution `REFERENCE_WIDTH × REFERENCE_HEIGHT = 1920 × 1080`.

A UI unit is "one pixel at 1080p with zoom 100%". To get physical pixels,
multiply by `UiTheme::effective_scale(viewport, user_zoom)`. Never write a
raw pixel size in a screen file — use UI units and let the design system do
the conversion.

See section 9 for the responsive rules that drive the scale.

| Token                       | Value   | Used for                                       |
|-----------------------------|---------|------------------------------------------------|
| `tiny` / `small` / `medium` | 4 / 8 / 12   | Inline gaps inside controls.              |
| `large` / `xlarge` / `panel`| 18 / 24 / 24 | Section spacing, panel padding.           |
| `slot_size`                 | 56      | Canonical inventory / equipment slot side.     |
| `slot_size_hotbar_min/max`  | 46 / 58 | Clamp range for the responsive hotbar.         |
| `slot_gap` / `slot_row_gap` | 7 / 10  | Spacing inside slot grids.                     |
| `icon_size` / `icon_inset`  | 32 / 12 | Item icon square inside a slot.                |
| `badge_diameter`            | 20      | Quantity badge box.                            |
| `button_height`             | 44      | Primary action buttons (craft, sort, close).   |
| `control_height`            | 40      | Search bar height.                             |
| `chip_height`               | 38      | Filter chip height.                            |
| `border_thin/medium/thick`  | 1 / 2 / 4 | Stroke widths. Selected slots use `thick`.   |
| `radius_slot`               | 4       | Slot corner radius.                            |
| `radius_panel`              | 8       | Panel corner radius.                           |
| `radius_control`            | 6       | Buttons, search bar.                           |
| `radius_pill`               | 999     | Filter chips.                                  |
| `hotbar_bottom_margin_*`    | 24 / 42 | Clamp for hotbar Y offset across viewports.    |
| `tooltip_offset`            | 12      | Distance from the anchor to the tooltip edge.  |
| `tooltip_padding`           | 12      | Inner padding inside the tooltip.              |
| `tooltip_max_width`         | 320     | Tooltips wrap, never spill across the screen.  |

### Text sizes

| Role      | Size | Use                                          |
|-----------|------|----------------------------------------------|
| Title     | 30   | Panel titles ("Inventory", "Craft").         |
| Section   | 18   | Sub-headings inside a panel.                 |
| Body      | 16   | Item descriptions, tooltip body.             |
| Muted     | 14   | Hints, hotkeys, secondary metadata.          |
| Badge     | 16   | Item quantity badges.                        |
| Control   | 16   | Button labels, chip labels, search input.    |

Resolve via `TextStyle::size_for(role)` and `TextStyle::color_for(role)`.
Never write a raw font size in a screen file.

### Responsive rule (summary)

```text
auto_scale     = clamp(min(vw / 1920, vh / 1080), 0.75, 1.60)
effective      = clamp(auto_scale * user_zoom.factor(), 0.70, 2.20)
pixel_size(u)  = u * effective
hotbar_slot    = clamp(spacing.slot_size * effective,
                       hotbar.slot_size_min * effective,
                       hotbar.slot_size_max * effective)
```

This is the only acceptable way to size hotbar slots and any other piece
of gameplay UI. Full rules in section 9.

---

## 4. Components

These are the components the design system must cover for the hotbar +
inventory + craft + chest flow to ship.

- **Panel** — `PanelStyle`. Dark translucent fill, warm gold border, soft
  shadow. Container for every modal screen.
- **Slot (normal / hovered / pressed / selected / empty / disabled /
  invalid / success / alert)** — `SlotStyle`. The hotbar, inventory, equipment
  and chest grids all use exactly this slot. No screen-local variants.
- **Hotbar** — `HotbarStyle`. 9 slots, bottom-anchored, clamped slot size,
  notice text floats above.
- **Quantity badge** — `QuantityBadgeStyle`. Bottom-right of a slot. Has a
  normal color, an `alert` color (near stack cap), and a `full` color
  (stack maxed). Shadow always on for legibility.
- **Button (normal / hovered / pressed / selected / disabled / alert /
  success)** — `ButtonStyle`. One height (44), one radius (6).
- **Search bar** — `SearchBarStyle`. Single height (40), focused state has a
  brighter border and slightly brighter fill.
- **Filter chip** — `FilterChipStyle`. Pill shape, selected state inverts to
  warm gold fill + bright text.
- **Tooltip** — `TooltipStyle`. Title + body + muted secondary line +
  optional success/alert line. Wraps at `tooltip_max_width`.
- **Text roles** — `TextStyle`. `Title`, `Section`, `Body`, `Muted`, `Badge`,
  `Notice`, `Control`. Each role has a single color and a single size.
- **Player notice** — `PlayerNoticeStyle`. Transient floating text shown to
  the player (e.g. "Inventory full", "Picked up"). Four semantic colors:
  info, success, alert, invalid.

Anything missing from this list (slot trays, durability bars, drag ghost,
etc.) must be added to the theme **before** any screen renders it.

---

## 5. Animations

All durations live in `UiMotion`. The rule is *snappy, never sluggish*. If a
duration interferes with combat or placement, it is too long.

| Token                       | Duration | Easing      | Behaviour                              |
|-----------------------------|----------|-------------|----------------------------------------|
| `slot_select_ms`            | 90 ms    | ease-out    | Gold border thickens, fill warms.      |
| `slot_pickup_ms`            | 180 ms   | ease-out    | Icon pops 1.10x then settles.          |
| `slot_invalid_flash_ms`     | 140 ms   | ease-in-out | Red border + red fill, fades out.      |
| `slot_success_flash_ms`     | 160 ms   | ease-out    | Green border + green fill, fades out.  |
| `panel_open_ms`             | 150 ms   | ease-out    | Scale 0.96→1.0, alpha 0→1.             |
| `panel_close_ms`            | 110 ms   | ease-in     | Alpha 1→0, scale 1.0→0.98 (faster).    |
| `filter_swap_ms`            | 90 ms    | ease-out    | Chip color crossfade, no movement.     |
| `tooltip_delay_ms`          | 250 ms   | —           | Hover dwell before reveal.             |
| `tooltip_fade_ms`           | 90 ms    | ease-out    | Fade in only; hides instantly.         |
| `notice_fade_in_ms`         | 90 ms    | ease-out    | Notice text appears.                   |
| `notice_hold_ms`            | 900 ms   | —           | Hold time before fading.               |
| `notice_fade_out_ms`        | 220 ms   | ease-in     | Notice text disappears.                |

Easing tokens are cubic-bezier control points (`ease_out`, `ease_in`,
`ease_in_out`) so renderer and shader paths agree on the curve.

Rules:

- Closing animations are **faster** than opening animations. Players who
  want to dismiss UI should never feel held back.
- Hover-only feedback never moves geometry; it only changes color.
- Invalid / success flashes never block input — the underlying action has
  already resolved by the time the flash starts.
- The test `motion_durations_stay_short_enough_to_not_delay_gameplay`
  enforces upper bounds on the interactive durations. Lower them if needed,
  never raise them above 200 ms without updating the test.

---

## 6. Readability

The interface must remain legible regardless of what's behind it: snow,
forest, cave, sky, water, lava. Rules in `UiReadability`:

- Every floating text element (notices, badges, hotkey hints) renders on a
  short `text_scrim` darkening or with a 1 px `text_outline`, never bare.
- `min_slot_alpha` is the floor for non-empty slots — a slot holding an
  item is never allowed to drop below that opacity even when the world
  behind it is bright.
- Modal panels (inventory, craft, chest) draw `modal_scrim` over the world
  so panel contrast is guaranteed.
- Selection uses gold, not just brightness — color-blind players must still
  see the selected slot as distinct.
- Empty hotbar slots remain visible: `slot.fill_empty` is dim but never
  fully transparent.
- Full hotbar: when 9 slots are filled, the gold selection still wins
  visually thanks to `selected_border_width = 4 * border_thin`.
- Inventory grid empty cells use `inventory_grid.empty_slot_fill` which is
  intentionally darker than world fog so they read as "slots" not "holes".

Test plan in code:
- Selected vs normal: `selected_slot_uses_stronger_border_than_normal_slot`.
- Pairwise state distinctness: `slot_distinguishes_every_gameplay_state`.

---

## 7. Anti-drift rules

These rules are mandatory. A PR that breaks them should be reverted, not
patched.

1. **No screen-local palette.** Color literals are forbidden in gameplay UI
   code. Read from `UiTheme::VOXELVERSE` or a future theme constant.
2. **No screen-local slot size.** Slots use `spacing.slot_size`,
   `hotbar.slot_size_*`, or `inventory_grid.slot_size`. Nothing else.
3. **No screen-local button style.** Every button reads `ButtonStyle`.
   "But this one is smaller" → add a `button_compact` field to the theme.
4. **No raw font sizes.** Use `TextStyle::size_for(role)`. Adding a new
   text role is cheap; sprinkling magic numbers is not.
5. **No raw durations.** Animations read from `UiMotion`. If a new
   animation appears, add the token first, then use it.
6. **One source of truth per concept.** A slot has one corner radius
   (`slot.radius`). A button has one height (`button.height`). If two
   screens want different ones, that means the system needs a second style,
   not a one-off override.
7. **Tests guard the contract.** `theme.rs` ships tests that enforce
   slot-state distinctness, viewport clamping, motion ceilings, and text
   resolution. They run on every CI build.

If a new visual need genuinely doesn't fit any existing token: **add a token
to `theme.rs`, write a test, document it here, then use it.** In that
order.

---

## 9. Responsive system

All responsive policy lives in `UiResponsive` on `UiTheme`. Screens do not
implement their own scaling — they call the helpers below.

### Reference resolution

Every size in the design system is expressed in **UI units** at
`1920 × 1080`. A button declared `height: 44.0` is 44 px tall on a 1080p
display at 100% zoom, 66 px tall at 150% zoom, 70 px tall on 4K at 100%
zoom, and so on.

The two constants are exported as `REFERENCE_WIDTH` and `REFERENCE_HEIGHT`.

### Auto scale

```rust
auto_scale = min(viewport.width / 1920,  viewport.height / 1080)
auto_scale = clamp(auto_scale, limits.min_auto_scale, limits.max_auto_scale)
```

Defaults:

| Limit              | Value | Reason                                            |
|--------------------|-------|---------------------------------------------------|
| `min_auto_scale`   | 0.75  | Below this the UI starts cropping legibility.     |
| `max_auto_scale`   | 1.60  | Above this the hotbar starts eating the viewport. |

Taking the *minimum* of width-ratio and height-ratio means:

- Ultrawide (3440 × 1440) → height-bound, scale ≈ 1.33.
- Portrait (1080 × 1920) → width-bound, scale ≈ 0.56 → clamped to 0.75.
- 720p (1280 × 720) → 0.66 → clamped to 0.75.
- 1440p (2560 × 1440) → 1.33.
- 4K (3840 × 2160) → 2.0 → clamped to 1.60.

### User zoom

Players choose one of five presets via `UserZoom`:

| Preset    | Factor |
|-----------|--------|
| `Small`   | 0.90   |
| `Normal`  | 1.00   |
| `Large`   | 1.10   |
| `XLarge`  | 1.25   |
| `XXLarge` | 1.50   |

No intermediate values are supported. Five steps cover every reasonable
monitor — more would just be noise in the settings menu.

### Effective scale

```rust
effective = clamp(auto_scale * user_zoom.factor(),
                  limits.min_scale,  // 0.70
                  limits.max_scale)  // 2.20
```

This is the **only** multiplier any UI code is allowed to apply to a UI
unit. The helpers `UiTheme::effective_scale(viewport, zoom)` and
`UiTheme::scale_units(units, viewport, zoom)` do this for you.

### Anchors

Every screen positions itself via `UiAnchor` — never raw pixel offsets:

```text
TopLeft     TopCenter     TopRight
CenterLeft  Center        CenterRight
BottomLeft  BottomCenter  BottomRight
```

`UiTheme::anchor_origin(viewport, anchor, size, margin)` returns the
pixel-space origin. The hotbar uses `BottomCenter`. The inventory, craft and
chest panels use `Center`. Tooltips are anchored to the slot they describe
with an offset of `spacing.tooltip_offset`.

### Panel constraints

Large panels declare their bounds in `PanelConstraints`:

| Panel       | Min W | Max W | Max W ratio | Max H ratio |
|-------------|-------|-------|-------------|-------------|
| Inventory   | 560   | 880   | 60% of vw   | 80% of vh   |
| Craft       | 480   | 720   | 50% of vw   | 78% of vh   |
| Chest       | 560   | 880   | 55% of vw   | 70% of vh   |

`PanelConstraints::resolve(viewport, scale, desired_w, desired_h)` returns
the final pixel size, guaranteed to fit inside the viewport minus a
`viewport_margin` of 32 UI units on each side. The desired size is shrunk
to fit `max_*_ratio` first, then to fit the actual window.

When the viewport is too small for `min_*`, the panel renders at its
floor size and the content inside becomes scrollable. The UI must never
crush content to fit — it scrolls instead.

### Adaptive grid

`AdaptiveGrid` decides how many columns a grid uses based on available
width:

```rust
columns = clamp(floor((available + gap) / (slot + gap)),
                min_columns, max_columns)
```

Inventory defaults: `min=5`, `preferred=8`, `max=12`.

- Wide viewport → grow toward `max_columns`, panel stays the same height,
  content fits.
- Narrow viewport → fall back to `min_columns` and let the panel scroll
  vertically. Never shrink slots below the design size to "fit one more
  column".

### Hotbar

- Always `BottomCenter`-anchored, margin clamped in
  `[spacing.hotbar_bottom_margin_min, spacing.hotbar_bottom_margin_max]`.
- Slot size = `hotbar_slot_size(viewport, user_zoom)` — square, scaled,
  clamped by the hotbar's own min/max.
- Quantity badges sit at the bottom-right of each slot at
  `(spacing.badge_diameter)` and use the readability scrim so they stay
  legible against any biome.

### What pixels are NOT used for

Percentages of the viewport are allowed in exactly one place: panel
`max_*_ratio` limits, to cap how much of the screen a panel can cover.
Percentages must never define the size of slots, icons, badges, text, or
buttons — those are UI units passed through `effective_scale`.

### Tests guarding the responsive contract

- `viewport_scaling_clamps_in_sensible_range` — auto scale is 1.0 at 1080p
  and respects the auto clamps at 720p / 4K.
- `effective_scale_combines_auto_and_user_zoom` — user zoom multiplies the
  auto scale and the combined value never exceeds `max_scale`.
- `user_zoom_presets_cover_required_steps` — the five required presets
  (0.90, 1.00, 1.10, 1.25, 1.50) are exactly what `UserZoom::ALL` returns.
- `hotbar_slot_size_obeys_clamps` — slot is square and proportional at
  720p Small zoom and at 4K XXLarge zoom.
- `panel_constraints_never_overflow_viewport` — even a "desired" panel of
  2000 × 2000 UI units shrinks to fit a 1024 × 600 window.
- `adaptive_grid_adapts_columns_to_available_width` — grows toward
  `max_columns` on wide viewports, falls back to `min_columns` on narrow.
- `anchor_origin_keeps_blocks_inside_viewport` — anchor math centers and
  edge-aligns correctly.

---

## 10. Definition of done (for this milestone)

This design system is considered complete for the hotbar milestone if and
only if all of the following are true:

- [x] The hotbar can be rendered using only `UiTheme` styles — no literal
      colors, sizes, or durations in the hotbar code.
- [x] A future inventory grid can be built from `PanelStyle`,
      `InventoryGridStyle`, `SlotStyle`, `QuantityBadgeStyle`, `ButtonStyle`,
      `SearchBarStyle`, `FilterChipStyle`, `TooltipStyle` with no new
      bespoke style.
- [x] Slots are identical across hotbar, inventory, equipment and chest
      contexts (same shape, same states, same selection treatment).
- [x] Buttons are identical across craft, sort, close, and future actions
      (same height, same radius, same hover/pressed treatment).
- [x] Text hierarchy is resolved: Title > Section > Body > Muted, with
      Badge / Notice / Control as specialised roles.
- [x] `Selected`, `Empty`, `Disabled` and `Invalid` are pairwise distinct
      and the test `slot_distinguishes_every_gameplay_state` enforces it.
- [x] Animation tokens exist for selection, pickup, invalid, panel
      open/close, filter swap, tooltip and notices.
- [x] Readability tokens exist (text scrim, text outline, modal scrim,
      min slot alpha).
- [x] Responsive scaling is provided via `UiTheme::auto_scale`,
      `UiTheme::effective_scale`, `UiTheme::scale_units`,
      `UiTheme::hotbar_slot_size`, and `UiTheme::anchor_origin`.
- [x] The hotbar reads correctly at 1080p, 1440p, and 4K, and at every
      user-zoom preset from 90% to 150%.
- [x] The inventory panel is `Center`-anchored, capped at 60% of viewport
      width, and falls back to a scrollable layout below `min_width`.
- [x] Adaptive grids grow / shrink their column count via `AdaptiveGrid`
      without ever changing the slot size.
- [x] No screen implements its own scaling logic — every screen calls
      `effective_scale` or one of the helpers built on top of it.
- [x] Percentages are only used to cap large panels; slots, icons, badges,
      text and buttons stay in UI units.
- [x] No important gameplay UI style is hardcoded screen by screen.

When all boxes are checked, the hotbar can be wired up against this design
system. Until then, the foundation is incomplete and gameplay UI work
should wait.
