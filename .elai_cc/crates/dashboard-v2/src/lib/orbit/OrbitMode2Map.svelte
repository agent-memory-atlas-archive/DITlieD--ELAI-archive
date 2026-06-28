<script lang="ts">
  // OrbitMode2Map — codebase MAP mode. Renders NO SVG; the 3D galaxy
  // (Fruchterman-Reingold layout, PageRank-sized nodes, integration-
  // point rings, edge LineSegments) lives in three-orbit-modes.ts and
  // is mounted by StarCanvas. This wrapper exposes a small badge and
  // the symbol count for orientation only.

  import type { OrbitFixture, ZoomTarget, PageRankNodeStub } from './types';

  interface Props {
    fixture: OrbitFixture;
    zoomToLevel: (target: ZoomTarget) => void;
  }
  let { fixture, zoomToLevel: _zoomToLevel }: Props = $props();

  const nodes = $derived<PageRankNodeStub[]>(fixture.pagerank_nodes ?? []);
</script>

<div class="map-badge"><span>codebase map</span><span class="dim">{nodes.length} symbols</span></div>

<style>
  .map-badge {
    position: absolute;
    top: var(--s-3);
    left: var(--s-5);
    z-index: 2;
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
    pointer-events: none;
    font-family: var(--font-mono);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-secondary);
  }
  .map-badge .dim { color: var(--text-ghost); font-size: 10px; }
</style>
