<script lang="ts">
  // D-26 — LineageGraphView.
  //
  // Read-only SVG visualization of an archive template's parent chain.
  // The chain is fetched via `get_archive_lineage(template_id)` (additive
  // Tauri command in crates/dashboard-v2/src-tauri/src/commands_d26.rs).
  //
  // Layout: vertical chain (root at top, queried leaf at bottom). Nodes are
  // colored by archive_status. No external graph library — plain SVG so
  // we add zero deps (cytoscape IS in package.json but the orchestrator
  // brief says no new deps + reuse plain SVG; we use a deliberately simple
  // vertical layout that fits a parent-only chain cleanly).
  //
  // Producer: ArchiveChain (D-1). Consumer: this component + ArchiveExplorerView.

  type ReasoningTemplateDto = {
    id: string;
    domain: string;
    pattern_kind: string;
    verifier_required: string;
    content_hash_hex: string;
    parent_id: string | null;
    created_at_ms: number;
    archive_status: string;
    body_len: number;
  };

  type LineageNode = {
    template: ReasoningTemplateDto;
    self_hash: string;
    parent_hash: string | null;
    promoted_at_ms: number | null;
    retired_at_ms: number | null;
    retirement_reason: string | null;
    depth: number;
  };

  type ArchiveLineage = {
    entries: LineageNode[];
    root_id: string | null;
    depth: number;
  };

  let { templateId = '', onSelect = (_id: string) => {} }: {
    templateId?: string;
    onSelect?: (id: string) => void;
  } = $props();

  let lineage = $state<ArchiveLineage | null>(null);
  let loading = $state(false);
  let loadError = $state<string | null>(null);

  async function callInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[lineage-graph] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload(id: string) {
    if (!id) {
      lineage = null;
      loadError = null;
      return;
    }
    loading = true;
    loadError = null;
    try {
      const out = await callInvoke<ArchiveLineage>('get_archive_lineage', { templateId: id });
      if (out === undefined) {
        lineage = { entries: [], root_id: null, depth: 0 };
      } else {
        lineage = out;
      }
    } catch (e) {
      loadError = String(e);
      lineage = null;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    reload(templateId);
  });

  function shortHash(h: string): string {
    if (h.length <= 12) return h;
    return `${h.slice(0, 8)}…${h.slice(-4)}`;
  }

  function statusColor(status: string): string {
    switch (status) {
      case 'promoted':
        return 'rgba(50, 170, 80, 0.85)';
      case 'candidate':
        return 'rgba(220, 180, 50, 0.85)';
      case 'retired':
        return 'rgba(200, 80, 80, 0.85)';
      case 'frozen':
        return 'rgba(70, 130, 220, 0.85)';
      default:
        return 'rgba(128, 128, 128, 0.85)';
    }
  }

  // SVG layout constants.
  const NODE_W = 220;
  const NODE_H = 56;
  const V_GAP = 32;
  const PAD = 16;

  let svgW = $derived(NODE_W + PAD * 2);
  let svgH = $derived(
    lineage && lineage.entries.length > 0
      ? lineage.entries.length * (NODE_H + V_GAP) - V_GAP + PAD * 2
      : NODE_H + PAD * 2,
  );
</script>

<section class="lineage-graph-view">
  <header class="lg-header">
    <h3>Template Lineage</h3>
    {#if templateId}
      <code class="lg-templ-id">{shortHash(templateId)}</code>
    {/if}
  </header>

  {#if !templateId}
    <p class="muted">Select a template in the Archive Explorer to view its lineage.</p>
  {:else if loading}
    <p class="muted">Loading lineage…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Lineage load failed.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if !lineage || lineage.entries.length === 0}
    <p class="muted">Template not found in archive chain.</p>
  {:else}
    <div class="lg-meta">
      <span>Depth: <strong>{lineage.depth}</strong></span>
      {#if lineage.root_id}
        <span>Root: <code>{shortHash(lineage.root_id)}</code></span>
      {/if}
    </div>
    <svg width={svgW} height={svgH} role="img" aria-label="Template lineage graph">
      <!-- Edges first so nodes paint on top. -->
      {#each lineage.entries as node, i (node.self_hash)}
        {#if i > 0}
          <line
            x1={PAD + NODE_W / 2}
            y1={PAD + (i - 1) * (NODE_H + V_GAP) + NODE_H}
            x2={PAD + NODE_W / 2}
            y2={PAD + i * (NODE_H + V_GAP)}
            stroke="currentColor"
            stroke-opacity="0.45"
            stroke-width="1.5"
          />
        {/if}
      {/each}
      {#each lineage.entries as node, i (node.self_hash)}
        <g
          class="lg-node"
          transform="translate({PAD}, {PAD + i * (NODE_H + V_GAP)})"
          role="button"
          tabindex="0"
          aria-label={`Lineage node ${i}: ${node.template.archive_status}`}
          onclick={() => onSelect(node.template.id)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onSelect(node.template.id); }}
        >
          <rect
            width={NODE_W}
            height={NODE_H}
            rx="4"
            ry="4"
            fill={statusColor(node.template.archive_status)}
            fill-opacity="0.25"
            stroke={statusColor(node.template.archive_status)}
            stroke-width="1.5"
          />
          <text x="10" y="18" font-size="11" font-family="monospace" fill="currentColor">
            {shortHash(node.template.id)}
          </text>
          <text x="10" y="34" font-size="10" font-family="monospace" fill="currentColor" opacity="0.75">
            {node.template.domain} / {node.template.pattern_kind}
          </text>
          <text x="10" y="50" font-size="10" font-family="monospace" fill="currentColor" opacity="0.75">
            {node.template.archive_status} · d{node.depth}
          </text>
        </g>
      {/each}
    </svg>
  {/if}
</section>

<style>
  .lineage-graph-view {
    padding: 1rem;
    font-size: 0.9rem;
  }
  .lg-header {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }
  .lg-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }
  .lg-templ-id {
    font-family: 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.78rem;
    opacity: 0.75;
  }
  .lg-meta {
    display: flex;
    gap: 1rem;
    margin-bottom: 0.5rem;
    font-size: 0.78rem;
    opacity: 0.85;
  }
  .lg-meta code {
    font-family: 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.78rem;
  }
  .lg-node {
    cursor: pointer;
  }
  .lg-node:hover rect {
    fill-opacity: 0.45;
  }
  .lg-node:focus {
    outline: none;
  }
  .lg-node:focus rect {
    stroke-width: 2.5;
  }
  .muted {
    opacity: 0.6;
  }
  .error-banner {
    background: rgba(200, 80, 80, 0.12);
    border: 1px solid rgba(200, 80, 80, 0.45);
    border-radius: 4px;
    padding: 0.75rem;
  }
  .error-banner pre {
    white-space: pre-wrap;
    font-family: 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.78rem;
    margin: 0.4rem 0;
  }
</style>
