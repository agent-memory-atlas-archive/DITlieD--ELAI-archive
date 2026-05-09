// three-orbit.ts — galaxy of decomposition nodes (no ring strokes).
//
// Renders a flat Group on the xz-plane (y=0) containing:
//   • central HDR sun (the active concept's core; bloom pass picks it up)
//   • phase nodes — every phase, on the phase ring
//   • step nodes — EVERY step of EVERY phase, on the step ring (density)
//   • baby nodes — EVERY baby of EVERY step, on the baby ring (density)
//   • the active baby gets HDR colour so bloom highlights it
//
// No ring lines. The "rings" emerge implicitly from node positions, the
// way real galaxies do — the spiral arm is only visible because stars
// are arranged in a spiral, not because lines are drawn between them.
//
// The group is FLAT (no rotation). The camera sits above the plane at
// pitch ≈40° looking down — that's what creates the tilted-disc view.
// The user explicitly wants the camera to do the tilting, not the geometry.
//
// Position math matches `OrbitConcept.svelte` (polar + per-id seeded
// jitter using `lcg`, `hashId`, `combineSeeds`) so node positions are
// deterministic per task_id.

import * as THREE from 'three';
import type {
  OrbitFixture,
  ConceptStub,
  PhaseStub,
  StepStub,
  BabyStepStub,
} from '../orbit/types.ts';
import { lcg, hashId, combineSeeds } from '../orbit/lcg.ts';

export interface ThreeOrbitConfig {
  /** Phase / step / baby ring radii in world units. Tuned to look like
   *  the user's reference solar-system: the inner ring (phases) is
   *  ~hugging-the-sun, outer ring (babies) sits well out. */
  R_PHASE: number;
  R_STEP: number;
  R_BABY: number;
  /** Sun (active concept core) world radius. */
  sunRadius: number;
  sunColor: THREE.Color;          // HDR — RGB > 1 trips bloom
  /** Per-tier node radii. */
  phaseNodeRadius: number;
  stepNodeRadius: number;
  babyNodeRadius: number;
  /** Per-status colours (LDR, do not bloom). */
  inactivePhaseColor: THREE.Color;
  inactiveStepColor: THREE.Color;
  inactiveBabyColor: THREE.Color;
  donePhaseColor: THREE.Color;
  doneStepColor: THREE.Color;
  doneBabyColor: THREE.Color;
  activePhaseColor: THREE.Color;
  activeStepColor: THREE.Color;
  /** Active baby is HDR — RGB > 1 trips bloom for the focal point. */
  activeBabyHdrColor: THREE.Color;
  /** Position jitter scales (radial / angular). Match OrbitConcept's
   *  ±14 px radial / ±0.07 rad angular jitter so the layout feels like
   *  a constellation, not a perfect ring. */
  jitterRadius: number;
  jitterTheta: number;
}

export const DEFAULT_ORBIT_CONFIG: ThreeOrbitConfig = {
  // Orbit 1.8× of the previous compact layout: sun's screen-space
  // glow can never reach the orbit area, regardless of zoom. The
  // sun + corona stays bottom-of-frame; phase / step / baby sit
  // clearly above. Combined with the LDR sun + sprite corona below,
  // the sun's "effect" has a defined world-space extent that scales
  // naturally with camera distance (unlike post-process bloom which
  // is screen-space and overlaps the orbit at close zooms).
  R_PHASE: 810,
  R_STEP: 1530,
  R_BABY: 2340,
  // Sun is the biggest "star" in the galaxy — clearly larger than any
  // node. Hierarchy: sun > phase > step > baby (concept > phases >
  // steps > babies in execution-mode terms). Sun is HDR so it blooms
  // via the post-process pass; the wide orbit radii (1.8× compact)
  // keep that bloom from reaching the active branch at any zoom, so
  // the "merge" issue does not return.
  sunRadius: 35,
  sunColor: new THREE.Color(0.92, 1.05, 1.00),
  phaseNodeRadius: 14,
  stepNodeRadius: 10,
  babyNodeRadius: 7,
  // LDR teal palette — kept below the bloom threshold (0.85) so non-
  // active nodes read as visible-but-not-glowing dots. Active phase /
  // step are noticeably brighter; only the active baby (HDR) blooms.
  inactivePhaseColor: new THREE.Color(0.55, 0.78, 0.72),
  inactiveStepColor: new THREE.Color(0.50, 0.72, 0.66),
  inactiveBabyColor: new THREE.Color(0.42, 0.62, 0.58),
  donePhaseColor: new THREE.Color(0.50, 0.74, 0.66),
  doneStepColor: new THREE.Color(0.46, 0.68, 0.60),
  doneBabyColor: new THREE.Color(0.40, 0.60, 0.54),
  activePhaseColor: new THREE.Color(0.62, 0.84, 0.78),
  activeStepColor: new THREE.Color(0.66, 0.84, 0.78),
  // LDR (below bloom threshold). Used to be HDR (1.40, 2.00, 1.75)
  // with post-process bloom as the focal-point glow, but bloom is
  // screen-space and was producing the same merge problem the sun
  // had. Now the active baby gets the same sun-style sprite corona
  // (additive, hardcoded teal in haloSpecsForType / refreshActiveStates),
  // which has a defined world-space extent and can't bleed across
  // the screen.
  activeBabyHdrColor: new THREE.Color(0.84, 0.94, 0.88),
  jitterRadius: 14,
  jitterTheta: 0.07,
};

export interface OrbitNodeRef {
  id: string;
  type: 'phase' | 'step' | 'baby';
  mesh: THREE.Mesh;
  /** Inner halo Sprite (brighter, smaller). Always present; tier-based
   *  base scale + opacity, animated for the active baby. */
  halo: THREE.Sprite;
  haloMat: THREE.SpriteMaterial;
  haloBaseScale: number;
  haloBaseOpacity: number;
  /** Outer halo Sprite (fainter, larger). Replicates the sun's two-
   *  layer corona at smaller scale so each node reads as a mini-sun
   *  rather than a single soft glow lost in the sun's wash. */
  outerHalo: THREE.Sprite;
  outerHaloMat: THREE.SpriteMaterial;
  outerHaloBaseScale: number;
  outerHaloBaseOpacity: number;
  worldX: number;
  worldY: number;
  worldZ: number;
}

