/**
 * graphEvaluator.ts
 *
 * Real per-pixel graph evaluation for ProceduralGraph.
 * Topologically sorts nodes, then evaluates each node in order for every pixel.
 * Connections drive the data flow — upstream output is passed as downstream input.
 */

import type { GraphNode, GraphNodeKind, ProceduralGraph } from "../../types/studio";
import {
  adjustSaturation,
  clamp01,
  hexToRgb,
  mix,
  multiply,
  multiplyRgb,
  overlay,
  quantizeColor,
  screen,
  type Rgb,
} from "../procedural/color";
import { fbm, organicCells, smoothStep, valueNoise } from "../procedural/noise";
import { hashString } from "../procedural/seed";

// ---------------------------------------------------------------------------
// Evaluation value types
// ---------------------------------------------------------------------------

/** A color (RGB channels 0..255). */
type EvalColor = Rgb;
/** A scalar (Float / Mask 0..1). */
type EvalFloat = number;
/** Either type, keyed by output port name. */
type EvalOutput = Map<string, EvalColor | EvalFloat>;

// ---------------------------------------------------------------------------
// Topological sort (Kahn's algorithm — guarantees no-cycle prerequisite)
// ---------------------------------------------------------------------------

export function topologicalSort(graph: ProceduralGraph): string[] {
  const inDegree: Map<string, number> = new Map();
  const dependents: Map<string, string[]> = new Map();

  for (const node of graph.nodes) {
    inDegree.set(node.id, 0);
    dependents.set(node.id, []);
  }

  for (const conn of graph.connections) {
    if (!inDegree.has(conn.toNode) || !inDegree.has(conn.fromNode)) continue;
    inDegree.set(conn.toNode, (inDegree.get(conn.toNode) ?? 0) + 1);
    dependents.get(conn.fromNode)?.push(conn.toNode);
  }

  const queue: string[] = [];
  for (const [id, deg] of inDegree) {
    if (deg === 0) queue.push(id);
  }

  const order: string[] = [];
  while (queue.length > 0) {
    const current = queue.shift()!;
    order.push(current);
    for (const dep of dependents.get(current) ?? []) {
      const next = (inDegree.get(dep) ?? 1) - 1;
      inDegree.set(dep, next);
      if (next === 0) queue.push(dep);
    }
  }

  // Append any remaining nodes (handles cycles gracefully by skipping them)
  for (const node of graph.nodes) {
    if (!order.includes(node.id)) order.push(node.id);
  }

  return order;
}

// ---------------------------------------------------------------------------
// Input resolution
// ---------------------------------------------------------------------------

function resolveInput(
  nodeId: string,
  portName: string,
  graph: ProceduralGraph,
  cache: Map<string, EvalOutput>,
): EvalColor | EvalFloat | undefined {
  const conn = graph.connections.find(
    (c) => c.toNode === nodeId && c.toPort === portName,
  );
  if (!conn) return undefined;
  return cache.get(conn.fromNode)?.get(conn.fromPort);
}

function getFloat(val: EvalColor | EvalFloat | undefined, fallback: number): number {
  if (val === undefined) return fallback;
  if (typeof val === "number") return val;
  // Convert color to luminance
  return clamp01((val.r * 0.299 + val.g * 0.587 + val.b * 0.114) / 255);
}

function getColor(val: EvalColor | EvalFloat | undefined, fallback: string): EvalColor {
  if (val === undefined) return hexToRgb(fallback);
  if (typeof val === "number") {
    const v = val * 255;
    return { r: v, g: v, b: v };
  }
  return val;
}

function numParam(node: GraphNode, name: string, fallback: number): number {
  const v = Number(node.params[name]);
  return Number.isFinite(v) ? v : fallback;
}

function strParam(node: GraphNode, name: string, fallback: string): string {
  const v = node.params[name];
  return typeof v === "string" ? v : fallback;
}

function boolParam(node: GraphNode, name: string, fallback: boolean): boolean {
  const v = node.params[name];
  return typeof v === "boolean" ? v : fallback;
}

// ---------------------------------------------------------------------------
// Per-node evaluation
// ---------------------------------------------------------------------------

