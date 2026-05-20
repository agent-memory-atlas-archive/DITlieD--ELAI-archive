<script lang="ts">
  // D-26 — ArchiveExplorerView.
  //
  // Full-page archive explorer: search/filter recent ReasoningTemplate
  // archive entries, side panel with template metadata, lineage graph
  // pinned to the right via LineageGraphView. Pulls one composite payload
  // from `get_template_archive_combined` so the explorer renders headline
  // tripwire + BAI activity without three round-trips.
  //
  // Read-only. No producer. Consumers of D-1 (archive_chain) + D-4
  // (tripwire) + D-5 (BAI) via the D-26 composite command.

  import LineageGraphView from './LineageGraphView.svelte';

  type TemplateSummary = {
    id: string;
    domain: string;
    pattern_kind: string;
    archive_status: string;
    content_hash_hex: string;
    parent_hash_hex: string | null;
    created_at_ms: number;
    body_len: number;
  };

  type TripwireSummary = {
    seq: number;
    ts: number;
    kind: string;
    tripwire_kind: string | null;
    candidate_id: string | null;
    observed_value: number | null;
    threshold: number | null;
  };

  type ArmSummary = {
    template_id: string;
    alpha: number;
    beta: number;
    n_pulls: number;
    mean_estimate: number;
    was_promoted: boolean;
  };

  type CombinedArchiveView = {
    templates: TemplateSummary[];
    recent_tripwires: TripwireSummary[];
    bai_arms: ArmSummary[];
  };

  let combined = $state<CombinedArchiveView | null>(null);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let searchQuery = $state('');
  let statusFilter = $state<string>('all');
  let selectedId = $state<string>('');
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[archive-explorer] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<CombinedArchiveView>('get_template_archive_combined', { limit: 100 });
    if (out === undefined) {
      combined = { templates: [], recent_tripwires: [], bai_arms: [] };
      loadError = null;
    } else {
      combined = out;
      loadError = null;
    }
    loaded = true;
  }

  async function reloadWithErrorSurface() {
    try {
      if (typeof window === 'undefined') return;
      const mod = await import('@tauri-apps/api/core');
      const out = await mod.invoke<CombinedArchiveView>('get_template_archive_combined', { limit: 100 });
      combined = out;
      loadError = null;
    } catch (e) {
      loadError = String(e);
    } finally {
      loaded = true;
    }
  }

  $effect(() => {
    reload();
    pollHandle = setInterval(reload, 10_000);
    return () => {
      if (pollHandle) {
        clearInterval(pollHandle);
        pollHandle = null;
      }
    };
  });

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

  const filtered = $derived.by(() => {
    if (!combined) return [] as TemplateSummary[];
    const q = searchQuery.trim().toLowerCase();
    return combined.templates.filter((t) => {
      if (statusFilter !== 'all' && t.archive_status !== statusFilter) return false;
      if (q.length === 0) return true;
      return (
        t.id.toLowerCase().includes(q) ||
        t.domain.toLowerCase().includes(q) ||
        t.pattern_kind.toLowerCase().includes(q) ||
        t.content_hash_hex.toLowerCase().includes(q)
      );
    });
  });

  const selectedTemplate = $derived.by(() => {
    if (!combined || !selectedId) return null;
    return combined.templates.find((t) => t.id === selectedId) ?? null;
  });
</script>

