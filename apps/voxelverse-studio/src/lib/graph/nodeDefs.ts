import type { GraphNodeKind } from "../../types/studio";

// ---------------------------------------------------------------------------
// Port & param type system
// ---------------------------------------------------------------------------

/** The data-type carried by a graph connection. */
export type EvalType = "Color" | "Float" | "Mask";

/** Definition of one input or output port on a node. */
export type NodePortDef = {
  name: string;
  type: EvalType;
  label: string;
  /** Default value used when the port has no incoming connection. */
  default?: number | string; // hex string for Color, number for Float/Mask
};

export type ParamType = "Color" | "Float" | "Bool" | "Select";

/** Definition of one editable parameter on a node. */
export type NodeParamDef = {
  name: string;
  type: ParamType;
  default: string | number | boolean;
  label: string;
  min?: number;
  max?: number;
  step?: number;
  /** For Select params. */
  options?: string[];
  /** Whether this param appears in Simple Mode. */
  exposed?: boolean;
  exposedLabel?: string;
};

export type NodeCategory = "input" | "pattern" | "shape" | "warp" | "blend" | "adjust" | "output";

/** Full static definition of a node type. */
export type NodeDef = {
  kind: GraphNodeKind;
  label: string;
  category: NodeCategory;
  /** Color used in the graph canvas header. */
  color: string;
  inputs: NodePortDef[];
  outputs: NodePortDef[];
  params: NodeParamDef[];
};

// ---------------------------------------------------------------------------
// Node definitions
// ---------------------------------------------------------------------------

