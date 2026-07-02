// ============================================================
// ELAI Dashboard v2 — API surface
//
// Each function tries the real Tauri backend command first via
// `@tauri-apps/api/core` `invoke()`. If the runtime is NOT Tauri
// (e.g. plain `pnpm dev` against the Vite server with no native
// shell), the call falls through to the mock data in `mock-data.ts`
// so the dashboard remains usable in browser-only mode.
//
// Function names mirror the Tauri command names 1:1 (snake_case)
// so wiring is mechanically obvious. JS passes camelCase argument
// keys; Tauri v2 deserializes them into the Rust handler's
// snake_case parameter names.
// ============================================================

import type {
  FsmSnapshot,
  // B-26 T4 — Plan progress for the MonitorWindow phase-dot row.
  PlanProgress,
  QueryTrace,
  SearchMode,
  EmbeddingBreaker,
  RecallMetrics,
  GrcBudget,
  GroundingSweep,
  CsqEntry,
  EvidenceGate,
  AgentEntry,
  SupervisionEvent,
  ContextBudget,
  DecompNode,
  PipelineNode,
  ExecutablePipelineDag,
  DagRepairEvent,
  DagRepairEscalated,
  // B-7 (SCHEMA_VERSION 100): producer-wire surfaces.
  PackageRefVerified,
  // B-7 Task #8: PhaseSizingSplitter split-badge surface.
  PhaseSizedSplit,
  // B-7 Step 9: TreePositionChanged cursor surface.
  TreePosition,
  // B-7 Step 10: per-baby-step failure retry history.
  FailureRecord,
  ResearchTreeNode,
  VramGauge,
  InferenceMetrics,
  RuleLifecycleCounts,
  Rule,
  DeprecatedPattern,
  SourceTierDistribution,
  WebAragState,
  BuildSlot,
  DevServer,
  ProjectProfile,
  MistakeEntry,
  InterrogationNode,
  FailureTaxonomyResult,
  BspSuperstep,
  SessionBranch,
  AntiSpiralCounter,
  HeartbeatEntry,
  BaselineResults,
  DepAuditResult,
  MsiStatus,
  SteeringStatus,
  McpStatus,
  GateStatus,
  AgentRole,
  ModelRouting,
  SystemStatus,
  GateMode,
  AutoAdvanceStats,
  OptimizationProposal,
  InterventionAction,
  FeasibilityReport,
  ProjectSummary,
  SidecarStatus,
  SidecarHealth,
  HookExecution,
  ShortCoderStats,
  GgSignals,
  DkgStats,
  BubbleUpEvent,
  FsmState,
  RemoteSidecarProgress,
  ScaffoldTemplate,
  ApplyScaffoldArgs,
  LintFinding,
  LighthouseReport,
  SandboxStatus,
  OAuthStatus,
  OAuthLoginInfo,
  CloudModelInfo,
  CloudSpendInfo,
  // C-5 follow-on — Cloud routing status (RouteBlocked / WarnLimitReached / CostEstimate).
  CloudRouteStatus,
  MutationGaugeInfo,
  // C-32 — Code-Completeness Layer 2 gauge.
  CompletenessRatioInfo,
  SelfModProposalRow,
  SelfModMsiSnapshot,
  // F-8 / Slice A5 — SCF symbolic-conflict feed row.
  SymbolicConflict,
  // F-11 — config-DGM admit-loop feed row.
  DarwinianAdmit,
  // H-46 — CMP parent-selection feed row.
  CladeSelection,
  // F-12 — semantic-contradiction feed.
  SemanticContradictionsResponse,
  // F-13 / Slice A5 — session-archive feed row.
  SessionArchive,
  ReflectState,
  MediaValidationEntry,
  MediaSsbEvent,
  ReviewDecision,
  // B-40 — Idea-Framer types.
  IdeaFramingState,
  TradeoffResponse,
  // B-42 — Cosmos artifact tier.
  CosmosArtifactStatus,
  // B-41 — Per-level audit timeline.
  LevelAuditVerdictJson,
  // F-4 Slice B-5 — TestIntegrityEvent fabrication badge.
  TestIntegrityEventJson,
  // B-37 Phase 4M — Multi-runtime adapter surface.
  RuntimeInfo,
  RuntimeBackendId,
  RuntimeAgentRole,
  TestReport,
  // C-4 — Tauri commands orchestration + cloud spend surface.
  PhaseDetailResponse,
  BabyStepDetailResponse,
  PhaseProgressRow,
  RecentActionRow,
  FileChangeRow,
  ActionDetailResponse,
  ListProjectsResponse,
  ProjectEntry,
  ResearchNodeHistoryRow,
  // E-17 SP4 — RAG retrieval-quality beacon row types.
  DeepPoolCappedRow,
  RrfQualityGateRow,
  HydeSkippedCodeQueryRow,
  RrfComponentWeightsRow,
  // B-4 — HyDE hypothesis-preview row for the QueryTrace inspector.
  HydeQueryRow,
  // B-5 — full CSQ verdict log row + per-role GRC config row.
  CsqLogRow,
  GrcConfigRow,
  // B-5 GAP-1 — anti-rubber-stamp rejection log row.
  AuditorRubberStampRejectionRow,
  // B-3 — feature-flag + query-tier + rule-transition surfaces.
  FeatureFlags,
  QueryTierStats,
  RuleTransitionRow,
  SessionHandoffEntry,
  // C-41 — Application Skills System DTOs.
  SkillManifest,
  SkillRoutingEvent,
  // F-6 / A5 — learned-skill library Trust-Ladder DTO.
  SkillLibraryResponse,
  // F-5 / A5 — staleness-quarantine cascade DTO.
  StalenessQuarantineResponse,
  // F-7 / A5 — template-ranking (Laplace utility) DTO.
  TemplateRankingResponse,
  // E-25 Tranche N (s-22600 2026-06-01): Compaction-Ladder dashboard.
  CompactionLadderStateDto,
  CompactionEventDto,
} from './types';

import {
  FSM_SNAPSHOT,
  QUERY_TRACES,
  EMBEDDING_BREAKERS,
  RECALL_METRICS,
  GRC_BUDGETS,
  GROUNDING_SWEEP,
  CSQ_PENDING,
  EVIDENCE_GATES,
  ACTIVE_AGENTS,
  SUPERVISION_LOG,
  CONTEXT_BUDGET,
  DECOMP_TREE,
  PIPELINE_NODES,
  DAG_REPAIR_EVENTS,
  DAG_REPAIR_ESCALATED,
  RESEARCH_TREE,
  VRAM_GAUGES,
  INFERENCE_METRICS,
  RULE_LIFECYCLE_COUNTS,
  ACTIVE_RULES,
  DEPRECATED_PATTERNS,
  SOURCE_TIER_DISTRIBUTION,
  WEB_ARAG_STATE,
  BUILD_SLOTS,
  DEV_SERVERS,
  PROJECT_PROFILE,
  MISTAKE_BOOK,
  BSP_SUPERSTEPS,
  SESSION_BRANCHES,
  ANTI_SPIRAL_COUNTER,
  HEARTBEATS,
  BASELINE_RESULTS,
  DEP_AUDIT,
  MSI_STATUS,
  STEERING_STATUS,
  MCP_STATUS,
  GATE_STATUS,
  MODEL_ROUTING,
  SYSTEM_STATUS,
  AUTO_ADVANCE_STATS,
  OPTIMIZATION_PROPOSALS,
  FEASIBILITY_REPORT,
  RECENT_PROJECTS,
  SIDECAR_STATUS,
  HOOK_EXECUTIONS,
  SHORTCODER_STATS,
  GG_SIGNALS,
  DKG_STATS,
  BUBBLE_UP_EVENTS,
  GATE_STATUS as GATE_STATUS_IMPORT,
  SCAFFOLD_TEMPLATES,
} from './mock-data';

// ------------------------------------------------------------------
// Tauri detection + invoke wrapper
// ------------------------------------------------------------------

const TAURI_AVAILABLE: boolean =
  typeof window !== 'undefined' &&
  // Tauri 2 sets `__TAURI_INTERNALS__`; the older `__TAURI__` exists for
  // backward compat. Either present means we're inside the native shell.
  (Boolean((window as any).__TAURI_INTERNALS__) ||
    Boolean((window as any).__TAURI__));

// B-3 V2 rebaseline: `VITE_ELAI_DASHBOARD_OFFLINE=1` (compile-time) or
// `window.__ELAI_DASHBOARD_OFFLINE__=true` (runtime, set by the Tauri shell
// from the `ELAI_DASHBOARD_OFFLINE` env var via the feature-flag command)
// forces stub.ts to short-circuit Tauri invocation and serve mock-data.
// Useful for headless screenshot runs, smoke tests, and dashboard-dev
// iteration without booting the full orchestrator stack.
const DASHBOARD_OFFLINE: boolean =
  (typeof window !== 'undefined' &&
    Boolean((window as any).__ELAI_DASHBOARD_OFFLINE__)) ||
  (typeof import.meta !== 'undefined' &&
    (import.meta as any).env?.VITE_ELAI_DASHBOARD_OFFLINE === '1');

let _invokeFn:
  | (<T>(cmd: string, args?: Record<string, unknown>) => Promise<T>)
  | null = null;

async function ensureInvoke(): Promise<typeof _invokeFn> {
  if (_invokeFn !== null || !TAURI_AVAILABLE) return _invokeFn;
  try {
    const mod = await import('@tauri-apps/api/core');
    _invokeFn = mod.invoke;
  } catch (e) {
    console.warn('[stub] @tauri-apps/api/core import failed; falling back to mocks', e);
    _invokeFn = null;
  }
  return _invokeFn;
}

/** Try to invoke a Tauri command; on any failure (or if not in Tauri,
 *  or if the dashboard is forced offline via `ELAI_DASHBOARD_OFFLINE`),
 *  return `undefined` so the caller falls through to mock data. */
async function tryInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T | undefined> {
  if (!TAURI_AVAILABLE) return undefined;
  // B-3 V2 rebaseline: short-circuit when the dashboard is forced offline.
  // The `get_feature_flags` command is exempt — it is the canonical source
  // of the runtime offline signal so the shell can hydrate
  // `window.__ELAI_DASHBOARD_OFFLINE__` from the Tauri env-var read.
  if (DASHBOARD_OFFLINE && cmd !== 'get_feature_flags') return undefined;
  const inv = await ensureInvoke();
  if (!inv) return undefined;
  try {
    return (await inv<T>(cmd, args)) as T;
  } catch (e) {
    console.warn(`[stub] invoke('${cmd}') rejected — falling back to mock`, e);
    return undefined;
  }
}

export function delay(min = 80, max = 240): Promise<void> {
  if (TAURI_AVAILABLE) return Promise.resolve();
  const ms = min + Math.random() * (max - min);
  return new Promise((r) => setTimeout(r, ms));
}

export function logStub(name: string, payload?: unknown): void {
  console.info('[stub]', name, payload ?? '');
}

// In-memory mutable state for the mock fallback
let _fsmState: FsmSnapshot = { ...FSM_SNAPSHOT };
let _gateMode: GateMode = GATE_STATUS.mode;
let _modelRouting: ModelRouting[] = [...MODEL_ROUTING];
let _buildSlots: BuildSlot[] = [...BUILD_SLOTS];

// ------------------------------------------------------------------
// FSM
// ------------------------------------------------------------------

export async function get_fsm_state(): Promise<FsmSnapshot> {
  logStub('get_fsm_state');
  const live = await tryInvoke<FsmSnapshot>('get_fsm_state');
  if (live) return live;
  await delay();
  return { ..._fsmState };
}

// ------------------------------------------------------------------
// B-26 T4 — plan progress for the MonitorWindow phase-dot row.
// Returns the live Tauri payload when available; falls through to a
// pending-only stub when offline so the dashboard renders an empty
// 5-phase row instead of an error.
// ------------------------------------------------------------------
export async function get_plan_progress(): Promise<PlanProgress> {
  logStub('get_plan_progress');
  const live = await tryInvoke<PlanProgress>('get_plan_progress');
  if (live) return live;
  await delay();
  return {
    phases: [
      { num: 1, label: 'Plan', steps: 0, status: 'pending' },
      { num: 2, label: 'Feasibility', steps: 0, status: 'pending' },
      { num: 3, label: 'Execute', steps: 0, status: 'pending' },
      { num: 4, label: 'Audit', steps: 0, status: 'pending' },
      { num: 5, label: 'Memorize', steps: 0, status: 'pending' },
    ],
    current_phase: null,
    total_pct: 0,
  };
}

// ------------------------------------------------------------------
// SYSTEM STATUS
// ------------------------------------------------------------------

export async function get_system_status(): Promise<SystemStatus> {
  logStub('get_system_status');
  const live = await tryInvoke<SystemStatus>('get_system_status');
  if (live) return live;
  await delay();
  return { ...SYSTEM_STATUS, fsm: { ..._fsmState } };
}

// ------------------------------------------------------------------
// B-29 Phase 5: IMPACT ANALYSIS (symbol-graph blast radius)
// ------------------------------------------------------------------

/**
 * Most recent symbol_query SSB row reshaped for the KnowledgeDrawer panel.
 * Backed by the `get_impact_analysis` Tauri command (see
 * crates/dashboard-v2/src-tauri/src/commands.rs:932).
 */
export interface ImpactAnalysis {
  target: string;
  total_files: number;
  hop_bands: { depth: number; files: string[] }[];
  unresolved_pct: number;
}

const EMPTY_IMPACT_ANALYSIS: ImpactAnalysis = {
  target: '',
  total_files: 0,
  hop_bands: [],
  unresolved_pct: 0.0,
};

export async function get_impact_analysis(): Promise<ImpactAnalysis> {
  logStub('get_impact_analysis');
  const live = await tryInvoke<ImpactAnalysis>('get_impact_analysis');
  if (live) return live;
  await delay();
  return { ...EMPTY_IMPACT_ANALYSIS };
}

// ------------------------------------------------------------------
// TASK / WORKFLOW
// ------------------------------------------------------------------

export async function submit_task(
  task: string,
  autonomous?: boolean,
): Promise<{ taskId: string }> {
  logStub('submit_task', { task, autonomous });
  const live = await tryInvoke<{ taskId: string }>('submit_task', {
    task,
    autonomous: autonomous ?? false,
  });
  if (live) return live;
  await delay(150, 240);
  const taskId = `t-${Date.now()}`;
  _fsmState = { ..._fsmState, state: 'PLANNING', current_step_id: '' };
  return { taskId };
}

export async function submit_intervention_response(
  actionId: string,
  response: InterventionAction,
  payload?: unknown,
): Promise<{ ok: boolean }> {
  logStub('submit_intervention_response', { actionId, response, payload });
  const live = await tryInvoke<{ ok: boolean }>('submit_intervention_response', {
    actionId,
    response,
    payload,
  });
  if (live) return live;
  await delay();
  return { ok: true };
}

// C2: submit credential form values to the orchestrator's credential gate.
export async function submit_credential_response(
  actionId: string,
  credentials: Record<string, string>,
): Promise<{ ok: boolean }> {
  logStub('submit_credential_response', { actionId, fieldCount: Object.keys(credentials).length });
  const live = await tryInvoke<{ ok: boolean }>('submit_credential_response', {
    actionId,
    credentials,
  });
  if (live) return live;
  await delay();
  return { ok: true };
}

export async function open_project_from_folder(path: string): Promise<ProjectSummary> {
  logStub('open_project_from_folder', { path });
  const live = await tryInvoke<ProjectSummary>('open_project_from_folder', { path });
  if (live) return live;
  await delay(100, 200);
  const existing = RECENT_PROJECTS.find((p) => p.path === path);
  if (existing) return existing;
  const name = path.split(/[\\/]/).filter(Boolean).pop() ?? 'unknown';
  return { name, path, last_opened_iso: new Date().toISOString() };
}

export async function get_feasibility_report(): Promise<FeasibilityReport> {
  logStub('get_feasibility_report');
  const live = await tryInvoke<FeasibilityReport>('get_feasibility_report');
  if (live) return live;
  await delay();
  return FEASIBILITY_REPORT;
}

// E-22 phase-4F P5: stub_results.msgpack reader. Live-only — the mock
// returns an empty envelope when invoke() is unavailable so the browser-
// only dev mode does not synthesize fake stub results.
export interface StubResultsEnvelope {
  task_id: string;
  path: string;
  count: number;
  results: Array<{
    step_id: string;
    stub_passed: boolean;
    elapsed_ms: number;
    error: string | null;
    alternative_index: number;
    stdout_hash: string | null;
    stderr_summary: string | null;
    resource_delta: {
      vram_bytes_before: number;
      vram_bytes_after: number;
      ram_bytes_before: number;
      ram_bytes_after: number;
    } | null;
  }>;
}

export async function get_stub_results_msgpack(
  taskId: string,
): Promise<StubResultsEnvelope> {
  logStub('get_stub_results_msgpack', { taskId });
  const live = await tryInvoke<StubResultsEnvelope>('get_stub_results_msgpack', {
    taskId,
  });
  if (live) return live;
  return { task_id: taskId, path: '', count: 0, results: [] };
}

// ------------------------------------------------------------------
// GATE / CONTROL
// ------------------------------------------------------------------

export async function set_gate_mode(mode: GateMode): Promise<GateStatus> {
  logStub('set_gate_mode', { mode });
  const live = await tryInvoke<GateStatus>('set_gate_mode', { mode: mode.toLowerCase() });
  if (live) return live;
  await delay(80, 140);
  _gateMode = mode;
  return { ...GATE_STATUS_IMPORT, mode };
}

export async function get_gate_status(): Promise<GateStatus> {
  logStub('get_gate_status');
  const live = await tryInvoke<GateStatus>('get_gate_status');
  if (live) return live;
  await delay();
  return { ...GATE_STATUS, mode: _gateMode };
}

export async function get_auto_advance_stats(): Promise<AutoAdvanceStats> {
  logStub('get_auto_advance_stats');
  const live = await tryInvoke<AutoAdvanceStats>('get_auto_advance_stats');
  if (live) return live;
  await delay();
  return AUTO_ADVANCE_STATS;
}

export async function respond_to_confirmation(
  confirmationId: string,
  decision: 'confirm' | 'cancel',
): Promise<{ ok: boolean }> {
  logStub('respond_to_confirmation', { confirmationId, decision });
  const live = await tryInvoke<{ ok: boolean }>('respond_to_confirmation', {
    confirmationId,
    decision,
  });
  if (live) return live;
  await delay();
  return { ok: true };
}

// ------------------------------------------------------------------
// MODEL ROUTING
// ------------------------------------------------------------------

export async function set_model_routing(role: AgentRole, target: string): Promise<ModelRouting[]> {
  logStub('set_model_routing', { role, target });
  const live = await tryInvoke<ModelRouting[]>('set_model_routing', { role, target });
  if (live) return live;
  await delay(80, 140);
  _modelRouting = _modelRouting.map((r) => (r.role === role ? { ...r, target } : r));
  return [..._modelRouting];
}

// E-3 (SCHEMA_VERSION 188) MVP: emit one SSB row when the SettingsWindow
// "show model details" toggle changes. Wire-dark unless
// ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE=1 on the backend; returns
// {ok, emitted} in both cases. Closes the cross-check finding
// "SettingsWindow explicitly exposes MODEL names per role".
export async function record_model_visibility(
  role: AgentRole,
  modelHidden: boolean,
  sessionId: string | null = null,
): Promise<{ ok: boolean; emitted: boolean }> {
  logStub('record_model_visibility', { role, modelHidden, sessionId });
  const live = await tryInvoke<{ ok: boolean; emitted: boolean }>(
    'record_model_visibility',
    { role, modelHidden, sessionId },
  );
  if (live) return live;
  return { ok: true, emitted: false };
}

// E-3 sub-phase 4B (SCHEMA_VERSION 190) — runtime-handoff event
// recorder. Fans the 6 RuntimeHandoffEventKind arms into a single SSB
// row per call. Wire-dark unless ELAI_E3_RUNTIME_HANDOFF_ENABLE=1.
export type RuntimeHandoffEventKind =
  | 'unified_preamble_applied'
  | 'runtime_prewarm_started'
  | 'runtime_prewarm_complete'
  | 'handoff_payload_compressed'
  | 'persona_vector_applied'
  | 'style_normalizer_applied';

export async function record_runtime_handoff_event(
  kind: RuntimeHandoffEventKind,
  role: string | null = null,
  backend: string | null = null,
  latencyMs: number | null = null,
  details: Record<string, unknown> | null = null,
): Promise<{ ok: boolean; emitted: boolean }> {
  logStub('record_runtime_handoff_event', { kind, role, backend, latencyMs, details });
  const live = await tryInvoke<{ ok: boolean; emitted: boolean }>(
    'record_runtime_handoff_event',
    { kind, role, backend, latencyMs, details },
  );
  if (live) return live;
  return { ok: true, emitted: false };
}

// ------------------------------------------------------------------
// PROPOSALS
// ------------------------------------------------------------------

export async function get_active_optimizations(): Promise<OptimizationProposal[]> {
  logStub('get_active_optimizations');
  const live = await tryInvoke<OptimizationProposal[]>('get_active_optimizations');
  if (live) return live;
  await delay();
  return OPTIMIZATION_PROPOSALS.filter((p) => p.status === 'pending');
}

// B-23 Phase 4H Group F (4H.8) — these now return the self-mod shape from
// the Rust Tauri impl. The legacy `OptimizationProposal[]` mock-data return
// is preserved for browser-only dev mode where the Rust handler is absent.
export async function get_pending_proposals(): Promise<
  SelfModProposalRow[] | OptimizationProposal[]
