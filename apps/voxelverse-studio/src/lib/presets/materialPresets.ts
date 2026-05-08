import type {
  MaterialFaceDef,
  MaterialKind,
  MaterialPatternLayer,
  MaterialPresetChoice,
  MaterialStylePreset,
  ProceduralMaterialRecipe,
} from "../../types/studio";
import { createBlueprintFromRecipe } from "../blueprint/materialBlueprint";
import { randomSeed } from "../procedural/seed";

export const materialPresetChoices: MaterialPresetChoice[] = [
  { kind: "grass_top", label: "Grass Top", description: "Soft green top with broad clean patches." },
  { kind: "grass_side", label: "Grass Side", description: "Dirt base with an irregular grass edge." },
  { kind: "dirt_base", label: "Dirt", description: "Warm soil with soft blotches and rounded pebbles." },
  { kind: "stone_base", label: "Stone", description: "Clean grey rock patches with optional cracks." },
  { kind: "sand_base", label: "Sand", description: "Warm simple grains for beaches and deserts." },
  { kind: "wood_rings", label: "Wood", description: "Stylized rings and stripes for logs." },
  { kind: "custom", label: "Custom", description: "Blank procedural material recipe." },
];

export const materialStyles: { id: MaterialStylePreset; label: string }[] = [
  { id: "soft_natural", label: "Soft Natural" },
  { id: "clean_stylized", label: "Clean Stylized" },
  { id: "rich_organic", label: "Rich Organic" },
  { id: "simple_flat", label: "Simple Flat" },
];

function defaultColor(kind: MaterialKind, role: "base" | "low" | "high") {
  const colors: Record<MaterialKind, [string, string, string]> = {
    grass_top: ["#7BAA32", "#5F8D29", "#9ACB4E"],
    grass_side: ["#6FA236", "#6A492C", "#8FBC47"],
    dirt_base: ["#7B5635", "#5E3E27", "#9C7048"],
    stone_base: ["#8A8D8F", "#64686B", "#B1B3B4"],
    sand_base: ["#CDAE68", "#AA884D", "#DFC783"],
    wood_rings: ["#A97842", "#6D4528", "#C69656"],
    custom: ["#8A8D8F", "#64686B", "#B1B3B4"],
  };
  return colors[kind][role === "base" ? 0 : role === "low" ? 1 : 2];
}

function displayName(kind: MaterialKind) {
  return materialPresetChoices.find((choice) => choice.kind === kind)?.label ?? "Material";
}

function pathName(kind: MaterialKind) {
  const paths: Record<MaterialKind, string> = {
    grass_top: "grass/top",
    grass_side: "grass/side",
    dirt_base: "dirt/base",
    stone_base: "stone/base",
    sand_base: "sand/base",
    wood_rings: "wood/rings",
    custom: "custom/material",
  };
  return paths[kind];
}

function layersFor(kind: MaterialKind, detail: number, patchScale: number, contrast: number): MaterialPatternLayer[] {
  if (kind === "grass_side") {
    return [
      layer("top_color_band", "edge_band", "mix", 1, 1, 0.34, "#6FA236", 0.28, "top_band"),
      layer("large_soft_shapes", "soft_blotches", "overlay", detail, patchScale, contrast, "#8A6240"),
      layer("rounded_spots", "rounded_pebbles", "mix", detail * 0.75, 10, 0.28, "#A27952"),
    ];
  }
  if (kind === "dirt_base") {
    return [
      layer("large_soft_shapes", "soft_blotches", "overlay", detail, patchScale, contrast, "#9C7048"),
      layer("rounded_spots", "rounded_pebbles", "mix", detail * 0.85, 11, 0.26, "#B08359"),
    ];
  }
  if (kind === "stone_base") {
    return [
      layer("large_cell_patches", "patch_cells", "overlay", detail, patchScale, contrast, "#B1B3B4"),
      layer("subtle_cracks", "cracks", "shadow", detail * 0.55, 18, 0.16, "#55595C", 0.82),
    ];
  }
  if (kind === "wood_rings") {
    return [
      layer("rings", "rings", "overlay", detail, 7, contrast, "#C69656", undefined, "center_soft", "radial"),
      layer("grain", "stripes", "shadow", detail * 0.6, 14, 0.16, "#6D4528", undefined, "none", "horizontal"),
    ];
  }
  if (kind === "sand_base") {
    return [
      layer("soft_grain", "soft_noise", "highlight", detail * 0.6, 12, 0.12, "#DFC783"),
      layer("dune_blotches", "soft_blotches", "overlay", detail * 0.4, 5, 0.1, "#AA884D"),
    ];
  }
  if (kind === "custom") {
    return [layer("soft_noise", "soft_noise", "overlay", detail, patchScale, contrast, defaultColor(kind, "high"))];
  }
  return [
    layer("broad_patches", "soft_blotches", "overlay", detail, patchScale, contrast, defaultColor(kind, "high")),
    layer("soft_noise", "soft_noise", "shadow", detail * 0.35, 18, 0.1, defaultColor(kind, "low")),
  ];
}