const DEFS: NodeDef[] = [
  // ── INPUT ─────────────────────────────────────────────────────────────────
  {
    kind: "color",
    label: "Color",
    category: "input",
    color: "#2563eb",
    inputs: [],
    outputs: [{ name: "out", type: "Color", label: "Color" }],
    params: [
      { name: "value", type: "Color", default: "#7BAA32", label: "Color", exposed: true, exposedLabel: "Color" },
    ],
  },
  {
    kind: "float",
    label: "Float",
    category: "input",
    color: "#2563eb",
    inputs: [],
    outputs: [{ name: "out", type: "Float", label: "Value" }],
    params: [
      { name: "value", type: "Float", default: 0.5, label: "Value", min: 0, max: 1, step: 0.01, exposed: true, exposedLabel: "Value" },
    ],
  },

  // ── PATTERNS ──────────────────────────────────────────────────────────────
  {
    kind: "noise_fbm",
    label: "Noise FBM",
    category: "pattern",
    color: "#0f766e",
    inputs: [],
    outputs: [{ name: "out", type: "Mask", label: "Mask" }],
    params: [
      { name: "frequency", type: "Float", default: 4.0, label: "Frequency", min: 0.5, max: 32, step: 0.5, exposed: true, exposedLabel: "Scale" },
      { name: "octaves", type: "Float", default: 3, label: "Octaves", min: 1, max: 6, step: 1 },
      { name: "warp", type: "Float", default: 0, label: "Warp", min: 0, max: 1, step: 0.05 },
    ],
  },
  {
    kind: "voronoi",
    label: "Voronoi",
    category: "pattern",
    color: "#0f766e",
    inputs: [],
    outputs: [{ name: "out", type: "Mask", label: "Mask" }],
    params: [
      { name: "scale", type: "Float", default: 6.0, label: "Scale", min: 1, max: 32, step: 0.5, exposed: true, exposedLabel: "Cell Scale" },
      { name: "mode", type: "Select", default: "cells", label: "Mode", options: ["cells", "edges", "f2_minus_f1"] },
    ],
  },
  {
    kind: "stripes",
    label: "Stripes",
    category: "pattern",
    color: "#0f766e",
    inputs: [],
    outputs: [{ name: "out", type: "Mask", label: "Mask" }],
    params: [
      { name: "frequency", type: "Float", default: 8, label: "Count", min: 1, max: 64, step: 1, exposed: true, exposedLabel: "Stripe Count" },
      { name: "direction", type: "Select", default: "horizontal", label: "Direction", options: ["horizontal", "vertical"] },
      { name: "warp", type: "Float", default: 0.1, label: "Warp", min: 0, max: 1, step: 0.01 },
    ],
  },
  {
    kind: "rings",
    label: "Rings",
    category: "pattern",
    color: "#0f766e",
    inputs: [],
    outputs: [{ name: "out", type: "Mask", label: "Mask" }],
    params: [
      { name: "scale", type: "Float", default: 8, label: "Ring Count", min: 1, max: 64, step: 1, exposed: true, exposedLabel: "Ring Count" },
      { name: "warp", type: "Float", default: 0, label: "Warp", min: 0, max: 1, step: 0.01 },
    ],
  },
  {
    kind: "dots",
    label: "Dots",
    category: "pattern",
    color: "#0f766e",
    inputs: [],
    outputs: [{ name: "out", type: "Mask", label: "Mask" }],
    params: [
      { name: "scale", type: "Float", default: 6, label: "Scale", min: 1, max: 32, step: 0.5 },
      { name: "threshold", type: "Float", default: 0.72, label: "Dot Size", min: 0.5, max: 0.95, step: 0.01, exposed: true, exposedLabel: "Dot Size" },
    ],
  },
  {
    kind: "flat",
    label: "Flat",
    category: "pattern",
    color: "#0f766e",
    inputs: [],
    outputs: [{ name: "out", type: "Mask", label: "Value" }],
    params: [
      { name: "value", type: "Float", default: 1, label: "Value", min: 0, max: 1, step: 0.01 },
    ],
  },

  // ── SHAPES ────────────────────────────────────────────────────────────────
  {
    kind: "gradient",
    label: "Gradient",
    category: "shape",
    color: "#0e7490",
    inputs: [],
    outputs: [{ name: "out", type: "Mask", label: "Mask" }],
    params: [
      { name: "direction", type: "Select", default: "vertical", label: "Direction", options: ["vertical", "horizontal", "radial", "diagonal"] },
      { name: "invert", type: "Bool", default: false, label: "Invert" },
    ],
  },
  {
    kind: "band",
    label: "Band",
    category: "shape",
    color: "#0e7490",
    inputs: [],
    outputs: [{ name: "out", type: "Mask", label: "Mask" }],
    params: [
      { name: "position", type: "Float", default: 0.5, label: "Position", min: 0, max: 1, step: 0.01, exposed: true, exposedLabel: "Position" },
      { name: "width", type: "Float", default: 0.25, label: "Width", min: 0, max: 1, step: 0.01, exposed: true, exposedLabel: "Width" },
      { name: "softness", type: "Float", default: 0.1, label: "Softness", min: 0, max: 0.5, step: 0.01 },
    ],
  },
  {
    kind: "edge_mask",
    label: "Edge Mask",
    category: "shape",
    color: "#0e7490",
    inputs: [],
    outputs: [{ name: "out", type: "Mask", label: "Mask" }],
    params: [
      { name: "width", type: "Float", default: 0.15, label: "Edge Width", min: 0.01, max: 0.5, step: 0.01, exposed: true, exposedLabel: "Edge Width" },
      { name: "softness", type: "Float", default: 0.1, label: "Softness", min: 0, max: 0.5, step: 0.01 },
    ],
  },

  // ── WARP ──────────────────────────────────────────────────────────────────
  {
    kind: "domain_warp",
    label: "Domain Warp",
    category: "warp",
    color: "#7c3aed",
    inputs: [
      { name: "input", type: "Mask", label: "Input" },
    ],
    outputs: [{ name: "out", type: "Mask", label: "Warped" }],
    params: [
      { name: "strength", type: "Float", default: 0.3, label: "Strength", min: 0, max: 2, step: 0.05, exposed: true, exposedLabel: "Warp Strength" },
      { name: "frequency", type: "Float", default: 3, label: "Frequency", min: 0.5, max: 16, step: 0.5 },
    ],
  },

  // ── BLEND ─────────────────────────────────────────────────────────────────
  {
    kind: "blend_mix",
    label: "Mix",
    category: "blend",
    color: "#b45309",
    inputs: [
      { name: "a", type: "Color", label: "A", default: "#000000" },
      { name: "b", type: "Color", label: "B", default: "#ffffff" },
      { name: "factor", type: "Mask", label: "Factor", default: 0.5 },
    ],
    outputs: [{ name: "out", type: "Color", label: "Result" }],
    params: [
      { name: "factor", type: "Float", default: 0.5, label: "Factor", min: 0, max: 1, step: 0.01, exposed: true, exposedLabel: "Mix Factor" },
    ],
  },
  {
    kind: "blend_multiply",
    label: "Multiply",
    category: "blend",
    color: "#b45309",
    inputs: [
      { name: "a", type: "Color", label: "A", default: "#7f7f7f" },
      { name: "b", type: "Color", label: "B", default: "#7f7f7f" },
    ],
    outputs: [{ name: "out", type: "Color", label: "Result" }],
    params: [],
  },
  {
    kind: "blend_screen",
    label: "Screen",
    category: "blend",
    color: "#b45309",
    inputs: [
      { name: "a", type: "Color", label: "A", default: "#000000" },
      { name: "b", type: "Color", label: "B", default: "#000000" },
    ],
    outputs: [{ name: "out", type: "Color", label: "Result" }],
    params: [],
  },
  {
    kind: "blend_overlay",
    label: "Overlay",
    category: "blend",
    color: "#b45309",
    inputs: [
      { name: "a", type: "Color", label: "A", default: "#7BAA32" },
      { name: "b", type: "Color", label: "B", default: "#9ACB4E" },
      { name: "strength", type: "Mask", label: "Strength", default: 0.5 },
    ],
    outputs: [{ name: "out", type: "Color", label: "Result" }],
    params: [
      { name: "strength", type: "Float", default: 0.5, label: "Strength", min: 0, max: 1, step: 0.01, exposed: true, exposedLabel: "Blend Strength" },
    ],
  },

  // ── ADJUST ────────────────────────────────────────────────────────────────
  {
    kind: "remap",
    label: "Remap",
    category: "adjust",
    color: "#64748b",
    inputs: [{ name: "input", type: "Mask", label: "Input" }],
    outputs: [{ name: "out", type: "Mask", label: "Output" }],
    params: [
      { name: "inMin", type: "Float", default: 0, label: "In Min", min: 0, max: 1, step: 0.01 },
      { name: "inMax", type: "Float", default: 1, label: "In Max", min: 0, max: 1, step: 0.01 },
      { name: "outMin", type: "Float", default: 0, label: "Out Min", min: 0, max: 1, step: 0.01, exposed: true, exposedLabel: "Out Min" },
      { name: "outMax", type: "Float", default: 1, label: "Out Max", min: 0, max: 1, step: 0.01, exposed: true, exposedLabel: "Out Max" },
    ],
  },
  {
    kind: "contrast_adjust",
    label: "Contrast",
    category: "adjust",
    color: "#64748b",
    inputs: [{ name: "input", type: "Mask", label: "Input" }],
    outputs: [{ name: "out", type: "Mask", label: "Output" }],
    params: [
      { name: "contrast", type: "Float", default: 1.5, label: "Contrast", min: 0, max: 4, step: 0.05, exposed: true, exposedLabel: "Contrast" },
      { name: "brightness", type: "Float", default: 0, label: "Brightness", min: -1, max: 1, step: 0.01 },
    ],
  },
  {
    kind: "colorize",
    label: "Colorize",
    category: "adjust",
    color: "#64748b",
    inputs: [{ name: "mask", type: "Mask", label: "Mask" }],
    outputs: [{ name: "out", type: "Color", label: "Color" }],
    params: [
      { name: "colorLow", type: "Color", default: "#000000", label: "Shadow Color", exposed: true, exposedLabel: "Shadow" },
      { name: "colorHigh", type: "Color", default: "#ffffff", label: "Highlight Color", exposed: true, exposedLabel: "Highlight" },
    ],
  },
  {
    kind: "quantize",
    label: "Quantize",
    category: "adjust",
    color: "#64748b",
    inputs: [{ name: "input", type: "Color", label: "Color" }],
    outputs: [{ name: "out", type: "Color", label: "Output" }],
    params: [
      { name: "steps", type: "Float", default: 5, label: "Steps", min: 2, max: 16, step: 1, exposed: true, exposedLabel: "Color Steps" },
      { name: "blend", type: "Float", default: 0.42, label: "Blend", min: 0, max: 1, step: 0.01 },
    ],
  },

  // ── OUTPUT ────────────────────────────────────────────────────────────────
  {
    kind: "material_output",
    label: "Material Output",
    category: "output",
    color: "#be123c",
    inputs: [
      { name: "albedo", type: "Color", label: "Albedo" },
      { name: "roughness", type: "Mask", label: "Roughness", default: 0.75 },
    ],
    outputs: [],
    params: [],
  },
];

// ---------------------------------------------------------------------------
// Lookups
// ---------------------------------------------------------------------------

export const NODE_DEFS: Map<GraphNodeKind, NodeDef> = new Map(
  DEFS.map((def) => [def.kind, def]),
);

export function getNodeDef(kind: GraphNodeKind | string): NodeDef | undefined {
  return NODE_DEFS.get(kind as GraphNodeKind);
}

export const NODE_CATEGORIES: NodeCategory[] = [
  "input", "pattern", "shape", "warp", "blend", "adjust", "output",
];

export const NODES_BY_CATEGORY: Record<NodeCategory, NodeDef[]> = {
  input: DEFS.filter((d) => d.category === "input"),
  pattern: DEFS.filter((d) => d.category === "pattern"),
  shape: DEFS.filter((d) => d.category === "shape"),
  warp: DEFS.filter((d) => d.category === "warp"),
  blend: DEFS.filter((d) => d.category === "blend"),
  adjust: DEFS.filter((d) => d.category === "adjust"),
  output: DEFS.filter((d) => d.category === "output"),
};