> {
  logStub('get_pending_proposals');
  const live = await tryInvoke<SelfModProposalRow[]>('get_pending_proposals');
  if (live) return live;
  await delay();
  return OPTIMIZATION_PROPOSALS.filter((p) => p.status === 'pending');
}

export async function get_applied_proposals(): Promise<
  SelfModProposalRow[] | OptimizationProposal[]
> {
  logStub('get_applied_proposals');
  const live = await tryInvoke<SelfModProposalRow[]>('get_applied_proposals');
  if (live) return live;
  await delay();
  return OPTIMIZATION_PROPOSALS.filter((p) => p.status === 'applied');
}

// F-8 / Slice A5 — held-pending SCF symbolic-conflict feed. No mock data:
// browser-only dev returns an empty array (steady state for the detector is
// no rows; conflicts only appear when a self-mod variant contradicts a
// locked-key invariant). Live path reads the `get_symbolic_conflicts`
// Tauri command newest-first.
export async function get_symbolic_conflicts(): Promise<SymbolicConflict[]> {
  logStub('get_symbolic_conflicts');
  const live = await tryInvoke<SymbolicConflict[]>('get_symbolic_conflicts');
  if (live) return live;
  await delay();
  return [];
}

// F-12 — semantic-contradiction feed. No mock data: browser-only dev returns
// the empty envelope (steady state is no rows; rows only appear after the live
// memory tick's deterministic-NLI second pass emits a memory_conflict_detected
// row with conflict_kind="semantic_contradiction"). Live path reads the
// `get_semantic_contradictions` Tauri command newest-first.
export async function get_semantic_contradictions(
  limit = 100,
): Promise<SemanticContradictionsResponse> {
  logStub('get_semantic_contradictions', { limit });
  const live = await tryInvoke<SemanticContradictionsResponse>(
    'get_semantic_contradictions',
    { limit },
  );
  if (live) return live;
  await delay();
  return { viewer_attached: false, pairs: [] };
}

// F-11 — config-DGM admit-loop feed. No mock data: browser-only dev returns an
// empty array (steady state is no rows; admit/reject/loop-closed rows only
// appear after a `darwin-loop` run with ELAI_DARWINIAN_LOOP_ENABLE=1 + feature
// darwinian-evolve). Live path reads the `get_darwinian_admits` Tauri command
// newest-first.
export async function get_darwinian_admits(): Promise<DarwinianAdmit[]> {
  logStub('get_darwinian_admits');
  const live = await tryInvoke<DarwinianAdmit[]>('get_darwinian_admits');
  if (live) return live;
  await delay();
  return [];
}

// H-46 — CMP parent-selection feed. Browser-only dev returns an empty array
// (steady state is no rows; clade_selected rows only appear after a `darwin-loop`
// run with ELAI_DARWINIAN_LOOP_ENABLE=1 + feature darwinian-evolve). Live path
// reads the `get_clade_selections` Tauri command newest-first.
export async function get_clade_selections(): Promise<CladeSelection[]> {
  logStub('get_clade_selections');
  const live = await tryInvoke<CladeSelection[]>('get_clade_selections');
  if (live) return live;
  await delay();
  return [];
}

// F-13 / Slice A5 — session-archive feed. Browser-only dev returns an empty
// array (steady state is no rows: archives only appear when a stale-version SSB
// is opened with the wire-dark ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE gate set). Live
// path reads the `get_session_archives` Tauri command newest-first.
export async function get_session_archives(): Promise<SessionArchive[]> {
  logStub('get_session_archives');
  const live = await tryInvoke<SessionArchive[]>('get_session_archives');
  if (live) return live;
  await delay();
  return [];
}

export async function apply_proposal_from_user(proposalId: string): Promise<{ ok: boolean }> {
  logStub('apply_proposal_from_user', { proposalId });
  const live = await tryInvoke<{ ok: boolean }>('apply_proposal_from_user', { proposalId });
  if (live) return live;
  await delay(100, 200);
  return { ok: true };
}

export async function reject_proposal(proposalId: string): Promise<{ ok: boolean }> {
  logStub('reject_proposal', { proposalId });
  const live = await tryInvoke<{ ok: boolean }>('reject_proposal', { proposalId });
  if (live) return live;
  await delay();
  return { ok: true };
}

// ------------------------------------------------------------------
// MSI / STEERING / MCP
// ------------------------------------------------------------------

// B-23 Phase 4H Group F (4H.8) — get_msi_status now returns an array of
// per-PipelineMode self-mod MSI snapshots. The legacy multi-session MSI
// shape stays available via the `MsiStatus` interface for any caller that
// still expects it (none in v2 today).
export async function get_msi_status(): Promise<SelfModMsiSnapshot[] | MsiStatus> {
  logStub('get_msi_status');
  const live = await tryInvoke<SelfModMsiSnapshot[]>('get_msi_status');
  if (live) return live;
  await delay();
  return MSI_STATUS;
}

export async function get_steering_status(): Promise<SteeringStatus> {
  logStub('get_steering_status');
  const live = await tryInvoke<SteeringStatus>('get_steering_status');
  if (live) return live;
  await delay();
  return STEERING_STATUS;
}

export async function recompute_steering_vector(): Promise<SteeringStatus> {
  logStub('recompute_steering_vector');
  const live = await tryInvoke<SteeringStatus>('recompute_steering_vector');
  if (live) return live;
  await delay(180, 240);
  return { ...STEERING_STATUS, last_computed_ts: Date.now(), magnitude: 0.81 };
}

export async function get_mcp_status(): Promise<McpStatus> {
  logStub('get_mcp_status');
  const live = await tryInvoke<McpStatus>('get_mcp_status');
  if (live) return live;
  await delay();
  return MCP_STATUS;
}

// ------------------------------------------------------------------
// KNOWLEDGE
// ------------------------------------------------------------------

export async function get_query_trace_recent(n: number): Promise<QueryTrace[]> {
  logStub('get_query_trace_recent', { n });
  const live = await tryInvoke<QueryTrace[]>('get_query_trace_recent', { n });
  if (live) return live;
  await delay();
  return QUERY_TRACES.slice(0, n);
}

// B-4 task #5: singular per-query QueryTrace inspector. Returns the
// aggregate SSB row (query_id / tier / stage_count / result_count /
// total_latency_ms) for one query, or {known:false} when no match.
export interface QueryTraceDetail {
  known: boolean;
  seq?: number;
  ts?: number;
  query_id?: string;
  tier?: string;
  stage_count?: number;
  result_count?: number;
  total_latency_ms?: number;
}

export async function get_query_trace(query_id: string): Promise<QueryTraceDetail> {
  logStub('get_query_trace', { query_id });
  const live = await tryInvoke<QueryTraceDetail>('get_query_trace', { query_id });
  if (live) return live;
  await delay();
  return { known: false };
}

// B-4 task #13: index version + upgrade-action surface.
export interface IndexVersionSnapshot {
  known: boolean;
  version?: number;
  dkg_schema_version?: number;
  vector_dimensions?: { code: number; text: number };
  bm25_format?: string;
  created_at?: number;
  error?: string;
}

export async function get_index_version(): Promise<IndexVersionSnapshot> {
  logStub('get_index_version');
  const live = await tryInvoke<IndexVersionSnapshot>('get_index_version');
  if (live) return live;
  await delay();
  return { known: false };
}

export interface UpgradeActionSnapshot {
  known: boolean;
  action?: 'Reuse' | 'RebuildBm25Only' | 'FullRebuild';
  error?: string;
}

export async function get_upgrade_action(): Promise<UpgradeActionSnapshot> {
  logStub('get_upgrade_action');
  const live = await tryInvoke<UpgradeActionSnapshot>('get_upgrade_action');
  if (live) return live;
  await delay();
  return { known: false };
}

// B-4: HyDE hypothesis-preview feed for the QueryTrace inspector. Falls
// back to [] in browser-only mode (no Tauri runtime) — "no HyDE rows yet"
// is a legitimate steady state, not a mock-worthy scenario.
export async function get_hyde_query_recent(n: number): Promise<HydeQueryRow[]> {
  logStub('get_hyde_query_recent', { n });
  const live = await tryInvoke<HydeQueryRow[]>('get_hyde_query_recent', { n });
  if (live) return live;
  await delay();
  return [];
}

// ------------------------------------------------------------------
// E-17 SP4: RAG retrieval-quality beacon row wrappers.
//
// Each wrapper mirrors the Tauri command name 1:1 and falls back to an
// empty array when outside the Tauri runtime (browser-only `pnpm dev`).
// No mock fixtures — the four feeds are steady-state empty for most
// sessions (only DEEP-tier queries that trip the cap / floor / HyDE-skip
// produce rows), so the drawer's empty state is the correct browser
// rendering path.
// ------------------------------------------------------------------

export async function get_deep_pool_capped_recent(n: number): Promise<DeepPoolCappedRow[]> {
  logStub('get_deep_pool_capped_recent', { n });
  const live = await tryInvoke<DeepPoolCappedRow[]>('get_deep_pool_capped_recent', { n });
  if (live) return live;
  await delay();
  return [];
}

export async function get_rrf_quality_gate_recent(n: number): Promise<RrfQualityGateRow[]> {
  logStub('get_rrf_quality_gate_recent', { n });
  const live = await tryInvoke<RrfQualityGateRow[]>('get_rrf_quality_gate_recent', { n });
  if (live) return live;
  await delay();
  return [];
}

export async function get_hyde_skipped_code_query_recent(
  n: number,
): Promise<HydeSkippedCodeQueryRow[]> {
  logStub('get_hyde_skipped_code_query_recent', { n });
  const live = await tryInvoke<HydeSkippedCodeQueryRow[]>(
    'get_hyde_skipped_code_query_recent',
    { n },
  );
  if (live) return live;
  await delay();
  return [];
}

export async function get_rrf_component_weights_recent(
  n: number,
): Promise<RrfComponentWeightsRow[]> {
  logStub('get_rrf_component_weights_recent', { n });
  const live = await tryInvoke<RrfComponentWeightsRow[]>(
    'get_rrf_component_weights_recent',
    { n },
  );
  if (live) return live;
  await delay();
  return [];
}

export async function get_search_mode(): Promise<{
  mode: SearchMode;
  history: { mode: SearchMode; ts: number }[];
}> {
  logStub('get_search_mode');
  const live = await tryInvoke<{ mode: SearchMode; history: { mode: SearchMode; ts: number }[] }>(
    'get_search_mode',
  );
  if (live) return live;
  await delay();
  return {
    mode: 'HYBRID',
    history: [
      { mode: 'BM25_ONLY', ts: Date.now() - 44 * 60 * 1000 },
      { mode: 'HYBRID', ts: Date.now() - 43 * 60 * 1000 },
    ],
  };
}

export async function get_embedding_breakers(): Promise<EmbeddingBreaker[]> {
  logStub('get_embedding_breakers');
  const live = await tryInvoke<EmbeddingBreaker[]>('get_embedding_breakers');
  if (live) return live;
  await delay();
  return EMBEDDING_BREAKERS;
}

export async function get_recall_metrics(): Promise<RecallMetrics[]> {
  logStub('get_recall_metrics');
  const live = await tryInvoke<RecallMetrics[]>('get_recall_metrics');
  if (live) return live;
  await delay();
  return RECALL_METRICS;
}

// ------------------------------------------------------------------
// B-4: knowledge dashboard reads — canary drift, freshness state,
// vector-cache hit rate, pagerank top, query tier stats.
//
// Each wrapper mirrors the matching Tauri command name 1:1 and falls
// back to a `known:false` empty shape when outside the Tauri runtime
// (browser-only `pnpm dev`) so the panels render "(no data)" instead
// of crashing on undefined.
// ------------------------------------------------------------------

export interface CanaryDriftSnapshot {
  known: boolean;
  similarity: number;
  drift_threshold: number;
  passed: boolean;
  reference_present: boolean;
  ts: number;
}

export async function get_canary_drift(): Promise<CanaryDriftSnapshot> {
  logStub('get_canary_drift');
  const live = await tryInvoke<CanaryDriftSnapshot>('get_canary_drift');
  if (live) return live;
  await delay();
  return {
    known: false,
    similarity: 0.0,
    drift_threshold: 0.85,
    passed: false,
    reference_present: false,
    ts: 0,
  };
}

export interface FreshnessStateSnapshot {
  known: boolean;
  decay_rate: number;
  access_boost: number;
  last_recalc_ms: number;
}

export async function get_freshness_state(): Promise<FreshnessStateSnapshot> {
  logStub('get_freshness_state');
  const live = await tryInvoke<FreshnessStateSnapshot>('get_freshness_state');
  if (live) return live;
  await delay();
  return { known: false, decay_rate: 0.0, access_boost: 0.0, last_recalc_ms: 0 };
}

export interface VectorCacheHitRateSnapshot {
  known: boolean;
  session_id: string;
  hits: number;
  misses: number;
  hit_rate_pct: number;
  ts: number;
}

export async function get_vector_cache_hit_rate(): Promise<VectorCacheHitRateSnapshot> {
  logStub('get_vector_cache_hit_rate');
  const live = await tryInvoke<VectorCacheHitRateSnapshot>('get_vector_cache_hit_rate');
  if (live) return live;
  await delay();
  return { known: false, session_id: '', hits: 0, misses: 0, hit_rate_pct: 0.0, ts: 0 };
}

export async function get_dkg_stats(): Promise<DkgStats> {
  logStub('get_dkg_stats');
  const live = await tryInvoke<DkgStats>('get_dkg_stats');
  if (live) return live;
  await delay();
  return DKG_STATS;
}

export async function get_language_grammar_status(): Promise<{ loaded: string[], failed: [string, string][] }> {
  logStub('get_language_grammar_status');
  const live = await tryInvoke<{ loaded: string[], failed: [string, string][] }>('get_language_grammar_status');
  if (live) return live;
  await delay();
  return { loaded: [], failed: [] };
}

// ------------------------------------------------------------------
// REASONING
// ------------------------------------------------------------------

export async function get_grc_budgets(): Promise<GrcBudget[]> {
  logStub('get_grc_budgets');
  const live = await tryInvoke<GrcBudget[]>('get_grc_budgets');
  if (live) return live;
  await delay();
  return GRC_BUDGETS;
}

export async function get_grounding_sweep(): Promise<GroundingSweep> {
  logStub('get_grounding_sweep');
  const live = await tryInvoke<GroundingSweep>('get_grounding_sweep');
  if (live) return live;
  await delay();
  return GROUNDING_SWEEP;
}

export async function get_csq_pending(): Promise<CsqEntry[]> {
  logStub('get_csq_pending');
  const live = await tryInvoke<CsqEntry[]>('get_csq_pending');
  if (live) return live;
  await delay();
  return CSQ_PENDING;
}

// B-5: full CSQ verdict log (every csq_evaluated row, not just blocked
// ones). The CsqBlockCard filters this for Critical-without-guard rows.
// Falls back to [] in browser-only mode — "no CSQ verdicts yet" is a
// legitimate steady state.
export async function get_csq_log(): Promise<CsqLogRow[]> {
  logStub('get_csq_log');
  const live = await tryInvoke<CsqLogRow[]>('get_csq_log');
  if (live) return live;
  await delay();
  return [];
}

// B-5: per-role GRC budget config table (compile-time default_grc_config).
// Falls back to [] in browser-only mode (no Tauri runtime); the
// ReasoningDrawer GRC-config table renders an empty-state note in that case.
export async function get_agent_grc_config(): Promise<GrcConfigRow[]> {
  logStub('get_agent_grc_config');
  const live = await tryInvoke<GrcConfigRow[]>('get_agent_grc_config');
  if (live) return live;
  await delay();
  return [];
}

// C-3: research budget snapshot (admitted / pruned / token usage of the
// most recent research run). Backed by the get_research_budget Tauri
// command -> SsbViewer::research_budget_snapshot. `known: false` until a
// research_budget_* SSB row type lands (C-1 producer scope); the
// ResearchDrawer renders its honest "no data yet" note in that case.
export interface ResearchBudgetSnapshot {
  known: boolean;
  admitted: number;
  pruned: number;
  tokens_used: number;
  tokens_max: number;
}

export async function get_research_budget(): Promise<ResearchBudgetSnapshot> {
  logStub('get_research_budget');
  const live = await tryInvoke<ResearchBudgetSnapshot>('get_research_budget');
  if (live) return live;
  await delay();
  return { known: false, admitted: 0, pruned: 0, tokens_used: 0, tokens_max: 0 };
}

// C-3: source-tier distribution (T1..T5 counts of the last research run).
// Backed by get_source_tier_distribution -> SsbViewer::source_tier_distribution.
// Flat t1..t5 shape (distinct from the per-row SourceTierDistribution[]
// used by get_source_quality_tiers). `known: false` until a producer lands.
export interface SourceTierCounts {
  known: boolean;
  t1: number;
  t2: number;
  t3: number;
  t4: number;
  t5: number;
}

export async function get_source_tier_distribution(): Promise<SourceTierCounts> {
  logStub('get_source_tier_distribution');
  const live = await tryInvoke<SourceTierCounts>('get_source_tier_distribution');
  if (live) return live;
  await delay();
  return { known: false, t1: 0, t2: 0, t3: 0, t4: 0, t5: 0 };
}

// C-3: raw feasibility-report log (every feasibility_report SSB row, distinct
// from get_feasibility_report which returns the v2 PASS/FAIL summary). Falls
// back to [] in browser-only mode; "no feasibility probes yet" is a legit
// steady state.
export interface FeasibilityLogRow {
  task_id: string;
  seq: number;
  ts_ms: number;
  verdict: string;
  score: number;
  skip_reason: string | null;
  novel_step_count: number;
  escalations_count: number;
  vram_margin_pct: number;
  ram_margin_pct: number;
  disk_margin_pct: number;
}

export async function get_feasibility_log(): Promise<FeasibilityLogRow[]> {
  logStub('get_feasibility_log');
  const live = await tryInvoke<FeasibilityLogRow[]>('get_feasibility_log');
  if (live) return live;
  await delay();
  return [];
}

export async function get_gg_signals(): Promise<GgSignals> {
  logStub('get_gg_signals');
  const live = await tryInvoke<GgSignals>('get_gg_signals');
  if (live) return live;
  await delay();
  return GG_SIGNALS;
}

// B-5 GAP-1: anti-rubber-stamp rejection log. Returns recent
// `auditor_rubber_stamp_rejected` SSB rows so the ReasoningDrawer can
// render "Auditor: N rubber stamps rejected this session". Falls back
// to [] in browser-only mode (no Tauri runtime); "no rejections yet"
// is a legitimate steady state (the counter shows 0).
export async function get_auditor_rubber_stamp_rejections(
  limit?: number,
): Promise<AuditorRubberStampRejectionRow[]> {
  logStub('get_auditor_rubber_stamp_rejections');
  const live = await tryInvoke<AuditorRubberStampRejectionRow[]>(
    'get_auditor_rubber_stamp_rejections',
    limit !== undefined ? { limit } : undefined,
  );
  if (live) return live;
  await delay();
  return [];
}

// ------------------------------------------------------------------
// ORCHESTRATION
// ------------------------------------------------------------------

export async function get_sandbox_status(): Promise<SandboxStatus> {
  logStub('get_sandbox_status');
  const live = await tryInvoke<SandboxStatus>('get_sandbox_status');
  if (live) return live;
  await delay();
  // Fallback mock: show A++ in browser-dev mode (no OS sandbox available)
  return {
    tier: 'A++',
    profile_count: 0,
    allowed_paths_per_agent: [
      { role: 'Executor',        path_count: 1 },
      { role: 'Auditor',         path_count: 2 },
      { role: 'ContextCompiler', path_count: 2 },
      { role: 'Researcher',      path_count: 2 },
    ],
    last_visibility_filter_hit_ms: null,
    // E-15 additive fields (browser-dev defaults).
    backend_selected: 'None',
    restore_p50_ms: null,
    snapshot_count: 0,
    uniqueness_regenerations_count: 0,
    researcher_net_allowed_hosts: ['arxiv.org', 'huggingface.co'],
    researcher_net_blocked_attempts: [],
  };
}

export async function get_evidence_gates(): Promise<EvidenceGate[]> {
  logStub('get_evidence_gates');
  const live = await tryInvoke<EvidenceGate[]>('get_evidence_gates');
  if (live) return live;
  await delay();
  return EVIDENCE_GATES;
}

export async function get_agents(): Promise<AgentEntry[]> {
  logStub('get_agents');
  const live = await tryInvoke<AgentEntry[]>('get_agents');
  if (live) return live;
  await delay();
  return ACTIVE_AGENTS;
}

export async function get_supervision_log(): Promise<SupervisionEvent[]> {
  logStub('get_supervision_log');
  const live = await tryInvoke<SupervisionEvent[]>('get_supervision_log');
  if (live) return live;
  await delay();
  return SUPERVISION_LOG;
}

export async function get_context_budget(): Promise<ContextBudget> {
  logStub('get_context_budget');
  const live = await tryInvoke<ContextBudget>('get_context_budget');
  if (live) return live;
  await delay();
  return CONTEXT_BUDGET;
}

export async function get_bsp_supersteps(): Promise<BspSuperstep[]> {
  logStub('get_bsp_supersteps');
  const live = await tryInvoke<BspSuperstep[]>('get_bsp_supersteps');
  if (live) return live;
  await delay();
  return BSP_SUPERSTEPS;
}

export async function get_session_branches(): Promise<SessionBranch[]> {
  logStub('get_session_branches');
  const live = await tryInvoke<SessionBranch[]>('get_session_branches');
  if (live) return live;
  await delay();
  return SESSION_BRANCHES;
}

// B-9: branch mutation stubs — commands exist on backend but return an error
// when no SessionDag handle is available (CLI-only operations).
export async function fork_session_branch(name: string): Promise<unknown> {
  logStub('fork_session_branch', name);
  return tryInvoke<unknown>('fork_session_branch', { name });
}

