<script lang="ts">
  // E-38 (SCHEMA_VERSION 175) -- 2026 AI roadmap signals + EU AI Act
  // compliance pre-gate dashboard panel. Renders the most recent
  // ai_roadmap_signal_event SSB row per kind:
  //   - compliance (Blocked or Verified, with assertion + hint)
  //   - mcp_transport_version_logged (current vs roadmap target)
  //   - posture_doc_stale (path + age days)
  //   - mrcr_unverified backend counter
  // Self-polling 5s. See workflow/reasoning/E-38-decisions.md.
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  type ComplianceTile = {
    kind: string;
    assertion_label: string;
    hint: string;
    assertions_checked: number;
    plan_session_id: string;
    ts_ms: number;
  };
  type McpTransportTile = {
    mcp_transport_current: string;
    mcp_transport_target: string;
    ts_ms: number;
  };
  type PostureStaleTile = {
    posture_doc_path: string;
    posture_doc_mtime_ms: number;
    age_days: number;
    ts_ms: number;
  };
  type RoadmapSignalsState = {
    last_compliance: ComplianceTile | null;
    mcp_transport: McpTransportTile | null;
    posture_stale: PostureStaleTile | null;
    mrcr_unverified_count: number;
    grand_total: number;
  };

  let panelState: RoadmapSignalsState | null = $state(null);
  let loadError: string | null = $state(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function refresh() {
    try {
      panelState = await invoke<RoadmapSignalsState>("get_roadmap_signals_state", { limit: 200 });
      loadError = null;
    } catch (e) {
      loadError = String(e);
    }
  }

  function complianceLabel(kind: string): string {
    return kind === "compliance_blocked" ? "BLOCK" : kind === "compliance_verified" ? "PASS" : "—";
  }

  function fmtAssertions(mask: number): string {
    const bits: string[] = [];
    if (mask & 0x1) bits.push("clock");
    if (mask & 0x2) bits.push("seed");
    if (mask & 0x4) bits.push("template");
    if (mask & 0x8) bits.push("audit");
    return bits.length ? bits.join(",") : "none";
  }

  onMount(() => {
    refresh();
    pollHandle = setInterval(refresh, 5000);
  });

  onDestroy(() => {
    if (pollHandle) clearInterval(pollHandle);
  });
</script>

<div class="roadmap-panel">
  <header class="rp-header">
    <span class="rp-title">2026 Roadmap Signals</span>
    {#if panelState}<span class="rp-count">rows: {panelState.grand_total}</span>{/if}
  </header>
  {#if loadError}
    <div class="rp-error">load error: {loadError}</div>
  {/if}
  <div class="rp-grid">
    <!-- EU AI Act compliance tile -->
    <div class="rp-tile" class:rp-block={panelState?.last_compliance?.kind === "compliance_blocked"}>
      <div class="rp-tile-label">EU AI Act gate</div>
      {#if panelState?.last_compliance}
        <div class="rp-tile-value">{complianceLabel(panelState.last_compliance.kind)}</div>
        {#if panelState.last_compliance.kind === "compliance_blocked"}
          <div class="rp-tile-stats" title="failing assertion">
            assertion: {panelState.last_compliance.assertion_label}
          </div>
          <div class="rp-tile-stats" title="remediation hint">
            hint: {panelState.last_compliance.hint}
          </div>
        {:else}
          <div class="rp-tile-stats" title="assertions bitmask">
            checked: {fmtAssertions(panelState.last_compliance.assertions_checked)}
          </div>
        {/if}
      {:else}
        <div class="rp-tile-empty">no data</div>
      {/if}
    </div>

    <!-- MCP transport version tile -->
    <div class="rp-tile">
      <div class="rp-tile-label">MCP transport</div>
      {#if panelState?.mcp_transport}
        <div class="rp-tile-value">{panelState.mcp_transport.mcp_transport_current}</div>
        <div class="rp-tile-stats" title="roadmap target">
          target: {panelState.mcp_transport.mcp_transport_target}
        </div>
      {:else}
        <div class="rp-tile-empty">no beacon yet</div>
      {/if}
    </div>

    <!-- Posture doc staleness tile -->
    <div class="rp-tile" class:rp-block={panelState?.posture_stale != null}>
      <div class="rp-tile-label">Posture doc</div>
      {#if panelState?.posture_stale}
        <div class="rp-tile-value">{panelState.posture_stale.age_days}d stale</div>
        <div class="rp-tile-stats" title="posture doc path">
          {panelState.posture_stale.posture_doc_path}
        </div>
      {:else}
        <div class="rp-tile-value">fresh</div>
        <div class="rp-tile-stats">&lt; 90 days or unsignaled</div>
      {/if}
    </div>

    <!-- MRCR-unverified backend counter -->
    <div class="rp-tile" class:rp-block={(panelState?.mrcr_unverified_count ?? 0) > 0}>
      <div class="rp-tile-label">MRCR unverified</div>
      <div class="rp-tile-value">{panelState?.mrcr_unverified_count ?? 0}</div>
      <div class="rp-tile-stats">backends with ≥100K context lacking measured score</div>
    </div>
  </div>
</div>

<style>
  .roadmap-panel { display: flex; flex-direction: column; gap: 0.5rem; padding: 0.75rem; }
  .rp-header { display: flex; align-items: center; gap: 0.5rem; }
  .rp-title { font-weight: 600; }
  .rp-count { font-size: 0.75rem; opacity: 0.7; margin-left: auto; }
  .rp-error { color: #f55; font-size: 0.875rem; }
  .rp-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 0.5rem; }
  .rp-tile {
    padding: 0.5rem;
    border: 1px solid var(--color-border, #444);
    border-radius: 6px;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .rp-tile.rp-block { border-color: #f55; }
  .rp-tile-label { font-weight: 500; font-size: 0.875rem; }
  .rp-tile-value { font-size: 1rem; font-variant-numeric: tabular-nums; }
  .rp-tile-stats { font-size: 0.75rem; opacity: 0.85; word-break: break-all; }
  .rp-tile-empty { font-size: 0.875rem; opacity: 0.5; }
</style>
