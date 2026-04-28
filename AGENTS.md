# AGENTS.md

# VoxelVerse - Permanent Project Constitution

This document defines the permanent vision, architecture principles, engineering rules, gameplay pillars, and agent behavior for VoxelVerse.

It is not a changelog.
It is not a migration note.
It is not a temporary planning file.
It is not a sprint plan.

Its purpose is to give any human or AI agent the stable truth of the project:
- what VoxelVerse is
- what VoxelVerse is not
- how the codebase must be shaped
- what must never be done
- how to extend the project without creating duplication, confusion, or architectural drift

When in doubt, follow this document over local habits.

---

# 1. Project identity

## 1.1 What VoxelVerse is

VoxelVerse is a moddable voxel sandbox-survival game built around:
- round planets
- fully data-driven content
- accessible survival and construction
- strong exploration motivation
- long-term systemic depth without overwhelming complexity

The game is designed to be understandable and enjoyable by a very broad audience, from children to experienced sandbox players.

VoxelVerse is not trying to win by adding the most systems.
VoxelVerse wins by being:
- clearer
- more reactive
- more alive
- more moddable
- more elegant
- more motivating to explore and build

The player should feel:
- wonder when discovering a planet
- comfort when building a home
- curiosity when seeing a landmark
- progression through better tools and deeper understanding
- attachment to places they transform

---

## 1.2 Core fantasy

The fantasy of VoxelVerse is:

> "You arrive on a small living round world, survive, understand it, shape it, and gradually turn it into your own home before reaching beyond it."

The player does not just survive in a random procedural map.
The player learns, inhabits, and transforms a world.

---

## 1.3 Product pillars

VoxelVerse stands on these permanent pillars:

### 1. Exploration with purpose
Exploration must always create desire.
The player should want to go farther because they see, suspect, or anticipate something meaningful.

### 2. Construction without friction
Building must be fast, legible, satisfying, and expressive.
The player must never pay excessive mechanical effort for the visual precision of the world.

### 3. Survival without chores
Survival must create tension and rhythm, not paperwork.
The game must avoid tedious systems, excessive micromanagement, and maintenance fatigue.

### 4. A world that reacts
The world must visibly respond to the player's actions:
- light changes safety
- structures change comfort
- terrain changes movement
- systems unlocked through discovery affect future play

### 5. Data-driven modding
Blocks, items, recipes, loot tables, world generation, entities, and balancing must be defined by data wherever possible.
Code defines systems and rules.
Content defines what exists inside those systems.

### 6. Compiled content pipeline
VoxelVerse is not only data-driven.
VoxelVerse is data-driven through a compiled content pipeline.

Raw pack files are not the runtime truth.
They are parsed, validated, normalized, resolved, compiled, and transformed into compact runtime registries.

This distinction is permanent and fundamental.

---

# 2. Audience and design philosophy

## 2.1 Accessibility target

VoxelVerse must remain accessible to a player aged 6 as well as a player aged 80.

This means:
- low cognitive friction
- simple readable UI
- immediate cause/effect
- limited number of survival meters
- visual clarity over simulation realism
- easy first minutes
- systems that layer gradually

Complexity is allowed.
Confusion is not.

---

## 2.2 Simplicity rule

The project must prefer:
- depth from interactions
over
- depth from excessive rules

A mechanic is good when:
- it is easy to understand
- it creates many meaningful situations
- it composes well with other mechanics
- it remains useful over time

A mechanic is bad when:
- it requires explanation before intuition
- it adds repetitive maintenance
- it duplicates another system
- it exists only because another game had it

---

## 2.3 "Not a chore" rule

VoxelVerse must never become a survival bureaucracy.

Avoid design that turns the player into:
- an inventory accountant
- a maintenance worker
- a menu operator
- a repetitive block-placement machine

The player should feel like:
- an explorer
- a builder
- a survivor
- a discoverer
- a world-shaper