export async function merge_session_branch(name: string): Promise<unknown> {
  logStub('merge_session_branch', name);
  return tryInvoke<unknown>('merge_session_branch', { name });
}

export async function close_session_branch(name: string): Promise<void> {
  logStub('close_session_branch', name);
  await tryInvoke<void>('close_session_branch', { name });
}

// C-23: load the latest handoff snapshot from the active SessionDag.
// Returns metadata for the CommandWindow status card (no in-memory
// session rehydration happens client-side — that belongs to the CLI).
// Throws when the backend command rejects (no active session, no
// handoff snapshot, or SQLite open failed).
export interface LoadHandoffResult {
  session_id: string;
  snap_id: string;
  fsm_state: string;
  message_count: number;
  parent_chain_len: number;
}

export async function load_handoff(): Promise<LoadHandoffResult> {
  logStub('load_handoff');
  const result = await tryInvoke<LoadHandoffResult>('load_handoff');
  if (!result) {
    throw new Error('load_handoff: backend command returned no result');
  }
  return result;
}

export async function get_anti_spiral_counter(): Promise<AntiSpiralCounter> {
  logStub('get_anti_spiral_counter');
  const live = await tryInvoke<AntiSpiralCounter>('get_anti_spiral_counter');
  if (live) return live;
  await delay();
  return ANTI_SPIRAL_COUNTER;
}

export async function get_heartbeats(): Promise<HeartbeatEntry[]> {
  logStub('get_heartbeats');
  const live = await tryInvoke<HeartbeatEntry[]>('get_heartbeats');
  if (live) return live;
  await delay();
  return HEARTBEATS;
}

// ------------------------------------------------------------------
// PIPELINE / DECOMPOSITION
// ------------------------------------------------------------------

export async function get_decomposition_tree(): Promise<DecompNode> {
  logStub('get_decomposition_tree');
  const live = await tryInvoke<DecompNode>('get_decomposition_tree');
  if (live) return live;
  await delay();
  return DECOMP_TREE;
}

export async function get_bubble_up_events(): Promise<BubbleUpEvent[]> {
  logStub('get_bubble_up_events');
  const live = await tryInvoke<BubbleUpEvent[]>('get_bubble_up_events');
  if (live) return live;
  await delay();
  return BUBBLE_UP_EVENTS;
}

// B-7 STEP 7: returns the full ExecutablePipeline DAG (nodes +
// validation_errors + repair_log). The Tauri command now returns a
// structured object instead of a flat node array. Falls back to a
// mock DAG shape when the desktop shell is unavailable.
export async function get_executable_pipeline(): Promise<ExecutablePipelineDag> {
  logStub('get_executable_pipeline');
  const live = await tryInvoke<ExecutablePipelineDag>('get_executable_pipeline');
  if (live) return live;
  await delay();
  return {
    nodes: PIPELINE_NODES,
    validation_errors: DAG_REPAIR_EVENTS.map((e) => ({
      kind: e.action_kind,
      round: e.round,
      message: e.reason,
    })),
    repair_log: DAG_REPAIR_EVENTS.map((e) => ({
      action_kind: e.action_kind,
      round: e.round,
      reason: e.reason,
      escalated: false,
    })).concat(
      DAG_REPAIR_ESCALATED.map((e) => ({
        action_kind: 'escalated',
        round: e.rounds,
        reason: e.last_error,
        escalated: true,
        remaining_issues: e.remaining_issues,
      })),
    ),
  };
}

export async function get_dag_repair_log(): Promise<DagRepairEvent[]> {
  logStub('get_dag_repair_log');
  const live = await tryInvoke<DagRepairEvent[]>('get_dag_repair_log');
  if (live) return live;
  await delay();
  return DAG_REPAIR_EVENTS;
}

// B-7 STEP 6: dag_repair_escalated rows from SSB. Surfaced alongside
// get_dag_repair_log in the OrchestrationDrawer DAG REPAIR section.
export async function get_dag_repair_escalated(): Promise<DagRepairEscalated[]> {
  logStub('get_dag_repair_escalated');
  const live = await tryInvoke<DagRepairEscalated[]>('get_dag_repair_escalated');
  if (live) return live;
  await delay();
  return DAG_REPAIR_ESCALATED;
}

// B-7 (SCHEMA_VERSION 100): producer-wire surface — package_ref_verified
// rows from SSB. Used by PlanWindow concept/phase detail pane. Cold start
// returns an empty array (no mock data fallback — these are session-scoped
// outcomes; a static mock would be misleading).
export async function get_package_verifications(): Promise<PackageRefVerified[]> {
  logStub('get_package_verifications');
  const live = await tryInvoke<PackageRefVerified[]>('get_package_verifications');
  if (live) return live;
  await delay();
  return [];
}

// B-7 Task #8: PhaseSizingSplitter split-badge surface. Returns
// task_split rows that carry phase_sizing context (splitter =
// "phase_sizing", phase_id != null). Cold start returns an empty
// array (no mock data — splits are session-scoped outcomes).
export async function get_phase_sized_splits(): Promise<PhaseSizedSplit[]> {
  logStub('get_phase_sized_splits');
  const live = await tryInvoke<PhaseSizedSplit[]>('get_phase_sized_splits');
  if (live) return live;
  await delay();
  return [];
}

// B-7 Step 9: TreeNavigator state cursor. Returns recent tree_position_changed
// rows from SSB (most recent first). PlanWindow uses the latest row's
// current_node to highlight the navigator cursor. Cold start returns an empty
// array (no mock data — cursor positions are session-scoped outcomes).
export async function get_tree_positions(): Promise<TreePosition[]> {
  logStub('get_tree_positions');
  const live = await tryInvoke<TreePosition[]>('get_tree_positions');
  if (live) return live;
  await delay();
  return [];
}

// B-7 Step 10: per-baby-step failure retry history. Returns recent
// failure_recorded rows from SSB (most recent first). PlanWindow baby-step
// detail pane filters these by baby_step_id to show retry history. Cold
// start returns an empty array (no mock data — failures are session-scoped
// outcomes).
export async function get_failure_records(): Promise<FailureRecord[]> {
  logStub('get_failure_records');
  const live = await tryInvoke<FailureRecord[]>('get_failure_records');
  if (live) return live;
  await delay();
  return [];
}

// ------------------------------------------------------------------
// RESEARCH
// ------------------------------------------------------------------

export async function get_research_tree(): Promise<ResearchTreeNode> {
  logStub('get_research_tree');
  const live = await tryInvoke<ResearchTreeNode>('get_research_tree');
  if (live) return live;
  await delay();
  return RESEARCH_TREE;
}

export async function get_source_quality_tiers(): Promise<SourceTierDistribution[]> {
  logStub('get_source_quality_tiers');
  const live = await tryInvoke<SourceTierDistribution[]>('get_source_quality_tiers');
  if (live) return live;
  await delay();
  return SOURCE_TIER_DISTRIBUTION;
}

// ------------------------------------------------------------------
// PROVIDERS
// ------------------------------------------------------------------

export async function get_vram(): Promise<VramGauge[]> {
  logStub('get_vram');
  const live = await tryInvoke<VramGauge[]>('get_vram');
  if (live) return live;
  await delay();
  return VRAM_GAUGES;
}

export async function get_inference_metrics(): Promise<InferenceMetrics> {
  logStub('get_inference_metrics');
  const live = await tryInvoke<InferenceMetrics>('get_inference_metrics');
  if (live) return live;
  await delay();
  return INFERENCE_METRICS;
}

export async function get_sidecar_status(): Promise<SidecarStatus> {
  logStub('get_sidecar_status');
  const live = await tryInvoke<SidecarStatus>('get_sidecar_status');
  if (live) return live;
  await delay();
  return SIDECAR_STATUS;
}

// ------------------------------------------------------------------
// RULES
// ------------------------------------------------------------------

export async function get_rules_lifecycle(): Promise<RuleLifecycleCounts> {
  logStub('get_rules_lifecycle');
  const live = await tryInvoke<RuleLifecycleCounts>('get_rules_lifecycle');
  if (live) return live;
  await delay();
  return RULE_LIFECYCLE_COUNTS;
}

export async function get_active_rules(state: FsmState): Promise<Rule[]> {
  logStub('get_active_rules', { state });
  const live = await tryInvoke<Rule[]>('get_active_rules', { fsmState: state });
  if (live) return live;
  await delay();
  return ACTIVE_RULES.filter((r) => r.active_in_states.includes(state));
}

export async function get_deprecated_patterns(): Promise<DeprecatedPattern[]> {
  logStub('get_deprecated_patterns');
  const live = await tryInvoke<DeprecatedPattern[]>('get_deprecated_patterns');
  if (live) return live;
  await delay();
  return DEPRECATED_PATTERNS;
}

/** B-3 #11. RulesDrawer rule_transition feed. Returns the most recent N
 *  rule lifecycle transitions emitted on the SSB stream. Falls back to
 *  `[]` (not mock data) because "no transitions yet" is a legitimate
 *  empty state — the dashboard renders an explicit empty note. */
export async function get_rule_transitions(n = 5): Promise<RuleTransitionRow[]> {
  logStub('get_rule_transitions', { n });
  const live = await tryInvoke<RuleTransitionRow[]>('get_rule_transitions', { n });
  if (live) return live;
  await delay();
  return [];
}

/** B-3 #3. KnowledgeDrawer query-tier histogram. Falls back to a zeroed
 *  shape (sample_size=0 is the "no data yet" signal). */
export async function get_query_tier_stats(): Promise<QueryTierStats> {
  logStub('get_query_tier_stats');
  const live = await tryInvoke<QueryTierStats>('get_query_tier_stats');
  if (live) return live;
  await delay();
  return {
    fast_pct: 0,
    standard_pct: 0,
    deep_pct: 0,
    avg_latency_ms_fast: 0,
    avg_latency_ms_standard: 0,
    avg_latency_ms_deep: 0,
    sample_size: 0,
  };
}

/** B-3 #27/#28 + V2 rebaseline. Compile-time + runtime feature flags
 *  driving panel visibility for future-phase UI sneak-ins. Falls back to
 *  all-false so the panels stay hidden in pure-browser mode. */
export async function get_feature_flags(): Promise<FeatureFlags> {
  logStub('get_feature_flags');
  const live = await tryInvoke<FeatureFlags>('get_feature_flags');
  if (live) return live;
  // Note: stub.ts is loaded BEFORE the panel mount decision, so the
  // mock path returns "all false" to keep ProposalsPanel +
  // AutoAdvanceStats hidden in pure-browser mode (matching the V2
  // CLASS C deferral until B-22/B-23 producers land).
  return {
    autonomous_mode: false,
    self_modification: false,
    dashboard_offline: false,
  };
}

/** B-3 #19 (CLASS B, V2-rebaseline): isolation_metric renderer
 *  defer-flag. V2 has no ActivityEntry.svelte; the isolation_metric
 *  caption/orbit-signal is intentionally NOT rendered until Tier C IPC
 *  interception lands in B-18. This stub exists as a stable callable
 *  surface so any future renderer (orbit caption, drawer panel) has a
 *  one-line wire point — until then it returns the explicit deferral
 *  envelope so callers do not silently render zero rows. */
export interface IsolationMetricStatus {
  enabled: boolean;
  deferred_until: string;
  reason: string;
}
export async function get_isolation_metric_status(): Promise<IsolationMetricStatus> {
  logStub('get_isolation_metric_status');
  // Not wired to a Tauri command yet — Tier C IPC interception (B-18)
  // is the producer; until then the renderer must hide itself. We do
  // NOT call tryInvoke here because there is no command behind it; the
  // shape is the canonical "not yet" envelope.
  return {
    enabled: false,
    deferred_until: 'B-18',
    reason: 'Tier C IPC interception not yet implemented; compute_isolation_metric has no production caller',
  };
}

// ------------------------------------------------------------------
// WEB
// ------------------------------------------------------------------

export async function get_web_arag_state(): Promise<WebAragState> {
  logStub('get_web_arag_state');
  const live = await tryInvoke<WebAragState>('get_web_arag_state');
  if (live) return live;
  await delay();
  return WEB_ARAG_STATE;
}

export async function get_build_slots(): Promise<BuildSlot[]> {
  logStub('get_build_slots');
  const live = await tryInvoke<BuildSlot[]>('get_build_slots');
  if (live) return live;
  await delay();
  return _buildSlots;
}

export async function get_dev_servers(): Promise<DevServer[]> {
  logStub('get_dev_servers');
  const live = await tryInvoke<DevServer[]>('get_dev_servers');
  if (live) return live;
  await delay();
  return DEV_SERVERS;
}

// Phase A (s-69820) — WEB SCAFFOLD + DEV SERVER + FIM SECTION FILL
export async function list_scaffold_templates(): Promise<ScaffoldTemplate[]> {
  logStub('list_scaffold_templates');
  const live = await tryInvoke<ScaffoldTemplate[]>('list_scaffold_templates');
  if (live) return live;
  await delay();
  return SCAFFOLD_TEMPLATES;
}

export async function apply_scaffold_template(
  args: ApplyScaffoldArgs,
): Promise<{ ok: boolean; template: string; target_dir: string }> {
  logStub('apply_scaffold_template', args);
  const live = await tryInvoke<{ ok: boolean; template: string; target_dir: string }>(
    'apply_scaffold_template',
    args as unknown as Record<string, unknown>,
  );
  if (live) return live;
  await delay();
  return { ok: true, template: args.template, target_dir: args.target_dir };
}

export async function fim_fill_section(
  target_path: string,
  hint: string,
): Promise<{ ok: boolean; strategy: string; target_path: string; fill_chars: number }> {
  logStub('fim_fill_section', { target_path, hint });
  const live = await tryInvoke<{
    ok: boolean;
    strategy: string;
    target_path: string;
    fill_chars: number;
  }>('fim_fill_section', { target_path, hint });
  if (live) return live;
  await delay();
  return { ok: false, strategy: 'mock', target_path, fill_chars: 0 };
}

export async function run_web_build_check(
  project_dir: string,
  command: string,
  output_dir: string,
): Promise<{ success: boolean; exit_code: number; output_dir: string; stderr_errors: string[] }> {
  logStub('run_web_build_check', { project_dir, command, output_dir });
  const live = await tryInvoke<{
    success: boolean;
    exit_code: number;
    output_dir: string;
    stderr_errors: string[];
  }>('run_web_build_check', { projectDir: project_dir, command, outputDir: output_dir });
  if (live) return live;
  await delay();
  return { success: false, exit_code: -1, output_dir, stderr_errors: ['stub: not in tauri'] };
}

export async function start_dev_server_for_project(
  project_dir: string,
  command: string,
  port: number,
): Promise<{ ok: boolean; project_id: string; pid: number; port: number }> {
  logStub('start_dev_server_for_project', { project_dir, command, port });
  const live = await tryInvoke<{ ok: boolean; project_id: string; pid: number; port: number }>(
    'start_dev_server_for_project',
    { projectDir: project_dir, command, port },
  );
  if (live) return live;
  await delay();
  return { ok: false, project_id: project_dir, pid: 0, port };
}

export async function stop_dev_server_for_project(
  project_id: string,
): Promise<{ ok: boolean; project_id: string }> {
  logStub('stop_dev_server_for_project', { project_id });
  const live = await tryInvoke<{ ok: boolean; project_id: string }>(
    'stop_dev_server_for_project',
    { projectId: project_id },
  );
  if (live) return live;
  await delay();
  return { ok: false, project_id };
}

// ------------------------------------------------------------------
// Phase B (s-44680) — WEB LINT (B2) + LIGHTHOUSE / CWV (B3+B4)
//
// Tauri commands (commands.rs:4076-4144) take camelCase JS args; the Rust
// signatures use snake_case. Browser-dev fallbacks return typed empty
// values (LintFinding[] = [], LighthouseReport | null = null, boolean =
// false) so the dashboard renders cleanly without a sidecar / Chromium.
// ------------------------------------------------------------------

interface WebLintWireResponse {
  ok: boolean;
  project_dir: string;
  tool_count: number;
  findings: LintFinding[];
}

interface LighthouseWireResponse {
  ok: boolean;
  lcp_ms: number;
  cls: number;
  inp_ms: number;
  perf_score: number;
}

interface ChromiumProbeWireResponse {
  ok: boolean;
  available: boolean;
  reason?: string;
}

export async function run_web_lint_check(
  project_dir: string,
  eslint: boolean,
  stylelint: boolean,
  axe: boolean,
): Promise<LintFinding[]> {
  logStub('run_web_lint_check', { project_dir, eslint, stylelint, axe });
  const live = await tryInvoke<WebLintWireResponse>('run_web_lint_check', {
    projectDir: project_dir,
    eslint,
    stylelint,
    axe,
  });
  if (live) return live.findings;
  await delay();
  return [];
}

export async function run_lighthouse_audit(
  project_dir: string,
  url: string,
): Promise<LighthouseReport | null> {
  logStub('run_lighthouse_audit', { project_dir, url });
  const live = await tryInvoke<LighthouseWireResponse>('run_lighthouse_audit', {
    projectDir: project_dir,
    url,
  });
  if (live) {
    return {
      lcp_ms: live.lcp_ms,
      cls: live.cls,
      inp_ms: live.inp_ms,
      perf_score: live.perf_score,
    };
  }
  await delay();
  return null;
}

export async function probe_chromium_runtime(): Promise<boolean> {
  logStub('probe_chromium_runtime');
  const live = await tryInvoke<ChromiumProbeWireResponse>('probe_chromium_runtime');
  if (live) return live.available;
  await delay();
  return false;
}

// ------------------------------------------------------------------
// PROJECT INTELLIGENCE
// ------------------------------------------------------------------

export async function get_project_intelligence(): Promise<ProjectProfile> {
  logStub('get_project_intelligence');
  const live = await tryInvoke<ProjectProfile>('get_project_intelligence');
  if (live) return live;
  await delay();
  return PROJECT_PROFILE;
}

// ------------------------------------------------------------------
// MISTAKE / INTERROGATION / FAILURE TAXONOMY
// ------------------------------------------------------------------

export async function get_mistake_book(babyStepId?: string): Promise<MistakeEntry[]> {
  logStub('get_mistake_book', { babyStepId });
  const live = await tryInvoke<MistakeEntry[]>('get_mistake_book', { babyStepId });
  if (live) return live;
  await delay();
  if (babyStepId) return MISTAKE_BOOK.filter((m) => m.baby_step_id === babyStepId);
  return MISTAKE_BOOK;
}

export async function get_interrogation_tree(language: string): Promise<InterrogationNode> {
  logStub('get_interrogation_tree', { language });
  const live = await tryInvoke<InterrogationNode>('get_interrogation_tree', { language });
  if (live) return live;
  await delay();
  return {
    id: 'in-root',
    question: `What category best describes the failure in your ${language} code?`,
    choices: [
      { id: 'in-a', label: 'Incorrect logic / wrong algorithm', next_node_id: 'in-logic' },
      { id: 'in-b', label: 'Type mismatch or borrow checker error', next_node_id: 'in-type' },
      { id: 'in-c', label: 'Missing or wrong dependency', next_node_id: 'in-dep' },
      { id: 'in-d', label: 'Async / concurrency issue', next_node_id: 'in-async' },
      { id: 'in-e', label: 'Panic or unwrap in production path', next_node_id: 'in-panic' },
      { id: 'in-f', label: 'Test isolation problem', next_node_id: 'in-test' },
      { id: 'in-g', label: 'None of the above', next_node_id: undefined },
    ],
  };
}

export async function get_failure_taxonomy_last(): Promise<FailureTaxonomyResult> {
  logStub('get_failure_taxonomy_last');
  const live = await tryInvoke<FailureTaxonomyResult>('get_failure_taxonomy_last');
  if (live) return live;
  await delay();
  return {
    classification: 'ASSERTION_FAILED',
    confidence: 0.89,
    baby_step_id: 'bs-3.2.2',
    file: 'tests/revocation_tests.rs',
    line: 62,
    raw_message: 'assertion `left == right` failed\nleft: false\nright: true',
    ts: Date.now() - 30 * 60 * 1000,
  };
}

// ------------------------------------------------------------------
// BASELINE / DEP AUDIT
// ------------------------------------------------------------------

export async function get_baseline_results(): Promise<BaselineResults> {
  logStub('get_baseline_results');
  const live = await tryInvoke<BaselineResults>('get_baseline_results');
  if (live) return live;
  await delay();
  return BASELINE_RESULTS;
}

export async function get_dep_audit_results(): Promise<DepAuditResult[]> {
  logStub('get_dep_audit_results');
  const live = await tryInvoke<DepAuditResult[]>('get_dep_audit_results');
  if (live) return live;
  await delay();
  return DEP_AUDIT;
}

// ------------------------------------------------------------------
// HOOKS
// ------------------------------------------------------------------

export async function get_hook_executions(): Promise<HookExecution[]> {
  logStub('get_hook_executions');
  const live = await tryInvoke<HookExecution[]>('get_hook_executions');
  if (live) return live;
  await delay();
  return HOOK_EXECUTIONS;
}

export async function get_shortcoder_stats(): Promise<ShortCoderStats> {
  logStub('get_shortcoder_stats');
  const live = await tryInvoke<ShortCoderStats>('get_shortcoder_stats');
  if (live) return live;
  await delay();
  return SHORTCODER_STATS;
}

// ------------------------------------------------------------------
// EXPLORE
// ------------------------------------------------------------------

export interface ExploreFileRef {
  path: string;
  symbol?: string;
  line?: number;
}

export interface ExploreAnswer {
  id: string;
  query: string;
  answer: string;
  file_refs: ExploreFileRef[];
  sources_count: number;
  ts: number;
}