function evaluateNode(
  node: GraphNode,
  graph: ProceduralGraph,
  cache: Map<string, EvalOutput>,
  seed: number,
  u: number,
  v: number,
): EvalOutput {
  const out: EvalOutput = new Map();
  const k = node.kind as GraphNodeKind;
  const nodeSeed = seed + hashString(node.id);

  if (k === "color") {
    out.set("out", hexToRgb(strParam(node, "value", "#7f7f7f")));

  } else if (k === "float") {
    out.set("out", clamp01(numParam(node, "value", 0.5)));

  } else if (k === "noise_fbm") {
    const freq = numParam(node, "frequency", 4);
    const octaves = Math.round(numParam(node, "octaves", 3));
    const warp = numParam(node, "warp", 0);
    let wu = u, wv = v;
    if (warp > 0) {
      wu += (fbm(nodeSeed + 17, u * 3, v * 3, 2) - 0.5) * warp;
      wv += (fbm(nodeSeed + 31, u * 3, v * 3, 2) - 0.5) * warp;
    }
    out.set("out", fbm(nodeSeed, wu * freq, wv * freq, octaves));

  } else if (k === "voronoi") {
    const scale = numParam(node, "scale", 6);
    const mode = strParam(node, "mode", "cells");
    const cells = organicCells(nodeSeed, u, v, scale);
    if (mode === "edges") {
      out.set("out", 1 - smoothStep(0, 0.12, cells));
    } else if (mode === "f2_minus_f1") {
      const cells2 = organicCells(nodeSeed + 7, u, v, scale);
      out.set("out", clamp01(cells2 - cells));
    } else {
      out.set("out", 1 - cells);
    }

  } else if (k === "stripes") {
    const freq = numParam(node, "frequency", 8);
    const warp = numParam(node, "warp", 0.1);
    const dir = strParam(node, "direction", "horizontal");
    const axis = dir === "vertical" ? v : u;
    const noise = warp > 0 ? (fbm(nodeSeed, u * 3, v * 3, 2) - 0.5) * warp : 0;
    out.set("out", 0.5 + Math.sin((axis + noise) * freq * Math.PI * 2) * 0.5);

  } else if (k === "rings") {
    const scale = numParam(node, "scale", 8);
    const warp = numParam(node, "warp", 0);
    const dx = u - 0.5;
    const dy = v - 0.5;
    const radius = Math.sqrt(dx * dx + dy * dy);
    const noise = warp > 0 ? (valueNoise(nodeSeed, u * 4, v * 4) - 0.5) * warp * 0.2 : 0;
    out.set("out", 0.5 + Math.sin((radius + noise) * scale * Math.PI * 2) * 0.5);

  } else if (k === "dots") {
    const scale = numParam(node, "scale", 6);
    const threshold = numParam(node, "threshold", 0.72);
    const cells = 1 - organicCells(nodeSeed, u, v, scale);
    out.set("out", smoothStep(threshold, 1, cells));

  } else if (k === "flat") {
    out.set("out", numParam(node, "value", 1));

  } else if (k === "gradient") {
    const dir = strParam(node, "direction", "vertical");
    const invert = boolParam(node, "invert", false);
    let val: number;
    if (dir === "radial") {
      const dx = u - 0.5, dy = v - 0.5;
      val = 1 - clamp01(Math.sqrt(dx * dx + dy * dy) * 2);
    } else if (dir === "horizontal") {
      val = u;
    } else if (dir === "diagonal") {
      val = (u + v) * 0.5;
    } else {
      val = v; // vertical
    }
    out.set("out", invert ? 1 - val : val);

  } else if (k === "band") {
    const position = numParam(node, "position", 0.5);
    const width = numParam(node, "width", 0.25);
    const softness = numParam(node, "softness", 0.1);
    const half = width * 0.5;
    const dist = Math.abs(v - position);
    out.set("out", 1 - smoothStep(half, half + softness, dist));

  } else if (k === "edge_mask") {
    const width = numParam(node, "width", 0.15);
    const softness = numParam(node, "softness", 0.1);
    const edge = Math.min(u, v, 1 - u, 1 - v);
    out.set("out", 1 - smoothStep(width, width + softness, edge));

  } else if (k === "domain_warp") {
    const strength = numParam(node, "strength", 0.3);
    const freq = numParam(node, "frequency", 3);
    const inputVal = resolveInput(node.id, "input", graph, cache);
    const baseVal = getFloat(inputVal, 0.5);
    // Apply FBM warp on the input mask value
    const wx = (fbm(nodeSeed + 17, u * freq, v * freq, 2) - 0.5) * strength;
    const wy = (fbm(nodeSeed + 31, u * freq, v * freq, 2) - 0.5) * strength;
    const warpedVal = fbm(nodeSeed, (u + wx) * freq, (v + wy) * freq, 2);
    out.set("out", clamp01(mix({ r: baseVal * 255, g: baseVal * 255, b: baseVal * 255 }, { r: warpedVal * 255, g: warpedVal * 255, b: warpedVal * 255 }, 0.7).r / 255));

  } else if (k === "blend_mix") {
    const aRaw = resolveInput(node.id, "a", graph, cache);
    const bRaw = resolveInput(node.id, "b", graph, cache);
    const factorRaw = resolveInput(node.id, "factor", graph, cache);
    const a = getColor(aRaw, "#000000");
    const b = getColor(bRaw, "#ffffff");
    const factor = getFloat(factorRaw, numParam(node, "factor", 0.5));
    out.set("out", mix(a, b, factor));

  } else if (k === "blend_multiply") {
    const a = getColor(resolveInput(node.id, "a", graph, cache), "#7f7f7f");
    const b = getColor(resolveInput(node.id, "b", graph, cache), "#7f7f7f");
    out.set("out", multiplyRgb(a, b));

  } else if (k === "blend_screen") {
    const a = getColor(resolveInput(node.id, "a", graph, cache), "#000000");
    const b = getColor(resolveInput(node.id, "b", graph, cache), "#000000");
    out.set("out", screen(a, b));

  } else if (k === "blend_overlay") {
    const aRaw = resolveInput(node.id, "a", graph, cache);
    const bRaw = resolveInput(node.id, "b", graph, cache);
    const strengthRaw = resolveInput(node.id, "strength", graph, cache);
    const a = getColor(aRaw, "#7BAA32");
    const b = getColor(bRaw, "#9ACB4E");
    const strength = getFloat(strengthRaw, numParam(node, "strength", 0.5));
    out.set("out", mix(a, overlay(a, b), strength));

  } else if (k === "remap") {
    const inputVal = getFloat(resolveInput(node.id, "input", graph, cache), 0.5);
    const inMin = numParam(node, "inMin", 0);
    const inMax = numParam(node, "inMax", 1);
    const outMin = numParam(node, "outMin", 0);
    const outMax = numParam(node, "outMax", 1);
    const range = inMax - inMin;
    const t = range === 0 ? 0 : (inputVal - inMin) / range;
    out.set("out", clamp01(outMin + (outMax - outMin) * t));

  } else if (k === "contrast_adjust") {
    const inputVal = getFloat(resolveInput(node.id, "input", graph, cache), 0.5);
    const contrast = numParam(node, "contrast", 1.5);
    const brightness = numParam(node, "brightness", 0);
    const adjusted = (inputVal - 0.5) * contrast + 0.5 + brightness;
    out.set("out", clamp01(adjusted));

  } else if (k === "colorize") {
    const maskVal = getFloat(resolveInput(node.id, "mask", graph, cache), 0.5);
    const low = hexToRgb(strParam(node, "colorLow", "#000000"));
    const high = hexToRgb(strParam(node, "colorHigh", "#ffffff"));
    out.set("out", mix(low, high, maskVal));

  } else if (k === "quantize") {
    const inputColor = getColor(resolveInput(node.id, "input", graph, cache), "#7f7f7f");
    const steps = numParam(node, "steps", 5);
    const blendAmt = numParam(node, "blend", 0.42);
    const sat = adjustSaturation(inputColor, 1);
    const quantized = quantizeColor(sat, steps, blendAmt);
    out.set("out", multiply(quantized, 1));

  } else if (k === "material_output") {
    // Material output just passes its inputs through for reading after evaluation
    const albedo = resolveInput(node.id, "albedo", graph, cache);
    const roughness = resolveInput(node.id, "roughness", graph, cache);
    out.set("albedo", albedo ?? { r: 127, g: 127, b: 127 });
    out.set("roughness", roughness ?? 0.75);
  }

  return out;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

type GraphPreviewOptions = {
  seedOverride?: number;
  size?: number;
};

/**
 * Evaluates a ProceduralGraph into a canvas PNG data URL.
 * Returns an empty string if the graph has no output node or no canvas support.
 */
export function evaluateGraphDataUrl(
  graph: ProceduralGraph,
  options: GraphPreviewOptions = {},
): string {
  const size = options.size ?? 128;
  const seed = options.seedOverride ?? 0;

  const canvas = document.createElement("canvas");
  canvas.width = size;
  canvas.height = size;
  const context = canvas.getContext("2d");
  if (!context) return "";

  const order = topologicalSort(graph);
  const outputNode = graph.nodes.find((n) => n.kind === "material_output");
  if (!outputNode) return "";

  const image = context.createImageData(size, size);

  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const u = x / size;
      const v = y / size;
      const cache = new Map<string, EvalOutput>();

      for (const nodeId of order) {
        const node = graph.nodes.find((n) => n.id === nodeId);
        if (!node) continue;
        cache.set(nodeId, evaluateNode(node, graph, cache, seed, u, v));
      }

      const outCache = cache.get(outputNode.id);
      const albedo = outCache?.get("albedo");
      const color: Rgb = albedo && typeof albedo !== "number"
        ? albedo
        : { r: 127, g: 127, b: 127 };

      const idx = (y * size + x) * 4;
      image.data[idx] = Math.round(Math.max(0, Math.min(255, color.r)));
      image.data[idx + 1] = Math.round(Math.max(0, Math.min(255, color.g)));
      image.data[idx + 2] = Math.round(Math.max(0, Math.min(255, color.b)));
      image.data[idx + 3] = 255;
    }
  }

  context.putImageData(image, 0, 0);
  return canvas.toDataURL("image/png");
}
