// sky-dome.ts — procedural backdrop sphere ("camera inside a ball").
//
// One large `THREE.SphereGeometry` rendered with `THREE.BackSide` and a
// custom `ShaderMaterial`. The shader paints, in a single fragment pass:
//   • Multi-octave FBM nebula clouds in three palette colors (teal /
//     steel / muted lavender) pulled from CSS tokens. Each cloud uses
//     a different frequency + offset so the layout reads as varied
//     atmospheric tints rather than a uniform wash.
//   • A dense procedural background star field via 3D cell-hashing on
//     the view direction (~2k visible pinpoints across three size
//     tiers). Each cell rolls a phase + period, so stars slowly
//     fade in/out at different rates — the user's "stars appear and
//     disappear in random places" without any CPU cost.
//   • A subtle radial darkening so the central sun's bloom keeps
//     visual primacy.
//
// All output is clamped to LDR (RGB < 0.78) so the EffectComposer's
// bloom threshold (0.85) never picks the dome up — the sun stays the
// only blooming object.
//
// The sphere is positioned to follow the camera each frame (set in
// `StarCanvas.svelte::tick`) so stars/nebulas read as fixed at
// "infinity" — camera rotation reveals different parts of the dome,
// camera translation does not parallax them.

import * as THREE from 'three';

export interface SkyDomeConfig {
  radius?: number;
  widthSegments?: number;
  heightSegments?: number;
  /** Seed offset baked into the noise sampling — same value reproduces
   *  the same nebula layout across reloads. */
  seed?: number;
}

export interface SkyDomeRefs {
  mesh: THREE.Mesh;
  material: THREE.ShaderMaterial;
  /** Update per-frame uniforms (called from the rAF tick). */
  setFrame(timeMs: number, reducedMotion: boolean): void;
  /** Update the camera→sun direction (unit vec3). Currently unused
   *  by the shader; kept wired for cheap future re-enable. */
  setSunDir(dir: THREE.Vector3): void;
  dispose(): void;
}

