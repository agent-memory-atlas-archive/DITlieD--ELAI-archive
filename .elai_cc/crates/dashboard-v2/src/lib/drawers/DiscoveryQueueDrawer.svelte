<script lang="ts">
  // D-27 — DiscoveryQueueDrawer.
  //
  // Lists recent DiscoveryCandidate-classified templates pending
  // adversarial verification. Pure read off the D-27 composite
  // `get_mode_routing_refusal_combined`; we filter the
  // `discovery_queue` field locally for the drawer table.
  //
  // Each row shows: template_id, novelty class, dkg_match_count,
  // days_waiting, and a Quick-promote button. The button is a no-op
  // signal-emitter for now (console.log) — promotion semantics will
  // wire to a follow-up adversarial-verification dispatcher; the
  // affordance lives in the dashboard so operators can flag the
  // candidate without paying CLI friction.
  //
  // Read-only producer-side. No new SSB variant.
  //
  // Grounding: arXiv:2406.05761 (LLM novelty detection — three-bucket
  // taxonomy); docs/specs/guide/real-problem-solving-grounded-2026-
  // 05-19.md §1 Mode C + §4.8.

  type DiscoveryQueueRow = {
    template_id: string;
    class: string;
    dkg_match_count: number;
    near_duplicate_threshold: number;
    ts_ms: number;
    days_waiting: number;
  };

  type ModeRoutingRefusalView = {
    mode_breakdown: Record<string, number>;
    refusal_rate_current: number;
    refusal_health_band: string;
    discovery_queue: DiscoveryQueueRow[];
    recent_classifications: unknown[];
    dominant_mode: string | null;
    limit_applied: number;
  };

  let rows = $state<DiscoveryQueueRow[]>([]);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[discovery-queue] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    try {
      const out = await callInvoke<ModeRoutingRefusalView>(
        'get_mode_routing_refusal_combined',
        { limit: 100 },
      );
      if (out === undefined) {
        rows = [];
        loadError = null;
      } else {
        rows = out.discovery_queue;
        loadError = null;
      }
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

  function fmtTs(ms: number): string {
    if (!ms) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function onQuickPromote(row: DiscoveryQueueRow) {
    // Signal-only: surface intent to the console for now. The follow-
    // up wiring path (adversarial-verification dispatcher) is owned by
    // a downstream session per the D-27 followups note.
    console.log('[discovery-queue] quick-promote requested', {
      template_id: row.template_id,
      class: row.class,
      dkg_match_count: row.dkg_match_count,
      days_waiting: row.days_waiting,
    });
  }
</script>

<section class="discovery-queue">
  <header class="dq-header">
    <h3>Discovery Queue</h3>
    <span class="dq-sub">{rows.length} pending</span>
    {#if loadError}
      <span class="dq-error">{loadError}</span>
    {/if}
  </header>

  {#if !loaded}
    <p class="dq-empty">loading discovery queue…</p>
  {:else if rows.length === 0}
    <p class="dq-empty">no DiscoveryCandidate classifications in the recent window.</p>
  {:else}
    <table class="dq-table">
      <thead>
        <tr>
          <th>template_id</th>
          <th>class</th>
          <th>dkg matches</th>
          <th>days waiting</th>
          <th>classified at</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.template_id + ':' + row.ts_ms)}
          <tr>
            <td class="dq-id">{row.template_id}</td>
            <td>{row.class}</td>
            <td class="dq-num">{row.dkg_match_count}</td>
            <td class="dq-num">{row.days_waiting}</td>
            <td>{fmtTs(row.ts_ms)}</td>
            <td>
              <button
                class="dq-promote"
                type="button"
                onclick={() => onQuickPromote(row)}
              >Quick-promote</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .discovery-queue {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px;
  }
  .dq-header {
    display: flex;
    align-items: baseline;
    gap: 12px;
    border-bottom: 1px solid var(--border-subtle, #2a2a2a);
    padding-bottom: 4px;
  }
  .dq-header h3 {
    margin: 0;
    font-size: 12px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }
  .dq-sub {
    font-size: 10px;
    color: var(--text-muted, #888);
  }
  .dq-error {
    font-size: 10px;
    color: #f66;
  }
  .dq-empty {
    font-size: 11px;
    color: var(--text-muted, #888);
    margin: 4px 0;
  }
  .dq-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
  }
  .dq-table th,
  .dq-table td {
    border-bottom: 1px solid var(--border-subtle, #232323);
    padding: 4px 6px;
    text-align: left;
  }
  .dq-table th {
    font-weight: 500;
    color: var(--text-muted, #888);
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.05em;
  }
  .dq-id {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
  }
  .dq-num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .dq-promote {
    font-size: 10px;
    padding: 2px 8px;
    border: 1px solid var(--border-subtle, #2a2a2a);
    background: var(--surface-2, #1f1f1f);
    color: inherit;
    cursor: pointer;
  }
  .dq-promote:hover {
    border-color: #4a8;
  }
</style>