Not a clerk with a pickaxe.

---

# 3. Permanent gameplay truths

## 3.1 Survival philosophy

Survival exists to provide rhythm, stakes, and motivation.
It must not bury the player in management.

Preferred:
- health
- hunger
- optionally light stamina or effort systems if they serve feel

Avoid by default:
- over-detailed thirst
- disease simulation
- excessive temperature bookkeeping
- inventory weight punishment
- maintenance systems that interrupt the fun loop too often

---

## 3.2 Construction philosophy

Construction is one of the core joys of VoxelVerse.

The world uses 50 cm voxels, but the player must never experience this as tedious micro-placement.

Permanent construction rule:

> The engine may be precise.
> The player interaction must remain broad, fast, and satisfying.

This means:
- smart snapping
- line placement
- surface fill
- wall generation
- stair generation
- terrain shaping tools
- contextual connection rules
- optional precision when wanted, not mandatory all the time

The player should see fine detail without paying with excessive clicks.

---

## 3.3 Exploration philosophy

Exploration must be driven by visible desire, not just coordinates.

Preferred motivators:
- landmarks
- visible anomalies
- silhouettes on the horizon
- biome contrasts
- structures
- lighting cues
- rare resources tied to discovery
- world transformation opportunities

The player should often think:
- "What is that?"
- "Can I reach it?"
- "What happens if I activate this?"
- "What can I build with what I found?"

Not:
- "I guess I should walk in a random direction for twenty minutes."

---

## 3.4 Progression philosophy

Progression must combine:
- material progression
- geographic progression
- comfort progression
- knowledge progression

The player should not only get stronger.
The player should also get:
- faster
- safer
- more expressive
- more efficient
- more capable of shaping the world

Progression should unlock better play, not just larger numbers.

---

## 3.5 Planet philosophy

Round planets are not a visual gimmick.
They are central to identity.

A round planet should:
- create memorable geography
- create visible world curvature
- make travel feel meaningful
- make landmarks readable
- make bases visible in relation to the whole world
- create attachment to a complete place

The player should feel they live on a world, not on an endless noise field.

---

# 4. Architectural law

VoxelVerse must be built with clear, enforced boundaries.

A crate exists to answer one responsibility.
A module exists to express one coherent concern.
A file exists to hold one meaningful concept.

If a file, module, or crate starts answering multiple unrelated questions, it must be split.

---

## 4.1 Architectural goals

The codebase must be:
- data-driven
- modular
- explicit
- low-duplication
- low-coupling
- high-cohesion
- safe to extend
- safe for AI-assisted editing

The architecture must make the correct change obvious.

---

## 4.2 Stable layer model

VoxelVerse is organized around these permanent layers:

### 1. Foundation
General utilities, low-level shared primitives, math.

### 2. Engine runtime
Voxel storage, world runtime, physics, meshing, rendering, input.

### 3. Content schema
All raw data definitions and schema types for blocks, items, entities, recipes, tags, settings, worldgen definitions, language, UI themes, and related content.

### 4. Pack loading
Filesystem discovery, pack manifests, path conventions, raw parsing, and loading order.

### 5. Content compilation
Normalization, deep validation, reference resolution, merge and override application, tag expansion, compiled content construction, and preparation for runtime use.

### 6. Runtime registries
Compact ids, typed tables, indexes, and fast runtime access to compiled content.

### 7. Simulation and gameplay
Player rules, mining, placement, crafting, loot, interactions, progression.

### 8. World generation
Procedural algorithms that consume compiled content and runtime registries to produce runtime world data.

This layered model is permanent even if internal folders evolve.

---

# 5. Crate responsibilities

The exact set of crates may evolve, but the responsibilities below are stable and must remain distinct.

## 5.1 Core
Core contains only the smallest shared building blocks:
- generic ids
- shared errors
- minimal utility traits
- tiny low-level abstractions