const VERT = /* glsl */ `
  precision highp float;
  varying vec3 vDir;
  void main() {
    // BackSide: we are inside the sphere. Use the local-space position
    // (not world) as the view direction so the dome rotates with the
    // mesh (which we'll re-anchor to the camera each frame in JS).
    vDir = normalize(position);
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;

const FRAG = /* glsl */ `
  precision highp float;
  varying vec3 vDir;

  uniform float uTimeMs;
  uniform float uReducedMotion;
  uniform float uSeed;
  uniform vec3  uNebulaTeal;
  uniform vec3  uNebulaSteel;
  uniform vec3  uNebulaViolet;
  uniform vec3  uNebulaDeep;
  uniform vec3  uStarColor;
  // Direction from camera to sun (world space, unit vector). Updated
  // each frame from JS — currently unused in the shader but kept
  // wired so a future cone-attenuation re-enable doesn't need a
  // round-trip through StarCanvas.
  uniform vec3  uSunDir;

  // ---- 3D hash + value noise (seam-free on the sphere) -------------
  float hash13(vec3 p) {
    p = fract(p * 0.1031);
    p += dot(p, p.yzx + 33.33);
    return fract((p.x + p.y) * p.z);
  }

  float noise3(vec3 p) {
    vec3 i = floor(p);
    vec3 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float n000 = hash13(i + vec3(0.0, 0.0, 0.0));
    float n100 = hash13(i + vec3(1.0, 0.0, 0.0));
    float n010 = hash13(i + vec3(0.0, 1.0, 0.0));
    float n110 = hash13(i + vec3(1.0, 1.0, 0.0));
    float n001 = hash13(i + vec3(0.0, 0.0, 1.0));
    float n101 = hash13(i + vec3(1.0, 0.0, 1.0));
    float n011 = hash13(i + vec3(0.0, 1.0, 1.0));
    float n111 = hash13(i + vec3(1.0, 1.0, 1.0));
    float nx00 = mix(n000, n100, f.x);
    float nx10 = mix(n010, n110, f.x);
    float nx01 = mix(n001, n101, f.x);
    float nx11 = mix(n011, n111, f.x);
    float nxy0 = mix(nx00, nx10, f.y);
    float nxy1 = mix(nx01, nx11, f.y);
    return mix(nxy0, nxy1, f.z);
  }

  // 5-octave fractal Brownian motion. Lacunarity 2.07, gain 0.5 — the
  // 5th octave adds visible high-frequency wisp detail (the "low
  // quality blob" complaint on the 4-octave version was the missing
  // top end). Total amplitude ~0.97, fed into tight smoothstep masks.
  float fbm(vec3 p) {
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < 5; i++) {
      v += a * noise3(p);
      p *= 2.07;
      a *= 0.5;
    }
    return v;
  }

  // ---- Cell-hashed background star field ---------------------------
  // Sampled in 3D directly on the view direction so there is no UV
  // pole pinch and no equirectangular seam at the meridian.
  vec3 starTier(vec3 d, float cellSize, float threshold, float briMul, float t, float seedOff) {
    vec3 cell = floor(d * cellSize + seedOff);
    float starHash = hash13(cell + uSeed * 7.31);
    if (starHash < threshold) return vec3(0.0);

    vec3 frac = fract(d * cellSize + seedOff);
    vec3 starOffset = vec3(
      hash13(cell * 1.7 + 11.0),
      hash13(cell * 2.3 + 23.0),
      hash13(cell * 3.1 + 37.0)
    );
    vec3 toStar = frac - starOffset;
    float dist = length(toStar);

    // Star radius in cell-fractional units. Smaller cells → smaller
    // stars on screen; multi-tier coverage handles "varied sizes".
    float baseSize = 0.05 + hash13(cell * 4.7) * 0.10;
    float baseBri  = 0.45 + hash13(cell * 5.7) * 0.55;

    // Per-cell time-driven flicker. Phase + period from cell hash so
    // each star has its own rhythm; persistence threshold lets ~60%
    // of stars stay mostly-on while ~40% strongly fluctuate (fade in
    // and out — the user's "appear/disappear in random places").
    float phase = hash13(cell * 6.1) * 6.2831853;
    float per   = 3.5 + hash13(cell * 7.3) * 9.5;
    float lifeT = sin(t * 6.2831853 / per + phase) * 0.5 + 0.5;
    float persistence = hash13(cell * 9.3);
    if (persistence > 0.45) lifeT = mix(0.55, 1.0, lifeT);
    else                    lifeT = pow(lifeT, 2.2);

    float intensity = smoothstep(baseSize, 0.0, dist) * baseBri * briMul * lifeT;
    return intensity * uStarColor;
  }

  vec3 starField(vec3 d, float t) {
    vec3 col = vec3(0.0);
    // Tier 0: largest cells, brightest, sparsest stars
    col += starTier(d,  28.0, 0.62, 1.00, t,  0.0);
    // Tier 1: medium cells, medium brightness
    col += starTier(d,  62.0, 0.55, 0.65, t, 17.3);
    // Tier 2: smallest cells, dimmest, densest pinpoint dust
    col += starTier(d, 130.0, 0.50, 0.34, t, 41.9);
    return col;
  }

  // ---- Wide atmospheric tint --------------------------------------
  // Reference image #1 (the 2D phase backdrop) has subtle teal-navy
  // depth across the ENTIRE frame — there is no pure-black void
  // anywhere. Implementation: an always-on baseline tint + two
  // low-frequency variation layers at different scales/seeds. Result
  // is a uniform-feeling atmospheric depth with subtle "warmer"
  // regions, no localized cloud features required. The localized
  // nebula clouds below are kept as a sparse decorative top layer
  // for cosmetic peaks.
  vec3 atmospheric(vec3 d) {
    // Very faint universal baseline — keeps the deepest "dark"
    // areas from being mathematically-pure black (so the corners
    // read as deep navy-teal instead of dead canvas). Tuned so
    // baseline-only pixels post-ACES-sRGB land around rgb(2, 13,
    // 15). All visible color comes from the localized nebula
    // regions below — the reference image has true dark corners
    // and a *concentrated* glow zone, NOT a uniform wash.
    return uNebulaSteel * 0.015;
  }

  // ---- Scene-centered gradient ----------------------------------------
  // pow(g,3) concentrates the bright zone so screen corners are dark.
  // Large FBM noise (±0.11) variation on g_raw completely destroys the
  // circular boundary — the transition zone spans ±25° of organic edge
  // so no sphere-cap outline is ever visible.
  vec3 nebulaGradient(vec3 d) {
    vec3 fixedDir = normalize(vec3(0.0, -0.22, -1.0));
    float noise = fbm(d * 1.5) * 0.22 - 0.11;
    float g = pow(max(0.0, dot(d, fixedDir) + noise), 3.0);
    vec3 innerColor = vec3(0.001, 0.020, 0.016);
    vec3 outerColor = vec3(0.001, 0.003, 0.018);
    return mix(outerColor, innerColor, smoothstep(0.3, 0.85, g)) * g;
  }

  void main() {
    vec3 d = normalize(vDir);
    float t = (1.0 - uReducedMotion) * uTimeMs * 0.001;

    vec3 deep = uNebulaDeep;
    vec3 atm  = atmospheric(d);
    vec3 neb  = nebulaGradient(d);
    vec3 stars = starField(d, t);

    vec3 col = deep + atm + neb + stars;

    // Hard clamp below the bloom threshold (0.85) — the dome must
    // never bloom, otherwise its diffuse haze would swallow the sun's
    // halo and break the visual hierarchy.
    col = clamp(col, vec3(0.0), vec3(0.78));

    gl_FragColor = vec4(col, 1.0);
  }
