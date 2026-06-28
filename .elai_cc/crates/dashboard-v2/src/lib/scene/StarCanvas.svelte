<script lang="ts">
  // StarCanvas — three.js-backed depthful star backdrop.
  //
  // Replaces the Canvas2D renderer with a real 3D scene:
  //   • THREE.Points per layer with a custom ShaderMaterial. Per-star
  //     twinkle is computed in the vertex shader (zero CPU cost).
  //   • Perspective-correct size attenuation: far stars shrink with
  //     distance, near stars stay big. True 3D parallax falls out for
  //     free as the camera shifts.
  //   • UnrealBloomPass (post-processing) gives HDR-bright objects a
  //     procedural corona — stars below the threshold render crisp; a
  //     centred HDR "sun" sprite blooms.
  //   • Reduced-motion: the shader uniform `uReducedMotion=1` freezes
  //     twinkle; the shoot scheduler is suppressed.
  //
  // The pure-TS data model (`star-state.ts`, `star-shoot.ts`) is unchanged
  // — three.js consumes its `StarSlot[]` arrays, syncing slot lifecycle
  // mutations back to GPU buffers each frame.

  import { onMount, onDestroy } from 'svelte';
  import * as THREE from 'three';
  import {
    buildStarField,
    DEFAULT_SEED,
    type Density,
    type Tone,
    type StarField,
    type StarSlot,
  } from './star-state.ts';
  import {
    makeScheduler,
    tickScheduler,
    type ShootScheduler,
    type Mode,
    type Shooter,
  } from './star-shoot.ts';
  import { createThreeScene, type ThreeSceneRefs } from './three-scene.ts';
  import { buildAllStarLayers, type StarPointsLayer } from './star-points.ts';
  import { buildSkyDome, type SkyDomeRefs } from './sky-dome.ts';
  import {
    buildThreeOrbit,
    DEFAULT_ORBIT_CONFIG,
    CONCEPT_HALO_OUTER_SCALE,
    CONCEPT_HALO_INNER_SCALE,
    type ThreeOrbitRefs,
    type OrbitNodeRef,
  } from './three-orbit.ts';
  import { buildThreeMap, buildThreeResearch } from './three-orbit-modes.ts';
  import { PlanetScan, defaultGlyphsFor, progressFrom, nodeRadiusOf, type GateState } from './planet-scan.ts';
  import { NodeMandala, emptyAnchors, emptyTextContent, type MandalaAnchors, type MandalaTextContent } from '../orbit/node-mandala.ts';
  import type { OrbitFixture, OrbitMode, ZoomLevel } from '../orbit/types.ts';

  interface Props {
    density?: Density;
    tone?: Tone;
    centered?: boolean;
    seed?: number;
    mode?: Mode;
    /** Show the three.js orbit (galaxy of nodes + central HDR sun on a
     *  flat plane). The camera (pitched above the plane) does the
     *  tilting — the orbit geometry stays flat. */
    showOrbit?: boolean;
    /** Fixture supplies node positions for the three.js galaxy. When
     *  not given, only the sun renders. */
    orbitFixture?: OrbitFixture;
    /** Camera pitch (degrees below horizontal). 0 = flat side view; 90
     *  = top-down. Default 40 — "looking down at a tilted disc". */
    cameraPitchDegrees?: number;
    /** Camera distance from origin in world units. Smaller = closer to
     *  the sun ("leap" effect). Default takes scene's defaultDistance. */
    cameraDistance?: number;
    mouseX?: number;
    mouseY?: number;
    /** When false, skip projecting node world positions to screen and
     *  do not render the HTML label overlay. Default false — the user
     *  prefers an unlabeled galaxy with the caption below the canvas
     *  doing all the naming. */
    showLabels?: boolean;
    /** Current zoom level — drives which node the camera focuses on.
     *  concept = wide shot; phase/step/baby = fly to that active node. */
    zoomLevel?: ZoomLevel;
    /** Which 3D builder produces the orbit geometry: 'plan' (default,
     *  decomposition tree), 'map' (codebase galaxy), 'research'
     *  (tendril paths). Plan mode is the legacy path. */
    orbitMode?: OrbitMode;
    /** Optional fly-to-node target id. When set, the chase camera
     *  resolves to this node instead of the active phase/step/baby.
     *  Cleared by the parent after the warp finishes. */
    flyToNodeId?: string | null;
    /** Notify when a node mesh is clicked. Receives the node id +
     *  world coords so the parent can issue a flyToNodeId. */
    onNodeClick?: (id: string, x: number, y: number, z: number) => void;
    /** Notify on raycaster hover transitions. id=null means hover-out.
     *  type is the node tier (plan mode) or 'phase' for non-plan modes
     *  (map/research store every node as 'phase' in OrbitNodeRef). */
    onNodeHover?: (id: string | null, type: 'phase' | 'step' | 'baby' | null) => void;
    /** When set, StarCanvas projects this node's world position to canvas-
     *  local pixel coordinates each frame and pushes them via
     *  `onPanelTrack` so OrbitCanvas can anchor the NodePanel overlay.
     *  Special id '__sun__' resolves to the world origin (sun). */
    panelTrackNodeId?: string | null;
    /** Per-frame projection callback driven by `panelTrackNodeId`.
     *  visible=false when the node is behind the camera or off-screen. */
    onPanelTrack?: (x: number, y: number, visible: boolean) => void;
    /** Per-frame projected anchors for the V3 hybrid NodePanel:
     *  centre node, main-panel anchor, + 4-or-5 compass markers. */
    onMandalaAnchors?: (anchors: MandalaAnchors) => void;
    /** Variant hints for the NodeMandala — drive the 5th NW marker
     *  (integration-point) and tier-coloured markers (research). */
    mandalaShowNwMarker?: boolean;
    mandalaTierColored?: boolean;
    /** Flat 3D-text payload for the mandala labels (title / subtitle /
     *  status / fileRef / liveAction / etc). Empty strings hide the
     *  corresponding 3D text element. */
    mandalaText?: MandalaTextContent;
    /** Fired (with debouncing) when the chase camera enters/leaves a
     *  warp. Used by the parent to gate DOM overlays (e.g. NodePanel)
     *  on warp completion so they don't fade in mid-flight. Single
     *  source of truth for warp duration is WARP_DURATION_MS in this
     *  component — consumers should NOT hard-code it; subscribe here. */
    onWarpStateChange?: (warping: boolean) => void;
    /** Trigger the "product done" super-sun sequence. Set true when
     *  fsm.state === 'COMPLETE'. Animation runs once per transition. */
    productDone?: boolean;
    /** Show G6 perf overlay (fps, draw calls, triangles). Toggled by
     *  the parent on `~` key (only in dev). */
    showPerfOverlay?: boolean;
  }

  let {
    density = 'normal',
    tone = 'mixed',
    centered = false,
    seed = DEFAULT_SEED,
    mode = 'active',
    showOrbit = true,
    orbitFixture,
    cameraPitchDegrees = 12,
    cameraDistance,
    mouseX = 0,
    mouseY = 0,
    showLabels = false,
    zoomLevel = 'concept',
    orbitMode = 'plan',
    flyToNodeId = null,
    onNodeClick,
    onNodeHover,
    panelTrackNodeId = null,
    onPanelTrack,
    onMandalaAnchors,
    mandalaShowNwMarker = false,
    mandalaTierColored = false,
    mandalaText = emptyTextContent(),
    onWarpStateChange,
    productDone = false,
    showPerfOverlay = false,
  }: Props = $props();

  // Warp emit dedupe — emit only on transition. Tracks the last value
  // pushed to onWarpStateChange so we don't spam the callback every
  // frame. Pre-init to false matches the initial "not warping" state at
  // mount; the first start-warp call will flip to true and emit once.
  let _lastWarpEmit = false;
  function emitWarpState(warping: boolean): void {
    if (warping === _lastWarpEmit) return;
    _lastWarpEmit = warping;
    onWarpStateChange?.(warping);
  }

  let containerEl = $state<HTMLDivElement | null>(null);
  let reducedMotion = $state(false);

  // HTML label overlay state. Each entry carries the projected screen
  // coords (in CSS pixels) and the label text. The label is hidden when
  // the node is behind the camera or culled. Updated each frame from
  // updateLabelProjections().
  interface LabelEntry {
    id: string;
    text: string;
    type: 'phase' | 'step' | 'baby';
    isActive: boolean;
    x: number;
    y: number;
    visible: boolean;
  }
  let labels = $state<LabelEntry[]>([]);

  function updateLabelProjections(): void {
    if (!showLabels) {
      if (labels.length > 0) labels = [];
      return;
    }
    if (!three || !orbit || !containerEl || !orbitFixture) return;
    const rect = containerEl.getBoundingClientRect();
    const w = rect.width;
    const h = rect.height;
    const apId = orbitFixture.active_phase_id;
    const asId = orbitFixture.active_step_id;
    const abId = orbitFixture.active_baby_step_id;
    // Find concept-side label lookup. Labels use the fixture's concept
    // hierarchy so we get human-readable strings, not ids.
    const concept = orbitFixture.concepts.find((c) => c.id === orbitFixture.active_concept_id) ?? orbitFixture.concepts[0];
    if (!concept) return;
    const phaseById = new Map<string, string>();
    const stepById = new Map<string, string>();
    const babyById = new Map<string, string>();
    for (const p of concept.phases) {
      phaseById.set(p.id, p.label);
      for (const s of p.steps) {
        stepById.set(s.id, s.label);
        for (const b of s.baby_steps) babyById.set(b.id, b.label);
      }
    }
    const tmp = new THREE.Vector3();
    const out: LabelEntry[] = [];
    for (const n of orbit.nodes) {
      // Only label active branch + active phase/step's siblings (avoid
      // a wall of text). Phases always shown; steps only for active
      // phase; babies only for active step.
      let text: string | undefined;
      let isActive = false;
      if (n.type === 'phase') {
        text = phaseById.get(n.id);
        isActive = n.id === apId;
      } else if (n.type === 'step') {
        // Only show step labels under the active phase.
        const parentPhase = concept.phases.find((p) => p.steps.some((s) => s.id === n.id));
        if (parentPhase && parentPhase.id !== apId) continue;
        text = stepById.get(n.id);
        isActive = n.id === asId;
      } else {
        // Only show baby labels under the active step.
        const parentStep = concept.phases.flatMap((p) => p.steps).find((s) => s.baby_steps.some((b) => b.id === n.id));
        if (parentStep && parentStep.id !== asId) continue;
        text = babyById.get(n.id);
        isActive = n.id === abId;
      }
      if (!text) continue;
      tmp.set(n.worldX, n.worldY, n.worldZ);
      tmp.project(three.camera);
      // tmp is now in NDC. Cull if outside frustum or behind camera.
      const visible = tmp.z > -1 && tmp.z < 1 && tmp.x > -1.05 && tmp.x < 1.05 && tmp.y > -1.05 && tmp.y < 1.05;
      const sx = (tmp.x * 0.5 + 0.5) * w;
      const sy = (1 - (tmp.y * 0.5 + 0.5)) * h;
      out.push({ id: n.id, text, type: n.type, isActive, x: sx, y: sy, visible });
    }
    labels = out;
  }

  let three: ThreeSceneRefs | null = null;
  let layers: StarPointsLayer[] = [];
  let field: StarField | null = null;
  let scheduler: ShootScheduler | null = null;
  let orbit: ThreeOrbitRefs | null = null;
  // Planet-scan layer: Saturn rings + tech-glyph billboards around the
  // active focal node. Only visible at the deepest zoom ('baby-step').
  // Mounted once into three.scene when the scene boots; rebuilds glyphs
  // each time the focal node changes.
  let planetScan: PlanetScan | null = null;
  // V3 hybrid NodePanel: NodeMandala wraps the focal node when a panel
  // is bound. Rebuilt when panelTrackNodeId or orbitMode changes;
  // disposed on unfocus and unmount.
  let nodeMandala: NodeMandala | null = null;
  let _lastMandalaNodeId: string | null = null;
  let _lastMandalaMode: OrbitMode | null = null;
  // Reusable anchors object — emitted to the parent each frame.
  const _mandalaAnchors: MandalaAnchors = emptyAnchors();
  let skydome: SkyDomeRefs | null = null;
  let shooterMesh: THREE.Points | null = null;
  let shooterGeo: THREE.BufferGeometry | null = null;
  let shooterMat: THREE.PointsMaterial | null = null;
  let colorMap: Record<string, string> = {};
  let rafId: number | null = null;
  let startMs = 0;
  let lastMs = 0;

  // Mouse-driven camera offset (small, subtle look-around). Lowered from
  // 22 → 12; the previous gain produced visible per-pixel popping as the
  // raw mouse delta fed straight into camera.position. Smoothed below.
  const MOUSE_CAMERA_GAIN = 12;
  // Smoothed mouse offsets — lerp toward raw mouseX/mouseY each frame so
  // jittery hardware input or fast cursor moves don't translate 1:1 into
  // camera position changes. Time-based rate (per-second) — see tick().
  let smoothMouseX = 0;
  let smoothMouseY = 0;
  const MOUSE_RATE = 6.3;       // mouse smoothing half-life ≈ 110 ms
  const CAM_RATE   = 1.21;      // chase-cam settle half-life ≈ 575 ms
  const ZOOM_RATE  = 2.45;      // zoom-depth ease half-life ≈ 282 ms

  // Chase-camera state. smoothFocus = lookAt point (the active node);
  // smoothCamPos = camera world position (previous node's area, slightly
  // above-and-behind). Each frame we resolve targets from the current
  // zoomLevel and either:
  //   (a) ease through a fixed-duration warp (after a zoomLevel change)
  //   (b) slow-lerp residually toward the resolved target (idle settle)
  // The warp uses easeInOutQuint over 1.8s — flat start, sharp middle,
  // flat end — to give the "build up speed → cruise → decel" feel of
  // covering a long distance in space. Plain lerp(0.02) felt like a
  // glide; the eased warp feels like a leap.
  const smoothFocus = new THREE.Vector3(0, 0, -350);
  const smoothCamPos = new THREE.Vector3(0, 180, 400);
  const _focusTarget = new THREE.Vector3();
  const _camPosTarget = new THREE.Vector3();

  // Warp state — populated by the zoomLevel $effect, consumed in tick.
  const WARP_DURATION_MS = 1800;
  const _warpFromCam = new THREE.Vector3();
  const _warpFromFocus = new THREE.Vector3();
  const _warpToCam = new THREE.Vector3();
  const _warpToFocus = new THREE.Vector3();
  // Reused per-frame direction vector — camera → sun (world origin).
  // Allocated once to avoid GC churn in the rAF tick.
  const _sunDir = new THREE.Vector3();
  let warpStartMs = -1;        // -1 = not warping
  let prevZoomLevel: ZoomLevel | null = null;

  // easeInOutQuint — t∈[0,1]. 0 at t=0, 1 at t=1; flat-fast-flat curve.
  function easeInOutQuint(t: number): number {
    return t < 0.5
      ? 16 * t * t * t * t * t
      : 1 - Math.pow(-2 * t + 2, 5) / 2;
  }

  // Per-zoom-level "perspective depth" — what would have been the camera
  // travelling deeper into space if we weren't using a chase-cam. At
  // concept zoom the camera sits beside the sun, so the depth feeling
  // is shallow (0); at baby zoom the leap has carried us deep, so far
  // stars should feel like they've approached (1). Stars sample this
  // through uPerspectiveBoost (max with the in-flight warp progress)
  // and amplify their size proportionally per layer.
  const ZOOM_DEPTH: Record<ZoomLevel, number> = {
    concept:    0.0,
    phase:      0.40,
    step:       0.70,
    'baby-step': 1.00,
  };
  let smoothZoomDepth = 0;

  // Subject-relative offsets. Every zoom level uses the SAME framing
  // as the concept zoom (which the user approved), just re-centred on
  // a different subject:
  //   concept → subject = sun (0,0,0)
  //   phase   → subject = active phase node
  //   step    → subject = active step node
  //   baby    → subject = active baby node
  // Camera ends up 180 above and 400 behind the subject; focus is
  // 350 ahead — same numbers as the working concept shot, so each
  // zoom level reads like "the concept view, but the SUBJECT is now
  // this node".
  const SUBJECT_CAM_DY = 180;
  const SUBJECT_CAM_DZ = 400;
  const SUBJECT_FOCUS_DZ = -350;

  function resolveCameraTargets(focusOut: THREE.Vector3, camPosOut: THREE.Vector3): void {
    let sx = 0, sy = 0, sz = 0;
    if (orbit && orbitFixture && zoomLevel !== 'concept') {
      const wantedType = zoomLevel === 'phase' ? 'phase'
                       : zoomLevel === 'step'  ? 'step'
                       : 'baby';
      const targetId = zoomLevel === 'phase' ? orbitFixture.active_phase_id
                     : zoomLevel === 'step'  ? orbitFixture.active_step_id
                     : orbitFixture.active_baby_step_id;
      if (targetId) {
        const node = orbit.nodes.find((n) => n.type === wantedType && n.id === targetId);
        if (node) {
          sx = node.worldX;
          sy = node.worldY;
          sz = node.worldZ;
        }
      }
    }
    focusOut.set(sx, sy, sz + SUBJECT_FOCUS_DZ);
    camPosOut.set(sx, sy + SUBJECT_CAM_DY, sz + SUBJECT_CAM_DZ);
  }

  // -----------------------------------------------------------------
  // Planet-scan helpers — read fixture telemetry into ring/glyph inputs
  // and drive the layer's visibility from the current zoom + focal node.
  // -----------------------------------------------------------------
  // Scratch vector reused per frame to avoid GC churn.
  const _planetCentre = new THREE.Vector3();

  function activeFocalNode(): { id: string; ref: OrbitNodeRef } | null {
    if (!orbit || !orbitFixture) return null;
    // Plan mode: focal = active baby. Map / research: focal = whatever
    // node id resolves first by active_baby_step_id (rare in those modes,
    // so the layer typically stays hidden except in plan).
    const id = orbitFixture.active_baby_step_id;
    if (!id) return null;
    const ref = orbit.nodes.find((n) => n.id === id);
    if (!ref) return null;
    return { id, ref };
  }

  function buildPlanetScanInputs(focal: { id: string; ref: OrbitNodeRef }) {
    if (!orbitFixture) return null;
    const radius = nodeRadiusOf(focal.ref);
    _planetCentre.set(focal.ref.worldX, focal.ref.worldY, focal.ref.worldZ);
    const gates: GateState[] = (orbitFixture.evidence_gates ?? []).map((g) => ({
      passed: g.verdict === 'pass',
      label: g.name,
    }));
    // Fall back to 5 pending gates if the fixture doesn't carry them yet.
    const filledGates = gates.length > 0 ? gates : Array.from({ length: 5 }, () => ({ passed: false }));
    const passedCount = filledGates.filter((g) => g.passed).length;
    const progress = progressFrom({
      fsm: orbitFixture.fsm,
      evidenceTotal: filledGates.length,
      evidencePassed: passedCount,
    });
    // Resolve a concept name + step index for richer glyphs.
    const concept = orbitFixture.concepts.find((c) => c.id === orbitFixture?.active_concept_id)
                 ?? orbitFixture.concepts[0];
    const phase = concept?.phases.find((p) => p.id === orbitFixture?.active_phase_id);
    const step = phase?.steps.find((s) => s.id === orbitFixture?.active_step_id);
    let stepIndex: number | undefined;
    let stepTotal: number | undefined;
    if (phase && step) {
      const idx = phase.steps.findIndex((s) => s.id === step.id);
      if (idx >= 0) {
        stepIndex = idx + 1;
        stepTotal = phase.steps.length;
      }
    }
    const glyphs = defaultGlyphsFor({
      focalId: focal.id,
      fsm: orbitFixture.fsm,
      grcStep: orbitFixture.grc_step,
      evidenceTotal: filledGates.length,
      evidencePassed: passedCount,
      conceptName: concept?.name,
      phase: phase?.label,
      stepIndex,
      stepTotal,
    });
    return {
      centre: _planetCentre,
      radius,
      progress,
      gates: filledGates,
      glyphs,
    };
  }

  // Diagnostic — log only on transitions of nodeMandalaActive so devtools
  // shows when planet-scan is taking over vs ceding to the V3 mandala.
  let _lastPlanetScanActive: boolean | null = null;
  function updatePlanetScan(nowMs: number): void {
    if (!planetScan || !three) return;
    const focal = activeFocalNode();
    const mandalaActive = nodeMandala !== null;
    const wantsVisible =
      orbitMode === 'plan'
      && zoomLevel === 'baby-step'
      && focal !== null
      // When the V3 NodeMandala is bound, it owns the rings — hide the
      // legacy PlanetScan so the two layers don't double-up.
      && !mandalaActive;
    if (_lastPlanetScanActive !== wantsVisible) {
      _lastPlanetScanActive = wantsVisible;
      // eslint-disable-next-line no-console
      console.debug('[planet-scan] nodeMandalaActive=', mandalaActive,
        'wantsVisible=', wantsVisible, 'zoomLevel=', zoomLevel);
    }
    if (wantsVisible && focal) {
      const inputs = buildPlanetScanInputs(focal);
      if (inputs) planetScan.update(inputs, focal.id);
      planetScan.setVisible(true, nowMs);
    } else {
      planetScan.setVisible(false, nowMs);
    }
    planetScan.tick(nowMs);
    planetScan.faceCamera(three.camera);
  }

  // -----------------------------------------------------------------
  // NodeMandala — V3 hybrid NodePanel layer 1 (3D rings + compass
  // markers + tendrils). Rebuilt when the bound panelTrackNodeId or
  // orbitMode changes; disposed on unfocus / unmount. The DOM mini-cards
  // and main panel are anchored to the projected positions emitted from
  // here via onMandalaAnchors.
  // -----------------------------------------------------------------
  function resolveMandalaTarget(): { id: string; centre: THREE.Vector3; radius: number } | null {
    if (!three) return null;
    const id = panelTrackNodeId;
    if (!id) return null;
    if (id === '__sun__') {
      // Sun root — use a placeholder radius; visual scale matches the
      // sun's halo.
      return { id, centre: new THREE.Vector3(0, 0, 0), radius: 12 };
    }
    if (!orbit) return null;
    const ref = orbit.nodes.find((n) => n.id === id);
    if (!ref) return null;
    return {
      id,
      centre: new THREE.Vector3(ref.worldX, ref.worldY, ref.worldZ),
      radius: nodeRadiusOf(ref),
    };
  }

  function buildMandalaParams(target: { id: string; centre: THREE.Vector3; radius: number }) {
    const gates: GateState[] = (orbitFixture?.evidence_gates ?? []).map((g) => ({
      passed: g.verdict === 'pass',
      label: g.name,
    }));
    const filledGates = gates.length > 0 ? gates : Array.from({ length: 5 }, () => ({ passed: false }));
    const passedCount = filledGates.filter((g) => g.passed).length;
    const progress = progressFrom({
      fsm: orbitFixture?.fsm,
      evidenceTotal: filledGates.length,
      evidencePassed: passedCount,
    });
    // The center "%" slot is gone (overlapped the node sphere). We
    // pass-through the variant text as-is; the bar+percent on the
    // right side already encodes progress visibly.
    const text: MandalaTextContent = mandalaText;

    // Mandala radius — zoom-INVARIANT on screen.
    //
    // Default behaviour was `radius = nodeRadiusOf(node)`, which made
    // the mandala scale with the focal node's geometry — sun (huge)
    // got a huge mandala, baby-step (tiny) got a tiny mandala. The user
    // wants every zoom level (except concept with multi-concepts) to
    // show the mandala at roughly the same on-screen size. Compute the
    // world radius that puts the OUTER ring (6.34 × node radius) at
    // ~24% of half-screen-height regardless of camera distance.
    const cam = three?.camera as THREE.PerspectiveCamera | undefined;
    let consistentRadius = target.radius;
    if (cam && typeof cam.fov === 'number') {
      const camDist = cam.position.distanceTo(target.centre);
      const halfHeight = camDist * Math.tan((cam.fov * Math.PI) / 180 / 2);
      // 6.34 = RING_OUTER_OUTER (× node-radius units). Solve for radius
      // so the OUTER ring sits at ~38% of half-screen-height — that
      // gives the mandala enough breathing room for the text
      // readouts to flank it without overflowing the viewport edges.
      consistentRadius = halfHeight * 0.060;
    }

    return {
      centre: target.centre,
      radius: consistentRadius,
      progress,
      gates: filledGates,
      showNwMarker: mandalaShowNwMarker,
      tierColored: mandalaTierColored,
      text,
    };
  }

  function disposeMandalaIfAny(): void {
    if (nodeMandala) {
      nodeMandala.dispose();
      nodeMandala = null;
    }
    _lastMandalaNodeId = null;
    _lastMandalaMode = null;
  }

  // (Re)build / update the mandala based on current panelTrackNodeId.
  function syncMandala(nowMs: number): void {
    if (!three) return;
    const target = resolveMandalaTarget();
    if (!target) {
      // No bound node — fade out and (after fade) dispose so the next
      // hover starts crisp. Cheap: NodeMandala holds its own fade clock.
      if (nodeMandala) {
        nodeMandala.setVisible(false, nowMs);
        // Defer disposal until the fade-out completes — the tick loop
        // will check `currentOpacity ≈ 0` and reap below.
      }
      return;
    }
    // Rebuild on node-id or mode change so resources are clean.
    const needsRebuild =
      !nodeMandala
      || _lastMandalaNodeId !== target.id
      || _lastMandalaMode !== orbitMode;
    if (needsRebuild) {
      disposeMandalaIfAny();
      nodeMandala = new NodeMandala();
      nodeMandala.build(three.scene, buildMandalaParams(target));
      _lastMandalaNodeId = target.id;
      _lastMandalaMode = orbitMode;
    } else if (nodeMandala) {
      nodeMandala.update(buildMandalaParams(target));
    }
    if (!nodeMandala) return;
    // Warp gate: only START the fade-in once the camera has settled.
    // While warpStartMs is active, keep the mandala invisible so the
    // user sees the camera fly to the new node FIRST and THEN the UI
    // settle in. The tick loop below also detects warp completion and
    // calls setVisible(true) when warpStartMs goes back to -1.
    const isWarping = warpStartMs >= 0
      && (nowMs - warpStartMs) < WARP_DURATION_MS;
    if (isWarping) {
      nodeMandala.setVisible(false, nowMs);
    } else {
      nodeMandala.setVisible(true, nowMs);
    }
  }

  // Diagnostic flags — log only on transitions, not per frame.
  let _lastAnchorsEmittedAlive = false;
  function tickMandala(dt: number, nowMs: number, canvasW: number, canvasH: number): void {
    if (!three) return;
    if (nodeMandala) {
      nodeMandala.tick(dt, nowMs, three.camera);
      // Per-frame zoom-invariant scale: keep the mandala's outer ring
      // at the same fraction of viewport height regardless of camera
      // distance. Cheap — single Vector3 distance + scalar set.
      {
        const cam = three.camera as THREE.PerspectiveCamera;
        if (typeof cam.fov === 'number') {
          const camDist = cam.position.distanceTo(nodeMandala.group.position);
          const halfHeight = camDist * Math.tan((cam.fov * Math.PI) / 180 / 2);
          const r = halfHeight * 0.060;
          if (r > 0.01) nodeMandala.group.scale.setScalar(r);
        }
      }
      // Reap a fully-faded mandala — caller has unbound the panel and
      // the fade-out completed. Skip if still mid-fade.
      // Access opacity indirectly via group.visible (set false when
      // fadeToOpacity=0 settles).
      if (!nodeMandala.group.visible && !panelTrackNodeId) {
        disposeMandalaIfAny();
      }
    }
    if (onMandalaAnchors && nodeMandala) {
      // tiltGroup matrices need to be current for marker world-space
      // computation. Three.js auto-updates on render, but we project
      // BEFORE render — force a one-shot update so the projection sees
      // the mandala's actual world transforms.
      nodeMandala.group.updateMatrixWorld(true);
      nodeMandala.computeAnchors(three.camera, canvasW, canvasH, _mandalaAnchors);
      onMandalaAnchors(_mandalaAnchors);
      if (!_lastAnchorsEmittedAlive) {
        _lastAnchorsEmittedAlive = true;
        // eslint-disable-next-line no-console
        console.debug('[mandala] anchors', JSON.parse(JSON.stringify(_mandalaAnchors)));
      }
    } else if (onMandalaAnchors) {
      // Push an "all hidden" frame so the parent fades the DOM out.
      _mandalaAnchors.node.visible = false;
      _mandalaAnchors.mainPanel.visible = false;
      _mandalaAnchors.n.visible = false;
      _mandalaAnchors.e.visible = false;
      _mandalaAnchors.s.visible = false;
      _mandalaAnchors.w.visible = false;
      _mandalaAnchors.nw.visible = false;
      onMandalaAnchors(_mandalaAnchors);
      if (_lastAnchorsEmittedAlive) {
        _lastAnchorsEmittedAlive = false;
        // eslint-disable-next-line no-console
        console.debug('[mandala] anchors hidden (no mandala bound)');
      }
    }
  }

  // React to panelTrackNodeId / orbitMode / variant-flag changes — rebuild
  // or update the mandala accordingly. The actual tick happens inside the
  // rAF loop (we just kick the build here).
  //
  // CRITICAL: read reactive props FIRST, guard on `three` SECOND. Svelte 5
  // $effect only tracks deps actually read during the run; if we early-
  // return on `!three` before touching the props, the effect never tracks
  // them and never re-fires when `panelTrackNodeId` changes after mount.
  // That bug left the mandala unbuilt and the legacy planet-scan visible
  // because syncMandala() was never invoked post-mount.
  $effect(() => {
    // Read the reactive props so Svelte tracks them — must run before any
    // early return.
    const id   = panelTrackNodeId;
    const mode = orbitMode;
    void mandalaShowNwMarker;
    void mandalaTierColored;
    // mandalaText updates whenever the variant content changes — track
    // every field so a hover-swap (which replaces the whole text object)
    // re-runs syncMandala and pushes the new strings into the troika
    // labels.
    void mandalaText.title;
    void mandalaText.subtitle;
    void mandalaText.breadcrumb;
    void mandalaText.status;
    void mandalaText.elapsed;
    void mandalaText.fileRef;
    void mandalaText.liveAction;
    void mandalaText.meta;
    if (!three) return;
    // eslint-disable-next-line no-console
    console.debug('[mandala] build', { nodeId: id, hasNode: !!id, mode });
    syncMandala(performance.now());
  });

  // Force re-evaluation of the mandala once the scene is constructed.
  // `three` and `orbit` are plain locals (not $state) so the effect above
  // can't track them — we toggle a tiny $state flag at the end of the
  // mount sequence to invalidate any reactive consumers that early-
  // returned during the first run.
  let _sceneReadyTick = $state(0);
  $effect(() => {
    void _sceneReadyTick;
    if (!three) return;
    syncMandala(performance.now());
  });

  // ----- FPS + GPU stats HUD ---------------------------------------
  // Only measures THIS renderer's work — independent of other GPU
  // users (games, LLM inference).
  // GPU time uses WebGL timer-query extensions when available;
  // browsers / drivers sometimes disable them (Chromium without
  // cross-origin isolation, older webviews). The fallback is a
  // performance.now() bracket around render() — that's CPU command-
  // submission time, labelled "cpu" so you don't misread it as GPU.
  let fpsDisplay = $state('— fps');
  let gpuDisplay = $state('— gpu');
  let perfDisplay = $state('');
  let lastFrameStamp = 0;
  const FPS_BUFFER_LEN = 30;
  const fpsBuffer: number[] = [];

  // ----- Perf debug overlay --------------------------------------------
  // Activated by `?perf=1` URL param. When on, the tick brackets each
  // major section with performance.now() and rolls timings through 120-
  // frame buffers. Section breakdown surfaces WHICH part of the frame
  // is spiking — without it, "fps 80 / gpu 4ms" hides everything that
  // isn't the render call itself. Spikes (frames > 1.8× rolling median)
  // are console.warn'd with a per-section breakdown so you can copy-
  // paste the bad frames into the next iteration.
  // Cost when off: zero — every code path is gated on PERF_DEBUG.
  // Note: Firefox quantizes performance.now() to 1–2 ms by default
  // (Spectre mitigation). Set `privacy.reduceTimerPrecision = false`
  // in about:config for sub-ms resolution while diagnosing.
  let PERF_DEBUG = false;
  let DISABLE_BLOOM = false;     // ?nobloom=1
  let FRAME_CAP_HZ = 0;          // ?cap=N (0 = uncapped)
  if (typeof window !== 'undefined') {
    const qs = new URLSearchParams(window.location.search);
    if (qs.get('perf') === '1' || window.localStorage?.getItem('elai-perf-debug') === '1') {
      PERF_DEBUG = true;
    }
    if (qs.get('nobloom') === '1') DISABLE_BLOOM = true;
    const cap = parseInt(qs.get('cap') ?? '0', 10);
    if (cap >= 24 && cap <= 240) FRAME_CAP_HZ = cap;
  }
  const PERF_BUFFER_LEN = 120;
  const perfTick: number[] = [];
  const perfSyncSlots: number[] = [];
  const perfSyncShoot: number[] = [];
  const perfCamHalo: number[] = [];
  const perfRender: number[] = [];
  const perfFrameTotal: number[] = [];
  let spikeCount = 0;
  let lastRenderTs = 0;          // for the optional frame cap

  function perfPush(buf: number[], v: number): void {
    buf.push(v);
    if (buf.length > PERF_BUFFER_LEN) buf.shift();
  }
  function perfStats(buf: number[]): { avg: number; max: number; p99: number } {
    if (buf.length === 0) return { avg: 0, max: 0, p99: 0 };
    let sum = 0, max = 0;
    for (const v of buf) { sum += v; if (v > max) max = v; }
    const sorted = [...buf].sort((a, b) => a - b);
    const p99 = sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * 0.99))];
    return { avg: sum / buf.length, max, p99 };
  }
  function perfMedian(buf: number[]): number {
    if (buf.length === 0) return 0;
    const sorted = [...buf].sort((a, b) => a - b);
    return sorted[Math.floor(sorted.length / 2)];
  }
  function fmt(n: number): string {
    return n.toFixed(2).padStart(5, ' ');
  }
  function updatePerfDisplay(): void {
    if (!PERF_DEBUG) return;
    const f = perfStats(perfFrameTotal);
    const t = perfStats(perfTick);
    const ss = perfStats(perfSyncSlots);
    const sh = perfStats(perfSyncShoot);
    const ch = perfStats(perfCamHalo);
    const r  = perfStats(perfRender);
    perfDisplay =
      `frame  avg ${fmt(f.avg)} p99 ${fmt(f.p99)} max ${fmt(f.max)}\n` +
      `tick   avg ${fmt(t.avg)} p99 ${fmt(t.p99)} max ${fmt(t.max)}\n` +
      `slots  avg ${fmt(ss.avg)} p99 ${fmt(ss.p99)} max ${fmt(ss.max)}\n` +
      `shoot  avg ${fmt(sh.avg)} p99 ${fmt(sh.p99)} max ${fmt(sh.max)}\n` +
      `cam/h  avg ${fmt(ch.avg)} p99 ${fmt(ch.p99)} max ${fmt(ch.max)}\n` +
      `render avg ${fmt(r.avg)} p99 ${fmt(r.p99)} max ${fmt(r.max)}\n` +
      `spikes ${spikeCount}` +
      (DISABLE_BLOOM ? ' [bloom OFF]' : '') +
      (FRAME_CAP_HZ ? ` [cap ${FRAME_CAP_HZ}Hz]` : '');
  }
  type TimerExt = { TIME_ELAPSED_EXT: number; GPU_DISJOINT_EXT: number };
  let gpuExt: TimerExt | null = null;
  let gpuQueries: (WebGLQuery | null)[] = [];
  let gpuQueryHead = 0;
  const GPU_QUERY_COUNT = 4;
  const GPU_BUFFER_LEN = 30;
  const gpuBuffer: number[] = [];
  let gpuLabel: 'gpu' | 'cpu' = 'gpu';
  let cpuRenderStart = 0;
  let statsFrameCount = 0;

  function gpuStatsSetup(gl: WebGLRenderingContext | WebGL2RenderingContext): void {
    // Try the WebGL2 name first, then the WebGL1 name (some drivers
    // expose the older extension even on a WebGL2 context).
    const ext = (gl.getExtension('EXT_disjoint_timer_query_webgl2')
              ?? gl.getExtension('EXT_disjoint_timer_query')) as TimerExt | null;
    if (!ext || typeof (gl as WebGL2RenderingContext).createQuery !== 'function') {
      // No GPU timer queries — fall back to CPU command-time bracket.
      gpuLabel = 'cpu';
      return;
    }
    gpuExt = ext;
    const gl2 = gl as WebGL2RenderingContext;
    gpuQueries = [];
    for (let i = 0; i < GPU_QUERY_COUNT; i++) gpuQueries.push(gl2.createQuery());
  }

  function gpuStatsBegin(gl: WebGL2RenderingContext): void {
    if (gpuExt) {
      const q = gpuQueries[gpuQueryHead % GPU_QUERY_COUNT];
      if (q) gl.beginQuery(gpuExt.TIME_ELAPSED_EXT, q);
    } else {
      // CPU fallback: stamp before issuing render commands.
      cpuRenderStart = performance.now();
    }
  }

  function gpuStatsEnd(gl: WebGL2RenderingContext): void {
    if (gpuExt) {
      gl.endQuery(gpuExt.TIME_ELAPSED_EXT);
      // Read the oldest query — most likely to have its result ready.
      const oldestIdx = (gpuQueryHead + 1) % GPU_QUERY_COUNT;
      const oldQ = gpuQueries[oldestIdx];
      if (oldQ) {
        const available = gl.getQueryParameter(oldQ, gl.QUERY_RESULT_AVAILABLE);
        const disjoint = gl.getParameter(gpuExt.GPU_DISJOINT_EXT);
        if (available && !disjoint) {
          const ns = gl.getQueryParameter(oldQ, gl.QUERY_RESULT) as number;
          const ms = ns / 1_000_000;
          gpuBuffer.push(ms);
          if (gpuBuffer.length > GPU_BUFFER_LEN) gpuBuffer.shift();
        }
      }
      gpuQueryHead++;
    } else {
      // CPU fallback: record time elapsed in the JS render call.
      const ms = performance.now() - cpuRenderStart;
      gpuBuffer.push(ms);
      if (gpuBuffer.length > GPU_BUFFER_LEN) gpuBuffer.shift();
    }
  }

  function gpuStatsDispose(gl: WebGL2RenderingContext): void {
    for (const q of gpuQueries) if (q) gl.deleteQuery(q);
    gpuQueries = [];
    gpuExt = null;
  }

  function updateStatsDisplay(): void {
    if (fpsBuffer.length > 0) {
      const avgMs = fpsBuffer.reduce((a, b) => a + b, 0) / fpsBuffer.length;
      const fps = avgMs > 0 ? 1000 / avgMs : 0;
      fpsDisplay = `${fps.toFixed(0)} fps`;
    }
    if (gpuBuffer.length > 0 && fpsBuffer.length > 0) {
      const avgRenderMs = gpuBuffer.reduce((a, b) => a + b, 0) / gpuBuffer.length;
      const avgFrameMs = fpsBuffer.reduce((a, b) => a + b, 0) / fpsBuffer.length;
      const pct = avgFrameMs > 0 ? (avgRenderMs / avgFrameMs) * 100 : 0;
      gpuDisplay = `${gpuLabel} ${avgRenderMs.toFixed(1)}ms · ${pct.toFixed(0)}%`;
    } else {
      gpuDisplay = `${gpuLabel} —`;
    }
  }

  function readColors(): Record<string, string> {
    if (typeof window === 'undefined') return {};
    const root = window.getComputedStyle(document.documentElement);
    const vars = [
      '--star-cool-white',
      '--star-pale-steel',
      '--star-pale-teal',
      '--star-warm-amber',
      '--star-pale-pink',
      '--star-shoot-head',
      '--star-shoot-trail',
      '--bg-void',
      '--nebula-teal',
      '--nebula-steel',
      '--nebula-violet',
      '--nebula-deep',
    ];
    const out: Record<string, string> = {};
    for (const v of vars) {
      const val = root.getPropertyValue(v).trim();
      out[v] = val || 'rgb(228,234,242)';
    }
    return out;
  }

  function buildOrbitForMode(mode: OrbitMode, fx: OrbitFixture): ThreeOrbitRefs {
    if (mode === 'map') return buildThreeMap(fx);
    if (mode === 'research') return buildThreeResearch(fx);
    return buildThreeOrbit(fx, DEFAULT_ORBIT_CONFIG);
  }

  function bgVoidHex(): number {
    const s = colorMap['--bg-void'] || '#0a0e12';
    if (s.startsWith('#')) return parseInt(s.slice(1), 16);
    const m = s.match(/rgb[a]?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/i);
    if (!m) return 0x0a0e12;
    return (parseInt(m[1], 10) << 16) | (parseInt(m[2], 10) << 8) | parseInt(m[3], 10);
  }

  function syncSlotsToGpu(): void {
    if (!field || !scheduler) return;
    // Build per-layer sets of slot IDs that are currently being mutated
    // by an in-flight explosion (baseAlpha/baseSize change every frame
    // during the rise/fall envelope). All other 'present' slots are
    // static between frames and don't need GPU re-upload.
    const explosionByLayer: Record<'far' | 'mid' | 'near', Set<number>> = {
      far: new Set(), mid: new Set(), near: new Set(),
    };
    for (const ex of scheduler.explosions) {
      explosionByLayer[ex.slotLayer].add(ex.slotId);
    }
    for (const layer of layers) {
      const slots = layer.layer === 'far' ? field.far
                  : layer.layer === 'mid' ? field.mid
                  : field.near;
      const explSet = explosionByLayer[layer.layer];
      for (let i = 0; i < slots.length; i++) {
        const s = slots[i];
        // Skip slots that haven't changed since last sync. A 'present'
        // slot with no active explosion has identical buffer values
        // frame-over-frame; uploading it again wastes driver time.
        // Three.js's WebGLAttributes only re-uploads when its version
        // counter advances — by not calling setX we leave the version
        // alone, so the entire buffer skips re-upload if no slot in it
        // changed. This is the largest single perf win on this scene.
        if (s.state === 'present' && !explSet.has(s.slotId)) continue;
        const fade = s.state === 'departed' ? 0
                   : s.state === 'fading-in' ? Math.max(0, Math.min(1, s.fadeIn))
                   : 1;
        layer.setFadeIn(s.slotId, fade);
        // Position only changes on regenSlot (state transitions to
        // 'fading-in'); the dynamic-only path covers that case.
        layer.setPosition(s.slotId, s.worldX, s.worldY, 0);
        layer.setBaseSize(s.slotId, s.baseSize);
        layer.setBaseAlpha(s.slotId, s.baseAlpha);
      }
    }
  }

  function setupShooterMesh(): void {
    if (!three || !scheduler) return;
    // Pre-allocate a Points mesh sized for the trail-length × maxConcurrent
    // upper bound. Each frame we update positions; unused slots get
    // alpha=0 via the fragment-side trail logic.
    const maxPoints = scheduler.config.maxConcurrent * (scheduler.config.trailLength + 1);
    const pos = new Float32Array(maxPoints * 3);
    const col = new Float32Array(maxPoints * 3);
    const sz = new Float32Array(maxPoints);
    const al = new Float32Array(maxPoints);
    shooterGeo = new THREE.BufferGeometry();
    shooterGeo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
    shooterGeo.setAttribute('color', new THREE.BufferAttribute(col, 3));
    shooterGeo.setAttribute('size', new THREE.BufferAttribute(sz, 1));
    shooterGeo.setAttribute('alpha', new THREE.BufferAttribute(al, 1));

    const vert = /* glsl */ `
      attribute float size;
      attribute float alpha;
      varying float vAlpha;
      varying vec3 vColor;
      void main() {
        vec4 mv = modelViewMatrix * vec4(position, 1.0);
        gl_Position = projectionMatrix * mv;
        gl_PointSize = size * (320.0 / max(1.0, -mv.z));
        vAlpha = alpha;
        vColor = color;
      }
    `;
    const frag = /* glsl */ `
      varying float vAlpha;
      varying vec3 vColor;
      void main() {
        vec2 c = gl_PointCoord - vec2(0.5);
        float r = length(c);
        if (r > 0.5) discard;
        float falloff = smoothstep(0.5, 0.0, r);
        gl_FragColor = vec4(vColor, vAlpha * falloff);
      }
    `;
    const mat = new THREE.ShaderMaterial({
      vertexShader: vert,
      fragmentShader: frag,
      vertexColors: true,
      transparent: true,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });

    shooterMesh = new THREE.Points(shooterGeo, mat);
    shooterMesh.frustumCulled = false;
    shooterMesh.geometry.setDrawRange(0, 0);
    three.scene.add(shooterMesh);
  }

  function syncShootersToGpu(): void {
    if (!shooterGeo || !scheduler || !three) return;
    const posAttr = shooterGeo.getAttribute('position') as THREE.BufferAttribute;
    const colAttr = shooterGeo.getAttribute('color') as THREE.BufferAttribute;
    const szAttr = shooterGeo.getAttribute('size') as THREE.BufferAttribute;
    const alAttr = shooterGeo.getAttribute('alpha') as THREE.BufferAttribute;

    let idx = 0;
    const headColor = parseRgb(colorMap['--star-shoot-head']);

    const leapHeadMul = scheduler.config.leapShooterHeadSizeMul;
    for (const sh of scheduler.shooters) {
      const headAlpha = shooterAlphaCurve(sh);
      const headScale = shooterScaleCurve(sh);
      const center = centerOf(sh.slotLayer);
      // Leap shooters render with a bigger head + thicker trail so they
      // visually arc through depth (the user wants some shooters to
      // read as 3D streaks, not flat points).
      const headPx  = (sh.isLeap ? 5.0 * leapHeadMul : 5.0) * headScale;
      const trailPx = (sh.isLeap ? 3.0 * leapHeadMul : 3.0) * headScale;
      writePoint(posAttr, colAttr, szAttr, alAttr, idx,
        sh.worldX - center.cx,
        -(sh.worldY - center.cy),
        sh.worldZ,
        headColor, headPx, headAlpha);
      idx++;
      // Trail points (tapered). Each trail entry carries its own z so
      // the trail extends through depth, not flat at the layer plane.
      for (let i = 0; i < sh.trail.length && idx < posAttr.count; i++, idx++) {
        const tFrac = 1 - i / sh.trailMax;
        writePoint(posAttr, colAttr, szAttr, alAttr, idx,
          sh.trail[i].worldX - center.cx,
          -(sh.trail[i].worldY - center.cy),
          sh.trail[i].worldZ,
          headColor, trailPx * tFrac, headAlpha * tFrac * 0.85);
      }
    }
    posAttr.needsUpdate = true;
    colAttr.needsUpdate = true;
    szAttr.needsUpdate = true;
    alAttr.needsUpdate = true;
    shooterGeo.setDrawRange(0, idx);
  }

  function writePoint(p: THREE.BufferAttribute, c: THREE.BufferAttribute, s: THREE.BufferAttribute, a: THREE.BufferAttribute,
                      i: number, x: number, y: number, z: number, color: [number, number, number], size: number, alpha: number): void {
    p.array[i * 3]     = x;
    p.array[i * 3 + 1] = y;
    p.array[i * 3 + 2] = z;
    c.array[i * 3]     = color[0];
    c.array[i * 3 + 1] = color[1];
    c.array[i * 3 + 2] = color[2];
    s.array[i] = size;
    a.array[i] = alpha;
  }

  function parseRgb(s: string | undefined): [number, number, number] {
    if (!s) return [0.9, 0.92, 0.95];
    const m = s.match(/rgb[a]?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/i);
    if (!m) return [0.9, 0.92, 0.95];
    return [parseInt(m[1], 10) / 255, parseInt(m[2], 10) / 255, parseInt(m[3], 10) / 255];
  }

  // Shooter alpha/scale curves duplicated here to avoid importing the
  // schedule module's pure-state functions only for one math op each.
  function shooterAlphaCurve(sh: Shooter): number {
    const t = sh.elapsed / sh.lifetime;
    if (t < 0.5) return 1;
    return Math.max(0, 1 - (t - 0.5) / 0.5);
  }
  function shooterScaleCurve(sh: Shooter): number {
    const t = sh.elapsed / sh.lifetime;
    if (t < 0.5) return 1;
    return Math.max(0.05, 1 - (t - 0.5) / 0.5 * 0.95);
  }

  function centerOf(layer: 'far' | 'mid' | 'near'): { cx: number; cy: number } {
    const layerObj = layers.find((l) => l.layer === layer);
    if (!layerObj || !field) return { cx: 0, cy: 0 };
    const slots = layer === 'far' ? field.far : layer === 'mid' ? field.mid : field.near;
    let mn = Infinity, mx = -Infinity, mny = Infinity, mxy = -Infinity;
    for (const s of slots) {
      if (s.worldX < mn) mn = s.worldX;
      if (s.worldX > mx) mx = s.worldX;
      if (s.worldY < mny) mny = s.worldY;
      if (s.worldY > mxy) mxy = s.worldY;
    }
    return { cx: (mn + mx) / 2, cy: (mny + mxy) / 2 };
  }

  function tick(ts: number): void {
    if (!three || !field || !scheduler) {
      rafId = null;
      return;
    }
    // Optional frame-rate cap (?cap=60 etc.) — skip the entire tick body
    // if not enough time has passed since the last rendered frame. The
    // rAF callback is still re-armed below so we keep getting ticks at
    // the monitor refresh; we just don't *render* on every one.
    if (FRAME_CAP_HZ > 0 && lastRenderTs > 0) {
      const targetMs = 1000 / FRAME_CAP_HZ;
      if (ts - lastRenderTs < targetMs - 0.5) {
        rafId = requestAnimationFrame(tick);
        return;
      }
    }
    lastRenderTs = ts;
    if (startMs === 0) startMs = ts;
    const elapsedMs = ts - startMs;
    const dt = lastMs === 0 ? 0 : Math.min(0.1, (ts - lastMs) / 1000);
    lastMs = ts;

    const tStart = PERF_DEBUG ? performance.now() : 0;

    // Advance scheduler (regenerates slots, advances trails, runs
    // explosions). Pass the rAF timestamp so explosion envelopes share
    // a clock with the shader's uTimeMs.
    tickScheduler(scheduler, field, dt, reducedMotion, ts);

    const tAfterTick = PERF_DEBUG ? performance.now() : 0;

    // Push slot lifecycle changes to GPU.
    syncSlotsToGpu();

    const tAfterSlots = PERF_DEBUG ? performance.now() : 0;

    // Compute the per-frame perspective boost. Two contributors:
    //   • warp progress — parabolic bell over the warp duration that
    //     transiently amps far stars while we leap.
    //   • smoothZoomDepth — eased lerp toward ZOOM_DEPTH[zoomLevel],
    //     persists after warp so each zoom level holds a different
    //     depth perspective.
    let warpProgress = 0;
    if (warpStartMs >= 0) {
      const wT = Math.min(1, (ts - warpStartMs) / WARP_DURATION_MS);
      const x = (wT - 0.5) * 2;
      warpProgress = Math.max(0, 1 - x * x);
    }
    const zoomFactor = 1 - Math.exp(-ZOOM_RATE * dt);
    smoothZoomDepth += (ZOOM_DEPTH[zoomLevel] - smoothZoomDepth) * zoomFactor;
    const perspectiveBoost = Math.max(warpProgress, smoothZoomDepth);

    // Update star shader uniforms.
    for (const layer of layers) {
      const u = layer.material.uniforms;
      u.uTimeMs.value = elapsedMs;
      u.uReducedMotion.value = reducedMotion ? 1.0 : 0.0;
      u.uPerspectiveBoost.value = perspectiveBoost;
    }

    // Sync shooters.
    syncShootersToGpu();

    const tAfterShoot = PERF_DEBUG ? performance.now() : 0;

    // Chase-camera with warp. On zoomLevel change the $effect captures
    // (current camPos / focus) into _warpFrom* and the new resolved
    // target into _warpTo*. While warping, we ease over WARP_DURATION_MS
    // for the long-distance leap feel. After arrival, fall through to
    // the slow lerp(0.02) residual settle so any later target nudges
    // (e.g. fixture position recompute) don't snap.
    resolveCameraTargets(_focusTarget, _camPosTarget);
    if (warpStartMs >= 0) {
      const t = Math.min(1, (ts - warpStartMs) / WARP_DURATION_MS);
      const e = easeInOutQuint(t);
      smoothFocus.lerpVectors(_warpFromFocus, _warpToFocus, e);
      smoothCamPos.lerpVectors(_warpFromCam, _warpToCam, e);
      if (t >= 1) {
        // Warp done. Lock to the resolved (live) target so any
        // residual nudge feeds the slow lerp from here.
        smoothFocus.copy(_focusTarget);
        smoothCamPos.copy(_camPosTarget);
        warpStartMs = -1;
        emitWarpState(false);
        // Trigger the mandala fade-in NOW that the camera has settled.
        // syncMandala held it at opacity 0 while warpStartMs was active;
        // once the warp completes the user sees the UI elegantly fade
        // in over FADE_IN_MS rather than appearing mid-flight.
        if (nodeMandala && panelTrackNodeId) {
          nodeMandala.setVisible(true, ts);
        }
      }
    } else {
      // Time-based settle — `factor = 1 - exp(-rate * dt)` keeps the
      // chase-cam moving at the same world-units-per-second regardless
      // of the user's monitor refresh rate. Old `lerp(0.02)` per frame
      // was 2× faster on a 120 Hz monitor than on a 60 Hz one.
      const camFactor = 1 - Math.exp(-CAM_RATE * dt);
      smoothFocus.lerp(_focusTarget, camFactor);
      smoothCamPos.lerp(_camPosTarget, camFactor);
      if (smoothFocus.distanceToSquared(_focusTarget) < 1) smoothFocus.copy(_focusTarget);
      if (smoothCamPos.distanceToSquared(_camPosTarget) < 1) smoothCamPos.copy(_camPosTarget);
    }

    // Smooth mouse parallax so hardware jitter / fast cursor moves don't
    // translate 1:1 into camera position pops. Time-based factor —
    // `1 - exp(-rate * dt)` matches the legacy 0.10/frame-at-60fps
    // behaviour but stays consistent at 30 / 75 / 120 / 144 / 240 Hz.
    // Without this, high-refresh users see the camera react too quickly
    // and low-refresh users see it lag.
    const mouseFactor = 1 - Math.exp(-MOUSE_RATE * dt);
    smoothMouseX += (mouseX - smoothMouseX) * mouseFactor;
    smoothMouseY += (mouseY - smoothMouseY) * mouseFactor;
    three.camera.position.set(
      smoothCamPos.x + smoothMouseX * MOUSE_CAMERA_GAIN,
      smoothCamPos.y - smoothMouseY * MOUSE_CAMERA_GAIN,
      smoothCamPos.z,
    );
    three.camera.lookAt(smoothFocus);

    // Anchor the sky dome to the camera position so it reads as
    // fixed at infinity — translation never parallaxes it; only
    // camera rotation reveals different parts of the dome. Cheap:
    // one Vec3 copy per frame. Also push the camera→sun direction
    // so the dome shader can dim its tint in the cone around the
    // sun and prevent the sun's bloom from merging with the
    // atmosphere.
    if (skydome) {
      skydome.mesh.position.copy(three.camera.position);
      three.camera.getWorldDirection(_sunDir);
      skydome.setSunDir(_sunDir);
      skydome.setFrame(elapsedMs, reducedMotion);
    }

    // Breath animation on the active baby's halo (replicates the SVG
    // halo-pulse). Period 2.6s; scale 1.0 → 1.18; opacity 0.85 → 1.0.
    // Both inner and outer halos breathe together for the mini-sun look.
    if (orbit && orbit.nodes.length > 0 && orbitFixture) {
      const abId = orbitFixture.active_baby_step_id;
      const activeBaby = orbit.nodes.find((n) => n.type === 'baby' && n.id === abId);
      if (activeBaby) {
        const phase = (elapsedMs / 2600) * Math.PI * 2;
        const scaleMod = 1.0 + Math.sin(phase) * 0.09;
        const opacityMod = 0.85 + (Math.sin(phase) * 0.5 + 0.5) * 0.15;
        const sIn = activeBaby.haloBaseScale * scaleMod;
        activeBaby.halo.scale.set(sIn, sIn, 1);
        activeBaby.haloMat.opacity = activeBaby.haloBaseOpacity * opacityMod;
        const sOut = activeBaby.outerHaloBaseScale * scaleMod;
        activeBaby.outerHalo.scale.set(sOut, sOut, 1);
        activeBaby.outerHaloMat.opacity = activeBaby.outerHaloBaseOpacity * opacityMod;
      }
      // Concept-halo gentle breathing (slower than the active baby).
      // Base scales come from three-orbit.ts so the two sites can't
      // drift; last session a hardcoded mismatch caused first-frame pop.
      const cPhase = (elapsedMs / 7000) * Math.PI * 2;
      const cMod = 1.0 + Math.sin(cPhase) * 0.04;
      const oS = CONCEPT_HALO_OUTER_SCALE * cMod;
      const iS = CONCEPT_HALO_INNER_SCALE * cMod;
      orbit.conceptHalos[0]?.scale.set(oS, oS, 1);
      orbit.conceptHalos[1]?.scale.set(iS, iS, 1);
    }

    // Drive the active-branch shader's animated dash.
    if (orbit) orbit.updateAnimation(elapsedMs);

    // Planet-scan: fade in at deepest zoom on the active focal node;
    // fade out on any other zoom level / mode swap. The PlanetScan
    // class is allocated once in the scene init; we just toggle it.
    updatePlanetScan(ts);

    // V3 NodeMandala: tick + project compass anchors. Hidden during
    // warps via parent's onWarpStateChange handshake (DOM layers fade
    // out independently).
    if (containerEl) {
      const r = containerEl.getBoundingClientRect();
      tickMandala(dt, ts, r.width, r.height);
    } else {
      tickMandala(dt, ts, 0, 0);
    }

    // Project node world positions to screen for the HTML label overlay.
    updateLabelProjections();

    // Push the panel-anchor projection (NodePanel DOM overlay) — only
    // when the parent has bound a panelTrackNodeId. Cheap: 1 Vector3
    // project + 1 callback per frame.
    emitPanelTrack();

    const tAfterCamHalo = PERF_DEBUG ? performance.now() : 0;

    // FPS sample (rAF delta since last frame).
    if (lastFrameStamp > 0) {
      const dtMs = ts - lastFrameStamp;
      fpsBuffer.push(dtMs);
      if (fpsBuffer.length > FPS_BUFFER_LEN) fpsBuffer.shift();
    }
    lastFrameStamp = ts;

    // GPU timer-query bracket — only every 15th frame. The query API
    // (`gl.getQueryParameter`, `gl.getParameter(GPU_DISJOINT_EXT)`) is
    // synchronous: each call forces the driver to flush the WebGL
    // command buffer and wait. On Firefox + Windows + ANGLE that's
    // genuinely expensive (often 1–3 ms per call). Polling every frame
    // wastes most of that cost since the HUD only updates every 30
    // frames anyway. Sampling every 15 frames is plenty for the readout
    // and removes ~20 ms/sec of synchronous-flush overhead.
    const gl = three.renderer.getContext() as WebGL2RenderingContext;
    const pollGpu = (statsFrameCount % 15) === 0;
    if (pollGpu) gpuStatsBegin(gl);
    three.render();
    if (pollGpu) gpuStatsEnd(gl);

    if (PERF_DEBUG) {
      const tEnd = performance.now();
      const dTick   = tAfterTick   - tStart;
      const dSlots  = tAfterSlots  - tAfterTick;
      const dShoot  = tAfterShoot  - tAfterSlots;
      const dCamHalo = tAfterCamHalo - tAfterShoot;
      const dRender = tEnd - tAfterCamHalo;
      const dTotal  = tEnd - tStart;
      perfPush(perfTick, dTick);
      perfPush(perfSyncSlots, dSlots);
      perfPush(perfSyncShoot, dShoot);
      perfPush(perfCamHalo, dCamHalo);
      perfPush(perfRender, dRender);
      perfPush(perfFrameTotal, dTotal);
      // Spike detection: any frame whose total CPU time exceeds 1.8×
      // the rolling median (and is at least 8 ms — cheap frames don't
      // need flagging) gets a console.warn with the breakdown so the
      // bad frames can be inspected directly.
      if (perfFrameTotal.length >= 30) {
        const med = perfMedian(perfFrameTotal);
        if (dTotal > med * 1.8 && dTotal > 8) {
          spikeCount++;
          console.warn(
            `[elai-perf] spike frame=${dTotal.toFixed(2)}ms median=${med.toFixed(2)}ms ` +
            `tick=${dTick.toFixed(2)} slots=${dSlots.toFixed(2)} shoot=${dShoot.toFixed(2)} ` +
            `camHalo=${dCamHalo.toFixed(2)} render=${dRender.toFixed(2)}`,
          );
        }
      }
    }

    // Refresh the readouts every ~half-second to keep the HUD stable.
    statsFrameCount++;
    if (statsFrameCount % 30 === 0) {
      updateStatsDisplay();
      updatePerfDisplay();
    }
    rafId = requestAnimationFrame(tick);
  }

  function resize(): void {
    if (!three || !containerEl) return;
    const rect = containerEl.getBoundingClientRect();
    three.resize(rect.width, rect.height, window.devicePixelRatio || 1);
  }

  onMount(() => {
    if (typeof window === 'undefined' || !containerEl) return;
    const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
    reducedMotion = mq.matches;
    const onMq = (e: MediaQueryListEvent) => { reducedMotion = e.matches; };
    mq.addEventListener('change', onMq);

    colorMap = readColors();

    three = createThreeScene(containerEl, {
      clearColor: bgVoidHex(),
      // Bloom only on HDR-bright objects (sun + active baby). Inactive
      // nodes stay below threshold so they read as crisp dots, not
      // glowing balls.
      // Bloom turned down — sun reads as glow, not floodlight. Smaller
      // radius + strength is also cheaper on the GPU (bloom is the
      // dominant per-frame cost).
      bloomThreshold: 0.85,
      // ?nobloom=1 disables bloom entirely (sets strength=0). Useful as
      // a perf A/B test — bloom is the dominant GPU cost on this scene
      // by a wide margin. If the page feels equally janky with bloom
      // off, the bottleneck isn't the GPU shader at all.
      bloomStrength: DISABLE_BLOOM ? 0.0 : 0.20,
      bloomRadius: 0.10,
      // Camera default distance scales with the ~2× orbit. Pitch 12°
      // keeps it eye-level with the disc. lookAt y > camera y so the
      // camera looks slightly UPWARD into the constellation.
      cameraDistance: 1500,
      cameraPitchDegrees: 12,
      lookAtX: 0,
      lookAtY: 180,
      lookAtZ: -460,
    });

    // Procedural sky dome — BackSide sphere with FBM nebula clouds +
    // dense cell-hashed bg stars. Sits at "infinity" (followed by the
    // camera each frame so position changes don't parallax it). Rendered
    // first via renderOrder = -1000, output clamped < 0.78 so the
    // dome's haze never crosses the bloom threshold (0.85). On layer 0
    // alongside stars and sun.
    skydome = buildSkyDome(colorMap, { radius: 14000, seed });
    three.scene.add(skydome.mesh);

    field = buildStarField(seed, tone, density);
    layers = buildAllStarLayers(field, colorMap, three.renderer.getPixelRatio());
    for (const layer of layers) three.scene.add(layer.points);

    scheduler = makeScheduler({
      regenSeed: seed,
      eventSeed: (Math.floor(performance.now()) ^ seed) >>> 0,
      mode,
      tone,
    });
    setupShooterMesh();
    gpuStatsSetup(three.renderer.getContext() as WebGL2RenderingContext);

    if (showOrbit) {
      orbit = buildOrbitForMode(
        orbitMode,
        orbitFixture ?? { task_id: 'placeholder', idea: '', fsm: 'IDLE', concepts: [] },
      );
      three.scene.add(orbit.group);
    }

    // Planet-scan layer (only meaningful for plan mode at deepest zoom).
    // Always mounted so fade-in/out can run smoothly; visibility gated
    // by zoomLevel === 'baby-step' inside the rAF tick loop.
    planetScan = new PlanetScan();
    three.scene.add(planetScan.group);

    // Wake up any reactive consumers that early-returned during their
    // first run because `three`/`orbit` were still null. Bumps a tracked
    // $state so the syncMandala effect re-runs now that the scene exists.
    _sceneReadyTick++;

    const ro = new ResizeObserver(() => resize());
    ro.observe(containerEl);
    resize();

    startMs = 0;
    lastMs = 0;
    rafId = requestAnimationFrame(tick);

    return () => {
      mq.removeEventListener('change', onMq);
      ro.disconnect();
      if (rafId !== null) cancelAnimationFrame(rafId);
      rafId = null;
      disposeMandalaIfAny();
      if (planetScan) {
        planetScan.dispose();
        planetScan = null;
      }
      if (three) {
        gpuStatsDispose(three.renderer.getContext() as WebGL2RenderingContext);
        three.dispose();
      }
      three = null;
      skydome = null;
    };
  });

  onDestroy(() => {
    if (rafId !== null) cancelAnimationFrame(rafId);
    rafId = null;
    disposeMandalaIfAny();
    if (planetScan) {
      planetScan.dispose();
      planetScan = null;
    }
    if (three) {
      gpuStatsDispose(three.renderer.getContext() as WebGL2RenderingContext);
      three.dispose();
    }
    three = null;
    skydome = null;
  });

  $effect(() => {
    if (!scheduler) return;
    if (scheduler.mode !== mode) scheduler.mode = mode;
  });

  // React to active-state changes (active phase / step / baby) without
  // rebuilding geometries.
  $effect(() => {
    if (!orbit || !orbitFixture) return;
    orbit.refreshActive(orbitFixture);
  });

  // React to zoomLevel changes — kick off an eased warp from the
  // camera's current pose to the resolved target for the new level.
  // Skip the very first run (initial mount has no "from"). If a warp is
  // already in flight, restart from the current camera position so the
  // user can chain zooms without snapping.
  $effect(() => {
    const level = zoomLevel;
    if (prevZoomLevel === null) {
      prevZoomLevel = level;
      return;
    }
    if (level === prevZoomLevel) return;
    prevZoomLevel = level;
    if (!three || !orbit) return;
    _warpFromCam.copy(smoothCamPos);
    _warpFromFocus.copy(smoothFocus);
    resolveCameraTargets(_warpToFocus, _warpToCam);
    warpStartMs = performance.now();
    emitWarpState(true);
  });

  // Rebuild orbit geometry when the mode changes. Plan/map/research
  // produce wholly different scenes — disposal of the old orbit is
  // mandatory or we leak GPU buffers per swap. The fixture is also
  // monitored so a fixture swap (different pagerank_nodes etc.) takes
  // effect for map/research; plan mode handles fixture changes via
  // refreshActive without rebuilding.
  //
  // CRITICAL: when the mode (or non-plan fixture) actually changes, the
  // chase-camera state (smoothCamPos / smoothFocus / prevZoomLevel) MUST
  // be re-synchronised with the new orbit. Otherwise:
  //   • smoothCamPos retains the OLD mode's focal-node pose, so on the
  //     next frame the camera looks at the new orbit from "beyond" or
  //     "below" the new focal node. The active-line shader (sun → phase
  //     → step → baby) then projects mirrored — visually "going
  //     BACKWARDS off camera" (user-reported bug at deepest zoom).
  //   • the zoomLevel $effect only fires on level CHANGES, so a mode
  //     swap that keeps zoomLevel constant (common: user zooms to baby,
  //     swaps mode, swaps back) never re-resolves camera targets.
  // Force a fresh warp from the current camera into the new orbit's
  // resolved targets so the camera always re-anchors to the current
  // focal node after every rebuild. This is also how plan-mode handles
  // a fixture change with no zoomLevel change (live update of active_*).
  let prevOrbitMode: OrbitMode | null = null;
  let prevFixtureRef: OrbitFixture | null = null;
  $effect(() => {
    if (!three) return;
    const mode = orbitMode;
    const fx = orbitFixture;
    const fixtureChangedForNonPlan =
      mode !== 'plan' && fx !== prevFixtureRef;
    const modeChanged = mode !== prevOrbitMode;
    if (!modeChanged && !fixtureChangedForNonPlan) return;
    prevOrbitMode = mode;
    prevFixtureRef = fx ?? null;
    // Mode swap means the bound focal node mesh might disappear; dispose
    // the V3 mandala unconditionally so we don't render stale rings
    // around a freed node ref.
    disposeMandalaIfAny();
    if (orbit) {
      three.scene.remove(orbit.group);
      orbit.dispose();
      orbit = null;
    }
    if (showOrbit) {
      orbit = buildOrbitForMode(
        mode,
        fx ?? { task_id: 'placeholder', idea: '', fsm: 'IDLE', concepts: [] },
      );
      three.scene.add(orbit.group);
      // Re-anchor camera: kick a warp from the CURRENT smooth pose into
      // the resolved target for the NEW orbit at the current zoomLevel.
      // Without this, smoothCamPos/Focus retain the prior mode's focal
      // anchor and the active-line projects mirrored at deep zooms.
      _warpFromCam.copy(smoothCamPos);
      _warpFromFocus.copy(smoothFocus);
      resolveCameraTargets(_warpToFocus, _warpToCam);
      warpStartMs = performance.now();
      emitWarpState(true);
      // Reset prevZoomLevel so a subsequent zoom-level change still
      // captures the (now post-warp) camera as its "from" pose. Without
      // this reset the very first zoom after a mode-swap could ease
      // from a position the user can't see (mid-warp camera).
      prevZoomLevel = zoomLevel;
    }
  });

  // -----------------------------------------------------------------
  // Click-to-fly camera + hover events. THREE.Raycaster on the canvas;
  // if a hit lands on a node mesh, fire onNodeClick or onNodeHover. The
  // parent stores flyToNodeId / panelTrackNodeId; we resolve to those
  // nodes elsewhere. Coordinates are in canvas-local pixel space.
  //
  // Hover: throttled to ~30 Hz (every 33 ms). Anything faster wastes
  // raycasts on cursor jitter; the panel only re-projects each frame
  // (~16 ms at 60 fps) so 30 Hz hover detection is plenty.
  // -----------------------------------------------------------------
  const _ray = new THREE.Raycaster();
  const _ndc = new THREE.Vector2();
  let _lastHoverId: string | null = null;
  let _lastHoverMs = 0;
  const HOVER_THROTTLE_MS = 33;
  function onCanvasClick(ev: MouseEvent): void {
    if (!three || !orbit || !containerEl) return;
    const rect = containerEl.getBoundingClientRect();
    _ndc.x = ((ev.clientX - rect.left) / rect.width) * 2 - 1;
    _ndc.y = -(((ev.clientY - rect.top) / rect.height) * 2 - 1);
    _ray.setFromCamera(_ndc, three.camera);
    // Build a fresh meshes-array from orbit.nodes — fast enough for the
    // small node counts we have (~50-200 per scene).
    const meshes = orbit.nodes.map((n) => n.mesh);
    if (meshes.length === 0) return;
    const hits = _ray.intersectObjects(meshes, false);
    if (hits.length === 0) return;
    const hitMesh = hits[0].object as THREE.Mesh;
    const ref = orbit.nodes.find((n) => n.mesh === hitMesh);
    if (!ref) return;
    onNodeClick?.(ref.id, ref.worldX, ref.worldY, ref.worldZ);
  }
  function onCanvasMouseMove(ev: MouseEvent): void {
    if (!onNodeHover) return;
    if (!three || !orbit || !containerEl) return;
    const now = ev.timeStamp || performance.now();
    if (now - _lastHoverMs < HOVER_THROTTLE_MS) return;
    _lastHoverMs = now;
    const rect = containerEl.getBoundingClientRect();
    _ndc.x = ((ev.clientX - rect.left) / rect.width) * 2 - 1;
    _ndc.y = -(((ev.clientY - rect.top) / rect.height) * 2 - 1);
    _ray.setFromCamera(_ndc, three.camera);
    const meshes = orbit.nodes.map((n) => n.mesh);
    if (meshes.length === 0) {
      if (_lastHoverId !== null) { _lastHoverId = null; onNodeHover?.(null, null); }
      return;
    }
    const hits = _ray.intersectObjects(meshes, false);
    if (hits.length === 0) {
      if (_lastHoverId !== null) { _lastHoverId = null; onNodeHover?.(null, null); }
      return;
    }
    const hitMesh = hits[0].object as THREE.Mesh;
    const ref = orbit.nodes.find((n) => n.mesh === hitMesh);
    if (!ref) return;
    if (ref.id !== _lastHoverId) {
      _lastHoverId = ref.id;
      onNodeHover?.(ref.id, ref.type);
    }
  }
  function onCanvasMouseLeave(): void {
    if (_lastHoverId !== null) { _lastHoverId = null; onNodeHover?.(null, null); }
  }

  // ----- Per-frame projection for the NodePanel anchor. Driven from
  // tick() below; called only when panelTrackNodeId is set. The result
  // is dispatched via onPanelTrack(x, y, visible) so OrbitCanvas can
  // position its NodePanel without re-implementing projection here. ---
  const _panelProj = new THREE.Vector3();
  function emitPanelTrack(): void {
    if (!onPanelTrack || !three || !containerEl) return;
    const id = panelTrackNodeId;
    if (!id) return;
    let wx = 0, wy = 0, wz = 0;
    if (id === '__sun__') {
      // Sun is always at world origin; no node lookup required.
      wx = 0; wy = 0; wz = 0;
    } else {
      if (!orbit) return;
      const ref = orbit.nodes.find((n) => n.id === id);
      if (!ref) {
        onPanelTrack(0, 0, false);
        return;
      }
      wx = ref.worldX; wy = ref.worldY; wz = ref.worldZ;
    }
    const rect = containerEl.getBoundingClientRect();
    _panelProj.set(wx, wy, wz);
    _panelProj.project(three.camera);
    const visible =
      _panelProj.z > -1 && _panelProj.z < 1
      && _panelProj.x > -1.05 && _panelProj.x < 1.05
      && _panelProj.y > -1.05 && _panelProj.y < 1.05;
    const sx = (_panelProj.x * 0.5 + 0.5) * rect.width;
    const sy = (1 - (_panelProj.y * 0.5 + 0.5)) * rect.height;
    onPanelTrack(sx, sy, visible);
  }

  // Override resolveCameraTargets when flyToNodeId is set. We do this
  // by reading from a small reactive snapshot inside tick() rather than
  // patching the function — keeps the chase logic single-source.
  let _flyToWorld: THREE.Vector3 | null = null;
  $effect(() => {
    if (!flyToNodeId || !orbit) {
      _flyToWorld = null;
      return;
    }
    const ref = orbit.nodes.find((n) => n.id === flyToNodeId);
    if (!ref) {
      _flyToWorld = null;
      return;
    }
    _flyToWorld = new THREE.Vector3(ref.worldX, ref.worldY, ref.worldZ);
    if (!three) return;
    _warpFromCam.copy(smoothCamPos);
    _warpFromFocus.copy(smoothFocus);
    // Camera offset same as plan-mode subject framing.
    _warpToFocus.set(_flyToWorld.x, _flyToWorld.y, _flyToWorld.z + SUBJECT_FOCUS_DZ);
    _warpToCam.set(
      _flyToWorld.x,
      _flyToWorld.y + SUBJECT_CAM_DY,
      _flyToWorld.z + SUBJECT_CAM_DZ,
    );
    warpStartMs = performance.now();
    emitWarpState(true);
  });

  // -----------------------------------------------------------------
  // Product-done super-sun. When productDone flips true, scale the sun
  // by 4× over 4 seconds and ramp bloom strength up. Other nodes fade
  // to 30% opacity. Triggered once per transition.
  // -----------------------------------------------------------------
  let _superSunStartMs = $state(-1);
  let prevProductDone = false;
  $effect(() => {
    if (productDone && !prevProductDone) {
      _superSunStartMs = performance.now();
    }
    prevProductDone = productDone;
  });

  function applySuperSun(ts: number): void {
    if (_superSunStartMs < 0 || !orbit || !three) return;
    const dur = 4000;
    const t = Math.min(1, (ts - _superSunStartMs) / dur);
    const scale = 1 + 3 * t;            // 1 → 4
    orbit.sun.scale.set(scale, scale, scale);
    // Ramp bloom strength on the composer's first bloom pass. Stops at
    // 0.6 (default 0.20). Disabled if URL bloom override active.
    if (!DISABLE_BLOOM) {
      three.bloom.strength = 0.20 + (0.40 * t);
    }
    // Fade non-sun meshes to 30% opacity. We mutate transparent flag +
    // opacity in-place; nodes restore on COMPLETE → IDLE elsewhere.
    if (t === 1) {
      // Sustain: keep super-sun until the fixture transitions away.
    }
    for (const n of orbit.nodes) {
      const mat = n.mesh.material as THREE.MeshBasicMaterial;
      if (!mat) continue;
      mat.transparent = true;
      mat.opacity = 1.0 - 0.7 * t;
      mat.needsUpdate = true;
    }
  }

  // Called from tick(); updates super-sun + draw-call HUD.
  function postTick(ts: number): void {
    applySuperSun(ts);
  }
  // Re-arm tick() to call postTick before render. Done by overriding
  // the original tick via a small wrapper isn't straightforward in Svelte
  // because tick is module-local; instead we splice into the existing
  // updatePerfDisplay path: pass a callback that the existing rAF loop
  // calls on each frame. Simpler: use a separate rAF inside an effect
  // that observes both productDone state and ticks the super-sun.
  let _superSunRaf: number | null = null;
  $effect(() => {
    if (_superSunStartMs < 0) return;
    const loop = () => {
      const ts = performance.now();
      postTick(ts);
      if (ts - _superSunStartMs < 4500) {
        _superSunRaf = requestAnimationFrame(loop);
      }
    };
    if (_superSunRaf !== null) cancelAnimationFrame(_superSunRaf);
    _superSunRaf = requestAnimationFrame(loop);
    return () => {
      if (_superSunRaf !== null) cancelAnimationFrame(_superSunRaf);
      _superSunRaf = null;
    };
  });

  // -----------------------------------------------------------------
  // Draw-call / triangle / GPU-mem HUD (G6). Toggled via showPerfOverlay
  // prop (parent watches `~` key when location.hash includes 'dev').
  // Reads renderer.info each frame; cheap.
  // -----------------------------------------------------------------
  let drawCallsDisplay = $state('0 calls');
  let trianglesDisplay = $state('0 tris');
  let memDisplay = $state('— mem');
  $effect(() => {
    if (!showPerfOverlay) return;
    let id: number | null = null;
    const tick = () => {
      if (three) {
        const info = three.renderer.info;
        drawCallsDisplay = `${info.render.calls} calls`;
        trianglesDisplay = `${info.render.triangles.toLocaleString()} tris`;
        // memory.geometries + memory.textures (approx)
        memDisplay = `${info.memory.geometries}g · ${info.memory.textures}t`;
      }
      id = requestAnimationFrame(tick);
    };
    id = requestAnimationFrame(tick);
    return () => { if (id !== null) cancelAnimationFrame(id); };
  });
