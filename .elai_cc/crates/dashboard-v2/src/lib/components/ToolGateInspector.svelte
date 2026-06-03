<!-- C-35 ToolGateInspector.svelte: Shows FSM gate config + 60s token savings histogram. -->
<!-- Svelte 5 runes. Pairs with SSB tool_gated/tool_eager/tool_lazy_resolved/tool_denied. -->
<!-- Backend: get_tool_gate_snapshot + list_tool_gate_rows Tauri commands -->
<!-- (dashboard-v2/src-tauri/src/commands_c35.rs). -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface ToolGateRow {
    tool_name: string;
    fsm_state: string;
    prompt_tokens_saved: number;
    action_type: string;
    ts_ms: number;
    // F-3 / Slice C5 optional fields (absent on legacy gate rows).
    commit_hash?: string;
    allowed_categories?: string[];
    denied_tool?: string;
    taint_source?: string;
  }

  interface GateSnapshot {
    fsm_state: string;
    eager: string[];
    gated: string[];
    denied: string[];
    feature_enabled: boolean;
    note: string;
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
      // C-35: Tauri invoke calls replace the legacy fetch() stubs. The
      // backend modules are registered in dashboard-v2/src-tauri/src/lib.rs
      // alongside commands_empire (Phase 4J / B-25 precedent).
      const snap = await invoke<GateSnapshot>("get_tool_gate_snapshot", {
        fsmState: null,
      }).catch((err: unknown) => {
        console.debug("[ToolGateInspector] get_tool_gate_snapshot failed", err);
        return null;
      });
      if (snap) snapshot = snap;

      const rows = await invoke<ToolGateRow[]>("list_tool_gate_rows", {
        windowMs: 60_000,
        limit: 200,
      }).catch((err: unknown) => {
        console.debug("[ToolGateInspector] list_tool_gate_rows failed", err);
        return [] as ToolGateRow[];
      });
      recentRows = rows.slice(-200);
      const now = Date.now();
      const buckets = Array(BUCKET_COUNT).fill(0);
      for (const row of rows) {
        const bucket = Math.floor((now - row.ts_ms) / BUCKET_MS);
        if (bucket >= 0 && bucket < BUCKET_COUNT) buckets[bucket] += row.prompt_tokens_saved;
      }
      histogram = buckets.reverse();
      lastRefreshTs = Date.now();
      loadError = null;
    } catch (e: unknown) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  let totalTokensSaved = $derived(recentRows.reduce((sum, r) => sum + r.prompt_tokens_saved, 0));
  let deniedCount = $derived(recentRows.filter(r => r.action_type === "tool_denied").length);
  let lazyCount = $derived(recentRows.filter(r => r.action_type === "tool_lazy_resolved").length);

  // F-3 / Slice C5: Tool Filter / Data-plane derived counts.
  let committedCount = $derived(recentRows.filter(r => r.action_type === "tool_filter_committed").length);
  let quarantinedCount = $derived(recentRows.filter(r => r.action_type === "untrusted_var_quarantined").length);

  // ASR (Allow-Set Reduction) tiles: compare legacy tool_denied rows that
  // have NO commit_hash (pre-F-3, no commitment in effect) against
  // untrusted_var_quarantined rows (post-F-3, commitment active).
  // Both counts are computed from the same recentRows window so they are
  // directly comparable as a before/after indicator.
  let asrBefore = $derived(recentRows.filter(r => r.action_type === "tool_denied" && !r.commit_hash).length);
  let asrAfter = $derived(quarantinedCount);

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
      {#if !snapshot.feature_enabled}<p class="muted">{snapshot.note}</p>{/if}
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
  <section class="filter-dataplane">
    <h3>Tool Filter / Data-plane (F-3)</h3>
    <dl>
      <dt>Commitments (60s)</dt><dd>{committedCount}</dd>
      <dt>Quarantined (60s)</dt><dd>{quarantinedCount}</dd>
      <dt>ASR before (no commit)</dt><dd class="asr-before">{asrBefore}</dd>
      <dt>ASR after (quarantined)</dt><dd class="asr-after">{asrAfter}</dd>
    </dl>
    {#if committedCount > 0}
      <div class="committed-list">
        <b>Recent commitments</b>
        {#each recentRows.filter(r => r.action_type === "tool_filter_committed") as row (row.ts_ms)}
          <div class="commit-row">
            <span class="chip hash" title={row.commit_hash}>{(row.commit_hash ?? "").slice(0, 12)}</span>
            <span class="categories">{(row.allowed_categories ?? []).join(", ") || "(none)"}</span>
            <span class="task-id muted">{row.tool_name}</span>
          </div>
        {/each}
      </div>
    {/if}
    {#if quarantinedCount > 0}
      <div class="quarantined-list">
        <b>Quarantined tools</b>
        {#each recentRows.filter(r => r.action_type === "untrusted_var_quarantined") as row (row.ts_ms)}
          <div class="quarantine-row">
            <span class="denied-tool">{row.denied_tool ?? "(unknown)"}</span>
            {#if row.taint_source}<span class="muted">via {row.taint_source}</span>{/if}
          </div>
        {/each}
      </div>
    {/if}
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
  .filter-dataplane{margin-top:1rem}
  .commit-row,.quarantine-row{display:flex;gap:0.5rem;align-items:baseline;font-size:0.8rem;margin-top:0.25rem}
  .chip.hash{background:#1e3a5f;color:#7ecfff;padding:0 0.35rem;border-radius:3px;font-size:0.7rem;font-family:monospace}
  .categories{color:#aaa}
  .task-id{font-size:0.7rem}
  .denied-tool{color:red;font-weight:bold}
  .asr-before{color:orange}
  .asr-after{color:steelblue}
  .bars{display:flex;gap:4px;align-items:flex-end;height:80px}
  .bar-wrap{display:flex;flex-direction:column;align-items:center;width:40px;height:100%}
  .bar{width:100%;background:steelblue;min-height:1px}
  .bar-label{font-size:0.65rem;color:grey;margin-top:2px}
</style>