`;

function parseRgb(s: string | undefined, fallback: [number, number, number]): [number, number, number] {
  if (!s) return fallback;
  const m = s.match(/rgb[a]?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/i);
  if (!m) return fallback;
  return [parseInt(m[1], 10) / 255, parseInt(m[2], 10) / 255, parseInt(m[3], 10) / 255];
}

export function buildSkyDome(colorMap: Record<string, string>, cfg: SkyDomeConfig = {}): SkyDomeRefs {
  const radius = cfg.radius ?? 14000;
  const widthSegments = cfg.widthSegments ?? 64;
  const heightSegments = cfg.heightSegments ?? 32;
  const seed = cfg.seed ?? 0xC0FFEE;

  const tealRgb   = parseRgb(colorMap['--nebula-teal'],   [0.25, 0.56, 0.61]);
  const steelRgb  = parseRgb(colorMap['--nebula-steel'],  [0.30, 0.42, 0.56]);
  const violetRgb = parseRgb(colorMap['--nebula-violet'], [0.42, 0.32, 0.49]);
  const deepRgb   = parseRgb(colorMap['--nebula-deep'],   [0.046, 0.071, 0.110]);
  const starRgb   = parseRgb(colorMap['--star-cool-white'], [0.894, 0.918, 0.949]);

  const geo = new THREE.SphereGeometry(radius, widthSegments, heightSegments);

  const mat = new THREE.ShaderMaterial({
    vertexShader: VERT,
    fragmentShader: FRAG,
    side: THREE.BackSide,
    depthWrite: false,
    depthTest: false,
    uniforms: {
      uTimeMs:        { value: 0 },
      uReducedMotion: { value: 0 },
      uSeed:          { value: (seed % 1024) / 1024 * 100 },
      uNebulaTeal:    { value: new THREE.Vector3(...tealRgb) },
      uNebulaSteel:   { value: new THREE.Vector3(...steelRgb) },
      uNebulaViolet:  { value: new THREE.Vector3(...violetRgb) },
      uNebulaDeep:    { value: new THREE.Vector3(...deepRgb) },
      uStarColor:     { value: new THREE.Vector3(...starRgb) },
      // Initial sun direction — overwritten each frame by setSunDir.
      // Default points down-Z so the very first frame (before tick
      // runs) doesn't apply the cone in an arbitrary direction.
      uSunDir:        { value: new THREE.Vector3(0, 0, -1) },
    },
  });

  const mesh = new THREE.Mesh(geo, mat);
  // Draw before everything else so transparent additive stars/halos
  // composite on top correctly.
  mesh.renderOrder = -1000;
  // Layer 0 = bloomable scene; the dome's clamped LDR output will not
  // cross the bloom threshold, so it does not bloom but still composes
  // correctly with the post-process pass.
  mesh.layers.set(0);
  mesh.frustumCulled = false;

  return {
    mesh,
    material: mat,
    setFrame(timeMs, reducedMotion) {
      mat.uniforms.uTimeMs.value = timeMs;
      mat.uniforms.uReducedMotion.value = reducedMotion ? 1.0 : 0.0;
    },
    setSunDir(dir) {
      mat.uniforms.uSunDir.value.copy(dir);
    },
    dispose() {
      geo.dispose();
      mat.dispose();
    },
  };
}