Core must not know:
- blocks
- items
- planets
- recipes
- players
- rendering details

Core must remain small.

If "core" starts knowing the game world, it is no longer core.

---

## 5.2 Math
Math contains geometry and pure mathematical logic:
- AABB
- ray
- intersection
- planes, directions, transforms if needed

Math must remain domain-light.
Math must not know blocks, players, worldgen content, or rendering pipelines.

---

## 5.3 Voxel runtime
Voxel runtime contains the low-level voxel language of the engine:
- voxel coordinates
- chunk coordinates
- chunk sizes
- voxel storage
- faces and directions
- compact runtime block ids

This layer must not encode concrete content such as "stone", "grass", "iron_ore" as hardcoded truth.

Concrete content belongs to content registries, not to the low-level voxel engine.

---

## 5.4 World runtime
World runtime contains:
- planets
- chunk maps
- sparse structures
- gravity topology
- runtime world queries
- read and write access to runtime world state

World runtime stores compact runtime ids, not filesystem strings or raw content definitions.

World runtime must not directly own pack loading logic.
World runtime must not directly own content compilation logic.

---

## 5.5 Meshing
Meshing transforms runtime voxel data into mesh data.
It may include:
- greedy meshing
- extraction logic
- mesh buffers
- LOD meshing rules if they are geometry-side

Meshing is not world generation.
Meshing is not rendering.
Meshing is not gameplay.

---

## 5.6 Physics
Physics owns:
- movement
- collisions
- solvers
- character-body interaction with world geometry

Physics does not own inventory, hotbar, recipes, or progression.

---

## 5.7 Input
Input owns:
- keyboard, mouse, and controller state
- action mapping
- input bindings
- abstract player intent

Input expresses what the player wants to do.
Input does not decide the rules of what happens.

Example:
- input may say "place block"
- gameplay decides whether placement is valid

---

## 5.8 Content schema
Content schema owns the raw data definitions for:
- blocks
- items
- entities
- placeables
- recipes
- loot tables
- tags
- settings
- lang
- UI themes
- worldgen definitions
- weather
- structures
- flora and fauna
- planet types
- biome definitions
- universe definitions

Content schema defines the shape of the data.
It does not:
- scan the filesystem
- assign runtime ids
- build final registries
- execute gameplay rules
- define render backends

---

## 5.9 Pack loading
Pack loading owns:
- discovering packs
- reading manifests
- reading files
- parsing raw documents
- pack ordering
- dependency order
- pack-level diagnostics related to discovery and parsing

Pack loading must not:
- implement gameplay rules
- assign compact runtime ids
- become the owner of compiled content
- become the runtime registry layer

---

## 5.10 Content compilation
Content compilation owns:
- path-derived identity normalization
- deep validation
- reference resolution
- merge and override application
- tag resolution and expansion
- semantic validation
- compilation of raw documents into compiled content
- preparation of data for runtime registries

Content compilation must not:
- own filesystem discovery
- define gameplay rules
- define rendering pipelines
- become a dumping ground for unrelated engine logic

---

## 5.11 Runtime registries
Runtime registries own:
- compact runtime ids
- typed tables
- lookup indexes
- fast access to compiled content
- memory-efficient runtime representations

Runtime registries are the runtime truth for content access.
Gameplay, world generation, and rendering consume registries, not raw pack documents.

Runtime registries must not:
- parse files
- own filesystem paths
- reintroduce raw content ambiguity
- become a second schema layer

---

## 5.12 World generation
World generation owns algorithms.
It consumes compiled content, settings, and runtime registries to build runtime worlds.

World generation may include:
- noise
- climate sampling
- biome selection
- ore placement
- flora placement
- structure placement
- fauna spawning rules
- planet generation pipeline

World generation must not define its own hardcoded content lists if those can be data-driven.

---

