import type { BlockDef, MaterialFaceDef, MaterialStylePreset, PackProject } from "../types/studio";
import { createBlockFromPreset } from "../lib/presets/blockPresets";
import { createMaterialFromPreset } from "../lib/presets/materialPresets";
import { validatePack } from "../lib/validation/packValidator";

// ---------------------------------------------------------------------------
// Material library — at least 10 face-based procedural materials.
// Each material uses a distinct kind / style / seed combination so that the
// blueprint graph and pattern layers produce visually different results.
// ---------------------------------------------------------------------------

// The 'kind' parameter here maps to the internal MaterialKind type inside
// materialPresets.ts. Using 'as any' is intentional: the type is private to
// the presets module and the string values are stable.
type InternalKind = Parameters<typeof createMaterialFromPreset>[0];

function mat(kind: InternalKind, style: MaterialStylePreset, seed: number, displayName?: string): MaterialFaceDef {
  const m = createMaterialFromPreset(kind, style);
  m.seed = seed;
  if (displayName) m.displayName = displayName;
  return m;
}

const grassTop      = mat("grass_top",   "soft_natural",   149, "Cartoon Grass Top");
const grassSide     = mat("grass_side",  "soft_natural",   251, "Cartoon Grass Side");
const dirtBase      = mat("dirt_base",   "soft_natural",   220, "Warm Dirt");
const stoneBase     = mat("stone_base",  "clean_stylized", 931, "Smooth Stone");
const cobblestone   = mat("cobblestone", "rich_organic",   612, "Rounded Cobblestone");
const sandBase      = mat("sand_base",   "rich_organic",   404, "Sun-Warmed Sand");
const snowTop       = mat("snow_top",    "clean_stylized", 909, "Fluffy Snow");
const oakRings      = mat("wood_rings",  "rich_organic",   773, "Oak Log Rings");
const oakPlanks     = mat("planks",      "soft_natural",   774, "Oak Planks");
const oakLeaves     = mat("leaves",      "rich_organic",   555, "Lush Oak Leaves");
const mossCarpet    = mat("moss",        "rich_organic",   556, "Mossy Carpet");
const redBrick      = mat("brick",       "clean_stylized", 360, "Terracotta Brick");
const ironOre       = mat("ore_iron",    "rich_organic",   888, "Rusty Iron Ore");
const goldOre       = mat("ore_gold",    "rich_organic",   889, "Glittering Gold Ore");
const cyanCrystal   = mat("crystal",     "clean_stylized", 1212, "Cyan Crystal");
const moltenLava    = mat("lava",        "rich_organic",   666, "Molten Lava");
const magmaCrust    = mat("magma",       "rich_organic",   667, "Magma Crust");
const frostedIce    = mat("ice",         "clean_stylized", 1313, "Frosted Ice");

const materials: MaterialFaceDef[] = [
  grassTop, grassSide, dirtBase, stoneBase, cobblestone, sandBase, snowTop,
  oakRings, oakPlanks, oakLeaves, mossCarpet, redBrick,
  ironOre, goldOre, cyanCrystal, moltenLava, magmaCrust, frostedIce,
];

// ---------------------------------------------------------------------------
// Demo block library — at least 10 blocks that exercise face combinations,
// transparency, glow, hardness, and gameplay variety.
// ---------------------------------------------------------------------------

type BlockPatch = {
  id?: string;
  displayName?: string;
  seed?: number;
  category?: string;
  tags?: string[];
  gameplay?: Partial<BlockDef["gameplay"]>;
  render?: Partial<BlockDef["render"]>;
};

function tweak(block: BlockDef, patch: BlockPatch) {
  if (patch.gameplay) Object.assign(block.gameplay, patch.gameplay);
  if (patch.render) Object.assign(block.render, patch.render);
  if (patch.id !== undefined) block.id = patch.id;
  if (patch.displayName !== undefined) block.displayName = patch.displayName;
  if (patch.seed !== undefined) block.seed = patch.seed;
  if (patch.category !== undefined) block.category = patch.category;
  if (patch.tags !== undefined) block.tags = patch.tags;
  return block;
}

const grassBlock = tweak(
  createBlockFromPreset("cube", { top: grassTop.id, side: grassSide.id, bottom: dirtBase.id }),
  { id: "core:grass", displayName: "Grass Block", seed: 1001, category: "Terrain", tags: ["terrain", "natural", "grass"],
    gameplay: { drops: ["core:dirt"] } },
);

const dirtBlock = tweak(
  createBlockFromPreset("cube", { all: dirtBase.id }),
  { id: "core:dirt", displayName: "Dirt", seed: 1002, category: "Terrain", tags: ["terrain", "soil"],
    gameplay: { hardness: 0.3, breakSpeedPreset: "soft", drops: ["core:dirt"] } },
);

const stoneBlock = tweak(
  createBlockFromPreset("cube", { all: stoneBase.id }),
  { id: "core:stone", displayName: "Stone", seed: 1003, category: "Stone", tags: ["terrain", "rock"],
    gameplay: { hardness: 1.5, breakSpeedPreset: "hard", drops: ["core:cobblestone"] } },
);

const cobbleBlock = tweak(
  createBlockFromPreset("cube", { all: cobblestone.id }),
  { id: "core:cobblestone", displayName: "Cobblestone", seed: 1004, category: "Stone", tags: ["terrain", "rock", "rounded"],
    gameplay: { hardness: 1.7, breakSpeedPreset: "hard", drops: ["core:cobblestone"] } },
);