/** Sonar-pulse ring rendered flat on the orbit plane around an active
 *  node. Expands outward + fades out, then loops. Two rings per tier are
 *  phase-offset by 0.5 of the period so the pulse feels continuous. */
export interface ActivePulseRef {
  tier: 'phase' | 'step' | 'baby';
  mesh: THREE.Mesh;
  mat: THREE.MeshBasicMaterial;
  /** Start / end world-space radius of the ring's outer edge. */
  rStart: number;
  rEnd: number;
  /** Pulse period in seconds. */
  period: number;
  /** Phase offset within the cycle (0..1). */
  phase: number;
}

export interface ThreeOrbitRefs {
  group: THREE.Group;
  sun: THREE.Mesh;
  /** Two concentric ambient halos around the sun (replicates the SVG
   *  concept-centre's layered halo rings). */
  conceptHalos: THREE.Sprite[];
  nodes: OrbitNodeRef[];
  /** Active-branch curve: sun → active phase → active step → active
   *  baby. Curved Catmull-Rom with a flowing-dash shader for the
   *  "energy line" feel. */
  activeLine: THREE.Line | null;
  activeLineMat: THREE.ShaderMaterial | null;
  /** Faint Catmull-Rom curves connecting every parent→child pair
   *  (sun→phase, phase→step, step→baby). Replicates the SVG ambient-
   *  line dash-flow feel so the whole tree reads as connected. The
   *  active branch is masked out (rendered by activeLine instead). */
  ambientLines: THREE.LineSegments | null;
  ambientLinesMat: THREE.ShaderMaterial | null;
  /** Sonar-pulse rings around active nodes (one pair per tier). Lie
   *  flat on the xz plane so they read as "radar from above". */
  activePulses: ActivePulseRef[];
  dispose(): void;
  /** Rebuild active-state highlighting + active-branch curve. */
  refreshActive(fixture: OrbitFixture): void;
  /** Per-frame uniform update for the dash-flow shader. */
  updateAnimation(elapsedMs: number): void;
}

// Canonical "active branch" anchor angle. With polar3D's z-flip below,
// theta = π/2 sends the active phase/step/baby to NEGATIVE z (away from
// camera), giving the "going forward into the distance" feel. Every
// fixture's active branch must land here regardless of its phase index;
// see ACTIVE_BRANCH_THETA invariant in build path.
//
// Bug history: previously phaseAngle(i, n) = -π/2 + 2π·i/n, so a
// fixture with active phase index 0 (e.g. ORBIT_FIXTURE_SMALL — "task")
// landed the active phase at theta=-π/2 (POSITIVE z, toward camera).
// At baby zoom the chase-cam then sat at sz + 400 ≈ +2740 looking back
// at sz - 350 ≈ +1990, putting the sun (z=0) ~2740 units BEHIND the
// camera — exactly the "sun is far away at max zoom" report. Anchoring
// the active branch to a canonical angle removes the index dependency.
const ACTIVE_BRANCH_THETA = Math.PI / 2;

// Legacy index-based angle math — retained inline only as a reference for
// the bug-history comment above. Active-branch positioning now uses
// ACTIVE_BRANCH_THETA directly. If sibling phases / steps / babies are
// ever rendered alongside the active branch, reintroduce phaseAngle /
// stepAngleFor / babyAngleFor here, but keep the active-branch path
// pinned to ACTIVE_BRANCH_THETA so the chase camera invariant holds.

// Polar → 3D world on the xz plane (y=0). We invert the z mapping so
// SVG's "bottom" (theta=+π/2) lands at -z in world space — i.e. AWAY
// from the camera. With the camera positioned high and looking forward
// at lookAt (0, 0, -380), the active branch (typically at theta ≈ π/2
// in our fixtures) recedes INTO the distance, which is the "going
// forward" effect the user described.
function polar3D(theta: number, radius: number): [number, number, number] {
  return [Math.cos(theta) * radius, 0, -Math.sin(theta) * radius];
}

// Per-id seeded jitter pair, matching OrbitConcept.
function jitterFor(rootSeed: number, id: string, jitterR: number, jitterT: number): { dr: number; dt: number } {
  const rng = lcg(combineSeeds(rootSeed, hashId(id)));
  const jr = (rng() - 0.5) * 2; // [-1, 1]
  const jt = (rng() - 0.5) * 2;
  return { dr: jr * jitterR, dt: jt * jitterT };
}

function clearGroup(g: THREE.Group): void {
  while (g.children.length > 0) {
    const c = g.children[0];
    g.remove(c);
    if ((c as THREE.Mesh).geometry) (c as THREE.Mesh).geometry.dispose();
    const m = (c as THREE.Mesh).material;
    if (m) {
      if (Array.isArray(m)) for (const mm of m) mm.dispose();
      else m.dispose();
    }
  }
}

