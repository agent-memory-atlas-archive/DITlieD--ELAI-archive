<script lang="ts">
  // Starfield — thin shim that mounts the three.js StarCanvas so the
  // existing call site (OrbitCanvas.svelte) keeps compiling.
  //
  // Replaces the legacy SVG starfield with:
  //   • three depth-layered star fields with log-normal sizes
  //   • per-star twinkle (GPU vertex shader)
  //   • independent shooting-star particles tied to star slots
  //   • UnrealBloomPass for HDR-bright objects (the central "sun")
  //   • camera dolly: callers can pass cameraZ to drive a leap-through
  //     -space zoom; perspective division gives true 3D parallax for
  //     free.

  import StarCanvas from '../scene/StarCanvas.svelte';
  import type { OrbitFixture, OrbitMode, ZoomLevel } from '../orbit/types';
  import type { MandalaAnchors, MandalaTextContent } from '../orbit/node-mandala';

  type Density = 'sparse' | 'normal' | 'dense';
  type Tone = 'teal' | 'steel' | 'mixed';

  interface Props {
    density?: Density;
    tone?: Tone;
    centered?: boolean;
    /** Legacy prop, kept for API parity; the new scheduler runs always. */
    shootingStar?: boolean;
    /** Fixture supplies node positions for the three.js galaxy. */
    orbitFixture?: OrbitFixture;
    /** Camera distance from origin (world units). Smaller = closer
     *  ("zoomed in" — leap toward the sun). Animated by the caller
     *  during click-to-zoom / wheel-zoom. */
    cameraDistance?: number;
    /** Camera pitch (degrees). 40 = looking down at a tilted disc. */
    cameraPitchDegrees?: number;
    /** Mouse-driven offset (px); StarCanvas applies a small parallax. */
    mouseX?: number;
    mouseY?: number;
    /** Project node positions and render an HTML label overlay. Off
     *  by default — the OrbitCaption below the canvas names the active
     *  baby step instead. */
    showLabels?: boolean;
    /** Current zoom level — drives which node the 3D camera targets.
     *  concept = sun overview; phase/step/baby = fly to that active
     *  node. Transitions between levels animate as the leap. */
    zoomLevel?: ZoomLevel;
    /** Which 3D mode to render: plan / map / research. */
    orbitMode?: OrbitMode;
    /** Node id to fly camera toward. */
    flyToNodeId?: string | null;
    /** Click callback — node id + world position. */
    onNodeClick?: (id: string, x: number, y: number, z: number) => void;
    /** Hover callback — fires on raycaster hover transitions. */
    onNodeHover?: (id: string | null, type: 'phase' | 'step' | 'baby' | null) => void;
    /** Node id whose projected screen position is tracked for the
     *  NodePanel DOM overlay. Special id '__sun__' = world origin. */
    panelTrackNodeId?: string | null;
    /** Per-frame projection callback for the panel anchor (driven by
     *  panelTrackNodeId). */
    onPanelTrack?: (x: number, y: number, visible: boolean) => void;
    /** Per-frame projected mandala anchors for the V3 NodePanel
     *  (centre, main panel, and 4-or-5 compass markers). */
    onMandalaAnchors?: (anchors: MandalaAnchors) => void;
    /** When true, NodeMandala renders the 5th NW marker. */
    mandalaShowNwMarker?: boolean;
    /** When true, NodeMandala recolours markers per source tier. */
    mandalaTierColored?: boolean;
    /** Flat 3D-text payload for the in-scene mandala labels. */
    mandalaText?: MandalaTextContent;
    /** Fired when the chase camera enters/leaves a warp. The parent
     *  uses this to gate DOM overlays (NodePanel) on warp completion. */
    onWarpStateChange?: (warping: boolean) => void;
    /** Trigger super-sun on product completion. */
    productDone?: boolean;
    /** Toggle G6 perf HUD (dev-only). */
    showPerfOverlay?: boolean;
  }

  let {
    density = 'normal',
    tone = 'mixed',
    centered = false,
    shootingStar: _legacy = false,
    orbitFixture,
    cameraDistance,
    cameraPitchDegrees = 12,
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
    mandalaText,
    onWarpStateChange,
    productDone = false,
    showPerfOverlay = false,
  }: Props = $props();
</script>

<StarCanvas
  {density}
  {tone}
  {centered}
  {orbitFixture}
  {cameraDistance}
  {cameraPitchDegrees}
  {mouseX}
  {mouseY}
  {showLabels}
  {zoomLevel}
  {orbitMode}
  {flyToNodeId}
  {onNodeClick}
  {onNodeHover}
  {panelTrackNodeId}
  {onPanelTrack}
  {onMandalaAnchors}
  {mandalaShowNwMarker}
  {mandalaTierColored}
  {mandalaText}
  {onWarpStateChange}
  {productDone}
  {showPerfOverlay}
  mode="active"
/>
