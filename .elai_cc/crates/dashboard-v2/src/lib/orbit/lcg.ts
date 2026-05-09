// orbit/lcg.ts
// Seeded linear-congruential generator for entropy-quarantined determinism.
// Same seed -> identical sequence -> identical orbit positions for the same task.
// Pattern copied from Starfield.svelte (line 36-43) for consistency.

export function lcg(seed: number): () => number {
  let s = seed >>> 0;
  return () => {
    s = Math.imul(1664525, s) + 1013904223;
    s >>>= 0;
    return s / 4294967296;
  };
}

// FNV-1a 32-bit hash. Used to fold an id string into a numeric seed.
// Not cryptographic — only for visual determinism.
export function hashId(id: string): number {
  let h = 0x811c9dc5 >>> 0;
  for (let i = 0; i < id.length; i++) {
    h ^= id.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h >>> 0;
}

// Combine two ids into one seed (e.g. task_id + concept_id).
export function combineSeeds(a: number, b: number): number {
  return ((Math.imul(a, 0x9E3779B1) >>> 0) ^ b) >>> 0;
}

// Convenience: deterministic random in [min, max) from an id.
export function jitterFromId(id: string, min: number, max: number): number {
  const rng = lcg(hashId(id));
  return min + rng() * (max - min);
}
