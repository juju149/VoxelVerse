# VoxelVerse V1 Gameplay Loop

V1 gameplay must be small, complete and satisfying.
A tiny loop with perfect feel beats fifty half-coded systems.

## Core loop

```text
explore -> gather -> craft -> improve -> build -> explore farther
```

Every V1 feature must strengthen this loop.

## First 15 minutes target

### Minute 0 to 2

- Player spawns safely.
- Player sees landscape and landmarks.
- UI shows hotbar unobtrusively.
- Player can move, look, jump, crouch if available.
- Player can hit a block and see cracks.

### Minute 2 to 5

- Player gathers wood/stone/fiber or equivalents.
- Drops enter hotbar or inventory clearly.
- First recipe becomes available.
- UI teaches crafting without heavy tutorial text.

### Minute 5 to 10

- Player crafts first tool.
- Tool clearly mines better than hand.
- Player mines first ore or stronger block.
- Player places blocks to build a small structure.

### Minute 10 to 15

- Player crafts first station.
- Station unlocks useful recipes.
- Player understands next biome/resource goal.

## Mining V1

Mining is hit-based and persistent.

Required behavior:

- left click or hold triggers rhythmic strikes;
- each strike applies damage;
- block crack level updates;
- damage remains if player looks away;
- wrong tool can still hit but may not drop resources;
- blocked/unbreakable blocks give clear feedback;
- breaking block edits world and refreshes dirty chunks;
- drops are added to inventory or spawned in world;
- sound changes by block material;
- impact strength drives animation and audio.

## Tool progression

Minimum V1 tool tiers:

1. hand;
2. wood/basic tool;
3. stone tool;
4. metal or improved tool.

Tool data must come from content where possible:

- tool type tag;
- tier;
- speed;
- durability if enabled;
- hand model;
- hit sound class.

## Durability rule

Durability is optional for V1. If included, it must be fully readable:

- visible durability bar;
- clear break warning;
- repair or replacement path;
- no silent disappearance without feedback.

If durability cannot be polished, disable it for V1 instead of shipping annoyance.

## Crafting V1

Crafting should start simple.

V1 station set:

1. personal hand crafting;
2. construction workbench;
3. weapon/tool workbench;
4. armor/clothing workbench if armor exists;
5. furnace/processor if smelting exists.

Do not add ten stations for tiny differences.

## Recipes V1

Every recipe must answer:

- where do ingredients come from?
- why does player want output?
- what new action does it unlock?
- is it visible in UI at the right time?

Bad recipe:

```text
3 random plants -> decorative dust with no use
```

Good recipe:

```text
fiber + stick -> basic binding -> first tool and torch path
```

## Inventory V1

Inventory is central to the game.

Required:

- 9-slot hotbar;
- inventory grid;
- stack splitting or at least reliable stack movement;
- shift-click quick move;
- selected slot clarity;
- item tooltip;
- category filter if inventory grows;
- recipe suggestions based on carried items;
- no lost items on full inventory without feedback.

## Building V1

Minimum building:

- place selected block;
- block preview or clear target selection;
- cannot place inside player;
- cannot place out of reach;
- consumes item only on success;
- placement sound and animation;
- dirty chunks refresh.

Optional but high value:

- line placement;
- fill placement;
- stairs/walls later, not before basic loop feels great.

## Combat and fauna V1

Combat can stay small.

Minimum fauna if included:

- passive animal wandering;
- can be observed;
- can drop simple resource if killed;
- not performance heavy;
- not required for first progression unless polished.

Do not add complex enemy combat before mining, crafting, building and UI are solid.

## Player guidance

The player needs a reason to continue.

V1 can use:

- discovery popups;
- simple codex entries;
- recipe unlock notices;
- landmark hints;
- biome/resource goals;
- subtle tutorial cards.

Avoid giant tutorial walls.

## Save V1

A V1 playable game should save at least:

- player position;
- inventory/hotbar;
- voxel overrides;
- block damage;
- broken props;
- world time;
- station/storage state if implemented.

If save is not implemented, V1 is not truly playable.

## Gameplay gate

Gameplay is V1-ready when:

- first 15 minutes are understandable;
- mining feels good with hand and tools;
- block damage persists;
- crafting unlocks real progression;
- player can build a small shelter/base;
- inventory supports the loop without frustration;
- all actions have audio/visual feedback;
- no item can be acquired but not used unless decorative by design;
- no recipe leads nowhere;
- no gameplay rule is hardcoded when it belongs in content.
