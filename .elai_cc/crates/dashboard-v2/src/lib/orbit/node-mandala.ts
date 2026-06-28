// node-mandala.ts — "Datastream Mandala" 3D layer that surrounds the
// focused orbit node. Combines:
//   • 3 interpretive Saturn-style rings (progress / evidence-gates /
//     time-drift) — geometry primitives borrowed from planet-scan.ts so
//     the two layers stay in sync.
//   • 4 compass-anchored 3D markers (N/E/S/W) carrying IDENTITY /
//     LOCATION / STATUS / ACTIVITY semantics. Wireframe LineSegments —
//     crisp, below the bloom threshold so they read as digital glyphs.
//   • 4 (or 5) tendrils — TubeGeometry along QuadraticBezierCurve3 from
//     the node centre to each marker, slightly arched above the orbital
//     plane (0.3 unit lift). Subtle emissive pulse at 0.5 Hz, ±0.08
//     intensity (sandbox flagged ±0.20 as too strong).
//
// Lifecycle:
//   build(scene, ...)  → adds owned root group to the scene
//   update(...)        → re-anchors / refreshes ring telemetry
//   tick(dt, time)     → animates rotation + tendril pulse
//   dispose()          → scene.remove + dispose every geometry/material/
//                        texture; safe to call multiple times.
//
// All time-based constants use `factor = 1 - exp(-rate * dt)` smoothing
// (or simple dt-scaled accumulators) — never a per-frame magic number.

import * as THREE from 'three';
import { Text as TroikaText } from 'troika-three-text';
import {
  ORBIT_OVERLAY_LAYER,
} from '../scene/three-orbit.ts';
import {
  ARC_MAX_SEGMENTS,
  TEAL_BRIGHT,
  TEAL_MID,
  TEAL_DIM,
  type GateState,
  type RingMesh,
  buildArcRingGeometry,
  makeArcMesh,
  updateArcGeometry,
  updateGateSegmentsGeometry,
} from '../scene/planet-scan.ts';
import jetbrainsMonoUrl from '@fontsource-variable/jetbrains-mono/files/jetbrains-mono-latin-wght-normal.woff2?url';

// Compass keys + a 5th NW slot used by the integration-point variant.
export type MandalaAnchorKey = 'n' | 'e' | 's' | 'w' | 'nw';

// 3D UI bracket — a single corner of a dynamic readout frame. Two
// thin BoxGeometry arms forming an L-shape. Position + scale are
// re-applied each frame in `layoutReadout()` based on text bounds.
interface BracketCorner {
  hMesh: THREE.Mesh;
  vMesh: THREE.Mesh;
  hGeo: THREE.BoxGeometry;
  vGeo: THREE.BoxGeometry;
  mat: THREE.MeshBasicMaterial;
  /** Sign convention: -1 = left/bottom, +1 = right/top. */
  cornerX: -1 | 1;
  cornerY: -1 | 1;
}

// 3D readout frame — a set of 4 corner brackets + a thin row divider,
// surrounding 1-2 troika text slots whose bounds drive the frame size.
interface ReadoutFrame {
  texts: TroikaText[];
  brackets: BracketCorner[]; // 4 corners (TL, TR, BL, BR)
  divider: { mesh: THREE.Mesh; geo: THREE.BoxGeometry; mat: THREE.MeshBasicMaterial };
  padX: number;              // padding around bounds (node-radius units)
  padY: number;
  baseBracketOpacity: number;
  baseDividerOpacity: number;
}

// Easing — used by the sonar-pulse radial expansion.
function easeOutQuad(t: number): number {
  return 1 - (1 - t) * (1 - t);
}

/** Flat text payload rendered as 3D troika-text floating in the
 *  scene around the focal node. All strings are optional — empty
 *  strings hide the corresponding text element. */
export interface MandalaTextContent {
  /** Big sans title above the mandala. */
  title: string;
  /** Mono uppercase breadcrumb at the very top. */
  breadcrumb: string;
  /** Smaller sans subtitle just under the title. */
  subtitle: string;
  /** Mono percentage rendered INSIDE the inner ring (e.g. "29%"). */
  progressPercent: string;
  /** Mono status string to the right of the node (e.g. "◐ EXECUTING"). */
  status: string;
  /** Mono dim line under the status (e.g. "4s elapsed"). */
  elapsed: string;
  /** Mono file reference below the mandala. */
  fileRef: string;
  /** Live action line under the file ref (e.g. "▶ writing fn pool_builder"). */
  liveAction: string;
  /** Compact meta strip at the very bottom (tokens · ctx · tests). */
  meta: string;
}

export function emptyTextContent(): MandalaTextContent {
  return {
    title: '', breadcrumb: '', subtitle: '',
    progressPercent: '', status: '', elapsed: '',
    fileRef: '', liveAction: '', meta: '',
  };
}

export interface MandalaParams {
  /** World-space centre of the focal node. */
  centre: THREE.Vector3;
  /** Node radius in world units (rings + markers scale off this). */
  radius: number;
  /** Progress fraction in [0, 1]. Drives the inner ring's filled arc. */
  progress: number;
  /** Evidence gates — each lit/dim segment on the middle ring. */
  gates: GateState[];
  /** When true, render the 5th NW marker (integration-point). */
  showNwMarker?: boolean;
  /** When true, recolour the markers per source-tier (research mode). */
  tierColored?: boolean;
  /** Flat text payload — drives the 3D troika-text labels. */
  text: MandalaTextContent;
}

// Ring radii (× node radius). Sized for visual presence — the mandala
// is the *embodiment* of the focal node, not a small embellishment.
// Sandbox flagged 2.5/3.0/3.5× as "too small / not part of 3D UI".
const RING_INNER_INNER = 4.10;
const RING_INNER_OUTER = 4.32;     // ~4.2× — progress arc
const RING_MID_INNER   = 5.05;
const RING_MID_OUTER   = 5.27;     // ~5.2× — evidence gates
const RING_OUTER_INNER = 6.10;
const RING_OUTER_OUTER = 6.34;     // ~6.2× — time drift

// Per-ring rotation rates (rad/s). Outer ring is the visible drift; the
// other two breathe a small idle rotation so they don't read as static.
const RING_INNER_RATE = 0.020;
const RING_MID_RATE   = 0.020;
const RING_OUTER_RATE = 0.105;     // ≈ 1 rotation / 60 s

// Marker placement — distance from node centre (× node radius). Just
// outside the outer ring (6.34×) at 6.65× — close enough to read as
// "attached" to the readout but past the ring so they don't clip.
const MARKER_RADIUS_FACTOR = 6.65;
// Marker geometry sizes (world units). 2× the prior values so the
// glyphs read at typical zoom distances without depending on bloom.
const HEX_RADIUS    = 0.26;
const PILLAR_W      = 0.09;
const PILLAR_H      = 0.62;
const OCTA_RADIUS   = 0.26;
const TETRA_RADIUS  = 0.28;

