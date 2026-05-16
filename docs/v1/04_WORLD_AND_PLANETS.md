# VoxelVerse V1 World And Planet Design

V1 has one playable planet profile, but the architecture must already support many planets later.
The goal is to sell the illusion of an infinite metaverse through quality, distance, atmosphere and procedural variation, not through loading everything at once.

## V1 planet target

The V1 planet should be small enough to polish and large enough to feel explorable.

Target properties:

- spherical quad planet with 6 faces;
- visible curvature from hills and high points;
- high-detail voxel chunks near player;
- lower-detail tiles far away;
- no visible holes during streaming;
- deterministic generation from seed;
- sparse runtime edits;
- persistent block damage;
- destructible props tracked separately.

## World composition

The V1 world is:

```text
planet profile
+ procedural terrain base
+ biome/climate fields
+ ore/cave/vegetation/structure/fauna rules
+ sparse voxel overrides
+ persistent block damage
+ broken props
+ world time
+ player/entity state
+ station/storage state
```

The generated planet is not stored as a full voxel array.

## Procedural pipeline

Target V1 order:

1. planet profile;
2. coordinate system;
3. climate fields;
4. biome weights;
5. height composition;
6. terrain layers;
7. caves;
8. ores;
9. surface vegetation;
10. props;
11. structures;
12. fauna spawns;
13. ambience tags.

Every stage must be deterministic from seed.

## Biome V1 list

V1 should favor fewer, better biomes.

Minimum polished set:

1. starter meadow forest;
2. birch or pale forest;
3. rocky hills;
4. red desert or dry badlands;
5. snow ridge;
6. mushroom or alien grove;
7. beach/coast if water is ready;
8. cave biome if caves are playable.

Each biome must bring:

- visual identity;
- natural blocks;
- natural items;
- at least one reason to visit;
- at least one unique craft ingredient or material;
- at least one ambience/sound identity;
- at least one landmark or prop family.

## Terrain quality rules

Terrain must be readable for gameplay.

Rules:

- spawn must not be in hostile or confusing terrain;
- player must see walkable paths near spawn;
- cliffs must be interesting but not constantly blocking;
- caves must have clear entrances if enabled;
- ore access must not require blind digging for first progression;
- no biome boundary should look like a hard square seam;
- curvature must not deform nearby voxels into visually broken shapes.

## Quad sphere seam rules

The V1 must hide or solve face edge and corner issues.

Acceptable approaches:

- better coordinate mapping;
- seam-aware meshing;
- shared border sampling;
- normal smoothing where visually safe;
- skirts or stitching for LOD;
- avoid spawning fine repeated props exactly on seams;
- debug overlay for face and seam positions.

Unacceptable:

- ignoring visibly deformed cubes at face corners;
- telling the player it is normal;
- hiding seams only with fog if near-player cubes are wrong.

## Infinite illusion

The player should feel the world continues beyond what is simulated.

Use:

- atmospheric fog;
- cloud layers;
- horizon silhouettes;
- LOD terrain continuity;
- distant landmarks;
- sky color gradients;
- biome color bands;
- sound ambience changing by area;
- map/codex framing the planet as one of many worlds.

Do not use:

- infinite draw distance;
- huge active chunk counts;
- unbounded mesh queues;
- fake empty horizon;
- pop-in near crosshair.

## Runtime edits

A player edit creates:

- voxel override if placed/removed differs from generated terrain;
- dirty chunk list;
- block damage clear for edited coordinate;
- broken prop entry if support block removed;
- optional dropped item event;
- optional station/storage/entity state update.

Runtime edits must be serializable for saves later.

## Persistent block damage

Persistent block damage is core V1 identity.

Rules:

- damage remains after the player stops mining;
- damage is per coordinate and voxel id;
- if the voxel changes, damage is invalidated;
- render overlay shows clear crack stages;
- damage saves later in V1 save format;
- Pack data defines hardness and tool requirements;
- gameplay defines strike rhythm.

## Worldgen diagnostics

V1 debug overlay must show:

- face;
- chunk key;
- biome id/name;
- height;
- surface layer;
- active LOD key under cursor;
- worldgen cache hit/miss;
- props baked or skipped;
- dirty chunks pending.

## World V1 gate

The world is V1-ready when:

- spawn is stable and curated;
- at least 5 polished biomes exist;
- terrain is beautiful from spawn;
- no face-edge cube deformation ruins close gameplay;
- no visible streaming holes;
- edits update chunks correctly;
- block damage persists while playing;
- props break or disappear logically;
- worldgen errors fail before runtime;
- performance stays bounded.
