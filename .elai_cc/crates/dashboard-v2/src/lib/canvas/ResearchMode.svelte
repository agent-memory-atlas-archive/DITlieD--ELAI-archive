<script lang="ts">
  // ResearchMode.svelte — RESEARCH clipping-frame mode on the canvas. (H-12)
  //
  // DATA REALITY (documented in workflow/reasoning/H-12-decisions.md):
  //
  // REAL (wired now):
  //   - researchData.researchTree (from drawers.svelte.ts / get_research_tree)
  //     is the live research tree. It has: id, query, stage, children,
  //     source_count, converged, phi, psi, stage_events.
  //   - researchData listens to 'research-progress' + 'research-done' Tauri
  //     events for live incremental updates during a running research session.
  //   - The tree renders real nodes (query labels, stage badges, convergence
  //     state, source counts) from the actual backend.
  //
  // STUBBED (registered below, will resolve in H-13 or a later H-plan unit):
  //   - contradiction_with edges (red lines between contradicting sources):
  //     contradiction_with lives on TendrilEvent (orbit/types.ts) which is
  //     orbit-fixture-only data. ResearchTreeNode (api/types.ts:1247) does NOT
  //     carry contradiction_with, and get_research_tree (commands.rs) returns
  //     children:[] with no TendrilEvent payloads. The real research pipeline
  //     (crates/orchestrator/src/research_tree.rs) does not yet emit
  //     TendrilEvent-equivalent data to the frontend.
  //     Stub marker: TODO[H-12.research.contradiction]: wire contradiction edges
  //     will_resolve_in: H-13 (canvas op dispatch) or a dedicated research-wiring
  //     slice once the backend emits source-level contradiction signals.
  //   - cites edges (source-to-node leaf connections): ResearchTreeNode has
  //     source_count but no source URLs. TendrilEvent carries source_url + tier
  //     but only exists in orbit fixture data.
  //     Stub marker: TODO[H-12.research.cites]: wire cites edges from source list
  //     will_resolve_in: same as contradiction (backend must emit source payloads).
  //
  // DATA FEED: researchData from drawers.svelte.ts (GD-1: REUSE existing store).
  // The research store listens to Tauri events for live updates; we bootstrap it
  // here so it is active when ResearchMode is mounted. teardown on destroy.

  import { onMount, onDestroy } from 'svelte';
  import { researchData } from '../state/drawers.svelte';
  import { canvas } from '../state/canvas.svelte';
  import type { ResearchTreeNode } from '../api/types';
  import NodeStateGlyph from './NodeStateGlyph.svelte';

  // ---------------------------------------------------------------------------
  // Named constants (no magic numbers / strings)
  // ---------------------------------------------------------------------------

  /** Maximum tree depth to render before clipping. */
  const MAX_RENDER_DEPTH = 6;

  /** Stage display labels. */
  const STAGE_LABELS: Record<string, string> = {
    FORMULATE: 'formulate',
    SEARCH:    'search',
    EXTRACT:   'extract',
    REFLECT:   'reflect',
    CONVERGE:  'converge',
  };

  // ---------------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------------

  /** Recursively count nodes in a ResearchTreeNode tree. */
  function countResearchNodes(node: ResearchTreeNode): number {
    let n = 1;
    for (const child of node.children ?? []) {
      n += countResearchNodes(child);
    }
    return n;
  }

  /** Return the stage label for display. Unknown stages fallback to the raw value. */
  function stageLabel(stage: string): string {
    return STAGE_LABELS[stage] ?? stage.toLowerCase();
  }

  // ---------------------------------------------------------------------------
  // Store lifecycle — bootstrap so research events stream in while mounted.
  // teardown on destroy so the refcount goes back down (the drawer refcount
  // pattern in drawers.svelte.ts is idempotent for extra bootstrap calls).
  // ---------------------------------------------------------------------------

  // Tracks whether bootstrap() actually ran so teardown() can match it 1:1,
  // unconditional of canvas.mode at destroy time (Svelte fires onDestroy AFTER
  // mode has already changed away from 'research', so a canvas.mode guard here
  // would always be false and teardown would never run — refcount / timer leak).
  let _bootstrapped = false;

  onMount(() => {
    // H-12 WARN fix: only bootstrap the research data store when research mode
    // is actually active. ResearchMode.svelte is always rendered inside the
    // canvas.mode === 'research' branch of CanvasWindow.svelte, so canvas.mode
    // should be 'research' at mount time. The guard eliminates the unnecessary
    // get_research_tree IPC that would otherwise fire on every canvas window
    // mount regardless of the active mode.
    if (canvas.mode === 'research') {
      researchData.bootstrap();
      _bootstrapped = true;
    }
  });

  onDestroy(() => {
    // Unconditional of canvas.mode: by the time onDestroy fires, mode is already
    // different (Svelte unmounts reactively after the mode condition flips), so
    // gating on canvas.mode here would silently skip teardown on every mode switch.
    // _bootstrapped guarantees teardown matches bootstrap exactly 1:1.
    if (_bootstrapped) {
      researchData.teardown();
      _bootstrapped = false;
    }
  });

  // ---------------------------------------------------------------------------
  // Reactive derived state
  // ---------------------------------------------------------------------------

  const tree = $derived(researchData.researchTree);
  const nodeCount = $derived(countResearchNodes(tree));

  /** True when there is real research data (not the empty placeholder). */
  const hasData = $derived(
    tree.id !== 'empty' && tree.query !== 'no active research'
  );

  const hasError = $derived(researchData.lastProgressError !== null);