const EXPLORE_MOCK_ANSWERS: Record<string, ExploreAnswer> = {
  'how does the auth module validate tokens?': {
    id: 'ea-001',
    query: 'how does the auth module validate tokens?',
    answer:
      'Token validation in `crates/auth/src/lib.rs` follows a 3-step pipeline.\n\n1. **Decode header** — `jwt::decode_header(raw)` extracts the algorithm. Only `HS256` and `ES256` are accepted; anything else triggers an immediate `Err(AuthError::UnsupportedAlgorithm)`.\n\n2. **Signature verification** — `Token::validate(raw, &secret)` calls `jsonwebtoken::decode` with a `Validation` struct that enforces `leeway = 0` (strict expiry), required claim `sub`, and audience check against `config.audience`. A failed decode returns `Err(AuthError::InvalidSignature)`.\n\n3. **Revocation check** — after signature passes, `RevocationStore::get(blake3_hash(raw))` is called. If `Some(true)` (token is revoked), the middleware short-circuits with `403 Forbidden`. If the store is unavailable (Err), the policy is fail-closed: the token is rejected, not bypassed.',
    file_refs: [
      { path: 'crates/auth/src/lib.rs', symbol: 'validate_token', line: 88 },
      { path: 'crates/auth/src/token.rs', symbol: 'Token::validate', line: 34 },
    ],
    sources_count: 4,
    ts: Date.now() - 15 * 60 * 1000,
  },
};

/** EXPLORE_CODE wire — hits the local BM25 index of the active project root.
 *  Distinct from `run_explore_query`, which hits SearXNG web search via the
 *  sidecar (Oracle/RESEARCH path). Returns a typed answer with hits[] and
 *  a status discriminator the UI can switch on (ok / not_indexed / error).
 *  Browser-dev fallback returns an empty `error` status so the card still
 *  renders without crashing. */
export interface KnowledgeAnswer {
  id: string;
  query: string;
  hits: import('./types').KnowledgeHit[];
  status: 'ok' | 'not_indexed' | 'error';
  message?: string;
  ts: number;
}

export async function query_knowledge_engine(query: string): Promise<KnowledgeAnswer> {
  logStub('query_knowledge_engine', { query });
  const live = await tryInvoke<KnowledgeAnswer>('query_knowledge_engine', { query });
  if (live) return live;
  return {
    id: `ke-${Date.now()}`,
    query,
    hits: [],
    status: 'error',
    message: 'tauri not available — browser-dev mode',
    ts: Date.now(),
  };
}

export async function run_explore_query(query: string): Promise<ExploreAnswer> {
  logStub('run_explore_query', { query });
  const live = await tryInvoke<ExploreAnswer>('run_explore_query', { query });
  if (live) return live;
  await delay(120, 240);
  const lower = query.trim().toLowerCase();
  const known = EXPLORE_MOCK_ANSWERS[lower];
  if (known) return { ...known, ts: Date.now() };
  return {
    id: `ea-${Date.now()}`,
    query,
    answer: `DKG search returned 3 relevant nodes for "${query}". Run inside Tauri shell for live results.`,
    file_refs: [{ path: 'crates/orchestrator/src/lib.rs', symbol: undefined, line: 1 }],
    sources_count: 3,
    ts: Date.now(),
  };
}

export async function continue_task(
  taskId: string,
): Promise<{ ok: boolean; restoredContext: string }> {
  logStub('continue_task', { taskId });
  const live = await tryInvoke<{ ok: boolean; restoredContext: string }>('continue_task', {
    taskId,
  });
  if (live) return live;
  await delay(150, 240);
  return { ok: true, restoredContext: `Session context for task ${taskId} restored from DKG snapshot.` };
}

// ------------------------------------------------------------------
// SETTINGS / BUDGETS / EXECUTION MODE / MCP TOGGLE
// ------------------------------------------------------------------

export async function set_local_max_context(tokens?: number): Promise<{ ok: boolean }> {
  logStub('set_local_max_context', { tokens });
  const live = await tryInvoke<{ ok: boolean }>('set_local_max_context', { tokens });
  if (live) return live;
  await delay();
  return { ok: true };
}

export async function get_local_budget(): Promise<{
  local_max_context: number | null;
  local_min_prompt: boolean;
}> {
  logStub('get_local_budget');
  const live = await tryInvoke<{ local_max_context: number | null; local_min_prompt: boolean }>(
    'get_local_budget',
  );
  if (live) return live;
  await delay();
  return { local_max_context: null, local_min_prompt: false };
}

export async function set_local_min_prompt(enabled: boolean): Promise<{ ok: boolean }> {
  logStub('set_local_min_prompt', { enabled });
  const live = await tryInvoke<{ ok: boolean }>('set_local_min_prompt', { enabled });
  if (live) return live;
  await delay();
  return { ok: true };
}

export async function get_local_min_prompt(): Promise<{ enabled: boolean }> {
  logStub('get_local_min_prompt');
  const live = await tryInvoke<{ enabled: boolean }>('get_local_min_prompt');
  if (live) return live;
  await delay();
  return { enabled: false };
}

export async function set_mcp_enabled(enabled: boolean): Promise<{ ok: boolean }> {
  logStub('set_mcp_enabled', { enabled });
  const live = await tryInvoke<{ ok: boolean }>('set_mcp_enabled', { enabled });
  if (live) return live;
  await delay();
  return { ok: true };
}

export async function set_execution_mode(
  mode: 'LOCAL' | 'CLOUD' | 'HYBRID',
): Promise<{ ok: boolean; mode: string }> {
  logStub('set_execution_mode', { mode });
  const live = await tryInvoke<{ ok: boolean; mode: string }>('set_execution_mode', { mode });
  if (live) return live;
  await delay();
  return { ok: true, mode };
}

export async function get_execution_mode(): Promise<{ mode: string }> {
  logStub('get_execution_mode');
  const live = await tryInvoke<{ mode: string }>('get_execution_mode');
  if (live) return live;
  await delay();
  return { mode: 'LOCAL' };
}

// ------------------------------------------------------------------
// CLOUD AUTH (Anthropic OAuth login surface)
// ------------------------------------------------------------------

export async function get_oauth_status(): Promise<OAuthStatus> {
  logStub('get_oauth_status');
  const live = await tryInvoke<OAuthStatus>('get_oauth_status');
  if (live) return live;
  await delay();
  return {
    logged_in: false,
    display_name: null,
    expires_at: null,
    scopes: [],
    expired: false,
  };
}

export async function start_oauth_login(): Promise<OAuthLoginInfo> {
  logStub('start_oauth_login');
  const live = await tryInvoke<OAuthLoginInfo>('start_oauth_login');
  if (live) return live;
  await delay();
  return {
    auth_url: 'https://claude.ai/oauth/authorize?stub=1',
    callback_port: 4545,
    redirect_uri: 'http://localhost:4545/callback',
    event: 'oauth-login-complete',
  };
}

export async function do_oauth_logout(): Promise<{ ok: boolean }> {
  logStub('do_oauth_logout');
  const live = await tryInvoke<{ ok: boolean }>('do_oauth_logout');
  if (live) return live;
  await delay();
  return { ok: true };
}

export async function open_url_in_browser(url: string): Promise<{ ok: boolean }> {
  logStub('open_url_in_browser', { url });
  const live = await tryInvoke<{ ok: boolean }>('open_url_in_browser', { url });
  if (live) return live;
  // Browser-only fallback: just open in a new tab.
  if (typeof window !== 'undefined') window.open(url, '_blank', 'noopener');
  return { ok: true };
}

// ------------------------------------------------------------------
// CLOUD MODEL PICKER
// ------------------------------------------------------------------

export async function get_cloud_model(): Promise<CloudModelInfo> {
  logStub('get_cloud_model');
  const live = await tryInvoke<CloudModelInfo>('get_cloud_model');
  if (live) return live;
  await delay();
  return {
    model_id: 'claude-sonnet-4-6',
    allowlist: [
      'claude-opus-4-7',
      'claude-opus-4-6',
      'claude-sonnet-4-6',
      'claude-haiku-4-5-20251001',
      'claude-haiku-4-5',
    ],
    default: 'claude-sonnet-4-6',
  };
}

export async function set_cloud_model(
  modelId: string,
): Promise<{ ok: boolean; model_id: string }> {
  logStub('set_cloud_model', { modelId });
  const live = await tryInvoke<{ ok: boolean; model_id: string }>('set_cloud_model', {
    modelId,
  });
  if (live) return live;
  await delay();
  return { ok: true, model_id: modelId };
}

// ------------------------------------------------------------------
// CLOUD SPEND (B-27)
// ------------------------------------------------------------------

export async function get_cloud_spend(): Promise<CloudSpendInfo> {
  logStub('get_cloud_spend');
  const live = await tryInvoke<CloudSpendInfo>('get_cloud_spend');
  if (live) return live;
  await delay();
  // Stub default: zeroes everywhere. MonitorWindow's gauge rows are
  // conditionally hidden when both task_usd and day_usd are 0, matching
  // the live default-zero shape returned until SsbViewer aggregation
  // lands (B-27.5 follow-up).
  return {
    task_usd: 0,
    day_usd: 0,
    cache_creation_tokens: 0,
    cache_read_tokens: 0,
    redaction_count: 0,
    // I-10: default to undefined (no-data state); CloudPanel renders "--"
    // for ttl_tier when undefined.
    ttl_tier: undefined,
    breakeven_reads: undefined,
    guard_trips_today: undefined,
  };
}

// ------------------------------------------------------------------
// CLOUD ROUTE STATUS (C-5 follow-on — SCHEMA_VERSION 103)
// ------------------------------------------------------------------

export async function get_cloud_route_status(): Promise<CloudRouteStatus> {
  logStub('get_cloud_route_status');
  const live = await tryInvoke<CloudRouteStatus>('get_cloud_route_status');
  if (live !== undefined && live !== null) return live;
  await delay();
  // Stub default: all-null. CloudPanel hides the banners while every
  // field is null (no SSB row of that type written yet).
  return {
    route_blocked: null,
    warn_limit_reached: null,
    cost_estimate: null,
  };
}

// ------------------------------------------------------------------
// MUTATION GAUGE (C-28)
// ------------------------------------------------------------------

export async function get_mutation_gauge(): Promise<MutationGaugeInfo | null> {
  logStub('get_mutation_gauge');
  const live = await tryInvoke<MutationGaugeInfo | null>('get_mutation_gauge');
  if (live !== undefined) return live;
  await delay();
  // Stub default: null = no mutation run yet. MonitorWindow renders a
  // grey "no mutation run yet" placeholder for the null case.
  return null;
}

// ------------------------------------------------------------------
// COMPLETENESS RATIO (C-32) -- Code-Completeness Layer 2 gauge.
// ------------------------------------------------------------------

export async function get_completeness_ratio(): Promise<CompletenessRatioInfo> {
  logStub('get_completeness_ratio');
  const live = await tryInvoke<CompletenessRatioInfo>('get_completeness_ratio');
  if (live !== undefined && live !== null) return live;
  await delay();
  // Stub default: `{ known: false }` = no skeleton flushed yet for the
  // current session. MonitorWindow renders the grey placeholder while
  // `known` is false (audit-C32 W2 close-out).
  return { known: false };
}

export async function set_webhook_url(url: string): Promise<{ ok: boolean; webhook_url: string }> {
  logStub('set_webhook_url', { url });
  const live = await tryInvoke<{ ok: boolean; webhook_url: string }>('set_webhook_url', { url });
  if (live) return live;
  await delay();
  return { ok: true, webhook_url: url };
}

export async function get_webhook_url(): Promise<{ webhook_url: string | null }> {
  logStub('get_webhook_url');
  const live = await tryInvoke<{ webhook_url: string | null }>('get_webhook_url');
  if (live) return live;
  await delay();
  return { webhook_url: null };
}

// ------------------------------------------------------------------
// CHAT
// ------------------------------------------------------------------
// B-14 POLISH: the intent-only `submit_chat_message` wrapper was removed
// (spec stub_ban). The dashboard now streams real tokens via `stream_chat`
// below. The Tauri command is still registered (commands.rs) for SSB-row
// compatibility but has no frontend caller.
//
// The three wrappers below (`get_chat_history`, `emit_chat_response`,
// `classify_intent`) are currently uncalled from the frontend — the
// streaming path uses `stream_chat`/`cancel_stream`, and intent
// classification uses the synchronous TS-side `classifyIntent` in
// `utils/intent.ts`. They are retained as wiring-contract surface: the
// `classify_intent` Tauri command is the production caller that makes
// the orchestrator `intent_classifier::classify_intent` symbol
// reachable, and `emit_chat_response`/`get_chat_history` remain
// available for future SSB-backed card rendering.

export async function get_chat_history(limit: number): Promise<unknown[]> {
  logStub('get_chat_history', { limit });
  const live = await tryInvoke<unknown[]>('get_chat_history', { limit });
  if (live) return live;
  await delay();
  return [];
}

export async function emit_chat_response(
  cardKind: string,
  body: string,
  sessionId?: string,
): Promise<{ seq: number }> {
  logStub('emit_chat_response', { cardKind, body, sessionId });
  const live = await tryInvoke<{ seq: number }>('emit_chat_response', {
    cardKind,
    body,
    sessionId,
  });
  if (live) return live;
  await delay();
  return { seq: Date.now() };
}

export async function classify_intent(text: string): Promise<{ intent: string; text: string }> {
  logStub('classify_intent', { text });
  const live = await tryInvoke<{ intent: string; text: string }>('classify_intent', { text });
  if (live) return live;
  await delay();
  return { intent: 'AMBIGUOUS', text };
}

export async function get_recent_tasks(limit: number): Promise<unknown[]> {
  logStub('get_recent_tasks', { limit });
  const live = await tryInvoke<unknown[]>('get_recent_tasks', { limit });
  if (live) return live;
  await delay();
  return [];
}

// ------------------------------------------------------------------
// STREAMING CHAT (B-14)
// ------------------------------------------------------------------

export async function stream_chat(
  message: string,
  requestId: string,
  options?: { thinkingEnabled?: boolean; effortLevel?: string },
): Promise<{ streamId: string; offline?: boolean; reflect?: boolean }> {
  logStub('stream_chat', { message, requestId, options });
  // B-14 STEP 2: invoke the new `stream_chat` Tauri command (added in
  // WIRE-1) which streams real sidecar `/v1/messages` SSE tokens over the
  // unified `chat_event` channel. The server generates the stream_id and
  // returns it; the frontend filters `chat_event` payloads by that id.
  // B-14 wiring-close: thinkingEnabled/effortLevel are now forwarded into
  // the Tauri invoke payload; the Rust command injects them into the
  // sidecar request body (thinking block + budget tokens).
  const live = await tryInvoke<{ stream_id: string }>('stream_chat', {
    message,
    sessionId: `dash-v2-${requestId}`,
    thinkingEnabled: options?.thinkingEnabled ?? false,
    effortLevel: options?.effortLevel ?? null,
  });
  if (live) return { streamId: live.stream_id };
  // Browser-dev mock: keep the requestId as streamId so the mock drip
  // path in streamChat() stays self-consistent (no server to generate one).
  await delay();
  return { streamId: requestId };
}

export async function cancel_stream(streamId: string): Promise<{ ok: boolean }> {
  logStub('cancel_stream', { streamId });
  // B-14 STEP 2: invoke the new `cancel_stream` Tauri command which aborts
  // the detached streaming task by its server-issued stream_id. Tauri
  // camelCases the Rust `stream_id` arg to `streamId`.
  const live = await tryInvoke<{ cancelled: boolean; error?: string }>('cancel_stream', {
    streamId,
  });
  if (live) return { ok: live.cancelled };
  await delay();
  return { ok: true };
}

const MOCK_STREAM_RESPONSE =
  "I'm running in browser-dev mode (no sidecar). " +
  "Start elai with `elai.bat --remote` to connect to the GPU box and get real responses.";

/**
 * High-level helper: invokes stream_chat, subscribes to Tauri events, and
 * routes callbacks for each token, stream end, and error.
 *
 * In browser-dev mode (no Tauri), simulates token drip from MOCK_STREAM_RESPONSE.
 *
 * @returns unlisten — call this to cancel the subscription (not the stream).
 */
export async function streamChat(
  prompt: string,
  requestId: string,
  onToken: (text: string) => void,
  onEnd: (stopReason: string) => void,
  onError: (message: string, offline: boolean) => void,
  options?: { thinkingEnabled?: boolean; effortLevel?: string; onStarted?: (intent: string | null) => void },
): Promise<() => void> {
  if (!TAURI_AVAILABLE) {
    // Browser-dev mock: drip tokens with 80ms delay.
    const tokens = MOCK_STREAM_RESPONSE.split(' ');
    let cancelled = false;
    let i = 0;
    function drip() {
      if (cancelled || i >= tokens.length) {
        if (!cancelled) onEnd('end_turn');
        return;
      }
      onToken((i === 0 ? '' : ' ') + tokens[i]);
      i++;
      setTimeout(drip, 80);
    }
    setTimeout(drip, 80);
    return () => { cancelled = true; };
  }

  // B-14 STEP 2: Tauri path. Subscribe to the unified `chat_event`
  // channel BEFORE invoking stream_chat so the streaming_started / first
  // token events are not lost. The server generates the stream_id and
  // returns it from stream_chat; we filter events by that id once known.
  // Events arriving before the invoke resolves (only streaming_started,
  // which carries no content) are dropped harmlessly.
  // Dynamic import keeps the Tauri API out of browser-only bundles.
  const { listen } = await import('@tauri-apps/api/event');

  type ChatEventPayload = {
    stream_id: string;
    event_type: 'streaming_started' | 'token' | 'response_complete' | 'error';
    token?: string;
    error?: string;
    intent?: string;
  };

  let resolvedStreamId: string | null = null;
  let unlisten: (() => void) | null = null;
  let finished = false;

  function finish(errMsg?: string, offline = false) {
    if (finished) return;
    finished = true;
    if (unlisten) { unlisten(); unlisten = null; }
    if (errMsg) onError(errMsg, offline);
    else onEnd('end_turn');
  }

  unlisten = await listen<ChatEventPayload>('chat_event', (e) => {
    if (resolvedStreamId === null || e.payload.stream_id !== resolvedStreamId) return;
    switch (e.payload.event_type) {
      case 'token':
        if (e.payload.token) onToken(e.payload.token);
        break;
      case 'response_complete':
        finish();
        break;
      case 'error': {
        const msg = e.payload.error && e.payload.error.length > 0
          ? e.payload.error
          : 'stream error';
        // The new command reports sidecar connection failures as
        // "sidecar connect failed: ..."; surface those as offline so the
        // caller shows the elai.bat --remote hint.
        finish(msg, /connect|sidecar/i.test(msg));
        break;
      }
      case 'streaming_started':
        // B-14 wiring-close: surface server-side intent (from the Rust
        // classify_intent call) to the store via the optional onStarted
        // callback. The TS classifier still owns frontend routing; this
        // supplies server-side metadata (additive, nullable).
        if (options?.onStarted) options.onStarted(e.payload.intent ?? null);
        break;
    }
  });

  // Fire the stream — returns immediately with { streamId }.
  let streamId: string;
  try {
    const result = await stream_chat(prompt, requestId, options);
    streamId = result.streamId;
    resolvedStreamId = streamId;
  } catch (err) {
    finish(`stream_chat invoke failed: ${String(err)}`);
    return () => { if (unlisten) unlisten(); };
  }

  // Cleanup: unlisten + abort the inflight generation. On user-initiated
  // cancel the aborted tokio task emits no response_complete, so we call
  // onEnd here to finalize the response card (drop the typing indicator).
  // On natural completion finish() has already run and this is a no-op.
  return () => {
    if (unlisten) { unlisten(); unlisten = null; }
    if (!finished) {
      finished = true;
      cancel_stream(streamId).catch(() => {});
      onEnd('cancelled');
    }
  };
}

// ------------------------------------------------------------------
// GROUNDING CACHE
// ------------------------------------------------------------------

export async function get_grounding_cache_stats(): Promise<{
  total_keys: number;
  hits: number;
  misses: number;
  last_query_first_ms: number;
  last_query_cache_ms: number;
}> {
  logStub('get_grounding_cache_stats');
  const live = await tryInvoke<{
    total_keys: number;
    hits: number;
    misses: number;
    last_query_first_ms: number;
    last_query_cache_ms: number;
  }>('get_grounding_cache_stats');
  if (live) return live;
  await delay();
  return {
    total_keys: 0,
    hits: 0,
    misses: 0,
    last_query_first_ms: 0,
    last_query_cache_ms: 0,
  };
}

// ------------------------------------------------------------------
// SIDECAR HEALTH (D.1) — richer probe, distinct from get_sidecar_status
// ------------------------------------------------------------------

/** D.1: probes /health + /v1/models via the Tauri backend.
 *  Returns the §10.5 SidecarHealth shape.
 *  Falls back to loaded=false if the backend is unreachable or throws. */
export async function get_sidecar_health(): Promise<SidecarHealth> {
  logStub('get_sidecar_health');
  const live = await tryInvoke<SidecarHealth>('get_sidecar_health');
  if (live) return live;
  await delay();
  return {
    loaded: false,
    model: '',
    fa2: false,
    dola: false,
    steering: false,
    turboquant: 'OFF',
    vram_available: false,
    vram_total: 0,
    vram_used: 0,
    vram_free: 0,
    head_dim: 0,
    layers: 0,
    kv_quant: false,
    mcp_bridge_active: false,
    tokens_per_sec: 0,
    latency_p50: 0,
    latency_p99: 0,
    context_window: 0,
  };
}

// ------------------------------------------------------------------
// D.1 — REINDEX NOW
// ------------------------------------------------------------------

