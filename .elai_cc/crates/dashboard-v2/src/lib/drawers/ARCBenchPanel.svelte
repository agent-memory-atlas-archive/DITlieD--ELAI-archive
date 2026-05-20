<script lang="ts">
  // D-20 — ARCBenchPanel.
  //
  // Three sections:
  //   1. Verdict banner (honestly_evaluated | game_with_memorization |
  //      refused) + last-run summary chips (total, honestly evaluated
  //      count, dominant_mode, dominant_novelty_class).
  //   2. Recent runs table — one row per arc_benchmark_completed SSB
  //      entry with ts + verdict + dominant_* + total.
  //   3. Per-mode + per-novelty-class breakdown chips derived from the
  //      most recent run (dominant_mode + dominant_novelty_class).
  //
  // No producer logic lives here — the orchestrator
  // (`elai_orchestrator::benchmarks::arc_agi2_runner::ARCAGI2Runner::run_all`)
  // emits the SSB row server-side. Polls on a 10s cadence matching the
  // sibling D-{10,11,12,13,16,18,22} panels.
  //
  // Grounding: TODO_DOC[D-20] paper-ingestor follow-up to anchor
  // against ARC-AGI-2 dataset description + ARC Prize 2025 Tech Report.

  import { onMount, onDestroy } from 'svelte';

  type ARCBenchRow = {
    total: number;
    honestly_evaluated_count: number;
    gating_verdict: string;
    fixture_blake3: string;
    dominant_mode: string;
    dominant_novelty_class: string;
    ts_ms: number;
  };

  type RecentARCBenchRuns = {
    rows: ARCBenchRow[];
    total: number;
    last_verdict: string;
    last_dominant_mode: string;
    last_dominant_novelty_class: string;
  };

  let recent = $state<RecentARCBenchRuns>({
    rows: [],
    total: 0,
    last_verdict: '',
    last_dominant_mode: '',
    last_dominant_novelty_class: '',
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
      console.warn(`[arc-bench] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const r = await callInvoke<RecentARCBenchRuns>('get_arc_bench_recent', { limit: 25 });
    if (r !== undefined) recent = r;
    loaded = true;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms || ms <= 0) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function verdictClass(v: string): string {
    switch (v) {
      case 'honestly_evaluated':
        return 'verdict-honest';
      case 'game_with_memorization':
        return 'verdict-game';
      case 'refused':
        return 'verdict-refused';
      default:
        return 'verdict-unknown';
    }
  }

  function verdictLabel(v: string): string {
    switch (v) {
      case 'honestly_evaluated':
        return 'HONESTLY EVALUATED';
      case 'game_with_memorization':
        return 'MEMORIZATION GAMING';
      case 'refused':
        return 'REFUSED';
      default:
        return v.toUpperCase();
    }
  }

  function shortBlake3(s: string): string {
    if (!s || s.length < 12) return s;
    return `${s.slice(0, 8)}…${s.slice(-4)}`;
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
  <h2>ARC-AGI-2 OOD Probe</h2>

  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else if recent.total === 0}
    <p class="muted">No arc_benchmark_completed rows recorded.</p>
  {:else}
    <section class="banner">
      <div class={`verdict ${verdictClass(recent.last_verdict)}`}>
        <strong>{verdictLabel(recent.last_verdict)}</strong>
      </div>
      <div class="chips">
        <span class="chip">runs: {recent.total}</span>
        <span class="chip">mode: <code>{recent.last_dominant_mode || '—'}</code></span>
        <span class="chip">novelty: <code>{recent.last_dominant_novelty_class || '—'}</code></span>
      </div>
    </section>

    <section class="rows">
      <header>
        <strong>Recent runs</strong>
        <span class="muted">{recent.rows.length} rows</span>
      </header>
      <table>
        <thead>
          <tr>
            <th>When</th>
            <th>Verdict</th>
            <th>Total</th>
            <th>Honest</th>
            <th>Dominant mode</th>
            <th>Dominant novelty</th>
            <th>Fixture</th>
          </tr>
        </thead>
        <tbody>
          {#each recent.rows as row}
            <tr>
              <td>{fmtTs(row.ts_ms)}</td>
              <td>
                <span class={`verdict-pill ${verdictClass(row.gating_verdict)}`}>
                  {verdictLabel(row.gating_verdict)}
                </span>
              </td>
              <td>{row.total}</td>
              <td>{row.honestly_evaluated_count}</td>
              <td><code>{row.dominant_mode}</code></td>
              <td><code>{row.dominant_novelty_class}</code></td>
              <td><code title={row.fixture_blake3}>{shortBlake3(row.fixture_blake3)}</code></td>
            </tr>
          {/each}
        </tbody>
      </table>
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
    margin-bottom: 1.25rem;
  }

  section > header {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    margin-bottom: 0.35rem;
  }

  .banner {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .verdict {
    padding: 0.3rem 0.6rem;
    border-radius: 4px;
    border: 1px solid var(--border, #2a2a2a);
    font-size: 0.9rem;
    letter-spacing: 0.05em;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .chip {
    padding: 0.15rem 0.45rem;
    border-radius: 4px;
    border: 1px solid var(--border, #2a2a2a);
    font-size: 0.8rem;
    opacity: 0.88;
  }

  .verdict-pill {
    display: inline-block;
    padding: 0.05rem 0.4rem;
    border-radius: 8px;
    font-size: 0.72rem;
    border: 1px solid var(--border, #2a2a2a);
  }

  .verdict-honest { color: #6fdb6f; border-color: #6fdb6f; }
  .verdict-game { color: #ff6b6b; border-color: #ff6b6b; }
  .verdict-refused { color: #d9b04e; border-color: #d9b04e; }
  .verdict-unknown { color: #888; }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th, td {
    text-align: left;
    padding: 0.25rem 0.5rem;
    border-bottom: 1px solid var(--border, #2a2a2a);
    vertical-align: top;
  }

  th {
    font-weight: 600;
    opacity: 0.75;
  }

  code {
    font-size: 0.8rem;
    opacity: 0.92;
  }

  .muted {
    opacity: 0.55;
  }
</style>
