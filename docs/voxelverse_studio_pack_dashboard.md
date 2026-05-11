# VoxelVerse Studio - Pack Dashboard (future)

This document describes the **future** dashboard that will sit on top of the
content pipeline. It is documentation, not an implementation plan. The
dashboard is not built yet, and **must not** be built before the rest of the
pipeline is stable.

The dashboard is a cockpit. It does not own data. The pack remains the source
of truth.

---

## Goals

- Surface the current state of the core pack at a glance.
- Make Pack Doctor reports actionable: every error and warning becomes a
  navigation target.
- Show coverage: which blocks have items, which items have recipes, which
  textures are referenced.
- Show progression reachability for the basic loop.
- Provide editing entry points (open file, open brief, open material).

---

## What the dashboard reads

The dashboard's only inputs:

```
assets/packs/core/                         (raw pack)
assets/packs/core/generated/reports/       (Pack Doctor JSON)
```

It reads `.ron` files. It reads `core_pack_report.json`. It reads briefs from
`source/production/`. It does not have a database.

---

## What the dashboard writes

- It writes `.ron` files in `defs/`.
- It writes texture references and material updates.
- It writes briefs in `source/production/`.
- It does not write to `generated/` directly; it triggers Pack Doctor and
  consumes the output.

When the dashboard writes, the result is **identical** to what a human or AI
agent would write by hand. No proprietary format. No hidden state. The
dashboard could be uninstalled tomorrow and the pack would be unchanged.

---

## What the dashboard never does

- It never holds a separate "studio database" of truth.
- It never overrides the pack's contents with cached state.
- It never compiles content directly to the runtime; it goes through
  `vv-pack-compiler`.
- It never replaces Pack Doctor or `validate_content.ps1`; it triggers them
  and presents the results.

If any feature would require the dashboard to be authoritative over the pack,
that feature is wrong. Re-design the feature instead.

---

## Likely first views

1. **Health view** - errors, warnings, score, last run time.
2. **Block grid** - 12-column tile grid of all blocks with thumbnails,
   warnings, and quick links.
3. **Item grid** - same shape as the block grid.
4. **Coverage view** - placeable blocks without items, items without recipes,
   ores without worldgen.
5. **Progression view** - the basic loop reachability tree.
6. **Brief view** - editable briefs and prompts from `source/production/`.

---

## When to build it

Only when **all** of the following are true:

- Pack Doctor V1 is stable and trusted.
- The JSON report schema has stopped changing.
- The MVP content set is complete and the pack is green.
- The team has a real, repeated, painful workflow that the dashboard would
  fix (not a vague desire for a "studio").

Until then, the dashboard is a document. It exists so that future agents do
not improvise it.