// Procedural radial-gradient texture used for node halos. One canvas
// shared across all halo Sprites; per-node colour + opacity come from
// the SpriteMaterial.
let _haloTexture: THREE.CanvasTexture | null = null;
function getHaloTexture(): THREE.CanvasTexture {
  if (_haloTexture) return _haloTexture;
  if (typeof document === 'undefined') {
    // Server-side / test fallback — return a 1x1 canvas.
    const fake = new THREE.DataTexture(new Uint8Array([255, 255, 255, 255]), 1, 1);
    fake.needsUpdate = true;
    _haloTexture = fake as unknown as THREE.CanvasTexture;
    return _haloTexture;
  }
  const SIZE = 128;
  const canvas = document.createElement('canvas');
  canvas.width = SIZE;
  canvas.height = SIZE;
  const ctx = canvas.getContext('2d');
  if (!ctx) {
    const fake = new THREE.DataTexture(new Uint8Array([255, 255, 255, 255]), 1, 1);
    fake.needsUpdate = true;
    _haloTexture = fake as unknown as THREE.CanvasTexture;
    return _haloTexture;
  }
  // Soft radial gradient — bright core, smooth falloff. Additive blend
  // means the brightness multiplies; a wide gentle gradient gives the
  // "glow" rather than a hard-edged halo.
  const cx = SIZE / 2, cy = SIZE / 2;
  const grd = ctx.createRadialGradient(cx, cy, 0, cx, cy, SIZE / 2);
  grd.addColorStop(0.00, 'rgba(255,255,255,1.0)');
  grd.addColorStop(0.10, 'rgba(255,255,255,0.85)');
  grd.addColorStop(0.30, 'rgba(255,255,255,0.40)');
  grd.addColorStop(0.55, 'rgba(255,255,255,0.16)');
  grd.addColorStop(0.80, 'rgba(255,255,255,0.04)');
  grd.addColorStop(1.00, 'rgba(255,255,255,0.00)');
  ctx.fillStyle = grd;
  ctx.fillRect(0, 0, SIZE, SIZE);
  _haloTexture = new THREE.CanvasTexture(canvas);
  _haloTexture.colorSpace = THREE.SRGBColorSpace;
  return _haloTexture;
}

interface HaloSpec {
  scale: number;
  opacity: number;
  color: THREE.Color;
}

// Layer used by the orbit overlay pass (nodes, node halos, active line,
// active pulses). The render pipeline renders layer 0 first (sun +
// stars + concept halos) and runs bloom on it, then renders layer 1
// objects on top with NO bloom and NORMAL blending — so the sun's
// bloom does not pile additively into the node halos.
export const ORBIT_OVERLAY_LAYER = 1;

function makeHalo(
  spec: HaloSpec,
  blending: THREE.Blending = THREE.AdditiveBlending,
): { sprite: THREE.Sprite; mat: THREE.SpriteMaterial } {
  const mat = new THREE.SpriteMaterial({
    map: getHaloTexture(),
    color: spec.color,
    transparent: true,
    opacity: spec.opacity,
    // All node halos use AdditiveBlending — sun-style corona on every
    // node. Safe from the previous "merge with sun bloom" issue
    // because the sun is now LDR (no bloom) and the orbit radii are
    // wide enough that node halos don't overlap each other on screen.
    blending,
    depthWrite: false,
  });
  const sprite = new THREE.Sprite(mat);
  sprite.scale.set(spec.scale, spec.scale, 1);
  return { sprite, mat };
}

// Pretty teal/aqua/jade palette — every halo color comes from here so
// the constellation stays in the same cool family. Per-node id picks
// one variant deterministically — different active subjects get
// different tints, so e.g. "active step A" and "active step B"
// don't look identical.
const HALO_PALETTE: Array<{ inner: THREE.Color; outer: THREE.Color }> = [
  { inner: new THREE.Color(0.40, 0.95, 0.82), outer: new THREE.Color(0.25, 0.78, 0.65) }, // teal (sun default)
  { inner: new THREE.Color(0.40, 0.85, 0.95), outer: new THREE.Color(0.25, 0.65, 0.82) }, // sea-blue
  { inner: new THREE.Color(0.55, 0.95, 0.75), outer: new THREE.Color(0.35, 0.78, 0.58) }, // mint
  { inner: new THREE.Color(0.50, 0.92, 0.88), outer: new THREE.Color(0.30, 0.74, 0.72) }, // pale aqua
  { inner: new THREE.Color(0.55, 0.92, 0.70), outer: new THREE.Color(0.35, 0.78, 0.52) }, // soft jade
  { inner: new THREE.Color(0.55, 0.78, 0.95), outer: new THREE.Color(0.35, 0.55, 0.82) }, // periwinkle
  { inner: new THREE.Color(0.45, 0.98, 0.92), outer: new THREE.Color(0.28, 0.82, 0.78) }, // ice
];

interface HaloVariation {
  paletteIdx: number;
  scaleJitter: number;     // 0.90 - 1.10 — per-node scale wobble
  opacityJitter: number;   // 0.85 - 1.15 — per-node opacity wobble
}

function haloVariationFor(id: string): HaloVariation {
  const rng = lcg(hashId(id));
  return {
    paletteIdx: Math.floor(rng() * HALO_PALETTE.length),
    scaleJitter: 0.90 + rng() * 0.20,
    opacityJitter: 0.85 + rng() * 0.30,
  };
}

// Per-tier halo defaults. Two layers per node — inner (bright/tight)
// and outer (faint/wide) — mimicking the sun's corona at smaller
// scale so each node reads as a mini-sun. Sun's two layers are
// 60/32 (outer/inner); these all stay BELOW that so the sun remains
// the dominant light source. Importance scales: phase > step > baby.
function haloSpecsForType(
  type: 'phase' | 'step' | 'baby',
  id: string,
): { inner: HaloSpec; outer: HaloSpec } {
  // Per-id variation: palette pick + scale/opacity jitter. Spec colors
  // come from the palette, NOT the mesh color — the corona is part of
  // the node's identity, decoupled from the active-state mesh tint.
  const v = haloVariationFor(id);
  const p = HALO_PALETTE[v.paletteIdx];
  let innerScale: number, outerScale: number, innerOp: number, outerOp: number;
  if (type === 'phase') {
    // Bumped 30/56 → 50/90 so the active phase has a visible aura
    // matching the baby's (additive sprite corona, palette-tinted).
    // Active boost (1.6×) lifts to 80/144 — clearly smaller than the
    // sun's 200 outer corona, clearly larger than step's.
    innerScale = 50; outerScale = 90;
    innerOp = 0.50; outerOp = 0.20;
  } else if (type === 'step') {
    // Bumped 24/46 → 40/75 — visible aura. Active boost (1.7×) →
    // 68/127.5.
    innerScale = 40; outerScale = 75;
    innerOp = 0.45; outerOp = 0.18;
  } else {
    // Baby — focal point. Active boost (2×) → 100/180.
    innerScale = 50; outerScale = 90;
    innerOp = 0.55; outerOp = 0.30;
  }
  return {
    inner: {
      scale: innerScale * v.scaleJitter,
      opacity: innerOp * v.opacityJitter,
      color: p.inner,
    },
    outer: {
      scale: outerScale * v.scaleJitter,
      opacity: outerOp * v.opacityJitter,
      color: p.outer,
    },
  };
}