export interface ReindexResult {
  ok: boolean;
  dkg_files: number;
  dkg_nodes: number;
  dkg_edges: number;
  bm25_files: number;
  bm25_chunks: number;
  profile_loaded?: boolean;
}

export async function reindex_now(): Promise<ReindexResult> {
  logStub('reindex_now');
  const live = await tryInvoke<ReindexResult>('reindex_now');
  if (live) return live;
  await delay();
  return { ok: false, dkg_files: 0, dkg_nodes: 0, dkg_edges: 0, bm25_files: 0, bm25_chunks: 0, profile_loaded: false };
}

// ------------------------------------------------------------------
// D.2 — RUN PLANNING TASK
// ------------------------------------------------------------------

export interface PlanningTaskResult {
  ok: boolean;
  requestId?: string;
  taskId?: string;
  seq?: number;
  error?: string;
}

export async function run_planning_task(
  task: string,
  projectId?: string,
): Promise<PlanningTaskResult> {
  logStub('run_planning_task', { task, projectId });
  const live = await tryInvoke<PlanningTaskResult>('run_planning_task', {
    task,
    projectId: projectId ?? null,
  });
  if (live) return live;
  return { ok: false, error: 'tauri not available in browser mode' };
}

export async function cancel_planning_task(
  requestId: string,
): Promise<{ ok: boolean; requestId: string; reason?: string }> {
  logStub('cancel_planning_task', { requestId });
  const live = await tryInvoke<{ ok: boolean; requestId: string; reason?: string }>(
    'cancel_planning_task',
    { requestId },
  );
  if (live) return live;
  return { ok: false, requestId, reason: 'tauri not available in browser mode' };
}

// ------------------------------------------------------------------
// D.3 — RUN RESEARCH
// ------------------------------------------------------------------

export interface ResearchResult {
  ok: boolean;
  requestId?: string;
  error?: string;
}

export async function run_research(
  query: string,
  depth?: number,
  breadth?: number,
): Promise<ResearchResult> {
  logStub('run_research', { query, depth, breadth });
  const live = await tryInvoke<ResearchResult>('run_research', {
    query,
    depth: depth ?? null,
    breadth: breadth ?? null,
  });
  if (live) return live;
  return { ok: false, error: 'tauri not available in browser mode' };
}

export async function cancel_research(
  requestId: string,
): Promise<{ ok: boolean; requestId: string; reason?: string }> {
  logStub('cancel_research', { requestId });
  const live = await tryInvoke<{ ok: boolean; requestId: string; reason?: string }>(
    'cancel_research',
    { requestId },
  );
  if (live) return live;
  return { ok: false, requestId, reason: 'tauri not available in browser mode' };
}

// ------------------------------------------------------------------
// D.4 — SETTINGS READ/WRITE (elai_cc.toml)
// ------------------------------------------------------------------

export async function fetch_settings_get(): Promise<{ toml: string; path: string }> {
  logStub('fetch_settings_get');
  const live = await tryInvoke<{ toml: string; path: string }>('fetch_settings_get');
  if (live) return live;
  await delay();
  return { toml: '', path: '' };
}

export async function fetch_settings_set(tomlText: string): Promise<{ ok: boolean; path: string }> {
  logStub('fetch_settings_set', { tomlText });
  const live = await tryInvoke<{ ok: boolean; path: string }>('fetch_settings_set', { tomlText });
  if (live) return live;
  await delay();
  return { ok: false, path: '' };
}

export async function obscura_test_fetch(
  url: string,
): Promise<{ ok: boolean; html?: string; truncated?: boolean; fetch_time_ms?: number; error?: string }> {
  logStub('obscura_test_fetch', { url });
  const live = await tryInvoke<{
    ok: boolean;
    html?: string;
    truncated?: boolean;
    fetch_time_ms?: number;
    error?: string;
  }>('obscura_test_fetch', { url });
  if (live) return live;
  return { ok: false, error: 'obscura binary not found or browser unavailable' };
}

// ------------------------------------------------------------------
// D.5 — RUN D5 LIVE CONTRACT
// ------------------------------------------------------------------

/** Spawns `cargo test -p elai_cc_contracts` with ELAI_LIVE_SIDECAR=1.
 *  Returns { requestId } immediately; listen to `d5-progress:{requestId}` and
 *  `d5-complete:{requestId}` Tauri events for streaming output and exit code.
 */
export async function run_d5_live_contract(
  testName?: string,
): Promise<{ requestId: string } | null> {
  logStub('run_d5_live_contract', { testName });
  const live = await tryInvoke<{ requestId: string }>(
    'run_d5_live_contract',
    testName !== undefined ? { testName } : {},
  );
  if (live) return live;
  return null;
}

/** Aborts an in-flight run_d5_live_contract task. */
export async function cancel_d5_live_contract(
  requestId: string,
): Promise<{ ok: boolean; request_id: string; reason?: string }> {
  logStub('cancel_d5_live_contract', { requestId });
  const live = await tryInvoke<{ ok: boolean; request_id: string; reason?: string }>(
    'cancel_d5_live_contract',
    { requestId },
  );
  if (live) return live;
  return { ok: false, request_id: requestId, reason: 'invoke failed' };
}

// ------------------------------------------------------------------
// D.7 — ONE-SHOT PROMPT
// ------------------------------------------------------------------

/** B-36 STEP 5: streaming callbacks for the `one_shot_prompt` SSE events.
 *  The backend emits `streaming_started` (once, before subprocess spawn),
 *  `turn` (per stdout/stderr line), and `response` (once, on completion).
 *  Callers pass these to receive live updates; the returned cancel function
 *  unlistens all three event channels. */
export interface OneShotPromptStreamCallbacks {
  onStreamingStarted?: (payload: { request_id: string; text: string }) => void;
  onTurn?: (payload: { request_id: string; line: string }) => void;
  onResponse?: (payload: { request_id: string; ok: boolean; text: string; exit_code: number | null }) => void;
}

export async function one_shot_prompt(
  text: string,
  stream?: OneShotPromptStreamCallbacks,
): Promise<{ ok: boolean; text: string }> {
  logStub('one_shot_prompt', { text });

  // B-36 STEP 5: subscribe to the SSE event channels BEFORE invoking so
  // `streaming_started` (emitted before the subprocess spawns) is not lost.
  // Dynamic import keeps the Tauri API out of browser-only bundles.
  let unlisten: (() => void) | null = null;
  if (stream) {
    try {
      const { listen } = await import('@tauri-apps/api/event');
      const un1 = stream.onStreamingStarted
        ? await listen<{ request_id: string; text: string }>('streaming_started', (e) => stream.onStreamingStarted!(e.payload))
        : null;
      const un2 = stream.onTurn
        ? await listen<{ request_id: string; line: string }>('turn', (e) => stream.onTurn!(e.payload))
        : null;
      const un3 = stream.onResponse
        ? await listen<{ request_id: string; ok: boolean; text: string; exit_code: number | null }>('response', (e) => stream.onResponse!(e.payload))
        : null;
      unlisten = () => {
        try { un1?.(); } catch (_) { /* drop */ }
        try { un2?.(); } catch (_) { /* drop */ }
        try { un3?.(); } catch (_) { /* drop */ }
      };
    } catch {
      // Tauri event API unavailable (browser mode) — streaming callbacks
      // simply never fire; the invoke still returns the final result.
    }
  }

  const live = await tryInvoke<{ ok: boolean; text: string }>('one_shot_prompt', { text });
  if (unlisten) { try { unlisten(); } catch (_) { /* drop */ } }
  if (live) return live;
  await delay();
  return { ok: false, text: '[browser-dev: no elai.exe available]' };
}

// ------------------------------------------------------------------
// D.8 — TOOL VISIBILITY
// ------------------------------------------------------------------

export async function get_tool_visibility(): Promise<{ hidden_tools: string[] }> {
  logStub('get_tool_visibility');
  const live = await tryInvoke<{ hidden_tools: string[] }>('get_tool_visibility');
  if (live) return live;
  await delay();
  return { hidden_tools: [] };
}

// ------------------------------------------------------------------
// D.11 — SIDECAR SETTINGS + RESTART
// ------------------------------------------------------------------

export async function get_sidecar_settings(): Promise<Record<string, unknown>> {
  logStub('get_sidecar_settings');
  const live = await tryInvoke<Record<string, unknown>>('get_sidecar_settings');
  if (live) return live;
  await delay();
  return {};
}

export async function set_sidecar_settings(
  settings: Record<string, unknown>,
): Promise<{ ok: boolean }> {
  logStub('set_sidecar_settings', { settings });
  const live = await tryInvoke<{ ok: boolean }>('set_sidecar_settings', { settings });
  if (live) return live;
  await delay();
  return { ok: false };
}

export async function restart_sidecar(): Promise<{ ok: boolean; async: boolean; note?: string }> {
  logStub('restart_sidecar');
  const live = await tryInvoke<{ ok: boolean; async: boolean; note?: string }>('restart_sidecar');
  if (live) return live;
  await delay();
  return { ok: false, async: false };
}

// ------------------------------------------------------------------
// D.12 — SIDECAR SEARCH
// ------------------------------------------------------------------

export interface SidecarSearchResult {
  url: string;
  title: string;
  content: string;
  score?: number;
}

export interface SidecarSearchResponse {
  query: string;
  results: SidecarSearchResult[];
  number_of_results: number;
  error?: string;
}

export async function sidecar_search(query: string): Promise<SidecarSearchResponse> {
  logStub('sidecar_search', { query });
  const live = await tryInvoke<SidecarSearchResponse>('sidecar_search', { query });
  if (live) return live;
  await delay();
  return { query, results: [], number_of_results: 0 };
}

// ------------------------------------------------------------------
// REMOTE SIDECAR BOOT OVERLAY
// ------------------------------------------------------------------

/** Kicks off the SSH boot + tunnel + health-poll sequence on the Tauri backend.
 *  Returns { requestId } immediately. Listen to `remote-sidecar-progress`
 *  Tauri events for step updates. */
export async function start_remote_sidecar(): Promise<{ requestId: string }> {
  logStub('start_remote_sidecar');
  const live = await tryInvoke<{ requestId: string }>('start_remote_sidecar');
  if (live) return live;
  // Browser-dev mock: return a fake requestId — overlay will still show.
  return { requestId: `rs-mock-${Date.now()}` };
}

/** Reads the current boot-step progress snapshot from the Tauri AppState.
 *  Returns null before start_remote_sidecar has been called. */
export async function get_remote_sidecar_progress(): Promise<RemoteSidecarProgress | null> {
  logStub('get_remote_sidecar_progress');
  const live = await tryInvoke<RemoteSidecarProgress | null>('get_remote_sidecar_progress');
  if (live !== undefined) return live;
  return null;
}

/** Aborts an in-flight boot task by its requestId. */
export async function cancel_remote_sidecar(requestId: string): Promise<{ ok: boolean }> {
  logStub('cancel_remote_sidecar', { requestId });
  const live = await tryInvoke<{ ok: boolean }>('cancel_remote_sidecar', { requestId });
  if (live) return live;
  return { ok: true };
}

// ------------------------------------------------------------------
// REFLECT (B-39)
// ------------------------------------------------------------------

export const MOCK_REFLECT_STATE: ReflectState = {
  enabled: false,
  last_triage: null,
  last_verdict: null,
  last_summary: null,
};

export async function get_reflect_state(): Promise<ReflectState> {
  logStub('get_reflect_state');
  const live = await tryInvoke<ReflectState>('get_reflect_state');
  if (live) return live;
  await delay();
  return MOCK_REFLECT_STATE;
}

/**
 * High-level helper: starts the remote-sidecar boot, subscribes to Tauri
 * progress events, and routes each update to `onStep`.
 *
 * In browser-dev mode (no Tauri), simulates the 4-step sequence with delays.
 *
 * @returns unlisten — call this to remove the Tauri event listener (does NOT
 *          cancel the background boot task; use cancel_remote_sidecar for that).
 */
export async function bootRemoteSidecar(
  onStep: (progress: RemoteSidecarProgress) => void,
): Promise<() => void> {
  if (!TAURI_AVAILABLE) {
    // Browser-dev simulation: walk through the 4 steps with delays.
    let cancelled = false;
    const steps: Array<[string, number]> = [
      ['ssh', 1200],
      ['tunnel', 900],
      ['health', 2500],
      ['done', 0],
    ];
    (async () => {
      for (const [step, delay_ms] of steps) {
        if (cancelled) return;
        onStep({ step: step as RemoteSidecarProgress['step'], status: 'running', ts_ms: Date.now() });
        await new Promise((r) => setTimeout(r, delay_ms));
        if (cancelled) return;
        onStep({ step: step as RemoteSidecarProgress['step'], status: 'ok', ts_ms: Date.now() });
      }
    })();
    return () => { cancelled = true; };
  }

  const { listen } = await import('@tauri-apps/api/event');
  const unlisten = await listen<RemoteSidecarProgress>('remote-sidecar-progress', (e) => {
    onStep(e.payload);
  });

  await start_remote_sidecar();
  return unlisten;
}

// ============================================================
// B-24 Phase 4I Group C8 — Media validation surface.
// 4 Tauri commands + 1 event listener helper. Mock fallback
// returns deterministic in-memory data so the panel renders
// in browser-only mode without a Tauri shell.
// ============================================================

// In-memory mock store for browser-only dev mode. Tauri runtime never
// touches this — `tryInvoke` returns the live entries straight from
// AppState.media_validation.
let _mockMediaEntries: MediaValidationEntry[] = [
  {
    step_id: 'mock-step-1',
    request_id: 'mock-step-1',
    media_type: 'ui_screenshot',
    file_path: '/tmp/mock/login.png',
    tier1: {
      pass: true,
      check_count: 4,
      latency_ms: 18,
      checks: [
        { name: 'resolution', pass: true, detail: '1440x900', latency_ms: 2 },
        { name: 'ssim', pass: true, detail: 'score 0.97', latency_ms: 8 },
        { name: 'visible_pixels', pass: true, detail: '99.4%', latency_ms: 4 },
        { name: 'color_depth', pass: true, detail: '24-bit', latency_ms: 4 },
      ],
    },
    tier2: {
      advisory_pass: true,
      model_used: 'mock-vlm',
      latency_ms: 920,
      structured_output: {
        overall_quality: 0.91,
        issues: [],
        confidence: 0.88,
        reasoning: 'clean layout, no obvious overflow',
      },
    },
    render_frames: [
      { adapter_name: 'ui_screenshot', frame_count: 1, latency_ms: 14, image_path: null },
    ],
    feedback_iterations: [
      { iteration: 0, issue_count: 0, converged: true, verdict: 'Pass', issues: [] },
    ],
    final_verdict: 'Pass',
    updated_at: Date.now() - 12_000,
  },
  {
    step_id: 'mock-step-2',
    request_id: 'mock-step-2',
    media_type: 'image',
    file_path: '/tmp/mock/hero.png',
    tier1: {
      pass: true,
      check_count: 3,
      latency_ms: 22,
      checks: [],
    },
    tier2: {
      advisory_pass: false,
      model_used: 'mock-vlm',
      latency_ms: 1340,
      structured_output: {
        overall_quality: 0.62,
        issues: [
          { severity: 'major', category: 'layout', description: 'CTA button overflows container' },
        ],
        confidence: 0.74,
        reasoning: 'button width exceeds parent at desktop viewport',
      },
    },
    render_frames: [
      { adapter_name: 'web', frame_count: 2, latency_ms: 480, image_path: null },
    ],
    feedback_iterations: [
      { iteration: 0, issue_count: 1, converged: false, verdict: null, issues: ['button_overflow'] },
      { iteration: 1, issue_count: 1, converged: true, verdict: 'HumanReview', issues: ['button_overflow'] },
    ],
    final_verdict: 'HumanReview',
    updated_at: Date.now() - 4_000,
  },
];

export async function get_media_validations(): Promise<MediaValidationEntry[]> {
  logStub('get_media_validations');
  const live = await tryInvoke<MediaValidationEntry[]>('get_media_validations');
  if (live) return live;
  await delay();
  return _mockMediaEntries.slice().sort((a, b) => b.updated_at - a.updated_at);
}

export async function get_media_validation_by_step(
  stepId: string,
): Promise<MediaValidationEntry | null> {
  logStub('get_media_validation_by_step', { stepId });
  const live = await tryInvoke<MediaValidationEntry | null>('get_media_validation_by_step', {
    stepId,
  });
  if (live !== undefined) return live;
  await delay();
  return _mockMediaEntries.find((e) => e.step_id === stepId) ?? null;
}

export async function submit_media_validation_for_review(
  stepId: string,
  decision: ReviewDecision,
): Promise<{ ok: boolean; error?: string }> {
  logStub('submit_media_validation_for_review', { stepId, decision });
  if (TAURI_AVAILABLE) {
    const inv = await ensureInvoke();
    if (inv) {
      try {
        await inv<void>('submit_media_validation_for_review', { stepId, decision });
        return { ok: true };
      } catch (e) {
        return { ok: false, error: String(e) };
      }
    }
  }
  await delay();
  // Mock: patch the verdict locally so the panel reflects the decision.
  const label = decision === 'RequestRevalidate' ? 'Degraded' : decision;
  _mockMediaEntries = _mockMediaEntries.map((e) =>
    e.step_id === stepId ? { ...e, final_verdict: label, updated_at: Date.now() } : e,
  );
  return { ok: true };
}

export async function subscribe_media_ssb_events(
  onEvent: (e: MediaSsbEvent) => void,
): Promise<() => void> {
  logStub('subscribe_media_ssb_events');
  if (!TAURI_AVAILABLE) {
    // Browser-only mode: synthesize a mock event so the panel can demonstrate
    // its reactive update flow during dev. One event ~5s apart on the second
    // mock entry, repeats.
    let cancelled = false;
    let i = 0;
    const tick = () => {
      if (cancelled) return;
      onEvent({
        action: i % 2 === 0 ? 'media_tier1_complete' : 'visual_feedback_iteration',
        step_id: 'mock-step-2',
        seq: 1000 + i,
        ts_ms: Date.now(),
      });
      i += 1;
      setTimeout(tick, 5_000);
    };
    setTimeout(tick, 1_500);
    return () => {
      cancelled = true;
    };
  }
  // Tauri path: ping the backend to confirm subscriber is alive, then
  // listen on the broadcast event name. The backend already spawned the
  // SSB subscriber at startup; subscribe_media_ssb_events is a confirm
  // hook, not the actual subscription.
  const inv = await ensureInvoke();
  if (inv) {
    try {
      await inv<void>('subscribe_media_ssb_events');
    } catch (e) {
      console.warn('[stub] subscribe_media_ssb_events backend confirm failed', e);
    }
  }
  const { listen } = await import('@tauri-apps/api/event');
  const unlisten = await listen<MediaSsbEvent>('media_ssb', (e) => {
    onEvent(e.payload);
  });
  return unlisten;
}

// ------------------------------------------------------------------
// B-40 — Idea-Framer surface. stub_ban: live Tauri tryInvoke ONLY,
// no mock-data fallback. Returns null when the backend is unavailable
// or no FRAMING substate is active.
// ------------------------------------------------------------------

/**
 * Latest IdeaFramingState for the active session. Returns `null` when:
 *   * No FRAMING substate is active.
 *   * The Tauri backend is unreachable (e.g. browser-only dev mode).
 *   * The backend returned an empty Optional<IdeaFramingState>.
 *
 * WIRING_CONTRACT stub_ban: this function MUST NOT return mock data.
 * The dashboard renders an empty-state placeholder when null.
 */
export async function get_idea_framing_state(): Promise<IdeaFramingState | null> {
  logStub('get_idea_framing_state');
  const live = await tryInvoke<IdeaFramingState | null>('get_idea_framing_state');
  return live ?? null;
}

/**
 * Submit tradeoff responses captured in IdeaFramingWindow. The orchestrator
 * persists them under workflow/sessions/{sid}/idea-framing/tradeoff-responses.toml
 * and (when all required tradeoffs answered) advances FRAMING -> PLANNING.
 *
 * WIRING_CONTRACT stub_ban: live Tauri only. Returns `{ok: false}` when the
 * backend is unreachable so the UI can surface a connectivity error
 * (no silent mock-success).
 */
export async function submit_tradeoff_responses(
  responses: TradeoffResponse[],
): Promise<{ ok: boolean }> {
  logStub('submit_tradeoff_responses', { responses });
  const live = await tryInvoke<{ ok: boolean }>('submit_tradeoff_responses', {
    responses,
  });
  return live ?? { ok: false };
}

// ---------------------------------------------------------------------------
// B-35 Phase C — Context Artifacts + Entry Points
// ---------------------------------------------------------------------------

export type ArtifactRecord = {
  name: string;
};

export type ArtifactSearchResult = {
  artifact_name: string;
  relative_path: string;
  content: string;
  start_line: number;
  end_line: number;
  score: number;
};

export type EntryPointDto = {
  id: string;
  name: string;
  file: string;
  line: number | null;
  reason: string;
};

export type EntryPointsResponse = {
  entry_points: EntryPointDto[];
  summary: {
    by_reason: Record<string, number>;
    total: number;
  };
};

/**
 * List all artifact names indexed in the context database at db_path.
 * Returns { artifact_names: string[] } on success; empty list when db is missing or unreachable.
 */
export async function list_artifacts(
  db_path: string,
): Promise<{ artifact_names: string[] }> {
  logStub('list_artifacts', { db_path });
  const live = await tryInvoke<{ artifact_names: string[] }>('list_artifacts', { db_path });
  return live ?? { artifact_names: [] };
}

/**
 * Search artifact chunks by keyword (Phase C: BM25 fallback) or cosine similarity.
 * Returns { results: ArtifactSearchResult[] }.
 */
