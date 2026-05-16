# VoxelVerse V1 UI And UX

VoxelVerse UI must be simple enough for a child, fast enough for a skilled player, and informative enough for a builder/modder.

## UI principles

1. The world stays the hero.
2. Hotbar is muscle memory.
3. Inventory is an extension of the hotbar.
4. Crafting is contextual, not a wiki hunt.
5. Tooltips explain without clutter.
6. Every action needs feedback.
7. UI must be readable over snow, sky, caves and forests.

## V1 screens

Required:

- HUD;
- hotbar;
- inventory;
- hand crafting panel;
- station crafting panel;
- chest/storage panel if storage exists;
- pause/settings;
- diagnostics overlay;
- item tooltip;
- player notices.

Optional:

- codex;
- map;
- character screen;
- recipe book.

## HUD

HUD should show only what matters.

Required:

- crosshair/target indicator;
- hotbar;
- selected item name on change;
- mining/building notices;
- interaction prompt if relevant;
- health/stamina only if implemented.

Forbidden:

- permanent clutter;
- huge panels during exploration;
- debug info outside debug mode.

## Inventory model

Inventory should feel like expanded hotbar.

Target layout:

```text
Top tabs: Inventory | Craft | Codex | Map | Settings
Left: player/avatar/equipment later
Center: inventory grid
Bottom: hotbar mirrored or integrated
Right: contextual craftable recipes or item details
```

V1 can start smaller, but must keep this direction.

## Slot behavior

Every slot must support states:

- empty;
- normal;
- hovered;
- selected;
- pressed;
- disabled;
- invalid;
- success;
- alert.

Every state must be visually distinct but not noisy.

## Mouse actions

Minimum:

- left click pick/place stack;
- right click split or place one if implemented;
- shift left click quick move;
- mouse wheel hotbar selection;
- number keys select hotbar;
- escape closes UI.

If right-click split is not implemented, right click must do nothing predictable and documented, not half-work.

## Tooltip V1

Tooltip should show:

- display name;
- category;
- stack count/max;
- short description;
- tags or tool type if relevant;
- mining tier/speed for tools;
- block hardness or required tool if useful;
- recipe hint if craftable or ingredient.

Tooltip must not cover the cursor target in a frustrating way.

## Crafting UX

Crafting should answer:

- what can I craft now?
- what am I missing?
- where do I get missing ingredient?
- what station is required?
- what will this item do?

V1 crafting UI must have:

- available recipes first;
- unavailable recipes visible only if useful;
- clear output preview;
- craft button state;
- quantity feedback;
- station name/title;
- no mystery gray icons without explanation.

## Station UX

All stations share a layout grammar:

```text
Left: input/storage slots
Center: process or recipe area
Right: output/result/details
Bottom: player inventory/hotbar
```

Station-specific UI can vary, but the player's inventory must always behave the same.

## Readability rules

UI must remain readable over:

- snow;
- bright sky;
- dark cave;
- forest foliage;
- water reflection;
- sunset.

Use:

- scrims;
- outlines;
- controlled opacity;
- consistent spacing;
- minimum text sizes;
- strong selected states.

## UI performance

UI must not rebuild unchanged geometry every frame.

Cache where possible:

- hotbar static geometry;
- inventory panel geometry;
- text layouts;
- item icon references;
- slot background meshes.

Rebuild only on:

- viewport resize;
- inventory revision change;
- selection change;
- hover change;
- theme change;
- language/text change.

## Diagnostics UI

Debug overlay must be quick to toggle and readable.

Suggested tabs:

- frame;
- renderer;
- streaming;
- worldgen;
- player;
- content;
- audio.

## UX gate

UI is V1-ready when:

- a player can use inventory without explanation;
- crafting next step is obvious;
- hotbar is small and readable;
- selected item is unmistakable;
- tooltips are useful but not huge;
- storage/station screens share consistent behavior;
- all common actions take few clicks;
- UI is readable over all biomes;
- UI does not hurt frame time.
