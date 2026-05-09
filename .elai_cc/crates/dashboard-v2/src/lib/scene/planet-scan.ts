// planet-scan.ts — Saturn-style telemetry rings rendered around the
// active focal node when the OrbitCanvas is at the deepest zoom level
// ('baby-step').
//
// Three concentric ring layers (slightly tilted off the orbital plane):
//   • Ring A (radii 2.5×–2.85× node radius) — progress-fraction arc.
//   • Ring B (radii 3.0×–3.30×)             — N evidence-gate segments,
//                                             teal=pass, dim=pending.
//   • Ring C (radii 3.5×–3.68×)             — slow rotating thin band
//                                             (1 full rotation / 60 s).
//
// Glyph billboards were removed (G-orbit overlay G7+): troika-three-text
// labels orbiting at radii 3.0/3.4/3.8× were too small to read at any
// zoom and competed with the new NodePanel.svelte DOM overlay (which
// owns the textual explanation). Rings now carry visual progress only;
// the panel carries text. `defaultGlyphsFor` is kept exported for
// callers that still want the text bullets, but PlanetScan no longer
// renders them in 3D.
//
// All time-based smoothing uses `factor = 1 - exp(-rate * dt)` so the
// motion is monitor-refresh-rate-independent (matches StarCanvas's chase
// camera).
//
// Lifecycle: PlanetScan owns its THREE.Group (added to the scene by the
// caller). Call setVisible(true) to fade in over 1.2s; setVisible(false)
// to fade out over 0.8s. update(focalNode, ...) re-anchors and re-draws
// telemetry; dispose() releases all GPU buffers.
//
// Active-baby radius: when the focal node is the active baby in plan
// mode, we read the radius from the mesh's SphereGeometry parameters.
// Other modes pass the radius explicitly via update().

import * as THREE from 'three';
import { ORBIT_OVERLAY_LAYER } from './three-orbit.ts';
import type { OrbitNodeRef } from './three-orbit.ts';

// Teal palette mirrors three-orbit + three-orbit-modes.
export const TEAL_BRIGHT = new THREE.Color(0.40, 0.95, 0.82);
export const TEAL_MID    = new THREE.Color(0.30, 0.78, 0.68);
export const TEAL_DIM    = new THREE.Color(0.20, 0.45, 0.40);

// Tilt angle of the ring stack off the xz plane (~15°).
const RING_TILT_RAD = (15 * Math.PI) / 180;

// Per-ring rotation rates (rad/s).
const RING_A_RATE = 0.020;
const RING_B_RATE = 0.020;
const RING_C_RATE = (Math.PI * 2) / 60.0;        // 1 rotation / 60 s

// Fade durations.
const FADE_IN_MS  = 1200;
const FADE_OUT_MS = 800;

// Default progress / gate / time-on-step values when not provided by the
// fixture. Keep them visible so the rings still read as decorative.
const DEFAULT_PROGRESS = 0.42;
const DEFAULT_GATE_COUNT = 5;

export interface GateState {
  /** A teal "pass" segment lights bright; "pending" stays dim. */
  passed: boolean;
  /** Optional label rendered as a debug glyph; ignored for ring B. */
  label?: string;
}

export interface PlanetScanInputs {
  /** World-space centre of the focal node (the planet). */
  centre: THREE.Vector3;
  /** Node radius in world units (used to scale every ring + glyph). */
  radius: number;
  /** Progress fraction in [0, 1]. Drives Ring A's filled arc length. */
  progress: number;
  /** Evidence gates — each lit-or-dim segment on Ring B. */
  gates: GateState[];
  /** Glyph labels (3–5 short strings). Ring positions cycle across radii. */
  glyphs: string[];
}

export interface RingMesh {
  mesh: THREE.Mesh;
  geo: THREE.BufferGeometry;
  mat: THREE.MeshBasicMaterial;
}

// Build a partial-arc ring as a manual triangle strip (RingGeometry only
// renders a full ring; THREE.RingGeometry has thetaStart/Length but the
// API is awkward to update each frame and re-allocating geometry per
// progress tick is wasteful — manual triangles let us pre-allocate the
// buffer and just update vertex positions).
//
// Ring A uses this. We pre-allocate `MAX_SEGMENTS * 6` vertices (two
// triangles per segment) and update positions each frame based on
// `progress`. innerR / outerR are set once at build time.
export const ARC_MAX_SEGMENTS = 96;

