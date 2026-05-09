import type { ProceduralGraph } from "../../types/studio";

// ---------------------------------------------------------------------------
// Default graphs
// ---------------------------------------------------------------------------

/**
 * Minimal graph for a new blank material:
 *   Color → material_output.albedo
 */
export function createDefaultGraph(baseColor = "#7f7f7f"): ProceduralGraph {
  return {
    version: 1,
    nodes: [
      {
        id: "color_1",
        kind: "color",
        label: "Base Color",
        position: { x: 80, y: 160 },
        params: { value: baseColor },
        exposedParams: ["value"],
      },
      {
        id: "material_output",
        kind: "material_output",
        label: "Material Output",
        position: { x: 460, y: 160 },
        params: {},
        exposedParams: [],
      },
    ],
    connections: [
      {
        id: "color_1-out-material_output-albedo",
        fromNode: "color_1",
        fromPort: "out",
        toNode: "material_output",
        toPort: "albedo",
      },
    ],
  };
}

/**
 * Grass-like graph:
 *   FBM noise → Colorize(shadow, highlight) → Mix(base, colorize) → Output
 */
export function createGrassGraph(): ProceduralGraph {
  return {
    version: 1,
    nodes: [
      { id: "base_color", kind: "color", label: "Grass Color", position: { x: 60, y: 80 }, params: { value: "#7BAA32" }, exposedParams: ["value"] },
      { id: "shadow_color", kind: "color", label: "Shadow", position: { x: 60, y: 200 }, params: { value: "#5F8D29" }, exposedParams: ["value"] },
      { id: "highlight_color", kind: "color", label: "Highlight", position: { x: 60, y: 320 }, params: { value: "#9ACB4E" }, exposedParams: ["value"] },
      { id: "fbm_1", kind: "noise_fbm", label: "Patch Noise", position: { x: 60, y: 440 }, params: { frequency: 4, octaves: 3, warp: 0.1 }, exposedParams: ["frequency"] },
      { id: "colorize_1", kind: "colorize", label: "Grass Tones", position: { x: 300, y: 260 }, params: { colorLow: "#5F8D29", colorHigh: "#9ACB4E" }, exposedParams: ["colorLow", "colorHigh"] },
      { id: "mix_1", kind: "blend_mix", label: "Blend", position: { x: 520, y: 160 }, params: { factor: 0.6 }, exposedParams: ["factor"] },
      { id: "material_output", kind: "material_output", label: "Material Output", position: { x: 740, y: 160 }, params: {}, exposedParams: [] },
    ],
    connections: [
      { id: "fbm-colorize", fromNode: "fbm_1", fromPort: "out", toNode: "colorize_1", toPort: "mask" },
      { id: "base-mix-a", fromNode: "base_color", fromPort: "out", toNode: "mix_1", toPort: "a" },
      { id: "colorize-mix-b", fromNode: "colorize_1", fromPort: "out", toNode: "mix_1", toPort: "b" },
      { id: "mix-output", fromNode: "mix_1", fromPort: "out", toNode: "material_output", toPort: "albedo" },
    ],
  };
}

/**
 * Stone/rock-like graph:
 *   Voronoi + FBM → Colorize → Quantize → Output
 */
export function createStoneGraph(): ProceduralGraph {
  return {
    version: 1,
    nodes: [
      { id: "voronoi_1", kind: "voronoi", label: "Rock Cells", position: { x: 60, y: 100 }, params: { scale: 5, mode: "f2_minus_f1" }, exposedParams: ["scale"] },
      { id: "fbm_1", kind: "noise_fbm", label: "Surface Noise", position: { x: 60, y: 260 }, params: { frequency: 8, octaves: 4, warp: 0.08 }, exposedParams: ["frequency"] },
      { id: "mix_1", kind: "blend_mix", label: "Mix", position: { x: 280, y: 180 }, params: { factor: 0.4 }, exposedParams: ["factor"] },
      { id: "colorize_1", kind: "colorize", label: "Stone Tones", position: { x: 460, y: 180 }, params: { colorLow: "#4a4a4a", colorHigh: "#9a9a9a" }, exposedParams: ["colorLow", "colorHigh"] },
      { id: "quantize_1", kind: "quantize", label: "Stylize", position: { x: 640, y: 180 }, params: { steps: 4, blend: 0.5 }, exposedParams: ["steps"] },
      { id: "material_output", kind: "material_output", label: "Material Output", position: { x: 840, y: 180 }, params: {}, exposedParams: [] },
    ],
    connections: [
      { id: "voronoi-mix-a", fromNode: "voronoi_1", fromPort: "out", toNode: "mix_1", toPort: "a" },
      { id: "fbm-mix-b", fromNode: "fbm_1", fromPort: "out", toNode: "mix_1", toPort: "b" },
      { id: "mix-colorize", fromNode: "mix_1", fromPort: "out", toNode: "colorize_1", toPort: "mask" },
      { id: "colorize-quantize", fromNode: "colorize_1", fromPort: "out", toNode: "quantize_1", toPort: "input" },
      { id: "quantize-output", fromNode: "quantize_1", fromPort: "out", toNode: "material_output", toPort: "albedo" },
    ],
  };
}

/**
 * Wood planks graph:
 *   Stripes (rings) + FBM grain → Colorize → Output
 */
export function createWoodGraph(): ProceduralGraph {
  return {
    version: 1,
    nodes: [
      { id: "rings_1", kind: "rings", label: "Wood Rings", position: { x: 60, y: 120 }, params: { scale: 6, warp: 0.15 }, exposedParams: ["scale"] },
      { id: "fbm_1", kind: "noise_fbm", label: "Wood Grain", position: { x: 60, y: 280 }, params: { frequency: 16, octaves: 2, warp: 0 }, exposedParams: ["frequency"] },
      { id: "mix_1", kind: "blend_mix", label: "Mix", position: { x: 280, y: 200 }, params: { factor: 0.35 }, exposedParams: [] },
      { id: "colorize_1", kind: "colorize", label: "Wood Tones", position: { x: 460, y: 200 }, params: { colorLow: "#5C3A1E", colorHigh: "#A0622A" }, exposedParams: ["colorLow", "colorHigh"] },
      { id: "material_output", kind: "material_output", label: "Material Output", position: { x: 680, y: 200 }, params: {}, exposedParams: [] },
    ],
    connections: [
      { id: "rings-mix-a", fromNode: "rings_1", fromPort: "out", toNode: "mix_1", toPort: "a" },
      { id: "fbm-mix-b", fromNode: "fbm_1", fromPort: "out", toNode: "mix_1", toPort: "b" },
      { id: "mix-colorize", fromNode: "mix_1", fromPort: "out", toNode: "colorize_1", toPort: "mask" },
      { id: "colorize-output", fromNode: "colorize_1", fromPort: "out", toNode: "material_output", toPort: "albedo" },
    ],
  };
}

/** Map from templateKey (used in materialPresets) to a graph factory. */
export function createGraphForTemplate(templateKey: string): ProceduralGraph {
  if (templateKey.includes("grass")) return createGrassGraph();
  if (templateKey.includes("stone") || templateKey.includes("cobble") || templateKey.includes("rock")) return createStoneGraph();
  if (templateKey.includes("wood") || templateKey.includes("plank") || templateKey.includes("log")) return createWoodGraph();
  return createDefaultGraph("#888888");
}