export function buildThreeOrbit(
  fixture: OrbitFixture,
  config: ThreeOrbitConfig = DEFAULT_ORBIT_CONFIG,
): ThreeOrbitRefs {
  // Group is flat. The camera (pitched above the plane) does the
  // tilting — see three-scene.ts cameraPitchDegrees.
  const group = new THREE.Group();

  // ----- sun -----
  const sunGeo = new THREE.SphereGeometry(config.sunRadius, 32, 32);
  const sunMat = new THREE.MeshBasicMaterial({ color: config.sunColor, transparent: false });
  const sun = new THREE.Mesh(sunGeo, sunMat);
  sun.position.set(0, 0, 0);
  group.add(sun);

  // ----- nodes -----
  const nodes: OrbitNodeRef[] = [];

  const activeConcept: ConceptStub | undefined =
    fixture.concepts.find((c) => c.id === fixture.active_concept_id) ?? fixture.concepts[0];
  if (!activeConcept) {
    // No concept yet — render only the sun + concept halos. The disc
    // is empty until the fixture lands, which is the honest state.
    return makeRefs(group, sun, nodes, null, sunGeo, sunMat, config, fixture);
  }

  const rootSeed = combineSeeds(hashId(fixture.task_id), hashId(activeConcept.id));
  const phases = activeConcept.phases;

  // Phase nodes — only the ACTIVE phase. Inactive phases stay hidden
  // until elai gets to them. The sun + active phase + active step +
  // active baby + flowing line is the entire visible decomposition;
  // everything else is future work and would just be noise.
  //
  // Active-branch anchoring: the active phase is placed at the canonical
  // ACTIVE_BRANCH_THETA (π/2) instead of phaseAngle(activeIdx, count).
  // Reason: only the active branch is rendered (siblings are hidden),
  // so phase index relative to siblings has no visual purpose, but the
  // chase camera (StarCanvas resolveCameraTargets) is tuned for the
  // active branch sitting at -z (south of origin). With phaseAngle()
  // the active phase landed at +z whenever its index was 0, leaving the
  // sun behind the camera at deep zoom. ACTIVE_BRANCH_THETA pins all
  // active branches to the same canonical world quadrant; the existing
  // jitter (id-seeded) preserves layout uniqueness.
  const phasePositions = new Map<string, THREE.Vector3>();
  for (let i = 0; i < phases.length; i++) {
    const p = phases[i];
    if (p.id !== fixture.active_phase_id) continue;
    const theta = ACTIVE_BRANCH_THETA;
    const j = jitterFor(rootSeed, p.id, config.jitterRadius, config.jitterTheta);
    const [x, y, z] = polar3D(theta + j.dt, config.R_PHASE + j.dr);
    const node = makeNodeMesh(config.phaseNodeRadius, config.inactivePhaseColor);
    node.position.set(x, y, z);
    node.layers.set(ORBIT_OVERLAY_LAYER);
    group.add(node);
    const specs = haloSpecsForType('phase', p.id);
    const inner = makeHalo(specs.inner);
    const outer = makeHalo(specs.outer);
    inner.sprite.position.set(x, y, z);
    outer.sprite.position.set(x, y, z);
    inner.sprite.layers.set(ORBIT_OVERLAY_LAYER);
    outer.sprite.layers.set(ORBIT_OVERLAY_LAYER);
    // Add outer first so the brighter inner draws on top.
    group.add(outer.sprite);
    group.add(inner.sprite);
    nodes.push({
      id: p.id, type: 'phase', mesh: node, worldX: x, worldY: y, worldZ: z,
      halo: inner.sprite, haloMat: inner.mat,
      haloBaseScale: specs.inner.scale, haloBaseOpacity: specs.inner.opacity,
      outerHalo: outer.sprite, outerHaloMat: outer.mat,
      outerHaloBaseScale: specs.outer.scale, outerHaloBaseOpacity: specs.outer.opacity,
    });
    phasePositions.set(p.id, new THREE.Vector3(x, y, z));
  }

  // Step + baby nodes — only the ACTIVE step and ACTIVE baby. Sibling
  // steps/babies stay hidden; they're future work, not "useless dots
  // floating in space". The flowing line connects sun → active phase
  // → active step → active baby. That's the whole picture.
  //
  // Both step and baby are anchored to ACTIVE_BRANCH_THETA (same as the
  // active phase) so the entire active branch is collinear with the
  // sun → -z axis. Without this, stepAngleFor/babyAngleFor would inherit
  // the offending phaseAngle() base and re-introduce the +z drift in
  // single-step-per-phase fixtures (the early-return when stepCount<=1
  // returned baseTheta verbatim, which was -π/2 for the "task" fixture).
  const stepPositions = new Map<string, THREE.Vector3>();
  const babyPositions = new Map<string, THREE.Vector3>();
  for (let pi = 0; pi < phases.length; pi++) {
    const phase = phases[pi];
    if (phase.id !== fixture.active_phase_id) continue;
    const steps = phase.steps;
    for (let si = 0; si < steps.length; si++) {
      if (steps[si].id !== fixture.active_step_id) continue;
      const s = steps[si];
      const theta = ACTIVE_BRANCH_THETA;
      const j = jitterFor(rootSeed, s.id, config.jitterRadius, config.jitterTheta);
      const [x, y, z] = polar3D(theta + j.dt, config.R_STEP + j.dr);
      const node = makeNodeMesh(config.stepNodeRadius, config.inactiveStepColor);
      node.position.set(x, y, z);
      node.layers.set(ORBIT_OVERLAY_LAYER);
      group.add(node);
      const sSpecs = haloSpecsForType('step', s.id);
      const sInner = makeHalo(sSpecs.inner);
      const sOuter = makeHalo(sSpecs.outer);
      sInner.sprite.position.set(x, y, z);
      sOuter.sprite.position.set(x, y, z);
      sInner.sprite.layers.set(ORBIT_OVERLAY_LAYER);
      sOuter.sprite.layers.set(ORBIT_OVERLAY_LAYER);
      group.add(sOuter.sprite);
      group.add(sInner.sprite);
      nodes.push({
        id: s.id, type: 'step', mesh: node, worldX: x, worldY: y, worldZ: z,
        halo: sInner.sprite, haloMat: sInner.mat,
        haloBaseScale: sSpecs.inner.scale, haloBaseOpacity: sSpecs.inner.opacity,
        outerHalo: sOuter.sprite, outerHaloMat: sOuter.mat,
        outerHaloBaseScale: sSpecs.outer.scale, outerHaloBaseOpacity: sSpecs.outer.opacity,
      });
      stepPositions.set(s.id, new THREE.Vector3(x, y, z));

      // Baby nodes — only the ACTIVE baby. Sibling babies stay
      // hidden; they're upcoming work, not current.
      const babies = s.baby_steps.filter((b) => b.id === fixture.active_baby_step_id);
      for (let bi = 0; bi < babies.length; bi++) {
        const b = babies[bi];
        const btheta = ACTIVE_BRANCH_THETA;
        const bj = jitterFor(rootSeed, b.id, config.jitterRadius, config.jitterTheta);
        const [bx, by, bz] = polar3D(btheta + bj.dt, config.R_BABY + bj.dr);
        const bnode = makeNodeMesh(config.babyNodeRadius, config.inactiveBabyColor);
        bnode.position.set(bx, by, bz);
        // Baby is LDR — no bloom contribution. Overlay layer only
        // (its glow comes from the additive sprite corona, not the
        // post-process bloom pass).
        bnode.layers.set(ORBIT_OVERLAY_LAYER);
        group.add(bnode);
        const bSpecs = haloSpecsForType('baby', b.id);
        const bInner = makeHalo(bSpecs.inner);
        const bOuter = makeHalo(bSpecs.outer);
        bInner.sprite.position.set(bx, by, bz);
        bOuter.sprite.position.set(bx, by, bz);
        bInner.sprite.layers.set(ORBIT_OVERLAY_LAYER);
        bOuter.sprite.layers.set(ORBIT_OVERLAY_LAYER);
        group.add(bOuter.sprite);
        group.add(bInner.sprite);
        nodes.push({
          id: b.id, type: 'baby', mesh: bnode, worldX: bx, worldY: by, worldZ: bz,
          halo: bInner.sprite, haloMat: bInner.mat,
          haloBaseScale: bSpecs.inner.scale, haloBaseOpacity: bSpecs.inner.opacity,
          outerHalo: bOuter.sprite, outerHaloMat: bOuter.mat,
          outerHaloBaseScale: bSpecs.outer.scale, outerHaloBaseOpacity: bSpecs.outer.opacity,
        });
        babyPositions.set(b.id, new THREE.Vector3(bx, by, bz));
      }
    }
  }

  // No ambient connection edges. Lines = current-task trail only;
  // inactive nodes float as dots until elai gets to them. The
  // bright active-line shader (sun → active phase → active step →
  // active baby) is the only line in the scene.
  return makeRefs(group, sun, nodes, null, sunGeo, sunMat, config, fixture);
}