<section class="archive-explorer-view">
  <header class="ae-header">
    <h2>Archive Explorer</h2>
    <div class="ae-controls">
      <input
        type="search"
        placeholder="Search id / domain / pattern / hash…"
        bind:value={searchQuery}
        class="ae-search"
      />
      <select bind:value={statusFilter} class="ae-filter">
        <option value="all">All statuses</option>
        <option value="candidate">Candidate</option>
        <option value="promoted">Promoted</option>
        <option value="retired">Retired</option>
        <option value="frozen">Frozen</option>
      </select>
      <button class="ae-reload" onclick={reloadWithErrorSurface}>Reload</button>
    </div>
  </header>

  {#if !loaded}
    <p class="muted">Loading archive…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Archive verification failed.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if !combined || combined.templates.length === 0}
    <p class="muted">No archive entries yet.</p>
  {:else}
    <div class="ae-summary">
      <div class="ae-summary-card">
        <span class="ae-summary-label">Templates</span>
        <span class="ae-summary-value">{combined.templates.length}</span>
      </div>
      <div class="ae-summary-card">
        <span class="ae-summary-label">Recent tripwires</span>
        <span class="ae-summary-value">{combined.recent_tripwires.length}</span>
      </div>
      <div class="ae-summary-card">
        <span class="ae-summary-label">BAI arms</span>
        <span class="ae-summary-value">{combined.bai_arms.length}</span>
      </div>
    </div>

    <div class="ae-body">
      <div class="ae-list">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Domain</th>
              <th>Pattern</th>
              <th>Status</th>
              <th>Created</th>
            </tr>
          </thead>
          <tbody>
            {#each filtered as t (t.id)}
              <tr
                class:selected={t.id === selectedId}
                onclick={() => (selectedId = t.id)}
              >
                <td><code>{shortHash(t.id)}</code></td>
                <td><span class="badge">{t.domain}</span></td>
                <td><span class="badge">{t.pattern_kind}</span></td>
                <td><span class="badge badge-status-{t.archive_status}">{t.archive_status}</span></td>
                <td><time>{fmtTs(t.created_at_ms)}</time></td>
              </tr>
            {/each}
            {#if filtered.length === 0}
              <tr><td colspan="5" class="muted">No entries match filter.</td></tr>
            {/if}
          </tbody>
        </table>
      </div>

      <aside class="ae-side">
        {#if selectedTemplate}
          <h3>Template detail</h3>
          <dl>
            <dt>ID</dt><dd><code>{selectedTemplate.id}</code></dd>
            <dt>Domain</dt><dd>{selectedTemplate.domain}</dd>
            <dt>Pattern</dt><dd>{selectedTemplate.pattern_kind}</dd>
            <dt>Status</dt><dd>{selectedTemplate.archive_status}</dd>
            <dt>Content hash</dt><dd><code>{shortHash(selectedTemplate.content_hash_hex)}</code></dd>
            <dt>Parent hash</dt>
            <dd>
              {#if selectedTemplate.parent_hash_hex}
                <code>{shortHash(selectedTemplate.parent_hash_hex)}</code>
              {:else}
                <span class="muted">root</span>
              {/if}
            </dd>
            <dt>Body bytes</dt><dd>{selectedTemplate.body_len}</dd>
            <dt>Created</dt><dd><time>{fmtTs(selectedTemplate.created_at_ms)}</time></dd>
          </dl>
        {:else}
          <p class="muted">Select a row to inspect.</p>
        {/if}

        <LineageGraphView
          templateId={selectedId}
          onSelect={(id) => (selectedId = id)}
        />
      </aside>
    </div>
  {/if}
</section>

<style>
  .archive-explorer-view {
    padding: 1rem;
    font-size: 0.9rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .ae-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 1rem;
  }
  .ae-header h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }
  .ae-controls {
    display: flex;
    gap: 0.5rem;
  }
  .ae-search {
    padding: 0.25rem 0.5rem;
    font-size: 0.85rem;
    border: 1px solid currentColor;
    background: transparent;
    border-radius: 3px;
    min-width: 18rem;
  }
  .ae-filter {
    padding: 0.25rem 0.5rem;
    font-size: 0.85rem;
    background: transparent;
    border: 1px solid currentColor;
    border-radius: 3px;
  }
  .ae-reload {
    padding: 0.25rem 0.6rem;
    font-size: 0.85rem;
    background: transparent;
    border: 1px solid currentColor;
    cursor: pointer;
    border-radius: 3px;
  }
  .ae-summary {
    display: flex;
    gap: 0.75rem;
  }
  .ae-summary-card {
    display: flex;
    flex-direction: column;
    padding: 0.5rem 0.85rem;
    border: 1px solid rgba(128, 128, 128, 0.3);
    border-radius: 4px;
    min-width: 7rem;
  }
  .ae-summary-label {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    opacity: 0.7;
  }
  .ae-summary-value {
    font-size: 1.1rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .ae-body {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 22rem;
    gap: 1rem;
  }
  .ae-list table {
    width: 100%;
    border-collapse: collapse;
  }
  .ae-list th, .ae-list td {
    padding: 0.25rem 0.45rem;
    text-align: left;
    border-bottom: 1px solid rgba(128, 128, 128, 0.25);
    font-variant-numeric: tabular-nums;
  }
  .ae-list th {
    font-weight: 600;
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    opacity: 0.75;
  }
  .ae-list tr {
    cursor: pointer;
  }
  .ae-list tr:hover {
    background: rgba(128, 128, 128, 0.08);
  }
  .ae-list tr.selected {
    background: rgba(70, 130, 220, 0.15);
  }
  .ae-side {
    border-left: 1px solid rgba(128, 128, 128, 0.25);
    padding-left: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .ae-side h3 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
  }
  .ae-side dl {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.2rem 0.6rem;
    margin: 0;
    font-size: 0.82rem;
  }
  .ae-side dt {
    opacity: 0.7;
  }
  .ae-side dd {
    margin: 0;
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
  .badge-status-promoted { background: rgba(50, 170, 80, 0.2); }
  .badge-status-candidate { background: rgba(220, 180, 50, 0.2); }
  .badge-status-retired { background: rgba(200, 80, 80, 0.2); }
  .badge-status-frozen { background: rgba(70, 130, 220, 0.2); }
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
