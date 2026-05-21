<!-- C-35 ToolGateInspector.svelte: Shows FSM gate config + 60s token savings histogram. -->
<!-- Svelte 5 runes. Pairs with SSB tool_gated/tool_eager/tool_lazy_resolved/tool_denied. -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  interface ToolGateRow {
    tool_name: string;
    fsm_state: string;
    prompt_tokens_saved: number;
    action_type: string;
    ts_ms: number;
  }

  interface GateSnapshot {
    fsm_state: string;
    eager: string[];
    gated: string[];
    denied: string[];
  }

  let snapshot = $state<GateSnapshot | null>(null);
  let recentRows = $state<ToolGateRow[]>([]);
  let loadError = $state<string | null>(null);
  let lastRefreshTs = $state<number | null>(null);

  const BUCKET_COUNT = 6;
  const BUCKET_MS = 10_000;
  let histogram = $state<number[]>(Array(BUCKET_COUNT).fill(0));
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  async function refresh(): Promise<void> {
    try {
      const r = await fetch("/api/tool-gate/snapshot").catch(() => null);
      if (r && r.ok) snapshot = await r.json() as GateSnapshot;
      const r2 = await fetch("/api/ssb/tool-gate-rows?window_ms=60000").catch(() => null);
      if (r2 && r2.ok) {
        const rows = await r2.json() as ToolGateRow[];
        recentRows = rows.slice(-200);
        const now = Date.now();
        const buckets = Array(BUCKET_COUNT).fill(0);
        for (const row of rows) {
          const bucket = Math.floor((now - row.ts_ms) / BUCKET_MS);
          if (bucket >= 0 && bucket < BUCKET_COUNT) buckets[bucket] += row.prompt_tokens_saved;
        }
        histogram = buckets.reverse();
      }
      lastRefreshTs = Date.now();
      loadError = null;
    } catch (e: unknown) { loadError = e instanceof Error ? e.message : String(e); }
  }

  let totalTokensSaved = $derived(recentRows.reduce((sum, r) => sum + r.prompt_tokens_saved, 0));
  let deniedCount = $derived(recentRows.filter(r => r.action_type === "tool_denied").length);
  let lazyCount = $derived(recentRows.filter(r => r.action_type === "tool_lazy_resolved").length);

  onMount(() => { void refresh(); pollInterval = setInterval(() => { void refresh(); }, 5_000); });
  onDestroy(() => { if (pollInterval !== null) clearInterval(pollInterval); });
</script>

<div class="tool-gate-inspector">
  <header>
    <h2>Tool Gate Inspector (C-35)</h2>
    {#if lastRefreshTs}<span class="ts">refreshed {new Date(lastRefreshTs).toLocaleTimeString()}</span>{/if}
  </header>
  {#if loadError}<p class="error">{loadError}</p>{/if}
  {#if snapshot}
    <section class="gate-state">
      <h3>Active Gate: <code>{snapshot.fsm_state}</code></h3>
      <div class="buckets">
        <div class="bucket eager"><b>Eager ({snapshot.eager.length})</b>
          {#each snapshot.eager as n}<div>{n}</div>{/each}</div>
        <div class="bucket gated"><b>Gated ({snapshot.gated.length})</b>
          {#each snapshot.gated as n}<div>{n}</div>{/each}</div>
        <div class="bucket denied"><b>Denied ({snapshot.denied.length})</b>
          {#each snapshot.denied as n}<div>{n}</div>{/each}</div>
      </div>
    </section>
  {:else}<p class="muted">Gate snapshot unavailable (manifest absent or mcp disabled).</p>
  {/if}
  <section class="metrics">
    <h3>Last 60s Metrics</h3>
    <dl>
      <dt>Tokens saved</dt><dd>{totalTokensSaved}</dd>
      <dt>Denied calls</dt><dd>{deniedCount}</dd>
      <dt>Lazy resolves</dt><dd>{lazyCount}</dd>
    </dl>
  </section>
  <section class="histogram">
    <h3>Token Savings (60s, 10s buckets)</h3>
    <div class="bars">
      {#each histogram as bucket, i}
        <div class="bar-wrap">
          <div class="bar" style="height:{bucket > 0 ? Math.min(100,(bucket/Math.max(...histogram))*100) : 0}%" title="{bucket} tokens"></div>
          <span class="bar-label">{(BUCKET_COUNT - i) * 10 - 10}s</span>
        </div>
      {/each}
    </div>
  </section>
</div>
<style>
  .tool-gate-inspector{font-family:monospace;padding:1rem}
  header{display:flex;align-items:baseline;gap:1rem;margin-bottom:0.5rem}
  h2{margin:0;font-size:1rem}
  .ts{color:grey;font-size:0.75rem}
  .error{color:red}.muted{color:grey}
  .buckets{display:flex;gap:1rem;margin-top:0.5rem}
  .bucket{flex:1;font-size:0.8rem}
  .bucket.eager{color:green}.bucket.gated{color:orange}.bucket.denied{color:red}
  dl{display:grid;grid-template-columns:max-content 1fr;gap:0.25rem 1rem}
  dt{font-weight:bold}
  .bars{display:flex;gap:4px;align-items:flex-end;height:80px}
  .bar-wrap{display:flex;flex-direction:column;align-items:center;width:40px;height:100%}
  .bar{width:100%;background:steelblue;min-height:1px}
  .bar-label{font-size:0.65rem;color:grey;margin-top:2px}
</style>
