<script lang="ts">
  // OrbitCanvas — universal 3D orbit canvas. Mounts a single StarCanvas
  // (three.js) and dispatches geometry per mode (plan / map / research).
  // The legacy SVG mode wrappers (OrbitMode1Plan / OrbitMode2Map /
  // OrbitMode3Research) are now thin caption/badge layers; they render
  // NO SVG. The 3D scene replaces them entirely.

  import type { OrbitFixture, OrbitMode, ZoomState, ZoomTarget } from './types';
  import { defaultZoomState, defaultScaleForLevel, zoomShallower, zoomDeeper } from './zoom';
  import { easeOutCubic } from './geometry';
  import { Starfield } from '$lib/ui';
  import OrbitMode1Plan from './OrbitMode1Plan.svelte';
  import OrbitMode2Map from './OrbitMode2Map.svelte';
  import OrbitMode3Research from './OrbitMode3Research.svelte';
  import OrbitDeepSpace from './OrbitDeepSpace.svelte';
  import { resolveVariant, flattenVariantText } from './node-panel-variants';
  import { emptyAnchors, emptyTextContent, type MandalaAnchors } from './node-mandala';

  interface Props {
    fixture: OrbitFixture;
    mode?: OrbitMode;
  }
  let { fixture, mode }: Props = $props();

  // ---------------------------------------------------------------
  // Mode selection: explicit prop wins; otherwise derive from FSM.
  // ---------------------------------------------------------------
  const resolvedMode: OrbitMode = $derived.by(() => {
    if (mode) return mode;
    if (fixture.fsm === 'RESEARCHING') return 'research';
    if (fixture.fsm === 'IDLE' && fixture.pagerank_nodes && fixture.pagerank_nodes.length > 0) return 'map';
    return 'plan';
  });

  // ---------------------------------------------------------------
  // Zoom state — drives chase-camera distance + level inside StarCanvas.
  // For plan mode the zoom levels concept/phase/step/baby map to nodes
  // in the decomposition tree; for map/research the levels are reused
  // as a simple wide/medium/close framing.
  // ---------------------------------------------------------------
  let zoomState = $state<ZoomState>(defaultZoomState());
  let activeAnim: number | null = null;
  let lastAutoZoomMode = $state<OrbitMode | null>(null);
  const ZOOM_DURATION_MS = 420;

  function startZoom(target: { level: ZoomState['level']; cx: number; cy: number; scale: number }) {
    if (activeAnim !== null) cancelAnimationFrame(activeAnim);
    const startState: ZoomState = { ...zoomState };
    const startMs = performance.now();

    const tick = (ts: number) => {
      const t = Math.min(1, (ts - startMs) / ZOOM_DURATION_MS);
      const eased = easeOutCubic(t);
      zoomState = {
        level: eased < 0.5 ? startState.level : target.level,
        cx: startState.cx + (target.cx - startState.cx) * eased,
        cy: startState.cy + (target.cy - startState.cy) * eased,
        scale: startState.scale + (target.scale - startState.scale) * eased,
      };
      if (t < 1) activeAnim = requestAnimationFrame(tick);
      else activeAnim = null;
    };
    activeAnim = requestAnimationFrame(tick);
  }

  function zoomToLevel(target: ZoomTarget) {
    startZoom({ level: target.level, cx: target.x, cy: target.y, scale: target.scale });
  }

  function zoomOut() {
    const level = zoomShallower(zoomState);
    startZoom({ level, cx: 320, cy: 280, scale: defaultScaleForLevel(level) });
  }

  function zoomIn() {
    const level = zoomDeeper(zoomState);
    if (level === zoomState.level) return; // already at deepest
    startZoom({ level, cx: zoomState.cx, cy: zoomState.cy, scale: defaultScaleForLevel(level) });
  }

  // ---------------------------------------------------------------
  // Mouse parallax — translate svg 0..4px on mouse move; preserves
  // the "looking through a window into space" feel from the trial.
  // ---------------------------------------------------------------
  let mouseX = $state(0);
  let mouseY = $state(0);
  let canvasEl = $state<HTMLDivElement | null>(null);
  let reducedMotion = $state(false);

  $effect(() => {
    if (typeof window === 'undefined') return;
    const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
    reducedMotion = mq.matches;
    const handler = (e: MediaQueryListEvent) => { reducedMotion = e.matches; };
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  });

  function onMouseMove(e: MouseEvent) {
    if (reducedMotion || !canvasEl) return;
    const rect = canvasEl.getBoundingClientRect();
    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height / 2;
    mouseX = ((e.clientX - cx) / (rect.width / 2)) * 4;
    mouseY = ((e.clientY - cy) / (rect.height / 2)) * 4;
  }
  function onMouseLeave() {
    mouseX = 0;
    mouseY = 0;
  }

  // ---------------------------------------------------------------
  // Wheel + keyboard shortcuts
  // ---------------------------------------------------------------
  function onWheel(e: WheelEvent) {
    if (reducedMotion) return;
    e.preventDefault();
    if (e.deltaY < 0) zoomIn();
    else zoomOut();
  }

  // Local override — when the user presses ArrowLeft/Right we cycle the
  // active concept without mutating the upstream fixture. Multi-concept
  // tasks let the user "fly" between concepts; single-concept tasks
  // ignore the arrow key.
  let activeConceptOverride = $state<string | null>(null);
  const effectiveActiveConceptId = $derived(
    activeConceptOverride ?? fixture.active_concept_id ?? fixture.concepts[0]?.id ?? null
  );

  // Effective fixture passed to children — same as the upstream fixture
  // except active_concept_id is overridden when the user has navigated.
  const effectiveFixture: OrbitFixture = $derived(
    effectiveActiveConceptId === fixture.active_concept_id || !effectiveActiveConceptId
      ? fixture
      : { ...fixture, active_concept_id: effectiveActiveConceptId }
  );

  function cycleConcept(direction: 1 | -1): void {
    const concepts = fixture.concepts;
    if (concepts.length <= 1) return;
    const cur = effectiveActiveConceptId;
    const idx = Math.max(0, concepts.findIndex((c) => c.id === cur));
    const next = (idx + direction + concepts.length) % concepts.length;
    activeConceptOverride = concepts[next].id;
  }

  // ---------------------------------------------------------------
  // G6 perf overlay (dev-only). Toggled on `~` key. Only active when
  // window.location.hash includes 'dev'. Survives mode swaps.
  // ---------------------------------------------------------------
  let showPerfOverlay = $state(false);
  $effect(() => {
    if (typeof window === 'undefined') return;
    const onKey = (e: KeyboardEvent) => {
      if ((e.target as HTMLElement | null)?.tagName === 'INPUT') return;
      if ((e.target as HTMLElement | null)?.tagName === 'TEXTAREA') return;
      if (e.key === 'Escape' && zoomState.level !== 'concept') zoomOut();
      else if ((e.key === '+' || e.key === '=') && !e.ctrlKey && !e.metaKey) zoomIn();
      else if (e.key === '-' && !e.ctrlKey && !e.metaKey) zoomOut();
      else if (e.key === 'ArrowRight' && !e.ctrlKey && !e.metaKey) cycleConcept(1);
      else if (e.key === 'ArrowLeft' && !e.ctrlKey && !e.metaKey) cycleConcept(-1);
      else if (e.key === '`' || e.key === '~') {
        if (window.location.hash.includes('dev')) {
          showPerfOverlay = !showPerfOverlay;
        }
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });

  $effect(() => {
    return () => {
      if (activeAnim !== null) cancelAnimationFrame(activeAnim);
    };
  });

  // Auto-zoom-out when entering research mode so tendrils extending
  // outside the orbit are visible. Per user direction (Q4): research is
  // autonomous, so the camera zooms itself.
  $effect(() => {
    const m = resolvedMode;
    if (m === lastAutoZoomMode) return;
    lastAutoZoomMode = m;
    if (m === 'research') {
      startZoom({ level: 'concept', cx: 320, cy: 280, scale: 0.7 });
    } else if (m === 'plan' || m === 'map') {
      startZoom({ level: 'concept', cx: 320, cy: 280, scale: 1.0 });
    }
  });

  // Camera distance for the three.js scene — same numbers the SVG
  // mode used so the perspective stays consistent.
  const STARFIELD_CAMERA_DIST_DEFAULT = 1500;
  const STARFIELD_DIST_MIN = 35;
  const STARFIELD_DIST_MAX = 6000;
  const STARFIELD_DIST_EXP = 2.30;
  const starfieldCameraDistance = $derived(
    Math.max(
      STARFIELD_DIST_MIN,
      Math.min(
        STARFIELD_DIST_MAX,
        STARFIELD_CAMERA_DIST_DEFAULT / Math.pow(Math.max(0.4, zoomState.scale), STARFIELD_DIST_EXP),
      ),
    )
  );

  // Generic depth indicator (1..N) instead of concept/phase/step/
  // baby-step names — different tasks have different decomposition
  // depths so the names aren't meaningful in general.
  const ZOOM_LEVEL_INDEX: Readonly<Record<string, number>> = {
    'concept': 1, 'phase': 2, 'step': 3, 'baby-step': 4,
  };
  const ZOOM_LEVEL_TOTAL = 4;
  const levelLabel = $derived(`${ZOOM_LEVEL_INDEX[zoomState.level] ?? 1} / ${ZOOM_LEVEL_TOTAL}`);

  // Click-to-fly target. Cleared after the warp completes (~1.8s).
  let flyToNodeId = $state<string | null>(null);
  function onNodeClick(id: string, _x: number, _y: number, _z: number): void {
    flyToNodeId = id;
    // Pin the NodePanel for the clicked node. Pin survives hover-out;
    // click empty canvas (handled below) unpins.
    selectedNodeId = id;
    selectedNodeType = hoveredNodeType ?? 'phase';
    // Auto-clear after the warp window so subsequent clicks on the same
    // node still re-trigger the effect.
    setTimeout(() => {
      flyToNodeId = null;
    }, 1900);
  }

  // ---------------------------------------------------------------
  // Per-node UI overlay (NodePanel).
  // - hoveredNodeId tracks raycaster output from StarCanvas (clears on
  //   hover-out).
  // - selectedNodeId pins on click; cleared by clicking empty canvas
  //   (onCanvasEmptyClick). selectedNodeId wins over hoveredNodeId.
  // - At baby-step zoom with no hover/selection, the panel falls back
  //   to the active focal node (active baby) so it replaces the now-
  //   removed planet-scan glyphs as the textual surface.
  // ---------------------------------------------------------------
  let hoveredNodeId = $state<string | null>(null);
  let hoveredNodeType = $state<'phase' | 'step' | 'baby' | null>(null);
  let selectedNodeId = $state<string | null>(null);
  let selectedNodeType = $state<'phase' | 'step' | 'baby' | null>(null);
  // Per-frame projected anchors. The DOM panel layer is gone, but we
  // still consume mandalaAnchors.node.visible to gate variant rendering
  // (no point computing variant content if the focal node isn't on
  // screen). Kept around for any future per-frame DOM overlays.
  let mandalaAnchors = $state<MandalaAnchors>(emptyAnchors());
  let canvasW = $state(0);
  let canvasH = $state(0);

  function onNodeHover(id: string | null, type: 'phase' | 'step' | 'baby' | null): void {
    hoveredNodeId = id;
    hoveredNodeType = type;
  }
  function onMandalaAnchorsCb(a: MandalaAnchors): void {
    // Copy fields one-shot — MandalaAnchors is reused on the producer
    // side, so we MUST snapshot or Svelte's reactivity sees no diff.
    mandalaAnchors = {
      node:      { x: a.node.x,      y: a.node.y,      visible: a.node.visible },
      mainPanel: { x: a.mainPanel.x, y: a.mainPanel.y, visible: a.mainPanel.visible },
      n:  { x: a.n.x,  y: a.n.y,  visible: a.n.visible },
      e:  { x: a.e.x,  y: a.e.y,  visible: a.e.visible },
      s:  { x: a.s.x,  y: a.s.y,  visible: a.s.visible },
      w:  { x: a.w.x,  y: a.w.y,  visible: a.w.visible },
      nw: { x: a.nw.x, y: a.nw.y, visible: a.nw.visible },
    };
  }

  // Chase-camera warp state surfaced from StarCanvas. The NodePanel
  // gates its fade-in on this — while the camera is leaping, the panel
  // holds at opacity 0; once the warp completes, it fades in elegantly
  // (see NodePanel.svelte FADE_IN_MS / FADE_EASE). Single source of
  // truth for the warp duration is StarCanvas.WARP_DURATION_MS — the
  // boolean handshake means OrbitCanvas does not duplicate that
  // constant.
  let cameraWarping = $state(false);
  function onWarpStateChange(warping: boolean): void {
    cameraWarping = warping;
  }

  // Active panel target. Selected wins; otherwise hover; otherwise focal
  // at deepest zoom. Resolves to '__sun__' when zoom is concept-level
  // (no node selected) so the panel can describe the concept root.
  const panelTarget = $derived.by<{ id: string; type: 'phase' | 'step' | 'baby' } | null>(() => {
    if (selectedNodeId && selectedNodeType) {
      return { id: selectedNodeId, type: selectedNodeType };
    }
    if (hoveredNodeId && hoveredNodeType) {
      return { id: hoveredNodeId, type: hoveredNodeType };
    }
    if (zoomState.level === 'baby-step' && fixture.active_baby_step_id) {
      return { id: fixture.active_baby_step_id, type: 'baby' };
    }
    if (zoomState.level === 'step' && fixture.active_step_id) {
      return { id: fixture.active_step_id, type: 'step' };
    }
    if (zoomState.level === 'phase' && fixture.active_phase_id) {
      return { id: fixture.active_phase_id, type: 'phase' };
    }
    return null;
  });

  // Diagnostic — log binding transitions only (not per state read).
  let _lastBoundLog = '';
  $effect(() => {
    const tag = `${panelTarget?.id ?? '∅'}/${panelTarget?.type ?? '∅'}/${zoomState.level}`
              + `/h=${hoveredNodeId ?? '∅'}/s=${selectedNodeId ?? '∅'}`;
    if (tag === _lastBoundLog) return;
    _lastBoundLog = tag;
    // eslint-disable-next-line no-console
    console.debug('[mandala] bind', {
      panelTrackNodeId: panelTarget?.id ?? null,
      panelType: panelTarget?.type ?? null,
      zoomLevel: zoomState.level,
      hovered: hoveredNodeId,
      selected: selectedNodeId,
    });
  });

  // Resolve the active variant content once per panel-target change. The
  // mandala flags (5th NW marker, tier-coloured markers) feed back into
  // StarCanvas via props; the compass + main content drive the DOM
  // overlays below.
  const variantContent = $derived.by(() => {
    if (!panelTarget) return null;
    return resolveVariant({
      fixture: effectiveFixture,
      mode: resolvedMode,
      nodeId: panelTarget.id,
      nodeType: panelTarget.type,
    });
  });

  // Diagnostic — log DOM panel render gating transitions only.
  let _lastPanelsLog = '';
  $effect(() => {
    const mainVisible = mandalaAnchors.mainPanel.visible;
    const miniCount = (mandalaAnchors.n.visible ? 1 : 0)
                    + (mandalaAnchors.e.visible ? 1 : 0)
                    + (mandalaAnchors.s.visible ? 1 : 0)
                    + (mandalaAnchors.w.visible ? 1 : 0)
                    + (mandalaAnchors.nw.visible ? 1 : 0);
    const tag = `${!!variantContent}/${mainVisible}/${miniCount}/${cameraWarping}`;
    if (tag === _lastPanelsLog) return;
    _lastPanelsLog = tag;
    // eslint-disable-next-line no-console
    console.debug('[panels] render', {
      hasVariant: !!variantContent,
      mainPanelVisible: mainVisible,
      miniCardCount: miniCount,
      anchorsValid: mandalaAnchors.node.visible,
      cameraWarping,
    });
  });
  const mandalaShowNwMarker = $derived(variantContent?.showNwMarker ?? false);
  const mandalaTierColored  = $derived(variantContent?.tierColored ?? false);

  // ---------------------------------------------------------------
  // Per-node UI is now FULLY 3D — no DOM cards, no DOM panel, no SVG
  // connector layer. NodeMandala renders troika-three-text labels in
  // world space (billboarded toward camera every frame); OrbitCanvas
  // just derives the flat text payload from the resolved variant and
  // hands it to Starfield as a single prop.
  // ---------------------------------------------------------------
  const mandalaText = $derived.by(() =>
    variantContent ? flattenVariantText(variantContent) : emptyTextContent()
  );

  // Track canvas size for NodePanel mirror logic.
  $effect(() => {
    if (typeof window === 'undefined') return;
    if (!canvasEl) return;
    const update = () => {
      const r = canvasEl!.getBoundingClientRect();
      canvasW = r.width;
      canvasH = r.height;
    };
    update();
    const ro = new ResizeObserver(update);
    ro.observe(canvasEl);
    return () => ro.disconnect();
  });

  // Empty-canvas click — unpins the selected panel. The Starfield root
  // already calls onNodeClick when it raycasts a hit; if no hit, the
  // click bubbles up here.
  function onCanvasEmptyClick(ev: MouseEvent): void {
    // Only unpin when the click landed on the canvas root, not on the
    // panel surfaces themselves (they stop propagation via pointer-events:
    // auto). Both the main panel and any mini-card are protected.
    const tgt = ev.target as HTMLElement | null;
    if (tgt && (tgt.closest('.main-panel') || tgt.closest('.mini-card'))) return;
    if (selectedNodeId !== null) {
      // If the click also hit a node, onNodeClick would have already
      // re-set selectedNodeId synchronously before this handler ran
      // (Svelte's onclick runs before document-level handlers). We use
      // a microtask check: if the most recent click changed selection,
      // leave it alone.
      const before = selectedNodeId;
      queueMicrotask(() => {
        if (selectedNodeId === before) {
          selectedNodeId = null;
          selectedNodeType = null;
        }
      });
    }
  }

  // Product-done super-sun trigger (G3).
  const productDone = $derived(fixture.fsm === 'COMPLETE');
</script>

<!-- Orbit canvas root carries pointer interactions (drag-to-look, click
     to dismiss panel, mousemove for parallax + raycaster hover). The
     keyboard shortcuts (Esc/+/- / ArrowLeft / ArrowRight) live in a
     window-level $effect above, not on this div. role="application"
     signals "forward keys/pointer to the app" to assistive tech. -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="orbit-canvas-root"
  bind:this={canvasEl}
  onmousemove={onMouseMove}
  onmouseleave={onMouseLeave}
  onwheel={onWheel}
  onclick={onCanvasEmptyClick}
  role="application"
  aria-label="Orbit visualization"
>
  <Starfield
    density="dense"
    tone="mixed"
    centered
    orbitFixture={effectiveFixture}
    cameraDistance={starfieldCameraDistance}
    zoomLevel={zoomState.level}
    orbitMode={resolvedMode}
    {flyToNodeId}
    {onNodeClick}
    {onNodeHover}
    panelTrackNodeId={panelTarget?.id ?? null}
    onMandalaAnchors={onMandalaAnchorsCb}
    {mandalaShowNwMarker}
    {mandalaTierColored}
    {mandalaText}
    {onWarpStateChange}
    {productDone}
    {showPerfOverlay}
    {mouseX}
    {mouseY}
  />
  <OrbitDeepSpace visible={zoomState.scale < 1.05} />

  <!-- DOM panel layer is GONE. All per-node UI now renders as 3D
       troika-three-text inside NodeMandala (camera-billboarded). -->

  <!-- Mode wrappers are now caption/badge-only; they render NO SVG.
       The 3D scene above is the entire orbit visualization. -->
  <div class="orbit-stage">
    {#if resolvedMode === 'plan'}
      <OrbitMode1Plan {fixture} {zoomState} {zoomToLevel} />
    {:else if resolvedMode === 'map'}
      <OrbitMode2Map {fixture} {zoomToLevel} />
    {:else}
      <OrbitMode3Research {fixture} {zoomToLevel} />
    {/if}
  </div>

  <!-- subtle zoom-level indicator, bottom-right -->
  <div class="zoom-hud" aria-hidden="true">
    <span class="zoom-prefix">zoom</span>
    <span class="zoom-level">{levelLabel}</span>
  </div>
</div>

<style>
  .orbit-canvas-root {
    position: relative;
    width: 100%;
    height: 100%;
    /* Single smooth additive teal bloom in the centre. No banding.
       The Starfield's center-mask handles the vignette darkening. */
    background:
      radial-gradient(ellipse 45% 38% at 50% 50%,
        rgba(30,138,122,0.10) 0%,
        rgba(30,138,122,0.04) 35%,
        transparent 70%),
      var(--bg-base);
    overflow: hidden;
    isolation: isolate;
    cursor: default;
  }

  .orbit-stage {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 1;
  }

  .zoom-hud {
    position: absolute;
    bottom: var(--s-2);
    right: var(--s-4);
    z-index: 3;
    display: flex;
    align-items: baseline;
    gap: 6px;
    pointer-events: none;
    font-family: var(--font-mono);
    color: var(--text-ghost);
    opacity: 0;
    transition: opacity 240ms var(--ease-std);
  }
  .orbit-canvas-root:hover .zoom-hud { opacity: 1; }
  .zoom-prefix {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.15em;
  }
  .zoom-level {
    font-size: 10px;
    color: var(--text-secondary);
    text-transform: lowercase;
  }

</style>
