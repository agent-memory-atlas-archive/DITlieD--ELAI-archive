<!--
  Phase 4J / B-25 — EmpireBridgePanel.

  cross-lang-pair: dashboard/tauri/api
  Pairs with two Tauri commands in src-tauri/src/commands_empire.rs:
    - get_empire_status            -> EmpireStatus
    - list_empire_factory_lines    -> FactoryLine[]

  States rendered:
    - state = "unavailable"   -> "Empire bridge unavailable; activate when
                                  factory-brain deployed" empty-state notice
                                  (orchestrator was built without the
                                  `empire` feature).
    - state = "disconnected"  -> bridge present, factory-brain unreachable
                                  (EMPIRE-PENDING: socket boot).
    - state = "connecting"    -> mid-connect handshake.
    - state = "connected"     -> live; render the factory-line list with
                                  brain_id + engine_tier + neuron_count +
                                  last_heartbeat.

  Polls every 10s (same cadence as RuntimePanel / GrammarCompilePanel).
  Svelte 5 runes ($state / $derived / $effect). Plain HTML/inline styles —
  self-contained, no $lib/ui dependency.
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    get_empire_status,
    list_empire_factory_lines,
    type EmpireStatus,
    type EmpireState,
    type FactoryLine,
  } from '$lib/api/empire';

  let status = $state<EmpireStatus>({
    state: 'unavailable',
    feature_enabled: false,
    factory_line_count: 0,
    note: 'Empire bridge unavailable; activate when factory-brain deployed.',
  });
  let lines = $state<FactoryLine[]>([]);
  let loadError = $state<string | null>(null);
  let lastRefreshTs = $state<number | null>(null);
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  async function refresh(): Promise<void> {
    try {
      // The TS wrappers below internally call invoke() against the Tauri
      // backend; both fall back to a deterministic empty shape when running
      // in browser-only mode.
      const [s, fl] = await Promise.all([
        get_empire_status(),
        list_empire_factory_lines(),
      ]);
      status = s;
      lines = fl;
      loadError = null;
      lastRefreshTs = Date.now();
    } catch (err) {
      loadError = err instanceof Error ? err.message : String(err);
    }
  }

  onMount(() => {
    void refresh();
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

  const isConnected = $derived(status.state === 'connected');
  const isUnavailable = $derived(status.state === 'unavailable');
  const showEmpty = $derived(lines.length === 0);

  function badgeClass(s: EmpireState): string {
    switch (s) {
      case 'connected':
        return 'badge ok';
      case 'connecting':
        return 'badge pending';
      case 'disconnected':
        return 'badge warn';
      case 'unavailable':
      default:
        return 'badge muted';
    }
  }

  function formatTs(ms: number | null): string {
    if (!ms) return '—';
    try {
      return new Date(ms).toISOString().replace('T', ' ').slice(0, 19);
    } catch {
      return String(ms);
    }
  }
</script>

<section class="empire-bridge-panel" aria-label="Empire bridge">
  <header>
    <h3>Empire bridge</h3>
    <span class={badgeClass(status.state)}>{status.state}</span>
  </header>

  <div class="summary">
    <div class="metric">
      <span class="label">Feature</span>
      <span class="value">{status.feature_enabled ? 'enabled' : 'disabled'}</span>
    </div>
    <div class="metric">
      <span class="label">Factory lines</span>
      <span class="value">{status.factory_line_count}</span>
    </div>
    <div class="metric">
      <span class="label">Last refresh</span>
      <span class="value">{formatTs(lastRefreshTs)}</span>
    </div>
  </div>

  {#if loadError}
    <p class="error">load error: {loadError}</p>
  {/if}

  {#if isUnavailable}
    <p class="empty">
      Empire bridge unavailable; activate when factory-brain deployed.
      <br />
      <code>{status.note}</code>
    </p>
  {:else if !isConnected}
    <p class="empty">
      Bridge state: <strong>{status.state}</strong>. {status.note}
    </p>
  {:else if showEmpty}
    <p class="empty">Connected. No factory lines yet.</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Brain</th>
          <th>Tier</th>
          <th>Neurons</th>
          <th>Status</th>
          <th>Last heartbeat</th>
        </tr>
      </thead>
      <tbody>
        {#each lines as line (line.brain_id)}
          <tr>
            <td><code>{line.brain_id}</code></td>
            <td>{line.engine_tier}</td>
            <td>{line.neuron_count}</td>
            <td>{line.status}</td>
            <td>{formatTs(line.last_heartbeat_ms)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .empire-bridge-panel {
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
    font-size: 13px;
    font-weight: 600;
  }
  .badge {
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 10px;
    text-transform: lowercase;
  }
  .badge.ok {
    background: #1e3b1e;
    color: #9cf09c;
  }
  .badge.pending {
    background: #1e303b;
    color: #9ccaf0;
  }
  .badge.warn {
    background: #3b321e;
    color: #f0d49c;
  }
  .badge.muted {
    background: #2a2a33;
    color: #9090a8;
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
    font-size: 10px;
    color: #9090a8;
  }
  .metric .value {
    font-size: 13px;
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
  .empty {
    color: #9090a8;
    font-size: 12px;
  }
  .error {
    color: #f06c6c;
    font-size: 12px;
  }
  code {
    background: #1a1a25;
    padding: 1px 4px;
    border-radius: 3px;
  }
</style>
