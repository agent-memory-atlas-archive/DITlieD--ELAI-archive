<!--
  E-15 SandboxBackendPanel — surfaces the FirecrackerHotPool +
  ResearcherNetSetup + AppContainer trio runtime selection in the
  orchestration drawer.

  cross-lang-pair: dashboard/tauri/api (atomic with commands.rs ::
  get_sandbox_status + types.ts SandboxStatus + stub.ts mock).

  Patterns followed:
    - GatesPanel.svelte: $state / $derived / $effect rune skeleton.
    - RuntimePanel.svelte: pill badges + tabular layout.
    - OrchestrationDrawer.svelte sandbox-tier-row: compact summary mode.

  Svelte 5 syntax only ($state / $derived / $effect / $props). No
  `$:`, no svelte-4 stores. NO new dependencies (panel uses only
  api/types + api/stub already wired into the drawer).

  RUNPOD-PENDING: live FirecrackerHotPool counters require an
  Arc<SandboxObservability> on AppState; today restore_p50_ms,
  snapshot_count, uniqueness_regenerations_count render as the static
  values returned by get_sandbox_status (null / 0 / 0).
-->
<script lang="ts">
  import type { SandboxStatus, SandboxBackendKind } from '../api/types';

  // -----------------------------------------------------------------
  // Props
  // -----------------------------------------------------------------
  /** SandboxStatus from the orchestrator (parent owns the poll). */
  interface Props {
    sandboxStatus: SandboxStatus | null;
  }
  let { sandboxStatus }: Props = $props();

  // -----------------------------------------------------------------
  // Derived state — backend label + pill colour
  // -----------------------------------------------------------------
  function backendTone(backend: SandboxBackendKind): string {
    if (backend === 'Firecracker')  return 'teal';
    if (backend === 'Gvisor')       return 'warn';
    if (backend === 'AppContainer') return 'steel';
    return 'ghost';
  }

  function backendLabel(backend: SandboxBackendKind): string {
    if (backend === 'Firecracker')  return 'firecracker';
    if (backend === 'Gvisor')       return 'gvisor';
    if (backend === 'AppContainer') return 'appcontainer';
    return 'none';
  }

  function formatLatency(ms: number | null): string {
    if (ms == null) return '—';
    if (ms < 1000) return `${Math.round(ms)}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  }

  // Last 10 blocked attempts (the orchestrator already truncates, but
  // defensive client-side cap keeps the panel render bounded).
  const blockedSample = $derived(
    sandboxStatus?.researcher_net_blocked_attempts.slice(-10) ?? [],
  );

  const allowedHosts = $derived(sandboxStatus?.researcher_net_allowed_hosts ?? []);
</script>

<section class="sandbox-backend-panel" aria-label="Sandbox backend">
  <header class="sandbox-backend-panel__header">
    <h4>Sandbox backend</h4>
    <span class="sandbox-backend-panel__tag" title="E-15">E-15</span>
  </header>

  {#if !sandboxStatus}
    <p class="sandbox-backend-panel__loading">Loading sandbox status…</p>
  {:else}
    <!-- 1. Backend pill + counters row -->
    <div class="sandbox-backend-panel__summary">
      <span
        class="sandbox-backend-panel__pill"
        data-tone={backendTone(sandboxStatus.backend_selected)}
        aria-label="Sandbox backend {backendLabel(sandboxStatus.backend_selected)}"
      >{backendLabel(sandboxStatus.backend_selected)}</span>
      <span class="sandbox-backend-panel__metric">
        <span class="sandbox-backend-panel__metric-key">snapshots</span>
        <span class="sandbox-backend-panel__metric-val mono">{sandboxStatus.snapshot_count}</span>
      </span>
      <span class="sandbox-backend-panel__metric">
        <span class="sandbox-backend-panel__metric-key">restore p50</span>
        <span class="sandbox-backend-panel__metric-val mono">{formatLatency(sandboxStatus.restore_p50_ms)}</span>
      </span>
      <span class="sandbox-backend-panel__metric">
        <span class="sandbox-backend-panel__metric-key">uniq regens</span>
        <span class="sandbox-backend-panel__metric-val mono">{sandboxStatus.uniqueness_regenerations_count}</span>
      </span>
    </div>

    <!-- 2. Researcher-net allowed hosts -->
    {#if allowedHosts.length > 0}
      <div class="sandbox-backend-panel__hosts">
        <div class="sandbox-backend-panel__row-label">researcher allowlist</div>
        <div class="sandbox-backend-panel__chip-row">
          {#each allowedHosts as host (host)}
            <span class="sandbox-backend-panel__chip mono">{host}</span>
          {/each}
        </div>
      </div>
    {/if}

    <!-- 3. Recent blocked egress sample (last 10) -->
    {#if blockedSample.length > 0}
      <div class="sandbox-backend-panel__blocked">
        <div class="sandbox-backend-panel__row-label">recent blocked egress</div>
        <ul class="sandbox-backend-panel__blocked-list">
          {#each blockedSample as attempt, i (i)}
            <li class="sandbox-backend-panel__blocked-item mono">
              <span class="sandbox-backend-panel__blocked-ip">{attempt.dest_ip}</span>
              <span class="sandbox-backend-panel__blocked-reason">{attempt.reason}</span>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  {/if}
</section>

<style>
  .sandbox-backend-panel {
    border: 1px solid var(--border, #2a2a2a);
    border-radius: 6px;
    padding: 0.6rem 0.85rem;
    background: var(--surface-1, #111);
    color: var(--text-primary, #eee);
    font-family: ui-sans-serif, system-ui, sans-serif;
    font-size: 0.8rem;
  }
  .sandbox-backend-panel__header {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-bottom: 0.5rem;
  }
  .sandbox-backend-panel__header h4 {
    margin: 0;
    font-size: 0.85rem;
  }
  .sandbox-backend-panel__tag {
    font-size: 0.6rem;
    padding: 0.05rem 0.3rem;
    background: var(--surface-3, #333);
    color: var(--text-secondary, #aac);
    border-radius: 3px;
    letter-spacing: 0.05em;
  }
  .sandbox-backend-panel__loading {
    color: var(--text-ghost, #888);
    margin: 0;
  }
  .sandbox-backend-panel__summary {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.45rem 0.85rem;
    margin-bottom: 0.5rem;
  }
  .sandbox-backend-panel__pill {
    font-size: 0.68rem;
    font-weight: 600;
    padding: 0.12rem 0.5rem;
    border-radius: 3px;
    text-transform: uppercase;
    color: #fff;
  }
  .sandbox-backend-panel__pill[data-tone="teal"]  { background: var(--teal, #134); color: #8df; }
  .sandbox-backend-panel__pill[data-tone="warn"]  { background: var(--warn, #553); color: #ffc; }
  .sandbox-backend-panel__pill[data-tone="steel"] { background: var(--steel, #234); color: #bce; }
  .sandbox-backend-panel__pill[data-tone="ghost"] { background: var(--surface-3, #333); color: #aaa; }
  .sandbox-backend-panel__metric {
    display: inline-flex;
    align-items: baseline;
    gap: 0.3rem;
  }
  .sandbox-backend-panel__metric-key {
    color: var(--text-secondary, #888);
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .sandbox-backend-panel__metric-val {
    color: var(--text-primary, #ddd);
    font-variant-numeric: tabular-nums;
  }
  .sandbox-backend-panel__hosts,
  .sandbox-backend-panel__blocked {
    border-top: 1px solid var(--border, #222);
    padding-top: 0.4rem;
    margin-top: 0.35rem;
  }
  .sandbox-backend-panel__row-label {
    color: var(--text-secondary, #888);
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: 0.3rem;
  }
  .sandbox-backend-panel__chip-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }
  .sandbox-backend-panel__chip {
    font-size: 0.68rem;
    padding: 0.08rem 0.42rem;
    background: var(--surface-2, #1a2a3a);
    color: var(--teal-bright, #8af);
    border-radius: 3px;
  }
  .sandbox-backend-panel__blocked-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.18rem;
  }
  .sandbox-backend-panel__blocked-item {
    display: flex;
    gap: 0.5rem;
    align-items: baseline;
    font-size: 0.7rem;
  }
  .sandbox-backend-panel__blocked-ip {
    color: var(--fail, #f88);
    min-width: 7rem;
  }
  .sandbox-backend-panel__blocked-reason {
    color: var(--text-ghost, #888);
  }
  .mono {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }
</style>