/** Faint Catmull-Rom curves for every parent→child pair in the active
 *  concept, sampled into LineSegments. The fragment shader runs a slow
 *  ambient dash flow keyed by per-vertex `aT`. Active branch (sun →
 *  active phase → active step → active baby) is excluded — the
 *  brighter activeLine shader renders that. */
const AMBIENT_SEGMENTS_PER_EDGE = 14;
function buildAmbientConnections(
  concept: ConceptStub,
  phasePositions: Map<string, THREE.Vector3>,
  stepPositions: Map<string, THREE.Vector3>,
  babyPositions: Map<string, THREE.Vector3>,
): { mesh: THREE.LineSegments; mat: THREE.ShaderMaterial } | null {
  // Collect all (a, b) pairs across the tree. Active branch IS included
  // (the active-line shader will overdraw its segments brighter).
  type Edge = { a: THREE.Vector3; b: THREE.Vector3 };
  const edges: Edge[] = [];
  const sun = new THREE.Vector3(0, 0, 0);
  for (const phase of concept.phases) {
    const pPos = phasePositions.get(phase.id);
    if (!pPos) continue;
    edges.push({ a: sun, b: pPos });
    for (const step of phase.steps) {
      const sPos = stepPositions.get(step.id);
      if (!sPos) continue;
      edges.push({ a: pPos, b: sPos });
      for (const baby of step.baby_steps) {
        const bPos = babyPositions.get(baby.id);
        if (!bPos) continue;
        edges.push({ a: sPos, b: bPos });
      }
    }
  }
  if (edges.length === 0) return null;

  const segPerEdge = AMBIENT_SEGMENTS_PER_EDGE;
  const vertsPerEdge = segPerEdge * 2; // LineSegments takes pairs
  const totalVerts = edges.length * vertsPerEdge;
  const positions = new Float32Array(totalVerts * 3);
  const ts = new Float32Array(totalVerts);

  const mid = new THREE.Vector3();
  const ctrl = new THREE.Vector3();
  for (let e = 0; e < edges.length; e++) {
    const { a, b } = edges[e];
    // Bow the curve toward the sun a little so edges arc gracefully
    // instead of cutting straight across the disc. Magnitude is a
    // fraction of the edge length so short edges stay tight.
    mid.lerpVectors(a, b, 0.5);
    const bowAmount = 0.18;
    ctrl.copy(mid).multiplyScalar(1 - bowAmount); // pull toward (0,0,0)
    const curve = new THREE.CatmullRomCurve3([a, ctrl, b], false, 'catmullrom', 0.5);
    const samples = curve.getPoints(segPerEdge); // length = segPerEdge + 1
    // Emit (0,1)(1,2)(2,3)... pairs.
    for (let s = 0; s < segPerEdge; s++) {
      const p0 = samples[s];
      const p1 = samples[s + 1];
      const baseIdx = (e * vertsPerEdge + s * 2);
      positions[baseIdx * 3] = p0.x;
      positions[baseIdx * 3 + 1] = p0.y;
      positions[baseIdx * 3 + 2] = p0.z;
      positions[(baseIdx + 1) * 3] = p1.x;
      positions[(baseIdx + 1) * 3 + 1] = p1.y;
      positions[(baseIdx + 1) * 3 + 2] = p1.z;
      const t0 = s / segPerEdge;
      const t1 = (s + 1) / segPerEdge;
      ts[baseIdx] = t0;
      ts[baseIdx + 1] = t1;
    }
  }

  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  geo.setAttribute('aT', new THREE.BufferAttribute(ts, 1));

  const mat = new THREE.ShaderMaterial({
    transparent: true,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
    uniforms: {
      uTime: { value: 0 },
      uColor: { value: new THREE.Color(0.30, 0.62, 0.55) },
      uOpacity: { value: 0.42 },
      uDashSpeed: { value: 0.14 }, // ~4x slower than active line
    },
    vertexShader: /* glsl */ `
      attribute float aT;
      varying float vT;
      void main() {
        vT = aT;
        gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
      }
    `,
    fragmentShader: /* glsl */ `
      precision highp float;
      uniform float uTime;
      uniform vec3  uColor;
      uniform float uOpacity;
      uniform float uDashSpeed;
      varying float vT;
      void main() {
        // Gentle dash flow: a soft baseline + a low-amplitude moving
        // bump per cycle. Amplitude stays small so inactive edges read
        // as background structure, not bright connectors.
        float u = vT * 4.0 - uTime * uDashSpeed;
        float bump = exp(-pow(fract(u) - 0.50, 2.0) * 22.0) * 0.45;
        float amount = 0.55 + bump;
        gl_FragColor = vec4(uColor, uOpacity * amount);
      }
    `,
  });

  const mesh = new THREE.LineSegments(geo, mat);
  mesh.frustumCulled = false;
  return { mesh, mat };
}

