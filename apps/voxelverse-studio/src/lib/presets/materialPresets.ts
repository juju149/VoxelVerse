import type {
  MaterialFaceDef,
  MaterialPatternLayer,
  MaterialStylePreset,
  MaterialTemplate,
  ProceduralMaterialRecipe,
} from "../../types/studio";
import { createBlueprintFromRecipe } from "../blueprint/materialBlueprint";
import { randomSeed } from "../procedural/seed";

// MaterialKind is an internal implementation detail of the template system.
// It is NOT exported — external code uses category: string on MaterialFaceDef.
type MaterialKind =
  | "grass_top" | "grass_side" | "dirt_base" | "stone_base" | "sand_base"
  | "wood_rings" | "snow_top" | "leaves" | "brick" | "lava" | "crystal"
  | "moss" | "ore_iron" | "ore_gold" | "ice" | "planks" | "magma"
  | "cobblestone" | "custom";

const KIND_CATEGORY: Record<MaterialKind, string> = {
  grass_top: "terrain", grass_side: "terrain", dirt_base: "terrain",
  sand_base: "terrain", snow_top: "terrain",
  stone_base: "stone", cobblestone: "stone",
  wood_rings: "wood", planks: "wood", leaves: "natural", moss: "natural",
  brick: "building",
  ore_iron: "ore", ore_gold: "ore",
  crystal: "special", ice: "special",
  lava: "liquid", magma: "liquid",
  custom: "custom",
};

/** Built-in material templates shown in the Template Gallery. */
export const materialTemplates: MaterialTemplate[] = [
  { templateKey: "grass_top",   label: "Grass Top",    description: "Soft cartoon green with broad clean patches.",   category: "terrain" },
  { templateKey: "grass_side",  label: "Grass Side",   description: "Dirt base with an irregular grass edge band.",   category: "terrain" },
  { templateKey: "dirt_base",   label: "Dirt",         description: "Warm soil with rounded pebbles and soft blotches.", category: "terrain" },
  { templateKey: "stone_base",  label: "Stone",        description: "Clean grey rock with optional fine cracks.",      category: "stone" },
  { templateKey: "cobblestone", label: "Cobblestone",  description: "Rounded stone cells, deep grout shadows.",        category: "stone" },
  { templateKey: "sand_base",   label: "Sand",         description: "Warm desert grains with gentle dunes.",           category: "terrain" },
  { templateKey: "snow_top",    label: "Snow",         description: "Bright fluffy snow with soft sparkle dots.",      category: "terrain" },
  { templateKey: "wood_rings",  label: "Wood Rings",   description: "Stylized log rings, rich grain stripes.",         category: "wood" },
  { templateKey: "planks",      label: "Planks",       description: "Tidy wood planks with horizontal grain bands.",   category: "wood" },
  { templateKey: "leaves",      label: "Leaves",       description: "Lush stylized canopy clusters.",                  category: "natural" },
  { templateKey: "moss",        label: "Moss",         description: "Velvety mossy carpet, organic green cells.",      category: "natural" },
  { templateKey: "brick",       label: "Brick",        description: "Warm terracotta bricks with deep grout.",         category: "building" },
  { templateKey: "ore_iron",    label: "Iron Ore",     description: "Stone matrix with rusty ore veins.",              category: "ore" },
  { templateKey: "ore_gold",    label: "Gold Ore",     description: "Stone matrix with bright golden flecks.",         category: "ore" },
  { templateKey: "crystal",     label: "Crystal",      description: "Translucent cyan facets, glowing accents.",       category: "special" },
  { templateKey: "lava",        label: "Lava",         description: "Molten orange flow with dark crust cracks.",      category: "liquid" },
  { templateKey: "magma",       label: "Magma",        description: "Dark crust pulsing with red-hot fissures.",       category: "liquid" },
  { templateKey: "ice",         label: "Ice",          description: "Frosted blue ice with subtle internal cracks.",   category: "special" },
];

// Kept for internal use by the template system (not exported to UI).
const materialStyles: { id: MaterialStylePreset; label: string }[] = [
  { id: "soft_natural", label: "Soft Natural" },
  { id: "clean_stylized", label: "Clean Stylized" },
  { id: "rich_organic", label: "Rich Organic" },
  { id: "simple_flat", label: "Simple Flat" },
];
void materialStyles; // suppress unused warning

