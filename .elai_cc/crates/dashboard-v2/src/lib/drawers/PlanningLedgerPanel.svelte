<script lang="ts">
  // E-46 — PlanningLedgerPanel.
  //
  // Renders three slices of the hierarchical-decomposition-tuning
  // surface:
  //   * Dual-ledger task tree with completion bars + status badges.
  //   * Recent AGENTLESS shortcut taxonomy histogram.
  //   * Goal-drift canary gauge (latest drift vs threshold +
  //     per-canary firing counts).
  //
  // Backed by three Tauri commands in commands_e46.rs:
  //   * get_dual_ledger_state(filter?)
  //   * get_recent_agentless_shortcuts({ limit })
  //   * get_goal_drift_status()
  //
  // No producer logic lives here — the orchestrator producers
  // (`agent_pool::process_task_advancement` /
  // `emit_agentless_shortcut_taken` / `emit_goal_drift_fired`) emit
  // the SSB rows separately. This panel polls on a 10s cadence
  // matching the sibling D-{1..10} panels.
  //
  // Grounding (mirrors module docstrings):
  //   * arxiv:2407.01489 (AGENTLESS leaf shortcut)
  //   * arxiv:2412.21154 (Devin/SWE-bench task-ledger pattern)
  //   * arxiv:2505.02709 (goal-drift PRM supervision)

  import { onMount, onDestroy } from 'svelte';

  type DualLedgerTaskDto = {
    task_id: string;
    parent_id: string;
    completion_pct: number;
    status: string;
    latest_blocker: string;
    last_update_ms: number;
  };

  type DualLedgerStateDto = {
    tasks: DualLedgerTaskDto[];
    n_complete: number;
    n_blocked: number;
  };

  type AgentlessShortcutRow = {
    task_id: string;
    reason: string;
    ts_ms: number;
  };

  type GoalDriftRow = {
    initial_goal_blake3: string;
    drift: number;
    threshold: number;
    step: number;
    ts_ms: number;
  };

  type GoalDriftStatusDto = {
    firings: GoalDriftRow[];
    firings_by_canary: Record<string, number>;
    latest_drift: number;
    latest_threshold: number;
  };

  let ledger = $state<DualLedgerStateDto>({ tasks: [], n_complete: 0, n_blocked: 0 });
  let shortcuts = $state<AgentlessShortcutRow[]>([]);
  let drift = $state<GoalDriftStatusDto>({
    firings: [],
    firings_by_canary: {},
    latest_drift: 0,
    latest_threshold: 0,
  });
  let loaded = $state(false);
  let filter = $state<string>('');
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[planning-ledger] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const l = await callInvoke<DualLedgerStateDto>('get_dual_ledger_state', {
      filter: filter || null,
    });
    const s = await callInvoke<AgentlessShortcutRow[]>('get_recent_agentless_shortcuts', {
      limit: 50,
    });
    const d = await callInvoke<GoalDriftStatusDto>('get_goal_drift_status');
    if (l !== undefined) ledger = l;
    if (s !== undefined) shortcuts = s;
    if (d !== undefined) drift = d;
    loaded = true;
  }

  function shortHash(h: string | null | undefined): string {
    if (!h) return '—';
    if (h.length <= 16) return h;
    return `${h.slice(0, 8)}…${h.slice(-4)}`;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms || ms <= 0) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function statusBadgeClass(status: string): string {
    switch (status) {
      case 'complete': return 'badge badge-pass';
      case 'blocked': return 'badge badge-fail';
      case 'in_progress': return 'badge badge-progress';
      case 'pending': return 'badge badge-pending';
      default: return 'badge';
    }
  }

  function reasonHistogram(rows: AgentlessShortcutRow[]): { reason: string; n: number }[] {
    const counts: Record<string, number> = {};
    for (const r of rows) {
      counts[r.reason] = (counts[r.reason] ?? 0) + 1;
    }
    return Object.entries(counts)
      .map(([reason, n]) => ({ reason, n }))
      .sort((a, b) => b.n - a.n);
  }

  function driftGaugeClass(d: number, t: number): string {
    if (t <= 0) return 'gauge gauge-neutral';
    if (d > t) return 'gauge gauge-fail';
    if (d > t * 0.75) return 'gauge gauge-warn';
    return 'gauge gauge-pass';
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
  <h2>Planning Ledger</h2>

  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else}
    <section class="ledger">
      <header>
        <strong>Dual ledger</strong>
        <span class="muted">{ledger.tasks.length} tasks</span>
        <span class="muted">{ledger.n_complete} complete</span>
        <span class="muted">{ledger.n_blocked} blocked</span>
        <span class="filter">
          <label>
            status filter:
            <select bind:value={filter} onchange={() => reload()}>
              <option value="">all</option>
              <option value="pending">pending</option>
              <option value="in_progress">in_progress</option>
              <option value="complete">complete</option>
              <option value="blocked">blocked</option>
            </select>
          </label>
        </span>
      </header>
      {#if ledger.tasks.length === 0}
        <p class="muted">No task_advanced rows yet (orchestrator hasn't run a decomposition).</p>
      {:else}
        <table>
          <thead>
            <tr><th>Task</th><th>Parent</th><th>Progress</th><th>Status</th><th>Blocker</th><th>Updated</th></tr>
          </thead>
          <tbody>
            {#each ledger.tasks as t}
              <tr>
                <td><code>{t.task_id}</code></td>
                <td><code>{t.parent_id || '—'}</code></td>
                <td>
                  <div class="bar">
                    <div class="fill" style="width: {Math.min(100, Math.max(0, t.completion_pct))}%"></div>
                    <span class="pct">{t.completion_pct.toFixed(0)}%</span>
                  </div>
                </td>
                <td><span class={statusBadgeClass(t.status)}>{t.status}</span></td>
                <td>{t.latest_blocker || '—'}</td>
                <td>{fmtTs(t.last_update_ms)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="shortcuts">
      <header>
        <strong>AGENTLESS shortcuts</strong>
        <span class="muted">{shortcuts.length} recent</span>
      </header>
      {#if shortcuts.length === 0}
        <p class="muted">No agentless shortcuts taken yet.</p>
      {:else}
        <ul class="histogram">
          {#each reasonHistogram(shortcuts) as bucket}
            <li>
              <span class="reason"><code>{bucket.reason}</code></span>
              <div class="bar inline">
                <div class="fill" style="width: {Math.min(100, (bucket.n * 100) / Math.max(1, shortcuts.length))}%"></div>
              </div>
              <span class="count">{bucket.n}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="drift">
      <header>
        <strong>Goal-drift canary</strong>
        <span class="muted">{drift.firings.length} firings</span>
      </header>
      <div class="gauge-row">
        <div class={driftGaugeClass(drift.latest_drift, drift.latest_threshold)}>
          <span class="gauge-label">latest drift</span>
          <span class="gauge-value">{drift.latest_drift.toFixed(3)}</span>
          <span class="gauge-threshold">/ {drift.latest_threshold.toFixed(3)}</span>
        </div>
      </div>
      {#if drift.firings.length === 0}
        <p class="muted">No drift firings recorded.</p>
      {:else}
        <table>
          <thead>
            <tr><th>When</th><th>Canary</th><th>Step</th><th>Drift</th><th>Threshold</th></tr>
          </thead>
          <tbody>
            {#each drift.firings.slice(0, 20) as f}
              <tr>
                <td>{fmtTs(f.ts_ms)}</td>
                <td><code>{shortHash(f.initial_goal_blake3)}</code></td>
                <td>{f.step}</td>
                <td>{f.drift.toFixed(3)}</td>
                <td>{f.threshold.toFixed(3)}</td>
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
    flex-wrap: wrap;
  }

  .filter {
    margin-left: auto;
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
    vertical-align: top;
  }

  .badge {
    display: inline-block;
    padding: 0.1rem 0.45rem;
    border-radius: 3px;
    background: var(--badge-bg, #333);
    color: var(--badge-fg, #fff);
    font-size: 0.75rem;
  }

  .badge-pass { background: #1f6d3a; }
  .badge-fail { background: #8a1f1f; }
  .badge-progress { background: #1f4f8a; }
  .badge-pending { background: #555; }

  .bar {
    position: relative;
    background: #222;
    border-radius: 3px;
    height: 1rem;
    min-width: 6rem;
  }

  .bar.inline {
    flex: 1;
    height: 0.85rem;
    margin: 0 0.5rem;
  }

  .fill {
    background: #2c8f4d;
    height: 100%;
    border-radius: 3px;
  }

  .pct {
    position: absolute;
    top: 0;
    left: 0.5rem;
    right: 0.5rem;
    text-align: center;
    font-size: 0.75rem;
    color: #fff;
    pointer-events: none;
  }

  .histogram {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .histogram li {
    display: flex;
    align-items: center;
    margin-bottom: 0.25rem;
  }

  .reason {
    min-width: 10rem;
  }

  .count {
    min-width: 2rem;
    text-align: right;
  }

  .gauge-row {
    display: flex;
    gap: 1rem;
    margin: 0.5rem 0;
  }

  .gauge {
    display: inline-flex;
    flex-direction: column;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    background: #222;
    border-left: 3px solid #555;
  }

  .gauge-pass { border-left-color: #2c8f4d; }
  .gauge-warn { border-left-color: #d09030; }
  .gauge-fail { border-left-color: #c33; }
  .gauge-neutral { border-left-color: #555; }

  .gauge-label {
    color: var(--muted, #888);
    font-size: 0.75rem;
  }

  .gauge-value {
    font-size: 1.15rem;
    font-weight: bold;
  }

  .gauge-threshold {
    color: var(--muted, #888);
    font-size: 0.75rem;
  }

  code {
    font-family: inherit;
    font-size: inherit;
  }
</style>
