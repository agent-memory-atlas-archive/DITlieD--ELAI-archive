<!--
  E-35 GrammarCompilePanel — XGrammar grammar-cache visibility.

  cross-lang-pair: dashboard/tauri/api
  Pairs with one Tauri command in src-tauri/src/commands_e35.rs:
    - get_grammar_compilations()  returns
      {
        rows: GrammarCompilationRow[],
        producer_status: 'pending_sidecar_wiring' | 'ready',
        expected_total: number,
        note: string
      }

  When `producer_status === 'pending_sidecar_wiring'` the panel renders an
  empty-state notice — the sidecar producer (XGrammarLogitsProcessor +
  grammar_cache_loaded SSB rows) lands in followups_for_other_sessions for
  E-35. The wire shape is stable so this panel does NOT change when the
  producer ships.

  Svelte 5 syntax only ($state / $derived / $effect). Plain HTML / inline
  styles to avoid pulling in $lib/ui — keeps the panel self-contained.
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  type GrammarCompilationRow = {
    schema_id: string;
    schema_hash: string;
    compile_status: 'compiled' | 'cached' | 'missing';
    last_used_ms: number;
    hit_count: number;
  };

  type GrammarCompilationsResponse = {
    rows: GrammarCompilationRow[];
    producer_status: 'pending_sidecar_wiring' | 'ready';
    expected_total: number;
    note: string;
  };

  let rows = $state<GrammarCompilationRow[]>([]);
  let producerStatus = $state<string>('pending_sidecar_wiring');
  let expectedTotal = $state<number>(64);
  let note = $state<string>('');
  let loadError = $state<string | null>(null);
  let lastRefreshTs = $state<number | null>(null);
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  type TauriInvoke = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
  let tauriInvoke: TauriInvoke | null = null;

  async function ensureInvoke(): Promise<TauriInvoke | null> {
    if (tauriInvoke !== null) return tauriInvoke;
    const w = typeof window !== 'undefined' ? (window as unknown as Record<string, unknown>) : {};
    if (!w.__TAURI_INTERNALS__ && !w.__TAURI__) {
      return null;
    }
    const mod = await import('@tauri-apps/api/core');
    tauriInvoke = mod.invoke as unknown as TauriInvoke;
    return tauriInvoke;
  }

  async function refresh(): Promise<void> {
    try {
      const invoke = await ensureInvoke();
      if (!invoke) {
        loadError = 'tauri not available (browser-only mode)';
        return;
      }
      const data = await invoke<GrammarCompilationsResponse>('get_grammar_compilations');
      rows = Array.isArray(data?.rows) ? data.rows : [];
      producerStatus = data?.producer_status ?? 'pending_sidecar_wiring';
      expectedTotal = typeof data?.expected_total === 'number' ? data.expected_total : 64;
      note = data?.note ?? '';
      loadError = null;
      lastRefreshTs = Date.now();
    } catch (err) {
      loadError = err instanceof Error ? err.message : String(err);
    }
  }

  onMount(() => {
    void refresh();
    // 10s poll cadence — same order of magnitude as RuntimePanel.
    pollTimer = setInterval(() => {
      void refresh();
    }, 10_000);
  });

  onDestroy(() => {
    if (pollTimer !== null) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  });

  const compiledCount = $derived(
    rows.filter((r) => r.compile_status === 'compiled' || r.compile_status === 'cached').length,
  );
  const missingCount = $derived(rows.filter((r) => r.compile_status === 'missing').length);
  const totalHits = $derived(rows.reduce((acc, r) => acc + (r.hit_count ?? 0), 0));

  function formatTs(ms: number): string {
    if (!ms) return '—';
    try {
      return new Date(ms).toISOString().replace('T', ' ').slice(0, 19);
    } catch {
      return String(ms);
    }
  }

  function shortHash(h: string): string {
    if (typeof h !== 'string') return '—';
    return h.length > 12 ? `${h.slice(0, 6)}…${h.slice(-4)}` : h;
  }
</script>

<section class="grammar-compile-panel">
  <header>
    <h3>XGrammar Grammar Cache</h3>
    <span class="badge {producerStatus === 'ready' ? 'ready' : 'pending'}">
      {producerStatus}
    </span>
  </header>

  <div class="summary">
    <div class="metric">
      <span class="label">Compiled / cached</span>
      <span class="value">{compiledCount} / {expectedTotal}</span>
    </div>
    <div class="metric">
      <span class="label">Missing</span>
      <span class="value">{missingCount}</span>
    </div>
    <div class="metric">
      <span class="label">Total mask applications</span>
      <span class="value">{totalHits}</span>
    </div>
  </div>

  {#if loadError}
    <p class="error">load error: {loadError}</p>
  {/if}

  {#if rows.length === 0}
    <p class="empty">
      {#if producerStatus === 'pending_sidecar_wiring'}
        Sidecar producer not yet wired. Expecting {expectedTotal} grammars
        (8 AgentRole values × 8 BabyStepSpec blocks). See
        <code>followups_for_other_sessions</code> in
        <code>crates/providers/src/runtime/sidecar_xgrammar_hook.rs</code>.
      {:else}
        No grammar compilations recorded yet.
      {/if}
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Schema ID</th>
          <th>Hash</th>
          <th>Status</th>
          <th>Hits</th>
          <th>Last used</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.schema_id + ':' + row.schema_hash)}
          <tr>
            <td><code>{row.schema_id}</code></td>
            <td><code>{shortHash(row.schema_hash)}</code></td>
            <td class="status status-{row.compile_status}">{row.compile_status}</td>
            <td>{row.hit_count ?? 0}</td>
            <td>{formatTs(row.last_used_ms)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  <footer>
    {#if note}
      <p class="note">{note}</p>
    {/if}
    {#if lastRefreshTs !== null}
      <p class="refresh">last refresh: {formatTs(lastRefreshTs)}</p>
    {/if}
  </footer>
</section>

<style>
  .grammar-compile-panel {
    padding: 12px;
    border: 1px solid #2a2a33;
    border-radius: 6px;
    background: #11111a;
    color: #e6e6f0;
    font-family: var(--font-mono, monospace);
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }
  .badge {
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 11px;
  }
  .badge.ready {
    background: #1e3b1e;
    color: #9cf09c;
  }
  .badge.pending {
    background: #3b321e;
    color: #f0d49c;
  }
  .summary {
    display: flex;
    gap: 16px;
    margin-bottom: 10px;
  }
  .metric {
    display: flex;
    flex-direction: column;
  }
  .metric .label {
    font-size: 11px;
    color: #9090a8;
  }
  .metric .value {
    font-size: 14px;
    font-weight: 600;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }
  th,
  td {
    text-align: left;
    padding: 4px 8px;
    border-bottom: 1px solid #1a1a25;
  }
  th {
    color: #9090a8;
    font-weight: 500;
  }
  .status {
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.5px;
  }
  .status-compiled {
    color: #6cd16c;
  }
  .status-cached {
    color: #6ce0f0;
  }
  .status-missing {
    color: #f06c6c;
  }
  .empty,
  .note {
    color: #9090a8;
    font-size: 12px;
  }
  .error {
    color: #f06c6c;
    font-size: 12px;
  }
  .refresh {
    color: #6a6a80;
    font-size: 11px;
  }
  code {
    background: #1a1a25;
    padding: 1px 4px;
    border-radius: 3px;
  }
</style>