type Palette = [string, string, string]; // base, low (shadow), high (highlight)

const PALETTES: Record<MaterialKind, Palette> = {
  grass_top:    ["#7BAA32", "#5F8D29", "#9ACB4E"],
  grass_side:   ["#6FA236", "#6A492C", "#8FBC47"],
  dirt_base:    ["#7B5635", "#5E3E27", "#9C7048"],
  stone_base:   ["#8A8D8F", "#64686B", "#B1B3B4"],
  cobblestone:  ["#7E8285", "#454A4E", "#A6AAAD"],
  sand_base:    ["#E5C679", "#B89150", "#F4DCA0"],
  snow_top:     ["#F1F6FB", "#C8D6E3", "#FFFFFF"],
  wood_rings:   ["#A97842", "#6D4528", "#C69656"],
  planks:       ["#B07A41", "#7B4C26", "#D49A60"],
  leaves:       ["#4F9A3A", "#2E6324", "#7BC152"],
  moss:         ["#5C8E3A", "#345622", "#8FBE5C"],
  brick:        ["#B25640", "#7A2F22", "#D27A60"],
  ore_iron:     ["#9B9CA0", "#5E5F62", "#C9A077"],
  ore_gold:     ["#90928F", "#5C5E5C", "#F2C84B"],
  crystal:      ["#74D9E8", "#2E7C9A", "#C5F5FB"],
  lava:         ["#FF7A1A", "#7A1F00", "#FFE082"],
  magma:        ["#3A1F18", "#0E0703", "#FF5A1F"],
  ice:          ["#B6E4F4", "#5C9BB8", "#E7F7FE"],
  custom:       ["#8A8D8F", "#64686B", "#B1B3B4"],
};

const PATHS: Record<MaterialKind, string> = {
  grass_top: "grass/top",
  grass_side: "grass/side",
  dirt_base: "dirt/base",
  stone_base: "stone/base",
  cobblestone: "cobblestone/base",
  sand_base: "sand/base",
  snow_top: "snow/top",
  wood_rings: "wood/oak_rings",
  planks: "wood/oak_planks",
  leaves: "leaves/oak",
  moss: "moss/carpet",
  brick: "brick/red",
  ore_iron: "ore/iron",
  ore_gold: "ore/gold",
  crystal: "crystal/cyan",
  lava: "lava/flow",
  magma: "magma/crust",
  ice: "ice/frosted",
  custom: "custom/material",
};

function defaultColor(kind: MaterialKind, role: "base" | "low" | "high") {
  return PALETTES[kind][role === "base" ? 0 : role === "low" ? 1 : 2];
}

function displayName(kind: MaterialKind) {
  return materialTemplates.find((t) => t.templateKey === kind)?.label ?? "Material";
}

function pathName(kind: MaterialKind) {
  return PATHS[kind];
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
  softness = 0.42,
  warp = 0.12,
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
    softness,
    warp,
    offsetX: 0,
    offsetY: 0,
    color,
    threshold,
    enabled: true,
  };
}

