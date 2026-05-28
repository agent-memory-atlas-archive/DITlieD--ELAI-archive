<script lang="ts">
  // E-39 -- cost + token economics dashboard panel.
  //
  // Surfaces 4 read-only views over `cost_token_event` SSB rows:
  //   - velocity-cap rolling-window status (1m / 10m / 1h / 24h)
  //   - recent cost-router decisions
  //   - recent AutoMix verifier verdicts
  //   - per-backend free-tier remaining quota
  //
  // Wire-dark unless ELAI_COST_TOKEN_ECONOMICS_ENABLE=1 on the
  // orchestrator side. Mirrors CriticDecisionPanel layout. Polls every
  // 5s, gracefully renders empty state when no rows exist.
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  type VelocityWindowSnapshot = {
    window: string;
    count: number;
    limit: number;
    kind: string;
    ts_ms: number;
  };
  type VelocityCapStatus = {
    windows: VelocityWindowSnapshot[];
    grand_total: number;
  };
  type CostRouterDecisionRow = {
    session_id: string | null;
    task_id: string | null;
    difficulty: number;
    budget_remaining_usd: number;
    chosen_backend: string;
    criticality: string;
    rationale: string;
    ts_ms: number;
  };
  type AutomixVerdictRow = {
    session_id: string | null;
    task_id: string | null;
    verdict: string;
    reason: string;
    ts_ms: number;
  };
  type FreeTierQuotaRow = {
    backend: string;
    status: string;
    remaining_pct: number;
    ts_ms: number;
  };

  // CLAUDE.md HARD RULE: do not shadow Svelte's $state with $state.
  // Named buckets for each Tauri command return.
  let cost_state_velocity = $state<VelocityCapStatus | null>(null);
  let cost_state_router = $state<CostRouterDecisionRow[]>([]);
  let cost_state_automix = $state<AutomixVerdictRow[]>([]);
  let cost_state_quotas = $state<FreeTierQuotaRow[]>([]);
  let cost_state_error: string | null = $state(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function refresh() {
    try {
      cost_state_velocity = await invoke<VelocityCapStatus>(
        "get_velocity_cap_status",
        { limit: 200 }
      );
      cost_state_router = await invoke<CostRouterDecisionRow[]>(
        "get_cost_router_recent",
        { limit: 25 }
      );
      cost_state_automix = await invoke<AutomixVerdictRow[]>(
        "get_automix_verdicts_recent",
        { limit: 25 }
      );
      cost_state_quotas = await invoke<FreeTierQuotaRow[]>(
        "get_free_tier_quotas",
        { limit: 50 }
      );
      cost_state_error = null;
    } catch (e) {
      cost_state_error = String(e);
    }
  }

  onMount(() => {
    refresh();
    pollHandle = setInterval(refresh, 5000);
  });

  onDestroy(() => {
    if (pollHandle) clearInterval(pollHandle);
  });

  function fmtUsd(cents: number): string {
    return `$${(cents / 100).toFixed(2)}`;
  }

  function backendLabel(s: string): string {
    return s.replace(/_/g, " ");
  }
</script>

<div class="cost-panel">
  <header class="cp-header">
    <span class="cp-title">Cost + Token Economics</span>
    {#if cost_state_velocity}
      <span class="cp-count"
        >velocity rows: {cost_state_velocity.grand_total}</span
      >
    {/if}
  </header>
  {#if cost_state_error}
    <div class="cp-error">load error: {cost_state_error}</div>
  {/if}

  <section class="cp-section">
    <h4 class="cp-section-title">Velocity caps (rolling windows)</h4>
    {#if cost_state_velocity && cost_state_velocity.windows.length > 0}
      <table class="cp-table">
        <thead>
          <tr>
            <th>window</th>
            <th>kind</th>
            <th>count</th>
            <th>limit</th>
          </tr>
        </thead>
        <tbody>
          {#each cost_state_velocity.windows as w}
            <tr class={w.count >= w.limit ? "cp-breached" : ""}>
              <td>{w.window}</td>
              <td>{w.kind}</td>
              <td>{w.count}</td>
              <td>{w.limit}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <div class="cp-empty">no breach rows yet</div>
    {/if}
  </section>

  <section class="cp-section">
    <h4 class="cp-section-title">Cost-router decisions (recent)</h4>
    {#if cost_state_router.length > 0}
      <table class="cp-table">
        <thead>
          <tr>
            <th>backend</th>
            <th>difficulty</th>
            <th>budget $</th>
            <th>criticality</th>
            <th>rationale</th>
          </tr>
        </thead>
        <tbody>
          {#each cost_state_router as r}
            <tr>
              <td>{backendLabel(r.chosen_backend)}</td>
              <td>{r.difficulty.toFixed(2)}</td>
              <td>{r.budget_remaining_usd.toFixed(2)}</td>
              <td>{r.criticality}</td>
              <td class="cp-rationale">{r.rationale}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <div class="cp-empty">no router decisions yet</div>
    {/if}
  </section>

  <section class="cp-section">
    <h4 class="cp-section-title">AutoMix verifier verdicts</h4>
    {#if cost_state_automix.length > 0}
      <table class="cp-table">
        <thead>
          <tr>
            <th>verdict</th>
            <th>reason</th>
          </tr>
        </thead>
        <tbody>
          {#each cost_state_automix as a}
            <tr class="cp-verdict-{a.verdict}">
              <td>{a.verdict}</td>
              <td class="cp-rationale">{a.reason}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <div class="cp-empty">no verifier verdicts yet</div>
    {/if}
  </section>

  <section class="cp-section">
    <h4 class="cp-section-title">Free-tier quotas</h4>
    {#if cost_state_quotas.length > 0}
      <table class="cp-table">
        <thead>
          <tr>
            <th>backend</th>
            <th>status</th>
            <th>remaining %</th>
          </tr>
        </thead>
        <tbody>
          {#each cost_state_quotas as q}
            <tr class="cp-quota-{q.status}">
              <td>{backendLabel(q.backend)}</td>
              <td>{q.status}</td>
              <td>{q.remaining_pct}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <div class="cp-empty">no free-tier quota events yet</div>
    {/if}
  </section>
</div>

<style>
  .cost-panel {
    font: 12px/1.4 ui-monospace, Menlo, Consolas, monospace;
    color: var(--text-fg, #d0d0d0);
    background: var(--panel-bg, transparent);
    padding: 8px 12px;
  }
  .cp-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 6px;
  }
  .cp-title {
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .cp-count {
    color: var(--text-dim, #888);
    font-size: 11px;
  }
  .cp-error {
    color: var(--err-fg, #d97f7f);
    padding: 4px 0;
  }
  .cp-section {
    margin-top: 8px;
  }
  .cp-section-title {
    margin: 6px 0 4px 0;
    font-size: 11px;
    text-transform: uppercase;
    color: var(--text-dim, #888);
    letter-spacing: 0.04em;
  }
  .cp-table {
    width: 100%;
    border-collapse: collapse;
  }
  .cp-table th,
  .cp-table td {
    text-align: left;
    padding: 2px 6px;
    border-bottom: 1px solid var(--panel-border, #333);
  }
  .cp-table th {
    color: var(--text-dim, #888);
    font-weight: 500;
  }
  .cp-rationale {
    color: var(--text-dim, #888);
  }
  .cp-empty {
    color: var(--text-dim, #888);
    font-style: italic;
    padding: 4px 0;
  }
  .cp-breached {
    color: var(--err-fg, #d97f7f);
  }
  .cp-verdict-continue {
    color: var(--ok-fg, #9fc9a9);
  }
  .cp-verdict-retry {
    color: var(--warn-fg, #ddc887);
  }
  .cp-verdict-escalate {
    color: var(--err-fg, #d97f7f);
  }
  .cp-quota-exceeded {
    color: var(--err-fg, #d97f7f);
  }
  .cp-quota-approaching {
    color: var(--warn-fg, #ddc887);
  }
</style>