// Tendril arch lift (world units, in node-radius space — multiplied by
// the focal radius before being added to the bezier control point).
const TENDRIL_ARCH_LIFT = 0.65;
const TENDRIL_TUBE_RADIUS = 0.022;
const TENDRIL_TUBULAR_SEGS = 28;
const TENDRIL_RADIAL_SEGS  = 6;

// Tendril emissive pulse — 0.5 Hz, ±0.08 intensity (NOT ±0.20: the
// sandbox flagged 0.20 as too strong).
const TENDRIL_PULSE_HZ = 0.5;
const TENDRIL_PULSE_AMP = 0.08;
const TENDRIL_PULSE_BASE = 0.55;

// Cross-axis "armillary" halos (× node radius). These are the VERTICAL
// LATERAL elements that give the mandala depth — they live perpendicular
// to the orbital plane so the camera sees them as rings rising up over
// the node from the side, regardless of orbital tilt. Sit between the
// mid (5.2×) and outer (6.2×) horizontal rings.
const HALO_RADIUS = 5.55;
const HALO_TUBE = 0.020;
const HALO_RADIAL_SEGS = 6;
const HALO_TUBULAR_SEGS = 80;
const HALO_DRIFT_RATE = 0.040;     // rad/s — slow rotation around y.

// Central vertical beam — subtle pillar of light along the y-axis. Adds
// a vertical "spine" to the mandala so depth reads even when the camera
// is roughly level with the orbital plane.
const BEAM_HEIGHT = 7.5;            // node-radius units (±3.75 above/below)
const BEAM_RADIUS = 0.028;
const BEAM_PULSE_HZ = 0.35;
const BEAM_PULSE_AMP = 0.10;
const BEAM_PULSE_BASE = 0.30;

// "Sonar pulse" — periodic outward-expanding ring centred on the node
// in the orbital plane. Creates rhythmic life. Period = 4.2s, max
// radius = 7.5× node radius (just past the outer drift ring).
const SONAR_PERIOD_S = 4.2;
const SONAR_MIN_RADIUS = 0.4;
const SONAR_MAX_RADIUS = 7.5;
const SONAR_TUBE = 0.020;

// Fade durations (ms). Bumped to 1100 so the mandala settles elegantly
// after the camera warp ends. setVisible(true) is now ALSO gated on the
// camera warp completing (in StarCanvas.syncMandala) so the fade-in
// never starts while the camera is still moving.
const FADE_IN_MS  = 1100;
const FADE_OUT_MS = 220;

// Tier palette for research mode — T1=bright, T2=mid, T3=dim.
const TIER_COLORS: Record<MandalaAnchorKey, THREE.Color> = {
  n:  TEAL_BRIGHT,
  e:  TEAL_MID,
  s:  TEAL_BRIGHT,
  w:  TEAL_DIM,
  nw: TEAL_MID,
};

interface Marker {
  group: THREE.Group;
  segments: THREE.LineSegments;
  geo: THREE.BufferGeometry;
  mat: THREE.LineBasicMaterial;
  /** Local position (in unit-radius space). Scaled by node radius via group.scale. */
  localOffset: THREE.Vector3;
  key: MandalaAnchorKey;
}

interface Tendril {
  mesh: THREE.Mesh;
  geo: THREE.TubeGeometry;
  mat: THREE.MeshBasicMaterial;
  key: MandalaAnchorKey;
  /** Curve regenerated when radius changes. */
  curve: THREE.QuadraticBezierCurve3;
}

/** NodeMandala — disposable 3D layer rendered around the focal node. */
export class NodeMandala {
  /** Public root — caller adds/removes from the scene. */
  readonly group = new THREE.Group();

  // Tilt frame mirrors PlanetScan's ~15° off-axis tilt so the two
  // layers visually merge into one mandala.
  private readonly tiltGroup = new THREE.Group();
  // Cross-axis frame — NOT tilted, so vertical halos stay perpendicular
  // to the orbital plane regardless of tilt.
  private readonly crossGroup = new THREE.Group();

  // Rings.
  private ringInner: RingMesh;
  private ringMid:   RingMesh;
  private ringOuter: RingMesh;

  // Cross-axis halo arcs (vertical lateral depth) + central vertical
  // beam through the node + radial sonar pulse.
  private haloXY: { mesh: THREE.Mesh; geo: THREE.TorusGeometry; mat: THREE.MeshBasicMaterial };
  private haloYZ: { mesh: THREE.Mesh; geo: THREE.TorusGeometry; mat: THREE.MeshBasicMaterial };
  private beam:   { mesh: THREE.Mesh; geo: THREE.CylinderGeometry; mat: THREE.MeshBasicMaterial };
  private sonar:  { mesh: THREE.Mesh; geo: THREE.TorusGeometry; mat: THREE.MeshBasicMaterial };

  // 3D text labels (troika). Each is positioned in world space relative
  // to the focal node and billboards toward camera every frame so text
  // is always readable from any angle. Text lives on ORBIT_OVERLAY_LAYER
  // so it does NOT trip the bloom pass — we want crisp readable glyphs,
  // not blurred halos.
  private textTitle:        TroikaText;
  private textSubtitle:     TroikaText;
  private textBreadcrumb:   TroikaText;
  private textProgress:     TroikaText;
  private textStatus:       TroikaText;
  private textElapsed:      TroikaText;
  private textFileRef:      TroikaText;
  private textLiveAction:   TroikaText;
  private textMeta:         TroikaText;
  private textGroup = new THREE.Group();

  // 3D UI decorations — split into TWO categories:
  //
  //   readoutTop/Bottom/Left/Right: 4 corner brackets + 1 row divider
  //     each, dynamically sized every frame from the actual text
  //     bounds (so brackets adapt to any string length).
  //
  //   uiDecorations: static decorations attached to the mandala
  //     itself (ring tick marks, decorative arcs, accent bars).
  //
  // Both fade with currentOpacity in tick().
  private readoutTop!:    ReadoutFrame;
  private readoutBottom!: ReadoutFrame;
  private readoutLeft!:   ReadoutFrame;
  private readoutRight!:  ReadoutFrame;
  private uiDecorations: Array<{ mesh: THREE.Mesh; geo: THREE.BufferGeometry; mat: THREE.MeshBasicMaterial; baseOpacity: number }> = [];

  // Markers + tendrils — keyed by compass position.
  private markers = new Map<MandalaAnchorKey, Marker>();
  private tendrils = new Map<MandalaAnchorKey, Tendril>();

