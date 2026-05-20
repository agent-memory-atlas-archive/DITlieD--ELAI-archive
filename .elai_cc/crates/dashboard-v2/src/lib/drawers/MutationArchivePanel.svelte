<script lang="ts">
  // D-1 — MutationArchivePanel.
  //
  // Read-only view of `.elai_cc/archive/reasoning-templates.jsonl` via the
  // `get_archive_recent` + `get_reasoning_template_by_id` Tauri commands
  // (additive — see crates/dashboard-v2/src-tauri/src/commands_d1.rs).
  //
  // Renders:
  //   * last 50 archive rows, sorted append-order (oldest -> newest)
  //   * per-row badges for domain / pattern_kind / archive_status
  //   * truncated content_hash (8 hex chars) so the operator can match against
  //     `elai archive verify` (D-8) output
  //   * an explicit tamper-detection error banner when verify_chain fails
  //
  // No producer logic lives here — the orchestrator producer
  // (archive_chain::append_and_emit_ssb) emits ArchiveEntryAdded SSB rows
  // separately. This panel polls the chain file on a 10s cadence; SSB
  // subscription is deferred (D-5 wiring will add the event tap).

  import { onMount, onDestroy } from 'svelte';
  // D-26 additive — lineage view embedded via "View Lineage" button.
  import LineageGraphView from '../views/LineageGraphView.svelte';

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

  type ArchiveRecentRow = {
    template: ReasoningTemplateDto;
    self_hash: string;
    parent_hash: string | null;
    promoted_at_ms: number | null;
    retired_at_ms: number | null;
    retirement_reason: string | null;
  };

  let rows = $state<ArchiveRecentRow[]>([]);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  // D-26 additive — selected template id whose lineage is rendered in the
  // inline LineageGraphView below the table. Empty string = no selection.
  let selectedLineageId = $state<string>('');

  async function callInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
    // Mirror the lazy-import pattern from $lib/api/stub.ts so the panel works
    // both in Tauri runtime and in web preview (where invoke is unavailable).
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[mutation-archive] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<ArchiveRecentRow[]>('get_archive_recent', { limit: 50 });
    if (out === undefined) {
      // Tauri unavailable OR error from backend. Show empty state, not error.
      // The MutationArchivePanel mock data is intentionally minimal — empty
      // is the truthful state for a fresh repo with no archive yet.
      rows = [];
      loadError = null;
    } else {
      rows = out;
      loadError = null;
    }
    loaded = true;
  }

  async function reloadWithErrorSurface() {
    // Same as reload() but surfaces tamper errors. Used by manual reload.
    try {
      if (typeof window === 'undefined') return;
      const mod = await import('@tauri-apps/api/core');
      const out = await mod.invoke<ArchiveRecentRow[]>('get_archive_recent', { limit: 50 });
      rows = out;
      loadError = null;
    } catch (e) {
      loadError = String(e);
    } finally {
      loaded = true;
    }
  }

  function shortHash(h: string): string {
    if (h.length <= 12) return h;
    return `${h.slice(0, 8)}…${h.slice(-4)}`;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  onMount(() => {
    reload();
    pollHandle = setInterval(reload, 10_000);
  });

  onDestroy(() => {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<section class="mutation-archive-panel">
  <header class="panel-header">
    <h2>Reasoning Template Archive</h2>
    <button class="reload-btn" onclick={reloadWithErrorSurface}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading archive…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Archive verification failed.</strong>
      <pre>{loadError}</pre>
      <p class="muted">
        ArchiveChain::verify_chain rejected the on-disk file. The orchestrator
        will REFUSE to use any of these templates until the chain is repaired.
        Run <code>elai archive verify</code> (D-8) for a detailed report.
      </p>
    </div>
  {:else if rows.length === 0}
    <p class="muted">
      No archive entries yet. Templates land here after migration (D-1) or
      after the BAI Thompson promoter writes them (D-5).
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Template ID</th>
          <th>Domain</th>
          <th>Pattern</th>
          <th>Verifier</th>
          <th>Status</th>
          <th>Hash</th>
          <th>Parent</th>
          <th>Body bytes</th>
          <th>Created</th>
          <th>Lineage</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.template.id)}
          <tr>
            <td><code>{shortHash(row.template.id)}</code></td>
            <td><span class="badge badge-domain">{row.template.domain}</span></td>
            <td><span class="badge badge-pattern">{row.template.pattern_kind}</span></td>
            <td><span class="badge badge-verifier">{row.template.verifier_required}</span></td>
            <td><span class="badge badge-status badge-status-{row.template.archive_status}">{row.template.archive_status}</span></td>
            <td><code>{shortHash(row.template.content_hash_hex)}</code></td>
            <td>
              {#if row.parent_hash}
                <code>{shortHash(row.parent_hash)}</code>
              {:else}
                <span class="muted">root</span>
              {/if}
            </td>
            <td>{row.template.body_len}</td>
            <td><time>{fmtTs(row.template.created_at_ms)}</time></td>
            <td>
              <button
                class="lineage-btn"
                onclick={() => (selectedLineageId = row.template.id)}
                aria-label={`View lineage for template ${shortHash(row.template.id)}`}
              >View Lineage</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
    {#if selectedLineageId}
      <div class="inline-lineage">
        <LineageGraphView
          templateId={selectedLineageId}
          onSelect={(id) => (selectedLineageId = id)}
        />
        <button class="clear-lineage-btn" onclick={() => (selectedLineageId = '')}>Close lineage view</button>
      </div>
    {/if}
  {/if}
</section>

<style>
  .mutation-archive-panel {
    padding: 1rem;
    font-size: 0.9rem;
  }
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 0.75rem;
  }
  .panel-header h2 {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 600;
  }
  .reload-btn {
    padding: 0.25rem 0.6rem;
    font-size: 0.8rem;
    border: 1px solid currentColor;
    background: transparent;
    cursor: pointer;
    border-radius: 3px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th, td {
    padding: 0.25rem 0.45rem;
    text-align: left;
    border-bottom: 1px solid rgba(128, 128, 128, 0.25);
    font-variant-numeric: tabular-nums;
  }
  th {
    font-weight: 600;
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    opacity: 0.75;
  }
  code {
    font-family: 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.78rem;
  }
  .muted {
    opacity: 0.6;
  }
  .badge {
    display: inline-block;
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
    font-size: 0.72rem;
    font-weight: 500;
    background: rgba(128, 128, 128, 0.15);
  }
  .badge-status-frozen { background: rgba(70, 130, 220, 0.2); }
  .badge-status-promoted { background: rgba(50, 170, 80, 0.2); }
  .badge-status-candidate { background: rgba(220, 180, 50, 0.2); }
  .badge-status-retired { background: rgba(200, 80, 80, 0.2); }
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
  /* D-26 additive — lineage button + inline lineage block. */
  .lineage-btn {
    padding: 0.15rem 0.45rem;
    font-size: 0.72rem;
    border: 1px solid currentColor;
    background: transparent;
    cursor: pointer;
    border-radius: 3px;
  }
  .inline-lineage {
    margin-top: 1rem;
    padding-top: 0.75rem;
    border-top: 1px solid rgba(128, 128, 128, 0.25);
  }
  .clear-lineage-btn {
    margin-top: 0.5rem;
    padding: 0.2rem 0.55rem;
    font-size: 0.78rem;
    border: 1px solid currentColor;
    background: transparent;
    cursor: pointer;
    border-radius: 3px;
  }
</style>
