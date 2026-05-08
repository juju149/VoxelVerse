import type { PackProject } from "../types/studio";
import { createBlockFromPreset } from "../lib/presets/blockPresets";
import { createMaterialFromPreset } from "../lib/presets/materialPresets";
import { validatePack } from "../lib/validation/packValidator";

const grassTop = createMaterialFromPreset("grass_top", "soft_natural");
grassTop.seed = 149;
const grassSide = createMaterialFromPreset("grass_side", "soft_natural");
grassSide.seed = 251;
const dirtBase = createMaterialFromPreset("dirt_base", "soft_natural");
dirtBase.seed = 220;
const stoneBase = createMaterialFromPreset("stone_base", "clean_stylized");
stoneBase.seed = 931;

const grassBlock = createBlockFromPreset("cube", {
  top: "core:grass/top",
  side: "core:grass/side",
  bottom: "core:dirt/base",
});
grassBlock.seed = 1001;

const dirtBlock = createBlockFromPreset("cube", {
  all: "core:dirt/base",
});
dirtBlock.id = "core:dirt";
dirtBlock.displayName = "Dirt";
dirtBlock.seed = 1002;
dirtBlock.gameplay.hardness = 0.3;
dirtBlock.gameplay.breakSpeedPreset = "soft";
dirtBlock.gameplay.drops = ["core:dirt"];

const stoneBlock = createBlockFromPreset("cube", {
  all: "core:stone/base",
});
stoneBlock.id = "core:stone";
stoneBlock.displayName = "Stone";
stoneBlock.seed = 1003;
stoneBlock.gameplay.hardness = 1.5;
stoneBlock.gameplay.breakSpeedPreset = "hard";
stoneBlock.gameplay.drops = ["core:stone"];
stoneBlock.category = "Stone";
stoneBlock.tags = ["terrain", "rock"];

const project: PackProject = {
  schemaVersion: 4,
  id: "my-first-pack",
  namespace: "core",
  name: "My First Pack",
  path: "assets/packs/core",
  packSeed: 42,
  seedPolicy: {
    packSeed: 42,
    previewPositionSeed: 0,
  },
  materials: [grassTop, grassSide, dirtBase, stoneBase],
  blocks: [grassBlock, dirtBlock, stoneBlock],
  validationIssues: [],
  hasUnsavedChanges: false,
};

project.validationIssues = validatePack(project);

export const initialProject = project;
