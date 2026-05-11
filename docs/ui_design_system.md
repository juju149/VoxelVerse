# VoxelVerse UI Design System

The runtime source of truth is `apps/voxelverse-game/src/ui/theme.rs`.

This theme exists so gameplay interfaces share one visual language instead of
hardcoding colors and spacing per screen.

## Identity

VoxelVerse UI should feel warm, readable, premium, adventurous, and practical.
It should support fast play first: hotbar, inventory, craft, chests, equipment,
menus, and tooltips must all use the same tokens.

## Component Rules

- Panels use a dark translucent fill with warm gold borders.
- Slots are regular, predictable squares with a darker inner well.
- Selected slots use the strong gold border from the slot style.
- Quantity badges use the shared quantity badge text color and sizing.
- Buttons, filter chips, search bars, and inventory grids must read their
  colors and spacing from `UiTheme`.
- Empty or disabled controls should be dimmed, not replaced by neutral gray.
- Future UI themes should compile into the same shape as `UiTheme`, so mods can
  swap presentation without changing gameplay code.

## Current Components

- `PanelStyle`: shared container style for inventory, craft, chests, menus.
- `SlotStyle`: hotbar, inventory grid, equipment, quick tools.
- `HotbarStyle`: 9-slot bottom bar layout.
- `InventoryGridStyle`: future inventory grid dimensions.
- `ButtonStyle`: future action buttons such as craft, sort, close.
- `FilterChipStyle`: future inventory filters.
- `SearchBarStyle`: future inventory/craft search fields.
- `TextStyle`: title, section, body, muted, badge, notice, control text.
- `QuantityBadgeStyle`: item counts and temporary player-facing notices.

Do not introduce screen-local palettes for gameplay UI. Add a token or
component style to the theme first, then consume it from the renderer or UI
component.