function layersFor(kind: MaterialKind, detail: number, patchScale: number, contrast: number): MaterialPatternLayer[] {
  switch (kind) {
    case "grass_top":
      return [
        layer("broad_patches", "soft_blotches", "overlay", detail, patchScale, contrast, "#9ACB4E"),
        layer("micro_blades", "soft_noise", "highlight", detail * 0.5, 22, 0.14, "#B6DC6E"),
        layer("shadow_pools", "organic_cells", "shadow", detail * 0.45, 9, 0.18, "#5F8D29", undefined, "edge_wear"),
      ];
    case "grass_side":
      return [
        layer("top_color_band", "edge_band", "mix", 1, 1, 0.34, "#6FA236", 0.28, "top_band"),
        layer("dirt_blotches", "soft_blotches", "overlay", detail, patchScale, contrast, "#8A6240"),
        layer("pebbles", "rounded_pebbles", "mix", detail * 0.75, 10, 0.28, "#A27952"),
      ];
    case "dirt_base":
      return [
        layer("blotches", "soft_blotches", "overlay", detail, patchScale, contrast, "#9C7048"),
        layer("pebbles", "rounded_pebbles", "mix", detail * 0.85, 11, 0.26, "#B08359"),
        layer("micro_grain", "soft_noise", "shadow", 0.08, 24, 0.1, "#5E3E27"),
      ];
    case "stone_base":
      return [
        layer("patches", "patch_cells", "overlay", detail, patchScale, contrast, "#B1B3B4"),
        layer("cracks", "cracks", "shadow", detail * 0.55, 18, 0.16, "#55595C", 0.82),
        layer("micro_specks", "soft_noise", "highlight", 0.06, 28, 0.08, "#D5D7D8"),
      ];
    case "cobblestone":
      return [
        layer("cobbles", "rounded_pebbles", "overlay", 0.9, 5, 0.42, "#A6AAAD", undefined, "none", "warped_uv", 0.3, 0.06),
        layer("grout", "cracks", "shadow", 0.7, 6, 0.32, "#2C2F32", 0.55),
        layer("specks", "soft_noise", "highlight", 0.08, 30, 0.1, "#E1E3E5"),
      ];
    case "sand_base":
      return [
        layer("dunes", "soft_blotches", "overlay", detail * 0.6, 4, 0.12, "#F4DCA0"),
        layer("grain", "soft_noise", "highlight", 0.18, 38, 0.1, "#FFEFC2"),
        layer("warm_lows", "organic_cells", "shadow", 0.1, 7, 0.14, "#B89150"),
      ];
    case "snow_top":
      return [
        layer("drifts", "soft_blotches", "overlay", 0.18, 5, 0.08, "#FFFFFF"),
        layer("sparkle", "dots", "highlight", 0.12, 60, 0.6, "#FFFFFF", 0.86),
        layer("cool_shadow", "soft_noise", "shadow", 0.08, 10, 0.12, "#C8D6E3"),
      ];
    case "wood_rings":
      return [
        layer("rings", "rings", "overlay", detail, 7, contrast, "#C69656", undefined, "center_soft", "radial"),
        layer("grain", "stripes", "shadow", detail * 0.6, 14, 0.16, "#6D4528", undefined, "none", "horizontal"),
        layer("knot", "dots", "shadow", 0.16, 3, 0.5, "#5A3520", 0.88, "center_soft"),
      ];
    case "planks":
      return [
        layer("plank_bands", "bands", "overlay", 0.9, 6, 0.32, "#7B4C26", 0.5, "none", "horizontal", 0.18, 0.02),
        layer("grain", "stripes", "shadow", 0.32, 22, 0.18, "#5A3520", undefined, "none", "horizontal"),
        layer("highlights", "soft_noise", "highlight", 0.12, 30, 0.1, "#D49A60"),
      ];
    case "leaves":
      return [
        layer("clusters", "organic_cells", "overlay", 0.7, 8, 0.42, "#7BC152"),
        layer("dark_pockets", "organic_cells", "shadow", 0.5, 6, 0.36, "#2E6324", undefined, "edge_wear"),
        layer("specks", "dots", "highlight", 0.18, 28, 0.5, "#C0E78A", 0.82),
      ];
    case "moss":
      return [
        layer("velvet", "soft_blotches", "overlay", 0.6, 6, 0.3, "#8FBE5C"),
        layer("dark_pockets", "organic_cells", "shadow", 0.45, 9, 0.32, "#345622"),
        layer("micro_fuzz", "soft_noise", "highlight", 0.18, 38, 0.12, "#B7DC7A"),
      ];
    case "brick":
      return [
        layer("brick_bands", "bands", "overlay", 1, 4, 0.32, "#D27A60", 0.5, "none", "horizontal", 0.05, 0),
        layer("brick_offset", "stripes", "shadow", 0.9, 8, 0.3, "#3D1A12", 0.5, "none", "vertical", 0.05, 0),
        layer("face_blotches", "soft_blotches", "overlay", 0.28, 9, 0.18, "#9E4434"),
        layer("wear", "soft_noise", "highlight", 0.1, 22, 0.1, "#E89B82"),
      ];
    case "ore_iron":
      return [
        layer("matrix_patches", "patch_cells", "overlay", 0.4, 7, 0.22, "#B1B3B4"),
        layer("rust_veins", "organic_cells", "highlight", 0.55, 5, 0.42, "#C9A077", 0.62),
        layer("dark_pits", "dots", "shadow", 0.28, 14, 0.4, "#2E2E30", 0.86),
      ];
    case "ore_gold":
      return [
        layer("matrix_patches", "patch_cells", "overlay", 0.35, 7, 0.22, "#A6AAAD"),
        layer("gold_flecks", "dots", "highlight", 0.55, 18, 0.6, "#F2C84B", 0.78),
        layer("dark_cracks", "cracks", "shadow", 0.3, 16, 0.18, "#3A3B3D"),
      ];
    case "crystal":
      return [
        layer("facets", "patch_cells", "overlay", 0.7, 5, 0.45, "#C5F5FB"),
        layer("inner_glow", "soft_blotches", "highlight", 0.45, 4, 0.3, "#B5EFFB", undefined, "center_soft"),
        layer("dark_seams", "cracks", "shadow", 0.3, 9, 0.22, "#1F4F66"),
        layer("sparkle", "dots", "highlight", 0.18, 40, 0.7, "#FFFFFF", 0.9),
      ];
    case "lava":
      return [
        layer("flow_blotches", "soft_blotches", "overlay", 0.7, 5, 0.45, "#FFE082"),
        layer("hot_veins", "cracks", "highlight", 0.6, 7, 0.4, "#FFB347", 0.55),
        layer("crust", "organic_cells", "shadow", 0.45, 6, 0.36, "#7A1F00", undefined, "edge_wear"),
        layer("embers", "dots", "highlight", 0.22, 28, 0.6, "#FFF6C2", 0.88),
      ];
    case "magma":
      return [
        layer("crust_blotches", "soft_blotches", "overlay", 0.6, 6, 0.42, "#1F0E08"),
        layer("hot_cracks", "cracks", "highlight", 0.85, 6, 0.55, "#FF5A1F", 0.45),
        layer("ember_dots", "dots", "highlight", 0.3, 22, 0.65, "#FFC857", 0.86),
      ];
    case "ice":
      return [
        layer("frost_patches", "soft_blotches", "overlay", 0.4, 5, 0.18, "#E7F7FE"),
        layer("inner_cracks", "cracks", "shadow", 0.4, 11, 0.22, "#5C9BB8", 0.58),
        layer("sparkle", "dots", "highlight", 0.18, 50, 0.7, "#FFFFFF", 0.9),
      ];
    case "custom":
      return [layer("soft_noise", "soft_noise", "overlay", detail, patchScale, contrast, defaultColor(kind, "high"))];
  }
}