## 5.13 Gameplay
Gameplay owns the rules of the game:
- player state
- inventory
- hotbar
- mining logic
- placement logic
- crafting execution
- smelting execution
- drops and loot application
- interactions
- survival rules
- progression

Gameplay may consume runtime registries, but it must not re-declare content definitions in code.

---

## 5.14 Rendering
Rendering owns visual representation:
- pipelines
- shaders
- GPU buffers
- upload logic
- visibility and culling on the rendering side
- shadows
- overlays
- UI rendering assistance

Rendering consumes runtime data and compiled rendering-related content definitions.
Rendering does not own world rules.

---

# 6. Data-driven law

VoxelVerse is permanently data-driven.

This means:
- content is authored as data
- code interprets and executes systems
- content packs define what exists
- runtime systems consume compiled content through registries

---

## 6.1 What must be data-driven

The following must be data-driven whenever possible:
- blocks
- items
- entities
- placeables
- recipes
- loot tables
- tags
- balancing values
- world settings
- biome definitions
- planet types
- flora
- fauna
- ores
- structures
- weather
- localization
- UI theme data

---

## 6.2 What should stay in code

The following belong primarily in code:
- simulation algorithms
- meshing algorithms
- physics solvers
- placement validation logic
- crafting execution systems
- registry construction
- pathfinding or navigation systems
- render backends
- performance-critical runtime logic

---

## 6.3 Runtime ids rule

The runtime world must never store verbose content keys or raw strings per voxel.

World runtime stores compact runtime ids.
Runtime registries map:
- stable content keys
to
- compact runtime ids

This is mandatory for performance, memory efficiency, and clarity.

---

## 6.4 Raw vs compiled vs runtime

All content systems should distinguish between:
- raw definitions loaded from packs
- compiled content produced by content compilation
- runtime registry data used by the engine

Raw definitions may contain symbolic references:
- keys
- tags
- logical resource references
- localization handles

Compiled content must be:
- normalized
- validated
- resolved
- structurally reliable

Runtime registry data must be:
- compact
- fast
- indexed
- ready for engine consumption

Never blur these three stages.

---

## 6.5 Path-as-identity rule

Content identity is derived from pack namespace and canonical path conventions.

The path is the source of identity.
Do not duplicate this identity as a second mandatory id field inside raw content files unless there is a very strong and explicit reason.

This avoids:
- duplicated truth
- drift between file name and declared id
- accidental mismatches
- unnecessary authoring burden

---

## 6.6 Logical references over filesystem leakage

Raw content may reference:
- content keys
- tag keys
- lang keys
- logical resource references

Do not make gameplay or runtime definitions depend on raw filesystem paths more than necessary.

Logical resource references should remain stable even if internal asset storage evolves.

---

# 7. Anti-duplication law

Code duplication is one of the most dangerous long-term failures in this project.

The project must avoid:
- duplicated structs with slightly different names
- multiple sources of truth for the same concept
- repeated parsing logic
- repeated validation logic
- repeated block, item, or worldgen concepts under different modules
- repeated helper utilities hidden in unrelated files

---

## 7.1 One concept, one owner

Each concept must have exactly one canonical owner.

Examples:
- block definition schema -> content schema
- pack parsing -> pack loading
- content compilation -> content compilation layer
- runtime ids -> runtime registries
- world storage -> world runtime
- player inventory -> gameplay
- input action mapping -> input
- AABB and ray math -> math
- voxel coordinates -> voxel runtime

If the same concept appears in two places, one of them is probably wrong.

---

## 7.2 Do not create shadow abstractions

Do not create a second "almost the same" type because it is convenient locally.

Bad examples:
- multiple chunk position types with slightly different semantics
- multiple block descriptor types that actually represent the same layer
- multiple ad hoc recipe structs
- multiple LOD policies in different crates without a single owner
- multiple content key representations that drift apart

Prefer:
- reusing the canonical type
- introducing a clearly named adapter
- defining a strict boundary conversion when layers differ

---