export function buildArcRingGeometry(innerR: number, outerR: number): THREE.BufferGeometry {
  const verts = new Float32Array(ARC_MAX_SEGMENTS * 6 * 3);
  const cols  = new Float32Array(ARC_MAX_SEGMENTS * 6 * 3);
  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(verts, 3));
  geo.setAttribute('color',    new THREE.BufferAttribute(cols, 3));
  geo.setDrawRange(0, 0);
  // Stash radii as user-data so the update path can rebuild without
  // closure capture from caller scope.
  geo.userData = { innerR, outerR };
  return geo;
}

export function updateArcGeometry(
  geo: THREE.BufferGeometry,
  startTheta: number,
  endTheta: number,
  segmentCount: number,
  color: THREE.Color,
): void {
  const innerR = (geo.userData.innerR as number) ?? 0;
  const outerR = (geo.userData.outerR as number) ?? 0;
  const seg = Math.max(0, Math.min(ARC_MAX_SEGMENTS, segmentCount | 0));
  const posAttr = geo.getAttribute('position') as THREE.BufferAttribute;
  const colAttr = geo.getAttribute('color') as THREE.BufferAttribute;
  const span = endTheta - startTheta;
  for (let i = 0; i < seg; i++) {
    const t0 = startTheta + (span * i) / seg;
    const t1 = startTheta + (span * (i + 1)) / seg;
    const x0i = Math.cos(t0) * innerR, z0i = Math.sin(t0) * innerR;
    const x1i = Math.cos(t1) * innerR, z1i = Math.sin(t1) * innerR;
    const x0o = Math.cos(t0) * outerR, z0o = Math.sin(t0) * outerR;
    const x1o = Math.cos(t1) * outerR, z1o = Math.sin(t1) * outerR;
    // Two triangles per segment: (i,o,o') + (i,o',i').
    const base = i * 6 * 3;
    // tri 1: inner-start, outer-start, outer-end
    posAttr.array[base + 0] = x0i; posAttr.array[base + 1] = 0; posAttr.array[base + 2] = z0i;
    posAttr.array[base + 3] = x0o; posAttr.array[base + 4] = 0; posAttr.array[base + 5] = z0o;
    posAttr.array[base + 6] = x1o; posAttr.array[base + 7] = 0; posAttr.array[base + 8] = z1o;
    // tri 2: inner-start, outer-end, inner-end
    posAttr.array[base + 9]  = x0i; posAttr.array[base + 10] = 0; posAttr.array[base + 11] = z0i;
    posAttr.array[base + 12] = x1o; posAttr.array[base + 13] = 0; posAttr.array[base + 14] = z1o;
    posAttr.array[base + 15] = x1i; posAttr.array[base + 16] = 0; posAttr.array[base + 17] = z1i;
    // Vertex colours (uniform across the segment).
    for (let v = 0; v < 6; v++) {
      const o = base + v * 3;
      colAttr.array[o]     = color.r;
      colAttr.array[o + 1] = color.g;
      colAttr.array[o + 2] = color.b;
    }
  }
  geo.setDrawRange(0, seg * 6);
  posAttr.needsUpdate = true;
  colAttr.needsUpdate = true;
  // Recompute bounds — without this raycaster + frustum-cull go stale.
  geo.computeBoundingSphere();
}

export function makeArcMesh(innerR: number, outerR: number, baseColor: THREE.Color, baseOpacity: number): RingMesh {
  const geo = buildArcRingGeometry(innerR, outerR);
  const mat = new THREE.MeshBasicMaterial({
    color: 0xffffff,             // base white; vertex colour modulates
    vertexColors: true,
    transparent: true,
    opacity: baseOpacity,
    side: THREE.DoubleSide,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });
  const mesh = new THREE.Mesh(geo, mat);
  mesh.frustumCulled = false;
  mesh.layers.set(ORBIT_OVERLAY_LAYER);
  // Stash baseColor for incremental updates (gate ring lights bright/dim).
  mesh.userData = { baseColor: baseColor.clone() };
  return { mesh, geo, mat };
}

/** Update a ring B geometry with N equal "gate" arc segments separated
 *  by small gaps. Each segment is lit (TEAL_BRIGHT) when the gate
 *  passed, otherwise dim (TEAL_DIM). Exported so NodeMandala can reuse
 *  the same buffer-update strategy for its own gate ring. */
