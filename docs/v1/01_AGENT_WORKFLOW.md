# VoxelVerse V1 Agent Workflow

This document defines how an AI coding agent must work on VoxelVerse.
The goal is to make short prompts like `code`, `continue`, or `next` safe.

## Default behavior on `code` or `continue`

When the user gives a short continuation prompt, the agent must not ask what to do if the next step is discoverable from docs and code.

The agent must:

1. Read `AGENTS.md`.
2. Read `docs/v1/10_ROADMAP_V1.md`.
3. Identify the earliest unfinished phase.
4. Read relevant code before changing anything.
5. Implement the smallest high-impact task in that phase.
6. Validate.
7. Report result and next step.

## Task selection priority

Always choose work in this order:

1. compile or test failure;
2. architectural violation;
3. performance regression;
4. broken gameplay loop;
5. broken content validation;
6. visual artifact;
7. UX friction;
8. missing polish;
9. optional feature.

A broken foundation beats a shiny feature.

## Before coding checklist

For every task, the agent must know:

- target player value;
- target technical value;
- files to read;
- files likely to change;
- code to delete;
- tests to add or update;
- validation commands;
- manual smoke test.

If this is not clear, inspect more code. Do not guess.

## Reading protocol

### Rendering task

Read:

- `crates/vv-render/src/lib.rs`;
- `crates/vv-render/src/renderer.rs`;
- target render submodule;
- `crates/vv-meshing/src/lib.rs` if geometry changes;
- relevant WGSL shader;
- `docs/v1/05_RENDERING_AND_PERFORMANCE.md`.

### Gameplay task

Read:

- `crates/vv-gameplay/src/lib.rs`;
- target gameplay module;
- app wiring in `apps/voxelverse-game/src/app/runtime_loop.rs`;
- world APIs in `crates/vv-world/src/planet.rs` if blocks change;
- `docs/v1/06_GAMEPLAY_LOOP.md`.

### Content task

Read:

- `crates/vv-content-schema/src/object.rs`;
- `crates/vv-pack-loader/src/loader.rs`;
- `crates/vv-pack-compiler/src/object_compiler.rs`;
- `docs/v1/03_CONTENT_AND_MODDING.md`;
- `docs/PACK_V1.md`.

### Worldgen task

Read:

- `crates/vv-worldgen/src/lib.rs`;
- `crates/vv-worldgen/src/procedural/mod.rs`;
- relevant procedural submodules;
- relevant `assets/packs/core/defs/world/**` files;
- `docs/v1/04_WORLD_AND_PLANETS.md`.

### UI task

Read:

- `crates/vv-render/src/ui/mod.rs`;
- `crates/vv-render/src/ui/theme.rs`;
- `crates/vv-render/src/renderer/inventory*`;
- `apps/voxelverse-game/src/app/inventory_events.rs`;
- `docs/v1/07_UI_UX.md`.

## Implementation rules

### Make boundaries sharper

Every change must make ownership clearer.

Examples:

- input sampling belongs to gameplay or app input, not renderer;
- chunk streaming policy belongs to render streaming, not app loop;
- pack validation belongs to compiler or doctor, not runtime;
- UI geometry belongs to UI modules, not random event handlers;
- audio event choice belongs to gameplay feedback mapping, not scattered calls.

### Delete old path first

If replacing a system:

1. find all callers;
2. remove the old public API;
3. update call sites;
4. delete old module;
5. run tests.

Do not leave old and new systems side by side.

### No silent behavior changes

When gameplay feel changes, add explicit tuning constants or data fields with names.
Do not hide design decisions inside magic numbers.

Bad:

```rust
cooldown = 0.37;
```

Good:

```rust
const WOOD_PICKAXE_BASE_STRIKE_COOLDOWN_SECONDS: f32 = 0.37;
```

Better when moddable:

```ron
mining: (base_cooldown_seconds: 0.37)
```

## Completion report format

Every agent response after coding must use this structure:

```md
## Done
- ...

## Files changed
- `path`: what changed

## Deleted
- `path`: why removed

## Validation
- `command`: pass/fail

## Manual smoke test
- what was tested in game

## Risks
- known limitations

## Next step
- exact next task
```

## Stop conditions

The agent must stop and fix the current step if:

- formatting fails;
- clippy fails;
- tests fail;
- game fails to start;
- pack doctor reports warnings for touched content;
- FPS or frame time clearly regresses;
- new code duplicates existing concept;
- old path still exists;
- feature is not understandable to a player.

## How to handle uncertainty

If the agent is unsure, it must inspect more code or make a conservative local improvement.
Do not ask the user to repeat information already available in docs or code.

## Ideal agent mindset

Be strict like a compiler, curious like a designer, and tidy like a person who knows future-you will have to debug this at 2 a.m.
