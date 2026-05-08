import type {
  MaterialBlueprint,
  MaterialBlueprintNode,
  MaterialBlueprintNodeKind,
  MaterialPatternKind,
  MaterialPatternLayer,
  MaterialStylePreset,
  ProceduralMaterialRecipe,
} from "../../types/studio";

const outputNode: MaterialBlueprintNode = {
  id: "output",
  kind: "output",
  label: "Output Material",
  position: { x: 760, y: 180 },
  params: {},
};

export function createBlueprintFromRecipe(recipe: ProceduralMaterialRecipe): MaterialBlueprint {
  const nodes: MaterialBlueprintNode[] = [
    {
      id: "palette",
      kind: "palette",
      label: "Palette",
      position: { x: 40, y: 80 },
      params: {
        baseColor: recipe.baseColor,
        shadowColor: recipe.shadowColor,
        highlightColor: recipe.highlightColor,
      },
    },
    {
      id: "stylization",
      kind: "stylization",
      label: "Stylization",
      position: { x: 420, y: 40 },
      params: { ...recipe.stylization },
    },
    {
      id: "surface",
      kind: "surface",
      label: "Surface",
      position: { x: 420, y: 270 },
      params: { ...recipe.surface },
    },
    {
      id: "variation",
      kind: "variation",
      label: "Variation",
      position: { x: 40, y: 340 },
      params: { ...recipe.variation },
    },
    ...recipe.patternLayers.map((layer, index) => patternNodeFromLayer(layer, index)),
    outputNode,
  ];

  return {
    nodes,
    links: nodes
      .filter((node) => node.kind !== "output")
      .map((node) => ({ id: `${node.id}-output`, from: `${node.id}.out`, to: "output.in" })),
  };
}

export function compileRecipeFromBlueprint(
  blueprint: MaterialBlueprint,
  fallback: ProceduralMaterialRecipe,
): ProceduralMaterialRecipe {
  const palette = blueprint.nodes.find((node) => node.kind === "palette");
  const stylization = blueprint.nodes.find((node) => node.kind === "stylization");
  const surface = blueprint.nodes.find((node) => node.kind === "surface");
  const variation = blueprint.nodes.find((node) => node.kind === "variation");
  const patternLayers = blueprint.nodes
    .filter((node) => node.kind === "pattern")
    .map((node) => layerFromPatternNode(node));

  return {
    ...fallback,
    baseColor: stringParam(palette, "baseColor", fallback.baseColor),
    shadowColor: stringParam(palette, "shadowColor", fallback.shadowColor),
    highlightColor: stringParam(palette, "highlightColor", fallback.highlightColor),
    patternLayers,
    surface: {
      roughness: numberParam(surface, "roughness", fallback.surface.roughness),
      heightStrength: numberParam(surface, "heightStrength", fallback.surface.heightStrength),
      normalStrength: numberParam(surface, "normalStrength", fallback.surface.normalStrength),
      edgeSoftness: numberParam(surface, "edgeSoftness", fallback.surface.edgeSoftness),
    },
    stylization: {
      colorSteps: numberParam(stylization, "colorSteps", fallback.stylization.colorSteps),
      smoothing: numberParam(stylization, "smoothing", fallback.stylization.smoothing),
      saturation: numberParam(stylization, "saturation", fallback.stylization.saturation),
      valueBoost: numberParam(stylization, "valueBoost", fallback.stylization.valueBoost),
      microDetail: numberParam(stylization, "microDetail", fallback.stylization.microDetail),
    },
    variation: {
      enabled: boolParam(variation, "enabled", fallback.variation.enabled),
      perBlockStrength: numberParam(variation, "perBlockStrength", fallback.variation.perBlockStrength),
      colorJitter: numberParam(variation, "colorJitter", fallback.variation.colorJitter),
      patternJitter: numberParam(variation, "patternJitter", fallback.variation.patternJitter),
    },
  };
}

export function createBlueprintNode(kind: MaterialBlueprintNodeKind, x: number, y: number, index: number): MaterialBlueprintNode {
  if (kind === "pattern") {
    return patternNodeFromLayer({
      id: `pattern_${index}`,
      kind: "soft_blotches",
      blend: "overlay",
      domain: "warped_uv",
      mask: "none",
      strength: 0.2,
      scale: 6,
      contrast: 0.18,
      softness: 0.42,
      warp: 0.12,
      offsetX: 0,
      offsetY: 0,
      threshold: 0.5,
      color: "#9ACB4E",
      enabled: true,
    }, index, { x, y });
  }

  const defaults: Record<MaterialBlueprintNodeKind, MaterialBlueprintNode["params"]> = {
    palette: { baseColor: "#7BAA32", shadowColor: "#5F8D29", highlightColor: "#9ACB4E" },
    stylization: { colorSteps: 5, smoothing: 0.48, saturation: 1.08, valueBoost: 1.02, microDetail: 0.08 },
    surface: { roughness: 0.78, heightStrength: 0.14, normalStrength: 0.22, edgeSoftness: 0.42 },
    variation: { enabled: true, perBlockStrength: 0.18, colorJitter: 0.08, patternJitter: 0.12 },
    output: {},
    pattern: {},
  };

  return {
    id: `${kind}_${index}`,
    kind,
    label: labelForKind(kind),
    position: { x, y },
    params: defaults[kind],
  };
}

function patternNodeFromLayer(layer: MaterialPatternLayer, index: number, position?: { x: number; y: number }): MaterialBlueprintNode {
  return {
    id: layer.id || `pattern_${index + 1}`,
    kind: "pattern",
    label: layer.id ? labelFor(layer.id) : `Pattern ${index + 1}`,
    position: position ?? { x: 230, y: 120 + index * 170 },
    params: { ...layer },
  };
}

function layerFromPatternNode(node: MaterialBlueprintNode): MaterialPatternLayer {
  return {
    id: node.id,
    kind: stringParam(node, "kind", "soft_blotches") as MaterialPatternKind,
    blend: stringParam(node, "blend", "overlay") as MaterialPatternLayer["blend"],
    domain: stringParam(node, "domain", "warped_uv") as MaterialPatternLayer["domain"],
    mask: stringParam(node, "mask", "none") as MaterialPatternLayer["mask"],
    strength: numberParam(node, "strength", 0.2),
    scale: numberParam(node, "scale", 6),
    contrast: numberParam(node, "contrast", 0.18),
    softness: numberParam(node, "softness", 0.42),
    warp: numberParam(node, "warp", 0.12),
    offsetX: numberParam(node, "offsetX", 0),
    offsetY: numberParam(node, "offsetY", 0),
    threshold: numberParam(node, "threshold", 0.5),
    color: stringParam(node, "color", "#9ACB4E"),
    enabled: boolParam(node, "enabled", true),
  };
}

function stringParam(node: MaterialBlueprintNode | undefined, name: string, fallback: string) {
  const value = node?.params[name];
  return typeof value === "string" ? value : fallback;
}

function numberParam(node: MaterialBlueprintNode | undefined, name: string, fallback: number) {
  const value = Number(node?.params[name]);
  return Number.isFinite(value) ? value : fallback;
}

function boolParam(node: MaterialBlueprintNode | undefined, name: string, fallback: boolean) {
  const value = node?.params[name];
  return typeof value === "boolean" ? value : fallback;
}

function labelForKind(kind: MaterialBlueprintNodeKind) {
  return kind.charAt(0).toUpperCase() + kind.slice(1);
}

function labelFor(value: string) {
  return value.split("_").map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join(" ");
}

export function stylePresetLabel(style: MaterialStylePreset) {
  return style.split("_").map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join(" ");
}
