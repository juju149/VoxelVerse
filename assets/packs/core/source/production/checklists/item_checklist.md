# Item Checklist

Use this list when authoring or reviewing an item. Every box must be checkable
before the item is considered shipped.

Reference rules: [`docs/content_rules.md`](../../../../../../docs/content_rules.md)
sections 2 (naming), 3 (identity), 5 (items).

---

## 1. Brief

- [ ] A brief exists at `source/production/briefs/<stem>.md` (mandatory for
      new items, optional for existing).

## 2. Bucket

- [ ] The item lives in the correct bucket:
  - `defs/items/blocks/` for placeable block items
  - `defs/items/resources/` for raw materials and ingredients
  - `defs/items/tools/` for tools
  - `defs/items/weapons/` for weapons
  - `defs/items/consumables/` for consumables
  - `defs/items/food/` for food
  - `defs/items/misc/` for everything else (use sparingly)

## 3. Definition

- [ ] `defs/items/<bucket>/<stem>.item.ron` exists.
- [ ] `display_name` is human-readable, capitalized.
- [ ] `category` is set.
- [ ] `stack_size` is in `[1, 99]` unless documented.
- [ ] `visual.inventory_icon` references an icon (or sprite) that exists or is
      planned.
- [ ] `visual.world_model` is set when relevant (`BlockItem(...)` for block
      items, voxel model for held items).
- [ ] `gameplay` is appropriate:
  - `PlaceBlock("core:block/...")` for block items, target block exists and is
    placeable.
  - Tool / weapon / consumable variants are filled with tier, damage,
    durability, mining speed as applicable.

## 4. Tags

- [ ] Item carries the right baseline tag (`core:tag/item/block`,
      `core:tag/item/tool/...`, `core:tag/item/resource/...`).

## 5. Source / reachability

- [ ] At least one of the following exists:
  - A recipe whose output is this item.
  - A loot table that drops this item.
  - A worldgen entry that produces this item.
  - An explicit entry in `source/production/allowed_unused.ron`.

## 6. Usage / downstream

- [ ] The item is used somewhere: as a recipe input, as a placeable, as an
      equipable, or as a documented terminal item.

## 7. Validation

- [ ] `tools/validate_content.ps1` passes.
- [ ] `tools/pack_doctor.ps1` passes; the item is not listed under
      `unused.items` or `missing.block_items`.
- [ ] `cargo test -p vv-pack-loader -p vv-pack-compiler` passes.

If any box stays unchecked, the item is not done.