function surfaceFor(kind: MaterialKind, flat: boolean) {
  const base = {
    roughness: 0.78,
    heightStrength: flat ? 0.04 : 0.14,
    normalStrength: flat ? 0.04 : 0.22,
    edgeSoftness: 0.42,
  };
  switch (kind) {
    case "stone_base":
    case "cobblestone":
    case "brick":
      return { ...base, roughness: 0.88, heightStrength: flat ? 0.06 : 0.28, normalStrength: flat ? 0.06 : 0.34 };
    case "grass_top":
    case "leaves":
    case "moss":
      return { ...base, roughness: 0.74, heightStrength: flat ? 0.04 : 0.12 };
    case "sand_base":
      return { ...base, roughness: 0.92, heightStrength: flat ? 0.04 : 0.1 };
    case "snow_top":
      return { ...base, roughness: 0.6, heightStrength: flat ? 0.04 : 0.1 };
    case "ice":
    case "crystal":
      return { ...base, roughness: 0.18, heightStrength: 0.06, normalStrength: 0.18, edgeSoftness: 0.22 };
    case "lava":
    case "magma":
      return { ...base, roughness: 0.5, heightStrength: flat ? 0.04 : 0.18, normalStrength: 0.24 };
    case "wood_rings":
    case "planks":
      return { ...base, roughness: 0.7, heightStrength: 0.12, normalStrength: 0.2 };
    case "ore_iron":
    case "ore_gold":
      return { ...base, roughness: 0.82, heightStrength: 0.18, normalStrength: 0.28 };
    default:
      return base;
  }
}

function patchScaleFor(kind: MaterialKind) {
  switch (kind) {
    case "stone_base": return 7;
    case "dirt_base": return 5;
    case "cobblestone": return 4;
    case "brick": return 4;
    case "leaves":
    case "moss": return 8;
    case "crystal":
    case "ice": return 5;
    case "lava":
    case "magma": return 5;
    case "snow_top": return 6;
    case "sand_base": return 4;
    default: return 6;
  }
}

