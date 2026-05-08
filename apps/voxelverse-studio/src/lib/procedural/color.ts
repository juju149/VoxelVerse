export type Rgb = { r: number; g: number; b: number };

export function clamp01(value: number) {
  return Math.max(0, Math.min(1, value));
}

export function hexToRgb(hex: string): Rgb {
  const clean = hex.replace("#", "").trim();
  const value = clean.length === 3
    ? clean.split("").map((part) => part + part).join("")
    : clean.padEnd(6, "0").slice(0, 6);
  const n = Number.parseInt(value, 16);
  if (Number.isNaN(n)) {
    return { r: 127, g: 127, b: 127 };
  }
  return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255 };
}

export function rgbToHex(color: Rgb) {
  const channel = (value: number) => Math.round(Math.max(0, Math.min(255, value))).toString(16).padStart(2, "0");
  return `#${channel(color.r)}${channel(color.g)}${channel(color.b)}`;
}

export function mix(a: Rgb, b: Rgb, t: number): Rgb {
  const f = clamp01(t);
  return {
    r: a.r + (b.r - a.r) * f,
    g: a.g + (b.g - a.g) * f,
    b: a.b + (b.b - a.b) * f,
  };
}

export function multiply(color: Rgb, factor: number): Rgb {
  return { r: color.r * factor, g: color.g * factor, b: color.b * factor };
}

export function add(color: Rgb, amount: number): Rgb {
  return { r: color.r + amount, g: color.g + amount, b: color.b + amount };
}

export function jitterColor(color: Rgb, jitter: number, random: number): Rgb {
  const amount = (random - 0.5) * 2 * jitter * 255;
  return add(color, amount);
}

export function screen(a: Rgb, b: Rgb): Rgb {
  return {
    r: 255 - ((255 - a.r) * (255 - b.r)) / 255,
    g: 255 - ((255 - a.g) * (255 - b.g)) / 255,
    b: 255 - ((255 - a.b) * (255 - b.b)) / 255,
  };
}

export function multiplyRgb(a: Rgb, b: Rgb): Rgb {
  return { r: (a.r * b.r) / 255, g: (a.g * b.g) / 255, b: (a.b * b.b) / 255 };
}

export function overlay(a: Rgb, b: Rgb): Rgb {
  const channel = (x: number, y: number) => x < 128 ? (2 * x * y) / 255 : 255 - (2 * (255 - x) * (255 - y)) / 255;
  return { r: channel(a.r, b.r), g: channel(a.g, b.g), b: channel(a.b, b.b) };
}

export function adjustSaturation(color: Rgb, saturation: number): Rgb {
  const gray = color.r * 0.299 + color.g * 0.587 + color.b * 0.114;
  return {
    r: gray + (color.r - gray) * saturation,
    g: gray + (color.g - gray) * saturation,
    b: gray + (color.b - gray) * saturation,
  };
}

export function quantizeColor(color: Rgb, steps: number, amount: number): Rgb {
  const count = Math.max(2, Math.floor(steps));
  const quantize = (value: number) => Math.round((value / 255) * (count - 1)) / (count - 1) * 255;
  return mix(color, { r: quantize(color.r), g: quantize(color.g), b: quantize(color.b) }, amount);
}
