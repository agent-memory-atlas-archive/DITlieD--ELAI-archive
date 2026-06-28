<!-- E-25 Tranche N (s-22600 2026-06-01) CompactionLadderPanel.svelte: -->
<!-- Compaction-Ladder inspector card. Pairs with the 2 E-25 Tauri commands: -->
<!--   get_compaction_ladder_state + recent_compaction_events -->
<!-- Backend: commands_e25.rs (read-only). Producer side is the -->
<!-- task_runner PreCompact branch, gated by -->
<!-- ELAI_E25_COMPACTION_LADDER_ENABLE. Mounted as the 7th SettingsWindow -->
<!-- inspector sibling after C-41's SkillsPanel. Resolves Tranche C C.5 -->
<!-- deferral (BLOCKED-ON-PRODUCER). -->
<!-- See workflow/sessions/s-22600/context-package-trancheN.md. -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as stub from '$lib/api/stub';
  import type { CompactionLadderStateDto, CompactionEventDto } from '$lib/api/types';

  let ladderState: CompactionLadderStateDto | null = $state(null);
  let events: CompactionEventDto[] = $state([]);
  let loadError: string | null = $state(null);
  let lastRefreshTs: number | null = $state(null);
  let pollHandle: number | null = null;

  const refreshLimit = 20;

  async function refresh() {
    try {
      const [snapshot, recent] = await Promise.all([
        stub.get_compaction_ladder_state(),
        stub.recent_compaction_events(refreshLimit),
      ]);
      ladderState = snapshot;
      events = recent;
      loadError = null;
      lastRefreshTs = Date.now();
    } catch (e) {
      loadError = String(e);
    }
  }

  onMount(() => {
    refresh();
    pollHandle = window.setInterval(refresh, 5000);
  });

  onDestroy(() => {
    if (pollHandle !== null) {
      window.clearInterval(pollHandle);
      pollHandle = null;
    }
  });

  function manualRefresh() {
    refresh();
  }

  function formatTs(ms: number): string {
    if (!ms) return '-';
    try {
      return new Date(ms).toLocaleTimeString();
    } catch {
      return String(ms);
    }
  }

  function formatBudget(n: number): string {
    if (n === 0) return '0';
    return n.toLocaleString();
  }

  // Tier-firing counts per canonical tier — drives the highlight ring on
  // the 6-tier overview row. Re-derived on every events[] change.
  const firingCountByTier = $derived.by((): Record<string, number> => {
    const out: Record<string, number> = {};
    for (const ev of events) {
      out[ev.tier] = (out[ev.tier] ?? 0) + 1;
    }
    return out;
  });

  const totalBytesFreed = $derived(
    events.reduce((acc: number, ev: CompactionEventDto) => acc + (ev.bytes_freed || 0), 0),
  );
</script>

