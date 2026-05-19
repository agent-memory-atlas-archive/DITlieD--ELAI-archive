<script lang="ts">
  import { onMount } from 'svelte';
  import { Select, SegmentedControl, Scrollbar, UnwiredNote } from '$lib/ui';
  import RuntimePanel from '$lib/components/RuntimePanel.svelte';
  import CloudPanel from '$lib/screens/CloudPanel.svelte';
  import { session } from '$lib/state/session.svelte';
  import { workspace } from '$lib/state/workspace.svelte';
  import * as stub from '$lib/api/stub';
  import type {
    AgentRole,
    SidecarStatus,
    GateStatus,
    McpStatus,
    ProjectProfile,
    DepAuditResult,
    ModelRouting,
    OAuthStatus,
    OAuthLoginCompletePayload,
    CloudModelInfo,
  } from '$lib/api/types';

  // Phase D wiring: replace mock-data direct imports with stub.ts polling.
  // We initialise to safe defaults and overwrite when the backend responds.
  let SIDECAR_STATUS = $state<SidecarStatus>({
    loaded: false, model: 'unavailable', fa2: false, dola: false, steering: false,
    turboquant: 'OFF', tokens_per_sec: 0, latency_p50_ms: 0, latency_p99_ms: 0,
    context_window: 0, mcp_bridge_active: false,
  });
  let GATE_STATUS = $state<GateStatus>({
    mode: 'INTERACTIVE', auto_approve_threshold: 0.85, pending_count: 0, auto_advanced_count: 0,
  });
  let MCP_STATUS = $state<McpStatus>({
    enabled: false, bridge_url: '', connected: false, tools_available: [],
  });
  let MODEL_ROUTING = $state<ModelRouting[]>([]);
  let PROJECT_PROFILE = $state<ProjectProfile>({
    tier1: { language: '', build_tool: '', framework: '', test_runner: '', ci: '', linter: '' },
    tier2: { conventions: [] },
    tier3: { architecture_inference: '', telescope_summaries: [] },
  });
  let DEP_AUDIT = $state<DepAuditResult[]>([]);
  const NOW = Date.now();

  onMount(() => {
    let alive = true;
    async function refresh() {
      try {
        const [sc, gs, mcp, sb, pp, da] = await Promise.all([
          stub.get_sidecar_status(),
          stub.get_gate_status(),
          stub.get_mcp_status(),
          stub.get_local_budget(),
          stub.get_project_intelligence(),
          stub.get_dep_audit_results(),
        ]);
        if (!alive) return;
        SIDECAR_STATUS = sc;
        GATE_STATUS = gs;
        MCP_STATUS = mcp;
        PROJECT_PROFILE = pp;
        DEP_AUDIT = da;
        // local_max_context budget — currently not surfaced into the form.
        void sb;
      } catch (err) {
        console.debug('[SettingsWindow] refresh failed', err);
      }
    }
    void refresh();
    const t = setInterval(refresh, 10_000);
    return () => { alive = false; clearInterval(t); };
  });

  // ------------------------------------------------------------------
  // Section 1: Execution
  // ------------------------------------------------------------------
  const MODEL_OPTIONS = [
    { value: 'Qwen2.5-Coder-7B-Instruct-abliterated', label: 'Qwen2.5-Coder 7B (abliterated)' },
    { value: 'Qwen2.5-Coder-3B-Instruct-abliterated', label: 'Qwen2.5-Coder 3B (abliterated)' },
    { value: 'Qwen2.5-Coder-32B-NF4-FA2-TQ4',          label: 'Qwen2.5-Coder 32B (NF4 FA2 TQ4)' },
    { value: 'claude-haiku-4-5',                       label: 'claude-haiku-4-5' },
    { value: 'claude-opus-4-7',                        label: 'claude-opus-4-7' },
    { value: 'claude-3-5-sonnet-20241022',             label: 'claude-3-5-sonnet' },
    { value: 'ollama:gemma3:4b',                       label: 'ollama:gemma3:4b' },
  ];

  const VRAM_OPTIONS = [
    { value: '8',  label: '8 GB' },
    { value: '12', label: '12 GB' },
    { value: '16', label: '16 GB' },
    { value: '24', label: '24 GB' },
    { value: '48', label: '48 GB' },
  ];

  const EXECUTION_MODE_OPTIONS = [
    { value: 'LOCAL', label: 'LOCAL' },
    { value: 'CLOUD', label: 'CLOUD' },
    { value: 'HYBRID', label: 'HYBRID' },
  ];

  const ROLES: AgentRole[] = [
    'executor', 'auditor', 'researcher', 'designer',
    'prompt-engineer', 'context-compiler', 'memory-writer', 'media-validator',
  ];

  // Build initial routing map. Defaults populate the dropdowns until the
  // backend's get_model_routing fan-out lands; user-driven changes flow
  // through stub.set_model_routing on every $effect tick (real call, Bucket A).
  const initialRouting: Record<AgentRole, string> = {
    executor: 'Qwen2.5-Coder-7B-Instruct-abliterated',
    auditor: 'Qwen2.5-Coder-7B-Instruct-abliterated',
    researcher: 'Qwen2.5-Coder-7B-Instruct-abliterated',
    designer: 'Qwen2.5-Coder-7B-Instruct-abliterated',
    'prompt-engineer': 'Qwen2.5-Coder-3B-Instruct-abliterated',
    'context-compiler': 'Qwen2.5-Coder-3B-Instruct-abliterated',
    'memory-writer': 'Qwen2.5-Coder-3B-Instruct-abliterated',
    'media-validator': 'Qwen2.5-Coder-7B-Instruct-abliterated',
  };
  let modelRouting = $state<Record<AgentRole, string>>({ ...initialRouting });

  // Local mirrors
  let executionMode = $state(session.mode);
  let vramBudget = $state('12');
  let cloudApiKey = $state('');
  let cloudSpendCap = $state<string>('');

  // ----------------------------------------------------------------
  // Cloud Auth (Anthropic OAuth login surface) + Cloud Model picker.
  // OAuth status is fetched on mount, refreshed every 15s, AND
  // refreshed in response to the `oauth-login-complete` Tauri event
  // (emitted by the backend when the callback handler finishes).
  // ----------------------------------------------------------------
  let oauthStatus = $state<OAuthStatus>({
    logged_in: false,
    display_name: null,
    expires_at: null,
    scopes: [],
    expired: false,
  });
  let oauthLoginPending = $state(false);
  let oauthError = $state<string | null>(null);
  let oauthAuthUrl = $state<string | null>(null);

  let cloudModelInfo = $state<CloudModelInfo>({
    model_id: 'claude-sonnet-4-6',
    allowlist: [],
    default: 'claude-sonnet-4-6',
  });
  // Bound to the Select component. The $effect below saves whenever the
  // selection diverges from `cloudModelInfo.model_id` (backend truth);
  // refreshCloudModel() updates both fields atomically so the effect's
  // equality check naturally skips the seed write.
  let cloudModelSelection = $state('claude-sonnet-4-6');
  let cloudModelError = $state<string | null>(null);

  const cloudModelOptions = $derived(
    cloudModelInfo.allowlist.length > 0
      ? cloudModelInfo.allowlist.map((id) => ({ value: id, label: id }))
      : [{ value: cloudModelSelection, label: cloudModelSelection }],
  );

  async function refreshOAuthStatus() {
    try {
      oauthStatus = await stub.get_oauth_status();
    } catch (err) {
      console.debug('[SettingsWindow] get_oauth_status failed', err);
    }
  }

  async function refreshCloudModel() {
    try {
      const fresh = await stub.get_cloud_model();
      // Update selection FIRST, then the truth field. Both happen in the
      // same microtask so the $effect runs once and sees next === prev.
      cloudModelSelection = fresh.model_id;
      cloudModelInfo = fresh;
      console.debug('[SettingsWindow] refreshCloudModel ->', fresh.model_id);
    } catch (err) {
      console.debug('[SettingsWindow] get_cloud_model failed', err);
    }
  }

  async function onLoginClick() {
    if (oauthLoginPending) return;
    const prevPending = oauthLoginPending;
    const prevError = oauthError;
    oauthLoginPending = true;
    oauthError = null;
    oauthAuthUrl = null;
    try {
      const info = await stub.start_oauth_login();
      oauthAuthUrl = info.auth_url;
    } catch (err) {
      oauthLoginPending = prevPending;
      oauthError = err instanceof Error ? err.message : String(err);
      oauthAuthUrl = prevError ? null : oauthAuthUrl;
    }
  }

  async function onLogoutClick() {
    const prevStatus = oauthStatus;
    oauthStatus = { ...oauthStatus, logged_in: false };
    try {
      await stub.do_oauth_logout();
      await refreshOAuthStatus();
    } catch (err) {
      oauthStatus = prevStatus;
      oauthError = err instanceof Error ? err.message : String(err);
    }
  }

  $effect(() => {
    const next = cloudModelSelection;
    const prev = cloudModelInfo.model_id;
    if (next === prev) return;
    console.debug('[SettingsWindow] cloud model save', { from: prev, to: next });
    cloudModelError = null;
    void stub.set_cloud_model(next)
      .then((r) => {
        // Advance backend-truth on success so subsequent re-runs of this
        // effect see next === prev and don't re-save the same value.
        cloudModelInfo = { ...cloudModelInfo, model_id: r.model_id };
        console.debug('[SettingsWindow] cloud model saved', r.model_id);
      })
      .catch((err: unknown) => {
        // Rollback the picker to the last known backend value.
        cloudModelSelection = prev;
        cloudModelError = err instanceof Error ? err.message : String(err);
        console.debug('[SettingsWindow] cloud model save failed', err);
      });
  });

  $effect(() => {
    let unlisten: (() => void) | null = null;
    let intervalHandle: ReturnType<typeof setInterval> | null = null;
    let alive = true;
    void refreshOAuthStatus();
    void refreshCloudModel();
    intervalHandle = setInterval(() => {
      if (alive) void refreshOAuthStatus();
    }, 15_000);
    void (async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        const handle = await listen<OAuthLoginCompletePayload>(
          'oauth-login-complete',
          (event) => {
            oauthLoginPending = false;
            oauthAuthUrl = null;
            const payload = event.payload;
            if (payload && payload.ok) {
              oauthError = null;
              void refreshOAuthStatus();
            } else {
              oauthError = (payload && payload.error) || 'oauth login failed';
            }
          },
        );
        if (alive) {
          unlisten = handle;
        } else {
          handle();
        }
      } catch (err) {
        console.debug('[SettingsWindow] oauth-login-complete listen failed', err);
      }
    })();
    return () => {
      alive = false;
      if (intervalHandle) clearInterval(intervalHandle);
      if (unlisten) {
        try { unlisten(); } catch (_) { /* unlisten failed — drop */ }
      }
    };
  });

  // Init guards
  let _modeInit = true;
  $effect(() => {
    const v = executionMode;
    if (_modeInit) { _modeInit = false; return; }
    // session.setMode now persists to localStorage AND calls set_execution_mode
    // internally, so no duplicate backend call here.
    session.setMode(v as 'LOCAL' | 'CLOUD' | 'HYBRID');
  });

  let _gateInit = true;
  $effect(() => {
    const v = gateModeLocal;
    if (_gateInit) { _gateInit = false; return; }
    session.setGateMode(v as 'INTERACTIVE' | 'AUTONOMOUS');
    stub.set_gate_mode(v as 'INTERACTIVE' | 'AUTONOMOUS').catch(() => {});
  });

  let _tqInit = true;
  $effect(() => {
    const v = tqMode;
    if (_tqInit) { _tqInit = false; return; }
    // TQ_OPTIONS values: 'OFF' -> 0 bits, 'TQ8' -> 8 bits, 'TQ4' -> 4 bits
    // NOTE: sidecar /v1/settings handler must accept turboquant_bits — verify server.py before relying on persistence
    stub.set_sidecar_settings({ turboquant_bits: v === 'TQ4' ? 4 : v === 'TQ8' ? 8 : 0 }).catch(() => {});
  });

  let _fontInit = true;
  $effect(() => {
    const v = fontScale;
    if (_fontInit) { _fontInit = false; return; }
    stub.logStub('settings.fontScale', { scale: v });
    const factor = v === 'S' ? '0.9' : v === 'L' ? '1.1' : '1';
    document.documentElement.style.setProperty('--font-scale', factor);
  });

  let _layoutInit = true;
  $effect(() => {
    const v = panelLayout;
    if (_layoutInit) { _layoutInit = false; return; }
    stub.logStub('settings.panelLayout', { layout: v });
    workspace.applyPanelLayoutOverride(
      v as 'activity-left' | 'activity-right' | 'auto',
      session.gateMode,
    );
  });

  // Track previous routing to detect role-by-role changes
  let _prevRouting: Record<AgentRole, string> = { ...initialRouting };
  $effect(() => {
    // Reactive over modelRouting object identity
    const next = modelRouting;
    for (const role of ROLES) {
      if (next[role] !== _prevRouting[role]) {
        const target = next[role];
        _prevRouting = { ..._prevRouting, [role]: target };
        stub.set_model_routing(role, target).catch(() => {});
      }
    }
  });

  let _vramInit = true;
  $effect(() => {
    const v = vramBudget;
    if (_vramInit) { _vramInit = false; return; }
    // NOTE: sidecar /v1/settings handler must accept vram_budget_gb — verify server.py before relying on persistence
    stub.set_sidecar_settings({ vram_budget_gb: Number(v) }).catch(() => {});
  });

  function onApiKeySubmit() {
    // sidecar /v1/settings handler must accept cloud_api_key — verify before relying on persistence
    stub.set_sidecar_settings({ cloud_api_key: cloudApiKey }).catch(() => {});
  }

  function onSpendCapSubmit() {
    // sidecar /v1/settings handler must accept cloud_spend_cap_usd — verify before relying on persistence
    stub.set_sidecar_settings({ cloud_spend_cap_usd: Number(cloudSpendCap) }).catch(() => {});
  }

  // ------------------------------------------------------------------
  // Section 2: Control
  // ------------------------------------------------------------------
  const GATE_MODE_OPTIONS = [
    { value: 'INTERACTIVE', label: 'INTERACTIVE' },
    { value: 'AUTONOMOUS',  label: 'AUTONOMOUS'  },
  ];

  let gateModeLocal = $state(session.gateMode);
  let autoApproveThreshold = $state<string>('0.85');
  let notificationsEnabled = $state(false);

  // Re-sync the local autoApprove input the first time the backend GATE_STATUS arrives.
  let _autoApproveSeeded = false;
  $effect(() => {
    if (!_autoApproveSeeded && GATE_STATUS.auto_approve_threshold) {
      autoApproveThreshold = String(GATE_STATUS.auto_approve_threshold);
      _autoApproveSeeded = true;
    }
  });

  function onAutoApproveSubmit() {
    // NO_BACKEND: requires set_auto_approve_threshold Tauri command (PART12).
    stub.logStub('settings.autoApprove', { threshold: autoApproveThreshold });
  }

  function toggleNotifications() {
    notificationsEnabled = !notificationsEnabled;
    stub.logStub('settings.notifications', { enabled: notificationsEnabled });
    if (notificationsEnabled && 'Notification' in window) {
      Notification.requestPermission().catch(() => {});
    }
  }

  // ------------------------------------------------------------------
  // Section 3: Project
  // ------------------------------------------------------------------
  let showOverrideEditor = $state(false);
  let overrideFileContent = $state('# Project override file\n# Add per-project CLAUDE.md overrides here.\n');
  let overrideSaveError = $state<string | null>(null);

  function openOverrideEditor() {
    showOverrideEditor = !showOverrideEditor;
    if (showOverrideEditor) {
      stub.fetch_settings_get().then((result) => {
        if (result.toml) overrideFileContent = result.toml;
      }).catch(() => {});
    }
  }

  function saveOverrideFile() {
    overrideSaveError = null;
    stub.fetch_settings_set(overrideFileContent).then(() => {
      showOverrideEditor = false;
    }).catch((err: unknown) => {
      overrideSaveError = err instanceof Error ? err.message : 'Save failed: invalid TOML or backend error';
    });
  }

  // ------------------------------------------------------------------
  // Section 4: Appearance
  // ------------------------------------------------------------------
  const FONT_SCALE_OPTIONS = [
    { value: 'S', label: 'S' },
    { value: 'M', label: 'M' },
    { value: 'L', label: 'L' },
  ];

  const LAYOUT_OPTIONS = [
    { value: 'activity-left',  label: 'Activity Left' },
    { value: 'activity-right', label: 'Activity Right' },
    { value: 'auto',           label: 'Auto by gate' },
  ];

  let fontScale = $state('M');
  let panelLayout = $state('activity-left');

  // ------------------------------------------------------------------
  // Section 5: Inference
  // ------------------------------------------------------------------
  let dolaEnabled = $state(false);
  let etcEnabled = $state(false);
  let steeringEnabled = $state(false);
  let mcpEnabled = $state(false);
  let mcpPingMs = $state<number | undefined>(undefined);
  let mcpError = $state<string | null>(null);

  const TQ_OPTIONS = [
    { value: 'OFF', label: 'OFF' },
    { value: 'TQ8', label: 'TQ8' },
    { value: 'TQ4', label: 'TQ4' },
  ];
  let tqMode = $state<string>('OFF');

  // Re-sync inference toggles from the backend on first poll.
  let _inferenceSeeded = false;
  $effect(() => {
    if (!_inferenceSeeded && SIDECAR_STATUS.loaded) {
      dolaEnabled = SIDECAR_STATUS.dola;
      steeringEnabled = SIDECAR_STATUS.steering;
      mcpEnabled = SIDECAR_STATUS.mcp_bridge_active;
      tqMode = SIDECAR_STATUS.turboquant;
      _inferenceSeeded = true;
    }
  });
  $effect(() => {
    mcpPingMs = MCP_STATUS.ping_ms;
  });

  const sidecarLabel = $derived(
    `${SIDECAR_STATUS.model} · FA2 ${SIDECAR_STATUS.fa2 ? 'ON' : 'OFF'} · DoLa ${dolaEnabled ? 'ON' : 'OFF'} · ETC ${etcEnabled ? 'ON' : 'OFF'} · ${tqMode}`
  );

  function toggleDola() {
    dolaEnabled = !dolaEnabled;
    // NOTE: dola is a boot flag; runtime toggle via /v1/settings may be no-op until sidecar handler is verified
    stub.set_sidecar_settings({ dola: dolaEnabled }).catch(() => {});
  }
  function toggleEtc() {
    etcEnabled = !etcEnabled;
    // NOTE: etc is a boot flag; runtime toggle via /v1/settings may be no-op until sidecar handler is verified
    stub.set_sidecar_settings({ etc: etcEnabled }).catch(() => {});
  }
  function toggleSteering() {
    steeringEnabled = !steeringEnabled;
    // NOTE: steering is a boot flag; runtime toggle via /v1/settings may be no-op until sidecar handler is verified
    stub.set_sidecar_settings({ steering: steeringEnabled }).catch(() => {});
  }
  async function recomputeSteering() {
    await stub.recompute_steering_vector().catch(() => {});
  }
  async function toggleMcp() {
    // WARN1 (PART12): Svelte 5 async onclick handlers that throw produce
    // unhandled rejections — try/catch is the only safe path. Optimistic
    // flip is reverted on failure so the toggle pill matches AppState.
    const previous = mcpEnabled;
    mcpEnabled = !previous;
    mcpError = null;
    try {
      await stub.set_mcp_enabled(mcpEnabled);
      if (mcpEnabled) {
        const status = await stub.get_mcp_status();
        mcpPingMs = status.ping_ms;
      } else {
        mcpPingMs = undefined;
      }
    } catch (err) {
      mcpEnabled = previous;
      mcpPingMs = previous ? mcpPingMs : undefined;
      mcpError = err instanceof Error ? err.message : 'mcp toggle failed';
    }
  }

  // ------------------------------------------------------------------
  // Section 6: Anti-staleness
  // ------------------------------------------------------------------
  let nightlyAuditEnabled = $state(true);
  let decayWindowDays = $state<string>('90');
  let showAuditResults = $state(false);

  const lastAuditTime = new Date(NOW - 8 * 60 * 60 * 1000);
  const lastAuditLabel = `${lastAuditTime.getUTCHours()}:${String(lastAuditTime.getUTCMinutes()).padStart(2, '0')} UTC`;

  function toggleNightlyAudit() {
    nightlyAuditEnabled = !nightlyAuditEnabled;
    // NO_BACKEND: nightly audit cron command pending (PART12).
    stub.logStub('settings.nightlyAudit', { enabled: nightlyAuditEnabled });
  }

  function onDecayWindowSubmit() {
    // NO_BACKEND: decay window TOML key unspecified; fetch_settings_set patch deferred until key confirmed.
    stub.logStub('settings.decayWindow', { days: decayWindowDays });
  }

  function toggleAuditResults() {
    showAuditResults = !showAuditResults;
    // NO_BACKEND: audit results viewer command not defined (OS open integration pending, PART12).
    stub.logStub('settings.viewAuditResults');
  }

  // ------------------------------------------------------------------
  // Helper: tab to commit a plain input value (Enter)
  // ------------------------------------------------------------------
  function onEnter(handler: () => void) {
    return (e: KeyboardEvent) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        handler();
      }
    };
  }