</script>

<div class="research-mode">
  <!-- Research frame header -->
  <div class="research-header">
    <span class="research-title">Research</span>
    <span class="research-meta">
      {#if hasData}
        {nodeCount} nodes
        {#if tree.converged}
          — <span class="converged-badge">converged</span>
        {:else if tree.stage}
          — <span class="stage-badge">{stageLabel(tree.stage)}</span>
        {/if}
      {:else}
        no active research
      {/if}
    </span>
    <!-- TODO[H-12.research.contradiction]: contradiction edge count badge.
         will_resolve_in: H-13 or dedicated research-wiring slice. -->
  </div>

  <!-- Research tree body -->
  <div class="research-body">
    {#if hasError}
      <div class="research-error">
        Research error: {researchData.lastProgressError}
      </div>
    {:else if !hasData}
      <div class="research-empty">
        No active research. Run a research session to see the tree here.
        <br />
        <span class="stub-note">
          Note: contradiction edges (red) and source cites are deferred
          (TODO[H-12.research.contradiction] / TODO[H-12.research.cites]).
          Will resolve when the backend emits TendrilEvent-equivalent source payloads.
        </span>
      </div>
    {:else}
      <!-- Root query -->
      <div class="research-root">
        <span class="root-icon">◎</span>
        <span class="root-query">{tree.query}</span>
        {#if tree.source_count > 0}
          <span class="root-sources">{tree.source_count} sources</span>
        {/if}
      </div>

      <!-- Tree nodes -->
      {#each tree.children ?? [] as child (child.id)}
        {@render researchNodeRow(child, 1)}
      {/each}
    {/if}
  </div>
</div>

<!-- Recursive snippet for a ResearchTreeNode row (lowercase name avoids
     component-name collision in svelte-check TypeScript inference). -->
{#snippet researchNodeRow(node: ResearchTreeNode, depth: number)}
  {#if depth < MAX_RENDER_DEPTH}
    <div
      class="research-node"
      class:converged={node.converged}
      style:padding-left="{depth * 16 + 8}px"
      aria-label="Research node: {node.query}"
    >
      <!-- Stage badge -->
      <span class="node-stage" title="Stage: {node.stage}">{stageLabel(node.stage)}</span>

      <!-- Query label -->
      <span class="node-query">{node.query}</span>

      <!-- Source count (real data from backend) -->
      {#if node.source_count > 0}
        <span class="node-sources" title="{node.source_count} sources used">
          {node.source_count}s
        </span>
      {/if}

      <!-- Convergence marker — quinary glyph (done = converged) -->
      <span class="converge-mark">
        <NodeStateGlyph state={node.converged ? 'done' : 'active'} size={12} label={node.converged ? 'converged' : 'researching'} />
      </span>

      <!-- phi/psi if present -->
      {#if node.phi !== undefined}
        <span class="node-phi" title="phi={node.phi?.toFixed(2)}">φ{node.phi?.toFixed(1)}</span>
      {/if}
    </div>

    <!-- Recurse children -->
    {#each node.children ?? [] as child (child.id)}
      {@render researchNodeRow(child, depth + 1)}
    {/each}
  {/if}
{/snippet}

<style>
  .research-mode {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    font-family: var(--font-mono);
    font-size: 10px;
    background: var(--bg);
  }

  .research-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--line);
    background: var(--bg);
    flex-shrink: 0;
  }

  .research-title {
    font-weight: 500;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--text);
  }

  .research-meta {
    color: var(--dim);
    font-size: 10px;
  }

  .converged-badge {
    color: var(--d-teal-bright);
    font-weight: 500;
  }

  .stage-badge {
    color: var(--dim);
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  .research-body {
    flex: 1 1 auto;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 6px 0;
  }

  .research-empty {
    padding: 24px 16px;
    color: var(--dim);
    font-size: 10px;
    text-align: center;
    line-height: 1.6;
  }

  .stub-note {
    display: block;
    margin-top: 8px;
    font-size: 9px;
    color: var(--ghost);
    font-style: italic;
  }

  .research-error {
    padding: 12px 16px;
    color: var(--text);
    font-size: 10px;
  }

  .research-root {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 12px;
    border-bottom: 1px solid var(--line);
    margin-bottom: 4px;
    font-size: 10px;
    color: var(--text);
  }

  .root-icon {
    color: var(--d-teal-bright);
    flex-shrink: 0;
  }

  .root-query {
    flex: 1 1 auto;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .root-sources {
    font-size: 9px;
    color: var(--dim);
    flex-shrink: 0;
  }

  .research-node {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 8px;
    min-height: 22px;
    border-left: 2px solid transparent;
    transition: background var(--dur-fast) var(--ease-out);
  }
  .research-node:hover { background: var(--raised); }
  .research-node.converged {
    border-left-color: var(--d-teal-deep);
    opacity: 0.7;
  }

  .node-stage {
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ghost);
    min-width: 50px;
    flex-shrink: 0;
  }

  .node-query {
    flex: 1 1 auto;
    color: var(--read);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 10px;
  }

  .node-sources {
    font-size: 9px;
    color: var(--dim);
    flex-shrink: 0;
  }

  .converge-mark {
    flex-shrink: 0;
    display: inline-flex;
  }

  .node-phi {
    font-size: 8px;
    color: var(--ghost);
    flex-shrink: 0;
  }
</style>
