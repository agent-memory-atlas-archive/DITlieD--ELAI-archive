<script lang="ts">
  // OrbitMode3Research — research-tree mode. Renders NO SVG; the 3D
  // tendril paths (QuadraticBezierCurve3, returning particles colored
  // by tier) are produced by three-orbit-modes.ts inside StarCanvas.
  // This wrapper only exposes a small "research mode" badge and the
  // node count for orientation.

  import type { OrbitFixture, ZoomTarget } from './types';
  import type { ResearchTreeNode } from '../api/types';

  interface Props {
    fixture: OrbitFixture;
    zoomToLevel: (target: ZoomTarget) => void;
  }
  let { fixture, zoomToLevel: _zoomToLevel }: Props = $props();

  const tree = $derived<ResearchTreeNode | undefined>(fixture.research_tree);

  const nodeCount = $derived.by(() => {
    if (!tree) return 0;
    let n = 0;
    function walk(t: ResearchTreeNode): void {
      n++;
      for (const c of t.children ?? []) walk(c);
    }
    walk(tree);
    return n;
  });
</script>

<div class="research-badge"><span>research mode</span><span class="dim">{nodeCount} nodes</span></div>

<style>
  .research-badge {
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
  .research-badge .dim { color: var(--text-ghost); font-size: 10px; }
</style>