</script>

<Scrollbar>
  <div class="settings">
    <div class="settings__banner">
      <UnwiredNote
        phase="PART11"
        feature="auto-approve threshold, nightly audit toggle, decay window, view-audit-results action"
        note="4 settings rows still UNWIRED: set_auto_approve_threshold, nightly audit cron, decay window key, audit results viewer command. Remaining 11 rows wired in PART11."
      />
    </div>

    <!-- Section: EXECUTION -->
    <section class="s-section">
      <div class="s-header">execution</div>

      <div class="s-row">
        <span class="s-label">Mode</span>
        <div class="s-control">
          <SegmentedControl options={EXECUTION_MODE_OPTIONS} bind:value={executionMode} />
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Cloud login</span>
        <div class="s-control s-control--wide">
          <span class="mono" style="margin-right: 0.75rem;">
            {#if oauthStatus.logged_in}
              [ logged in ]{#if oauthStatus.expires_at} · exp {oauthStatus.expires_at}{/if}
            {:else if oauthStatus.expired}
              [ expired ]
            {:else}
              [ not logged in ]
            {/if}
          </span>
          {#if oauthStatus.logged_in}
            <button
              class="toggle-pill mono"
              onclick={() => { void onLogoutClick(); }}
              disabled={oauthLoginPending}
            >Logout</button>
          {:else}
            <button
              class="toggle-pill mono"
              onclick={() => { void onLoginClick(); }}
              disabled={oauthLoginPending}
            >{oauthLoginPending ? 'Waiting…' : 'Login with Anthropic'}</button>
          {/if}
        </div>
      </div>

      {#if oauthAuthUrl}
        <div class="s-row">
          <span class="s-label"></span>
          <div class="s-control s-control--wide" style="display: flex; flex-direction: column; gap: 0.4rem; align-items: flex-start;">
            <span class="mono" style="font-size: 0.8em; opacity: 0.7;">
              If the browser did not open, copy or click the URL:
            </span>
            <textarea
              class="mono"
              readonly
              rows="2"
              style="width: 100%; font-size: 0.75em; padding: 0.3rem; user-select: all; resize: vertical;"
              onclick={(e: Event) => { (e.currentTarget as HTMLTextAreaElement).select(); }}
              value={oauthAuthUrl}
            ></textarea>
            <div style="display: flex; gap: 0.4rem;">
              <button
                class="toggle-pill mono"
                onclick={() => {
                  const u = oauthAuthUrl;
                  if (!u) return;
                  void navigator.clipboard.writeText(u).catch((err: unknown) => {
                    oauthError = `clipboard: ${err instanceof Error ? err.message : String(err)}`;
                  });
                }}
              >Copy</button>
              <button
                class="toggle-pill mono"
                onclick={() => {
                  const u = oauthAuthUrl;
                  if (!u) return;
                  void stub.open_url_in_browser(u).catch((err: unknown) => {
                    oauthError = err instanceof Error ? err.message : String(err);
                  });
                }}
              >Open in browser</button>
            </div>
          </div>
        </div>
      {/if}

      {#if oauthError}
        <div class="s-row">
          <span class="s-label"></span>
          <div class="s-control s-control--wide">
            <span class="mono" style="font-size: 0.8em; color: var(--danger, #d33);">
              {oauthError}
            </span>
          </div>
        </div>
      {/if}

      <div class="s-row">
        <span class="s-label">Cloud model</span>
        <div class="s-control s-control--wide">
          <Select
            options={cloudModelOptions}
            bind:value={cloudModelSelection}
          />
          <span class="mono" style="margin-left: 0.5rem; font-size: 0.8em; opacity: 0.6;">
            (used when Mode = CLOUD)
          </span>
        </div>
      </div>

      {#if cloudModelError}
        <div class="s-row">
          <span class="s-label"></span>
          <div class="s-control s-control--wide">
            <span class="mono" style="font-size: 0.8em; color: var(--danger, #d33);">
              {cloudModelError}
            </span>
          </div>
        </div>
      {/if}

      {#each ROLES as role}
        <div class="s-row">
          <span class="s-label">{role}</span>
          <div class="s-control s-control--wide">
            <Select
              options={MODEL_OPTIONS}
              bind:value={modelRouting[role]}
            />
          </div>
        </div>
      {/each}

      <div class="s-row">
        <span class="s-label">VRAM budget</span>
        <div class="s-control">
          <Select
            options={VRAM_OPTIONS}
            bind:value={vramBudget}
          />
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Cloud API key</span>
        <div class="s-control s-control--wide">
          <input
            class="bare-input mono"
            type="password"
            placeholder="sk-ant-…"
            bind:value={cloudApiKey}
            onkeydown={onEnter(onApiKeySubmit)}
          />
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Cloud spend cap</span>
        <div class="s-control">
          <input
            class="bare-input bare-input--num mono"
            type="number"
            placeholder="0.00"
            bind:value={cloudSpendCap}
            onkeydown={onEnter(onSpendCapSubmit)}
          />
        </div>
      </div>
    </section>

    <!-- Section: RUNTIME BACKENDS (B-37 Phase 4M)
         11-backend RuntimeAdapter table + per-role backend picker.
         Panel polls list_runtimes every 10s and surfaces stub badges
         for adapters still pending B-37C wiring (Ollama/Vllm) plus the
         seven CLI ports whose binaries may not be on PATH. -->
    <section class="s-section">
      <div class="s-header">runtime backends</div>
      <RuntimePanel />
    </section>

    <!-- Section: CLOUD PANEL (C-5) — live provider routing + spend cap +
         redaction events. Replaces the B-27 read-only mirror with a
         CloudPanel component that polls get_cloud_spend / get_execution_mode
         every 5s and wires the provider selector to set_execution_mode
         (LOCAL | HYBRID | CLOUD). The privacy profile / air-gapped fields
         remain elai_cc.toml-driven and are passed through as props. -->
    <section class="s-section">
      <div class="s-header">cloud</div>
      <CloudPanel
        capDailyUsd={Number.isFinite(Number(cloudSpendCap)) ? Number(cloudSpendCap) : null}
        capMonthlyUsd={null}
        privacyProfile="standard"
        airGapped={false}
      />
    </section>

    <!-- Section: CONTROL -->
    <section class="s-section">
      <div class="s-header">control</div>

      <div class="s-row">
        <span class="s-label">Gate mode</span>
        <div class="s-control">
          <SegmentedControl options={GATE_MODE_OPTIONS} bind:value={gateModeLocal} />
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Auto-approve threshold</span>
        <div class="s-control">
          <input
            class="bare-input bare-input--num mono"
            class:bare-input--disabled={session.gateMode !== 'AUTONOMOUS'}
            type="number"
            placeholder="0.92"
            bind:value={autoApproveThreshold}
            disabled={session.gateMode !== 'AUTONOMOUS'}
            onkeydown={onEnter(onAutoApproveSubmit)}
          />
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Browser notifications</span>
        <div class="s-control">
          <button
            class="toggle-pill mono"
            class:toggle-pill--on={notificationsEnabled}
            onclick={toggleNotifications}
            aria-pressed={notificationsEnabled}
          >{notificationsEnabled ? '[ ON ]' : '[ off ]'}</button>
        </div>
      </div>
    </section>

    <!-- Section: PROJECT -->
    <section class="s-section">
      <div class="s-header">project</div>

      <div class="s-row">
        <span class="s-label">Project path</span>
        <div class="s-control s-control--wide">
          <input
            class="bare-input mono bare-input--disabled"
            type="text"
            value={session.currentProject?.path ?? 'D:\\.ELAI_workflow\\.elai_cc'}
            disabled
          />
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Language</span>
        <div class="s-control">
          <span class="s-readonly s-readonly--accent mono">{PROJECT_PROFILE.tier1.language}</span>
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Build system</span>
        <div class="s-control">
          <span class="s-readonly mono">{PROJECT_PROFILE.tier1.build_tool}</span>
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Test runner</span>
        <div class="s-control">
          <span class="s-readonly mono">{PROJECT_PROFILE.tier1.test_runner}</span>
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Override file</span>
        <div class="s-control">
          <button class="link-btn" onclick={openOverrideEditor}>
            {showOverrideEditor ? 'close editor' : 'edit override'}
          </button>
        </div>
      </div>

      {#if showOverrideEditor}
        <div class="s-override-editor">
          <textarea
            class="bare-textarea mono"
            bind:value={overrideFileContent}
            rows={8}
            spellcheck={false}
          ></textarea>
          <div class="s-override-actions">
            <button class="btn-approve" onclick={saveOverrideFile}>Save</button>
            <button class="btn-outlined" onclick={() => { showOverrideEditor = false; overrideSaveError = null; }}>Cancel</button>
          </div>
          {#if overrideSaveError}
            <span class="s-save-error mono">{overrideSaveError}</span>
          {/if}
        </div>
      {/if}
    </section>

    <!-- Section: APPEARANCE -->
    <section class="s-section">
      <div class="s-header">appearance</div>

      <div class="s-row">
        <span class="s-label">Font scale</span>
        <div class="s-control">
          <SegmentedControl options={FONT_SCALE_OPTIONS} bind:value={fontScale} />
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Panel layout</span>
        <div class="s-control">
          <SegmentedControl options={LAYOUT_OPTIONS} bind:value={panelLayout} />
        </div>
      </div>
    </section>

    <!-- Section: INFERENCE -->
    <section class="s-section">
      <div class="s-header">inference</div>

      <div class="s-row">
        <span class="s-label">Sidecar status</span>
        <div class="s-control s-control--wide">
          <span class="s-readonly s-readonly--mono">{sidecarLabel}</span>
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">DoLa</span>
        <div class="s-control">
          <button class="toggle-pill mono" class:toggle-pill--on={dolaEnabled} onclick={toggleDola} aria-pressed={dolaEnabled}>{dolaEnabled ? '[ ON ]' : '[ off ]'}</button>
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">ETC</span>
        <div class="s-control">
          <button class="toggle-pill mono" class:toggle-pill--on={etcEnabled} onclick={toggleEtc} aria-pressed={etcEnabled}>{etcEnabled ? '[ ON ]' : '[ off ]'}</button>
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Steering vector</span>
        <div class="s-control s-control--inline-gap">
          <button class="toggle-pill mono" class:toggle-pill--on={steeringEnabled} onclick={toggleSteering} aria-pressed={steeringEnabled}>{steeringEnabled ? '[ ON ]' : '[ off ]'}</button>
          <button class="link-btn" onclick={recomputeSteering}>recompute</button>
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">MCP bridge</span>
        <div class="s-control s-control--inline-gap">
          <button class="toggle-pill mono" class:toggle-pill--on={mcpEnabled} onclick={toggleMcp} aria-pressed={mcpEnabled}>{mcpEnabled ? '[ ON ]' : '[ off ]'}</button>
          {#if mcpEnabled && mcpPingMs !== undefined}
            <span class="s-ping mono">{mcpPingMs}ms</span>
          {:else if mcpEnabled}
            <span class="s-ping mono">pinging…</span>
          {/if}
          {#if mcpError}
            <span class="s-error mono" data-testid="mcp-error" role="alert">err: {mcpError}</span>
          {/if}
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">TurboQuant</span>
        <div class="s-control">
          <SegmentedControl options={TQ_OPTIONS} bind:value={tqMode} />
        </div>
      </div>
    </section>

    <!-- Section: ANTI-STALENESS -->
    <section class="s-section">
      <div class="s-header">anti-staleness</div>

      <div class="s-row">
        <span class="s-label">Nightly dep audit</span>
        <div class="s-control s-control--inline-gap">
          <button class="toggle-pill mono" class:toggle-pill--on={nightlyAuditEnabled} onclick={toggleNightlyAudit} aria-pressed={nightlyAuditEnabled}>{nightlyAuditEnabled ? '[ ON ]' : '[ off ]'}</button>
          <span class="s-ping mono">last run {lastAuditLabel}</span>
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Confidence decay window</span>
        <div class="s-control">
          <input
            class="bare-input bare-input--num mono"
            type="number"
            placeholder="90"
            bind:value={decayWindowDays}
            onkeydown={onEnter(onDecayWindowSubmit)}
          />
        </div>
      </div>

      <div class="s-row">
        <span class="s-label">Last audit results</span>
        <div class="s-control">
          <button class="link-btn" onclick={toggleAuditResults}>
            {showAuditResults ? 'close results' : 'view results'}
          </button>
        </div>
      </div>

      {#if showAuditResults}
        <div class="audit-results">
          {#each DEP_AUDIT as dep}
            <div class="audit-row" class:audit-row--outdated={dep.outdated}>
              <span class="audit-crate mono">{dep.crate_or_pkg}</span>
              <span class="audit-ver mono">{dep.current_version} → {dep.latest_version}</span>
              {#if dep.security_advisory}
                <span class="audit-advisory mono">{dep.security_advisory}</span>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <div style="height: var(--s-8);"></div>
  </div>
</Scrollbar>

<style>
  /* ================================================================
     SettingsWindow — orbit aesthetic.
     Vertical section list. Section headers in dim small caps.
     No bordered containers per row. Section spacing via padding only.
     Toggles are [ ON ] / [ off ] outlined-text buttons.
     Number inputs: bare mono with teal-underline focus.
     Dropdowns: outlined Select; no chevron decoration.
  ================================================================ */

  .settings {
    padding: var(--s-5) var(--s-6) 0;
    min-height: 100%;
    background: var(--bg-void);
  }

  .settings__banner {
    margin-bottom: var(--s-4);
  }

  /* Section: padding-only separation, no dividers */
  .s-section {
    display: flex;
    flex-direction: column;
    padding: var(--s-3) 0 var(--s-6);
  }

  /* Section header — dim small caps */
  .s-header {
    font-family: var(--font-mono);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--text-ghost);
    margin-bottom: var(--s-3);
  }

  /* Setting row — flat, no borders, no chrome */
  .s-row {
    display: flex;
    align-items: center;
    min-height: 32px;
    gap: var(--s-3);
    padding: 2px 0;
  }

  .s-label {
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--text-secondary);
    min-width: 180px;
    flex-shrink: 0;
  }

  .s-control {
    display: flex;
    align-items: center;
    min-width: 0;
  }
  .s-control--wide { flex: 1; }
  .s-control--inline-gap { gap: var(--s-3); }

  /* Read-only text */
  .s-readonly {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
  }
  .s-readonly--accent { color: var(--teal); }
  .s-readonly--mono {
    font-family: var(--font-mono);
    font-size: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .s-ping {
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* ---- Bare input: no bg, no border, teal underline focus ---- */
  .bare-input {
    flex: 1;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-primary);
    font-size: 12px;
    padding: 4px 2px;
    height: 28px;
    width: 100%;
    outline: none;
    transition: border-color var(--dur-fast) var(--ease-std);
  }

  .bare-input::placeholder { color: var(--text-ghost); }

  .bare-input:focus { border-bottom-color: var(--teal); }

  /* Hide the number-input spinners */
  .bare-input--num::-webkit-outer-spin-button,
  .bare-input--num::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
  .bare-input--num { -moz-appearance: textfield; appearance: textfield; }

  .bare-input--disabled,
  .bare-input:disabled {
    color: var(--text-ghost);
    border-bottom-color: var(--border-subtle);
    cursor: not-allowed;
  }

  /* ---- Override file textarea: minimal chrome ---- */
  .bare-textarea {
    width: 100%;
    background: rgba(26, 32, 41, 0.6);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 12px;
    padding: var(--s-2) var(--s-3);
    resize: vertical;
    outline: none;
    line-height: 1.5;
    box-sizing: border-box;
    transition: border-color var(--dur-fast) var(--ease-std);
  }
  .bare-textarea:focus { border-color: var(--teal-line); }

  .s-override-editor {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    margin: var(--s-2) 0;
  }
  .s-override-actions {
    display: flex;
    gap: var(--s-2);
  }

  .s-save-error {
    font-size: 10px;
    color: var(--fail);
    margin-top: var(--s-1);
  }

  /* ---- Toggle pill: [ ON ] / [ off ] outlined-text button ---- */
  .toggle-pill {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    background: transparent;
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    padding: 3px 10px;
    cursor: pointer;
    outline: none;
    letter-spacing: 0.04em;
    transition:
      color var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std),
      background var(--dur-fast) var(--ease-std);
  }
  .toggle-pill:hover {
    color: var(--text-secondary);
    border-color: var(--teal-line);
  }
  .toggle-pill:focus-visible { box-shadow: var(--ring); }

  .toggle-pill--on {
    color: var(--teal);
    border-color: var(--teal-line);
    background: var(--teal-ghost);
  }
  .toggle-pill--on:hover {
    color: var(--teal-bright);
    border-color: var(--teal);
  }

  /* ---- Link-style buttons: tertiary actions ---- */
  .link-btn {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--teal);
    background: transparent;
    border: none;
    padding: 2px 0;
    cursor: pointer;
    outline: none;
    transition: color var(--dur-fast) var(--ease-std);
  }
  .link-btn:hover {
    color: var(--teal-bright);
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .link-btn:focus-visible { box-shadow: var(--ring); border-radius: var(--r-xs); }

  /* ---- Approve / outlined buttons (mirrors CommandWindow) ---- */
  .btn-approve {
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 500;
    color: var(--teal);
    background: transparent;
    border: 1px solid var(--teal-line);
    border-radius: var(--r-sm);
    padding: 4px 12px;
    cursor: pointer;
    outline: none;
    transition:
      background var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std);
  }
  .btn-approve:hover {
    background: var(--teal-ghost);
    color: var(--teal-bright);
    border-color: var(--teal);
  }
  .btn-approve:focus-visible { box-shadow: var(--ring); }

  .btn-outlined {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    background: transparent;
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    padding: 4px 12px;
    cursor: pointer;
    outline: none;
    transition:
      background var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std);
  }
  .btn-outlined:hover {
    background: var(--teal-ghost);
    color: var(--text-primary);
    border-color: var(--teal-line);
  }
  .btn-outlined:focus-visible { box-shadow: var(--ring); }

  /* ---- Audit results: ghost-text mono rows, no card chrome ---- */
  .audit-results {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding: var(--s-2) 0;
    margin: var(--s-1) 0;
  }
  .audit-row {
    display: flex;
    gap: var(--s-3);
    align-items: baseline;
    padding: 2px 0;
    font-size: 10px;
    color: var(--text-ghost);
  }
  .audit-row--outdated .audit-crate { color: var(--warn); }
  .audit-crate { min-width: 140px; flex-shrink: 0; color: var(--text-secondary); }
  .audit-ver { color: var(--text-ghost); flex-shrink: 0; }
  .audit-advisory {
    color: var(--fail);
    font-size: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    opacity: 0.85;
  }
</style>
