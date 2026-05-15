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
  DagRepairEvent,
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
  SelfModProposalRow,
  SelfModMsiSnapshot,
  ReflectState,
  MediaValidationEntry,
  MediaSsbEvent,
  ReviewDecision,
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

/** Try to invoke a Tauri command; on any failure (or if not in Tauri),
 *  return `undefined` so the caller falls through to mock data. */
async function tryInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T | undefined> {
  if (!TAURI_AVAILABLE) return undefined;
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
// TASK / WORKFLOW
// ------------------------------------------------------------------

export async function submit_task(task: string): Promise<{ taskId: string }> {
  logStub('submit_task', { task });
  const live = await tryInvoke<{ taskId: string }>('submit_task', { task });
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

export async function get_dkg_stats(): Promise<DkgStats> {
  logStub('get_dkg_stats');
  const live = await tryInvoke<DkgStats>('get_dkg_stats');
  if (live) return live;
  await delay();
  return DKG_STATS;
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

export async function get_gg_signals(): Promise<GgSignals> {
  logStub('get_gg_signals');
  const live = await tryInvoke<GgSignals>('get_gg_signals');
  if (live) return live;
  await delay();
  return GG_SIGNALS;
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

export async function get_executable_pipeline(): Promise<PipelineNode[]> {
  logStub('get_executable_pipeline');
  const live = await tryInvoke<PipelineNode[]>('get_executable_pipeline');
  if (live) return live;
  await delay();
  return PIPELINE_NODES;
}

export async function get_dag_repair_log(): Promise<DagRepairEvent[]> {
  logStub('get_dag_repair_log');
  const live = await tryInvoke<DagRepairEvent[]>('get_dag_repair_log');
  if (live) return live;
  await delay();
  return DAG_REPAIR_EVENTS;
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
  };
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

export async function submit_chat_message(
  text: string,
  intent: string,
  sessionId?: string,
): Promise<{ seq: number }> {
  logStub('submit_chat_message', { text, intent, sessionId });
  const live = await tryInvoke<{ seq: number }>('submit_chat_message', {
    text,
    intent,
    sessionId,
  });
  if (live) return live;
  await delay();
  return { seq: Date.now() };
}

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
  const live = await tryInvoke<{ streamId: string; offline?: boolean; reflect?: boolean }>(
    'start_chat_stream',
    {
      prompt: message,
      requestId,
      thinkingEnabled: options?.thinkingEnabled ?? false,
      effortLevel: options?.effortLevel ?? null,
    },
  );
  if (live) return live;
  // Browser-dev mock: simulate streaming via setTimeout token drip.
  // The mock emits tokens directly into the callback supplied by streamChat().
  return { streamId: requestId };
}

export async function cancel_stream(requestId: string): Promise<{ ok: boolean }> {
  logStub('cancel_stream', { requestId });
  const live = await tryInvoke<{ ok: boolean }>('cancel_chat_stream', { requestId });
  if (live) return live;
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
  options?: { thinkingEnabled?: boolean; effortLevel?: string },
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

  // Tauri path: invoke + listen.
  // Dynamic import keeps the Tauri API out of browser-only bundles.
  const { listen } = await import('@tauri-apps/api/event');

  const tokenEvent = `chat-stream:${requestId}`;
  const endEvent = `chat-stream-end:${requestId}`;

  const unlistenToken = await listen<{ text: string; stream_id: string }>(
    tokenEvent,
    (e) => onToken(e.payload.text),
  );
  const unlistenEnd = await listen<{
    stream_id: string;
    stop_reason?: string;
    error?: string;
    offline?: boolean;
  }>(endEvent, (e) => {
    unlistenToken();
    unlistenEnd();
    if (e.payload.error) {
      onError(e.payload.error, Boolean(e.payload.offline));
    } else {
      onEnd(e.payload.stop_reason ?? 'end_turn');
    }
  });

  // Fire the stream — this returns immediately with { streamId }.
  const result = await stream_chat(prompt, requestId, options);
  if (result.offline) {
    unlistenToken();
    unlistenEnd();
    onError(
      'sidecar offline — start with `elai.bat --remote` to use the friend\'s GPU box',
      true,
    );
    return () => {};
  }

  return () => {
    unlistenToken();
    unlistenEnd();
    cancel_stream(requestId).catch(() => {});
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

export async function one_shot_prompt(text: string): Promise<{ ok: boolean; text: string }> {
  logStub('one_shot_prompt', { text });
  const live = await tryInvoke<{ ok: boolean; text: string }>('one_shot_prompt', { text });
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