<div class="ladder-panel">
  <div class="header-row">
    <div class="title">Compaction Ladder</div>
    <button class="refresh-btn mono" onclick={manualRefresh} aria-label="Refresh compaction ladder">
      refresh
    </button>
  </div>

  {#if ladderState && !ladderState.enabled}
    <div class="banner">
      Compaction ladder is wire-dark by default. Set
      <code>ELAI_E25_COMPACTION_LADDER_ENABLE=1</code>
      on the orchestrator side to enable per-PreCompact tier traversal.
    </div>
  {/if}

  {#if loadError}
    <div class="error-banner">Compaction snapshot unavailable: {loadError}</div>
  {/if}

  <div class="section-label">
    Tier traversal (T1 -> T6, cheapest first)
    {#if ladderState}
      <span class="counts">target = {formatBudget(ladderState.default_target_tokens)} tokens</span>
    {/if}
  </div>
  {#if ladderState}
    <div class="tier-row">
      {#each ladderState.tiers as tier, i (tier)}
        {@const firings = firingCountByTier[tier] ?? 0}
        <div class="tier-cell" class:tier-cell--active={firings > 0}>
          <div class="tier-ordinal mono">T{i + 1}</div>
          <div class="tier-name mono">{tier}</div>
          <div class="tier-firings mono">{firings}x</div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty">Loading tier traversal...</div>
  {/if}

  <div class="section-label">
    Recent compaction events ({events.length} / {refreshLimit})
    {#if events.length > 0}
      <span class="counts">total_bytes_freed = {formatBudget(totalBytesFreed)}</span>
    {/if}
  </div>
  {#if events.length === 0}
    <div class="empty">
      No <code>compaction_ladder_tier_applied</code> SSB rows recorded yet.
      The PreCompact hook only emits when
      <code>ELAI_E25_COMPACTION_LADDER_ENABLE=1</code> AND a PreCompact
      dispatch fires inside <code>task_runner::drive_task</code>.
    </div>
  {:else}
    <table class="events-table">
      <thead>
        <tr>
          <th>Time</th>
          <th>Session</th>
          <th>Tier</th>
          <th class="num">Bytes Freed</th>
          <th class="num">Budget Before</th>
          <th class="num">Budget After</th>
          <th class="num">ms</th>
        </tr>
      </thead>
      <tbody>
        {#each events as ev, i (i)}
          <tr>
            <td class="mono">{formatTs(ev.timestamp_ms)}</td>
            <td class="mono">{ev.session_id ?? '-'}</td>
            <td class="mono">T{ev.tier_ordinal} {ev.tier}</td>
            <td class="num mono">{formatBudget(ev.bytes_freed)}</td>
            <td class="num mono">{formatBudget(ev.budget_before)}</td>
            <td class="num mono">{formatBudget(ev.budget_after)}</td>
            <td class="num mono">{ev.wall_ms}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  <div class="footer">
    {#if lastRefreshTs}
      refreshed {new Date(lastRefreshTs).toLocaleTimeString()}
    {/if}
  </div>
</div>

<style>
  .ladder-panel {
    font-size: 12px;
    padding: 8px 0;
  }
  .header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }
  .title {
    font-weight: 600;
    font-size: 13px;
  }
  .refresh-btn {
    background: #1a1a1a;
    color: #ccc;
    border: 1px solid #2a2a2a;
    padding: 2px 8px;
    font-size: 10px;
    cursor: pointer;
    border-radius: 2px;
  }
  .refresh-btn:hover {
    background: #2a2a2a;
  }
  .banner {
    background: #1c1a14;
    color: #d0b070;
    font-size: 10px;
    padding: 6px 8px;
    margin-bottom: 8px;
    border-left: 2px solid #d0a040;
  }
  .error-banner {
    color: #d05050;
    font-size: 10px;
    padding: 4px 0;
  }
  .empty {
    color: #888;
    font-style: italic;
    padding: 6px 0;
    font-size: 10px;
  }
  .section-label {
    margin-top: 10px;
    margin-bottom: 4px;
    font-size: 10px;
    color: #aaa;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .section-label .counts {
    text-transform: none;
    letter-spacing: 0;
    color: #888;
    font-size: 10px;
    margin-left: 8px;
  }
  .tier-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 4px;
  }
  .tier-cell {
    background: #141414;
    border: 1px solid #2a2a2a;
    border-radius: 2px;
    padding: 4px 8px;
    min-width: 120px;
    flex: 1 1 120px;
  }
  .tier-cell--active {
    border-color: #80a0c0;
    background: #14202a;
  }
  .tier-ordinal {
    font-size: 10px;
    color: #888;
  }
  .tier-name {
    font-size: 10px;
    color: #ccc;
    margin: 2px 0;
  }
  .tier-firings {
    font-size: 10px;
    color: #80a0c0;
  }
  table.events-table {
    width: 100%;
    border-collapse: collapse;
  }
  table.events-table th,
  table.events-table td {
    text-align: left;
    padding: 3px 6px;
    border-bottom: 1px solid #2a2a2a;
    vertical-align: top;
  }
  .num {
    text-align: right;
  }
  .footer {
    margin-top: 6px;
    font-size: 10px;
    color: #888;
  }
  code {
    background: #1a1a1a;
    padding: 1px 4px;
    border-radius: 2px;
    font-size: 10px;
  }
  .mono {
    font-family: 'JetBrains Mono', 'Consolas', monospace;
    font-size: 10px;
  }
</style>