function layer(
  id: string,
  kind: MaterialPatternLayer["kind"],
  blend: MaterialPatternLayer["blend"],
  strength: number,
  scale: number,
  contrast: number,
  color: string,
  threshold?: number,
  mask: MaterialPatternLayer["mask"] = "none",
  domain: MaterialPatternLayer["domain"] = "warped_uv",
): MaterialPatternLayer {
  return {
    id,
    kind,
    blend,
    domain,
    mask,
    strength,
    scale,
    contrast,
    softness: 0.42,
    warp: 0.12,
    offsetX: 0,
    offsetY: 0,
    color,
    threshold,
    enabled: true,
  };
}

export function createMaterialFromPreset(kind: MaterialKind, style: MaterialStylePreset, namespace = "core"): MaterialFaceDef {
  const rich = style === "rich_organic";
  const flat = style === "simple_flat";
  const detail = flat ? 0.04 : rich ? 0.32 : 0.18;
  const patchScale = kind === "stone_base" ? 7 : kind === "dirt_base" ? 5 : 6;
  const contrast = flat ? 0.08 : rich ? 0.32 : 0.22;

  const recipe: ProceduralMaterialRecipe = {
    style,
    baseColor: defaultColor(kind, "base"),
    shadowColor: defaultColor(kind, "low"),
    highlightColor: defaultColor(kind, "high"),
    patternLayers: layersFor(kind, detail, patchScale, contrast),
    surface: {
      roughness: kind === "stone_base" ? 0.82 : kind === "grass_top" ? 0.74 : 0.78,
      heightStrength: flat ? 0.04 : kind === "stone_base" ? 0.2 : 0.14,
      normalStrength: flat ? 0.04 : 0.22,
      edgeSoftness: style === "clean_stylized" ? 0.24 : 0.42,
    },
    stylization: {
      colorSteps: style === "simple_flat" ? 3 : style === "rich_organic" ? 7 : 5,
      smoothing: style === "clean_stylized" ? 0.64 : 0.48,
      saturation: kind === "grass_top" || kind === "grass_side" ? 1.12 : 1,
      valueBoost: 1.02,
      microDetail: flat ? 0.02 : 0.08,
    },
    variation: {
      enabled: true,
      perBlockStrength: flat ? 0.04 : 0.18,
      colorJitter: flat ? 0.02 : 0.08,
      patternJitter: flat ? 0.03 : 0.12,
    },
    params: [
      { name: "detail", kind: "Float", value: detail, min: 0, max: 1 },
      { name: "patch_scale", kind: "Float", value: patchScale, min: 1, max: 20 },
      { name: "contrast", kind: "Float", value: contrast, min: 0, max: 1 },
      ...(kind === "grass_side"
        ? [{ name: "top_band_height", kind: "Float" as const, value: 0.28, min: 0.05, max: 0.65 }]
        : []),
    ],
  };

  return {
    id: `${namespace}:${pathName(kind)}`,
    displayName: displayName(kind),
    materialKind: kind,
    resolutionPreview: 128,
    seed: randomSeed(),
    blueprint: createBlueprintFromRecipe(recipe),
    recipe,
    status: "valid",
    previewVersion: 0,
  };
}
