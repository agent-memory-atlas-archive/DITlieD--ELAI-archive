<script lang="ts">
  // I-27 -- verified-outcome-rate vs compute convergence curve + Pareto view.
  //
  // Surfaces ConvergenceCurveMeasured SSB beacon rows from `elai bench convergence`
  // runs. Part of the H-9 cost cockpit tile family (reads off the SSB DB, same
  // pattern as CostPanel). Polls every 10s; renders curve table + Pareto overlay
  // + knee label + non-monotonic warnings. Wire-dark: tile is always visible but
  // the convergence_curve_measured action_type rows only appear after a
  // `elai bench convergence --suite <fixture> --seeds 5` run.
  //
  // cross-lang pair: this Svelte tile is the consumer side of the
  // ConvergenceCurveMeasured SSB row produced by bench_convergence.rs.
  // cross_lang_diff.py covers Tauri -> Svelte.
  import { onMount, onDestroy } from "svelte";

  type ConvergenceCurvePoint = {
    compute_level: string;
    mean_rate: number;
    ci_low: number;
    ci_high: number;
    cost_per_verified_task: number | null;
  };

  type ConvergenceCurveMeasuredRow = {
    fixture_suite: string;
    seed_count: number;
    task_count: number;
    curve_points: ConvergenceCurvePoint[];
    knee_compute_level: string;
    non_monotonic_at: string[];
    seed: number;
    fixture_blake3: string;
    runpod_pending: boolean;
    ts_ms: number;
  };

  let conv_rows = $state<ConvergenceCurveMeasuredRow[]>([]);
  let conv_error: string | null = $state(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;
  let tauriInvoke: (<T>(cmd: string, args?: Record<string, unknown>) => Promise<T>) | null = null;

  async function ensureInvoke(): Promise<typeof tauriInvoke> {
    if (tauriInvoke !== null) return tauriInvoke;
    const w = typeof window !== "undefined" ? (window as unknown as Record<string, unknown>) : {};
    if (!w.__TAURI_INTERNALS__ && !w.__TAURI__) {
      return null;
    }
    try {
      const mod = await import("@tauri-apps/api/core");
      tauriInvoke = mod.invoke;
    } catch (err) {
      console.warn("[ConvergencePanel] @tauri-apps/api/core import failed", err);
      tauriInvoke = null;
    }
    return tauriInvoke;
  }

  async function refresh() {
    try {
      const inv = await ensureInvoke();
      if (!inv) {
        conv_rows = [];
        return;
      }
      conv_rows = await inv<ConvergenceCurveMeasuredRow[]>(
        "get_convergence_curve_rows",
        { limit: 10 }
      );
      conv_error = null;
    } catch (e) {
      conv_error = String(e);
    }
  }

  onMount(() => {
    refresh();
    pollHandle = setInterval(refresh, 10000);
  });

  onDestroy(() => {
    if (pollHandle) clearInterval(pollHandle);
  });

  function fmtRate(r: number): string {
    return (r * 100).toFixed(1) + "%";
  }

  function fmtCost(c: number | null): string {
    return c !== null ? "$" + c.toFixed(4) : "—";
  }

  function tsLabel(ms: number): string {
    return new Date(ms).toISOString().slice(0, 19).replace("T", " ");
  }
</script>

<div class="conv-panel">
  <header class="conv-header">
    <span class="conv-title">Convergence Curve (I-27)</span>
    {#if conv_rows.length > 0}
      <span class="conv-count">{conv_rows.length} run{conv_rows.length !== 1 ? "s" : ""}</span>
    {/if}
  </header>

  {#if conv_error}
    <div class="conv-error">load error: {conv_error}</div>
  {/if}

  {#if conv_rows.length === 0 && !conv_error}
    <div class="conv-empty">
      No convergence runs yet. Run: <code>elai bench convergence --suite &lt;fixture&gt; --seeds 5</code>
    </div>
  {/if}

  {#each conv_rows as row}
    <section class="conv-run">
      <div class="conv-run-header">
        <span class="conv-suite">{row.fixture_suite}</span>
        <span class="conv-meta">seeds={row.seed_count} tasks={row.task_count}</span>
        <span class="conv-meta">{tsLabel(row.ts_ms)}</span>
        {#if row.runpod_pending}
          <span class="conv-badge conv-badge--pending">fixture-tier</span>
        {/if}
      </div>

      {#if row.non_monotonic_at.length > 0}
        <div class="conv-warning">
          non-monotonic at: {row.non_monotonic_at.join(", ")} — investigate
        </div>
      {/if}

      <div class="conv-knee">
        knee: <strong>{row.knee_compute_level}</strong>
      </div>

      <table class="conv-table">
        <thead>
          <tr>
            <th>compute</th>
            <th>rate</th>
            <th>CI 95%</th>
            <th>$/verified</th>
          </tr>
        </thead>
        <tbody>
          {#each row.curve_points as pt, i}
            <tr
              class={[
                pt.compute_level === row.knee_compute_level ? "conv-knee-row" : "",
                i > 0 && pt.mean_rate < row.curve_points[i - 1].mean_rate ? "conv-dip" : "",
              ]
                .filter(Boolean)
                .join(" ")}
            >
              <td>{pt.compute_level}</td>
              <td>{fmtRate(pt.mean_rate)}</td>
              <td class="conv-ci">[{fmtRate(pt.ci_low)}, {fmtRate(pt.ci_high)}]</td>
              <td>{fmtCost(pt.cost_per_verified_task)}</td>
            </tr>
          {/each}
        </tbody>
      </table>

      <details class="conv-meta-details">
        <summary>provenance</summary>
        <span class="conv-blake3">blake3: {row.fixture_blake3.slice(0, 16)}…</span>
        <span class="conv-seed">seed: {row.seed}</span>
      </details>
    </section>
  {/each}
</div>

<style>
  .conv-panel {
    font-family: var(--font-mono, monospace);
    font-size: 0.82rem;
    color: var(--text-primary, #d0d7de);
    background: var(--bg-surface, #151a20);
    border: 1px solid var(--bg-hover, #1a2028);
    border-radius: 6px;
    padding: 0.75rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .conv-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--bg-hover, #1a2028);
  }

  .conv-title {
    font-weight: 600;
    color: var(--teal-bright, #24a08e);
    letter-spacing: 0.02em;
  }

  .conv-count {
    font-size: 0.75rem;
    color: var(--text-secondary, #6e7681);
    margin-left: auto;
  }

  .conv-error {
    color: var(--fail, #c44040);
    font-size: 0.78rem;
  }

  .conv-empty {
    color: var(--text-secondary, #6e7681);
    font-size: 0.78rem;
    padding: 0.5rem 0;
  }

  .conv-empty code {
    color: var(--teal-bright, #24a08e);
    background: var(--bg-input, #1e2530);
    padding: 0.1em 0.3em;
    border-radius: 3px;
  }

  .conv-run {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding-bottom: 0.75rem;
    border-bottom: 1px dashed var(--bg-hover, #1a2028);
  }

  .conv-run:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .conv-run-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .conv-suite {
    font-weight: 600;
    color: var(--text-primary, #d0d7de);
  }

  .conv-meta {
    font-size: 0.74rem;
    color: var(--text-secondary, #6e7681);
  }

  .conv-badge {
    font-size: 0.7rem;
    padding: 0.1em 0.45em;
    border-radius: 3px;
    font-weight: 600;
    letter-spacing: 0.04em;
  }

  .conv-badge--pending {
    background: color-mix(in oklch, var(--warn, #c49b2a) 20%, transparent);
    color: var(--warn, #c49b2a);
    border: 1px solid color-mix(in oklch, var(--warn, #c49b2a) 40%, transparent);
  }

  .conv-warning {
    font-size: 0.76rem;
    color: var(--warn, #c49b2a);
    background: color-mix(in oklch, var(--warn, #c49b2a) 10%, transparent);
    border: 1px solid color-mix(in oklch, var(--warn, #c49b2a) 30%, transparent);
    border-radius: 4px;
    padding: 0.25rem 0.5rem;
  }

  .conv-knee {
    font-size: 0.76rem;
    color: var(--text-secondary, #6e7681);
  }

  .conv-knee strong {
    color: var(--teal-bright, #24a08e);
  }

  .conv-table {
    border-collapse: collapse;
    width: 100%;
    font-size: 0.78rem;
  }

  .conv-table th {
    text-align: left;
    color: var(--text-secondary, #6e7681);
    font-weight: 500;
    padding: 0.2rem 0.4rem;
    border-bottom: 1px solid var(--bg-hover, #1a2028);
    white-space: nowrap;
  }

  .conv-table td {
    padding: 0.18rem 0.4rem;
    border-bottom: 1px solid color-mix(in oklch, var(--bg-hover, #1a2028) 50%, transparent);
  }

  .conv-table tr:last-child td {
    border-bottom: none;
  }

  .conv-ci {
    color: var(--text-secondary, #6e7681);
    font-size: 0.73rem;
  }

  .conv-knee-row td {
    background: color-mix(in oklch, var(--teal-bright, #24a08e) 8%, transparent);
  }

  .conv-dip td {
    background: color-mix(in oklch, var(--warn, #c49b2a) 8%, transparent);
  }

  .conv-meta-details {
    font-size: 0.72rem;
    color: var(--text-secondary, #6e7681);
    cursor: pointer;
  }

  .conv-meta-details summary {
    user-select: none;
    display: inline-block;
  }

  .conv-blake3,
  .conv-seed {
    margin-left: 0.75rem;
    font-family: var(--font-mono, monospace);
  }

  @media (prefers-reduced-motion: reduce) {
    .conv-panel * {
      transition: none !important;
      animation: none !important;
    }
  }
</style>
