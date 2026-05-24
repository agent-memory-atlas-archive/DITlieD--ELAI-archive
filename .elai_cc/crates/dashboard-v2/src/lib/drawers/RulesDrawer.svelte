<script lang="ts">
  import { onMount } from 'svelte';
  import { Badge, Toggle, Scrollbar, UnwiredNote } from '$lib/ui';
  import { rulesData } from '$lib/state/drawers.svelte';
  import { fsm } from '$lib/state/fsm.svelte';
  import type { PredicateType, SourceTierDistribution } from '$lib/api/types';

  onMount(() => {
    rulesData.bootstrap();
    return rulesData.teardown;
  });

  // Session anchor for relative-time display. We compare against Date.now()
  // rather than a fixed mock timestamp so deprecated_patterns rows show
  // their real ages.
  const NOW = Date.now();

  // ------------------------------------------------------------------
  // Section 1 — LIFECYCLE COUNTS
  // ------------------------------------------------------------------
  const counts = $derived(rulesData.lifecycleCounts);

  interface LifecycleTile {
    key: keyof typeof counts;
    label: string;
    color: string;
    bg: string;
    large?: boolean;
  }

  const TILES: LifecycleTile[] = [
    { key: 'PROPOSED',  label: 'PROPOSED',  color: 'var(--text-ghost)',     bg: 'transparent' },
    { key: 'CANDIDATE', label: 'CANDIDATE', color: 'var(--steel)',          bg: 'var(--steel-ghost)' },
    { key: 'SHADOW',    label: 'SHADOW',    color: 'var(--warn)',           bg: 'rgba(195,154,58,0.08)' },
    { key: 'ACTIVE',    label: 'ACTIVE',    color: 'var(--teal-bright)',    bg: 'var(--teal-ghost)',    large: true },
    { key: 'RETIRED',   label: 'RETIRED',   color: 'var(--text-disabled)', bg: 'transparent' },
  ];

  // ------------------------------------------------------------------
  // Section 2 — ACTIVE RULES IN CURRENT FSM STATE
  // ------------------------------------------------------------------
  // FSM state is now read from the live `fsm` store (polls get_fsm_state every
  // 5s); rulesData also re-queries get_active_rules on its own 10s tick.
  const fsmState = $derived(fsm.state);
  const activeRules = $derived(rulesData.activeRules.filter(r => r.active_in_states.includes(fsmState)));

  function predicateTone(p: PredicateType): string {
    const map: Record<PredicateType, string> = {
      AST_PATTERN:       'teal',
      GREP_PATTERN:      'steel',
      SCHEMA_REQUIRED:   'warn',
      METRIC_GATE:       'fail',
      COMPOSITE:         'ghost',
      GRAPH_REACHABILITY:'ghost',
      VERSION_GATE:      'warn',
    };
    return map[p] ?? 'ghost';
  }

  // ------------------------------------------------------------------
  // Section 3 — PREDICATE TYPE DISTRIBUTION
  // ------------------------------------------------------------------
  const PREDICATE_TYPES: PredicateType[] = [
    'AST_PATTERN', 'GREP_PATTERN', 'SCHEMA_REQUIRED', 'METRIC_GATE',
    'COMPOSITE', 'GRAPH_REACHABILITY', 'VERSION_GATE',
  ];

  const PREDICATE_COLORS: Record<PredicateType, string> = {
    AST_PATTERN:        'var(--teal)',
    GREP_PATTERN:       'var(--steel)',
    SCHEMA_REQUIRED:    'var(--warn)',
    METRIC_GATE:        'var(--fail)',
    COMPOSITE:          'var(--text-secondary)',
    GRAPH_REACHABILITY: 'var(--text-ghost)',
    VERSION_GATE:       'rgba(195,154,58,0.7)',
  };

  const allActiveRules = $derived(rulesData.activeRules);
  const predicateCounts = $derived(PREDICATE_TYPES.map(pt => ({
    type: pt,
    count: allActiveRules.filter(r => r.predicate === pt).length,
    color: PREDICATE_COLORS[pt],
  })));
  const predicateTotal = $derived(predicateCounts.reduce((s, x) => s + x.count, 0) || 1);

  // ------------------------------------------------------------------
  // Section 4 — DEPRECATED PATTERNS
  // ------------------------------------------------------------------
  const deprecatedPatterns = $derived(rulesData.deprecatedPatterns.slice(0, 5));

  function relTime(ts: number): string {
    const diff = NOW - ts;
    const d = Math.floor(diff / 86_400_000);
    if (d >= 1) return `${d}d ago`;
    const h = Math.floor(diff / 3_600_000);
    if (h >= 1) return `${h}h ago`;
    return `<1h ago`;
  }

  function langTone(lang: string): string {
    const l = lang.toLowerCase();
    if (l === 'rust') return 'warn';
    if (l === 'python') return 'steel';
    if (l === 'typescript' || l === 'ts') return 'teal';
    return 'ghost';
  }

  // ------------------------------------------------------------------
  // Section 5 — VERSION_GATE STATE
  // Derived from active VERSION_GATE rules. The toggle is local-only
  // (no backend command exists yet for enable/disable; see UnwiredNote).
  // ------------------------------------------------------------------
  const versionGateRules = $derived(allActiveRules.filter(r => r.predicate === 'VERSION_GATE'));
  let versionGateEnabled = $state(true);
  const versionGateCount = $derived(versionGateRules.length);
  const versionGateLastUpdate = NOW - 8 * 3_600_000;

  function handleVersionGateToggle(checked: boolean) {
    versionGateEnabled = checked;
    console.info('[stub] rules.version_gate_toggle', { enabled: checked });
  }

  // ------------------------------------------------------------------
  // Section 6 — SOURCE TIER DISTRIBUTION
  // ------------------------------------------------------------------
  const sourceTiers = $derived<SourceTierDistribution[]>(rulesData.sourceTiers);
  const tierTotal = $derived(sourceTiers.reduce((s, t) => s + t.count, 0) || 1);

  // ------------------------------------------------------------------
  // Section 7 — LAST 5 RULE TRANSITIONS (B-3 #11)
  // ------------------------------------------------------------------
  // Reads the rulesData.lastTransitions $state populated by the 10s
  // poller via stub.get_rule_transitions(5) -> Tauri get_rule_transitions
  // -> SsbViewer.recent_rows_of_type('rule_transition', 5). Empty array
  // until the rules::lifecycle producer emits SsbActionType::RuleTransition
  // rows alongside its existing rule_transitions table writes.
  const lastTransitions = $derived(rulesData.lastTransitions);

  function transitionTone(toStatus: string): string {
    const s = toStatus.toUpperCase();
    if (s === 'ACTIVE') return 'teal';
    if (s === 'SHADOW') return 'warn';
    if (s === 'CANDIDATE') return 'steel';
    if (s === 'RETIRED') return 'ghost';
    if (s === 'PROPOSED') return 'ghost';
    return 'ghost';
  }

  const TIER_COLORS: Record<string, string> = {
    T1: 'var(--teal)',
    T2: 'var(--steel)',
    T3: 'var(--text-secondary)',
    T4: 'var(--warn)',
    T5: 'var(--text-ghost)',
  };