/** Build a flat ring (lies on the xz plane after rotation). Inner /
 *  outer radii are 0.85 / 1.0 so scaling the mesh by R_world gives a
 *  ring of thickness 15% R_world — thin enough to read as "sonar
 *  pulse", thick enough to stay visible at fast scale changes. */
function makePulseRingMesh(color: THREE.Color): { mesh: THREE.Mesh; mat: THREE.MeshBasicMaterial } {
  const geo = new THREE.RingGeometry(0.85, 1.0, 64);
  // RingGeometry sits on the xy plane by default; rotate -90° around X
  // so it lies flat on xz (the orbit plane).
  geo.rotateX(-Math.PI / 2);
  const mat = new THREE.MeshBasicMaterial({
    color,
    transparent: true,
    opacity: 0.0, // driven each frame by updateAnimation
    side: THREE.DoubleSide,
    depthWrite: false,
    // AdditiveBlending — sun is LDR (no bloom), so pulses adding
    // brightness on top of the dark bg / corona just gives them a
    // glowy radar look without re-introducing any merge.
    blending: THREE.AdditiveBlending,
  });
  const mesh = new THREE.Mesh(geo, mat);
  mesh.visible = false;
  mesh.layers.set(ORBIT_OVERLAY_LAYER);
  return { mesh, mat };
}

/** Build the 6 sonar pulses (2 per tier, phase-offset 0.5 of period
 *  for continuous feel). Returns refs for update/dispose. */
function buildActivePulses(): ActivePulseRef[] {
  const pulses: ActivePulseRef[] = [];
  const tierColor: Record<'phase' | 'step' | 'baby', THREE.Color> = {
    phase: new THREE.Color(0.45, 0.85, 0.78),
    step: new THREE.Color(0.50, 0.92, 0.82),
    baby: new THREE.Color(0.60, 1.00, 0.90),
  };
  // Tier-tuned start/end radii + period. Smaller than before so the
  // pulse doesn't dominate the screen when the camera is close.
  const tierSpec: Record<'phase' | 'step' | 'baby', { rStart: number; rEnd: number; period: number }> = {
    phase: { rStart: 18, rEnd: 50, period: 3.0 },
    step:  { rStart: 14, rEnd: 38, period: 2.6 },
    baby:  { rStart: 11, rEnd: 30, period: 2.0 },
  };
  for (const tier of ['phase', 'step', 'baby'] as const) {
    const spec = tierSpec[tier];
    for (let i = 0; i < 2; i++) {
      const { mesh, mat } = makePulseRingMesh(tierColor[tier]);
      pulses.push({
        tier, mesh, mat,
        rStart: spec.rStart, rEnd: spec.rEnd,
        period: spec.period,
        phase: i * 0.5,
      });
    }
  }
  return pulses;
}

function makeNodeMesh(radius: number, color: THREE.Color): THREE.Mesh {
  const geo = new THREE.SphereGeometry(radius, 16, 16);
  const mat = new THREE.MeshBasicMaterial({ color, transparent: false });
  return new THREE.Mesh(geo, mat);
}