## 7.3 No junk drawer files

Avoid generic catch-all files such as:
- `types.rs`
- `utils.rs`
- `helpers.rs`
- `common.rs`
- `misc.rs`

These files become hiding places for duplication and architectural erosion.

Prefer explicit files named after real concepts:
- `block.rs`
- `recipe.rs`
- `chunk_map.rs`
- `collision.rs`
- `registry.rs`
- `planet.rs`

---

# 8. Naming law

Names must reveal responsibility.

Prefer:
- precise names
- domain language
- stable terminology
- boring clarity over vague cleverness

Good names:
- `content_key`
- `block_registry`
- `chunk_map`
- `placement_rules`
- `recipe_resolver`
- `planet_generator`
- `pack_manifest`
- `raw_block_def`
- `compiled_recipe`

Bad names:
- `manager`
- `system`
- `processor`
- `handler`
- `stuff`
- `misc`
- `data`
unless the responsibility is genuinely exact and narrow

---

# 9. Dependency law

Dependencies must flow downward.
A lower-level crate must never depend on a higher-level crate.

Stable dependency principles:
- runtime foundations do not depend on gameplay
- content schema does not depend on rendering
- pack loading does not depend on gameplay rules
- content compilation does not depend on rendering
- world runtime does not depend on pack filesystem logic
- input does not depend on gameplay outcomes
- render does not define game rules

If adding a dependency feels convenient but crosses responsibilities, the architecture is likely wrong.

---

# 10. Modding law

VoxelVerse is built for moddability.
Modding is not an afterthought.

This means:
- content keys must be stable
- pack boundaries must be clear
- validation must be strict and helpful
- content data formats must be structured and consistent
- tags must be first-class
- override behavior must be deterministic
- engine logic must not hardcode content unnecessarily

Modding support is a core product feature, not optional decoration.

---

## 10.1 Mod-friendly rules

Prefer systems that let a modder:
- add a block without editing engine code
- add a biome without editing engine code
- add a recipe without editing engine code
- add worldgen content through data
- create variants through tags and defs
- understand errors from validation diagnostics

Never force content authors to touch low-level engine code unless absolutely necessary.

---

## 10.2 Pack authoring rules

Pack authors should work mainly with:
- defs
- lang
- logical resources
- tags
- recipes
- worldgen data

They should not need to understand:
- runtime ids
- internal engine storage layout
- meshing internals
- physics solver internals
- registry allocation strategies

A pack should feel like authored content, not engine surgery.

---

# 11. Rust engineering rules

## 11.1 Single responsibility first
Every module should do one thing well.

Before adding code, ask:
- what responsibility owns this?
- is this runtime, schema, pack loading, compilation, registry, generation, rendering, or gameplay?
- is there already a canonical home for this concept?

Do not add code to the nearest convenient file if the responsibility is elsewhere.

---

## 11.2 Prefer composition over giant files
Large files usually mean unclear boundaries.

Split when:
- multiple concerns coexist
- internal sections stop being tightly related
- the file requires scrolling to discover unrelated concepts
- a type could stand alone with a clear name

---

## 11.3 Keep public APIs small
Default to private.
Use `pub(crate)` before `pub`.

Expose only what should truly be used outside the module or crate.

A wide public API multiplies architectural chaos.

---

## 11.4 Avoid premature generic abstraction
Do not create a generic framework when a concrete solution is sufficient.

Prefer:
- a simple concrete type
- a small trait when there is a real abstraction boundary
- explicit conversion functions

Avoid generic layers created only because they "might be useful later".

---

## 11.5 Avoid macro-heavy hidden logic
Macros are acceptable when they:
- reduce obvious repetition
- improve clarity
- preserve debuggability

Macros are not acceptable when they:
- hide architecture
- encode runtime logic in opaque ways
- make content behavior hard to trace

---

## 11.6 Error handling
Errors must be explicit and informative.

