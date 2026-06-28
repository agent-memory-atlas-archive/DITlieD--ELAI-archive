<!-- E-18 ToolReliabilityInspector.svelte: Tool-use reliability gate panel. -->
<!-- Svelte 5 runes. Pairs with the 4 SSB action types: -->
<!--   tool_reliability_gate_passed / tool_reliability_gate_rejected / -->
<!--   tool_call_spilled / tool_call_retry_escalated -->
<!-- Backend: tool_reliability_status + list_tool_reliability_rows Tauri -->
<!-- commands at (dashboard-v2/src-tauri/src/commands_e18.rs). -->
<!-- Composes ABOVE C-47 CaseBaseInspector and alongside ToolGateInspector -->
<!-- (C-35) + AuthzPanel (C-42). Composition order: -->
<!--   AuthzPolicy -> ToolGate -> Calibration consult -> Reliability gate. -->
<!-- See workflow/reasoning/E-18-decisions.md. -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface ReliabilityStatusDto {
    inline_cap_bytes: number;
    default_retry_cap: number;
    spill_dir: string;
    pass_count: number;
    reject_count: number;
    spill_count: number;
    escalate_count: number;
    spill_queue_bytes: number;
  }

  interface ReliabilityRowDto {
    action_type: string;
    tool_name_hash: string;
    ts_ms: number;
    reject_reason_hash: string | null;
    spill_path_hash: string | null;
    attempt: number | null;
    escalate_to_role_hash: string | null;
    payload_bytes: number | null;
  }

  let status = $state<ReliabilityStatusDto | null>(null);
  let recentRows = $state<ReliabilityRowDto[]>([]);
  let loadError = $state<string | null>(null);
  let lastRefreshTs = $state<number | null>(null);
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  async function refresh(): Promise<void> {
    try {
      const snap = await invoke<ReliabilityStatusDto>(
        "tool_reliability_status",
        {},
      ).catch((err: unknown) => {
        console.debug(
          "[ToolReliabilityInspector] tool_reliability_status failed",
          err,
        );
        return null as ReliabilityStatusDto | null;
      });
      status = snap;

      const rows = await invoke<ReliabilityRowDto[]>(
        "list_tool_reliability_rows",
        { windowMs: 300_000, limit: 200 },
      ).catch((err: unknown) => {
        console.debug(
          "[ToolReliabilityInspector] list_tool_reliability_rows failed",
          err,
        );
        return [] as ReliabilityRowDto[];
      });
      recentRows = rows.slice(0, 200);
      lastRefreshTs = Date.now();
      loadError = null;
    } catch (e: unknown) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  let passCount = $derived(
    status?.pass_count ??
      recentRows.filter((r) => r.action_type === "tool_reliability_gate_passed").length,
  );
  let rejectCount = $derived(
    status?.reject_count ??
      recentRows.filter((r) => r.action_type === "tool_reliability_gate_rejected").length,
  );
  let spillCount = $derived(
    status?.spill_count ??
      recentRows.filter((r) => r.action_type === "tool_call_spilled").length,
  );
  let escalateCount = $derived(
    status?.escalate_count ??
      recentRows.filter((r) => r.action_type === "tool_call_retry_escalated").length,
  );

  onMount(() => {
    void refresh();
    pollInterval = setInterval(() => {
      void refresh();
    }, 5_000);
  });
  onDestroy(() => {
    if (pollInterval !== null) clearInterval(pollInterval);
  });
</script>

<div class="reliability-inspector">
  <header class="ri-header">
    <div class="ri-title">tool reliability (E-18)</div>
    <div class="ri-subtitle">
      reuse-heavy: D-11 + E-35 + C-47 + C-42 + C-35 composition
    </div>
  </header>

  {#if loadError}
    <div class="ri-error">load error: {loadError}</div>
  {/if}

  <div class="ri-summary">
    <div class="ri-stat">
      <span class="ri-label">passed</span>
      <span class="ri-value">{passCount}</span>
    </div>
    <div class="ri-stat">
      <span class="ri-label">rejected</span>
      <span class="ri-value ri-reject" class:ri-reject-on={rejectCount > 0}>
        {rejectCount}
      </span>
    </div>
    <div class="ri-stat">
      <span class="ri-label">spilled</span>
      <span class="ri-value ri-spill" class:ri-spill-on={spillCount > 0}>
        {spillCount}
      </span>
    </div>
    <div class="ri-stat">
      <span class="ri-label">escalated</span>
      <span class="ri-value ri-escalate" class:ri-escalate-on={escalateCount > 0}>
        {escalateCount}
      </span>
    </div>
  </div>

  {#if status}
    <div class="ri-config">
      <div class="ri-config-row">
        <span class="ri-label">inline cap</span>
        <span class="ri-value">{status.inline_cap_bytes} bytes</span>
      </div>
      <div class="ri-config-row">
        <span class="ri-label">retry cap</span>
        <span class="ri-value">{status.default_retry_cap}</span>
      </div>
      <div class="ri-config-row">
        <span class="ri-label">spill dir</span>
        <span class="ri-value ri-path">{status.spill_dir}</span>
      </div>
    </div>
  {:else}
    <div class="ri-empty">
      Reliability gate snapshot unavailable. The orchestrator will attach
      the live SSB emitter at runtime (RELIABILITY-PENDING).
    </div>
  {/if}

  {#if recentRows.length > 0}
    <details class="ri-recent">
      <summary>recent rows ({recentRows.length})</summary>
      <ul>
        {#each recentRows.slice(0, 20) as r, idx (idx + "-" + r.ts_ms)}
          <li>
            <span class="ri-tag">{r.action_type}</span>
            <span class="ri-hash">tool={r.tool_name_hash.slice(0, 8)}</span>
            {#if r.reject_reason_hash !== null}
              <span>reason={r.reject_reason_hash.slice(0, 8)}</span>
            {/if}
            {#if r.spill_path_hash !== null}
              <span>spill={r.spill_path_hash.slice(0, 8)}</span>
            {/if}
            {#if r.payload_bytes !== null}
              <span>{r.payload_bytes}B</span>
            {/if}
            {#if r.attempt !== null}
              <span>attempt={r.attempt}</span>
            {/if}
            {#if r.escalate_to_role_hash !== null}
              <span>role={r.escalate_to_role_hash.slice(0, 8)}</span>
            {/if}
          </li>
        {/each}
      </ul>
    </details>
  {/if}

  {#if lastRefreshTs}
    <footer class="ri-footer">
      refreshed {new Date(lastRefreshTs).toLocaleTimeString()}
    </footer>
  {/if}
</div>

<style>
  .reliability-inspector {
    font-family: var(--font-mono, "JetBrains Mono", monospace);
    font-size: 12px;
    color: var(--ink-default, #ddd);
    padding: 12px;
    border: 1px solid var(--rule-subtle, #333);
    border-radius: 6px;
  }
  .ri-header {
    margin-bottom: 8px;
  }
  .ri-title {
    font-weight: 600;
    font-size: 13px;
  }
  .ri-subtitle {
    font-size: 10px;
    opacity: 0.6;
  }
  .ri-error {
    color: var(--ink-danger, #f55);
    margin-bottom: 6px;
  }
  .ri-summary {
    display: flex;
    gap: 16px;
    margin-bottom: 10px;
    flex-wrap: wrap;
  }
  .ri-stat {
    display: flex;
    flex-direction: column;
  }
  .ri-label {
    font-size: 10px;
    opacity: 0.6;
  }
  .ri-value {
    font-weight: 600;
    font-size: 13px;
  }
  .ri-reject-on {
    color: var(--ink-warning, #fa3);
  }
  .ri-spill-on {
    color: var(--ink-info, #6cf);
  }
  .ri-escalate-on {
    color: var(--ink-danger, #f55);
  }
  .ri-config {
    margin-bottom: 8px;
  }
  .ri-config-row {
    display: flex;
    gap: 8px;
    align-items: baseline;
    padding: 2px 0;
  }
  .ri-path {
    font-size: 10px;
    opacity: 0.85;
  }
  .ri-empty {
    padding: 12px;
    background: var(--bg-subtle, #1a1a1a);
    border-radius: 4px;
    opacity: 0.8;
  }
  .ri-recent {
    margin-top: 8px;
  }
  .ri-recent summary {
    cursor: pointer;
    opacity: 0.85;
  }
  .ri-recent ul {
    list-style: none;
    padding-left: 0;
    margin: 6px 0 0 0;
    font-size: 10px;
  }
  .ri-recent li {
    display: flex;
    gap: 8px;
    padding: 2px 0;
    flex-wrap: wrap;
  }
  .ri-tag {
    padding: 1px 6px;
    background: var(--bg-tag, #2a2a2a);
    border-radius: 3px;
    font-size: 10px;
  }
  .ri-hash {
    font-family: var(--font-mono, "JetBrains Mono", monospace);
    opacity: 0.7;
  }
  .ri-footer {
    margin-top: 8px;
    font-size: 10px;
    opacity: 0.5;
  }
</style>