function refreshActiveStates(refs: ThreeOrbitRefs, fixture: OrbitFixture, config: ThreeOrbitConfig): void {
  const apId = fixture.active_phase_id;
  const asId = fixture.active_step_id;
  const abId = fixture.active_baby_step_id;
  for (const n of refs.nodes) {
    const mat = n.mesh.material as THREE.MeshBasicMaterial;
    let isActive = false;
    if (n.type === 'phase') {
      isActive = n.id === apId;
      mat.color = isActive ? config.activePhaseColor : config.inactivePhaseColor;
    } else if (n.type === 'step') {
      isActive = n.id === asId;
      mat.color = isActive ? config.activeStepColor : config.inactiveStepColor;
    } else {
      isActive = n.id === abId;
      mat.color = isActive ? config.activeBabyHdrColor : config.inactiveBabyColor;
    }
    mat.needsUpdate = true;

    // Halo: bigger + more opaque on active nodes; baby active gets the
    // breath animation (handled in the rAF tick via the refs). Two
    // halos per node (inner bright, outer faint) — both get the active
    // boost so the mini-sun corona reads at the right intensity.
    // Halo colors are fixed at sprite-creation time from the palette
    // (per node id). DON'T sync from mesh color — palette is the
    // node's identity. The active-state highlight is carried by the
    // mesh color + scale boost, not the halo tint.
    const specs = haloSpecsForType(n.type, n.id);
    if (isActive) {
      const scaleBoost = n.type === 'phase' ? 1.6 : n.type === 'step' ? 1.7 : 2.0;
      // Phase / step boost opacity so they pop more on active.
      // Baby's base opacity already matches the sun corona — leaving
      // it un-boosted keeps baby from outshining the sun.
      const opacityBoost = n.type === 'baby' ? 1.0 : 1.7;
      n.haloBaseScale = specs.inner.scale * scaleBoost;
      n.haloBaseOpacity = Math.min(1.0, specs.inner.opacity * opacityBoost);
      n.outerHaloBaseScale = specs.outer.scale * scaleBoost;
      n.outerHaloBaseOpacity = Math.min(1.0, specs.outer.opacity * opacityBoost);
    } else {
      n.haloBaseScale = specs.inner.scale;
      n.haloBaseOpacity = specs.inner.opacity;
      n.outerHaloBaseScale = specs.outer.scale;
      n.outerHaloBaseOpacity = specs.outer.opacity;
    }
    n.halo.scale.set(n.haloBaseScale, n.haloBaseScale, 1);
    n.haloMat.opacity = n.haloBaseOpacity;
    n.haloMat.needsUpdate = true;
    n.outerHalo.scale.set(n.outerHaloBaseScale, n.outerHaloBaseScale, 1);
    n.outerHaloMat.opacity = n.outerHaloBaseOpacity;
    n.outerHaloMat.needsUpdate = true;
  }
  rebuildActiveLine(refs, fixture);
  positionActivePulses(refs, fixture);
}

function positionActivePulses(refs: ThreeOrbitRefs, fixture: OrbitFixture): void {
  // Resolve target world positions for each tier from the active
  // phase / step / baby. If a tier has no active node yet, hide its
  // pulses so we don't ghost a stale position.
  const targets: Record<'phase' | 'step' | 'baby', THREE.Vector3 | null> = {
    phase: null, step: null, baby: null,
  };
  const apId = fixture.active_phase_id;
  const asId = fixture.active_step_id;
  const abId = fixture.active_baby_step_id;
  for (const n of refs.nodes) {
    if (n.type === 'phase' && n.id === apId) {
      targets.phase = new THREE.Vector3(n.worldX, n.worldY, n.worldZ);
    } else if (n.type === 'step' && n.id === asId) {
      targets.step = new THREE.Vector3(n.worldX, n.worldY, n.worldZ);
    } else if (n.type === 'baby' && n.id === abId) {
      targets.baby = new THREE.Vector3(n.worldX, n.worldY, n.worldZ);
    }
  }
  for (const p of refs.activePulses) {
    const t = targets[p.tier];
    if (!t) {
      p.mesh.visible = false;
      continue;
    }
    p.mesh.position.copy(t);
    p.mesh.visible = true;
  }
}

// Build / rebuild the active-branch curve. We sample a Catmull-Rom
// curve through (sun, active phase, active step, active baby) so the
// connection feels organic instead of zig-zaggy. Each vertex carries a
// `t` attribute (0..1 along the curve) which the shader uses to compute
// the flowing-dash animation.
const ACTIVE_CURVE_SEGMENTS = 128;
function rebuildActiveLine(refs: ThreeOrbitRefs, fixture: OrbitFixture): void {
  const apId = fixture.active_phase_id;
  const asId = fixture.active_step_id;
  const abId = fixture.active_baby_step_id;
  const phase = refs.nodes.find((n) => n.type === 'phase' && n.id === apId);
  const step = refs.nodes.find((n) => n.type === 'step' && n.id === asId);
  const baby = refs.nodes.find((n) => n.type === 'baby' && n.id === abId);

  const anchors: THREE.Vector3[] = [new THREE.Vector3(0, 0, 0)];
  if (phase) anchors.push(new THREE.Vector3(phase.worldX, phase.worldY, phase.worldZ));
  if (step) anchors.push(new THREE.Vector3(step.worldX, step.worldY, step.worldZ));
  if (baby) anchors.push(new THREE.Vector3(baby.worldX, baby.worldY, baby.worldZ));

  if (!refs.activeLine || anchors.length < 2) return;

  // Catmull-Rom with low tension produces a gentle through-curve.
  const curve = new THREE.CatmullRomCurve3(anchors, false, 'catmullrom', 0.35);
  const samples = curve.getPoints(ACTIVE_CURVE_SEGMENTS);

  const positions = new Float32Array(samples.length * 3);
  const ts = new Float32Array(samples.length);
  for (let i = 0; i < samples.length; i++) {
    positions[i * 3] = samples[i].x;
    positions[i * 3 + 1] = samples[i].y;
    positions[i * 3 + 2] = samples[i].z;
    ts[i] = i / Math.max(1, samples.length - 1);
  }

  refs.activeLine.geometry.dispose();
  const geom = new THREE.BufferGeometry();
  geom.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  geom.setAttribute('aT', new THREE.BufferAttribute(ts, 1));
  refs.activeLine.geometry = geom;
}

