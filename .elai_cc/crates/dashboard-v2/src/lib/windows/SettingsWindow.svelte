<script lang="ts">
  import { onMount } from 'svelte';
  import { Select, SegmentedControl, Scrollbar, UnwiredNote } from '$lib/ui';
  import RuntimePanel from '$lib/components/RuntimePanel.svelte';
  import EmpireBridgePanel from '$lib/components/EmpireBridgePanel.svelte';
  // C-35: dynamic tool gating dashboard panel. Reads the live tool-gate
  // snapshot + 60s histogram of prompt_tokens_saved across the 4 tool_*
  // SSB action types. Backend at commands_c35.rs.
  import ToolGateInspector from '$lib/components/ToolGateInspector.svelte';
  // C-42 (session 2): Semantic Gateway L2 RBAC zero-trust panel. Mounts
  // alongside ToolGateInspector. Backend at commands_c42.rs (3 read-only
  // Tauri commands). Paper grounding: arXiv:2604.25555 (MEDIUM reliability).
  import AuthzPanel from '$lib/components/AuthzPanel.svelte';
  // C-47: Case-based calibration inspector. Mounts alongside
  // ToolGateInspector + AuthzPanel inside SettingsWindow. Backend at
  // commands_c47.rs (2 read-only Tauri commands: tool_calibration_status
  // + list_tool_calibration_rows). Paper grounding: arXiv:2605.15041
  // (CAST — Case-Based Calibration of Adaptive Reasoning and Execution
  // for LLM Tool Use). See workflow/papers/case-based-calibration.md.
  import CaseBaseInspector from '$lib/components/CaseBaseInspector.svelte';
  // E-18: Tool-use reliability inspector. Mounts alongside
  // ToolGateInspector + AuthzPanel + CaseBaseInspector. Backend at
  // commands_e18.rs (2 read-only Tauri commands: tool_reliability_status
  // + list_tool_reliability_rows). Composes ABOVE the C-47 case base
  // (composition order: AuthzPolicy -> ToolGate -> Calibration consult
  // -> Reliability gate -> dispatch). Reuse-heavy posture: builds on
  // D-11 RegistryCheck + E-35 SearchReplaceMatcher / SidecarXGrammarHook
  // + C-47 ToolCalibrationRegistry + C-42 AuthzPolicy + C-35 ToolGate
  // rather than re-implementing those surfaces. See
  // workflow/reasoning/E-18-decisions.md.
  import ToolReliabilityInspector from '$lib/components/ToolReliabilityInspector.svelte';
  // E-37: PromptArmor / prompt-injection defense panel (OWASP LLM Top 10
  // LLM01). Mounts alongside ToolGateInspector + AuthzPanel +
  // CaseBaseInspector + ToolReliabilityInspector as the 5th inspector.
  // Backend at commands_e37.rs (2 read-only Tauri commands: safety_status
  // + list_safety_rows). Layers additively atop the E-24 mcp_schema_pin
  // + sanitize_result + llm_guard surface and the E-30 destructive_verb_
  // allowlist surface. See workflow/reasoning/E-37-decisions.md.
  import PromptArmorPanel from '$lib/components/PromptArmorPanel.svelte';
  // C-41: Application Skills System panel. Sub-phase 5B inspector card.
  // Polls list_application_skills + recent_skill_routing_events
  // (commands_c41.rs) every 5s. Surfaces the 9 canonical application
  // skills under .elai_cc/skills/ + a fall-through routing timeline.
  // Wire-dark unless ELAI_APPLICATION_SKILLS_ENABLE=1 on the
  // orchestrator side. See workflow/reasoning/C-41-decisions.md.
  import SkillsPanel from '$lib/components/SkillsPanel.svelte';
  // E-25 Tranche N: Compaction-Ladder inspector card. Pairs with the
  // task_runner PreCompact producer wire + commands_e25.rs Tauri reader.
  // Polls get_compaction_ladder_state + recent_compaction_events every
  // 5s. Renders the wire-dark banner when ELAI_E25_COMPACTION_LADDER_ENABLE
  // is unset on the orchestrator side. Resolves Tranche C C.5 deferral.
  import CompactionLadderPanel from '$lib/components/CompactionLadderPanel.svelte';
  // C-43: Phase 4H scope-freeze status badge. Minimal widget — reads
  // scope_freeze_status (commands_c43.rs) every 15s and renders a
  // green/red badge plus last-ratified timestamp. ELAI design
  // hypothesis; no paper grounding (see
  // workflow/sessions/s-25120/mistakes.md for the bad-cite analysis
  // of arXiv:2604.21725).
  import FreezeStatusBar from '$lib/widgets/FreezeStatusBar.svelte';
  import CloudPanel from '$lib/screens/CloudPanel.svelte';
  import { session } from '$lib/state/session.svelte';
  import { workspace } from '$lib/state/workspace.svelte';
  // E-33: persistent UX prefs (activity-ribbon visibility + motion override).
  import { uxPrefs, type MotionPref } from '$lib/state/ux_prefs.svelte';
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
    CompiledVariantRow,
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
  // I-36: compiled-variant badge chip state. Empty until a promoted variant
  // has been substituted (store is empty until the offline optimizer runs).
  let COMPILED_VARIANTS = $state<CompiledVariantRow[]>([]);

  onMount(() => {
    let alive = true;
    async function refresh() {
      try {
        const [sc, gs, mcp, sb, pp, da, cv] = await Promise.all([
          stub.get_sidecar_status(),
          stub.get_gate_status(),
          stub.get_mcp_status(),
          stub.get_local_budget(),
          stub.get_project_intelligence(),
          stub.get_dep_audit_results(),
          // I-36: poll compiled-variant badge every 10s with the rest.
          stub.get_compiled_variants(),
        ]);
        if (!alive) return;
        SIDECAR_STATUS = sc;
        GATE_STATUS = gs;
        MCP_STATUS = mcp;
        PROJECT_PROFILE = pp;
        DEP_AUDIT = da;
        COMPILED_VARIANTS = cv;
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

  // E-3 (SCHEMA_VERSION 188) MVP slice (TASK-E03-001): ROLE-shown /
  // MODEL-hidden by default. The user toggles "show model details" to
  // reveal the model dropdown per role. The toggle change emits one
  // SSB row via stub.record_model_visibility (wire-dark unless
  // ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE=1 on the backend). Reverses
  // the cross-check finding "SettingsWindow explicitly exposes MODEL
  // names per role" by making the MODEL hidden the default.
  let showModelDetails = $state<boolean>(false);

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
  // B-26 T2 — Webhook URL persistence
  // ------------------------------------------------------------------
  // Bound to set_webhook_url / get_webhook_url Tauri commands. Persists
  // on blur after http(s):// validation. Inline status text shows
  // 'saved' on success or 'invalid' on validation failure; cleared on
  // input. The Tauri side normalizes-trims and treats "" as a clear.
  let webhookUrl = $state<string>('');
  let webhookStatus = $state<'idle' | 'saved' | 'invalid'>('idle');
  let _webhookSeeded = false;

  onMount(() => {
    stub
      .get_webhook_url()
      .then((res) => {
        if (_webhookSeeded) return;
        webhookUrl = res?.webhook_url ?? '';
        _webhookSeeded = true;
      })
      .catch(() => {
        // sidecar offline — leave empty, user can still type and save once live
      });
  });

  function isValidWebhookUrl(s: string): boolean {
    const trimmed = s.trim();
    if (trimmed === '') return true; // "" = clear, valid
    return /^https?:\/\//i.test(trimmed);
  }

  function onWebhookUrlInput() {
    // Reset status as soon as the user types so stale 'saved'/'invalid'
    // labels do not persist past the next edit.
    webhookStatus = 'idle';
  }

  function onWebhookUrlBlur() {
    const trimmed = webhookUrl.trim();
    if (!isValidWebhookUrl(trimmed)) {
      webhookStatus = 'invalid';
      return;
    }
    stub
      .set_webhook_url(trimmed)
      .then((res) => {
        webhookUrl = res?.webhook_url ?? trimmed;
        webhookStatus = 'saved';
      })
      .catch((err) => {
        console.warn('[settings] set_webhook_url failed', err);
        webhookStatus = 'invalid';
      });
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

  // E-33 tasks 1 + 6: persistent UX preferences for ActivityRibbon
  // visibility + accessibility motion override. Bound through uxPrefs so
  // the change reflects across the dashboard immediately (motion store
  // updates the documentElement [data-motion] attribute, ActivityRibbon
  // collapses to zero height).
  let activityRibbonEnabled = $state(uxPrefs.activityRibbonEnabled);
  let accessibilityMotion = $state<MotionPref>(uxPrefs.motionPref);
  const MOTION_OPTIONS = [
    { value: 'auto', label: 'auto' },
    { value: 'reduced', label: 'reduced' },
    { value: 'full', label: 'full' },
  ];

  let _ribbonInit = true;
  $effect(() => {
    const v = activityRibbonEnabled;
    if (_ribbonInit) {
      _ribbonInit = false;
      return;
    }
    uxPrefs.setActivityRibbonEnabled(v);
  });

  let _motionInit = true;
  $effect(() => {
    const v = accessibilityMotion;
    if (_motionInit) {
      _motionInit = false;
      return;
    }
    uxPrefs.setMotionPref(v as MotionPref);
  });

  function toggleActivityRibbon() {
    activityRibbonEnabled = !activityRibbonEnabled;
  }

  // H-12: canvas-as-default toggle. Bound to workspace.canvasAsDefault via
  // workspace.setCanvasAsDefault(). When enabled, new sessions open with the
  // Canvas window as the primary panel. Persisted under CANVAS_DEFAULT_KEY
  // (separate from STORAGE_KEY — no schema invalidation). This is the UI
  // consumer that closes the R6 producer-without-consumer gap for the
  // setCanvasAsDefault() function.
  let canvasAsDefaultLocal = $state(workspace.canvasAsDefault);

  function toggleCanvasAsDefault() {
    canvasAsDefaultLocal = !canvasAsDefaultLocal;
    workspace.setCanvasAsDefault(canvasAsDefaultLocal);
  }

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

  // B-4 task #14: canary drift panel. Reads the latest CanaryDrift SSB
  // row emitted by the periodic cascade evaluator in cli::live_cli so
  // the operator can see real similarity-vs-reference numbers + a
  // "Recompute" affordance when the cosine has drifted below threshold.
  // Same poll cadence as the rest of this window (10s refresh loop).
  let canaryDriftState = $state<stub.CanaryDriftSnapshot>({
    known: false,
    similarity: 0.0,
    drift_threshold: 0.85,
    passed: false,
    reference_present: false,
    ts: 0,
  });
  async function refreshCanaryDrift() {
    try {
      canaryDriftState = await stub.get_canary_drift();
    } catch (e) {
      // wrapper falls back to known:false; suppress console spam.
    }
  }
  // Trigger one refresh on mount and re-poll alongside the main refresh.
  onMount(() => {
    let alive = true;
    void refreshCanaryDrift();
    const t = setInterval(() => { if (alive) void refreshCanaryDrift(); }, 10_000);
    return () => { alive = false; clearInterval(t); };
  });
  async function recomputeCanary() {
    // Trigger a fresh canary readiness check by submitting a tiny task
    // that runs `/symbol_impact target=__canary__`. The submit_task
    // path bumps the indexing pipeline which re-emits the CanaryDrift
    // row on the next cascade tick.
    try {
      await stub.submit_task('/symbol_impact target=__canary_recompute__');
      await refreshCanaryDrift();
    } catch (e) {
      // Best-effort: surface failure in console; the next periodic
      // poll will reflect any new state.
      console.debug('[SettingsWindow] canary recompute failed', e);
    }
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
            <span class="s-fail-text mono"><span class="s-fail-glyph" aria-hidden="true">✕</span> {oauthError}</span>
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
            <span class="s-fail-text mono"><span class="s-fail-glyph" aria-hidden="true">✕</span> {cloudModelError}</span>
          </div>
        </div>
      {/if}

      <!-- E-3 (SCHEMA_VERSION 188) MVP slice (TASK-E03-001): show-model-
           details toggle. Default OFF -> ROLE visible, MODEL hidden;
           toggle ON reveals the model dropdown for each role. Each flip
           emits one role_shown_model_hidden SSB row (wire-dark unless
           ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE=1). Reverses the cross-
           check finding "SettingsWindow explicitly exposes MODEL names
           per role". -->
      <div class="s-row">
        <span class="s-label">Model details</span>
        <div class="s-control">
          <label class="mono" style="cursor: pointer; user-select: none;">
            <input
              type="checkbox"
              bind:checked={showModelDetails}
              onchange={() => {
                // Emit one SSB row per role reflecting the new visibility.
                // model_hidden = !showModelDetails (toggle ON => model shown
                // => model_hidden=false). session_id is null here; the
                // backend resolves it from AppState if available.
                for (const role of ROLES) {
                  stub
                    .record_model_visibility(role, !showModelDetails, null)
                    .catch(() => {});
                }
              }}
            />
            {showModelDetails ? 'shown' : 'hidden (default)'}
          </label>
        </div>
      </div>

      {#each ROLES as role}
        <div class="s-row">
          <span class="s-label">{role}</span>
          <div class="s-control s-control--wide">
            {#if showModelDetails}
              <Select
                options={MODEL_OPTIONS}
                bind:value={modelRouting[role]}
              />
            {:else}
              <span class="mono" style="opacity: 0.55; font-size: 0.85em;">
                model hidden (toggle "show model details" to reveal)
              </span>
            {/if}
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

    <!-- Section: EMPIRE BRIDGE (B-25 Phase 4J)
         Polls get_empire_status + list_empire_factory_lines every 10s.
         Renders the empty-state notice when the orchestrator is built
         without the `empire` feature OR when the bridge is Disconnected
         (the EMPIRE-PENDING activation state). When connected, shows the
         factory-line table with brain_id + engine_tier + neuron_count +
         last_heartbeat. cross-lang-pair: dashboard/tauri/api. -->
    <section class="s-section">
      <div class="s-header">empire bridge</div>
      <EmpireBridgePanel />
    </section>

    <!-- Section: TOOL GATE INSPECTOR (C-35)
         Polls get_tool_gate_snapshot + list_tool_gate_rows every 5s.
         Renders the current FSM state's gate (eager / gated / denied
         tool names) plus a rolling 60s histogram of prompt_tokens_saved
         deltas across the 4 tool_* SSB action types. When the gate
         manifest is absent OR the mcp feature is off, renders the
         "Gate snapshot unavailable" empty state. cross-lang-pair:
         dashboard/tauri/api -->
    <section class="s-section">
      <div class="s-header">tool gate (C-35)</div>
      <ToolGateInspector />
    </section>

    <!-- Section: AUTHZ PANEL (C-42 session 2, semantic-gateway L2 RBAC)
         Mounts alongside ToolGateInspector. Polls gateway_authz_status
         + gateway_authz_recent_denies every 10s. The composed gate runs
         AuthzPolicy BEFORE the C-35 ToolGate; AuthzPolicy denials short-
         circuit (gate is never consulted). Reads authz-policy.toml at
         the workspace root. Paper grounding: arXiv:2604.25555 (MEDIUM
         reliability). L1 DeBERTa-v3 RUNPOD-PENDING; L3 Crypto-HITL
         DEFERRED-TO-FOLLOWUP. cross-lang-pair: dashboard/tauri/api -->
    <section class="s-section">
      <div class="s-header">authz (C-42 zero-trust)</div>
      <AuthzPanel />
    </section>

    <!-- Section: CASE BASE INSPECTOR (C-47 case-based calibration)
         Mounts alongside ToolGateInspector + AuthzPanel. Polls
         tool_calibration_status + list_tool_calibration_rows every 5s.
         Per-tool aggregate signal pattern (CAST Section 3.2 — NOT
         per-query nearest-neighbor retrieval). Renders shape_count,
         total_calls, error_count, last_ts_ms per tool, plus a recent
         SSB row tail showing tool_call_logged /
         tool_calibration_hint_emitted / tool_calibration_drift_detected
         events. Paper grounding: arXiv:2605.15041 (CAST). When the
         orchestrator has not wired a live ToolCalibrationRegistry yet,
         renders "Calibration registry unavailable" (CALIBRATION-
         PENDING). cross-lang-pair: dashboard/tauri/api -->
    <section class="s-section">
      <div class="s-header">case base (C-47 calibration)</div>
      <CaseBaseInspector />
    </section>

    <!-- Section: TOOL RELIABILITY INSPECTOR (E-18 tool-use reliability gate)
         Mounts alongside ToolGateInspector + AuthzPanel +
         CaseBaseInspector. Polls tool_reliability_status +
         list_tool_reliability_rows every 5s. Surfaces the 4 SSB action
         types (tool_reliability_gate_passed /
         tool_reliability_gate_rejected / tool_call_spilled /
         tool_call_retry_escalated) plus the configured inline cap
         (8 KiB) + default retry cap (1) + spill dir
         (.elai_cc/.elai_cc/spill). Reuse-heavy posture: composes on
         D-11 + E-35 + C-47 + C-42 + C-35 rather than re-implementing.
         When the orchestrator has not wired a live SSB emitter yet,
         renders "Reliability gate snapshot unavailable" (RELIABILITY-
         PENDING). cross-lang-pair: dashboard/tauri/api -->
    <section class="s-section">
      <div class="s-header">tool reliability (E-18)</div>
      <ToolReliabilityInspector />
    </section>

    <!-- Section: PROMPT-INJECTION DEFENSE (E-37, OWASP LLM Top 10 LLM01)
         5th inspector — mounts alongside ToolGateInspector + AuthzPanel
         + CaseBaseInspector + ToolReliabilityInspector. Polls
         safety_status + list_safety_rows (commands_e37.rs) every 5s.
         Surfaces the 9 E-37 SSB action types and the per-tool blake3
         description pin store. Layers additively atop the E-24
         mcp_schema_pin + sanitize_result + llm_guard surface and the
         E-30 destructive_verb_allowlist surface. Paper grounding:
         arXiv:2403.14720 Spotlight + arXiv:2507.15219 PromptArmor +
         arXiv:2503.18813 CaMeL + arXiv:2406.13352 AgentDojo +
         arXiv:2502.06703 cheap monitors. RUNPOD-PENDING: live
         adversarial fixture replay against deployed models (AgentDojo /
         EchoLeak / CometJacking). When the orchestrator has not wired a
         live SSB emitter yet, renders "no prompt-injection defense
         events recorded yet" (SAFETY-PENDING). cross-lang-pair:
         dashboard/tauri/api -->
    <section class="s-section">
      <div class="s-header">prompt-injection defense (E-37)</div>
      <PromptArmorPanel />
    </section>

    <!-- Section: APPLICATION SKILLS (C-41, sub-phase 5B)
         6th inspector card. Polls list_application_skills +
         recent_skill_routing_events (commands_c41.rs) every 5s.
         Surfaces 9 SKILL.md scaffolds under .elai_cc/skills/ +
         pre-classify routing telemetry (skill_triggered /
         skill_executed SSB rows). Wire-dark by default — the
         pre-classify hook is gated by ELAI_APPLICATION_SKILLS_ENABLE
         (DECISION-4 in C-41-decisions.md). Paper grounding:
         44% -> 83.3% full-match intent accuracy (Section 5.1 Table 2,
         workflow/papers/scientific-workflow-interface.md). -->
    <section class="s-section">
      <div class="s-header">application skills (C-41)</div>
      <SkillsPanel />
    </section>

    <!-- Section: COMPACTION LADDER (E-25, Tranche N producer wire)
         7th inspector card. Polls get_compaction_ladder_state +
         recent_compaction_events (commands_e25.rs) every 5s. Producer
         is the orchestrator's task_runner PreCompact branch, gated by
         ELAI_E25_COMPACTION_LADDER_ENABLE (DECISION D-N1.a in
         workflow/sessions/s-22600/context-package-trancheN.md).
         Resolves Tranche C C.5 deferral (BLOCKED-ON-PRODUCER). The
         compaction_ladder runner is SHIPPED at
         crates/orchestrator/src/compaction_ladder.rs; this panel is
         the dashboard surface for the 6-tier traversal +
         CompactionLadderTierApplied SSB rows. -->
    <section class="s-section">
      <div class="s-header">compaction ladder (E-25)</div>
      <CompactionLadderPanel />
    </section>

    <!-- Section: FREEZE STATUS (C-43, Phase 4H scope-freeze)
         Minimal badge widget. Polls scope_freeze_status (commands_c43)
         every 15s. Renders green when every frozen_items entry in
         scope-freeze.toml has on-disk mtime <= ratification_ts_ms; red
         otherwise. ELAI design hypothesis — no paper grounding (the
         source plan's arXiv:2604.21725 cite was wrong, see
         workflow/sessions/s-25120/mistakes.md). -->
    <section class="s-section">
      <div class="s-header">phase 4H freeze (C-43)</div>
      <FreezeStatusBar />
    </section>

    <!-- Section: COMPILED PROMPT VARIANTS (I-36, SCHEMA_VERSION 268)
         Badge chip showing the number of promoted compiled prompt variants
         that have been substituted via CompiledPromptStore::lookup.
         Empty/zero is the expected default until the offline GEPA optimizer
         runs (Tier-2 DEFER). Kill-switch: ELAI_PROMPT_COMPILE_DISABLE=1. -->
    <section class="s-section">
      <div class="s-header">compiled prompts (I-36)</div>
      <div class="s-row">
        <span class="s-label">promoted variants used</span>
        <div class="s-control">
          <span
            class="badge-chip"
            class:badge-chip--active={COMPILED_VARIANTS.length > 0}
            title={COMPILED_VARIANTS.length > 0
              ? `last: ${COMPILED_VARIANTS[0].role}/${COMPILED_VARIANTS[0].backend} @ ${new Date(COMPILED_VARIANTS[0].ts).toISOString()}`
              : 'no compiled variants in store (expected until offline optimizer runs)'}
          >
            {COMPILED_VARIANTS.length}
          </span>
        </div>
      </div>
      {#if COMPILED_VARIANTS.length > 0}
        <div class="s-row">
          <span class="s-label">last role</span>
          <span class="mono s-value">{COMPILED_VARIANTS[0].role}</span>
        </div>
        <div class="s-row">
          <span class="s-label">last backend</span>
          <span class="mono s-value">{COMPILED_VARIANTS[0].backend}</span>
        </div>
        <div class="s-row">
          <span class="s-label">variant hash</span>
          <span class="mono s-value">{COMPILED_VARIANTS[0].variant_hash.slice(0, 12)}&hellip;</span>
        </div>
      {/if}
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

      <div class="s-row">
        <span class="s-label">Webhook URL</span>
        <div class="s-control s-control--wide">
          <input
            class="bare-input mono"
            type="url"
            placeholder="https://discord.com/api/webhooks/..."
            bind:value={webhookUrl}
            oninput={onWebhookUrlInput}
            onblur={onWebhookUrlBlur}
            aria-label="Notification webhook URL"
          />
          {#if webhookStatus === 'saved'}
            <span class="s-readonly mono" aria-live="polite">saved</span>
          {:else if webhookStatus === 'invalid'}
            <span class="s-readonly s-readonly--accent mono" aria-live="polite">invalid</span>
          {/if}
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
            value={session.currentProject?.path ?? 'F:\\.ELAI_workflow\\.elai_cc'}
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

      <!-- E-33 task 1: ui.activity_ribbon.enabled. Persisted to localStorage
           via uxPrefs.setActivityRibbonEnabled; ActivityRibbon collapses to
           zero height when disabled (mount stays for the a11y polite region). -->
      <div class="s-row">
        <span class="s-label">Activity ribbon</span>
        <div class="s-control">
          <button
            class="toggle-pill mono"
            class:toggle-pill--on={activityRibbonEnabled}
            onclick={toggleActivityRibbon}
            aria-pressed={activityRibbonEnabled}
          >{activityRibbonEnabled ? '[ ON ]' : '[ off ]'}</button>
        </div>
      </div>

      <!-- E-33 task 6: accessibility.motion. auto -> follow OS prefers-reduced-motion;
           reduced -> force reduced regardless of OS; full -> force motion on. -->
      <div class="s-row">
        <span class="s-label">Motion</span>
        <div class="s-control">
          <SegmentedControl options={MOTION_OPTIONS} bind:value={accessibilityMotion} />
        </div>
      </div>

      <!-- H-12: canvas-as-default toggle. When ON, new sessions open with the
           Canvas window as the primary panel instead of Activity. Persisted to
           localStorage under CANVAS_DEFAULT_KEY (separate from workspace key).
           The orbit tab remains accessible; this only changes the boot default.
           Promotion path (W-2): graduates to default-ON at H-16 after one release. -->
      <div class="s-row">
        <span class="s-label">Canvas as default</span>
        <div class="s-control">
          <button
            class="toggle-pill mono"
            class:toggle-pill--on={canvasAsDefaultLocal}
            onclick={toggleCanvasAsDefault}
            aria-pressed={canvasAsDefaultLocal}
          >{canvasAsDefaultLocal ? '[ ON ]' : '[ off ]'}</button>
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

      <!-- B-4 task #14: CANARY DRIFT (cosine vs persisted reference vec) -->
      <div class="s-row">
        <span class="s-label">Canary drift</span>
        <div class="s-control s-control--inline-gap">
          {#if canaryDriftState.known}
            <span class="s-ping mono" class:s-error={!canaryDriftState.passed}>
              cosine {canaryDriftState.similarity.toFixed(4)}
              / thr {canaryDriftState.drift_threshold.toFixed(2)}
              {canaryDriftState.passed ? '' : '!'}
            </span>
            <button class="link-btn" onclick={recomputeCanary}>recompute</button>
          {:else}
            <span class="s-ping mono">no snapshot yet</span>
            <button class="link-btn" onclick={recomputeCanary}>recompute</button>
          {/if}
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
     SettingsWindow — DESIGN.md role-token instrument.
     Vertical grouped settings list inside the settings modal.
     Section headers: §4 static dot-label (5px teal dot + dim caps).
     Rows are flat — the grouping is the well, not per-row chrome.
     Toggles: [ ON ] / [ off ] hairline-outlined mono buttons.
     Inputs: bare mono with a center->out teal underline-draw focus.
     Status: §6 state language — shape + mono, never a foreign hue.
     One teal accent (--d-teal family); every neutral teal-tinted.
     Shared --fl focus ring + .btn base collapse duplicate rules (MR-1).
  ================================================================ */

  .settings {
    padding: var(--s-5) var(--s-6) 0;
    min-height: 100%;
    background: var(--bg);
    /* one teal focus ring, reused by every interactive element here */
    --fl: 0 0 0 1px var(--d-teal), 0 0 0 3px var(--d-teal-soft);
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

  /* Section header — §4 STATIC label: 5px teal dot + dim mono caps */
  .s-header {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    font-family: var(--font-mono);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.2em;
    color: var(--dim);
    margin-bottom: var(--s-3);
  }
  .s-header::before {
    content: '';
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--d-teal);
    flex-shrink: 0;
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
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--read);
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
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--read);
  }
  .s-readonly--accent { color: var(--d-teal); }
  .s-readonly--mono {
    font-family: var(--font-mono);
    font-size: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .s-ping {
    font-size: 10px;
    color: var(--dim);
  }

  /* §6 fail language — inline, no foreign hue: --text glyph + read body */
  .s-fail-text {
    font-size: 0.8em;
    color: var(--read);
  }
  .s-fail-glyph {
    color: var(--text);
    font-weight: 600;
    margin-right: 0.2em;
  }
  /* §6 fail (drift / advisory) — shape carries it; --text, never red */
  .s-error { color: var(--text); }

  /* ---- s-value: read-only mono label for settings rows ---- */
  .s-value {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--read);
  }

  /* ---- I-36: compiled-prompt variant count badge chip (inset well) ---- */
  .badge-chip {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 20px;
    padding: 0 6px;
    border-radius: var(--r-lg);
    background: var(--panel);
    border: 1px solid var(--line);
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--dim);
    transition: background var(--dur-fast) var(--ease-std),
                color var(--dur-fast) var(--ease-std),
                border-color var(--dur-fast) var(--ease-std);
  }
  .badge-chip--active {
    background: var(--d-teal-ghost);
    border-color: var(--d-teal-soft);
    color: var(--d-teal);
  }

  /* ---- Bare input: no bg, no border, teal underline focus ---- */
  .bare-input {
    flex: 1;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--line);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 4px 2px;
    height: 28px;
    width: 100%;
    outline: none;
    transition: border-color var(--dur-fast) var(--ease-out);
  }

  .bare-input::placeholder { color: var(--ghost); }

  .bare-input:focus { border-bottom-color: var(--d-teal); }

  /* Hide the number-input spinners */
  .bare-input--num::-webkit-outer-spin-button,
  .bare-input--num::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
  .bare-input--num { -moz-appearance: textfield; appearance: textfield; }

  .bare-input--disabled,
  .bare-input:disabled {
    color: var(--ghost);
    border-bottom-color: var(--line);
    cursor: not-allowed;
  }

  /* ---- Override file textarea: hairline inset well ---- */
  .bare-textarea {
    width: 100%;
    background: var(--sink);
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 12px;
    padding: var(--s-2) var(--s-3);
    resize: vertical;
    outline: none;
    line-height: var(--leading-code);
    box-sizing: border-box;
    transition: border-color var(--dur-fast) var(--ease-out);
  }
  .bare-textarea:focus { border-color: var(--d-teal-soft); }

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
    color: var(--text);
    margin-top: var(--s-1);
  }

  /* ---- Toggle pill: [ ON ] / [ off ] hairline-outlined mono button ---- */
  .toggle-pill {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--dim);
    background: transparent;
    border: 1px solid var(--line);
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
    color: var(--read);
    border-color: var(--d-teal-soft);
  }
  .toggle-pill:focus-visible { box-shadow: var(--fl); }

  .toggle-pill--on {
    color: var(--d-teal);
    border-color: var(--d-teal-soft);
    background: var(--d-teal-ghost);
  }
  .toggle-pill--on:hover {
    color: var(--d-teal-bright);
    border-color: var(--d-teal);
  }

  /* ---- Link-style buttons: tertiary actions, teal underline-draw ---- */
  .link-btn {
    position: relative;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--d-teal);
    background: transparent;
    border: none;
    padding: 2px 0;
    cursor: pointer;
    outline: none;
    transition: color var(--dur-fast) var(--ease-std);
  }
  .link-btn::after {
    content: '';
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 1px;
    background: var(--d-teal);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-base) var(--ease-out);
  }
  .link-btn:hover { color: var(--d-teal-bright); }
  .link-btn:hover::after { transform: scaleX(1); }
  .link-btn:focus-visible { box-shadow: var(--fl); border-radius: var(--r-xs); }

  /* ---- Action buttons: shared base, two color skins (MR-1 collapse) ---- */
  .btn-approve,
  .btn-outlined {
    font-family: var(--font-mono);
    font-size: 12px;
    background: transparent;
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    padding: 4px 12px;
    cursor: pointer;
    outline: none;
    transition:
      background var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std);
  }
  .btn-approve {
    font-weight: 500;
    color: var(--d-teal);
    border-color: var(--d-teal-soft);
  }
  .btn-approve:hover {
    background: var(--d-teal-ghost);
    color: var(--d-teal-bright);
    border-color: var(--d-teal);
  }
  .btn-outlined { color: var(--read); }
  .btn-outlined:hover {
    background: var(--d-teal-ghost);
    color: var(--text);
    border-color: var(--d-teal-soft);
  }
  .btn-approve:focus-visible,
  .btn-outlined:focus-visible { box-shadow: var(--fl); }

  /* ---- Audit results: dim mono rows, no card chrome ---- */
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
    color: var(--dim);
  }
  /* §6 warn — shape only, drawn in --text, no own color */
  .audit-row--outdated .audit-crate { color: var(--text); }
  .audit-row--outdated .audit-crate::before { content: '▲ '; }
  .audit-crate { min-width: 140px; flex-shrink: 0; color: var(--read); }
  .audit-ver { color: var(--dim); flex-shrink: 0; }
  .audit-advisory {
    color: var(--text);
    font-size: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    opacity: 0.85;
  }

  @media (prefers-reduced-motion: reduce) {
    .link-btn::after { transition: none; }
    .link-btn:hover::after { transform: scaleX(1); }
  }
</style>
