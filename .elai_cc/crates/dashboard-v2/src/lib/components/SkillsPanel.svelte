<!-- C-41 SkillsPanel.svelte: Application Skills System inspector card. -->
<!-- Svelte 5 runes. Pairs with the 2 C-41 Tauri commands: -->
<!--   list_application_skills + recent_skill_routing_events -->
<!-- Backend: commands_c41.rs (read-only). Producer side is the -->
<!-- task_runner pre-classify hook, gated by -->
<!-- ELAI_APPLICATION_SKILLS_ENABLE. Mounted as the 6th SettingsWindow -->
<!-- inspector sibling after E-37's PromptArmorPanel. -->
<!-- See workflow/reasoning/C-41-decisions.md (sub-phase 5A + 5B). -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as stub from '$lib/api/stub';
  import type {
    SkillManifest,
    SkillRoutingEvent,
    SkillLibraryResponse,
    SkillLibraryRow,
    SkillLadderEntry,
  } from '$lib/api/types';

  let skills = $state<SkillManifest[]>([]);
  let events = $state<SkillRoutingEvent[]>([]);
  let loadError = $state<string | null>(null);
  let lastRefreshTs = $state<number | null>(null);
  let pollHandle: number | null = null;
  let enabled = $state(false);

  // F-6 / A5 — learned-skill library Trust-Ladder state. Additive: this section
  // is independent of the C-41/E-43 application-skills render above. The
  // learned-skill library has NO wire-dark ENABLE gate (W-2); acquisition is
  // governed by the producer-side autonomy barrier, so this always queries.
  let library = $state<SkillLibraryResponse>({ events: [], ladder: [] });
  let libraryError = $state<string | null>(null);

  // The four rungs of the F-6 trust ladder, lowest to apex. Only S4 is cleared
  // for autonomous dispatch (sub-S4 routes to a human via the autonomy barrier).
  const LADDER_TIERS = ['s1', 's2', 's3', 's4'] as const;

  // Group the derived ladder snapshot by tier so the panel can render one
  // column/row per rung. BTreeMap-sorted on the Rust side; we keep insertion
  // order here (skill_id-sorted) within each tier bucket.
  let ladderByTier = $derived.by(() => {
    const buckets: Record<string, SkillLadderEntry[]> = {
      s1: [],
      s2: [],
      s3: [],
      s4: [],
    };
    for (const e of library.ladder) {
      (buckets[e.tier] ??= []).push(e);
    }
    return buckets;
  });

  // Read the orchestrator gate via the same import.meta convention the
  // dashboard uses elsewhere. Falling back to `false` keeps the empty-state
  // banner visible by default.
  $effect(() => {
    const fromEnv =
      (typeof import.meta !== 'undefined' &&
        (import.meta as ImportMeta & { env?: Record<string, string> }).env?.[
          'VITE_ELAI_APPLICATION_SKILLS_ENABLE'
        ]) ||
      '';
    enabled = fromEnv === '1' || fromEnv === 'true';
  });

  const refreshLimit = 20;

  async function refresh() {
    try {
      const [manifests, recent] = await Promise.all([
        stub.list_application_skills(),
        stub.recent_skill_routing_events(refreshLimit),
      ]);
      skills = manifests;
      events = recent;
      loadError = null;
      lastRefreshTs = Date.now();
    } catch (e) {
      loadError = String(e);
    }
    // F-6 / A5 — independent fetch so a learned-skill-library failure never
    // blanks the C-41/E-43 application-skills render above (and vice versa).
    try {
      library = await stub.get_skill_library(refreshLimit);
      libraryError = null;
    } catch (e) {
      libraryError = String(e);
    }
  }

  function tierLabel(tier: string): string {
    switch (tier) {
      case 's1':
        return 'S1 fresh';
      case 's2':
        return 'S2 proven';
      case 's3':
        return 'S3 exercised';
      case 's4':
        return 'S4 apex (autonomy-permitted)';
      default:
        return tier;
    }
  }

  function kindLabel(kind: string): string {
    switch (kind) {
      case 'acquired':
        return 'acquired';
      case 'trust_promoted':
        return 'promoted';
      case 'demoted_stale':
        return 'demoted (stale)';
      default:
        return kind;
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

  function hashToHex(h: number): string {
    // Display the low 8 hex digits — enough to identify the skill row
    // without crowding the routing-event table.
    const u = h >>> 0;
    return u.toString(16).padStart(8, '0').slice(-8);
  }

  function formatTs(ms: number): string {
    if (!ms) return '-';
    try {
      return new Date(ms).toLocaleTimeString();
    } catch {
      return String(ms);
    }
  }

  function nameForHash(h: number): string {
    // Linear scan over the 9 manifests is cheap — skill_name_hash is the
    // truncated blake3 of the name. We mirror the orchestrator's
    // skill_name_hash with a JS reimplementation OFF the hot path; the
    // dashboard does not need to compute it locally because the routing
    // event already carries the hash + we can look it up by walking the
    // currently-loaded manifest set. For visual parity we surface the
    // first match by name-hash equality; mismatches show as "-".
    // The JS-side hashing reimplementation lives intentionally OUT of
    // scope (would require a blake3 npm dep — Anti-Slopsquatting rule).
    return `name_hash=0x${hashToHex(h)}`;
  }

  let triggeredCount = $derived(
    events.filter((e) => e.action_type === 'skill_triggered').length,
  );
  let executedCount = $derived(
    events.filter((e) => e.action_type === 'skill_executed').length,
  );
  let fellThroughCount = $derived(events.filter((e) => e.fell_through).length);
</script>

<div class="skills-panel">
  <div class="header-row">
    <div class="title">Application Skills</div>
    <button class="refresh-btn mono" onclick={manualRefresh} aria-label="Refresh skills">
      refresh
    </button>
  </div>

  {#if !enabled}
    <div class="banner">
      Application skills are wire-dark by default. Set
      <code>ELAI_APPLICATION_SKILLS_ENABLE=1</code>
      to enable pre-classify routing.
    </div>
  {/if}

  {#if loadError}
    <div class="error-banner">Skills snapshot unavailable: {loadError}</div>
  {/if}

  <div class="section-label">Registered skills ({skills.length})</div>
  {#if skills.length === 0}
    <div class="empty">
      No application skills found under <code>.elai_cc/skills/</code>. The
      directory may not exist on this project, or the discovery walker
      failed silently (check tracing logs).
    </div>
  {:else}
    <table class="skills-table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Description</th>
          <th class="num">Mins</th>
          <th>Tags</th>
        </tr>
      </thead>
      <tbody>
        {#each skills as s (s.name)}
          <tr>
            <td class="mono">{s.name}</td>
            <td class="desc">{s.description}</td>
            <td class="num mono">{s.estimated_minutes}</td>
            <td class="tags">
              {#each s.tags as t (t)}
                <span class="tag-chip">{t}</span>
              {/each}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  <div class="section-label">
    Recent routing events ({events.length} / {refreshLimit})
    {#if events.length > 0}
      <span class="counts">
        triggered={triggeredCount} · executed={executedCount} · fell_through={fellThroughCount}
      </span>
    {/if}
  </div>
  {#if events.length === 0}
    <div class="empty">
      No <code>skill_triggered</code> or <code>skill_executed</code> SSB rows
      recorded yet. The pre-classify hook only emits when
      <code>ELAI_APPLICATION_SKILLS_ENABLE=1</code> on the orchestrator side.
    </div>
  {:else}
    <table class="events-table">
      <thead>
        <tr>
          <th>Time</th>
          <th>Type</th>
          <th>Skill</th>
          <th class="num">Score</th>
          <th class="num">Rank</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        {#each events as ev, i (i)}
          <tr class:row-fell-through={ev.fell_through}>
            <td class="mono">{formatTs(ev.timestamp_ms)}</td>
            <td class="mono">{ev.action_type}</td>
            <td class="mono">{nameForHash(ev.skill_name_hash)}</td>
            <td class="num mono">{ev.score.toFixed(2)}</td>
            <td class="num mono">{ev.top_k_rank}</td>
            <td>
              {#if ev.fell_through}
                <span class="badge-fell-through">fell-through</span>
              {:else if ev.action_type === 'skill_executed'}
                <span class="badge-ok">ok</span>
              {:else}
                <span class="badge-routed">routed</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  <!-- F-6 / A5 — learned-skill library Trust-Ladder. Additive section; the
       C-41/E-43 application-skills render above is untouched. -->
  <div class="section-label trust-ladder-head">
    Learned-skill Trust-Ladder
    {#if library.ladder.length > 0}
      <span class="counts">{library.ladder.length} skill(s) tracked</span>
    {/if}
  </div>

  {#if libraryError}
    <div class="error-banner">Skill library unavailable: {libraryError}</div>
  {/if}

  {#if library.ladder.length === 0}
    <div class="empty">
      No learned skills acquired yet. Skills are acquired after a critic-gated
      success (MiniCritic <code>Pass</code>) and climb S1 &rarr; S4; only
      <code>S4</code> is cleared for autonomous dispatch.
    </div>
  {:else}
    <div class="ladder">
      {#each LADDER_TIERS as tier (tier)}
        <div class="ladder-rung" class:rung-apex={tier === 's4'}>
          <div class="rung-label mono">{tierLabel(tier)}</div>
          <div class="rung-skills">
            {#if ladderByTier[tier].length === 0}
              <span class="rung-empty">—</span>
            {:else}
              {#each ladderByTier[tier] as s (s.skill_id)}
                <span
                  class="skill-chip mono"
                  title={`${s.invocations} inv · fail ${(s.failure_rate * 100).toFixed(0)}%`}
                >
                  {s.skill_id}
                </span>
              {/each}
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <div class="section-label">
    Recent library events ({library.events.length} / {refreshLimit})
  </div>
  {#if library.events.length === 0}
    <div class="empty">
      No <code>skill_acquired</code> / <code>skill_trust_promoted</code> /
      <code>skill_demoted_stale</code> SSB rows recorded yet.
    </div>
  {:else}
    <table class="events-table">
      <thead>
        <tr>
          <th>Time</th>
          <th>Skill</th>
          <th>Event</th>
          <th>Tier</th>
          <th class="num">Inv</th>
          <th class="num">Fail</th>
        </tr>
      </thead>
      <tbody>
        {#each library.events as ev (ev.seq)}
          <tr
            class:row-acquired={ev.kind === 'acquired'}
            class:row-demoted={ev.kind === 'demoted_stale'}
          >
            <td class="mono">{formatTs(ev.ts)}</td>
            <td class="mono">{ev.skill_id}</td>
            <td class="mono">{kindLabel(ev.kind)}</td>
            <td class="mono">
              {#if ev.prior_tier}
                {ev.prior_tier} &rarr; {ev.tier}
              {:else}
                {ev.tier}
              {/if}
            </td>
            <td class="num mono">{ev.invocations}</td>
            <td class="num mono">{(ev.failure_rate * 100).toFixed(0)}%</td>
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
  .skills-panel {
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
    font-size: 11px;
    cursor: pointer;
    border-radius: 2px;
  }
  .refresh-btn:hover {
    background: #2a2a2a;
  }
  .banner {
    background: #1c1a14;
    color: #d0b070;
    font-size: 11px;
    padding: 6px 8px;
    margin-bottom: 8px;
    border-left: 2px solid #d0a040;
  }
  .error-banner {
    color: #d05050;
    font-size: 11px;
    padding: 4px 0;
  }
  .empty {
    color: #888;
    font-style: italic;
    padding: 6px 0;
    font-size: 11px;
  }
  .section-label {
    margin-top: 10px;
    margin-bottom: 4px;
    font-size: 11px;
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
  table.skills-table,
  table.events-table {
    width: 100%;
    border-collapse: collapse;
  }
  table.skills-table th,
  table.skills-table td,
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
  .desc {
    color: #ccc;
    font-size: 11px;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 3px;
  }
  .tag-chip {
    background: #1a1a1a;
    color: #aaa;
    padding: 1px 5px;
    border-radius: 2px;
    font-size: 10px;
  }
  tr.row-fell-through td:first-child {
    color: #e0a050;
  }
  .badge-fell-through {
    background: #3a2a10;
    color: #e0a050;
    padding: 1px 5px;
    border-radius: 2px;
    font-size: 10px;
  }
  .badge-ok {
    background: #1a3a1a;
    color: #80c080;
    padding: 1px 5px;
    border-radius: 2px;
    font-size: 10px;
  }
  .badge-routed {
    background: #1a2a3a;
    color: #80a0c0;
    padding: 1px 5px;
    border-radius: 2px;
    font-size: 10px;
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
    font-size: 11px;
  }
  /* F-6 / A5 — Trust-Ladder styles. */
  .trust-ladder-head {
    margin-top: 16px;
    border-top: 1px solid #2a2a2a;
    padding-top: 10px;
  }
  .ladder {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 6px;
  }
  .ladder-rung {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 4px 6px;
    background: #141414;
    border-left: 2px solid #2a2a2a;
    border-radius: 2px;
  }
  .ladder-rung.rung-apex {
    border-left-color: #80c080;
    background: #131a13;
  }
  .rung-label {
    flex: 0 0 180px;
    color: #aaa;
    font-size: 10px;
  }
  .rung-skills {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .rung-empty {
    color: #555;
  }
  .skill-chip {
    background: #1a2a3a;
    color: #80a0c0;
    padding: 1px 6px;
    border-radius: 2px;
    font-size: 10px;
  }
  tr.row-acquired td:first-child {
    color: #80c080;
  }
  tr.row-demoted td:first-child {
    color: #e0a050;
  }
</style>
