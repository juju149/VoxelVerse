# VoxelVerse V1 Roadmap

This roadmap is ordered. Do not skip phases.
A later phase may only start when every previous phase gate passes.

## Phase 0: Documentation and agent control

Goal: make the project safe for autonomous continuation.

Tasks:

- add `AGENTS.md`;
- add V1 docs;
- mark V1 docs as source of truth;
- remove or rewrite docs that describe old layout as current target;
- define validation commands;
- define done criteria.

Gate:

- agent can answer what to do next without guessing;
- docs do not contradict the V1 architecture;
- no doc encourages legacy compatibility.

## Phase 1: Foundation cleanup

Goal: remove architectural rot before adding features.

Tasks:

- split app runtime loop into focused modules;
- introduce `GameApp` or equivalent state owner;
- route events through small routers;
- centralize feedback events;
- remove direct cross-system poking where possible;
- enforce file size limits;
- fix clippy warnings.

Gate:

- app loop is readable;
- no giant new functions;
- gameplay feedback can route to render/audio/UI;
- tests pass.

## Phase 2: Content pipeline V1 strictness

Goal: make packs impossible to misunderstand.

Tasks:

- align docs and code on `.object.ron` as canonical gameplay object;
- remove old separate content folder assumptions from docs/tools;
- make Pack Doctor stricter;
- forbid silent self-drop if V1 schema chooses explicit drops;
- formalize voxel model manifests;
- ensure station tags are fully qualified;
- improve error messages.

Gate:

- core pack passes doctor;
- broken content fails with precise error;
- no runtime fallback hides missing content.

## Phase 3: World and streaming stability

Goal: player can move without holes, spikes or seam ugliness.

Tasks:

- inspect quad sphere seams and corners;
- fix close-range cube deformation or define robust seam strategy;
- tune LOD split/hysteresis/budgets;
- improve parent LOD retention;
- stabilize dirty chunk rebuilds;
- add debug overlay for face/chunk/LOD;
- verify prop LOD does not spike.

Gate:

- no visible near-player seam corruption;
- no horizon holes;
- turning fast stays bounded;
- diagnostics prove budgets.

## Phase 4: Visual V1 beauty pass

Goal: make the world beautiful before gameplay expands.

Tasks:

- tune atmosphere/fog/sky;
- stabilize shadows;
- polish terrain textures;
- add/clean biome color identity;
- fix water or cut it from V1;
- tune first-person hand/tool render;
- add screenshot test/golden scene workflow.

Gate:

- spawn screenshot looks like a real game;
- no fog band artifacts;
- UI readable over world;
- stable FPS in release.

## Phase 5: Mining and tool feel

Goal: make the core action addictive.

Tasks:

- data-drive mining strike tuning where needed;
- persistent damage save path design;
- improve crack overlay stages;
- add hand/tool swing timing;
- route feedback events;
- improve sound variation;
- wrong-tool feedback;
- drops and inventory integration.

Gate:

- mining by hand and tool feels good;
- cracks persist;
- wrong tool is understandable;
- block break updates world and inventory correctly.

## Phase 6: Inventory, crafting and stations

Goal: complete the first progression loop.

Tasks:

- inventory polish;
- craftable recipe panel;
- missing ingredient hints;
- hand crafting;
- construction workbench;
- tool/weapon workbench;
- furnace/processor only if polished;
- storage UI if chest exists.

Gate:

- first tool can be crafted without external guide;
- station UI is clear;
- no item loss;
- recipes have sources and uses.

## Phase 7: Biomes and natural content MVP

Goal: make exploration meaningful.

Tasks:

- 5 to 8 polished biomes;
- natural blocks and items;
- trees/vegetation/props;
- ores and caves if ready;
- simple fauna if performance allows;
- ambience per biome;
- landmarks or structures.

Gate:

- every biome has gameplay reason;
- resources feed recipes;
- no decorative bloat without purpose;
- performance stable with props.

## Phase 8: Save and playable session

Goal: V1 becomes a game, not a toy session.

Tasks:

- save format for player/inventory/world edits/block damage;
- load flow;
- save corruption handling;
- versioning without legacy burden before release;
- manual save or autosave;
- tests for save roundtrip.

Gate:

- quit and reload preserves meaningful progress;
- no runtime panic on missing save;
- save format documented.

## Phase 9: V1 polish and release candidate

Goal: lock V1 scope and remove rough edges.

Tasks:

- bug triage;
- performance profiling;
- UX polish;
- audio mix;
- settings menu;
- controls screen;
- build packaging;
- README update;
- final content audit.

Gate:

- 30-minute playable session;
- no critical bug;
- all quality gates pass;
- docs match code;
- V1 scope frozen.

## Rule for agents

When asked to continue, pick the earliest phase with an unfinished gate.
Do not add a late feature because it is exciting if an earlier phase is unstable.
