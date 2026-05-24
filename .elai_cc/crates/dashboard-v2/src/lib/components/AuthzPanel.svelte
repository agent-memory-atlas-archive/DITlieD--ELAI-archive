<!--
  C-42 (session 2) AuthzPanel.svelte: Semantic Gateway L2 RBAC dashboard.
  Mounted INSIDE SettingsWindow at L894 (alongside ToolGateInspector
  precedent) — NOT registered as a new WindowType because the cross-
  cutting RBAC config belongs with the gate inspector + cloud config
  already on this page.

  Backend: gateway_authz_status / gateway_authz_recent_denies /
  gateway_authz_run_fuzz Tauri commands (commands_c42.rs).

  Paper grounding: arXiv:2604.25555 (MEDIUM reliability — single-author
  preprint). The 100% / 84.2% / 5.3x numbers from the paper are NOT
  surfaced here; they are design targets only.
-->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface AuthzRole {
    role: string;
    tools: string[];
    doc_owners: string[];
  }

  interface AuthzStatus {
    policy_file: string;
    default: string;
    roles: AuthzRole[];
    feature_enabled: boolean;
    note: string;
  }

  interface AuthzDenyRow {
    role_hash: number;
    tool_hash: number;
    deny_reason_label: string;
    deny_reason_hash: number;
    ts_ms: number;
  }

  interface AuthzFuzzReport {
    passed: number;
    failed: number;
    failures: string[];
    budget: string;
    full_500k_status: string;
    paper: string;
  }

  let status = $state<AuthzStatus | null>(null);
  let recentDenies = $state<AuthzDenyRow[]>([]);
  let fuzzReport = $state<AuthzFuzzReport | null>(null);
  let loadError = $state<string | null>(null);
  let lastRefreshTs = $state<number | null>(null);
  let fuzzRunning = $state<boolean>(false);
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  async function refresh(): Promise<void> {
    try {
      const s = await invoke<AuthzStatus>("gateway_authz_status").catch(
        (err: unknown) => {
          console.debug("[AuthzPanel] gateway_authz_status failed", err);
          return null;
        }
      );
      if (s) status = s;

      const denies = await invoke<AuthzDenyRow[]>(
        "gateway_authz_recent_denies",
        { windowMs: 60_000, limit: 100 }
      ).catch((err: unknown) => {
        console.debug(
          "[AuthzPanel] gateway_authz_recent_denies failed",
          err
        );
        return [] as AuthzDenyRow[];
      });
      recentDenies = denies.slice(0, 100);
      lastRefreshTs = Date.now();
      loadError = null;
    } catch (e: unknown) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  async function runFuzz(): Promise<void> {
    fuzzRunning = true;
    try {
      fuzzReport = await invoke<AuthzFuzzReport>("gateway_authz_run_fuzz");
    } catch (e: unknown) {
      console.debug("[AuthzPanel] gateway_authz_run_fuzz failed", e);
      fuzzReport = null;
    } finally {
      fuzzRunning = false;
    }
  }

  let authzDenies = $derived(
    recentDenies.filter((r) => r.deny_reason_label === "authz").length
  );
  let toolGateDenies = $derived(
    recentDenies.filter((r) => r.deny_reason_label === "tool_gate").length
  );

  onMount(() => {
    void refresh();
    pollInterval = setInterval(() => {
      void refresh();
    }, 10_000);
  });
  onDestroy(() => {
    if (pollInterval !== null) clearInterval(pollInterval);
  });
</script>

<div class="authz-panel">
  <header class="ap-header">
    <span class="ap-title">Authz (C-42 zero-trust)</span>
    <span class="ap-paper">arXiv:2604.25555 (MEDIUM)</span>
  </header>

  {#if loadError}
    <div class="ap-error">load error: {loadError}</div>
  {/if}

  {#if status}
    <section class="ap-section">
      <div class="ap-row">
        <span class="ap-label">policy file</span>
        <code class="ap-value mono">{status.policy_file}</code>
      </div>
      <div class="ap-row">
        <span class="ap-label">default</span>
        <span class="ap-value">{status.default}</span>
      </div>
      <div class="ap-row">
        <span class="ap-label">feature</span>
        <span class="ap-value">
          {status.feature_enabled ? "mcp on" : "mcp off"}
        </span>
      </div>
      <div class="ap-note">{status.note}</div>
    </section>

    <section class="ap-section">
      <div class="ap-section-title">roles ({status.roles.length})</div>
      {#if status.roles.length === 0}
        <div class="ap-empty">no roles declared — composed gate defaults to DENY-ALL</div>
      {:else}
        <table class="ap-table">
          <thead>
            <tr>
              <th>role</th>
              <th>tools</th>
              <th>doc_owners</th>
            </tr>
          </thead>
          <tbody>
            {#each status.roles as r}
              <tr>
                <td class="mono">{r.role}</td>
                <td class="mono">{r.tools.join(", ")}</td>
                <td class="mono">{r.doc_owners.join(", ") || "-"}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>
  {/if}

  <section class="ap-section">
    <div class="ap-section-title">recent denies (60s window)</div>
    <div class="ap-row">
      <span class="ap-label">authz</span>
      <span class="ap-value">{authzDenies}</span>
    </div>
    <div class="ap-row">
      <span class="ap-label">tool_gate</span>
      <span class="ap-value">{toolGateDenies}</span>
    </div>
    {#if recentDenies.length === 0}
      <div class="ap-empty">no rejections in window</div>
    {/if}
  </section>

  <section class="ap-section">
    <div class="ap-section-title">fuzz invariants</div>
    <div class="ap-row">
      <span class="ap-label">harness</span>
      <span class="ap-value">
        bounded 4 deterministic checks (I-1..I-4); 500K RUNPOD-PENDING
      </span>
    </div>
    <button
      class="ap-button"
      onclick={() => {
        void runFuzz();
      }}
      disabled={fuzzRunning}
    >
      {fuzzRunning ? "running..." : "run fuzz batch"}
    </button>
    {#if fuzzReport}
      <div class="ap-row">
        <span class="ap-label">passed</span>
        <span class="ap-value">{fuzzReport.passed}</span>
      </div>
      <div class="ap-row">
        <span class="ap-label">failed</span>
        <span class="ap-value">{fuzzReport.failed}</span>
      </div>
      {#if fuzzReport.failures.length > 0}
        <ul class="ap-failures">
          {#each fuzzReport.failures as f}
            <li class="mono">{f}</li>
          {/each}
        </ul>
      {/if}
    {/if}
  </section>

  {#if lastRefreshTs !== null}
    <footer class="ap-footer">
      last refresh {new Date(lastRefreshTs).toLocaleTimeString()}
    </footer>
  {/if}
</div>

<style>
  .authz-panel {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.5rem 0;
  }
  .ap-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    font-weight: 500;
  }
  .ap-paper {
    font-size: 0.75rem;
    opacity: 0.6;
  }
  .ap-error {
    color: #c44;
  }
  .ap-section {
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    padding: 0.5rem 0;
  }
  .ap-section-title {
    font-size: 0.85rem;
    opacity: 0.75;
    margin-bottom: 0.25rem;
  }
  .ap-row {
    display: flex;
    justify-content: space-between;
    padding: 2px 0;
    font-size: 0.85rem;
  }
  .ap-label {
    opacity: 0.7;
  }
  .ap-value {
    text-align: right;
  }
  .ap-note {
    font-size: 0.75rem;
    opacity: 0.7;
    margin-top: 0.25rem;
  }
  .ap-empty {
    font-size: 0.8rem;
    opacity: 0.6;
    font-style: italic;
  }
  .ap-table {
    width: 100%;
    font-size: 0.8rem;
    border-collapse: collapse;
  }
  .ap-table th {
    text-align: left;
    opacity: 0.7;
    padding: 2px 4px;
  }
  .ap-table td {
    padding: 2px 4px;
    border-top: 1px solid rgba(255, 255, 255, 0.04);
  }
  .ap-button {
    margin-top: 0.5rem;
    padding: 4px 10px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: inherit;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .ap-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .ap-failures {
    margin: 0.25rem 0 0 1.25rem;
    font-size: 0.8rem;
    color: #c44;
  }
  .ap-footer {
    font-size: 0.7rem;
    opacity: 0.5;
    text-align: right;
  }
  .mono {
    font-family: monospace;
  }
</style>