const ACTIVE_LINE_VERT = /* glsl */ `
  attribute float aT;
  varying float vT;
  void main() {
    vT = aT;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;
const ACTIVE_LINE_FRAG = /* glsl */ `
  precision highp float;
  uniform float uTime;       // seconds
  uniform vec3  uColor;
  uniform float uOpacity;
  uniform float uDashSpeed;  // dashes per second
  varying float vT;
  void main() {
    // Two layered animated stripes: a slow base glow + a faster bright
    // pulse that flows from sun toward the active baby. The result is
    // a steady teal trail with a moving "energy" highlight.
    float base = 0.35;
    float u = vT * 6.0 - uTime * uDashSpeed;
    float head = exp(-pow(fract(u) - 0.30, 2.0) * 36.0) * 0.85;
    float pulse = base + head;
    gl_FragColor = vec4(uColor, uOpacity * pulse);
  }
`;

// Sun's two-layer additive corona, sized so the sun stays the biggest
// glow even after we bumped phase/step/baby coronas up. Layered with
// the post-process bloom: bloom does some outer spread, sprites add
// the teal accent and extend the corona beyond bloom's reach.
export const CONCEPT_HALO_OUTER_SCALE = 200;
export const CONCEPT_HALO_INNER_SCALE = 120;

function buildConceptHalos(): THREE.Sprite[] {
  // Brighter opacity (0.22/0.42 → 0.30/0.55) — user wants the sun
  // clearly the brightest of the bunch.
  const tex = getHaloTexture();
  const outer = new THREE.SpriteMaterial({
    map: tex,
    color: new THREE.Color(0.25, 0.78, 0.65),
    transparent: true,
    opacity: 0.30,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  const inner = new THREE.SpriteMaterial({
    map: tex,
    color: new THREE.Color(0.40, 0.95, 0.82),
    transparent: true,
    opacity: 0.55,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  const outerS = new THREE.Sprite(outer);
  outerS.scale.set(CONCEPT_HALO_OUTER_SCALE, CONCEPT_HALO_OUTER_SCALE, 1);
  const innerS = new THREE.Sprite(inner);
  innerS.scale.set(CONCEPT_HALO_INNER_SCALE, CONCEPT_HALO_INNER_SCALE, 1);
  return [outerS, innerS];
}

function makeRefs(
  group: THREE.Group,
  sun: THREE.Mesh,
  nodes: OrbitNodeRef[],
  ambient: { mesh: THREE.LineSegments; mat: THREE.ShaderMaterial } | null,
  sunGeo: THREE.BufferGeometry,
  sunMat: THREE.Material,
  config: ThreeOrbitConfig,
  fixture: OrbitFixture,
): ThreeOrbitRefs {
  // Concept halo rings around the sun.
  const conceptHalos = buildConceptHalos();
  for (const h of conceptHalos) group.add(h);

  // Active-branch curve — Catmull-Rom through (sun, phase, step, baby)
  // with a flowing-dash shader. Geometry is rebuilt by rebuildActiveLine
  // on each refreshActive call.
  const lineMat = new THREE.ShaderMaterial({
    vertexShader: ACTIVE_LINE_VERT,
    fragmentShader: ACTIVE_LINE_FRAG,
    transparent: true,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
    uniforms: {
      uTime: { value: 0 },
      uColor: { value: new THREE.Color(0.32, 0.88, 0.74) },
      uOpacity: { value: 0.85 },
      uDashSpeed: { value: 0.55 },
    },
  });
  const lineGeo = new THREE.BufferGeometry();
  const activeLine = new THREE.Line(lineGeo, lineMat);
  activeLine.frustumCulled = false;
  activeLine.layers.set(ORBIT_OVERLAY_LAYER);
  group.add(activeLine);

  // Sonar-pulse rings on active nodes (phase, step, baby). Two rings
  // per tier, phase-offset 0.5 of the period for a continuous radar
  // feel. Position + visibility are wired in refreshActiveStates.
  const activePulses = buildActivePulses();
  for (const p of activePulses) group.add(p.mesh);

  const refs: ThreeOrbitRefs = {
    group,
    sun,
    conceptHalos,
    nodes,
    activeLine,
    activeLineMat: lineMat,
    ambientLines: ambient ? ambient.mesh : null,
    ambientLinesMat: ambient ? ambient.mat : null,
    activePulses,
    refreshActive(fx) {
      refreshActiveStates(refs, fx, config);
    },
    updateAnimation(elapsedMs) {
      const tSec = elapsedMs / 1000;
      lineMat.uniforms.uTime.value = tSec;
      if (refs.ambientLinesMat) refs.ambientLinesMat.uniforms.uTime.value = tSec;
      // Drive each pulse: scale 1→rEnd and opacity 0.7→0 across one
      // period, looping. Two pulses per tier are offset by 0.5 of the
      // cycle so the radar reads as continuous.
      for (const p of refs.activePulses) {
        if (!p.mesh.visible) continue;
        const cycle = ((tSec / p.period) + p.phase) % 1.0;
        const r = p.rStart + (p.rEnd - p.rStart) * cycle;
        p.mesh.scale.set(r, 1, r);
        // Ease the opacity so the pulse fades naturally, not linearly.
        const fade = 1.0 - cycle;
        p.mat.opacity = 0.7 * fade * fade;
      }
    },
    dispose() {
      sunGeo.dispose();
      sunMat.dispose();
      lineMat.dispose();
      if (refs.ambientLinesMat) refs.ambientLinesMat.dispose();
      if (refs.ambientLines) refs.ambientLines.geometry.dispose();
      for (const p of refs.activePulses) {
        p.mesh.geometry.dispose();
        p.mat.dispose();
      }
      for (const h of conceptHalos) (h.material as THREE.SpriteMaterial).dispose();
      clearGroup(group);
    },
  };
  refreshActiveStates(refs, fixture, config);
  return refs;
}