export async function search_artifacts(
  db_path: string,
  query: string,
  artifact_name?: string,
  limit?: number,
): Promise<{ results: ArtifactSearchResult[] }> {
  logStub('search_artifacts', { db_path, query, artifact_name, limit });
  const args: Record<string, unknown> = { db_path, query };
  if (artifact_name !== undefined) args.artifact_name = artifact_name;
  if (limit !== undefined) args.limit = limit;
  const live = await tryInvoke<{ results: ArtifactSearchResult[] }>('search_artifacts', args);
  return live ?? { results: [] };
}

/**
 * Trigger re-index of a single artifact. Phase C: clears chunks (full re-index via CLI).
 * WIRING_CONTRACT stub_ban: must not return mock-success when backend is unreachable.
 */
export async function index_artifacts(
  db_path: string,
  artifact_name: string,
): Promise<{ ok: boolean }> {
  logStub('index_artifacts', { db_path, artifact_name });
  const live = await tryInvoke<null>('index_artifacts', { db_path, artifact_name });
  return { ok: live !== undefined };
}

/**
 * Remove all chunks for an artifact from the context database.
 */
export async function remove_artifacts(
  db_path: string,
  artifact_name: string,
): Promise<{ artifact_name: string; chunks_removed: number }> {
  logStub('remove_artifacts', { db_path, artifact_name });
  const live = await tryInvoke<{ artifact_name: string; chunks_removed: number }>(
    'remove_artifacts',
    { db_path, artifact_name },
  );
  return live ?? { artifact_name, chunks_removed: 0 };
}

/**
 * Detect and list project entry points using the symbol graph reverse index.
 * Returns an empty list with a zeroed summary when the DKG is not yet built.
 */
export async function list_entry_points(
  include_tests?: boolean,
): Promise<EntryPointsResponse> {
  logStub('list_entry_points', { include_tests });
  const args: Record<string, unknown> = {};
  if (include_tests !== undefined) args.include_tests = include_tests;
  const live = await tryInvoke<EntryPointsResponse>('list_entry_points', args);
  return live ?? { entry_points: [], summary: { by_reason: {}, total: 0 } };
}

// ------------------------------------------------------------------
// B-42 + C-16 — COSMOS ARTIFACT TIER
// Read-only filesystem snapshot from ~/.elai/cosmos/. Empty array on
// cold-start (no cosmos root yet) — NOT mock data, a real empty state.
// `triggerCosmosRefresh` now returns a typed response with refreshed_count
// + wall-clock brackets (C-16 DECISION-D). `subscribeCosmosEvents` is the
// new live-update poll path (C-16 DECISION-C).
// ------------------------------------------------------------------

export async function getCosmosStatus(): Promise<CosmosArtifactStatus[]> {
  logStub('getCosmosStatus');
  const live = await tryInvoke<CosmosArtifactStatus[]>('get_cosmos_status');
  return live ?? [];
}

/** C-16 (DECISION-D): typed cosmos refresh response. */
export interface TriggerCosmosRefreshResponse {
  refreshed_count: number;
  started_at_ms: number;
  completed_at_ms: number;
}

/** C-16 (DECISION-C): one cosmos SSB event row. */
export interface CosmosEvent {
  seq: number;
  timestamp_ms: number;
  /** "refresh_started" | "artifact_updated" | "artifact_decayed" | "grounding_used" */
  kind: string;
  payload: { summary: string };
}

/** C-16 (DECISION-C): batch of cosmos events since last_seq. */
export interface CosmosEventBatch {
  last_seq: number;
  events: CosmosEvent[];
}

export async function triggerCosmosRefresh(
  name: string,
): Promise<{ ok: boolean; error?: string; response?: TriggerCosmosRefreshResponse }> {
  logStub('triggerCosmosRefresh', { name });
  if (!TAURI_AVAILABLE) {
    return { ok: false, error: 'tauri unavailable (browser mode)' };
  }
  const inv = await ensureInvoke();
  if (!inv) return { ok: false, error: 'tauri invoke unavailable' };
  try {
    const response = await inv('trigger_cosmos_refresh', { name }) as TriggerCosmosRefreshResponse;
    return { ok: true, response };
  } catch (e) {
    return { ok: false, error: String(e) };
  }
}

/** C-16 (DECISION-C): poll for cosmos SSB events since `lastSeq`. */
export async function subscribeCosmosEvents(
  lastSeq: number,
): Promise<CosmosEventBatch> {
  logStub('subscribeCosmosEvents', { lastSeq });
  const empty: CosmosEventBatch = { last_seq: lastSeq, events: [] };
  if (!TAURI_AVAILABLE) return empty;
  const inv = await ensureInvoke();
  if (!inv) return empty;
  try {
    const result = await inv('subscribe_cosmos_events', { lastSeq }) as CosmosEventBatch;
    return result ?? empty;
  } catch (e) {
    return empty;
  }
}

// ------------------------------------------------------------------
// B-41 — Per-level audit verdicts.
//
// LIVE Tauri invoke per brief: no mock fallback. Returns an empty array
// when the backend has no rows for the task or the IPC is unavailable.
// ------------------------------------------------------------------
export async function getPerLevelVerdicts(
  taskId: string,
): Promise<LevelAuditVerdictJson[]> {
  logStub('getPerLevelVerdicts', { taskId });
  const live = await tryInvoke<LevelAuditVerdictJson[]>('get_per_level_verdicts', {
    taskId,
  });
  return live ?? [];
}

// ------------------------------------------------------------------
// B-37 Phase 4M — Multi-runtime adapter surface.
//
// Three Tauri commands surfaced by RuntimePanel:
//   - list_runtimes() -> RuntimeInfo[]            (poll cadence ~10s)
//   - test_runtime(backend) -> TestReport         (on-demand probe)
//   - set_role_runtime(role, backend) -> void     (routing mutation)
//
// Browser-mode fallback returns a deterministic synthetic set so the
// panel renders during `pnpm dev` (no Tauri shell). The mock set
// reports `is_stub: true` for the nine adapters that report
// Unavailable in B-37A/B and `ready` for sidecar+anthropic_api.
// ------------------------------------------------------------------

const MOCK_RUNTIMES: RuntimeInfo[] = [
  {
    backend: 'sidecar',
    canonical_name: 'sidecar',
    roles: ['executor', 'researcher'],
    health: { status: 'ready' },
    is_stub: false,
    latency_ms: 42,
  },
  {
    backend: 'anthropic_api',
    canonical_name: 'anthropic_api',
    roles: ['designer', 'auditor'],
    health: { status: 'ready' },
    is_stub: false,
    latency_ms: 380,
  },
  {
    backend: 'claude_code',
    canonical_name: 'claude_code',
    roles: [],
    health: { status: 'unavailable', reason: 'stub: B-37 not implemented' },
    is_stub: true,
  },
  {
    backend: 'codex_cli',
    canonical_name: 'codex_cli',
    roles: [],
    health: { status: 'unavailable', reason: 'stub: B-37 not implemented' },
    is_stub: true,
  },
  {
    backend: 'opencode',
    canonical_name: 'opencode',
    roles: [],
    health: { status: 'unavailable', reason: 'stub: B-37 not implemented' },
    is_stub: true,
  },
  {
    backend: 'hermes',
    canonical_name: 'hermes',
    roles: [],
    health: { status: 'unavailable', reason: 'stub: B-37 not implemented' },
    is_stub: true,
  },
  {
    backend: 'gemini_cli',
    canonical_name: 'gemini_cli',
    roles: [],
    health: { status: 'unavailable', reason: 'stub: B-37 not implemented' },
    is_stub: true,
  },
  {
    backend: 'kiro_cli',
    canonical_name: 'kiro_cli',
    roles: [],
    health: { status: 'unavailable', reason: 'stub: B-37 not implemented' },
    is_stub: true,
  },
  {
    backend: 'copilot_cli',
    canonical_name: 'copilot_cli',
    roles: [],
    health: { status: 'unavailable', reason: 'stub: B-37 not implemented' },
    is_stub: true,
  },
  {
    backend: 'ollama',
    canonical_name: 'ollama',
    roles: [],
    health: { status: 'unavailable', reason: 'stub: B-37 not implemented' },
    is_stub: true,
  },
  {
    backend: 'vllm',
    canonical_name: 'vllm',
    roles: [],
    health: { status: 'unavailable', reason: 'stub: B-37 not implemented' },
    is_stub: true,
  },
];

export async function listRuntimes(): Promise<RuntimeInfo[]> {
  logStub('listRuntimes');
  const live = await tryInvoke<RuntimeInfo[]>('list_runtimes');
  if (live) return live;
  await delay();
  return MOCK_RUNTIMES.map((r) => ({ ...r, roles: [...r.roles] }));
}

export async function testRuntime(backend: RuntimeBackendId): Promise<TestReport> {
  logStub('testRuntime', { backend });
  const live = await tryInvoke<TestReport>('test_runtime', { backend });
  if (live) return live;
  await delay(120, 240);
  // Mock fallback: sidecar+anthropic_api report ready; stubs report
  // unavailable to stay honest about the B-37A/B state.
  const mock = MOCK_RUNTIMES.find((r) => r.canonical_name === backend);
  if (!mock) {
    return {
      backend,
      ok: false,
      health: { status: 'unavailable', reason: `unknown backend: ${backend}` },
      latency_ms: 0,
      error: `unknown backend: ${backend}`,
    };
  }
  const ok = mock.health.status === 'ready';
  return {
    backend,
    ok,
    health: mock.health,
    latency_ms: mock.latency_ms ?? 0,
    error: ok ? undefined : mock.health.reason,
  };
}

export async function setRoleRuntime(
  role: RuntimeAgentRole,
  backend: RuntimeBackendId,
): Promise<void> {
  logStub('setRoleRuntime', { role, backend });
  if (!TAURI_AVAILABLE) {
    await delay(60, 120);
    // Mutate the in-memory mock so subsequent listRuntimes() reflects
    // the change. Remove the role from every other backend first to
    // preserve the "one backend per role" invariant the Rust side
    // enforces via AppState.runtime_routing.
    for (const r of MOCK_RUNTIMES) {
      r.roles = r.roles.filter((x) => x !== role);
    }
    const target = MOCK_RUNTIMES.find((r) => r.canonical_name === backend);
    if (target && !target.roles.includes(role)) target.roles.push(role);
    return;
  }
  const inv = await ensureInvoke();
  if (!inv) throw new Error('tauri invoke unavailable');
  await inv<void>('set_role_runtime', { role, backend });
}

// ------------------------------------------------------------------
// E-51 — Runtime capability probe (MLX / LlamaCpp platform detect)
// ------------------------------------------------------------------

export async function runtimeCapabilityProbe(backend: string): Promise<Record<string, unknown>> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<Record<string, unknown>>('runtime_capability_probe', { backend });
}

// ------------------------------------------------------------------
// C-44 — Composer 2 runtime status surface (paid HTTP backend tier).
// Returns the adapter's current health + cumulative cost rollup
// derived from `backend_cost_accrued` SSB rows for the active
// session. Mocked when Tauri is unavailable.
// ------------------------------------------------------------------

export interface Composer2StatusReport {
  enabled: boolean;
  model: string;
  health: { status: 'ready' | 'degraded' | 'unavailable'; reason?: string };
  /** Cumulative cost in micro-dollars (1e-6 USD) for the active session. */
  cumulative_cost_micros: number;
  /** Total input tokens billed across all rolled-up rows. */
  cumulative_tokens_in: number;
  /** Total output tokens billed across all rolled-up rows. */
  cumulative_tokens_out: number;
  /** Number of invocations folded into the rollup. */
  invocations: number;
}

export async function runtimeComposer2Status(): Promise<Composer2StatusReport> {
  logStub('runtimeComposer2Status');
  const live = await tryInvoke<Composer2StatusReport>('runtime_composer2_status');
  if (live) return live;
  await delay(80, 160);
  return {
    enabled: false,
    model: 'composer-2',
    health: {
      status: 'unavailable',
      reason: 'composer2 disabled in elai_cc.toml (default opt-out)',
    },
    cumulative_cost_micros: 0,
    cumulative_tokens_in: 0,
    cumulative_tokens_out: 0,
    invocations: 0,
  };
}

// ------------------------------------------------------------------
// C-4 — Tauri commands orchestration + cloud spend surface
//
// Thin invoke() wrappers for the 9 commands added by the C-4 executor
// in src-tauri/src/commands.rs:7745..8060. Pattern mirrors
// get_executable_pipeline + get_cloud_spend: tryInvoke first, fall back
// to a graceful-empty / `known: false` stub so the dashboard renders
// in browser-only mode and during SSB cold-start.
// Argument keys are camelCase on the JS side; Tauri v2 deserializes
// them into the Rust handler's snake_case parameter names.
// ------------------------------------------------------------------

export async function get_phase_detail(phaseId: string): Promise<PhaseDetailResponse> {
  logStub('get_phase_detail', { phaseId });
  const live = await tryInvoke<PhaseDetailResponse>('get_phase_detail', { phaseId });
  if (live) return live;
  await delay();
  return { known: false, phase_id: phaseId };
}

export async function get_baby_step_detail(babyStepId: string): Promise<BabyStepDetailResponse> {
  logStub('get_baby_step_detail', { babyStepId });
  const live = await tryInvoke<BabyStepDetailResponse>('get_baby_step_detail', { babyStepId });
  if (live) return live;
  await delay();
  return { known: false, baby_step_id: babyStepId };
}

export async function get_phase_progress(): Promise<PhaseProgressRow[]> {
  logStub('get_phase_progress');
  const live = await tryInvoke<PhaseProgressRow[]>('get_phase_progress');
  if (live) return live;
  await delay();
  return [];
}

export async function get_recent_actions(
  limit?: number,
  offset?: number,
): Promise<RecentActionRow[]> {
  logStub('get_recent_actions', { limit, offset });
  const live = await tryInvoke<RecentActionRow[]>('get_recent_actions', { limit, offset });
  if (live) return live;
  await delay();
  return [];
}

export async function get_file_changes(sessionId: string): Promise<FileChangeRow[]> {
  logStub('get_file_changes', { sessionId });
  const live = await tryInvoke<FileChangeRow[]>('get_file_changes', { sessionId });
  if (live) return live;
  await delay();
  return [];
}

export async function get_action_detail(seq: number): Promise<ActionDetailResponse> {
  logStub('get_action_detail', { seq });
  const live = await tryInvoke<ActionDetailResponse>('get_action_detail', { seq });
  if (live) return live;
  await delay();
  return { known: false, seq };
}

export async function get_research_node_history(
  sessionId: string,
  nodeId: string,
): Promise<ResearchNodeHistoryRow[]> {
  logStub('get_research_node_history', { sessionId, nodeId });
  const live = await tryInvoke<ResearchNodeHistoryRow[]>('get_research_node_history', {
    sessionId,
    nodeId,
  });
  if (live) return live;
  await delay();
  return [];
}

// ------------------------------------------------------------------
// C-7 — ReviewWindow reviewer-verdict commands
//
// Thin invoke() wrappers for the 3 reviewer-verdict commands added by
// the C-7 executor in src-tauri/src/commands.rs. Each emits one
// ChangeDecision SSB row (SCHEMA_VERSION 49). `hunk_id` is omitted for
// file-level verdicts and supplied for per-hunk verdicts.
//
// On non-Tauri runtimes (browser-only dev), the wrappers fall back to a
// logStub no-op + `{ok: false}` so the dashboard renders without errors.
// ------------------------------------------------------------------

export interface ChangeDecisionResponse {
  ok: boolean;
  seq?: number;
  error?: string;
}

export async function accept_change(
  path: string,
  hunkId?: string,
): Promise<ChangeDecisionResponse> {
  logStub('accept_change', { path, hunkId });
  const live = await tryInvoke<ChangeDecisionResponse>('accept_change', {
    path,
    hunkId,
  });
  if (live) return live;
  await delay();
  return { ok: false, error: 'tauri invoke unavailable' };
}

export async function decline_change(
  path: string,
  hunkId?: string,
): Promise<ChangeDecisionResponse> {
  logStub('decline_change', { path, hunkId });
  const live = await tryInvoke<ChangeDecisionResponse>('decline_change', {
    path,
    hunkId,
  });
  if (live) return live;
  await delay();
  return { ok: false, error: 'tauri invoke unavailable' };
}

export async function retry_audit(
  path: string,
  hunkId?: string,
  note?: string,
): Promise<ChangeDecisionResponse> {
  logStub('retry_audit', { path, hunkId, note });
  const live = await tryInvoke<ChangeDecisionResponse>('retry_audit', {
    path,
    hunkId,
    note,
  });
  if (live) return live;
  await delay();
  return { ok: false, error: 'tauri invoke unavailable' };
}

export async function list_projects(): Promise<ListProjectsResponse> {
  logStub('list_projects');
  const live = await tryInvoke<ListProjectsResponse>('list_projects');
  if (live) return live;
  await delay();
  return { known: true, projects: [] };
}

export async function create_project(
  name: string,
  parentPath: string,
): Promise<ProjectEntry> {
  logStub('create_project', { name, parentPath });
  const live = await tryInvoke<ProjectEntry>('create_project', { name, parentPath });
  if (live) return live;
  await delay();
  // Browser-only fallback: synthesize an entry so the picker can show
  // the user something. Real fs writes only happen via the live Tauri
  // command path above.
  return {
    name,
    path: `${parentPath}/${name}`,
    last_opened_ms: Date.now(),
  };
}

// ------------------------------------------------------------------
// C-40 — GitHub PR adapter (gh CLI shell-out)
// cross-lang-pair: dashboard/tauri/api
//
// Five exports mirror the Tauri commands declared in
// `dashboard-v2/src-tauri/src/commands_c40.rs`. Each returns either the
// live Tauri response or a mock fallback so the dashboard renders even
// without an active SSB writer.
// ------------------------------------------------------------------

export interface PrOpenArgs {
  repo?: string;
  title: string;
  body?: string;
  head: string;
  base?: string;
  draft?: boolean;
}

export interface PrSummaryDto {
  number: number;
  title: string;
  state: 'open' | 'closed' | 'merged' | 'unknown';
  url: string;
  head_ref_name: string;
  base_ref_name: string;
  is_draft: boolean;
}

export interface PrCheckRunDto {
  name: string;
  status: string;
  conclusion: string;
}

export interface PrStatusDto {
  state: 'open' | 'closed' | 'merged' | 'unknown';
  review_decision: string;
  ci_rollup_label: string;
  review_state_label: string;
  check_runs: PrCheckRunDto[];
}

export interface ActivePrUrlDto {
  url: string | null;
  pr_number: number | null;
  ts_ms: number | null;
}

export async function github_pr_open(args: PrOpenArgs): Promise<PrSummaryDto> {
  logStub('github_pr_open', args);
  const live = await tryInvoke<PrSummaryDto>('github_pr_open', { args });
  if (live) return live;
  await delay();
  // Mock fallback: synthesize a draft-PR-style row so the dashboard
  // renders. Number 0 is the sentinel for "no live PR".
  return {
    number: 0,
    title: args.title,
    state: 'unknown',
    url: '',
    head_ref_name: args.head,
    base_ref_name: args.base ?? 'main',
    is_draft: args.draft ?? false,
  };
}

export async function github_pr_list(
  repo?: string,
  stateFilter?: string,
  limit?: number,
): Promise<PrSummaryDto[]> {
  logStub('github_pr_list', { repo, stateFilter, limit });
  const live = await tryInvoke<PrSummaryDto[]>('github_pr_list', {
    repo,
    stateFilter,
    limit,
  });
  if (live) return live;
  await delay();
  return [];
}

export async function github_pr_status(
  prNumber: number,
  repo?: string,
): Promise<PrStatusDto> {
  logStub('github_pr_status', { prNumber, repo });
  const live = await tryInvoke<PrStatusDto>('github_pr_status', {
    repo,
    prNumber,
  });
  if (live) return live;
  await delay();
  return {
    state: 'unknown',
    review_decision: '',
    ci_rollup_label: 'none',
    review_state_label: 'none',
    check_runs: [],
  };
}

export async function github_pr_close(
  prNumber: number,
  repo?: string,
): Promise<{ pr_number: number; closed: boolean }> {
  logStub('github_pr_close', { prNumber, repo });
  const live = await tryInvoke<{ pr_number: number; closed: boolean }>(
    'github_pr_close',
    { repo, prNumber },
  );
  if (live) return live;
  await delay();
  return { pr_number: prNumber, closed: false };
}

export async function github_pr_get_active_url(): Promise<ActivePrUrlDto> {
  logStub('github_pr_get_active_url');
  const live = await tryInvoke<ActivePrUrlDto>('github_pr_get_active_url');
  if (live) return live;
  await delay();
  return { url: null, pr_number: null, ts_ms: null };
}

// ============================================================================
// E-41: PR review surface (mirror of crates/orchestrator/src/github_pr/
// review.rs + comment_watcher.rs + package_verify.rs + semgrep_gate.rs +
// auto_merge_guard.rs). Reads the last N SSB rows for each variant so the
// GithubWindow PR Review section renders without needing a live gh call.
// ============================================================================

export interface PrReviewRecentDto {
  pr_number_hash: number;
  comment_kind_hash: number;
  comment_kind_label: string;
  ts_ms: number;
}

export interface PrCommentWatchedDto {
  pr_number_hash: number;
  new_comments_count: number;
  ts_ms: number;
}

export interface PrPackageVerifyResultDto {
  pr_number_hash: number;
  slopsquat_count: number;
  ts_ms: number;
}

export interface PrSemgrepGateResultDto {
  pr_number_hash: number;
  findings_count: number;
  severity_max_hash: number;
  ts_ms: number;
}

export interface PrAutoMergeBlockedDto {
  pr_number_hash: number;
  block_reason_hash: number;
  ts_ms: number;
}