export function updateGateSegmentsGeometry(
  geo: THREE.BufferGeometry,
  gates: GateState[],
  defaultGateCount: number = DEFAULT_GATE_COUNT,
): void {
  const N = Math.max(1, Math.min(ARC_MAX_SEGMENTS, gates.length || defaultGateCount));
  const seg = Math.max(N, Math.min(ARC_MAX_SEGMENTS, N * 6));
  const subPerGate = Math.floor(seg / N);
  const gapFrac = 0.10;

  const posAttr = geo.getAttribute('position') as THREE.BufferAttribute;
  const colAttr = geo.getAttribute('color') as THREE.BufferAttribute;
  const innerR = (geo.userData.innerR as number) ?? 0;
  const outerR = (geo.userData.outerR as number) ?? 0;

  let writeIdx = 0;
  for (let g = 0; g < N; g++) {
    const passed = gates[g]?.passed ?? false;
    const c = passed ? TEAL_BRIGHT : TEAL_DIM;
    const tStart = (g / N) * Math.PI * 2 + gapFrac * (Math.PI * 2 / N) * 0.5;
    const tEnd   = ((g + 1) / N) * Math.PI * 2 - gapFrac * (Math.PI * 2 / N) * 0.5;
    const span = tEnd - tStart;
    for (let i = 0; i < subPerGate; i++) {
      const a = tStart + (span * i) / subPerGate;
      const b = tStart + (span * (i + 1)) / subPerGate;
      const xai = Math.cos(a) * innerR, zai = Math.sin(a) * innerR;
      const xbi = Math.cos(b) * innerR, zbi = Math.sin(b) * innerR;
      const xao = Math.cos(a) * outerR, zao = Math.sin(a) * outerR;
      const xbo = Math.cos(b) * outerR, zbo = Math.sin(b) * outerR;
      const base = writeIdx * 6 * 3;
      if (base + 18 > posAttr.array.length) break;
      posAttr.array[base + 0]  = xai; posAttr.array[base + 1]  = 0; posAttr.array[base + 2]  = zai;
      posAttr.array[base + 3]  = xao; posAttr.array[base + 4]  = 0; posAttr.array[base + 5]  = zao;
      posAttr.array[base + 6]  = xbo; posAttr.array[base + 7]  = 0; posAttr.array[base + 8]  = zbo;
      posAttr.array[base + 9]  = xai; posAttr.array[base + 10] = 0; posAttr.array[base + 11] = zai;
      posAttr.array[base + 12] = xbo; posAttr.array[base + 13] = 0; posAttr.array[base + 14] = zbo;
      posAttr.array[base + 15] = xbi; posAttr.array[base + 16] = 0; posAttr.array[base + 17] = zbi;
      for (let v = 0; v < 6; v++) {
        const o = base + v * 3;
        colAttr.array[o]     = c.r;
        colAttr.array[o + 1] = c.g;
        colAttr.array[o + 2] = c.b;
      }
      writeIdx++;
    }
  }
  geo.setDrawRange(0, writeIdx * 6);
  posAttr.needsUpdate = true;
  colAttr.needsUpdate = true;
  geo.computeBoundingSphere();
}

export class PlanetScan {
  /** Public group — caller adds/removes from the scene. */
  readonly group = new THREE.Group();

  // Tilt frame: rings live under this so their normal is ~15° off y-axis.
  private readonly tiltGroup = new THREE.Group();

  private ringA: RingMesh;
  private ringB: RingMesh;
  private ringC: RingMesh;

  // Animation state.
  private startMs = -1;
  private fadeStartMs = -1;
  private fadeFromOpacity = 0;
  private fadeToOpacity = 0;
  private fadeDuration = 0;
  private currentOpacity = 0;
  private visible = false;

  // Cached inputs from the most recent update() call.
  private radius = 0;
  private progress = 0;
  private gates: GateState[] = [];
  private boundFocalId: string | null = null;

  // Ring rotation accumulators (rad).
  private ringARot = 0;
  private ringBRot = 0;
  private ringCRot = 0;

  // Last frame timestamp (ms). -1 = first frame.
  private lastTickMs = -1;

