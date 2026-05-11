# AI Content Automation

This document describes how AI agents fit into the VoxelVerse content
pipeline. The constitution applies: AI agents **generate proposals**, the pack
remains the source of truth, no AI agent invents a parallel architecture or a
parallel registry.

The pipeline rules in [`content_pipeline.md`](CONTENT_PIPELINE.md) and
[`content_rules.md`](content_rules.md) apply to AI-generated content with **no
exceptions**.

---

## Roles

The five roles below are loose specializations. A single human agent or AI
session can hold several roles, but each role has a clear deliverable.

### 1. Content Architect

Defines **what** to produce.

Produces:

- A list of files to create or modify.
- Dependencies between files (block needs material; material needs textures).
- The set of tags involved.
- Recipes that need to be authored alongside the block / item.
- Worldgen entries that need to be touched.

Output format: a Markdown brief in `source/production/briefs/<stem>.md` plus,
optionally, a TODO list in this same conversation.

### 2. Texture Agent

Produces visual assets.

Produces:

- PNG textures under `media/textures/<category>/<material_stem>/`.
- A face mapping description in the brief.
- Names matching `<material_stem>_<face>_<map>.png`.

Constraints:

- 256x256 for block surfaces unless documented.
- Albedo, normal, roughness must depict the same surface.
- No photorealistic detail - stay within VoxelVerse art direction.

### 3. RON Writer

Translates briefs and assets into definitions.

Produces:

- `.material.ron` under `defs/materials/`.
- `.block.ron` under `defs/blocks/`.
- `.item.ron` under `defs/items/`.
- `.loot.ron` under `defs/loot/`.
- `.recipe.ron` under `defs/recipes/` when the recipe schema is stable.

The RON Writer **never**:

- Hardcodes content in Rust.
- Creates files outside the canonical directory layout.
- Repeats path-derived IDs inside the file body.

### 4. Pack Doctor

Validates the pack (this role is automated; see [`pack_doctor`](#5-pack-doctor-tooling)).

Produces:

- Errors (blocking).
- Warnings (must be triaged).
- Unused content list.
- Missing-content list.
- Suggestions when a fix is obvious.

### 5. Gameplay Reviewer

Asks the gameplay questions automated checks can't answer:

- Does this content make the game more playable?
- Does progression remain clear?
- Does it add depth, or just noise?
- Does it pull weight relative to the cost of maintaining it?

The Gameplay Reviewer can veto content that passes Pack Doctor but harms the
first-hour loop or the MVP defined in
[`core_pack_mvp.md`](../assets/packs/core/source/production/core_pack_mvp.md).

---

## Workflow

A typical AI-driven content task should look like:

```
1. Content Architect    -> brief in source/production/briefs/
2. Texture Agent        -> PNGs in media/textures/
3. RON Writer           -> .ron files in defs/
4. Pack Doctor (auto)   -> JSON + HTML report in generated/reports/
5. Gameplay Reviewer    -> green light or feedback in the brief
6. Human commit         -> versioned change
```

A failed step is **never** an excuse to skip later steps; it is a reason to
re-enter the loop earlier.

---

## Hard constraints

AI agents **must not**:

- Invent new pack roots (e.g. `assets/packs/core/v2/`).
- Add new buckets to `defs/items/` without updating the pack loader.
- Bypass `pack_doctor.ps1` by writing directly to `generated/`.
- Author content directly in Rust source files.
- Edit `tools/validate_content.ps1` or `vv-pack-doctor` to silence a warning
  about their own content; fix the content instead.
- Duplicate concepts: one block per `.block.ron`, one material per
  `.material.ron`, one item per `.item.ron`.
- Mark a task done while errors or unjustified warnings remain.

---

## Where ambiguity lives

The pack schema is still evolving (especially recipes and worldgen). When the
schema is unclear:

- Stash the proposal in `source/production/draft/<topic>/` as a brief and a
  draft RON file.
- Do **not** invent a parallel format that "almost compiles".
- Flag the gap in the brief and stop. A human or a later agent will land the
  schema change.

This keeps the runtime untouched and the pack consistent.
