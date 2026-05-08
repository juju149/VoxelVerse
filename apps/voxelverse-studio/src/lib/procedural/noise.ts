import { hashNumbers } from "./seed";

export function smoothStep(edge0: number, edge1: number, x: number) {
  const t = Math.max(0, Math.min(1, (x - edge0) / (edge1 - edge0)));
  return t * t * (3 - 2 * t);
}

function lerp(a: number, b: number, t: number) {
  return a + (b - a) * t;
}

function lattice(seed: number, x: number, y: number) {
  return (hashNumbers(seed, x, y) % 10000) / 10000;
}

export function valueNoise(seed: number, x: number, y: number) {
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  const xf = x - xi;
  const yf = y - yi;
  const u = smoothStep(0, 1, xf);
  const v = smoothStep(0, 1, yf);
  const a = lattice(seed, xi, yi);
  const b = lattice(seed, xi + 1, yi);
  const c = lattice(seed, xi, yi + 1);
  const d = lattice(seed, xi + 1, yi + 1);
  return lerp(lerp(a, b, u), lerp(c, d, u), v);
}

export function fbm(seed: number, x: number, y: number, octaves = 3) {
  let value = 0;
  let amp = 0.5;
  let freq = 1;
  let total = 0;
  for (let i = 0; i < octaves; i += 1) {
    value += valueNoise(seed + i * 1013, x * freq, y * freq) * amp;
    total += amp;
    amp *= 0.5;
    freq *= 2;
  }
  return value / total;
}

export function organicCells(seed: number, x: number, y: number, scale: number) {
  const sx = x * scale;
  const sy = y * scale;
  const ix = Math.floor(sx);
  const iy = Math.floor(sy);
  let minDistance = 2;
  for (let yy = -1; yy <= 1; yy += 1) {
    for (let xx = -1; xx <= 1; xx += 1) {
      const cx = ix + xx + lattice(seed, ix + xx, iy + yy);
      const cy = iy + yy + lattice(seed + 33, ix + xx, iy + yy);
      const dx = sx - cx;
      const dy = sy - cy;
      minDistance = Math.min(minDistance, Math.sqrt(dx * dx + dy * dy));
    }
  }
  return Math.max(0, Math.min(1, minDistance));
}