  constructor() {
    this.tiltGroup.rotation.x = RING_TILT_RAD;
    this.group.add(this.tiltGroup);

    // Build all three rings at unit radius; we scale + reposition the
    // group in setCentre / update so the geometry stays static.
    //
    // Radii bumped from 1.60/1.95, 2.10/2.40, 2.60/2.78 to 2.50/2.85,
    // 3.00/3.30, 3.50/3.68 (≈1.5×) so the rings extend beyond the active
    // baby's halo (whose outer corona reaches ~radius * 4.7 at the active
    // boost). Previously the rings sat inside the halo and were
    // invisible; user reported "the rings don't work — you can't even
    // see them". Opacities bumped 0.85/0.70/0.50 → 0.95/0.85/0.65 so the
    // partial-arcs read at deep-zoom against the now-larger halo.
    this.ringA = makeArcMesh(2.50, 2.85, TEAL_BRIGHT, 0.95);
    this.ringB = makeArcMesh(3.00, 3.30, TEAL_MID,    0.85);
    this.ringC = makeArcMesh(3.50, 3.68, TEAL_BRIGHT, 0.65);
    this.tiltGroup.add(this.ringA.mesh);
    this.tiltGroup.add(this.ringB.mesh);
    this.tiltGroup.add(this.ringC.mesh);

    // Group hidden until first setVisible(true).
    this.group.visible = false;
  }

  /** Trigger fade-in / fade-out. Idempotent; chaining same value is a no-op. */
  setVisible(target: boolean, nowMs: number): void {
    if (target === this.visible) return;
    this.visible = target;
    this.fadeStartMs = nowMs;
    this.fadeFromOpacity = this.currentOpacity;
    this.fadeToOpacity = target ? 1.0 : 0.0;
    this.fadeDuration = target ? FADE_IN_MS : FADE_OUT_MS;
    if (target) {
      this.group.visible = true;
      if (this.startMs < 0) this.startMs = nowMs;
    }
  }

  /** True iff fully invisible AND not in a fade. Caller can dispose. */
  isFullyHidden(): boolean {
    return !this.visible && this.currentOpacity <= 0.001;
  }

  /** Re-bind to a focal node + telemetry. Must be called whenever the
   *  focal node changes; dispose-and-rebuild glyphs if the label list
   *  differs from the cached one. */
  update(inputs: PlanetScanInputs, focalId: string): void {
    this.boundFocalId = focalId;
    this.radius = inputs.radius;
    this.progress = Math.max(0, Math.min(1, inputs.progress));
    this.gates = inputs.gates;

    // Position the whole group at the focal node.
    this.group.position.copy(inputs.centre);
    // Scale rings by node radius so the geometry's unit-radius math
    // expresses world-radii via group.scale.
    this.group.scale.setScalar(this.radius);

    // Update arc geometry for ring A (progress) and ring B (gates).
    this.refreshRingA();
    this.refreshRingB();
    this.refreshRingC();
  }

  /** Per-frame tick. dtSec = seconds since last tick. */
  tick(nowMs: number): void {
    const dt = this.lastTickMs < 0 ? 0 : Math.max(0, (nowMs - this.lastTickMs) / 1000);
    this.lastTickMs = nowMs;

    // Fade animation.
    if (this.fadeStartMs >= 0) {
      const t = Math.min(1, (nowMs - this.fadeStartMs) / Math.max(1, this.fadeDuration));
      this.currentOpacity = this.fadeFromOpacity + (this.fadeToOpacity - this.fadeFromOpacity) * t;
      if (t >= 1) {
        this.currentOpacity = this.fadeToOpacity;
        this.fadeStartMs = -1;
        if (!this.visible) this.group.visible = false;
      }
    }

    // Apply opacity to every material.
    this.ringA.mat.opacity = 0.95 * this.currentOpacity;
    this.ringB.mat.opacity = 0.85 * this.currentOpacity;
    this.ringC.mat.opacity = 0.65 * this.currentOpacity;

    if (!this.group.visible) return;

    // Rotate rings.
    this.ringARot += RING_A_RATE * dt;
    this.ringBRot += RING_B_RATE * dt;
    this.ringCRot += RING_C_RATE * dt;
    this.ringA.mesh.rotation.y = this.ringARot;
    this.ringB.mesh.rotation.y = this.ringBRot;
    this.ringC.mesh.rotation.y = this.ringCRot;
  }

  /** No-op kept for caller compatibility. Glyph billboards were removed;
   *  the new NodePanel DOM overlay takes over the textual surface. */
  faceCamera(_camera: THREE.Camera): void {
    /* no-op since glyphs were removed; signature retained so StarCanvas
     * can keep its existing per-frame call site without a conditional. */
  }