Pack and content-related errors must:
- identify the resource
- identify the file
- identify the broken reference when possible
- explain what was expected

Panics are not a normal validation strategy.

---

## 11.7 Logging and diagnostics
Diagnostics should help both engine programmers and content authors.

When possible, errors should answer:
- what failed
- where it failed
- why it failed
- how to fix it

Especially for pack loading, content compilation, registry construction, and worldgen validation.

---

# 12. Gameplay implementation rules

## 12.1 No content hardcoding when data should own it
Do not hardcode:
- block lists
- recipe lists
- tool requirements
- biome definitions
- ore distributions
- weather tables
when those are intended to be content data

---

## 12.2 Rules in code, values in data
A good pattern:
- code decides how crafting works
- data decides which recipes exist

Another:
- code decides how mining works
- data decides block hardness, tags, required tool, drops

This split is fundamental.

---

## 12.3 The player should feel progress through comfort
Gameplay systems should reward the player with:
- better tools
- better building speed
- better traversal
- better safety
- better readability
- better world influence

Not just higher damage numbers.

---

# 13. World generation rules

## 13.1 Worldgen must consume content, not redefine it
Biome and structure logic may be algorithmic, but the content definitions themselves must be data-driven whenever practical.

Do not recreate content in worldgen code if runtime registries already define it.

---

## 13.2 Worldgen must be deterministic
Generation must be deterministic from explicit inputs:
- seed
- planet parameters
- compiled content and registries
- relevant settings

The same inputs must produce the same outputs unless explicitly designed otherwise.

---

## 13.3 Worldgen must stay explainable
Even procedural complexity should remain reasoned and debuggable.

Worldgen must not become an opaque chaos machine.
It must remain inspectable, reproducible, and tunable.

---

# 14. Rendering rules

## 14.1 Rendering reflects runtime, it does not own it
Rendering should consume runtime data and compiled rendering-related content definitions.

Rendering must not become the hidden owner of:
- gameplay flags
- world rules
- block semantics unrelated to visuals

---

## 14.2 Visual precision must not increase player friction
The project uses 50 cm voxels, but the user experience must remain broad and forgiving.

This applies to:
- build tools
- cursor logic
- snapping
- terrain editing
- visual previews

The renderer may show precision.
Gameplay must preserve ease.

---

# 15. Input rules

## 15.1 Input expresses intent only
Input translates device state into abstract actions.

Input does not decide:
- whether a block can be placed
- whether an item exists
- whether a recipe is valid
- whether an interaction succeeds

Those belong to gameplay and simulation layers.

---

# 16. Performance rules

## 16.1 Performance is a feature
VoxelVerse is a runtime-heavy game.
Performance is part of correctness.

Systems must be designed with:
- memory locality
- compact runtime ids
- chunk-aware processing
- bounded allocations
- explicit ownership
- predictable update flow

---

## 16.2 Optimize in the right layer
Do not move logic into a wrong crate or wrong abstraction layer just because it seems faster to implement there.

Bad architecture is not a valid optimization strategy.

---

## 16.3 Measure before distorting design
Do not introduce architectural damage for hypothetical performance wins.

Profile first.
Preserve clear ownership whenever possible.

---

# 17. AI agent operating rules

These rules are mandatory for any AI agent modifying the project.

## 17.1 First question rule
Before making changes, always ask internally:

1. What responsibility owns this change?
2. Which crate should own it?
3. Is there already a canonical type or module for this concept?
4. Am I introducing duplication?
5. Should this be data-driven instead of hardcoded?

If these questions are not answered, the change is not ready.

---

## 17.2 Never patch blindly
Do not scatter fixes in multiple unrelated modules to make compilation succeed quickly.
Find the real owner of the change and implement it there.

---

## 17.3 Never create duplicate concepts just to unblock yourself
Do not create:
- another block type
- another item descriptor
- another position type
- another registry concept
- another "temporary" helper system
when a canonical one already exists or should exist

