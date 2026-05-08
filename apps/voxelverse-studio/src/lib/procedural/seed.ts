export function hashString(value: string) {
  let hash = 2166136261;
  for (let i = 0; i < value.length; i += 1) {
    hash ^= value.charCodeAt(i);
    hash = Math.imul(hash, 16777619);
  }
  return hash >>> 0;
}

export function hashNumbers(...values: number[]) {
  let hash = 2166136261;
  for (const value of values) {
    hash ^= value >>> 0;
    hash = Math.imul(hash, 16777619);
  }
  return hash >>> 0;
}

export function randomSeed() {
  return Math.floor(100 + Math.random() * 999999);
}

export function seededRandom(seed: number) {
  let state = seed >>> 0;
  return () => {
    state = Math.imul(1664525, state) + 1013904223;
    return ((state >>> 0) / 4294967296);
  };
}

export function finalMaterialSeed(
  packSeed: number,
  blockId: string,
  blockSeed: number,
  materialId: string,
  face: string,
  previewPositionSeed = 0,
) {
  return hashNumbers(
    packSeed,
    hashString(blockId),
    blockSeed,
    hashString(materialId),
    hashString(face),
    previewPositionSeed,
  );
}
