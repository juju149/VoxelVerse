import type { MaterialFaceDef, MaterialParam, MaterialPatternLayer, ParamValue } from "../../types/studio";
import {
  adjustSaturation,
  clamp01,
  hexToRgb,
  jitterColor,
  mix,
  multiply,
  multiplyRgb,
  overlay,
  quantizeColor,
  screen,
  type Rgb,
} from "./color";
import { fbm, organicCells, smoothStep } from "./noise";
import { hashString, seededRandom } from "./seed";

type PreviewOptions = {
  seedOverride?: number;
  size?: number;
};

type SamplePoint = {
  u: number;
  v: number;
  x: number;
  y: number;
};

function paramValue(params: MaterialParam[], name: string, fallback: ParamValue) {
  return params.find((param) => param.name === name)?.value ?? fallback;
}

function numberParam(params: MaterialParam[], name: string, fallback: number) {
  const value = Number(paramValue(params, name, fallback));
  return Number.isFinite(value) ? value : fallback;
}

function writePixel(data: Uint8ClampedArray, index: number, color: Rgb) {
  data[index] = Math.round(Math.max(0, Math.min(255, color.r)));
  data[index + 1] = Math.round(Math.max(0, Math.min(255, color.g)));
  data[index + 2] = Math.round(Math.max(0, Math.min(255, color.b)));
  data[index + 3] = 255;
}

export function generateMaterialPreviewDataUrl(material: MaterialFaceDef, options: PreviewOptions = {}) {
  const size = options.size ?? material.resolutionPreview;
  const canvas = document.createElement("canvas");
  canvas.width = size;
  canvas.height = size;
  const context = canvas.getContext("2d");
  if (!context) {
    return "";
  }

  const seed = options.seedOverride ?? material.seed;
  const image = context.createImageData(size, size);
  const recipe = material.recipe;
  const random = seededRandom(seed + hashString(material.id));
  const base = jitterColor(hexToRgb(recipe.baseColor), recipe.variation.enabled ? recipe.variation.colorJitter : 0, random());
  const shadow = hexToRgb(recipe.shadowColor);
  const highlight = hexToRgb(recipe.highlightColor);
  const detail = numberParam(recipe.params, "detail", 0.18);
  const patchScale = numberParam(recipe.params, "patch_scale", 6);
  const contrast = numberParam(recipe.params, "contrast", 0.22);
  const variation = recipe.variation.enabled ? recipe.variation.perBlockStrength : 0;

  for (let y = 0; y < size; y += 1) {
    for (let x = 0; x < size; x += 1) {
      const point = { u: x / size, v: y / size, x, y };
      const broad = fbm(seed, point.u * patchScale, point.v * patchScale, 3);
      let color = mix(shadow, highlight, clamp01(0.46 + broad * contrast + detail * 0.2));
      color = mix(base, color, recipe.stylization.smoothing);

      for (const layer of recipe.patternLayers) {
        if (layer.enabled) {
          color = applyLayer(layer, seed, point, color);
        }
      }

      if (variation > 0) {
        const local = fbm(seed + 503, point.u * 2, point.v * 2, 2);
        color = mix(color, jitterColor(color, variation * 0.25, local), variation);
      }

      if (recipe.stylization.microDetail > 0) {
        const micro = fbm(seed + 881, point.u * 48, point.v * 48, 2) - 0.5;
        color = multiply(color, 1 + micro * recipe.stylization.microDetail);
      }

      color = adjustSaturation(color, recipe.stylization.saturation);
      color = multiply(color, recipe.stylization.valueBoost);
      color = quantizeColor(color, recipe.stylization.colorSteps, 0.42);
      writePixel(image.data, (y * size + x) * 4, color);
    }
  }

  context.putImageData(image, 0, 0);
  return canvas.toDataURL("image/png");
}

function applyLayer(layer: MaterialPatternLayer, seed: number, point: SamplePoint, current: Rgb) {
  const layerColor = hexToRgb(layer.color ?? "#ffffff");
  const factor = shapedPatternValue(layer, seed, point);
  const mask = maskValue(layer, seed, point);
  const amount = clamp01(factor * mask * layer.strength);
  const blended = blend(current, layerColor, layer.blend);
  return mix(current, blended, amount);
}

