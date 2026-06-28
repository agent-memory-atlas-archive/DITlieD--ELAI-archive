<!-- E-37 PromptArmorPanel.svelte: Prompt-injection defense status panel. -->
<!-- Svelte 5 runes. Pairs with the 9 E-37 SSB action types: -->
<!--   injection_attempt_detected / injection_attempt_blocked / -->
<!--   spotlight_wrapped / spotlight_breached / -->
<!--   tool_description_pin_mismatch / capability_gate_rejected / -->
<!--   provenance_tagged / prompt_armor_verdict / -->
<!--   injection_fixture_replayed -->
<!-- Backend: safety_status + list_safety_rows Tauri commands at -->
<!-- (dashboard-v2/src-tauri/src/commands_e37.rs). -->
<!-- Layers additively atop the E-24 mcp_schema_pin + sanitize_result + -->
<!-- llm_guard surface and the E-30 destructive_verb_allowlist surface. -->
<!-- Mounted as the 5th SettingsWindow inspector sibling after E-18's -->
<!-- ToolReliabilityInspector. See workflow/reasoning/E-37-decisions.md. -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface SafetyStatusDto {
    injection_detected_count: number;
    injection_blocked_count: number;
    spotlight_wrapped_count: number;
    spotlight_breached_count: number;
    pin_mismatch_count: number;
    capability_rejected_count: number;
    provenance_tagged_count: number;
    prompt_armor_verdict_count: number;
    fixture_replayed_count: number;
    pin_dir: string;
  }

  interface SafetyRowDto {
    action_type: string;
    ts_ms: number;
  }

  let status = $state<SafetyStatusDto | null>(null);
  let recentRows = $state<SafetyRowDto[]>([]);
  let loadError = $state<string | null>(null);
  let lastRefreshTs = $state<number | null>(null);
  let pollHandle: number | null = null;

  async function refresh() {
    try {
      const s = await invoke<SafetyStatusDto>("safety_status");
      status = s;
      const rows = await invoke<SafetyRowDto[]>("list_safety_rows", {
        limit: 50,
      });
      recentRows = rows;
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

  function totalEvents(s: SafetyStatusDto): number {
    return (
      s.injection_detected_count +
      s.injection_blocked_count +
      s.spotlight_wrapped_count +
      s.spotlight_breached_count +
      s.pin_mismatch_count +
      s.capability_rejected_count +
      s.provenance_tagged_count +
      s.prompt_armor_verdict_count +
      s.fixture_replayed_count
    );
  }
</script>

<div class="prompt-armor-panel">
  {#if loadError}
    <div class="error-banner">PromptArmor snapshot unavailable: {loadError}</div>
  {:else if status === null}
    <div class="empty">Loading prompt-injection defense status...</div>
  {:else if totalEvents(status) === 0}
    <div class="empty">
      No prompt-injection defense events recorded yet. The 9 E-37 SSB
      streams are empty (no ingest classifications fired this session).
    </div>
  {:else}
    <table class="counts">
      <thead>
        <tr>
          <th>Defense layer</th>
          <th>Events</th>
        </tr>
      </thead>
      <tbody>
        <tr><td>Injection detected (Suspect)</td><td>{status.injection_detected_count}</td></tr>
        <tr class="row-block"><td>Injection blocked (Malicious)</td><td>{status.injection_blocked_count}</td></tr>
        <tr><td>Spotlight wrapped</td><td>{status.spotlight_wrapped_count}</td></tr>
        <tr class="row-block"><td>Spotlight breached (forged-wrapper attempt)</td><td>{status.spotlight_breached_count}</td></tr>
        <tr class="row-block"><td>Tool-description pin mismatch</td><td>{status.pin_mismatch_count}</td></tr>
        <tr class="row-block"><td>Capability gate rejected</td><td>{status.capability_rejected_count}</td></tr>
        <tr><td>Provenance tagged (sampled)</td><td>{status.provenance_tagged_count}</td></tr>
        <tr><td>PromptArmor verdicts</td><td>{status.prompt_armor_verdict_count}</td></tr>
        <tr><td>Fixture replays</td><td>{status.fixture_replayed_count}</td></tr>
      </tbody>
    </table>
    <div class="footer">
      Pin store: <code>{status.pin_dir}</code>
      {#if lastRefreshTs}
        · refreshed {new Date(lastRefreshTs).toLocaleTimeString()}
      {/if}
    </div>
  {/if}
</div>

<style>
  .prompt-armor-panel {
    font-size: 12px;
    padding: 8px 0;
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
  }
  table.counts {
    width: 100%;
    border-collapse: collapse;
  }
  table.counts th,
  table.counts td {
    text-align: left;
    padding: 3px 6px;
    border-bottom: 1px solid #2a2a2a;
  }
  tr.row-block td:first-child {
    color: #e0a050;
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
</style>
