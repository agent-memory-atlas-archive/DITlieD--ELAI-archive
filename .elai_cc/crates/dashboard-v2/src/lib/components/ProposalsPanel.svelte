<script lang="ts">
  // B-23 Phase 4H Group F (4H.8) — Layer 1A proposals dashboard surface.
  // Pulls pending + applied proposals from the Tauri backend
  // (get_pending_proposals / get_applied_proposals) and lets the operator
  // approve or reject pending entries via the apply / reject Tauri
  // commands. Read-only fallback in browser-only mode.

  import { onMount, onDestroy } from 'svelte';
  import * as stub from '$lib/api/stub';
  import type { SelfModProposalRow } from '$lib/api/types';

  type Row = SelfModProposalRow & { status: 'pending' | 'applied' };

  let pending = $state<SelfModProposalRow[]>([]);
  let applied = $state<SelfModProposalRow[]>([]);
  let loadError = $state<string | null>(null);
  let busy = $state<string | null>(null);
  let timer: ReturnType<typeof setInterval> | null = null;

  function isSelfMod(p: unknown): p is SelfModProposalRow {
    return !!p && typeof p === 'object' && 'proposal_id' in (p as Record<string, unknown>);
  }

  async function load(): Promise<void> {
    try {
      const [p, a] = await Promise.all([
        stub.get_pending_proposals(),
        stub.get_applied_proposals(),
      ]);
      pending = (Array.isArray(p) ? p : []).filter(isSelfMod) as SelfModProposalRow[];
      applied = (Array.isArray(a) ? a : []).filter(isSelfMod) as SelfModProposalRow[];
      loadError = null;
    } catch (e) {
      loadError = String(e);
    }
  }

  async function approve(id: string) {
    busy = id;
    try {
      await stub.apply_proposal_from_user(id);
      await load();
    } catch (e) {
      loadError = `apply ${id}: ${e}`;
    } finally {
      busy = null;
    }
  }

  async function reject(id: string) {
    busy = id;
    try {
      await stub.reject_proposal(id);
      await load();
    } catch (e) {
      loadError = `reject ${id}: ${e}`;
    } finally {
      busy = null;
    }
  }

  onMount(() => {
    void load();
    timer = setInterval(() => void load(), 5_000);
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
  });

  let pendingRows = $derived<Row[]>(pending.map((r) => ({ ...r, status: 'pending' as const })));
  let appliedRows = $derived<Row[]>(applied.map((r) => ({ ...r, status: 'applied' as const })));
</script>

<div class="proposals-panel">
  <h3>Self-Mod Proposals (4H.3B)</h3>
  {#if loadError}
    <div class="err">load error: {loadError}</div>
  {/if}
  <section>
    <h4>Pending ({pendingRows.length})</h4>
    {#if pendingRows.length === 0}
      <div class="empty">no pending proposals</div>
    {:else}
      <ul>
        {#each pendingRows as row (row.proposal_id)}
          <li class:busy={busy === row.proposal_id}>
            <span class="id" title={row.proposal_id}>{row.proposal_id.slice(0, 14)}…</span>
            <span class="table">{row.table}</span>
            <span class="mode">{row.pipeline_mode}</span>
            <span class="fpr">fpr {row.fpr_estimate?.toFixed(3) ?? '?'}</span>
            <span class="occ">×{row.occurrences_seen ?? 0}</span>
            <button onclick={() => approve(row.proposal_id)} disabled={busy !== null}>
              apply
            </button>
            <button onclick={() => reject(row.proposal_id)} disabled={busy !== null}>
              reject
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section>
    <h4>Applied ({appliedRows.length})</h4>
    {#if appliedRows.length === 0}
      <div class="empty">no applied proposals yet</div>
    {:else}
      <ul>
        {#each appliedRows.slice(0, 20) as row (row.proposal_id)}
          <li>
            <span class="id" title={row.proposal_id}>{row.proposal_id.slice(0, 14)}…</span>
            <span class="table">{row.table}</span>
            <span class="mode">{row.pipeline_mode}</span>
            <span class="by">{row.applied_by ?? '—'}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

<style>
  .proposals-panel {
    font-family: var(--mono, ui-monospace, monospace);
    font-size: 12px;
    padding: 8px 12px;
  }
  h3 {
    font-size: 13px;
    margin: 0 0 8px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  h4 {
    font-size: 12px;
    margin: 8px 0 4px;
    color: var(--text-secondary, #888);
  }
  .err {
    color: var(--accent-red, #d33);
    margin: 4px 0;
  }
  .empty {
    color: var(--text-muted, #666);
    font-style: italic;
    padding: 4px 0;
  }
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  li {
    display: grid;
    grid-template-columns: 1fr 1fr 90px 90px 60px auto auto;
    gap: 8px;
    padding: 4px 0;
    border-bottom: 1px solid var(--border-soft, #2a2a2a);
    align-items: center;
  }
  li.busy {
    opacity: 0.5;
  }
  .id { color: var(--text-secondary, #aaa); }
  .table { color: var(--accent-blue, #4a90e2); }
  .mode { color: var(--text-muted, #888); }
  .fpr { color: var(--text-muted, #888); }
  .occ { color: var(--text-muted, #888); }
  .by { color: var(--text-muted, #888); }
  button {
    background: var(--bg-button, #1a1a1a);
    border: 1px solid var(--border-soft, #333);
    color: var(--text-primary, #ddd);
    padding: 2px 8px;
    cursor: pointer;
    font: inherit;
  }
  button:hover:not(:disabled) {
    background: var(--bg-button-hover, #252525);
  }
  button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
</style>