---

## 17.4 Prefer moving code to its rightful owner over copying it
If logic exists in the wrong place, move it.
Do not duplicate it into the new place.

---

## 17.5 Prefer canonical extension over local hacks
If a system needs a new capability, extend the owner cleanly.
Do not hack around the owner from a consumer crate.

---

## 17.6 Preserve data-driven architecture
If an AI agent is tempted to hardcode a list of blocks, recipes, biomes, tags, drops, or balance values, it must stop and check whether this belongs in pack data.

Most of the time, it does.

---

## 17.7 Avoid temporary scaffolding in permanent code
Do not leave behind:
- placeholder types
- duplicate transitional adapters
- debug-only architecture shortcuts
- unexplained compatibility layers
unless they are explicitly justified and documented

The codebase should converge toward clarity, not accumulate sediment.

---

## 17.8 Respect the project's conceptual vocabulary

Use the established language of the project:
- content key
- tag key
- lang key
- resource reference
- pack
- pack manifest
- raw definition
- raw document
- pack loader
- content compiler
- compiled content
- runtime id
- registry
- voxel
- chunk
- planet
- gameplay rule
- world generation
- render pipeline

Do not invent parallel vocabulary for the same concepts.

---

# 18. What must never be done

The following are project-level anti-patterns and must be avoided.

## 18.1 Never make `core` a dumping ground
If a type is game-specific, it likely does not belong in core.

## 18.2 Never hardcode content lists in engine crates
Do not encode "stone", "grass", "iron_ore", etc. as permanent engine truth if content packs should own them.

## 18.3 Never let input own gameplay state
No player inventory, hotbar logic, or crafting rules inside input.

## 18.4 Never let worldgen own meshing
Meshing is a separate responsibility.

## 18.5 Never let rendering become gameplay
Render reflects state. It does not define rules.

## 18.6 Never store content strings per voxel in runtime world data
Use compact runtime ids.

## 18.7 Never create vague files to postpone architecture decisions
No junk drawers.

## 18.8 Never solve duplication with inheritance chains or abstraction towers
Prefer clear ownership and small composition.

## 18.9 Never add a new crate without a true ownership boundary
A new crate must represent a real conceptual boundary, not a naming convenience.

## 18.10 Never optimize by violating responsibility boundaries
Fast wrong architecture becomes slow chaos later.

## 18.11 Never blur raw, compiled, and runtime content stages
Do not treat parsed pack files, compiled content, and runtime registry data as if they were the same thing.

## 18.12 Never make filesystem layout the runtime API
Filesystem layout is a source of authored content.
It is not the runtime access model.

---

# 19. Preferred development behavior

When implementing a new feature, follow this order:

1. Define the responsibility
2. Identify the owner layer and crate
3. Check whether the change belongs in content data, raw schema, compilation logic, runtime registry logic, runtime engine logic, or both
4. Reuse canonical types
5. Add or refine tests and validation
6. Keep public APIs small
7. Keep names precise
8. Remove duplication instead of adding more

---

# 20. Definition of done for architectural quality

A change is not complete just because it compiles.

A change is only complete when:
- responsibility is correct
- ownership is clear
- no obvious duplication was introduced
- data-driven logic stayed data-driven
- raw, compiled, and runtime stages stayed distinct
- public API growth is justified
- naming is explicit
- validation and errors remain useful
- the code helps future agents make the next correct change

---

# 21. Final principle

VoxelVerse must remain a coherent world, not a pile of features.

Every addition must reinforce:
- clarity
- wonder
- moddability
- elegance
- systemic depth
- construction joy
- exploration desire
- architectural sanity

If a change makes the project harder to understand, harder to extend, more duplicated, more hardcoded, more ambiguous between raw and runtime, or more tedious, it is not aligned with VoxelVerse.

Build the game like the planets it contains:
structured, alive, readable, and worth exploring.