</script>

<div
  class="star-canvas-root"
  class:starfield--centered={centered}
  class:click-enabled={onNodeClick !== undefined || onNodeHover !== undefined}
  bind:this={containerEl}
  onclick={onCanvasClick}
  onmousemove={onCanvasMouseMove}
  onmouseleave={onCanvasMouseLeave}
  role="presentation"
  aria-hidden="true"
>
  {#if centered}
    <div class="center-mask"></div>
  {/if}
  <div class="stats-hud" aria-hidden="true">
    <div class="stats-line">{fpsDisplay}</div>
    <div class="stats-line stats-line--gpu">{gpuDisplay}</div>
    {#if perfDisplay}
      <pre class="stats-line stats-line--perf">{perfDisplay}</pre>
    {/if}
  </div>
  {#if showPerfOverlay}
    <div class="perf-overlay" aria-hidden="true">
      <div class="po-line">{fpsDisplay}</div>
      <div class="po-line">{drawCallsDisplay}</div>
      <div class="po-line">{trianglesDisplay}</div>
      <div class="po-line">{memDisplay}</div>
    </div>
  {/if}
  {#if showLabels}
    <div class="orbit-labels">
      {#each labels as l (l.id)}
        {#if l.visible}
          <div
            class="orbit-label"
            class:orbit-label--active={l.isActive}
            class:orbit-label--phase={l.type === 'phase'}
            class:orbit-label--step={l.type === 'step'}
            class:orbit-label--baby={l.type === 'baby'}
            style="transform: translate3d({l.x}px, {l.y + 14}px, 0)"
          >{l.text}</div>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .star-canvas-root {
    position: absolute;
    inset: 0;
    pointer-events: none;
    overflow: hidden;
    z-index: 0;
  }
  .star-canvas-root.click-enabled {
    pointer-events: auto;
  }

  .perf-overlay {
    position: absolute;
    top: 8px;
    left: 8px;
    z-index: 6;
    display: flex;
    flex-direction: column;
    gap: 2px;
    pointer-events: none;
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 10px;
    line-height: var(--leading-code);
    letter-spacing: 0.04em;
    color: var(--teal-bright, rgba(75, 220, 180, 1));
    background: rgba(7, 9, 11, 0.78);
    border: 1px solid var(--border-subtle, rgba(120, 140, 160, 0.18));
    padding: 6px 8px;
    border-radius: 3px;
  }
  .po-line { white-space: nowrap; }

  .orbit-labels {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 3;
    overflow: hidden;
  }
  .orbit-label {
    position: absolute;
    top: 0;
    left: 0;
    transform-origin: 0 0;
    font: 10px/1.2 var(--font-mono, ui-monospace, monospace);
    color: var(--text-ghost, rgba(150, 165, 170, 0.65));
    letter-spacing: 0.04em;
    white-space: nowrap;
    user-select: none;
    /* Anchor centred-below the projected node — translate-x by -50%
       achieves "centred" without per-frame width measurement. */
    margin-left: -50px;
    width: 100px;
    text-align: center;
    text-shadow: 0 0 6px rgba(7, 9, 11, 0.92), 0 0 2px rgba(7, 9, 11, 1);
    transition: opacity 200ms ease-out, color 200ms ease-out;
  }
  .orbit-label--phase { color: var(--text-secondary, rgba(180, 195, 200, 0.85)); font-size: 10px; }
  .orbit-label--step  { color: var(--text-ghost, rgba(150, 165, 170, 0.70)); font-size: 10px; }
  .orbit-label--baby  { color: var(--text-ghost, rgba(150, 165, 170, 0.55)); font-size: 10px; }
  .orbit-label--active {
    color: var(--teal-bright, rgba(75, 220, 180, 1));
    font-size: 10px;
    letter-spacing: 0.06em;
  }

  .stats-hud {
    position: absolute;
    top: 8px;
    left: 8px;
    z-index: 5;
    display: flex;
    flex-direction: column;
    gap: 2px;
    pointer-events: none;
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 10px;
    line-height: var(--leading-code);
    letter-spacing: 0.04em;
    color: var(--text-ghost, rgba(150, 165, 170, 0.85));
    text-shadow: 0 0 4px rgba(7, 9, 11, 0.9), 0 0 1px rgba(7, 9, 11, 1);
  }
  .stats-line--gpu { opacity: 0.7; }
  .stats-line--perf {
    margin: 4px 0 0;
    padding: 4px 6px;
    background: rgba(7, 9, 11, 0.78);
    border: 1px solid rgba(120, 140, 160, 0.18);
    border-radius: 3px;
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 9.5px;
    line-height: var(--leading-code);
    letter-spacing: 0.02em;
    white-space: pre;
    color: var(--text-secondary, rgba(180, 195, 200, 0.85));
    text-shadow: none;
  }

  .center-mask {
    position: absolute;
    inset: 0;
    background: radial-gradient(
      ellipse 78% 72% at 50% 50%,
      rgba(7, 9, 11, 0)    0%,
      rgba(7, 9, 11, 0)    20%,
      rgba(7, 9, 11, 0.04) 38%,
      rgba(7, 9, 11, 0.12) 55%,
      rgba(7, 9, 11, 0.28) 70%,
      rgba(7, 9, 11, 0.55) 84%,
      rgba(7, 9, 11, 0.82) 94%,
      rgba(7, 9, 11, 0.92) 100%
    );
    pointer-events: none;
    z-index: 2;
  }
</style>