function shapedPatternValue(layer: MaterialPatternLayer, seed: number, point: SamplePoint) {
  const domain = domainPoint(layer, seed, point);
  let factor = 0;
  const layerSeed = seed + hashString(layer.id);

  if (layer.kind === "soft_noise") {
    factor = fbm(layerSeed, domain.u * layer.scale, domain.v * layer.scale, 3);
  } else if (layer.kind === "soft_blotches") {
    factor = smoothStep(0.28, 0.82, fbm(layerSeed, domain.u * layer.scale, domain.v * layer.scale, 3));
  } else if (layer.kind === "organic_cells" || layer.kind === "rounded_pebbles") {
    factor = 1 - organicCells(layerSeed, domain.u, domain.v, layer.scale);
    factor = smoothStep(layer.threshold ?? 0.55, 1, factor);
  } else if (layer.kind === "patch_cells") {
    const cells = 1 - organicCells(layerSeed, domain.u, domain.v, layer.scale);
    const broad = fbm(layerSeed + 29, domain.u * 4, domain.v * 4, 3);
    factor = clamp01(cells * 0.7 + broad * 0.35);
  } else if (layer.kind === "rings") {
    const dx = domain.u - 0.5;
    const dy = domain.v - 0.5;
    factor = 0.5 + Math.sin(Math.sqrt(dx * dx + dy * dy) * layer.scale * 18) * 0.5;
  } else if (layer.kind === "stripes" || layer.kind === "bands") {
    const axis = layer.domain === "vertical" ? domain.v : domain.u;
    factor = 0.5 + Math.sin((axis + fbm(layerSeed, domain.u * 3, domain.v * 3, 2) * layer.warp) * layer.scale * 6) * 0.5;
  } else if (layer.kind === "dots") {
    const cells = 1 - organicCells(layerSeed, domain.u, domain.v, layer.scale);
    factor = smoothStep(layer.threshold ?? 0.72, 1, cells);
  } else if (layer.kind === "cracks") {
    const cells = organicCells(layerSeed, domain.u, domain.v, layer.scale);
    factor = 1 - smoothStep(0.02, layer.softness * 0.18 + 0.03, Math.abs(cells - (layer.threshold ?? 0.28)));
  } else if (layer.kind === "edge_band") {
    factor = bandValue(layer, domain.v);
  } else {
    factor = 0.5;
  }

  return clamp01((factor - 0.5) * (1 + layer.contrast * 4) + 0.5);
}

function domainPoint(layer: MaterialPatternLayer, seed: number, point: SamplePoint) {
  let u = point.u + layer.offsetX;
  let v = point.v + layer.offsetY;
  if (layer.domain === "warped_uv" || layer.warp > 0) {
    const warp = layer.warp;
    u += (fbm(seed + hashString(layer.id) + 17, point.u * 3, point.v * 3, 2) - 0.5) * warp;
    v += (fbm(seed + hashString(layer.id) + 31, point.u * 3, point.v * 3, 2) - 0.5) * warp;
  }
  if (layer.domain === "radial") {
    const dx = point.u - 0.5;
    const dy = point.v - 0.5;
    u = Math.sqrt(dx * dx + dy * dy);
    v = Math.atan2(dy, dx) / (Math.PI * 2) + 0.5;
  } else if (layer.domain === "vertical") {
    u = point.v;
    v = point.u;
  } else if (layer.domain === "horizontal") {
    u = point.u;
    v = point.v;
  }
  return { u, v };
}

function maskValue(layer: MaterialPatternLayer, seed: number, point: SamplePoint) {
  if (layer.mask === "none") {
    return 1;
  }
  if (layer.mask === "top_band") {
    const height = layer.threshold ?? 0.28;
    const edge = height + (fbm(seed + hashString(layer.id) + 71, point.u * 6, 0.3, 2) - 0.5) * layer.warp;
    return 1 - smoothStep(edge, edge + layer.softness * 0.25, point.v);
  }
  if (layer.mask === "bottom_band") {
    const height = layer.threshold ?? 0.28;
    const edge = 1 - height + (fbm(seed + hashString(layer.id) + 73, point.u * 6, 0.3, 2) - 0.5) * layer.warp;
    return smoothStep(edge - layer.softness * 0.25, edge, point.v);
  }
  if (layer.mask === "vertical_gradient") {
    return clamp01(point.v);
  }
  if (layer.mask === "center_soft") {
    const dx = point.u - 0.5;
    const dy = point.v - 0.5;
    return 1 - smoothStep(0.1, 0.72, Math.sqrt(dx * dx + dy * dy));
  }
  if (layer.mask === "edge_wear") {
    const edge = Math.min(point.u, point.v, 1 - point.u, 1 - point.v);
    return 1 - smoothStep(0.02, 0.18 + layer.softness * 0.2, edge);
  }
  return 1;
}

function bandValue(layer: MaterialPatternLayer, value: number) {
  const cutoff = layer.threshold ?? 0.5;
  return 1 - smoothStep(cutoff, cutoff + layer.softness * 0.25, value);
}

function blend(base: Rgb, layer: Rgb, mode: MaterialPatternLayer["blend"]) {
  if (mode === "multiply") {
    return multiplyRgb(base, layer);
  }
  if (mode === "screen") {
    return screen(base, layer);
  }
  if (mode === "overlay") {
    return overlay(base, layer);
  }
  if (mode === "add") {
    return { r: base.r + layer.r * 0.35, g: base.g + layer.g * 0.35, b: base.b + layer.b * 0.35 };
  }
  if (mode === "subtract") {
    return { r: base.r - layer.r * 0.35, g: base.g - layer.g * 0.35, b: base.b - layer.b * 0.35 };
  }
  if (mode === "shadow") {
    return mix(multiply(base, 0.72), multiplyRgb(base, layer), 0.35);
  }
  if (mode === "highlight") {
    return mix(screen(base, layer), multiply(layer, 1.08), 0.35);
  }
  return layer;
}
