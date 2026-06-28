<!-- C-47 CaseBaseInspector.svelte: Per-tool case-base stats + recent calibration rows. -->
<!-- Svelte 5 runes. Pairs with SSB tool_call_logged / tool_calibration_hint_emitted / -->
<!-- tool_calibration_drift_detected. Backend: tool_calibration_status + -->
<!-- list_tool_calibration_rows Tauri commands at -->
<!-- (dashboard-v2/src-tauri/src/commands_c47.rs). -->
<!-- Paper grounding: arXiv:2605.15041 (CAST). -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface ToolCalibrationStatusDto {
    tool_name: string;
    shape_count: number;
    total_calls: number;
    error_count: number;
    last_ts_ms: number;
  }

  interface ToolCalibrationRowDto {
    tool_name_hash: string;
    call_shape_hash: string;
    action_type: string;
    ts_ms: number;
    post_call_audit_ok: boolean | null;
    error_rate: number | null;
    total_calls: number | null;
  }

  let statuses = $state<ToolCalibrationStatusDto[]>([]);
  let recentRows = $state<ToolCalibrationRowDto[]>([]);
  let loadError = $state<string | null>(null);
  let lastRefreshTs = $state<number | null>(null);
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  async function refresh(): Promise<void> {
    try {
      const snap = await invoke<ToolCalibrationStatusDto[]>(
        "tool_calibration_status",
        {},
      ).catch((err: unknown) => {
        console.debug("[CaseBaseInspector] tool_calibration_status failed", err);
        return [] as ToolCalibrationStatusDto[];
      });
      statuses = snap;

      const rows = await invoke<ToolCalibrationRowDto[]>(
        "list_tool_calibration_rows",
        { windowMs: 300_000, limit: 200 },
      ).catch((err: unknown) => {
        console.debug(
          "[CaseBaseInspector] list_tool_calibration_rows failed",
          err,
        );
        return [] as ToolCalibrationRowDto[];
      });
      recentRows = rows.slice(0, 200);
      lastRefreshTs = Date.now();
      loadError = null;
    } catch (e: unknown) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  let totalCallsAcrossTools = $derived(
    statuses.reduce((sum, s) => sum + s.total_calls, 0),
  );
  let totalErrorsAcrossTools = $derived(
    statuses.reduce((sum, s) => sum + s.error_count, 0),
  );
  let hintCount = $derived(
    recentRows.filter((r) => r.action_type === "tool_calibration_hint_emitted")
      .length,
  );
  let driftCount = $derived(
    recentRows.filter(
      (r) => r.action_type === "tool_calibration_drift_detected",
    ).length,
  );

  function errorRatePct(s: ToolCalibrationStatusDto): string {
    if (s.total_calls === 0) return "0%";
    return ((s.error_count / s.total_calls) * 100).toFixed(1) + "%";
  }

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

<div class="case-base-inspector">
  <header class="cbi-header">
    <div class="cbi-title">case base (C-47)</div>
    <div class="cbi-subtitle">
      arXiv:2605.15041 CAST — aggregate per-tool calibration telemetry
    </div>
  </header>

  {#if loadError}
    <div class="cbi-error">load error: {loadError}</div>
  {/if}

  <div class="cbi-summary">
    <div class="cbi-stat">
      <span class="cbi-label">tools tracked</span>
      <span class="cbi-value">{statuses.length}</span>
    </div>
    <div class="cbi-stat">
      <span class="cbi-label">total calls</span>
      <span class="cbi-value">{totalCallsAcrossTools}</span>
    </div>
    <div class="cbi-stat">
      <span class="cbi-label">total errors</span>
      <span class="cbi-value">{totalErrorsAcrossTools}</span>
    </div>
    <div class="cbi-stat">
      <span class="cbi-label">hints emitted</span>
      <span class="cbi-value">{hintCount}</span>
    </div>
    <div class="cbi-stat">
      <span class="cbi-label">drift fired</span>
      <span class="cbi-value cbi-drift" class:cbi-drift-on={driftCount > 0}>
        {driftCount}
      </span>
    </div>
  </div>

  {#if statuses.length === 0}
    <div class="cbi-empty">
      Calibration registry unavailable. The orchestrator will attach the live
      ToolCalibrationRegistry handle at runtime (CALIBRATION-PENDING).
    </div>
  {:else}
    <table class="cbi-table">
      <thead>
        <tr>
          <th>tool</th>
          <th>shapes</th>
          <th>calls</th>
          <th>errors</th>
          <th>err rate</th>
          <th>last seen</th>
        </tr>
      </thead>
      <tbody>
        {#each statuses as s (s.tool_name)}
          <tr>
            <td class="cbi-name">{s.tool_name}</td>
            <td>{s.shape_count}</td>
            <td>{s.total_calls}</td>
            <td>{s.error_count}</td>
            <td>{errorRatePct(s)}</td>
            <td>{s.last_ts_ms === 0 ? "-" : new Date(s.last_ts_ms).toISOString()}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  {#if recentRows.length > 0}
    <details class="cbi-recent">
      <summary>recent rows ({recentRows.length})</summary>
      <ul>
        {#each recentRows.slice(0, 20) as r, idx (idx + "-" + r.ts_ms)}
          <li>
            <span class="cbi-tag">{r.action_type}</span>
            <span class="cbi-hash">tool={r.tool_name_hash.slice(0, 8)}</span>
            {#if r.post_call_audit_ok !== null}
              <span>audit_ok={r.post_call_audit_ok}</span>
            {/if}
            {#if r.error_rate !== null}
              <span>err_rate={r.error_rate.toFixed(2)}</span>
            {/if}
            {#if r.total_calls !== null}
              <span>n={r.total_calls}</span>
            {/if}
          </li>
        {/each}
      </ul>
    </details>
  {/if}

  {#if lastRefreshTs}
    <footer class="cbi-footer">
      refreshed {new Date(lastRefreshTs).toLocaleTimeString()}
    </footer>
  {/if}
</div>

<style>
  .case-base-inspector {
    font-family: var(--font-mono, "JetBrains Mono", monospace);
    font-size: 12px;
    color: var(--ink-default, #ddd);
    padding: 12px;
    border: 1px solid var(--rule-subtle, #333);
    border-radius: 6px;
  }
  .cbi-header {
    margin-bottom: 8px;
  }
  .cbi-title {
    font-weight: 600;
    font-size: 13px;
  }
  .cbi-subtitle {
    font-size: 10px;
    opacity: 0.6;
  }
  .cbi-error {
    color: var(--ink-danger, #f55);
    margin-bottom: 6px;
  }
  .cbi-summary {
    display: flex;
    gap: 16px;
    margin-bottom: 10px;
    flex-wrap: wrap;
  }
  .cbi-stat {
    display: flex;
    flex-direction: column;
  }
  .cbi-label {
    font-size: 10px;
    opacity: 0.6;
  }
  .cbi-value {
    font-weight: 600;
    font-size: 13px;
  }
  .cbi-drift-on {
    color: var(--ink-warning, #fa3);
  }
  .cbi-empty {
    padding: 12px;
    background: var(--bg-subtle, #1a1a1a);
    border-radius: 4px;
    opacity: 0.8;
  }
  .cbi-table {
    width: 100%;
    border-collapse: collapse;
  }
  .cbi-table th,
  .cbi-table td {
    text-align: left;
    padding: 4px 8px;
    border-bottom: 1px solid var(--rule-subtle, #2a2a2a);
  }
  .cbi-name {
    font-weight: 500;
  }
  .cbi-recent {
    margin-top: 10px;
  }
  .cbi-recent summary {
    cursor: pointer;
    opacity: 0.7;
  }
  .cbi-recent ul {
    list-style: none;
    padding: 0;
    margin: 6px 0 0;
  }
  .cbi-recent li {
    display: flex;
    gap: 8px;
    font-size: 10px;
    padding: 2px 0;
  }
  .cbi-tag {
    font-weight: 500;
    color: var(--ink-accent, #6ab);
  }
  .cbi-hash {
    opacity: 0.7;
  }
  .cbi-footer {
    margin-top: 8px;
    opacity: 0.5;
    font-size: 10px;
  }
</style>
