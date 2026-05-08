import type { BlockDef, BlockKind, BlockTemplate, FaceMaterialRefs } from "../../types/studio";
import { randomSeed } from "../procedural/seed";

export const blockTemplates: BlockTemplate[] = [
  { kind: "cube", label: "Full Block", description: "Classic solid cube with material faces." },
  { kind: "slab", label: "Slab", description: "Half-height block, same material system." },
  { kind: "stairs", label: "Stairs", description: "Step block prepared for custom geometry." },
  { kind: "cross_plant", label: "Plant", description: "Cross-plane vegetation block." },
  { kind: "liquid", label: "Liquid", description: "Transparent fluid block." },
  { kind: "custom", label: "Custom", description: "Full RON control for special blocks." },
];

export function createBlockFromPreset(kind: BlockKind, materials: FaceMaterialRefs, namespace = "core"): BlockDef {
  const name = kind === "cross_plant" ? "Plant Block" : kind === "liquid" ? "Water Block" : kind === "slab" ? "Slab" : "Grass Block";
  const id = kind === "cross_plant" ? "plant" : kind === "liquid" ? "water" : kind === "slab" ? "slab" : "grass";
  const walkThrough = kind === "cross_plant" || kind === "liquid";

  return {
    id: `${namespace}:${id}`,
    displayName: name,
    seed: randomSeed(),
    geometry: {
      kind,
      collisionShape: kind === "cube" ? "solid_cube" : kind === "cross_plant" ? "cross" : kind === "liquid" ? "fluid" : "partial",
    },
    render: {
      materials,
      ambientOcclusion: kind !== "liquid",
      transparent: kind === "cross_plant" || kind === "liquid",
      cullFaces: kind !== "cross_plant",
      lightEmission: 0,
    },
    gameplay: {
      walkThrough,
      hardness: kind === "liquid" ? 100 : kind === "cross_plant" ? 0.1 : 0.6,
      breakSpeedPreset: kind === "cross_plant" ? "soft" : "normal",
      drops: kind === "liquid" ? [] : [`${namespace}:${id === "grass" ? "dirt" : id}`],
    },
    category: kind === "cross_plant" ? "Vegetation" : kind === "liquid" ? "Liquid" : "Terrain",
    tags: kind === "cross_plant" ? ["plant", "natural"] : kind === "liquid" ? ["liquid"] : ["terrain", "natural"],
    status: "valid",
  };
}
