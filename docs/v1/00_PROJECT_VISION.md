# VoxelVerse V1 Product Vision

VoxelVerse V1 is the first complete playable version of a spherical voxel survival and creation game.
It is not a tech demo, not a Minecraft clone with a round map, and not an engine sandbox.
It is a small but excellent game slice that proves the identity of VoxelVerse.

## Product promise

VoxelVerse V1 lets a player land on a small procedural spherical planet, explore beautiful biomes, mine blocks with satisfying persistent damage, collect resources, craft useful tools and stations, build a small base, survive light environmental pressure, and always understand what to do next.

The V1 must answer one question:

> Can this become the cleanest, most moddable, most satisfying voxel planet game?

## Player fantasy

The player is not just digging cubes. The player is settling a living tiny world.

The fantasy pillars:

1. **A planet under your feet**: the horizon curves, landmarks matter, the sky sells scale.
2. **Every hit matters**: mining is physical, rhythmic, persistent, and juicy.
3. **Everything is understandable**: blocks, items, recipes, stations and biomes are readable.
4. **Creation is fast**: building should feel expressive, not bureaucratic.
5. **The world is moddable by humans**: content files must be strict but clear.
6. **The engine is invisible**: no freeze, no empty horizon, no ugly LOD pop, no broken UI.

## Target V1 experience

The player starts on a planet with:

- a readable spawn area;
- 5 to 8 polished natural biomes;
- visible curvature and strong atmosphere;
- terrain that invites exploration;
- trees, plants, rocks, ores and simple fauna;
- a first-person character with hands/tool animation;
- hotbar, inventory, crafting and station UI;
- mining with persistent cracks;
- placing blocks;
- crafting progression;
- audio feedback for footsteps, mining, breaking, placing and UI;
- a minimal tutorial path through discoveries and codex entries;
- stable performance.

## V1 game loop

1. Wake up or spawn on the planet.
2. Look around and immediately see at least two interesting landmarks.
3. Pick up or mine basic resources.
4. Craft first tools.
5. Mine better resources faster.
6. Build a small shelter or platform.
7. Craft workbenches.
8. Unlock more recipes.
9. Explore another biome.
10. Return with new materials.
11. Improve tools, base and inventory.
12. Reach a V1 milestone objective.

## What V1 is not

V1 is not:

- multiplayer;
- infinite universe travel;
- full metaverse identity system;
- full automation factory;
- complex NPC villages;
- advanced combat RPG;
- full quest campaign;
- mod marketplace;
- vehicle system;
- giant planets with everything generated at once.

These may exist later. V1 must be a perfect seed, not a bloated asteroid belt.

## Design tone

VoxelVerse should feel:

- premium but simple;
- cozy but not childish;
- adventurous but not stressful;
- stylized but not noisy;
- readable from distance;
- satisfying in tiny actions;
- calm enough to build;
- mysterious enough to explore.

## Visual target

The V1 visual style is stylized modern voxel, not photorealistic.

Rules:

- large readable forms;
- simple clean textures;
- subtle PBR-lite lighting;
- no noisy micro-detail;
- no overgrown visual clutter;
- strong color identity per biome;
- atmospheric sky and fog doing much of the beauty work;
- blocks remain legible as blocks.

## Interaction target

Every frequent action must be fast:

- mine: one click or held rhythm, immediate feedback;
- place: one right click, clear preview;
- switch tool: mouse wheel or number key;
- craft: few clicks, clear result;
- move stack: fast gestures and shortcuts;
- close UI: instant return to world.

The UI must serve muscle memory. It must not behave like a spreadsheet wearing armor.

## Modding target

A modder should be able to add a simple block, item, recipe, biome feature or prop without reading engine code.

They should understand:

- where the file goes;
- what the file is called;
- what every field means;
- what references are valid;
- what error they made when compilation fails.

Strictness is kindness. A red compiler error today prevents a haunted runtime bug tomorrow.

## V1 success metrics

V1 is successful when:

- a new player understands the first 10 minutes without external explanation;
- the world looks good in screenshots from spawn;
- mining and placing feel good after 5 minutes and after 2 hours;
- a small base can be built smoothly;
- a modder can add a basic object from docs alone;
- the game runs without visible streaming holes;
- no critical system lives in a giant mixed-responsibility file;
- pack doctor rejects broken content with precise messages;
- an AI agent can continue development safely from docs.

## V1 North Star

Make the smallest complete VoxelVerse that feels like a real game and is built on an architecture clean enough to survive 10 years of content.