  dispose(): void {
    for (const r of [this.ringA, this.ringB, this.ringC]) {
      r.geo.dispose();
      r.mat.dispose();
    }
    this.tiltGroup.clear();
    this.group.clear();
  }

  // -----------------------------------------------------------------
  // Internals.
  // -----------------------------------------------------------------

  private refreshRingA(): void {
    // Filled arc length proportional to progress. Start at the top
    // (theta = -PI/2) and sweep clockwise as progress climbs to 1.
    const span = this.progress * Math.PI * 2;
    const start = -Math.PI / 2;
    const end = start + span;
    const seg = Math.max(1, Math.ceil(ARC_MAX_SEGMENTS * this.progress));
    updateArcGeometry(this.ringA.geo, start, end, seg, TEAL_BRIGHT);
  }

  private refreshRingB(): void {
    updateGateSegmentsGeometry(this.ringB.geo, this.gates, DEFAULT_GATE_COUNT);
  }

  private refreshRingC(): void {
    // Full ring, uniform teal. Built once.
    updateArcGeometry(
      this.ringC.geo,
      0, Math.PI * 2,
      ARC_MAX_SEGMENTS,
      TEAL_BRIGHT,
    );
  }

}

/** Read the world-space radius of a node ref by inspecting its mesh
 *  geometry. SphereGeometry stores its radius in `parameters.radius`.
 *  Returns a small fallback if the inspection fails. */
export function nodeRadiusOf(node: OrbitNodeRef): number {
  const geo = node.mesh.geometry as THREE.SphereGeometry;
  // SphereGeometry preserves its constructor parameters under
  // `parameters` — use that directly so we don't need a parallel field
  // on OrbitNodeRef.
  const r = geo?.parameters?.radius;
  if (typeof r === 'number' && r > 0) return r;
  return 7; // matches DEFAULT_ORBIT_CONFIG.babyNodeRadius fallback
}

/** Build glyph labels from an OrbitFixture + active node hint. Falls
 *  back to placeholder strings keyed off the focal node id when the
 *  fixture lacks live telemetry — keeps the planet-scan layer always
 *  legible without forcing the caller to provide data. */
export function defaultGlyphsFor(opts: {
  focalId: string;
  fsm?: string;
  grcStep?: string;
  evidenceTotal?: number;
  evidencePassed?: number;
  conceptName?: string;
  stepIndex?: number;
  stepTotal?: number;
  phase?: string;
}): string[] {
  const out: string[] = [];
  if (opts.phase) out.push(`phase: ${opts.phase}`);
  if (opts.stepIndex !== undefined && opts.stepTotal !== undefined) {
    out.push(`step: ${opts.stepIndex}/${opts.stepTotal}`);
  }
  if (opts.conceptName) out.push(`concept: ${opts.conceptName}`);
  if (opts.fsm) out.push(`fsm: ${opts.fsm}`);
  if (opts.evidenceTotal !== undefined) {
    const passed = opts.evidencePassed ?? 0;
    out.push(`evidence: ${passed}/${opts.evidenceTotal}`);
  }
  if (out.length === 0) out.push(`node: ${opts.focalId.slice(0, 12)}`);
  return out.slice(0, 5);
}

/** Compute progress fraction from fixture data. Falls back to a
 *  deterministic constant so the ring still reads as decorative when
 *  the live system hasn't reported progress yet. */
export function progressFrom(opts: {
  fsm?: string;
  evidenceTotal?: number;
  evidencePassed?: number;
}): number {
  if (opts.evidenceTotal !== undefined && opts.evidenceTotal > 0) {
    return Math.max(0, Math.min(1, (opts.evidencePassed ?? 0) / opts.evidenceTotal));
  }
  // FSM-driven heuristic — rough monotone progress through the workflow.
  switch (opts.fsm) {
    case 'IDLE':          return 0.05;
    case 'PLANNING':      return 0.15;
    case 'RESEARCHING':   return 0.30;
    case 'TESTING_FIRST': return 0.45;
    case 'EXECUTING':     return 0.65;
    case 'AUDITING':      return 0.85;
    case 'MEMORIZING':    return 0.95;
    case 'COMPLETE':      return 1.0;
    default:              return DEFAULT_PROGRESS;
  }
}