  // Animation state.
  private fadeStartMs = -1;
  private fadeFromOpacity = 0;
  private fadeToOpacity = 0;
  private fadeDuration = 0;
  private currentOpacity = 0;
  private visible = false;

  private radius = 0;
  private progress = 0;
  private gates: GateState[] = [];
  private showNwMarker = false;
  private tierColored = false;

  // Rotation accumulators (rad).
  private rInnerRot = 0;
  private rMidRot = 0;
  private rOuterRot = 0;
  private rCrossRot = 0;

  // Pulse phase accumulators (rad).
  private pulsePhase = 0;
  private beamPulsePhase = 0;
  // Sonar pulse — wraps every SONAR_PERIOD_S seconds.
  private sonarPhase = 0;

  private disposed = false;
  private builtIntoScene: THREE.Scene | null = null;

  constructor() {
    this.tiltGroup.rotation.x = (15 * Math.PI) / 180;
    this.group.add(this.tiltGroup);
    this.group.add(this.crossGroup);

    this.ringInner = makeArcMesh(RING_INNER_INNER, RING_INNER_OUTER, TEAL_BRIGHT, 0.95);
    this.ringMid   = makeArcMesh(RING_MID_INNER,   RING_MID_OUTER,   TEAL_MID,    0.85);
    this.ringOuter = makeArcMesh(RING_OUTER_INNER, RING_OUTER_OUTER, TEAL_BRIGHT, 0.55);
    this.tiltGroup.add(this.ringInner.mesh);
    this.tiltGroup.add(this.ringMid.mesh);
    this.tiltGroup.add(this.ringOuter.mesh);

    // Cross-axis halo XY — torus in the xy plane (default Torus
    // orientation lies flat with thickness in z; rotate 90° around X
    // so it stands "vertical" and faces the camera as a halo).
    {
      const geo = new THREE.TorusGeometry(HALO_RADIUS, HALO_TUBE, HALO_RADIAL_SEGS, HALO_TUBULAR_SEGS);
      const mat = new THREE.MeshBasicMaterial({
        color: TEAL_MID, transparent: true, opacity: 0.55,
        depthWrite: false, blending: THREE.AdditiveBlending,
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.layers.set(ORBIT_OVERLAY_LAYER);
      mesh.rotation.x = Math.PI / 2;     // stand it up
      this.crossGroup.add(mesh);
      this.haloXY = { mesh, geo, mat };
    }
    // Cross-axis halo YZ — perpendicular to haloXY.
    {
      const geo = new THREE.TorusGeometry(HALO_RADIUS, HALO_TUBE, HALO_RADIAL_SEGS, HALO_TUBULAR_SEGS);
      const mat = new THREE.MeshBasicMaterial({
        color: TEAL_MID, transparent: true, opacity: 0.45,
        depthWrite: false, blending: THREE.AdditiveBlending,
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.layers.set(ORBIT_OVERLAY_LAYER);
      mesh.rotation.x = Math.PI / 2;
      mesh.rotation.y = Math.PI / 2;     // rotate the standing torus 90° around Y
      this.crossGroup.add(mesh);
      this.haloYZ = { mesh, geo, mat };
    }
    // Central vertical beam — REMOVED. Its visual role (a vertical
    // spine through the node) is now served by the N/S markers + their
    // tendrils on the crossGroup, which provide the same axis cue
    // without the ambiguous "what is this beam?" reading.
    // (Field still exists but holds disposed/empty resources.)
    {
      const geo = new THREE.CylinderGeometry(0.001, 0.001, 0.001, 3, 1, true);
      const mat = new THREE.MeshBasicMaterial({ color: 0x000000, transparent: true, opacity: 0, visible: false });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.visible = false;
      this.beam = { mesh, geo, mat };
    }
    // Sonar pulse — outward-expanding torus in the orbital plane.
    // Lives under tiltGroup so it inherits the orbital tilt.
    {
      const geo = new THREE.TorusGeometry(1, SONAR_TUBE, 4, 96);
      const mat = new THREE.MeshBasicMaterial({
        color: TEAL_BRIGHT, transparent: true, opacity: 0.0,
        depthWrite: false, blending: THREE.AdditiveBlending,
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.layers.set(ORBIT_OVERLAY_LAYER);
      mesh.rotation.x = Math.PI / 2;
      this.tiltGroup.add(mesh);
      this.sonar = { mesh, geo, mat };
    }

    // E and W markers ride the tiltGroup (orbital plane left/right).
    // N, S, and NW are on the VERTICAL axis (top/bottom/diagonal),
    // mounted under crossGroup so they don't inherit the orbital tilt.
    // This puts the title's anchor straight ABOVE the node and the
    // live-action's anchor straight BELOW — the user feedback was that
    // front/back (±z) markers should be top/bottom (±y) instead.
    this.buildMarker('e',  PILLAR_GEO,    new THREE.Vector3(+MARKER_RADIUS_FACTOR, 0, 0), this.tiltGroup);
    this.buildMarker('w',  TETRA_GEO,     new THREE.Vector3(-MARKER_RADIUS_FACTOR, 0, 0), this.tiltGroup);
    this.buildMarker('n',  HEX_PRISM_GEO, new THREE.Vector3(0, +MARKER_RADIUS_FACTOR, 0), this.crossGroup);
    this.buildMarker('s',  OCTA_GEO,      new THREE.Vector3(0, -MARKER_RADIUS_FACTOR, 0), this.crossGroup);
    this.buildMarker('nw', HEX_PRISM_GEO, new THREE.Vector3(
      -MARKER_RADIUS_FACTOR * Math.SQRT1_2,
      +MARKER_RADIUS_FACTOR * Math.SQRT1_2,
      0,
    ), this.crossGroup);
    // NW marker hidden by default.
    const nw = this.markers.get('nw');
    if (nw) nw.group.visible = false;

    // Tendrils — one per compass slot. E/W under tiltGroup so they
    // arch through the orbital plane; N/S/NW under crossGroup so they
    // arch perpendicular to the vertical axis (in +z).
    this.buildTendril('e',  this.tiltGroup,  'horizontal');
    this.buildTendril('w',  this.tiltGroup,  'horizontal');
    this.buildTendril('n',  this.crossGroup, 'vertical');
    this.buildTendril('s',  this.crossGroup, 'vertical');
    this.buildTendril('nw', this.crossGroup, 'vertical');
    const nwTendril = this.tendrils.get('nw');
    if (nwTendril) nwTendril.mesh.visible = false;

    // 3D text labels — all under textGroup which inherits the focal
    // node's radius scale (set in update). All sizes/positions below
    // are expressed in NODE-RADIUS UNITS, so they scale 1:1 with the
    // mandala at every zoom level.
    //
    // Layout symmetry (matches the new vertical-marker layout):
    //   N marker (+y)  ←  TOP   = breadcrumb + title (anchored center)
    //   E marker (+x)  ←  RIGHT = status + elapsed (anchored left of marker)
    //   W marker (-x)  ←  LEFT  = file path + context (anchored right of marker)
    //   S marker (-y)  ←  BOT   = live action + meta (anchored center)
    // The central progress text (was at 0,0,0 = ON the node sphere)
    // is GONE — the right-side ▓▓▓░ XX% bar carries that data.
    this.group.add(this.textGroup);
    // Text positioned BEYOND the markers (markers at ±6.65) so the
    // glyph icons sit between the mandala and the readouts.
    //   TOP    →  y > +6.65   (text above N marker)
    //   BOTTOM →  y < -6.65   (text below S marker)
    //   RIGHT  →  x > +6.65   (anchorX=left, reads outward)
    //   LEFT   →  x < -6.65   (anchorX=right, reads outward)
    this.textTitle      = this.makeTextSlot({ size: 0.95, color: TEAL_BRIGHT, anchorY: 'middle', font: 'sans',  pos: [0,  7.55, 0] });
    this.textBreadcrumb = this.makeTextSlot({ size: 0.40, color: TEAL_DIM,    anchorY: 'middle', font: 'mono',  pos: [0,  8.55, 0], letterSpacing: 0.14 });
    // Right side — STATUS readout. Anchor LEFT, text extends outward to +x.
    this.textStatus     = this.makeTextSlot({ size: 0.55, color: TEAL_BRIGHT, anchorY: 'middle', font: 'mono',  pos: [7.45, 0.45, 0], anchorX: 'left' });
    this.textElapsed    = this.makeTextSlot({ size: 0.46, color: TEAL_DIM,    anchorY: 'middle', font: 'mono',  pos: [7.45, -0.45, 0], anchorX: 'left' });
    // Left side — FILE / LOCATION readout. Anchor RIGHT, text extends outward to -x.
    this.textFileRef    = this.makeTextSlot({ size: 0.55, color: TEAL_MID,    anchorY: 'middle', font: 'mono',  pos: [-7.45, 0.45, 0], anchorX: 'right' });
    this.textSubtitle   = this.makeTextSlot({ size: 0.46, color: TEAL_DIM,    anchorY: 'middle', font: 'mono',  pos: [-7.45, -0.45, 0], anchorX: 'right' });
    // Bottom — live action + meta.
    this.textLiveAction = this.makeTextSlot({ size: 0.62, color: TEAL_BRIGHT, anchorY: 'middle', font: 'mono',  pos: [0, -7.55, 0] });
    this.textMeta       = this.makeTextSlot({ size: 0.46, color: TEAL_DIM,    anchorY: 'middle', font: 'mono',  pos: [0, -8.55, 0] });
    // textProgress slot retained as a dummy (always empty content) so
    // existing update/dispose paths don't need conditional logic.
    this.textProgress   = this.makeTextSlot({ size: 0.001, color: TEAL_DIM, anchorY: 'middle', font: 'mono', pos: [0, -100, 0] });

    // ---- 3D UI DECORATIONS ---------------------------------------------
    //
    // 4 readout frames (top/bottom/left/right) — DYNAMICALLY sized each
    // frame from the actual text bounds (so they fit any string length).
    // Plus static decorations: 12 radial tick marks around the outer
    // ring, a partial decorative arc, and 2 vertical accent bars.
    this.readoutTop    = this.buildReadoutFrame([this.textBreadcrumb, this.textTitle],      0.55, 0.30);
    this.readoutBottom = this.buildReadoutFrame([this.textLiveAction, this.textMeta],       0.55, 0.30);
    this.readoutLeft   = this.buildReadoutFrame([this.textFileRef,    this.textSubtitle],   0.45, 0.28);
    this.readoutRight  = this.buildReadoutFrame([this.textStatus,     this.textElapsed],    0.45, 0.28);
    this.buildOuterTickMarks();
    this.buildDecorativeArcs();

    this.group.visible = false;
  }

  /** Internal — build a troika text slot with shared defaults.
   *  Several troika props (font, letterSpacing, etc.) aren't in the
   *  shipped TS types but are valid at runtime; cast through `any`
   *  for those assignments. */
  private makeTextSlot(opts: {
    size: number;
    color: THREE.Color;
    anchorY: 'top' | 'middle' | 'bottom';
    anchorX?: 'left' | 'center' | 'right';
    font: 'sans' | 'mono';
    pos: [number, number, number];
    letterSpacing?: number;
  }): TroikaText {
    const t = new TroikaText();
    const tx = t as unknown as Record<string, unknown>;
    t.text = '';
    t.fontSize = opts.size;
    // NOTE: deliberately NOT setting `font` to our bundled woff2 URLs.
    // Tauri's webview was failing to fetch the `?url`-imported assets
    // silently, leaving the SDF atlas empty so no glyphs ever rendered.
    // Troika's built-in Roboto fallback works without any fetch.
    void opts.font; void jetbrainsMonoUrl;
    // Troika expects color as hex int, hex string, or rgb tuple — NOT
    // a THREE.Color instance. Pass the integer hex value.
    tx.color = opts.color.getHex();
    t.anchorX = opts.anchorX ?? 'center';
    t.anchorY = opts.anchorY;
    t.outlineWidth = 0;
    // Single-material assumed for troika Text — never an array. Cast
    // to bypass the broad union type. DoubleSide + depthTest=false so
    // glyphs read regardless of billboard face AND always render on top
    // of the mandala geometry (no z-fighting against the rings).
    const mat = t.material as THREE.Material;
    mat.transparent = true;
    mat.depthWrite = false;
    mat.depthTest = false;
    mat.side = THREE.DoubleSide;
    if (opts.letterSpacing) tx.letterSpacing = opts.letterSpacing;
    t.position.set(opts.pos[0], opts.pos[1], opts.pos[2]);
    // Render order > 0 so text draws AFTER the bloom-target rings but
    // BEFORE the final compositor.
    t.renderOrder = 10;
    // Enable BOTH the default bloom layer (0) AND the overlay layer (1).
    // Text colour stays below the bloom threshold (0.85 HDR) so it
    // won't actually bloom — but rendering on both layers means it
    // appears in either render pass even if the layer-mask logic in
    // three-scene.ts doesn't include the overlay layer here.
    t.layers.enable(0);
    t.layers.enable(1);
    // Troika's bounding sphere is computed lazily after sync(); a freshly-
    // built mesh has a zero-radius sphere and gets frustum-culled. Disable
    // culling so the mesh renders regardless of bounds-test state.
    t.frustumCulled = false;
    t.visible = false;            // hidden until first non-empty content
    this.textGroup.add(t);
    return t;
  }

  /** Build NodeMandala into a parent scene. Idempotent: subsequent
   *  build() calls move the group between scenes. */
  build(scene: THREE.Scene, params: MandalaParams): void {
    if (this.disposed) {
      throw new Error('NodeMandala: build() called after dispose()');
    }
    if (this.builtIntoScene && this.builtIntoScene !== scene) {
      this.builtIntoScene.remove(this.group);
    }
    if (this.builtIntoScene !== scene) {
      scene.add(this.group);
      this.builtIntoScene = scene;
    }
    this.update(params);
  }

  /** Re-anchor and refresh ring/marker telemetry. Cheap — no allocations. */
  update(params: MandalaParams): void {
    if (this.disposed) return;
    this.radius = Math.max(0.0001, params.radius);
    this.progress = Math.max(0, Math.min(1, params.progress));
    this.gates = params.gates;
    this.showNwMarker = !!params.showNwMarker;
    this.tierColored = !!params.tierColored;

    // Position + scale in one shot. Geometry is unit-radius; group.scale
    // expands it to the node's world radius.
    this.group.position.copy(params.centre);
    this.group.scale.setScalar(this.radius);

    this.refreshRingInner();
    this.refreshRingMid();
    this.refreshRingOuter();

    // NW marker visibility (5th compass slot).
    const nw = this.markers.get('nw');
    if (nw) nw.group.visible = this.showNwMarker;
    const nwT = this.tendrils.get('nw');
    if (nwT) nwT.mesh.visible = this.showNwMarker;

    // Recolour markers in research-mode tier mapping.
    for (const [key, m] of this.markers) {
      const target = this.tierColored ? TIER_COLORS[key] : TEAL_BRIGHT;
      m.mat.color.copy(target);
    }

    // Push text content into the troika slots. sync() rebuilds the SDF
    // geometry async — fine to call repeatedly. visible toggles per slot
    // so empty-string fields don't render an invisible mesh.
    this.applyTextSlot(this.textTitle,      params.text.title);
    this.applyTextSlot(this.textSubtitle,   params.text.subtitle);
    this.applyTextSlot(this.textBreadcrumb, params.text.breadcrumb);
    this.applyTextSlot(this.textProgress,   params.text.progressPercent);
    this.applyTextSlot(this.textStatus,     params.text.status);
    this.applyTextSlot(this.textElapsed,    params.text.elapsed);
    this.applyTextSlot(this.textFileRef,    params.text.fileRef);
    this.applyTextSlot(this.textLiveAction, params.text.liveAction);
    this.applyTextSlot(this.textMeta,       params.text.meta);
  }

  private applyTextSlot(slot: TroikaText, content: string): void {
    if (slot.text !== content) {
      slot.text = content;
      slot.sync(() => {
        // Logged once per sync so you can confirm in Tauri devtools that
        // glyph atlases are actually building. If you never see this
        // line in the console, troika's worker / font load is failing
        // and no glyphs will ever render.
        // eslint-disable-next-line no-console
        console.debug('[mandala] text synced', { len: content.length, preview: content.slice(0, 24) });
      });
    }
    slot.visible = content.length > 0;
  }

  /** Trigger fade-in / fade-out. Idempotent on repeated same-value calls. */
  setVisible(target: boolean, nowMs: number): void {
    if (this.disposed) return;
    if (target === this.visible) return;
    this.visible = target;
    this.fadeStartMs = nowMs;
    this.fadeFromOpacity = this.currentOpacity;
    this.fadeToOpacity = target ? 1.0 : 0.0;
    this.fadeDuration = target ? FADE_IN_MS : FADE_OUT_MS;
    if (target) this.group.visible = true;
  }

  /** Per-frame tick. dt = seconds since last call. The camera is needed
   *  for billboarding the 3D text labels — every text mesh copies the
   *  camera's quaternion so it always faces the viewer. */
  tick(dt: number, nowMs: number, camera?: THREE.Camera): void {
    if (this.disposed) return;

    if (this.fadeStartMs >= 0) {
      const t = Math.min(1, (nowMs - this.fadeStartMs) / Math.max(1, this.fadeDuration));
      this.currentOpacity = this.fadeFromOpacity + (this.fadeToOpacity - this.fadeFromOpacity) * t;
      if (t >= 1) {
        this.currentOpacity = this.fadeToOpacity;
        this.fadeStartMs = -1;
        if (!this.visible) this.group.visible = false;
      }
    }

    // Ring opacities respect the per-ring base (matches planet-scan.ts).
    this.ringInner.mat.opacity = 0.95 * this.currentOpacity;
    this.ringMid.mat.opacity   = 0.85 * this.currentOpacity;
    this.ringOuter.mat.opacity = 0.55 * this.currentOpacity;
    // Cross-axis halos — slightly dimmer than mid horizontal ring so
    // the depth reads as supportive, not competitive with the focal rings.
    this.haloXY.mat.opacity = 0.55 * this.currentOpacity;
    this.haloYZ.mat.opacity = 0.45 * this.currentOpacity;

    if (!this.group.visible) return;

    // Rotate rings (dt-scaled). Markers ride the tiltGroup but DO NOT
    // counter-rotate — the design wants them at fixed compass positions
    // on the orbital plane. So markers stay attached to tiltGroup, NOT
    // to the rotating ring meshes.
    this.rInnerRot += RING_INNER_RATE * dt;
    this.rMidRot   += RING_MID_RATE   * dt;
    this.rOuterRot += RING_OUTER_RATE * dt;
    this.ringInner.mesh.rotation.y = this.rInnerRot;
    this.ringMid.mesh.rotation.y   = this.rMidRot;
    this.ringOuter.mesh.rotation.y = this.rOuterRot;

    // Cross-axis halos — slow drift around y so the armillary cage
    // reads as a 3D structure, not a static decal.
    this.rCrossRot += HALO_DRIFT_RATE * dt;
    this.crossGroup.rotation.y = this.rCrossRot;

    // Tendril emissive pulse — 0.5 Hz sin, ±0.08 intensity around BASE.
    this.pulsePhase += dt * TENDRIL_PULSE_HZ * Math.PI * 2;
    const pulse = TENDRIL_PULSE_BASE + Math.sin(this.pulsePhase) * TENDRIL_PULSE_AMP;
    for (const t of this.tendrils.values()) {
      t.mat.opacity = pulse * this.currentOpacity;
    }
    // Markers: flat opacity (crisp wireframes don't need a pulse).
    for (const m of this.markers.values()) {
      m.mat.opacity = this.currentOpacity * 0.85;
    }

    // Beam pulse — 0.35 Hz, ±0.10 around 0.30 base.
    this.beamPulsePhase += dt * BEAM_PULSE_HZ * Math.PI * 2;
    const beamPulse = BEAM_PULSE_BASE + Math.sin(this.beamPulsePhase) * BEAM_PULSE_AMP;
    this.beam.mat.opacity = beamPulse * this.currentOpacity;

    // Sonar pulse — wraps every SONAR_PERIOD_S seconds. Radius eases
    // outward; opacity fades from 0.6 → 0 over the period; resets
    // instantly at the wrap point so it reads as a fresh emit.
    this.sonarPhase += dt;
    const sonarT = (this.sonarPhase % SONAR_PERIOD_S) / SONAR_PERIOD_S;  // [0, 1)
    const sonarRadius = SONAR_MIN_RADIUS + (SONAR_MAX_RADIUS - SONAR_MIN_RADIUS) * easeOutQuad(sonarT);
    const sonarOpacity = 0.55 * (1 - sonarT) * this.currentOpacity;
    this.sonar.mesh.scale.setScalar(sonarRadius);
    this.sonar.mat.opacity = sonarOpacity;

    // 3D text billboarding + opacity. Each text element is anchored in
    // world space (so it lives WHERE the data is — over the node, beside
    // the rings, etc.) but rotates each frame to face the camera so the
    // glyphs are always upright and readable. Copying the camera's
    // quaternion is cheaper than text.lookAt(camera.position) and gives
    // identical visual results for distant cameras.
    if (camera) {
      this.textGroup.children.forEach((child) => {
        child.quaternion.copy(camera.quaternion);
      });
    }
    // Troika materials don't expose an opacity field directly, but we
    // can multiply via the fillOpacity property which the SDF shader
    // honours. fade in/out with the rest of the mandala.
    for (const t of [
      this.textTitle, this.textSubtitle, this.textBreadcrumb,
      this.textProgress, this.textStatus, this.textElapsed,
      this.textFileRef, this.textLiveAction, this.textMeta,
    ]) {
      t.fillOpacity = this.currentOpacity;
    }
    // Static UI decorations (ring tick marks, decorative arcs) fade
    // with the mandala.
    for (const d of this.uiDecorations) {
      d.mat.opacity = d.baseOpacity * this.currentOpacity;
    }
    // Dynamic readout frames — re-layout from current text bounds + fade.
    for (const r of [this.readoutTop, this.readoutBottom, this.readoutLeft, this.readoutRight]) {
      this.layoutReadout(r);
      const bracketOp = r.baseBracketOpacity * this.currentOpacity;
      for (const b of r.brackets) b.mat.opacity = bracketOp;
      r.divider.mat.opacity = r.baseDividerOpacity * this.currentOpacity;
    }
  }

  /** Project each compass marker (and node + main-panel anchor) to
   *  canvas-local pixel coordinates. Returns NaN-safe values; caller
   *  uses `visible` to decide whether to render the DOM card. */
  computeAnchors(
    camera: THREE.Camera,
    canvasW: number,
    canvasH: number,
    out: MandalaAnchors,
  ): void {
    if (this.disposed || !this.group.visible) {
      out.node.visible = false;
      out.mainPanel.visible = false;
      out.n.visible = false;
      out.e.visible = false;
      out.s.visible = false;
      out.w.visible = false;
      out.nw.visible = false;
      return;
    }
    const tmp = _projectScratch;
    // Node centre.
    project(this.group.position, camera, canvasW, canvasH, tmp, out.node);
    // Main panel anchor — 1.5 units BELOW the node (negative Y in world
    // space, which screens "above" or "below" depending on camera pitch
    // — at our 12° pitch this lands south of the node visually).
    _mainAnchorScratch.copy(this.group.position);
    _mainAnchorScratch.y -= 1.5 * this.radius;
    project(_mainAnchorScratch, camera, canvasW, canvasH, tmp, out.mainPanel);
    // Compass markers — convert local offset to world via tiltGroup
    // matrix. Multiplying through `tiltGroup.matrixWorld` gets the rendered
    // marker position even after the 15° tilt + node-radius scale.
    for (const [key, m] of this.markers) {
      _markerWorldScratch.copy(m.localOffset);
      _markerWorldScratch.applyMatrix4(this.tiltGroup.matrixWorld);
      const target = key === 'n' ? out.n
        : key === 'e' ? out.e
        : key === 's' ? out.s
        : key === 'w' ? out.w
        : out.nw;
      if (key === 'nw' && !this.showNwMarker) {
        target.visible = false;
        continue;
      }
      project(_markerWorldScratch, camera, canvasW, canvasH, tmp, target);
    }
  }

  /** Dispose all owned three.js resources. Safe to call multiple times. */
  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    if (this.builtIntoScene) {
      this.builtIntoScene.remove(this.group);
      this.builtIntoScene = null;
    }
    // Rings.
    for (const r of [this.ringInner, this.ringMid, this.ringOuter]) {
      r.geo.dispose();
      r.mat.dispose();
    }
    // Markers.
    for (const m of this.markers.values()) {
      m.geo.dispose();
      m.mat.dispose();
      m.group.removeFromParent();
    }
    this.markers.clear();
    // Tendrils.
    for (const t of this.tendrils.values()) {
      t.geo.dispose();
      t.mat.dispose();
      t.mesh.removeFromParent();
    }
    this.tendrils.clear();
    // Cross-axis halos + central beam + sonar pulse.
    this.haloXY.geo.dispose();
    this.haloXY.mat.dispose();
    this.haloXY.mesh.removeFromParent();
    this.haloYZ.geo.dispose();
    this.haloYZ.mat.dispose();
    this.haloYZ.mesh.removeFromParent();
    this.beam.geo.dispose();
    this.beam.mat.dispose();
    this.beam.mesh.removeFromParent();
    this.sonar.geo.dispose();
    this.sonar.mat.dispose();
    this.sonar.mesh.removeFromParent();
    // Troika text — each Text has its own dispose() which frees the SDF
    // geometry + material + atlas references.
    for (const t of [
      this.textTitle, this.textSubtitle, this.textBreadcrumb,
      this.textProgress, this.textStatus, this.textElapsed,
      this.textFileRef, this.textLiveAction, this.textMeta,
    ]) {
      t.dispose();
      t.removeFromParent();
    }
    // UI decorations — static ring ticks + decorative arcs.
    for (const d of this.uiDecorations) {
      d.geo.dispose();
      d.mat.dispose();
      d.mesh.removeFromParent();
    }
    this.uiDecorations.length = 0;
    // Readout frames — brackets + dividers (dynamic).
    for (const r of [this.readoutTop, this.readoutBottom, this.readoutLeft, this.readoutRight]) {
      if (!r) continue;
      for (const b of r.brackets) {
        b.hGeo.dispose();
        b.vGeo.dispose();
        b.mat.dispose();
        b.hMesh.removeFromParent();
        b.vMesh.removeFromParent();
      }
      r.brackets.length = 0;
      r.divider.geo.dispose();
      r.divider.mat.dispose();
      r.divider.mesh.removeFromParent();
    }
    this.tiltGroup.clear();
    this.crossGroup.clear();
    this.textGroup.clear();
    this.group.clear();
  }

  // -----------------------------------------------------------------
  // Internals.
  // -----------------------------------------------------------------

  /** Build a dynamic readout frame around the given text slots.
   *  Positions are placeholder (0,0) and unit-scaled — `layoutReadout`
   *  resizes/repositions them every frame from the live text bounds. */
  private buildReadoutFrame(
    texts: TroikaText[],
    padX: number,
    padY: number,
  ): ReadoutFrame {
    const ARM = 0.42;
    const TH  = 0.045;
    const baseBracketOpacity = 0.85;
    const baseDividerOpacity = 0.40;
    const brackets: BracketCorner[] = [];
    for (const cx of [-1, 1] as const) {
      for (const cy of [-1, 1] as const) {
        const mat = new THREE.MeshBasicMaterial({
          color: TEAL_BRIGHT, transparent: true, opacity: baseBracketOpacity,
          depthWrite: false, blending: THREE.AdditiveBlending,
        });
        // Unit BoxGeometry — re-scaled in layoutReadout.
        const hGeo = new THREE.BoxGeometry(1, 1, 1);
        const vGeo = new THREE.BoxGeometry(1, 1, 1);
        const hMesh = new THREE.Mesh(hGeo, mat);
        const vMesh = new THREE.Mesh(vGeo, mat);
        hMesh.scale.set(ARM, TH, 0.02);
        vMesh.scale.set(TH, ARM, 0.02);
        hMesh.layers.set(ORBIT_OVERLAY_LAYER);
        vMesh.layers.set(ORBIT_OVERLAY_LAYER);
        this.textGroup.add(hMesh);
        this.textGroup.add(vMesh);
        brackets.push({ hMesh, vMesh, hGeo, vGeo, mat, cornerX: cx, cornerY: cy });
      }
    }
    // Row divider between the two text lines.
    const dMat = new THREE.MeshBasicMaterial({
      color: TEAL_MID, transparent: true, opacity: baseDividerOpacity,
      depthWrite: false, blending: THREE.AdditiveBlending,
    });
    const dGeo = new THREE.BoxGeometry(1, 1, 1);
    const dMesh = new THREE.Mesh(dGeo, dMat);
    dMesh.scale.set(1, 0.03, 0.02);
    dMesh.layers.set(ORBIT_OVERLAY_LAYER);
    this.textGroup.add(dMesh);

    return { texts, brackets, divider: { mesh: dMesh, geo: dGeo, mat: dMat }, padX, padY, baseBracketOpacity, baseDividerOpacity };
  }

  /** Update bracket + divider positions from current text bounds.
   *  Cheap — only reads textRenderInfo (no allocations). Called every
   *  frame in tick() so brackets follow text as content changes.
   *  Falls through silently when texts haven't sync'd yet. */
  private layoutReadout(deco: ReadoutFrame): void {
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    let valid = false;
    for (const t of deco.texts) {
      if (!t.visible) continue;
      const info = (t as unknown as { textRenderInfo?: { blockBounds?: number[] } }).textRenderInfo;
      const bb = info?.blockBounds;
      if (!bb) continue;
      const tx = t.position.x;
      const ty = t.position.y;
      minX = Math.min(minX, bb[0] + tx);
      minY = Math.min(minY, bb[1] + ty);
      maxX = Math.max(maxX, bb[2] + tx);
      maxY = Math.max(maxY, bb[3] + ty);
      valid = true;
    }
    if (!valid) {
      // Hide brackets + divider when no text is ready yet.
      for (const b of deco.brackets) {
        b.hMesh.visible = false;
        b.vMesh.visible = false;
      }
      deco.divider.mesh.visible = false;
      return;
    }
    minX -= deco.padX; maxX += deco.padX;
    minY -= deco.padY; maxY += deco.padY;
    const ARM = 0.42;
    const TH  = 0.045;
    for (const b of deco.brackets) {
      const cx = b.cornerX < 0 ? minX : maxX;
      const cy = b.cornerY < 0 ? minY : maxY;
      b.hMesh.position.set(cx - b.cornerX * ARM / 2, cy, 0);
      b.vMesh.position.set(cx, cy - b.cornerY * ARM / 2, 0);
      b.hMesh.scale.set(ARM, TH, 0.02);
      b.vMesh.scale.set(TH, ARM, 0.02);
      b.hMesh.visible = true;
      b.vMesh.visible = true;
    }
    // Divider — horizontal line through the vertical midpoint.
    const cx = (minX + maxX) / 2;
    const cy = (minY + maxY) / 2;
    const w  = Math.max(0.5, (maxX - minX) - 0.6);
    deco.divider.mesh.position.set(cx, cy, 0);
    deco.divider.mesh.scale.set(w, 0.03, 0.02);
    deco.divider.mesh.visible = true;
  }

  /** 12 short radial tick marks around the outer ring (clock-style). */
  private buildOuterTickMarks(): void {
    const tickLen = 0.45;
    const tickThick = 0.05;
    const ringR = 6.34 + 0.15;
    const mat = new THREE.MeshBasicMaterial({
      color: TEAL_MID, transparent: true, opacity: 0.60,
      depthWrite: false, blending: THREE.AdditiveBlending,
    });
    for (let i = 0; i < 12; i++) {
      const angle = (i / 12) * Math.PI * 2;
      const geo = new THREE.BoxGeometry(tickLen, tickThick, 0.02);
      const mesh = new THREE.Mesh(geo, mat);
      // Position on the ring at this angle (in tiltGroup local space —
      // tiltGroup applies the orbital plane tilt).
      const cx = Math.cos(angle) * ringR;
      const cz = Math.sin(angle) * ringR;
      mesh.position.set(cx, 0, cz);
      // Orient the tick so its long axis points radially outward.
      mesh.rotation.y = -angle;
      mesh.layers.set(ORBIT_OVERLAY_LAYER);
      this.tiltGroup.add(mesh);
      this.uiDecorations.push({ mesh, geo, mat, baseOpacity: 0.60 });
    }
  }

  /** 2 decorative outer arcs — partial tori orbiting outside the
   *  outermost ring, at fixed angles. Adds visual interest without
   *  adding more "data" surface. */
  private buildDecorativeArcs(): void {
    const arcRadius = 6.34 + 0.55;
    const arcConfigs: Array<{ angle: number; arcSpan: number; opacity: number }> = [
      { angle: Math.PI * 0.12, arcSpan: Math.PI * 0.22, opacity: 0.45 },
      { angle: Math.PI * 1.18, arcSpan: Math.PI * 0.30, opacity: 0.35 },
    ];
    for (const cfg of arcConfigs) {
      const geo = new THREE.TorusGeometry(arcRadius, 0.025, 4, 60, cfg.arcSpan);
      const mat = new THREE.MeshBasicMaterial({
        color: TEAL_BRIGHT, transparent: true, opacity: cfg.opacity,
        depthWrite: false, blending: THREE.AdditiveBlending,
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.rotation.x = Math.PI / 2;     // lie flat on the orbital plane
      mesh.rotation.z = cfg.angle;
      mesh.layers.set(ORBIT_OVERLAY_LAYER);
      this.tiltGroup.add(mesh);
      this.uiDecorations.push({ mesh, geo, mat, baseOpacity: cfg.opacity });
    }
  }

  private buildMarker(
    key: MandalaAnchorKey,
    geoFactory: () => THREE.BufferGeometry,
    localOffset: THREE.Vector3,
    parent: THREE.Group,
  ): void {
    const sourceGeo = geoFactory();
    // Wireframe edges — crisp, below bloom threshold. EdgesGeometry
    // works for any position-attribute geometry; we ditch the source
    // geo immediately after building edges.
    const edgesGeo = new THREE.EdgesGeometry(sourceGeo, 1);
    sourceGeo.dispose();
    const mat = new THREE.LineBasicMaterial({
      color: TEAL_BRIGHT,
      transparent: true,
      opacity: 0.85,
      depthWrite: false,
    });
    const segments = new THREE.LineSegments(edgesGeo, mat);
    segments.layers.set(ORBIT_OVERLAY_LAYER);

    const grp = new THREE.Group();
    grp.position.copy(localOffset);
    grp.add(segments);
    parent.add(grp);

    this.markers.set(key, {
      group: grp,
      segments,
      geo: edgesGeo,
      mat,
      localOffset: localOffset.clone(),
      key,
    });
  }

  private buildTendril(
    key: MandalaAnchorKey,
    parent: THREE.Group,
    archAxis: 'horizontal' | 'vertical',
  ): void {
    // End point = marker offset; control = midpoint pulled out
    // perpendicular to the marker's axis so the tube arches.
    //   horizontal markers (E/W in orbital plane) → arch in +y
    //   vertical markers (N/S/NW in cross plane)  → arch in +z
    const m = this.markers.get(key);
    if (!m) return;
    const start = new THREE.Vector3(0, 0, 0);
    const end = m.localOffset.clone();
    const mid = start.clone().add(end).multiplyScalar(0.5);
    if (archAxis === 'horizontal') mid.y += TENDRIL_ARCH_LIFT;
    else                            mid.z += TENDRIL_ARCH_LIFT;
    const curve = new THREE.QuadraticBezierCurve3(start, mid, end);
    const geo = new THREE.TubeGeometry(
      curve,
      TENDRIL_TUBULAR_SEGS,
      TENDRIL_TUBE_RADIUS,
      TENDRIL_RADIAL_SEGS,
      false,
    );
    const mat = new THREE.MeshBasicMaterial({
      color: TEAL_BRIGHT,
      transparent: true,
      opacity: TENDRIL_PULSE_BASE,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.layers.set(ORBIT_OVERLAY_LAYER);
    parent.add(mesh);
    this.tendrils.set(key, { mesh, geo, mat, key, curve });
  }

  private refreshRingInner(): void {
    // Filled arc: 0 → 2π·progress. Start at top.
    const span = this.progress * Math.PI * 2;
    const start = -Math.PI / 2;
    const end = start + span;
    const seg = Math.max(1, Math.ceil(ARC_MAX_SEGMENTS * Math.max(0.001, this.progress)));
    updateArcGeometry(this.ringInner.geo, start, end, seg, TEAL_BRIGHT);
  }

  private refreshRingMid(): void {
    updateGateSegmentsGeometry(this.ringMid.geo, this.gates, 5);
  }

  private refreshRingOuter(): void {
    // Full ring, uniform teal.
    updateArcGeometry(
      this.ringOuter.geo,
      0, Math.PI * 2,
      ARC_MAX_SEGMENTS,
      TEAL_BRIGHT,
    );
  }
}

// -----------------------------------------------------------------
// Marker geometry factories.
// -----------------------------------------------------------------

function HEX_PRISM_GEO(): THREE.BufferGeometry {
  // Cylinder with 6 radial segments + 1 height segment = hex prism.
  return new THREE.CylinderGeometry(HEX_RADIUS, HEX_RADIUS, HEX_RADIUS * 1.6, 6, 1, false);
}
function PILLAR_GEO(): THREE.BufferGeometry {
  return new THREE.BoxGeometry(PILLAR_W, PILLAR_H, PILLAR_W);
}
function OCTA_GEO(): THREE.BufferGeometry {
  return new THREE.OctahedronGeometry(OCTA_RADIUS, 0);
}
function TETRA_GEO(): THREE.BufferGeometry {
  return new THREE.TetrahedronGeometry(TETRA_RADIUS, 0);
}

// -----------------------------------------------------------------
// Anchor-projection helpers.
// -----------------------------------------------------------------

export interface ScreenAnchor {
  x: number;
  y: number;
  visible: boolean;
}

export interface MandalaAnchors {
  node: ScreenAnchor;
  mainPanel: ScreenAnchor;
  n: ScreenAnchor;
  e: ScreenAnchor;
  s: ScreenAnchor;
  w: ScreenAnchor;
  nw: ScreenAnchor;
}

export function emptyAnchors(): MandalaAnchors {
  const z = (): ScreenAnchor => ({ x: 0, y: 0, visible: false });
  return {
    node: z(),
    mainPanel: z(),
    n: z(), e: z(), s: z(), w: z(), nw: z(),
  };
}

const _projectScratch = new THREE.Vector3();
const _mainAnchorScratch = new THREE.Vector3();
const _markerWorldScratch = new THREE.Vector3();

function project(
  world: THREE.Vector3,
  camera: THREE.Camera,
  canvasW: number,
  canvasH: number,
  scratch: THREE.Vector3,
  out: ScreenAnchor,
): void {
  scratch.copy(world);
  scratch.project(camera);
  const visible =
    scratch.z > -1 && scratch.z < 1
    && scratch.x > -1.05 && scratch.x < 1.05
    && scratch.y > -1.05 && scratch.y < 1.05;
  out.x = (scratch.x * 0.5 + 0.5) * canvasW;
  out.y = (1 - (scratch.y * 0.5 + 0.5)) * canvasH;
  out.visible = visible;
}

// Re-export buildArcRingGeometry so external callers can build their
// own arcs without re-importing planet-scan.ts. Keeps node-mandala's
// public surface self-contained.
export { buildArcRingGeometry };