export interface PrReviewStatusDto {
  recent_reviews: PrReviewRecentDto[];
  last_watcher_tick: PrCommentWatchedDto | null;
  last_slopsquat: PrPackageVerifyResultDto | null;
  last_semgrep: PrSemgrepGateResultDto | null;
  last_auto_merge_block: PrAutoMergeBlockedDto | null;
}

export async function pr_review_status(): Promise<PrReviewStatusDto> {
  logStub('pr_review_status');
  const live = await tryInvoke<PrReviewStatusDto>('pr_review_status');
  if (live) return live;
  await delay();
  return {
    recent_reviews: [],
    last_watcher_tick: null,
    last_slopsquat: null,
    last_semgrep: null,
    last_auto_merge_block: null,
  };
}

// ============================================================================
// C-47: Case-based calibration of tool-use telemetry.
// Mirrors the Rust DTOs in `dashboard-v2/src-tauri/src/commands_c47.rs`.
// Paper grounding: arXiv:2605.15041 (CAST). See
// workflow/papers/case-based-calibration.md.
// ============================================================================

export interface ToolCalibrationStatusDto {
  tool_name: string;
  shape_count: number;
  total_calls: number;
  error_count: number;
  last_ts_ms: number;
}

export interface ToolCalibrationRowDto {
  tool_name_hash: string;
  call_shape_hash: string;
  action_type: string;
  ts_ms: number;
  post_call_audit_ok: boolean | null;
  error_rate: number | null;
  total_calls: number | null;
}

export async function tool_calibration_status(): Promise<
  ToolCalibrationStatusDto[]
> {
  logStub('tool_calibration_status');
  const live = await tryInvoke<ToolCalibrationStatusDto[]>(
    'tool_calibration_status',
  );
  if (live) return live;
  await delay();
  return [];
}

export async function list_tool_calibration_rows(
  windowMs?: number,
  limit?: number,
): Promise<ToolCalibrationRowDto[]> {
  logStub('list_tool_calibration_rows', { windowMs, limit });
  const live = await tryInvoke<ToolCalibrationRowDto[]>(
    'list_tool_calibration_rows',
    { windowMs, limit },
  );
  if (live) return live;
  await delay();
  return [];
}

// ---------------------------------------------------------------------------
// E-12 Tier-B — Production fullstack codegen surface
// ---------------------------------------------------------------------------

export type FrameworkProfileDto = {
  project_class: string;
  profile_name: string;
  version_pin: string;
  language: string;
  package_manager: string;
  lint_config: string;
  test_runner: string;
  security_checks: string[];
  runpod_pending: boolean;
};

export type ProductionChecklistRunDto = {
  profile_name: string;
  target_path: string;
  rows: Array<{
    check: string;
    result:
      | { kind: 'ok'; findings: Array<unknown> }
      | { kind: 'toolchain_missing'; tool: string; install_hint: string }
      | { kind: 'error'; details: string; stderr_excerpt: string };
  }>;
  runpod_pending: boolean;
};

/**
 * E-12 Tier-B — resolve FrameworkProfile for a ProjectClass.
 * Wire-dark posture: returns the orchestrator's
 * `FrameworkProfile::generic()` fallback when no manifest matches,
 * NEVER mock data.
 */
export async function get_framework_profile(
  projectClass: string,
): Promise<FrameworkProfileDto | null> {
  logStub('get_framework_profile', { projectClass });
  const live = await tryInvoke<FrameworkProfileDto | null>(
    'get_framework_profile',
    { projectClass },
  );
  return live ?? null;
}

/**
 * E-12 Tier-B — run the 15-runner production checklist against a target
 * path. Live invocation of the 15 external CLIs is RUNPOD-PENDING — on
 * the local dev host every runner returns
 * `{kind: 'toolchain_missing', ...}`.
 */
export async function run_production_checklist(
  projectClass: string,
  targetPath: string,
): Promise<ProductionChecklistRunDto | null> {
  logStub('run_production_checklist', { projectClass, targetPath });
  const live = await tryInvoke<ProductionChecklistRunDto | null>(
    'run_production_checklist',
    { projectClass, targetPath },
  );
  return live ?? null;
}

// ------------------------------------------------------------------
// E-40 — Orchestrator-rooted symbol-graph dashboard surface.
// Three read-only Tauri commands feed the new SymbolGraphWindow.
// Reads .claude/tools/elai_graph.json on the Rust side. Mock-mode
// fallback returns empty results / zeroed status so the window
// degrades to a "no graph mounted" empty state. See
// workflow/reasoning/E-40-decisions.md.
// ------------------------------------------------------------------

export interface SymbolMatchDto {
  id: string;
  name: string;
  file: string;
  line: number;
  kind: string;
}

export interface RankedSymbolDto {
  id: string;
  score: number;
}

export interface SymbolGraphQueryReport {
  kind: string;
  symbol: string;
  result_count: number;
  results: SymbolMatchDto[];
  graph_path: string;
  node_count: number;
}

export interface SymbolGraphPageRankReport {
  seed: string;
  k: number;
  ranked: RankedSymbolDto[];
  graph_path: string;
  node_count: number;
}

export interface SymbolGraphStatusReport {
  graph_path: string;
  graph_present: boolean;
  node_count: number;
  lru_capacity: number;
}

export async function symbolGraphQuery(
  symbol: string,
  kind: string,
): Promise<SymbolGraphQueryReport> {
  logStub('symbolGraphQuery', { symbol, kind });
  const live = await tryInvoke<SymbolGraphQueryReport>('symbol_graph_query', {
    symbol,
    kind,
  });
  if (live) return live;
  await delay(60, 120);
  return {
    kind,
    symbol,
    result_count: 0,
    results: [],
    graph_path: '.claude/tools/elai_graph.json',
    node_count: 0,
  };
}

export async function symbolGraphPageRank(
  seed: string,
  k: number,
): Promise<SymbolGraphPageRankReport> {
  logStub('symbolGraphPageRank', { seed, k });
  const live = await tryInvoke<SymbolGraphPageRankReport>(
    'symbol_graph_pagerank',
    { seed, k },
  );
  if (live) return live;
  await delay(60, 120);
  return {
    seed,
    k,
    ranked: [],
    graph_path: '.claude/tools/elai_graph.json',
    node_count: 0,
  };
}

export async function symbolGraphStatus(): Promise<SymbolGraphStatusReport> {
  logStub('symbolGraphStatus');
  const live = await tryInvoke<SymbolGraphStatusReport>('symbol_graph_status');
  if (live) return live;
  await delay(60, 120);
  return {
    graph_path: '.claude/tools/elai_graph.json',
    graph_present: false,
    node_count: 0,
    lru_capacity: 1000,
  };
}

// ---------------------------------------------------------------------------
// E-23 — developer-discourse positioning + UX-guard. Two wire-dark Tauri
// commands the ContextStatePanel + TrustTelemetryPanel poll. Both return a
// zeroed shape when the orchestrator gate ELAI_DEVELOPER_DISCOURSE_ENABLE is
// unset (or the backend cannot reach SSB / FSM state) so the dashboard
// always renders a sane grid rather than blocking on a Tauri rejection.
// ---------------------------------------------------------------------------

export async function get_context_state(): Promise<import('./types').ContextState> {
  logStub('get_context_state');
  const live = await tryInvoke<import('./types').ContextState>('get_context_state');
  if (live) return live;
  await delay(50, 120);
  return {
    fsm_state: 'IDLE',
    substate: null,
    token_budget_pct: 0,
    mcp_think_topics: [],
    last_precompact_thresholds: [],
    prompt_injection_check_status: 'not_run',
    runtime_choice_badge: '',
    current_role: null,
    current_runtime: null,
  };
}

export async function get_trust_telemetry(): Promise<import('./types').TrustReport> {
  logStub('get_trust_telemetry');
  const live = await tryInvoke<import('./types').TrustReport>('get_trust_telemetry');
  if (live) return live;
  await delay(50, 120);
  return {
    tasks_per_hour: 0,
    findings_per_task: 0,
    revert_rate: 0,
    ungrounded_count: 0,
    stagnation_dist: {},
  };
}

// ---------------------------------------------------------------------------
// C-41 — Application Skills System (sub-phase 5B).
//
// Two wire-dark Tauri commands the SkillsPanel polls. When the Tauri
// backend is unavailable (browser-only Vite dev), fall back to the 9
// canonical manifests so the panel renders the static skill list. The
// routing-event stub returns an empty array — fall-through visualization
// only makes sense when a real producer is wired.
// ---------------------------------------------------------------------------

const C41_STUB_SKILLS: SkillManifest[] = [
  {
    name: 'simple_query',
    description: 'Answer a short factual question with no planning, no decomposition, no audit cycle.',
    type: 'application',
    trigger_examples: ['what is', 'define', 'explain briefly', 'lookup'],
    estimated_minutes: 1,
    tags: ['simple', 'factual', 'no-plan', 'fast-path'],
    version: '1.0.0',
    scope: 'query',
  },
  {
    name: 'focused_edit',
    description: 'Targeted single-file edit with a scoped auditor run only.',
    type: 'application',
    trigger_examples: ['fix this', 'change this line', 'rename this', 'patch'],
    estimated_minutes: 10,
    tags: ['edit', 'single-file', 'scoped-audit'],
    version: '1.0.0',
    scope: 'edit',
  },
  {
    name: 'heavy_plan',
    description: 'Multi-week SaaS or app feature decomposition with the full FSM cycle.',
    type: 'application',
    trigger_examples: ['build a saas', 'implement a feature', 'design a system'],
    estimated_minutes: 240,
    tags: ['plan', 'multi-phase', 'full-fsm'],
    version: '1.0.0',
    scope: 'plan',
  },
  {
    name: 'idea_intake',
    description: 'Layer A intake protocol — frame a raw idea into a decomposition tree.',
    type: 'application',
    trigger_examples: ['the boss wants', 'we need', 'idea:', 'brief:'],
    estimated_minutes: 20,
    tags: ['intake', 'framing', 'decomposition'],
    version: '1.0.0',
    scope: 'planning',
  },
  {
    name: 'oracle',
    description: 'Web-grounded research with citations, read-only output.',
    type: 'application',
    trigger_examples: ['/oracle', '/ask', 'how does', 'compare'],
    estimated_minutes: 5,
    tags: ['research', 'web-grounded', 'read-only'],
    version: '1.0.0',
    scope: 'research',
  },
  {
    name: 'planning_workflow',
    description: 'Resume, continue, or execute an existing phased plan.',
    type: 'application',
    trigger_examples: ['continue phase', 'resume plan', 'run plan'],
    estimated_minutes: 60,
    tags: ['plan', 'resume', 'fsm'],
    version: '1.0.0',
    scope: 'planning',
  },
  {
    name: 'paper_ingest',
    description: 'arXiv or DOI paper ingested into RAG with an interface map.',
    type: 'application',
    trigger_examples: ['ingest paper', 'arxiv:', 'cite this paper'],
    estimated_minutes: 15,
    tags: ['rag', 'paper', 'ingest'],
    version: '1.0.0',
    scope: 'research',
  },
  {
    name: 'source_ingest',
    description: 'SDK or repository ingested with an interface map and reliability tag.',
    type: 'application',
    trigger_examples: ['ingest source', 'ingest sdk', 'map this repo'],
    estimated_minutes: 30,
    tags: ['rag', 'source', 'ingest'],
    version: '1.0.0',
    scope: 'research',
  },
  {
    name: 'benchmark_run',
    description: 'Run a benchmark suite (SWE-bench, polyglot, HumanEval, etc) batched.',
    type: 'application',
    trigger_examples: ['run swe-bench', 'run benchmark', 'eval suite'],
    estimated_minutes: 120,
    tags: ['benchmark', 'eval', 'batch'],
    version: '1.0.0',
    scope: 'evaluation',
  },
];

export async function list_application_skills(): Promise<SkillManifest[]> {
  logStub('list_application_skills');
  const live = await tryInvoke<SkillManifest[]>('list_application_skills');
  if (live) return live;
  await delay(50, 120);
  return [...C41_STUB_SKILLS];
}

export async function recent_skill_routing_events(
  limit: number,
): Promise<SkillRoutingEvent[]> {
  logStub('recent_skill_routing_events', { limit });
  const live = await tryInvoke<SkillRoutingEvent[]>('recent_skill_routing_events', {
    limit,
  });
  if (live) return live;
  await delay(50, 120);
  return [];
}

// F-6 / A5 — learned-skill library Trust-Ladder surface. Merges the three
// SkillLibraryEvent arms into a newest-first timeline + a derived per-skill
// current-tier ladder. Falls back to an empty response off the desktop shell.
export async function get_skill_library(
  limit?: number,
): Promise<SkillLibraryResponse> {
  logStub('get_skill_library', { limit });
  const live = await tryInvoke<SkillLibraryResponse>('get_skill_library', {
    limit,
  });
  if (live) return live;
  await delay(50, 120);
  return { events: [], ladder: [] };
}

// F-5 / A5 — staleness-quarantine cascade surface. Returns the newest-first
// staleness_quarantine_broadcast row stream (fact_id + staleness ratio +
// dependents demoted + Cosmos artifacts flagged + dropped_count). Falls back to
// an empty response off the desktop shell.
export async function get_staleness_quarantine(
  limit?: number,
): Promise<StalenessQuarantineResponse> {
  logStub('get_staleness_quarantine', { limit });
  const live = await tryInvoke<StalenessQuarantineResponse>(
    'get_staleness_quarantine',
    { limit },
  );
  if (live) return live;
  await delay(50, 120);
  return { rows: [] };
}

// F-7 / A5 — template-ranking (Laplace utility) surface. Returns the
// newest-first template_utility_ranked row stream (concept_class +
// plan_skeleton_hash + smoothed utility + raw first_try_ratio + sample_n +
// chosen_pipeline_id). Falls back to an empty response off the desktop shell.
export async function get_template_ranking(
  limit?: number,
): Promise<TemplateRankingResponse> {
  logStub('get_template_ranking', { limit });
  const live = await tryInvoke<TemplateRankingResponse>(
    'get_template_ranking',
    { limit },
  );
  if (live) return live;
  await delay(50, 120);
  return { rows: [] };
}

// ============================================================
// E-25 Tranche N — Compaction-Ladder dashboard.
//
// Pairs with crates/dashboard-v2/src-tauri/src/commands_e25.rs.
// Producer is the orchestrator's task_runner PreCompact branch,
// gated by ELAI_E25_COMPACTION_LADDER_ENABLE. The browser-only
// fallback (no Tauri) returns a wire-dark default state + zero
// events so the panel renders the "no events recorded yet" empty
// state instead of throwing.
// ============================================================

const E25_STUB_TIERS: string[] = [
  'budget_reduction',
  'snip',
  'microcompact',
  'context_collapse',
  'auto_compact',
  'narrative_summary',
];

export async function get_compaction_ladder_state(): Promise<CompactionLadderStateDto> {
  logStub('get_compaction_ladder_state');
  const live = await tryInvoke<CompactionLadderStateDto>('get_compaction_ladder_state');
  if (live) return live;
  await delay(50, 120);
  return {
    enabled: false,
    default_target_tokens: 20_000,
    tiers: [...E25_STUB_TIERS],
  };
}

// ------------------------------------------------------------------
// F-4 Slice B-5 — TestIntegrityEvent surface for AuditTimelineWindow.
//
// Live Tauri invoke per brief: no mock fallback. Returns an empty array
// when the backend has no rows for the task or the IPC is unavailable.
// Command name "get_test_integrity_events" MUST match the Rust
// #[tauri::command] fn name exactly (Tauri snake_case convention).
// ------------------------------------------------------------------
export async function getTestIntegrityEvents(
  taskId: string,
): Promise<TestIntegrityEventJson[]> {
  logStub('getTestIntegrityEvents', { taskId });
  const live = await tryInvoke<TestIntegrityEventJson[]>('get_test_integrity_events', {
    taskId,
  });
  return live ?? [];
}

export async function recent_compaction_events(
  limit: number,
): Promise<CompactionEventDto[]> {
  logStub('recent_compaction_events', { limit });
  const live = await tryInvoke<CompactionEventDto[]>('recent_compaction_events', {
    limit,
  });
  if (live) return live;
  await delay(50, 120);
  return [];
}

// ------------------------------------------------------------------
// H-11 (SCHEMA_VERSION 220) — scoped filesystem surface + canvas
// activity overlay read path.
//
// Five exports mirror the Tauri commands declared in
// `dashboard-v2/src-tauri/src/commands_h11.rs`. ALL paths are
// canonicalized + scope-checked server-side; the client must NOT
// attempt to enforce the scope.
//
// cross-lang-pair: dashboard/tauri/api
// ------------------------------------------------------------------

/** A single filesystem entry returned by read_dir. */
export interface FsEntry {
  name: string;
  path: string;
  kind: 'file' | 'dir';
  size: number | null;
}

/** A recent canvas activity event returned by get_canvas_activity. */
export interface CanvasActivityEventDto {
  kind: 'reading' | 'editing' | 'validating' | 'researching';
  target_node_id: string;
  file_path: string;
  symbol_name: string | null;
  mode: string;
  label: string | null;
  task_id: string;
  ts: number;
}

/**
 * H-11: list directory entries within the project root.
 * `root` is the absolute project root path.
 * `relPath` is relative to root ("" or "." for the root itself).
 * Returns dirs first (alphabetical), then files (alphabetical).
 */
export async function read_dir(root: string, relPath: string): Promise<FsEntry[]> {
  logStub('read_dir', { root, relPath });
  const live = await tryInvoke<FsEntry[]>('read_dir', { root, relPath });
  if (live) return live;
  await delay(50, 120);
  return [];
}

/**
 * H-11: read a text file scoped to project root.
 * `path` must be an absolute path within a registered root.
 * Binary files should not be read this way.
 */
export async function read_file(path: string): Promise<string> {
  logStub('read_file', { path });
  const live = await tryInvoke<string>('read_file', { path });
  if (live !== null && live !== undefined) return live;
  await delay(50, 120);
  return '';
}

/**
 * H-11: open a path in the system default editor (non-blocking).
 * Scoped to project_root.
 */
export async function open_in_editor(path: string): Promise<void> {
  logStub('open_in_editor', { path });
  await tryInvoke<void>('open_in_editor', { path });
}

/**
 * H-11: register an additional project root (multi-root workspace support).
 * Does not persist across restarts in Tier-1 scope.
 */
export async function mount_root(path: string): Promise<void> {
  logStub('mount_root', { path });
  await tryInvoke<void>('mount_root', { path });
}

/**
 * H-11: return the N most recent canvas_activity_event SSB rows for the
 * canvasActivity.svelte.ts overlay subscriber.
 * `limit` defaults to 20, capped at 100 server-side.
 */
export async function get_canvas_activity(limit?: number): Promise<CanvasActivityEventDto[]> {
  logStub('get_canvas_activity', { limit });
  const live = await tryInvoke<CanvasActivityEventDto[]>('get_canvas_activity', {
    limit: limit ?? 20,
  });
  if (live) return live;
  await delay(50, 120);
  return [];
}

// ---------------------------------------------------------------------------
// H-13: canvas op stream + screenshot
// ---------------------------------------------------------------------------

/** H-13: a recent canvas op event returned by get_canvas_ops.
 *  Mirrors Rust CanvasOpDto (commands_h13.rs) + CanvasOpPayload in types.ts.
 *  Cross-lang pair: commands_h13.rs::CanvasOpDto.
 */
export interface CanvasOpDto {
  /** Op kind wire label (e.g. "node_created", "edge_connected"). */
  kind: string;
  /** Primary node id affected (empty for edge/annotation ops). */
  node_id: string;
  /** Flexible per-op JSON payload. */
  op_data: Record<string, unknown>;
  /** Canvas mode at emission time. */
  mode: string;
  /** Canonical task id. */
  task_id: string;
  /** Session id for correlation. */
  session_id: string;
  /** SSB row timestamp (unix ms). */
  ts: number;
}

/**
 * H-13: return the N most recent canvas_op_applied SSB rows for the
 * CanvasPanel.svelte op-application loop.
 * `limit` defaults to 20, capped at 100 server-side.
 */
export async function get_canvas_ops(limit?: number): Promise<CanvasOpDto[]> {
  logStub('get_canvas_ops', { limit });
  const live = await tryInvoke<CanvasOpDto[]>('get_canvas_ops', {
    limit: limit ?? 20,
  });
  if (live) return live;
  await delay(50, 120);
  return [];
}

/**
 * H-13: trigger a canvas screenshot capture.
 * Tier-2 stub: Tauri-native capture is not available in tauri 2.11.
 * CanvasPanel.svelte should use canvas.toDataURL() as the client-side fallback.
 * Returns null when the Tauri command is not available (expected in Tier-1).
 */
export async function capture_canvas_screenshot(
  mode?: string,
  taskId?: string,
): Promise<Uint8Array | null> {
  logStub('capture_canvas_screenshot', { mode, taskId });
  // Tier-2 stub: the Tauri command always returns Err in this release.
  // Return null so CanvasPanel falls through to canvas.toDataURL().
  return null;
}

// ---------------------------------------------------------------------------
// H-15: canvas view reply channel + taint-gated Select
// ---------------------------------------------------------------------------

/** H-15: a recent canvas view event returned by get_canvas_views.
 *  Mirrors Rust CanvasViewEventDto (commands_h15.rs) + CanvasViewPayload in ssb.rs.
 *  Cross-lang pair: commands_h15.rs::CanvasViewEventDto.
 *  DRAIN SEPARATION: canvas_view_event rows only (never mixed with canvas_op_applied).
 */