const sandBlock = tweak(
  createBlockFromPreset("cube", { all: sandBase.id }),
  { id: "core:sand", displayName: "Desert Sand", seed: 1005, category: "Terrain", tags: ["terrain", "desert", "loose"],
    gameplay: { hardness: 0.4, breakSpeedPreset: "soft", drops: ["core:sand"] } },
);

const snowBlock = tweak(
  createBlockFromPreset("cube", { all: snowTop.id }),
  { id: "core:snow", displayName: "Snow Block", seed: 1006, category: "Terrain", tags: ["terrain", "cold", "snow"],
    gameplay: { hardness: 0.2, breakSpeedPreset: "soft", drops: ["core:snow"] } },
);

const oakLogBlock = tweak(
  createBlockFromPreset("cube", { top: oakPlanks.id, side: oakRings.id, bottom: oakPlanks.id }),
  { id: "core:oak_log", displayName: "Oak Log", seed: 1007, category: "Wood", tags: ["wood", "tree"],
    gameplay: { hardness: 1.0, breakSpeedPreset: "normal", drops: ["core:oak_log"] } },
);

const oakPlanksBlock = tweak(
  createBlockFromPreset("cube", { all: oakPlanks.id }),
  { id: "core:oak_planks", displayName: "Oak Planks", seed: 1008, category: "Wood", tags: ["wood", "crafted"],
    gameplay: { hardness: 1.0, breakSpeedPreset: "normal", drops: ["core:oak_planks"] } },
);

const oakLeavesBlock = tweak(
  createBlockFromPreset("cube", { all: oakLeaves.id }),
  { id: "core:oak_leaves", displayName: "Oak Leaves", seed: 1009, category: "Vegetation", tags: ["leaves", "natural"],
    gameplay: { hardness: 0.2, breakSpeedPreset: "soft", drops: [] },
    render: { transparent: true, cullFaces: false, ambientOcclusion: true } },
);

const mossyStoneBlock = tweak(
  createBlockFromPreset("cube", { top: mossCarpet.id, side: mossCarpet.id, bottom: stoneBase.id }),
  { id: "core:mossy_stone", displayName: "Mossy Stone", seed: 1010, category: "Stone", tags: ["terrain", "rock", "moss"],
    gameplay: { hardness: 1.4, breakSpeedPreset: "hard", drops: ["core:mossy_stone"] } },
);

const brickBlock = tweak(
  createBlockFromPreset("cube", { all: redBrick.id }),
  { id: "core:bricks", displayName: "Red Bricks", seed: 1011, category: "Building", tags: ["crafted", "masonry"],
    gameplay: { hardness: 2.0, breakSpeedPreset: "hard", drops: ["core:bricks"] } },
);

const ironOreBlock = tweak(
  createBlockFromPreset("cube", { all: ironOre.id }),
  { id: "core:iron_ore", displayName: "Iron Ore", seed: 1012, category: "Ore", tags: ["ore", "rock", "metal"],
    gameplay: { hardness: 3.0, breakSpeedPreset: "very-hard", drops: ["core:iron_ore"] } },
);

const goldOreBlock = tweak(
  createBlockFromPreset("cube", { all: goldOre.id }),
  { id: "core:gold_ore", displayName: "Gold Ore", seed: 1013, category: "Ore", tags: ["ore", "rock", "metal", "precious"],
    gameplay: { hardness: 3.0, breakSpeedPreset: "very-hard", drops: ["core:gold_ore"] } },
);

const crystalBlock = tweak(
  createBlockFromPreset("cube", { all: cyanCrystal.id }),
  { id: "core:crystal", displayName: "Cyan Crystal", seed: 1014, category: "Magic", tags: ["crystal", "glowing", "magic"],
    gameplay: { hardness: 2.2, breakSpeedPreset: "hard", drops: ["core:crystal"] },
    render: { transparent: true, cullFaces: true, ambientOcclusion: false, lightEmission: 9 } },
);

const lavaBlock = tweak(
  createBlockFromPreset("liquid", { all: moltenLava.id }),
  { id: "core:lava", displayName: "Lava", seed: 1015, category: "Liquid", tags: ["liquid", "hot", "danger"],
    render: { lightEmission: 14 } },
);

const magmaBlock = tweak(
  createBlockFromPreset("cube", { all: magmaCrust.id }),
  { id: "core:magma_block", displayName: "Magma Block", seed: 1016, category: "Terrain", tags: ["hot", "danger", "rock"],
    gameplay: { hardness: 1.0, breakSpeedPreset: "normal", drops: ["core:magma_block"] },
    render: { lightEmission: 6 } },
);

const iceBlock = tweak(
  createBlockFromPreset("cube", { all: frostedIce.id }),
  { id: "core:ice", displayName: "Frosted Ice", seed: 1017, category: "Terrain", tags: ["cold", "slippery", "transparent"],
    gameplay: { hardness: 0.5, breakSpeedPreset: "soft", drops: [] },
    render: { transparent: true, cullFaces: true, ambientOcclusion: false } },
);

const blocks: BlockDef[] = [
  grassBlock, dirtBlock, stoneBlock, cobbleBlock, sandBlock, snowBlock,
  oakLogBlock, oakPlanksBlock, oakLeavesBlock, mossyStoneBlock,
  brickBlock, ironOreBlock, goldOreBlock, crystalBlock,
  lavaBlock, magmaBlock, iceBlock,
];

const project: PackProject = {
  schemaVersion: 5,
  id: "my-first-pack",
  namespace: "core",
  name: "My First Pack",
  path: "assets/packs/core",
  packSeed: 42,
  seedPolicy: {
    packSeed: 42,
    previewPositionSeed: 0,
  },
  materials,
  blocks,
  validationIssues: [],
  hasUnsavedChanges: false,
};

project.validationIssues = validatePack(project);

export const initialProject = project;
