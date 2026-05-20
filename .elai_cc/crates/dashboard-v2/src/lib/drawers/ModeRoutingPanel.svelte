<script lang="ts">
  // D-10 — ModeRoutingPanel.
  //
  // Read-only view of recent `claim_routed` SSB rows + per-mode
  // histogram via the `get_recent_claim_routings` Tauri command
  // (additive — see crates/dashboard-v2/src-tauri/src/commands_d10.rs).
  //
  // The orchestrator (NOT the policy model) picks the grounding mode
  // for every claim. This panel renders:
  //   * mode distribution histogram (5 columns, % of total)
  //   * recent rows table (claim_id, claim_type, mode, ts)
  //
  // No producer logic lives here — `agent_pool::route_claim` emits
  // the SSB row server-side. Polls on a 10s cadence matching the
  // other D-{1..9} panels.
  //
  // Grounding: arxiv:1908.04734 (Everitt current-RF principle, T1) —
  // agent must not edit the function that grades it.

  import { onMount, onDestroy } from 'svelte';

  type ClaimRoutedRow = {
    claim_id: string;
    claim_type: string;
    mode: string;
    ts_ms: number;
    ctx_summary_blake3: string;
  };

  type RecentClaimRoutings = {
    rows: ClaimRoutedRow[];
    counts_by_mode: Record<string, number>;
    total: number;
  };

  // Snake-case mode keys matching the Rust `GroundingMode::as_str` arm.
  const MODES: ReadonlyArray<string> = [
    'constructive',
    'falsifiability_cross_family',
    'conformal_refusal',
    'operational_deferred',
    'conservative_fallback',
  ];

  const MODE_LABELS: Record<string, string> = {
    constructive: 'CONSTRUCTIVE',
    falsifiability_cross_family: 'FALSIFIABILITY (CROSS-FAMILY)',
    conformal_refusal: 'CONFORMAL REFUSAL',
    operational_deferred: 'OPERATIONAL DEFERRED',
    conservative_fallback: 'CONSERVATIVE FALLBACK',
  };

  let data = $state<RecentClaimRoutings>({
    rows: [],
    counts_by_mode: {},
    total: 0,
  });
  let loaded = $state(false);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(
    cmd: string,
    args?: Record<string, unknown>,
  ): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[mode-routing] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const r = await callInvoke<RecentClaimRoutings>('get_recent_claim_routings', {
      limit: 50,
    });
    if (r !== undefined) {
      data = r;
    }
    loaded = true;
  }

  function pct(mode: string): number {
    if (data.total === 0) return 0;
    const n = data.counts_by_mode[mode] ?? 0;
    return Math.round((n / data.total) * 100);
  }

  function modeCount(mode: string): number {
    return data.counts_by_mode[mode] ?? 0;
  }

  function shortId(id: string): string {
    if (id.length <= 16) return id;
    return `${id.slice(0, 8)}…${id.slice(-4)}`;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms || ms <= 0) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function modeClass(mode: string): string {
    switch (mode) {
      case 'constructive':
        return 'mode-constructive';
      case 'falsifiability_cross_family':
        return 'mode-cross';
      case 'conformal_refusal':
        return 'mode-refusal';
      case 'operational_deferred':
        return 'mode-deferred';
      case 'conservative_fallback':
        return 'mode-fallback';
      default:
        return 'mode-unknown';
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

<div class="panel">
  <h2>Mode Routing</h2>

  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else}
    <section class="histogram">
      <header>
        <strong>Mode distribution</strong>
        <span class="muted">{data.total} routings</span>
      </header>
      {#if data.total === 0}
        <p class="muted">No claims routed yet.</p>
      {:else}
        <table>
          <thead>
            <tr><th>Mode</th><th>Count</th><th>%</th><th>Bar</th></tr>
          </thead>
          <tbody>
            {#each MODES as mode}
              <tr>
                <td><span class={modeClass(mode)}>{MODE_LABELS[mode] ?? mode}</span></td>
                <td>{modeCount(mode)}</td>
                <td>{pct(mode)}%</td>
                <td>
                  <div class="bar" style="width: {pct(mode)}%"></div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="recent">
      <header>
        <strong>Recent routings</strong>
        <span class="muted">{data.rows.length} rows</span>
      </header>
      {#if data.rows.length === 0}
        <p class="muted">No claim_routed rows recorded.</p>
      {:else}
        <table>
          <thead>
            <tr><th>When</th><th>Claim</th><th>Type</th><th>Mode</th></tr>
          </thead>
          <tbody>
            {#each data.rows as row}
              <tr>
                <td>{fmtTs(row.ts_ms)}</td>
                <td><code>{shortId(row.claim_id)}</code></td>
                <td>{row.claim_type}</td>
                <td><span class={modeClass(row.mode)}>{MODE_LABELS[row.mode] ?? row.mode}</span></td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>
  {/if}
</div>

<style>
  .panel {
    padding: 1rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 0.85rem;
  }

  h2 {
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  section {
    margin-top: 1rem;
  }

  section > header {
    display: flex;
    gap: 0.75rem;
    align-items: baseline;
    margin-bottom: 0.5rem;
  }

  .muted {
    color: var(--muted, #888);
  }

  table {
    border-collapse: collapse;
    width: 100%;
  }

  th, td {
    padding: 0.35rem 0.5rem;
    text-align: left;
    border-bottom: 1px solid var(--border, #2a2a2a);
    vertical-align: middle;
  }

  .bar {
    height: 8px;
    background: var(--accent, #4a8);
    border-radius: 2px;
  }

  .mode-constructive {
    color: #4a8;
  }

  .mode-cross {
    color: #f93;
  }

  .mode-refusal {
    color: #a8f;
  }

  .mode-deferred {
    color: #fc6;
  }

  .mode-fallback {
    color: #c66;
  }

  .mode-unknown {
    color: var(--muted, #888);
  }

  code {
    font-family: inherit;
    font-size: inherit;
  }
</style>
