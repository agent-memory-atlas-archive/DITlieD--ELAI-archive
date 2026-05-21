<!--
  B-37 RuntimePanel — surfaces the 11-backend RuntimeAdapter layer
  introduced by Phase 4M (multi-runtime backend abstraction).

  cross-lang-pair: dashboard/tauri/api
  Pairs with three Tauri commands in src-tauri/src/commands.rs:
    - list_runtimes()           returns RuntimeInfo[]
    - test_runtime(backend)     returns TestReport
    - set_role_runtime(role, backend)  mutates AppState.runtime_routing

  Patterns followed:
    - GatesPanel.svelte: $state / $effect rune skeleton + 5-10s poll.
    - GatesPanel.svelte:78-82: mock-badge for stub adapters (B-37 stubs
      report Unavailable until B-37C lands real Ollama/Vllm + the CLI
      adapters get their binaries on PATH).
    - ProvidersDrawer.svelte:69-74: capability flag-chip array.

  Svelte 5 syntax only ($state / $derived / $effect / $props). No
  `$:`, no svelte-4 stores, no async-trait-ish wrappers.
-->
<script lang="ts">
  import {
    listRuntimes,
    testRuntime,
    setRoleRuntime,
    runtimeCapabilityProbe,
  } from '$lib/api/stub';
  import {
    RUNTIME_BACKEND_IDS,
    RUNTIME_AGENT_ROLES,
  } from '$lib/api/types';
  import type {
    RuntimeInfo,
    RuntimeBackendId,
    RuntimeHealthStatus,
    TestReport,
    RuntimeAgentRole,
  } from '$lib/api/types';

  // -----------------------------------------------------------------
  // Polling state
  // -----------------------------------------------------------------
  let runtimes = $state<RuntimeInfo[]>([]);
  let loadError = $state<string | null>(null);
  let lastRefreshTs = $state<number | null>(null);

  /** Map backend -> in-flight test promise (used to disable the Test
   *  button + show a spinner). */
  let pendingTests = $state<Record<string, boolean>>({});
  /** Map backend -> most recent TestReport for inline display. */
  let lastTests = $state<Record<string, TestReport>>({});

  /** Pending optimistic role updates. Keyed by RuntimeAgentRole; values are
   *  the BACKEND id the user just selected. Cleared on successful
   *  refresh. */
  let pendingRoleUpdates = $state<Partial<Record<RuntimeAgentRole, RuntimeBackendId>>>({});

  /** Last error per role (set_role_runtime rejection); shown inline. */
  let roleErrors = $state<Partial<Record<RuntimeAgentRole, string>>>({});

  // -----------------------------------------------------------------
  // Polling effect — 10s cadence, mirrors GatesPanel's 5s but slower
  // because runtime probes are expensive (CLI subprocess spawn).
  // -----------------------------------------------------------------
  async function refresh(): Promise<void> {
    try {
      const next = await listRuntimes();
      runtimes = next;
      lastRefreshTs = Date.now();
      loadError = null;
      // Once a backend appears in the new snapshot with its updated
      // role assignment, drop the optimistic placeholder.
      const pending = { ...pendingRoleUpdates };
      for (const role of Object.keys(pending) as RuntimeAgentRole[]) {
        const wanted = pending[role];
        if (!wanted) continue;
        const target = next.find((r) => r.canonical_name === wanted);
        if (target && target.roles.includes(role)) {
          delete pending[role];
        }
      }
      pendingRoleUpdates = pending;
    } catch (err) {
      loadError = err instanceof Error ? err.message : String(err);
    }
  }

  $effect(() => {
    void refresh();
    const id = setInterval(() => {
      void refresh();
    }, 10_000);
    return () => clearInterval(id);
  });

  // -----------------------------------------------------------------
  // Derived state — per-role current backend (with optimistic
  // override), backend dropdown options, etc.
  // -----------------------------------------------------------------

  /** RuntimeAgentRole -> currently routed RuntimeBackendId (or `undefined`
   *  when no backend has the role assigned). */
  const roleToBackend = $derived.by(() => {
    const map: Partial<Record<RuntimeAgentRole, RuntimeBackendId>> = {};
    for (const rt of runtimes) {
      for (const role of rt.roles) {
        map[role] = rt.canonical_name;
      }
    }
    // Optimistic overrides win for instant UI feedback.
    for (const role of Object.keys(pendingRoleUpdates) as RuntimeAgentRole[]) {
      const wanted = pendingRoleUpdates[role];
      if (wanted) map[role] = wanted;
    }
    return map;
  });

  /** Dropdown options for the per-role backend picker. */
  const backendOptions = $derived(
    RUNTIME_BACKEND_IDS.map((id) => ({
      value: id,
      label: id,
    })),
  );

  // -----------------------------------------------------------------
  // Health pill colour (matches GatesPanel's red/yellow/green band).
  // -----------------------------------------------------------------
  function healthColor(status: RuntimeHealthStatus): string {
    if (status === 'ready') return '#1a7';
    if (status === 'degraded') return '#d80';
    return '#d33';
  }

  function healthLabel(status: RuntimeHealthStatus): string {
    if (status === 'ready') return 'ready';
    if (status === 'degraded') return 'degraded';
    return 'unavailable';
  }

  // -----------------------------------------------------------------
  // Test button handler — invokes test_runtime, drops result into
  // lastTests[backend] for inline display.
  // -----------------------------------------------------------------
  async function onTestClick(backend: RuntimeBackendId): Promise<void> {
    if (pendingTests[backend]) return;
    pendingTests = { ...pendingTests, [backend]: true };
    try {
      const report = await testRuntime(backend);
      lastTests = { ...lastTests, [backend]: report };
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      lastTests = {
        ...lastTests,
        [backend]: {
          backend,
          ok: false,
          health: { status: 'unavailable', reason: msg },
          latency_ms: 0,
          error: msg,
        },
      };
    } finally {
      const next = { ...pendingTests };
      delete next[backend];
      pendingTests = next;
    }
  }

  // -----------------------------------------------------------------
  // Role-runtime picker handler — optimistic + revert on failure.
  // -----------------------------------------------------------------
  async function onRoleBackendChange(
    role: RuntimeAgentRole,
    next: RuntimeBackendId,
  ): Promise<void> {
    const previous = roleToBackend[role];
    if (previous === next) return;
    pendingRoleUpdates = { ...pendingRoleUpdates, [role]: next };
    const prevErrors = { ...roleErrors };
    delete prevErrors[role];
    roleErrors = prevErrors;
    try {
      await setRoleRuntime(role, next);
      // Refresh now so the table reflects the new mapping without
      // waiting for the next 10s tick.
      await refresh();
    } catch (err) {
      // Revert optimistic update.
      const pending = { ...pendingRoleUpdates };
      delete pending[role];
      pendingRoleUpdates = pending;
      roleErrors = {
        ...roleErrors,
        [role]: err instanceof Error ? err.message : String(err),
      };
    }
  }

  function onRoleSelectChange(e: Event, role: RuntimeAgentRole): void {
    const target = e.currentTarget as HTMLSelectElement;
    const next = target.value as RuntimeBackendId;
    void onRoleBackendChange(role, next);
  }

  function formatLatency(ms: number | null | undefined): string {
    if (ms == null) return '—';
    if (ms < 1000) return `${Math.round(ms)}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  }

  function formatRoles(roles: RuntimeAgentRole[]): string {
    return roles.length === 0 ? '—' : roles.join(', ');
  }

  // -----------------------------------------------------------------
  // E-51: Per-backend capability probe results (populated on demand).
  // -----------------------------------------------------------------
  /** Per-backend capability probe results. Populated lazily on selection click. */
  let capabilityProbes = $state<Record<string, { available: boolean; reason: string; platform_badge: string }>>({});

  async function probeCapability(backend: string): Promise<void> {
    try {
      const result = await runtimeCapabilityProbe(backend);
      capabilityProbes = { ...capabilityProbes, [backend]: result as { available: boolean; reason: string; platform_badge: string } };
    } catch (err) {
      // Non-fatal — capability probe is best-effort
      capabilityProbes = { ...capabilityProbes, [backend]: { available: false, reason: String(err), platform_badge: 'error' } };
    }
  }
</script>

<section class="runtime-panel" aria-label="Runtime backends">
  <header class="runtime-panel__header">
    <h3>Runtime backends</h3>
    <span class="runtime-panel__tag" title="B-37 Phase 4M">B-37</span>
    {#if lastRefreshTs}
      <span class="runtime-panel__refresh" title="Last refresh">
        {new Date(lastRefreshTs).toLocaleTimeString()}
      </span>
    {/if}
    <button
      class="runtime-panel__refresh-btn"
      type="button"
      aria-label="Refresh runtimes"
      onclick={() => { void refresh(); }}
    >refresh</button>
  </header>

  {#if loadError}
    <p class="runtime-panel__error" role="alert">err: {loadError}</p>
  {/if}

  {#if runtimes.length === 0 && !loadError}
    <p class="runtime-panel__loading">Loading runtimes…</p>
  {:else if runtimes.length > 0}
    <table class="runtime-panel__table">
      <thead>
        <tr>
          <th scope="col">backend</th>
          <th scope="col">roles</th>
          <th scope="col">health</th>
          <th scope="col">stub</th>
          <th scope="col">latency</th>
          <th scope="col"><span class="sr-only">actions</span></th>
        </tr>
      </thead>
      <tbody>
        {#each runtimes as rt (rt.canonical_name)}
          {@const test = lastTests[rt.canonical_name]}
          {@const testing = pendingTests[rt.canonical_name] === true}
          <tr>
            <td class="runtime-panel__backend">
              <span class="mono">{rt.canonical_name}</span>
            </td>
            <td class="runtime-panel__roles">{formatRoles(rt.roles)}</td>
            <td>
              <span
                class="runtime-panel__pill"
                style="background: {healthColor(rt.health.status)};"
                title={rt.health.reason ?? ''}
                aria-label="health {healthLabel(rt.health.status)}"
              >
                {healthLabel(rt.health.status)}
              </span>
            </td>
            <td>
              {#if rt.is_stub}
                <span
                  class="runtime-panel__stub-badge"
                  title="Adapter is a B-37 stub (Unavailable until C/B implementation lands)"
                >stub</span>
              {:else}
                <span class="runtime-panel__dash">—</span>
              {/if}
            </td>
            <td class="runtime-panel__latency">
              {#if test}
                <span class:runtime-panel__latency--fail={!test.ok}>
                  {formatLatency(test.latency_ms)}
                </span>
              {:else}
                {formatLatency(rt.latency_ms)}
              {/if}
            </td>
            <td class="runtime-panel__actions">
              <button
                class="runtime-panel__test-btn"
                type="button"
                aria-label="Test {rt.canonical_name} runtime"
                disabled={testing}
                onclick={() => { void onTestClick(rt.canonical_name); }}
              >
                {#if testing}
                  testing…
                {:else}
                  test
                {/if}
              </button>
              {#if test && !testing}
                <span
                  class="runtime-panel__test-result"
                  class:runtime-panel__test-result--pass={test.ok}
                  class:runtime-panel__test-result--fail={!test.ok}
                  title={test.error ?? ''}
                >
                  {test.ok ? 'pass' : 'fail'}
                </span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    <!-- Role editor: per-RuntimeAgentRole -> RuntimeBackend mapping -->
    <div class="runtime-panel__roles-editor">
      <h4>Role routing</h4>
      <p class="runtime-panel__hint">
        Choose which runtime backend handles each agent role. Changes
        apply immediately; failed assignments revert and show an
        inline error.
      </p>
      <div class="runtime-panel__roles-grid" role="group" aria-label="Per-role runtime assignment">
        {#each RUNTIME_AGENT_ROLES as role (role)}
          {@const current = roleToBackend[role]}
          {@const pending = pendingRoleUpdates[role] != null}
          {@const err = roleErrors[role]}
          <label class="runtime-panel__role-row">
            <span class="runtime-panel__role-label mono">{role}</span>
            <select
              class="runtime-panel__role-select"
              aria-label="Runtime backend for {role}"
              value={current ?? ''}
              onchange={(e) => onRoleSelectChange(e, role)}
            >
              <option value="" disabled>— select backend —</option>
              {#each backendOptions as opt (opt.value)}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
            {#if pending}
              <span class="runtime-panel__role-pending" aria-live="polite">applying…</span>
            {/if}
            {#if err}
              <span class="runtime-panel__role-error" role="alert" title={err}>
                err: {err.length > 40 ? `${err.slice(0, 40)}…` : err}
              </span>
            {/if}
          </label>
        {/each}
      </div>
    </div>

    <!-- E-51: Platform capability badges for Mlx + LlamaCpp -->
    <div class="runtime-panel__capability-section">
      <h4>Edge runtime availability</h4>
      <div class="runtime-panel__capability-grid">
        {#each ['mlx', 'llama_cpp'] as edgeBackend (edgeBackend)}
          {@const probe = capabilityProbes[edgeBackend]}
          <div class="runtime-panel__capability-row">
            <span class="mono">{edgeBackend}</span>
            {#if edgeBackend === 'mlx'}
              <span class="runtime-panel__avail-badge runtime-panel__avail-badge--info">macOS arm64 only</span>
            {:else}
              <span class="runtime-panel__avail-badge runtime-panel__avail-badge--info">CPU / Vulkan / CUDA / Metal / HIP</span>
            {/if}
            <button
              class="runtime-panel__test-btn"
              type="button"
              onclick={() => { void probeCapability(edgeBackend); }}
            >probe</button>
            {#if probe}
              <span
                class="runtime-panel__avail-badge"
                class:runtime-panel__avail-badge--ok={probe.available}
                class:runtime-panel__avail-badge--fail={!probe.available}
                title={probe.reason}
              >{probe.platform_badge}</span>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</section>

<style>
  .runtime-panel {
    border: 1px solid #2a2a2a;
    border-radius: 6px;
    padding: 0.75rem 1rem;
    background: #111;
    color: #eee;
    font-family: ui-sans-serif, system-ui, sans-serif;
    font-size: 0.875rem;
  }
  .runtime-panel__header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  .runtime-panel__header h3 {
    margin: 0;
    font-size: 0.95rem;
    flex: 0 0 auto;
  }
  .runtime-panel__tag {
    font-size: 0.65rem;
    padding: 0.1rem 0.35rem;
    background: #333;
    color: #aac;
    border-radius: 3px;
    letter-spacing: 0.05em;
  }
  .runtime-panel__refresh {
    font-size: 0.7rem;
    color: #888;
    margin-left: auto;
    font-variant-numeric: tabular-nums;
  }
  .runtime-panel__refresh-btn {
    font-size: 0.7rem;
    padding: 0.15rem 0.5rem;
    background: #222;
    color: #eee;
    border: 1px solid #333;
    border-radius: 3px;
    cursor: pointer;
  }
  .runtime-panel__refresh-btn:hover,
  .runtime-panel__refresh-btn:focus-visible {
    background: #2a2a2a;
    outline: none;
    border-color: #555;
  }
  .runtime-panel__error {
    color: #f88;
    margin: 0 0 0.5rem;
  }
  .runtime-panel__loading {
    color: #888;
  }
  .runtime-panel__table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 0.75rem;
  }
  .runtime-panel__table th {
    text-align: left;
    color: #888;
    font-weight: 500;
    font-size: 0.7rem;
    text-transform: uppercase;
    padding: 0.2rem 0.4rem;
  }
  .runtime-panel__table td {
    padding: 0.3rem 0.4rem;
    border-top: 1px solid #222;
    font-size: 0.8rem;
    vertical-align: middle;
  }
  .runtime-panel__backend {
    width: 7rem;
  }
  .runtime-panel__roles {
    color: #ccc;
    font-size: 0.75rem;
  }
  .runtime-panel__pill {
    display: inline-block;
    font-size: 0.7rem;
    font-weight: 600;
    color: #fff;
    padding: 0.15rem 0.5rem;
    border-radius: 3px;
    text-transform: uppercase;
  }
  .runtime-panel__stub-badge {
    font-size: 0.65rem;
    padding: 0.1rem 0.4rem;
    background: #553;
    color: #ffc;
    border-radius: 3px;
    letter-spacing: 0.05em;
  }
  .runtime-panel__dash {
    color: #555;
  }
  .runtime-panel__latency {
    font-variant-numeric: tabular-nums;
    color: #aaa;
    width: 5rem;
  }
  .runtime-panel__latency--fail {
    color: #f88;
  }
  .runtime-panel__actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    white-space: nowrap;
  }
  .runtime-panel__test-btn {
    font-size: 0.7rem;
    padding: 0.15rem 0.55rem;
    background: #1f2a36;
    color: #cfe;
    border: 1px solid #2a3a4a;
    border-radius: 3px;
    cursor: pointer;
  }
  .runtime-panel__test-btn:hover:not([disabled]),
  .runtime-panel__test-btn:focus-visible:not([disabled]) {
    background: #2a3a4a;
    outline: none;
    border-color: #4a6a8a;
  }
  .runtime-panel__test-btn[disabled] {
    opacity: 0.55;
    cursor: progress;
  }
  .runtime-panel__test-result {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 0.05rem 0.35rem;
    border-radius: 3px;
  }
  .runtime-panel__test-result--pass {
    background: #134;
    color: #8df;
  }
  .runtime-panel__test-result--fail {
    background: #411;
    color: #f88;
  }
  .runtime-panel__roles-editor {
    border-top: 1px solid #222;
    padding-top: 0.6rem;
  }
  .runtime-panel__roles-editor h4 {
    margin: 0 0 0.25rem;
    font-size: 0.85rem;
  }
  .runtime-panel__hint {
    color: #888;
    font-size: 0.75rem;
    margin: 0 0 0.5rem;
  }
  .runtime-panel__roles-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
    gap: 0.4rem 0.75rem;
  }
  .runtime-panel__role-row {
    display: grid;
    grid-template-columns: 9rem 1fr auto;
    align-items: center;
    gap: 0.4rem;
  }
  .runtime-panel__role-label {
    font-size: 0.75rem;
    color: #cfd;
  }
  .runtime-panel__role-select {
    font-size: 0.75rem;
    padding: 0.15rem 0.3rem;
    background: #181818;
    color: #eee;
    border: 1px solid #2a2a2a;
    border-radius: 3px;
    cursor: pointer;
    min-width: 0;
  }
  .runtime-panel__role-select:focus-visible {
    outline: 1px solid #5af;
    outline-offset: 0;
    border-color: #5af;
  }
  .runtime-panel__role-pending {
    font-size: 0.7rem;
    color: #888;
    font-style: italic;
    grid-column: 2 / -1;
  }
  .runtime-panel__role-error {
    font-size: 0.7rem;
    color: #f88;
    grid-column: 2 / -1;
  }
  .runtime-panel__capability-section {
    border-top: 1px solid #222;
    padding-top: 0.6rem;
    margin-top: 0.5rem;
  }
  .runtime-panel__capability-section h4 {
    margin: 0 0 0.25rem;
    font-size: 0.85rem;
  }
  .runtime-panel__capability-grid {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .runtime-panel__capability-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8rem;
  }
  .runtime-panel__avail-badge {
    font-size: 0.65rem;
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
    font-weight: 500;
  }
  .runtime-panel__avail-badge--info { background: #1a2a3a; color: #8af; }
  .runtime-panel__avail-badge--ok   { background: #134; color: #8df; }
  .runtime-panel__avail-badge--fail { background: #411; color: #f88; }
  .mono {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