export interface CanvasViewEventDto {
  /** View event kind wire label ("view_spawned", "ui_action", "state_checkpoint"). */
  kind: string;
  /** Renderer view_type key (populated for view_spawned; empty for others). */
  view_type: string;
  /** Stable view instance identifier for per-tab filtering and reconstruction. */
  view_id: string;
  /** Seed op ids (view_spawned only; empty for ui_action / state_checkpoint). */
  seed_op_ids: string[];
  /** Selected node id (ui_action only; empty otherwise). */
  node_id: string;
  /** True if the G-1 taint gate flagged this node's content at Select handoff. */
  taint_flagged: boolean;
  /** Canonical task id. */
  task_id: string;
  /** Session id for correlation. */
  session_id: string;
  /** SSB row sequence number — the monotonic rebuild key for snapshot/delta reconstruction. */
  seq: number;
  /** SSB row timestamp (unix ms). */
  ts: number;
}

/** H-15: response from canvas_select_node — node content + provenance verdict. */
export interface CanvasSelectNodeResponse {
  /** The selected node id. */
  node_id: string;
  /** The node content. Check taint_flagged and allowed before passing to the model. */
  content: string;
  /** True if the G-1 taint gate flagged this content (provenance flag required). */
  taint_flagged: boolean;
  /** Taint source label if flagged (empty if clean). */
  taint_source: string;
  /** False if the gate BLOCKED the handoff (TrustLevel::Untrusted). */
  allowed: boolean;
  /** Human-readable block reason (empty if allowed=true). */
  block_reason: string;
}

/**
 * H-15: return the N most recent canvas_view_event SSB rows for the
 * canvasViewState.svelte.ts snapshot/delta reconstruction loop.
 * `limit` defaults to 20, capped at 100 server-side.
 * DRAIN SEPARATION: polls canvas_view_event ONLY (never canvas_op_applied).
 */
export async function get_canvas_views(limit?: number): Promise<CanvasViewEventDto[]> {
  logStub('get_canvas_views', { limit });
  const live = await tryInvoke<CanvasViewEventDto[]>('get_canvas_views', {
    limit: limit ?? 20,
  });
  if (live) return live;
  await delay(50, 120);
  return [];
}

/**
 * H-15: inbound CanvasUiAction::Select — taint-gated node content handoff.
 * Runs G-1 taint gate; emits canvas_view_event(UiAction) SSB row.
 * Check response.allowed before passing content to the model.
 * A tainted node's content MUST NOT silently re-enter the prompt (CANVAS §19).
 */
export async function canvas_select_node(
  nodeId: string,
  content: string,
  taskId?: string,
): Promise<CanvasSelectNodeResponse | null> {
  logStub('canvas_select_node', { nodeId, taskId });
  const live = await tryInvoke<CanvasSelectNodeResponse>('canvas_select_node', {
    node_id: nodeId,
    content,
    task_id: taskId,
  });
  return live ?? null;
}

/**
 * H-15: emit a StateCheckpoint canvas_view_event row for per-tab snapshot/delta.
 * Called every CANVAS_CHECKPOINT_INTERVAL_OPS view events or on tab focus-gained.
 * Returns the SSB seq of the emitted row (0 if SSB not available).
 */
export async function emit_canvas_checkpoint(
  viewId: string,
  taskId?: string,
): Promise<number> {
  logStub('emit_canvas_checkpoint', { viewId, taskId });
  const live = await tryInvoke<number>('emit_canvas_checkpoint', {
    view_id: viewId,
    task_id: taskId,
  });
  return live ?? 0;
}

// ---------------------------------------------------------------------------
// H-14: design mode opaque-id resolution + recast writeback + dev-preview proxy
// ---------------------------------------------------------------------------

/** H-14: server-side location entry for an opaque element id.
 *  Mirrors Rust OidResolveResponse (commands_h14.rs) and OidMapEntry (data_oid_tagger.rs).
 *  Cross-lang pair: commands_h14.rs::OidResolveResponse.
 */
export interface DesignOidEntry {
  /** The opaque id that was resolved. */
  oid: string;
  /** Canonical file path. */
  file: string;
  /** Source range [start_line, start_col, end_line, end_col] (1-based). */
  range: [number, number, number, number];
  /** Scope qualifier (hierarchical JSX path, e.g. "App > Header > nav > a"). */
  scope: string;
}

/** H-14: proxy status returned by start_dev_proxy / stop_dev_proxy.
 *  Mirrors Rust ProxyStatus (dev_preview_proxy.rs).
 *  Cross-lang pair: commands_h14.rs + dev_preview_proxy.rs::ProxyStatus.
 */
export interface DesignProxyStatus {
  running: boolean;
  listen_port: number;
  target_url: string;
  /** Tier-2 deferred: whether the painted-preview done-gate was verified. */
  paint_verified: boolean | null;
}

/** H-14: result of applying a recast attribute patch.
 *  Mirrors Rust PatchResult (recast_writeback.rs).
 *  Cross-lang pair: commands_h14.rs + recast_writeback.rs::PatchResult.
 */
export interface RecastPatchResult {
  applied: boolean;
  file: string;
  /** blake3 hex of the written file content (integrity check). */
  content_hash: string;
  bytes_written: number;
  tier_used: string;
}

/**
 * H-14: resolve an opaque data-oid to its source location {file, range, scope}.
 *
 * The server-side OidMap is populated at build time by the data-oid tagger.
 * Returns null when the Tauri runtime is not available (Tier-1 fallback).
 *
 * Cross-lang pair: commands_h14.rs::resolve_oid.
 */
export async function resolve_oid(oid: string): Promise<DesignOidEntry | null> {
  logStub('resolve_oid', { oid });
  const live = await tryInvoke<DesignOidEntry>('resolve_oid', { oid });
  return live ?? null;
}

/**
 * H-14: apply a deterministic style/text recast patch to a source file.
 *
 * kind='deterministic': applies the patch via format-preserving recast writeback.
 *   No model call. Token references only (not raw hex/px — CANVAS §8).
 * kind='structural': DEFERRED (stub-7, GD-5 gate). Returns null in Tier-1.
 *
 * Cross-lang pair: commands_h14.rs::apply_recast_patch.
 */
export async function apply_recast_patch(
  oid: string,
  attrName: string,
  newValue: string,
  kind: 'deterministic' | 'structural' = 'deterministic',
): Promise<RecastPatchResult | null> {
  logStub('apply_recast_patch', { oid, attrName, kind });
  const live = await tryInvoke<RecastPatchResult>('apply_recast_patch', {
    request: { oid, attr_name: attrName, new_value: newValue, kind },
  });
  return live ?? null;
}

/**
 * H-14: start the design-preview proxy for a target dev server URL.
 *
 * Tier-1: returns a ProxyStatus with running=false (stub-8; live TCP listener deferred).
 * Tier-2: the live proxy will return running=true when the painted-preview passes.
 *
 * Cross-lang pair: commands_h14.rs::start_dev_proxy.
 */
export async function start_dev_proxy(
  targetUrl: string,
  listenPort?: number,
): Promise<DesignProxyStatus | null> {
  logStub('start_dev_proxy', { targetUrl, listenPort });
  const live = await tryInvoke<DesignProxyStatus>('start_dev_proxy', {
    target_url: targetUrl,
    listen_port: listenPort,
  });
  return live ?? null;
}

/**
 * H-14: stop the design-preview proxy.
 *
 * Tier-1: no-op that returns a confirmation string (stub-8; live TCP listener deferred).
 *
 * Cross-lang pair: commands_h14.rs::stop_dev_proxy.
 */
export async function stop_dev_proxy(): Promise<string | null> {
  logStub('stop_dev_proxy', {});
  const live = await tryInvoke<string>('stop_dev_proxy', {});
  return live ?? null;
}

/** Response shape from the rewind_session Tauri command. */
export interface RewindSessionResponse {
  beacon_seq: number;
  scope: string;
  summarized: boolean;
  replay_session_id: string | null;
  replay_outcome: string;
  no_op: boolean;
  disabled: boolean;
}

/** H-36: one row from the agent_notification_emitted SSB journal.
 *  Matches the `AgentNotificationRow` Rust DTO in commands_h36.rs. */
export interface AgentNotificationRow {
  /** SSB sequence number -- monotonically increasing, use as a cursor. */
  id: number;
  /** "completion" | "needs_input" */
  kind: 'completion' | 'needs_input';
  /** Bitset: bit 0 = terminal escape attempted, bit 1 = OS notification attempted. */
  surfaces_attempted: number;
  /** Bitset: which surfaces succeeded. */
  surfaces_succeeded: number;
  /** Session ID at dispatch time. */
  session_id: string;
  /** Unix ms at dispatch time. */
  ts_ms: number;
}

/**
 * H-36: poll recent agent_notification_emitted SSB rows.
 *
 * Returns up to `limit` (default 20, max 100) rows ordered by seq ascending.
 * Returns an empty array when the SSB is unavailable or has no rows yet --
 * the frontend treats this as "no notifications yet" (no error, no toast).
 *
 * Cross-lang pair: commands_h36.rs::get_recent_notifications.
 */
export async function get_recent_notifications(
  limit?: number,
): Promise<AgentNotificationRow[]> {
  logStub('get_recent_notifications', { limit });
  const live = await tryInvoke<AgentNotificationRow[]>('get_recent_notifications', { limit });
  return live ?? [];
}

/** H-37: one row from the plan_decision_event SSB journal.
 *  Matches the `PlanDecisionRow` Rust DTO in commands_h37.rs. */
export interface PlanDecisionRow {
  /** SSB sequence number -- monotonically increasing, use as a cursor. */
  id: number;
  /** "approve" | "redirect" | "reject" | "auto_advance" */
  decision_kind: 'approve' | 'redirect' | 'reject' | 'auto_advance';
  /** FSM target state attempted (e.g. "TESTING_FIRST"; "PLANNING" on Redirect). */
  target_state: string;
  /** Risk classification: "LOW" | "MEDIUM" | "HIGH". */
  risk_level: string;
  /** Human reason for Reject/Redirect; null for Approve/AutoAdvance. */
  reason: string | null;
  /** FSM phase label at the time of the gate invocation. */
  plan_session_id: string;
}

/**
 * H-37: poll plan_decision_event SSB rows for a session.
 *
 * Returns up to `limit` (default 20, max 100) rows ordered by seq ascending.
 * Returns an empty array when the SSB is unavailable or has no rows yet.
 * The gate is wire-dark unless ELAI_PLAN_APPROVAL_GATE_ENABLE=1 on the
 * orchestrator side.
 *
 * Cross-lang pair: commands_h37.rs::get_plan_approval_decisions.
 */
export async function get_plan_approval_decisions(
  sessionId?: string,
  limit?: number,
): Promise<PlanDecisionRow[]> {
  logStub('get_plan_approval_decisions', { sessionId, limit });
  const live = await tryInvoke<PlanDecisionRow[]>('get_plan_approval_decisions', {
    sessionId,
    limit,
  });
  return live ?? [];
}

/**
 * H-35: rewind session state to a prior SSB journal point.
 *
 * Emits one `session_rewound` SSB beacon per successful rewind.
 * A no-op rewind (target_seq == head) emits no row and sets no_op=true.
 * Kill-switch: ELAI_REWIND_DISABLE suppresses all rewinds.
 *
 * Cross-lang pair: commands_h35.rs::rewind_session.
 */
export async function rewind_session(
  sessionId: string,
  targetSeq: number,
  scope: string = 'both',
  summarize: boolean = false,
): Promise<RewindSessionResponse | null> {
  logStub('rewind_session', { sessionId, targetSeq, scope, summarize });
  const live = await tryInvoke<RewindSessionResponse>('rewind_session', {
    session_id: sessionId,
    target_seq: targetSeq,
    scope,
    summarize,
  });
  return live ?? null;
}

// ---------------------------------------------------------------------------
// H-49: time-travel undo / redo / history / jump + restore-files-only
// ---------------------------------------------------------------------------

/** Response shape from H-49 time-travel Tauri commands. */
export interface TimeTravelResponse {
  disabled: boolean;
  no_op: boolean;
  beacon_seq: number;
  head: string | null;
  summary: string;
  divergence_detected: boolean;
}

/** One node in the H-49 undo-tree (returned by timetravel_history). */
export interface TimeTravelNode {
  snap_id: string;
  parent_id: string | null;
  is_head: boolean;
  /** Number of children. > 1 = branch point. */
  children_count: number;
  tag: string | null;
  created_at_ms: number;
  manifest_hash: string | null;
}

/** Request shape for timetravel_jump. */
export interface TimeTravelJumpRequest {
  session_id: string;
  target: string;
  grain?: string;
  scope?: string;
  dry_run?: boolean;
}

/**
 * H-49 — undo the most recent LLM file edit(s).
 *
 * Restores files to the pre-edit state at the requested grain.
 * Returns null when the SSB writer is unavailable.
 *
 * Cross-lang pair: commands_h49.rs::timetravel_undo.
 */
export async function timetravel_undo(
  sessionId: string,
  scope: string = 'code',
): Promise<TimeTravelResponse | null> {
  logStub('timetravel_undo', { sessionId, scope });
  const live = await tryInvoke<TimeTravelResponse>('timetravel_undo', {
    session_id: sessionId,
    scope,
  });
  return live ?? null;
}

/**
 * H-49 — redo the previously undone LLM edit.
 *
 * Cross-lang pair: commands_h49.rs::timetravel_redo.
 */
export async function timetravel_redo(
  sessionId: string,
  scope: string = 'code',
): Promise<TimeTravelResponse | null> {
  logStub('timetravel_redo', { sessionId, scope });
  const live = await tryInvoke<TimeTravelResponse>('timetravel_redo', {
    session_id: sessionId,
    scope,
  });
  return live ?? null;
}

/**
 * H-49 — jump HEAD to a specific snapshot node.
 *
 * Cross-lang pair: commands_h49.rs::timetravel_jump.
 */
export async function timetravel_jump(
  request: TimeTravelJumpRequest,
): Promise<TimeTravelResponse | null> {
  logStub('timetravel_jump', request);
  const live = await tryInvoke<TimeTravelResponse>('timetravel_jump', {
    request,
  });
  return live ?? null;
}

/**
 * H-49 — return the undo-tree nodes for the given session.
 *
 * Used by BranchTree.svelte to render the timeline.
 *
 * Cross-lang pair: commands_h49.rs::timetravel_history.
 */
export async function timetravel_history(
  sessionId: string,
): Promise<TimeTravelNode[]> {
  logStub('timetravel_history', { sessionId });
  const live = await tryInvoke<TimeTravelNode[]>('timetravel_history', {
    session_id: sessionId,
  });
  return live ?? [];
}

/**
 * H-49 — restore files to the state at a snapshot WITHOUT moving HEAD.
 *
 * Matches Cline's "Restore Files Only" affordance.
 *
 * Cross-lang pair: commands_h49.rs::timetravel_restore_files_only.
 */
export async function timetravel_restore_files_only(
  sessionId: string,
  target: string,
): Promise<TimeTravelResponse | null> {
  logStub('timetravel_restore_files_only', { sessionId, target });
  const live = await tryInvoke<TimeTravelResponse>('timetravel_restore_files_only', {
    session_id: sessionId,
    target,
  });
  return live ?? null;
}

/** H-49 paged transcript page shape. */
export interface TranscriptPage {
  rows: TranscriptRow[];
  total_count: number | null;
  first_seq: number | null;
  has_older: boolean;
  has_newer: boolean;
}

/** H-49 transcript row (matches commands_e29.rs TranscriptRow). */
export interface TranscriptRow {
  seq: number;
  ts_ms: number;
  action_type: string;
  agent_id: string;
  role: string;
  body: string;
  redacted: boolean;
}

/** H-49 transcript filter with cursor pagination fields. */
export interface TranscriptFilter {
  action_type_whitelist?: string[];
  agent_role?: string;
  limit?: number;
  before_seq?: number;
  after_seq?: number;
  total_count?: boolean;
  first_seq?: boolean;
}

/**
 * H-49 — paginated transcript fetch (cursor-based, replaces ENTRY_LIMIT=50).
 *
 * Cross-lang pair: commands_e29.rs::ssb_transcript_paged.
 */
export async function ssb_transcript_paged(
  sessionId: string,
  filter?: TranscriptFilter,
): Promise<TranscriptPage> {
  logStub('ssb_transcript_paged', { sessionId, filter });
  const live = await tryInvoke<TranscriptPage>('ssb_transcript_paged', {
    session_id: sessionId,
    filter,
  });
  return live ?? { rows: [], total_count: null, first_seq: null, has_older: false, has_newer: false };
}

// ---------------------------------------------------------------------------
// H-51: sleep-time handoff digest resume
// Cross-lang pair: commands_h51.rs + CommandWindow.svelte
// ---------------------------------------------------------------------------

/** H-51 topic handle returned by resume_offer. */
export interface TopicHandleDto {
  topic_slug: string;
  digest_path: string;
  loaded_tokens: number;
}

/** H-51 tiered load result returned by tiered_load_confirm. */
export interface TieredLoadResult {
  topic_slug: string;
  digest_content: string;
  loaded_tokens: number;
}

/**
 * H-51 — check for an open handoff topic in the active project root.
 *
 * Returns a TopicHandleDto when a digest file modified within 8 h is found,
 * otherwise null.  Respects ELAI_HANDOFF_AUTORESUME_DISABLE kill-switch.
 *
 * Cross-lang pair: commands_h51.rs::resume_offer.
 */
export async function resume_offer(): Promise<TopicHandleDto | null> {
  logStub('resume_offer', {});
  const live = await tryInvoke<TopicHandleDto | null>('resume_offer', {});
  return live ?? null;
}

/**
 * H-51 — confirm tiered load resume for the given topic slug.
 *
 * Emits ResumeConfirmed + TieredLoad SSB rows and returns the digest content.
 * Does NOT check ELAI_HANDOFF_AUTORESUME_DISABLE — this is explicit user intent.
 *
 * Cross-lang pair: commands_h51.rs::tiered_load_confirm.
 */
export async function tiered_load_confirm(slug: string): Promise<TieredLoadResult | null> {
  logStub('tiered_load_confirm', { slug });
  const live = await tryInvoke<TieredLoadResult>('tiered_load_confirm', { slug });
  return live ?? null;
}

// ------------------------------------------------------------------
// I-15 — oracle cache source discriminator
//
// Returns the source discriminator for the most-recent cache event on
// the given query hash hex.  "blake3_exact" = in-session exact hit
// (zero ONNX calls); "semantic_vector" = full pipeline ran; null =
// no row yet (cold start).
//
// Cross-lang pair: commands.rs::get_research_cache_source
//                  types.ts::OracleCacheSource
// ------------------------------------------------------------------
export async function get_research_cache_source(
  queryHashHex: string,
): Promise<import('./types').OracleCacheSource> {
  logStub('get_research_cache_source', { queryHashHex });
  const live = await tryInvoke<string | null>('get_research_cache_source', { queryHashHex });
  if (live === 'blake3_exact' || live === 'semantic_vector') return live;
  return null;
}

// ------------------------------------------------------------------
// I-38 (SCHEMA_VERSION 235) — QdeltaPanel
//
// Returns the most-recent `qdelta_measured` SSB flat-beacon row,
// or null when no measurement has been recorded yet (honest empty state).
//
// Live-model leg deferred (D-026); fixture-tier rows emitted by
// `elai bench qdelta --config <PATH>`.
//
// Cross-lang pair: commands_i38.rs::get_qdelta_latest
//                  types.ts::QdeltaLatest
// ------------------------------------------------------------------
export async function get_qdelta_latest(): Promise<import('./types').QdeltaLatest | null> {
  logStub('get_qdelta_latest', {});
  const live = await tryInvoke<import('./types').QdeltaLatest>('get_qdelta_latest');
  return live ?? null;
}

// ------------------------------------------------------------------
// I-33 (SCHEMA_VERSION 240)
// Cross-lang pair: commands_i33.rs::get_governance_metrics
//                  types.ts::GovernanceMetrics
// ------------------------------------------------------------------
export async function get_governance_metrics(): Promise<import('./types').GovernanceMetrics | null> {
  logStub('get_governance_metrics', {});
  const live = await tryInvoke<import('./types').GovernanceMetrics>('get_governance_metrics');
  return live ?? null;
}

// ------------------------------------------------------------------
// I-12 (SCHEMA_VERSION 251) -- PlaybookEvent panel rows.
//
// Returns the most-recent PlaybookEvent SSB rows (RulesDrawer playbook panel).
// Wire-dark until a RulesDrawer panel is wired; returns empty array in stub.
//
// Cross-lang pair: rule_curator.rs::PlaybookEventPayload
//                  types.ts::PlaybookEventPayload
// ------------------------------------------------------------------
// WIRE-DARK[I-12]: RulesDrawer playbook panel rows land in a future dashboard plan (see stubs.md id=I-12-playbook-drawer)
export async function get_playbook_entries(): Promise<import('./types').PlaybookEventPayload[]> {
  logStub('get_playbook_entries', {});
  const live = await tryInvoke<import('./types').PlaybookEventPayload[]>('get_playbook_entries');
  return live ?? [];
}

// ------------------------------------------------------------------
// I-36 (SCHEMA_VERSION 268) -- CompiledVariant badge chip rows.
//
// Returns up to 20 most-recent prompt_compiled SSB rows emitted when
// CompiledPromptStore::lookup returns a promoted variant and it is
// placed into CompletionConfig::compiled_system_override.
// Returns empty array when no substitution has occurred or
// ELAI_PROMPT_COMPILE_DISABLE=1.
//
// Cross-lang pair: commands_i36.rs::get_compiled_variants
//                  types.ts::CompiledVariantRow
// ------------------------------------------------------------------
export async function get_compiled_variants(): Promise<import('./types').CompiledVariantRow[]> {
  logStub('get_compiled_variants', {});
  const live = await tryInvoke<import('./types').CompiledVariantRow[]>('get_compiled_variants');
  return live ?? [];
}