</script>

<div class="rules-drawer">
  <Scrollbar>
    <div class="drawer-body">

      <!-- ── LIFECYCLE COUNTS ─────────────────────────────── -->
      <section class="section">
        <div class="section-heading">LIFECYCLE COUNTS</div>
        <div class="tile-row">
          {#each TILES as tile}
            <div
              class="tile"
              class:tile--large={tile.large}
              style="color:{tile.color}; background:{tile.bg};"
            >
              <span class="tile-label">{tile.label}</span>
              <span class="tile-count" class:tile-count--large={tile.large}>
                {counts[tile.key]}
              </span>
            </div>
          {/each}
        </div>
      </section>

      <!-- ── ACTIVE RULES IN CURRENT FSM STATE ─────────────── -->
      <section class="section">
        <div class="section-heading">
          ACTIVE RULES IN <span style="color:var(--teal);">{fsmState}</span>
        </div>
        {#if activeRules.length === 0}
          <div class="empty-note">No rules bound to this state.</div>
        {:else}
          <div class="rules-list scrollable-rules">
            {#each activeRules as rule}
              <div class="rule-row">
                <span class="rule-name">{rule.name}</span>
                <Badge tone={predicateTone(rule.predicate)} size="xs">
                  {rule.predicate.replace(/_/g, ' ')}
                </Badge>
                <span class="rule-domain">{rule.domain}</span>
              </div>
            {/each}
          </div>
        {/if}
      </section>

      <!-- ── PREDICATE TYPES ───────────────────────────────── -->
      <section class="section">
        <div class="section-heading">PREDICATE TYPES</div>
        <div class="stacked-bar-wrap">
          <div class="stacked-bar">
            {#each predicateCounts as p}
              {#if p.count > 0}
                <div
                  class="stacked-segment"
                  style="width:{(p.count / predicateTotal) * 100}%; background:{p.color};"
                  title="{p.type}: {p.count}"
                ></div>
              {/if}
            {/each}
          </div>
        </div>
        <div class="predicate-legend">
          {#each predicateCounts as p}
            <div class="legend-item">
              <span class="legend-dot" style="background:{p.color};"></span>
              <span class="legend-label">{p.type.replace(/_/g, '_')}</span>
              <span class="legend-count">{p.count}</span>
            </div>
          {/each}
        </div>
      </section>

      <!-- ── DEPRECATED PATTERNS ───────────────────────────── -->
      <section class="section">
        <div class="section-heading">DEPRECATED PATTERNS</div>
        {#each deprecatedPatterns as dp}
          <div class="deprecated-row">
            <div class="deprecated-top">
              <code class="deprecated-pattern">{dp.pattern}</code>
              <Badge tone={langTone(dp.language)} size="xs">{dp.language}</Badge>
            </div>
            {#if dp.replacement}
              <div class="deprecated-reason">use: {dp.replacement}</div>
            {/if}
            <div class="deprecated-ts">{relTime(dp.added_ts)}</div>
          </div>
        {/each}
      </section>

      <!-- ── VERSION_GATE STATE ────────────────────────────── -->
      <section class="section">
        <div class="section-heading">VERSION_GATE STATE</div>
        <div class="vg-row">
          <Toggle
            checked={versionGateEnabled}
            label={versionGateEnabled ? 'ENABLED' : 'DISABLED'}
            onchange={handleVersionGateToggle}
          />
          <span class="vg-meta">
            {versionGateCount} patterns checked
            · last update {relTime(versionGateLastUpdate)}
          </span>
        </div>
      </section>

      <!-- ── SOURCE TIER DISTRIBUTION ─────────────────────── -->
      <section class="section">
        <div class="section-heading">SOURCE TIER DISTRIBUTION</div>
        {#if rulesData.unwired.sourceTiers}
          <UnwiredNote
            phase="A-45"
            feature="T1..T5 source-tier counts of last research run"
            note="get_source_quality_tiers returns zeros until SourceQualityTable is wired into the research pipeline"
          />
        {:else}
          {#each sourceTiers as t}
            <div class="tier-row">
              <span class="tier-badge-custom" style="color:{TIER_COLORS[t.tier] ?? 'var(--text-ghost)'};">
                {t.tier}
              </span>
              <span class="tier-label">{t.label}</span>
              <div class="tier-bar-wrap">
                <div
                  class="tier-bar"
                  style="width:{(t.count / tierTotal) * 100}%; background:{TIER_COLORS[t.tier]};"
                ></div>
              </div>
              <span class="tier-count">{t.count}</span>
            </div>
          {/each}
        {/if}
      </section>

      <!-- ── LAST 5 RULE TRANSITIONS (B-3 #11) ─────────────── -->
      <section class="section section--last">
        <div class="section-heading">LAST 5 RULE TRANSITIONS</div>
        {#if lastTransitions.length === 0}
          <div class="empty-note">No transitions emitted yet (awaiting rules::lifecycle SSB wire-up).</div>
        {:else}
          <div class="rules-list">
            {#each lastTransitions as tr (tr.seq)}
              <div class="rule-row" title={tr.reason}>
                <span class="rule-name">{tr.rule_id}</span>
                <span class="transition-arrow mono">
                  {tr.from_status}&nbsp;&gt;&nbsp;
                </span>
                <Badge tone={transitionTone(tr.to_status)} size="xs">
                  {tr.to_status}
                </Badge>
                <span class="rule-domain">{relTime(tr.timestamp_ms)}</span>
              </div>
            {/each}
          </div>
        {/if}
      </section>

    </div>
  </Scrollbar>
</div>

<style>
  .rules-drawer {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .drawer-body {
    padding: var(--s-4);
    display: flex;
    flex-direction: column;
    gap: var(--s-6);
  }

  .section-heading {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: var(--text-secondary);
    text-transform: uppercase;
    margin-bottom: var(--s-3);
  }

  /* ── LIFECYCLE TILES ──────────────────────── */
  .tile-row {
    display: flex;
    gap: var(--s-2);
  }

  .tile {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-1);
    padding: var(--s-2) var(--s-1);
    border-radius: var(--r-sm);
    border: 1px solid var(--border-subtle);
    min-width: 0;
  }

  .tile--large {
    border-color: var(--teal-line);
  }

  .tile-label {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: inherit;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .tile-count {
    font-family: var(--font-mono);
    font-size: 18px;
    font-weight: 500;
    color: inherit;
    line-height: 1;
  }

  .tile-count--large {
    font-size: 22px;
    color: var(--teal-bright);
  }

  /* ── RULES LIST ───────────────────────────── */
  .rules-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .scrollable-rules {
    max-height: 200px;
    overflow-y: auto;
  }

  .rule-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 4px var(--s-2);
    border-radius: var(--r-xs);
    transition: background-color var(--dur-fast) var(--ease-std);
  }

  .rule-row:hover {
    background: var(--bg-hover);
  }

  .rule-name {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rule-domain {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex-shrink: 0;
  }

  /* ── PREDICATE CHART ──────────────────────── */
  .stacked-bar-wrap {
    margin-bottom: var(--s-2);
  }

  .stacked-bar {
    display: flex;
    height: 8px;
    border-radius: var(--r-xs);
    overflow: hidden;
    background: var(--bg-input);
    gap: 1px;
  }

  .stacked-segment {
    height: 100%;
    transition: width var(--dur-base) var(--ease-out);
  }

  .predicate-legend {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .legend-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .legend-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    flex: 1;
  }

  .legend-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* ── DEPRECATED PATTERNS ──────────────────── */
  .deprecated-row {
    padding: var(--s-2) 0;
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .deprecated-row:last-child {
    border-bottom: none;
  }

  .deprecated-top {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .deprecated-pattern {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-primary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .deprecated-reason {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    padding-left: 0;
  }

  .deprecated-ts {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* ── VERSION GATE ─────────────────────────── */
  .vg-row {
    display: flex;
    align-items: center;
    gap: var(--s-4);
    flex-wrap: wrap;
  }

  .vg-meta {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* ── SOURCE TIERS ─────────────────────────── */
  .tier-badge-custom {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.06em;
    padding: 1px 5px;
    border-radius: var(--r-pill);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .tier-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 4px 0;
  }

  .tier-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .tier-bar-wrap {
    flex: 1;
    height: 4px;
    background: var(--bg-input);
    border-radius: var(--r-xs);
    overflow: hidden;
  }

  .tier-bar {
    height: 100%;
    border-radius: var(--r-xs);
    transition: width var(--dur-base) var(--ease-out);
  }

  .tier-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    width: 20px;
    text-align: right;
    flex-shrink: 0;
  }

  .empty-note {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    padding: var(--s-2) 0;
  }

  /* B-3 #11 — rule_transition row inline arrow. */
  .transition-arrow {
    font-size: 10px;
    color: var(--text-ghost);
    white-space: nowrap;
    flex-shrink: 0;
  }
  .mono {
    font-family: var(--font-mono);
  }

  .section--last {
    padding-bottom: var(--s-6);
  }
</style>