export function createMaterialFromPreset(kind: MaterialKind, style: MaterialStylePreset, namespace = "core"): MaterialFaceDef {
  const rich = style === "rich_organic";
  const flat = style === "simple_flat";
  const detail = flat ? 0.04 : rich ? 0.32 : 0.18;
  const patchScale = patchScaleFor(kind);
  const contrast = flat ? 0.08 : rich ? 0.32 : 0.22;
  const isOrganic = kind === "grass_top" || kind === "grass_side" || kind === "leaves" || kind === "moss";
  const isHot = kind === "lava" || kind === "magma";
  const isCool = kind === "ice" || kind === "crystal" || kind === "snow_top";

  const recipe: ProceduralMaterialRecipe = {
    style,
    baseColor: defaultColor(kind, "base"),
    shadowColor: defaultColor(kind, "low"),
    highlightColor: defaultColor(kind, "high"),
    patternLayers: layersFor(kind, detail, patchScale, contrast),
    surface: surfaceFor(kind, flat),
    stylization: {
      colorSteps: style === "simple_flat" ? 3 : style === "rich_organic" ? 7 : 5,
      smoothing: style === "clean_stylized" ? 0.64 : 0.48,
      saturation: isOrganic ? 1.14 : isHot ? 1.22 : isCool ? 1.06 : 1,
      valueBoost: isHot ? 1.12 : 1.02,
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
      ...(kind === "lava" || kind === "magma"
        ? [{ name: "glow", kind: "Float" as const, value: kind === "magma" ? 0.55 : 0.7, min: 0, max: 1 }]
        : []),
      ...(kind === "crystal"
        ? [{ name: "facet_sharpness", kind: "Float" as const, value: 0.62, min: 0, max: 1 }]
        : []),
      ...(kind === "ore_iron" || kind === "ore_gold"
        ? [{ name: "vein_density", kind: "Float" as const, value: 0.55, min: 0, max: 1 }]
        : []),
      ...(kind === "brick"
        ? [{ name: "grout_thickness", kind: "Float" as const, value: 0.32, min: 0.05, max: 0.6 }]
        : []),
    ],
  };

  return {
    id: `${namespace}:${pathName(kind)}`,
    displayName: displayName(kind),
    category: KIND_CATEGORY[kind],
    resolutionPreview: 128,
    seed: randomSeed(),
    blueprint: createBlueprintFromRecipe(recipe),
    recipe,
    status: "valid",
    previewVersion: 0,
  };
}

/**
 * Creates a blank material with no pattern layers — the default starting
 * point when the user clicks "+ New Material".
 */
export function createEmptyMaterial(namespace = "core", displayName = "New Material"): MaterialFaceDef {
  const recipe: ProceduralMaterialRecipe = {
    style: "simple_flat",
    baseColor: "#888888",
    shadowColor: "#555555",
    highlightColor: "#AAAAAA",
    patternLayers: [],
    surface: { roughness: 0.75, heightStrength: 0.1, normalStrength: 0.2, edgeSoftness: 0.4 },
    stylization: { colorSteps: 5, smoothing: 0.48, saturation: 1.0, valueBoost: 1.0, microDetail: 0.05 },
    variation: { enabled: true, perBlockStrength: 0.15, colorJitter: 0.05, patternJitter: 0.1 },
    params: [],
  };

  const slugged = displayName
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "") || "new_material";

  return {
    id: `${namespace}:custom/${slugged}`,
    displayName,
    category: "custom",
    resolutionPreview: 128,
    seed: randomSeed(),
    blueprint: createBlueprintFromRecipe(recipe),
    recipe,
    status: "valid",
    previewVersion: 0,
  };
}

/**
 * Creates a material from a template key (the templateKey field from
 * materialTemplates). Returns null if the key is unknown.
 */
export function createMaterialFromTemplateKey(templateKey: string, namespace = "core"): MaterialFaceDef | null {
  if (templateKey in KIND_CATEGORY) {
    return createMaterialFromPreset(templateKey as MaterialKind, "soft_natural", namespace);
  }
  return null;
}
