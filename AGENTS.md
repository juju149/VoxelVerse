# VoxelVerse V1 Agent Constitution

This file is the operating system for every AI agent working on VoxelVerse.
It describes the V1 we want to build, not the temporary state of the current code.

When the user says `code`, `continue`, `continu`, or gives a short instruction, the agent must:

1. Read the relevant existing code first.
2. Read the relevant V1 documents in `docs/v1/`.
3. Identify the active phase and the next unfinished gate.
4. Implement only the smallest coherent step that moves the project toward V1.
5. Run the required validation commands.
6. Refuse to advance to the next step until the current step is perfect.

## Absolute laws

### No legacy before V1

VoxelVerse V1 is not released. Therefore:

- no backward compatibility layer is allowed;
- no legacy fields are allowed;
- no deprecated aliases are allowed;
- no migration shim is allowed;
- no fallback that hides broken content is allowed;
- no dead code is allowed;
- no duplicate system is allowed;
- no temporary architecture is allowed unless it is deleted in the same task.

If an old design conflicts with the V1 target, delete or replace it. Do not preserve it.

### One responsibility per file

A file must have one clear job. If a file grows because it owns several concepts, split it before adding more logic.

Hard limits:

- 0 to 500 lines: comfortable.
- 500 to 800 lines: acceptable but watch closely.
- 800 to 1000 lines: split before adding significant behavior.
- More than 1000 lines: not acceptable for new V1 work.

Exceptions require a written reason in the file header and must be temporary.

### Runtime never repairs content

The pack compiler and pack doctor are the gates. Runtime code must not guess, repair, silently ignore, auto-create missing gameplay definitions, or accept invalid content.

Allowed runtime fallback:

- GPU device lost recovery;
- missing optional debug UI data;
- safe render fallback for a non-gameplay visual placeholder during development.

Forbidden runtime fallback:

- unresolved item, block, recipe, station, biome, model, texture, sound, or tag;
- guessed self-drop unless the schema explicitly says so;
- inferred station type from recipe name;
- silently skipped broken content;
- auto-scan media as a permanent content identity mechanism.

### Read before write

Before modifying a system, the agent must read:

- the module root `lib.rs` or `mod.rs`;
- the target file;
- all direct callers;
- all direct tests;
- the matching V1 document.

No blind patches.

### Build the game, not just the code

Every implementation must be judged on:

- product value;
- gameplay feel;
- player clarity;
- performance;
- visual quality;
- code architecture;
- modder clarity;
- testability.

If a solution is technically correct but makes the game uglier, harder to mod, slower, or less satisfying, it is not done.

## Agent workflow

### Step 1: classify the request

The agent must classify every task into one or more areas:

- foundation;
- content pipeline;
- worldgen;
- voxel runtime;
- meshing;
- rendering;
- gameplay;
- UI and UX;
- audio;
- diagnostics;
- tooling;
- content authoring;
- docs.

### Step 2: find the phase

Use `docs/v1/ROADMAP_V1.md`.

If the task belongs to an unfinished earlier phase, work on that phase first. A later feature must not be built on a rotten floorboard.

### Step 3: define the finish line

Before coding, the agent must write internally:

- what file boundaries will exist after the change;
- what old code will be removed;
- what tests will prove the behavior;
- what manual smoke test proves the game feel;
- what diagnostics will confirm performance.

### Step 4: implement with deletion

Every refactor should reduce ambiguity. Prefer deleting old paths over adding wrappers.

Good change:

- one canonical system;
- one canonical data path;
- one canonical naming rule;
- one test suite.

Bad change:

- `new_`, `old_`, `legacy_`, `v2_`, `compat_` modules;
- duplicate registry ownership;
- runtime fallback for broken authoring;
- comments promising later cleanup.

### Step 5: validate

Minimum gate for Rust changes:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

Minimum gate for pack changes:

```powershell
cargo run -p vv-pack-doctor -- assets/packs/core
cargo test -p vv-pack-loader -p vv-pack-compiler
cargo test -p vv-worldgen
```

Minimum gate for render or performance changes:

```powershell
cargo run -p voxelverse --release
```

Then capture or read the in-game diagnostics overlay and verify:

- stable frame time;
- bounded draw calls;
- bounded visible chunks;
- bounded pending mesh jobs;
- bounded GPU upload time;
- no visible LOD holes;
- no obvious shader artifacts.

### Step 6: report exactly what changed

The final answer must include:

- files changed;
- systems touched;
- old code deleted;
- tests run;
- remaining risks;
- next exact step.

## Definition of perfect for a step

A step is perfect only if:

- it compiles;
- tests pass;
- clippy passes;
- no old path remains;
- no duplicate concept remains;
- no TODO hides required behavior;
- code is split by responsibility;
- names explain intent;
- content errors fail loudly;
- performance budget is preserved;
- the player experience is better or unchanged;
- the next step is obvious.

If any of these are false, continue the current step. Do not advance.

## Forbidden phrases in code comments

Do not add comments like:

- `temporary`;
- `for now`;
- `legacy`;
- `will fix later`;
- `quick hack`;
- `compat`;
- `old system`.

If the code needs that comment, the code is not ready.

## Canonical V1 document order

Read in this order:

1. `docs/v1/00_PROJECT_VISION.md`
2. `docs/v1/01_AGENT_WORKFLOW.md`
3. `docs/v1/02_TARGET_ARCHITECTURE.md`
4. `docs/v1/03_CONTENT_AND_MODDING.md`
5. `docs/v1/04_WORLD_AND_PLANETS.md`
6. `docs/v1/05_RENDERING_AND_PERFORMANCE.md`
7. `docs/v1/06_GAMEPLAY_LOOP.md`
8. `docs/v1/07_UI_UX.md`
9. `docs/v1/08_AUDIO_AND_FEEL.md`
10. `docs/v1/09_QUALITY_GATES.md`
11. `docs/v1/10_ROADMAP_V1.md`
12. `docs/v1/11_AGENT_TASK_PROTOCOL.md`
13. `docs/v1/12_CODEBASE_AUDIT_NOTES.md`

## V1 mantra

VoxelVerse must feel infinite, clear, reactive, moddable, performant, and beautiful.
The code must feel boring, strict, small, predictable, and impossible to misunderstand.
