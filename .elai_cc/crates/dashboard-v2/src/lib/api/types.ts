// ============================================================
// ELAI Dashboard — Domain TypeScript types
// All types consumed by stub.ts, mock-data.ts, and Svelte stores.
// ============================================================

// ------------------------------------------------------------------
// FSM
// ------------------------------------------------------------------
export type FsmState =
  | 'IDLE'
  | 'PLANNING'
  | 'FEASIBILITY_PROBE'
  | 'TESTING_FIRST'
  | 'EXECUTING'
  | 'AUDITING'
  | 'MEMORIZING'
  | 'COMPLETE'
  | 'CONFIGURING'
  | 'RESEARCHING';

export interface FsmTransition {
  from: FsmState;
  to: FsmState;
  ts: number;
  trigger?: string;
}

export interface FsmSnapshot {
  state: FsmState;
  current_step_id: string;
  progress: number; // 0-1
  history: FsmTransition[];
  agent_substate?: string;
}

// ------------------------------------------------------------------
// Execution / Gate modes
// ------------------------------------------------------------------
export type ExecutionMode = 'LOCAL' | 'CLOUD' | 'HYBRID';
export type GateMode = 'INTERACTIVE' | 'AUTONOMOUS';

// ------------------------------------------------------------------
// Window + Drawer keys
// ------------------------------------------------------------------
export type WindowType =
  | 'command'
  | 'plan'
  | 'activity'
  | 'review'
  | 'explore'
  | 'monitor'
  | 'settings';

export type DrawerKey =
  | 'knowledge'
  | 'reasoning'
  | 'orchestration'
  | 'rules'
  | 'research'
  | 'providers'
  | 'project-intelligence'
  | 'anti-staleness'
  | 'tools-hooks'
  | 'web-quality';

// ------------------------------------------------------------------
// Agent roles
// ------------------------------------------------------------------
export type AgentRole =
  | 'executor'
  | 'auditor'
  | 'researcher'
  | 'designer'
  | 'prompt-engineer'
  | 'context-compiler'
  | 'memory-writer'
  | 'media-validator';

export type AgentLifecycle = 'STARTING' | 'RUNNING' | 'IDLE' | 'DEAD' | 'RESTARTING';

export interface AgentEntry {
  role: AgentRole;
  lifecycle: AgentLifecycle;
  pid?: number;
  last_heartbeat_ms: number; // ms since last heartbeat
  grc_budget_used_pct: number; // 0-100
  death_classification?: FailureClassification;
}

// ------------------------------------------------------------------
// Activity entries
// ------------------------------------------------------------------
export type ActivityKind =
  | 'reasoning'
  | 'edit'
  | 'validate'
  | 'test'
  | 'preview'
  | 'cloud'
  | 'checkpoint'
  | 'mistake'
  | 'phase-transition';

export interface ActivityEntry {
  id: string;
  kind: ActivityKind;
  ts: number;
  step_id?: string;
  agent?: AgentRole;
  payload: ActivityPayload;
}

export type ActivityPayload =
  | ReasoningPayload
  | EditPayload
  | ValidatePayload
  | TestPayload
  | PreviewPayload
  | CloudPayload
  | CheckpointPayload
  | MistakePayload
  | PhaseTransitionPayload;

export interface ReasoningPayload {
  kind: 'reasoning';
  step_type: 'observe' | 'hypothesize' | 'plan_action' | 'act' | 'verify' | 'status';
  content: string;
  grounding_source?: string; // e.g. "code we just read", "WebSearch"
  confidence: 'Verified' | 'High confidence' | 'Uncertain';
  call_num: number;
  budget_remaining: number;
}

export interface EditPayload {
  kind: 'edit';
  file: string;
  added: number;
  removed: number;
  diff_summary: string;
  expanded?: boolean;
}

export interface ValidatePayload {
  kind: 'validate';
  tier: 1 | 2 | 3;
  verdict: 'ALLOW' | 'BLOCK' | 'WARN';
  rule_id: string;
  message: string;
}

export interface TestPayload {
  kind: 'test';
  pass: number;
  fail: number;
  crate: string;
  failures?: string[];
}

export interface PreviewPayload {
  kind: 'preview';
  url: string;
  label: string;
}

export interface CloudPayload {
  kind: 'cloud';
  tier: string; // e.g. "claude-3-5-sonnet"
  reason: string;
  tokens_used: number;
}

export interface CheckpointPayload {
  kind: 'checkpoint';
  level: 'BABY_STEP' | 'STEP' | 'PHASE' | 'CONCEPT';
  result: 'PASS' | 'FAIL' | 'PARTIAL';
  baby_step_id?: string;
}

export interface MistakePayload {
  kind: 'mistake';
  hash: string; // blake3 hash, no variable names
  pattern_id: string;
  description: string;
  severity: 'CRITICAL' | 'WARNING';
}

export interface PhaseTransitionPayload {
  kind: 'phase-transition';
  from_state: FsmState;
  to_state: FsmState;
  trigger: string;
}

// ------------------------------------------------------------------
// GRC (Grounded Reasoning Chain)
// ------------------------------------------------------------------
export interface GrcBudget {
  role: AgentRole;
  calls_remaining: number;
  chars_remaining: number;
  total_calls: number;
  total_chars: number;
  calls_used: number;
  chars_used: number;
}

export interface GroundingClaim {
  id: string;
  text: string;
  classification: 'GROUNDED' | 'VERIFY_NEEDED' | 'UNGROUNDED';
  source?: string;
}

export interface GroundingSweep {
  grounded: number;
  verify_needed: number;
  ungrounded: number;
  claims: GroundingClaim[];
}

export interface CsqEntry {
  id: string;
  severity: 'CRITICAL' | 'WARNING';
  question: string;
  guard_added: boolean;
  function_context: string;
}

// GG Sidecar logit signals
export interface GgSignals {
  mean_logprob: number;
  min_logprob: number;
  entropy_mean: number;
  entropy_max: number;
  token_count: number;
}

// ------------------------------------------------------------------
// Evidence gates
// ------------------------------------------------------------------
export type GateVerdict = 'pass' | 'block' | 'warn' | 'pending';

export interface EvidenceGate {
  id: string;
  name: string;
  verdict: GateVerdict;
  reason?: string;
  checked_at?: number;
}

// ------------------------------------------------------------------
// Plan / Decomposition
// ------------------------------------------------------------------
export type DecompLevel = 'concept' | 'phase' | 'step' | 'baby-step';

export type BabyStepBlock =
  | 'GROUNDING'
  | 'PRE_GATE'
  | 'TASK'
  | 'TDD'
  | 'DETERMINISTIC_CHECK'
  | 'ACCEPTANCE'
  | 'INTEGRATION'
  | 'EXIT_CRITERIA';

export interface BabyStepDetail {
  blocks: { block: BabyStepBlock; status: 'done' | 'active' | 'pending' | 'failed' }[];
}

export interface DecompNode {
  id: string;
  level: DecompLevel;
  label: string;
  status: 'done' | 'active' | 'pending' | 'failed';
  children?: DecompNode[];
  elapsed_ms?: number;
  detail?: BabyStepDetail; // only on baby-step level
}

export interface BubbleUpEvent {
  ts: number;
  level: DecompLevel;
  node_id: string;
  message: string;
  escalated: boolean;
}

// ------------------------------------------------------------------
// ExecutablePipeline DAG
// ------------------------------------------------------------------
export interface PipelineNode {
  id: string;
  label: string;
  kind: 'task' | 'gate' | 'repair' | 'merge';
  status: 'done' | 'active' | 'pending' | 'failed' | 'skipped';
  deps: string[]; // node IDs
}

export interface DagRepairEvent {
  ts: number;
  kind: 'Reorder' | 'Substitute';
  from_node_id: string;
  to_node_id?: string;
  t_dag_counter: number;
}

// ------------------------------------------------------------------
// Knowledge / Search
// ------------------------------------------------------------------
export type SearchMode = 'HYBRID' | 'SPARSE_ONLY' | 'BM25_ONLY';

export interface QueryStage {
  name: string;
  latency_ms: number;
  pass: boolean;
}

export interface QueryTrace {
  id: string;
  query: string;
  mode: SearchMode;
  stages: QueryStage[];
  total_ms: number;
  ts: number;
  hit_count: number;
}

export type BreakerState = 'CLOSED' | 'OPEN' | 'HALF_OPEN';

export interface EmbeddingBreaker {
  model: string;
  state: BreakerState;
  failures: number;
  success_rate: number; // 0-1
  last_failure_ts?: number;
}

export interface RecallMetrics {
  tier: 'FAST' | 'STANDARD' | 'DEEP';
  r_at_5: number;
  r_at_10: number;
  mrr: number;
  latency_ms: number;
}

export interface DkgStats {
  nodes: number;
  edges: number;
  bm25_chunks: number;
  vector_status: 'READY' | 'DEGRADED' | 'BUILDING';
  freshness_decay_pct: number; // 0-100
}

export interface FreshnessStatus {
  decay_window_days: number;
  oldest_ingested_ts: number;
  confidence_decay_pct: number;
}

// ------------------------------------------------------------------
// Rules
// ------------------------------------------------------------------
export type RuleLifecycle = 'PROPOSED' | 'CANDIDATE' | 'SHADOW' | 'ACTIVE' | 'RETIRED';

export type PredicateType =
  | 'AST_PATTERN'
  | 'GREP_PATTERN'
  | 'SCHEMA_REQUIRED'
  | 'METRIC_GATE'
  | 'COMPOSITE'
  | 'GRAPH_REACHABILITY'
  | 'VERSION_GATE';

export interface Rule {
  id: string;
  name: string;
  lifecycle: RuleLifecycle;
  predicate: PredicateType;
  domain: string;
  active_in_states: FsmState[];
  added_ts: number;
}

export interface RuleLifecycleCounts {
  PROPOSED: number;
  CANDIDATE: number;
  SHADOW: number;
  ACTIVE: number;
  RETIRED: number;
}

export interface DeprecatedPattern {
  id: string;
  pattern: string;
  language: string;
  added_ts: number;
  replacement?: string;
}

export interface VersionGateStatus {
  active: boolean;
  blocked_count: number;
  last_checked_ts: number;
}

export type SourceTier = 'T1' | 'T2' | 'T3' | 'T4' | 'T5';

export interface SourceTierDistribution {
  tier: SourceTier;
  label: string;
  count: number;
  pct: number;
}

// ------------------------------------------------------------------
// Intervention / Command cards
// ------------------------------------------------------------------
export type InterventionAction = 'approve' | 'reject' | 'modify' | 'redirect' | 'takeover';

export interface InterventionRequest {
  id: string;
  title: string;
  description: string;
  file?: string;
  diff_summary?: string;
  created_ts: number;
}

export type CardKind = 'status' | 'intervention' | 'error' | 'disambiguation' | 'steering' | 'user' | 'response';

export interface CommandCard {
  id: string;
  ts: number;
  kind: CardKind;
  payload: CardPayload;
}

export type CardPayload =
  | StatusCardPayload
  | InterventionCardPayload
  | ErrorCardPayload
  | DisambiguationCardPayload
  | SteeringCardPayload
  | UserCardPayload
  | ResponseCardPayload;

export interface StatusCardPayload {
  kind: 'status';
  message: string;
  fsm_state?: FsmState;
  step_id?: string;
}

export interface InterventionCardPayload {
  kind: 'intervention';
  request: InterventionRequest;
}

export interface ErrorCardPayload {
  kind: 'error';
  message: string;
  source: string;
  step_id?: string;
  recoverable: boolean;
}

export interface DisambiguationCardPayload {
  kind: 'disambiguation';
  question: string;
  options: { id: string; label: string }[];
}

export interface SteeringCardPayload {
  kind: 'steering';
  vector_label: string;
  delta_description: string;
  applied: boolean;
}

export interface UserCardPayload {
  kind: 'user';
  text: string;
  intent: import('./types').IntentType;
}

export interface ResponseCardPayload {
  kind: 'response';
  text: string;
  streaming: boolean;
  stream_id: string;
}

// ------------------------------------------------------------------
// Sidecar / Inference
// ------------------------------------------------------------------
export type TurboQuantMode = 'TQ4' | 'TQ8' | 'OFF';

export interface SidecarStatus {
  loaded: boolean;
  model: string;
  fa2: boolean;
  dola: boolean;
  steering: boolean;
  turboquant: TurboQuantMode;
  tokens_per_sec: number;
  latency_p50_ms: number;
  latency_p99_ms: number;
  context_window: number;
  mcp_bridge_active: boolean;
}

/** D.1 sidecar_health — richer health probe, distinct from SidecarStatus.
 *  Maps to TESTED-AND-DASHBOARD-WIRING §10.5 schema.
 *  When loaded=false, only the `error` field is guaranteed present.
 *
 *  vram_available: false when sidecar did not report VRAM metrics (current
 *  state — server.py /health lacks VRAM probe). When false, vram_total/used/free
 *  are 0.0 and must NOT be displayed as real values.
 *  head_dim: 0 = unknown (sentinel). Populated when /v1/models exposes field. */
export interface SidecarHealth {
  loaded: boolean;
  error?: string;
  model: string;
  fa2: boolean;
  dola: boolean;
  steering: boolean;
  turboquant: TurboQuantMode;
  vram_available: boolean;
  vram_total: number;
  vram_used: number;
  vram_free: number;
  head_dim: number;
  layers: number;
  kv_quant: boolean;
  mcp_bridge_active: boolean;
  tokens_per_sec: number;
  latency_p50: number;
  latency_p99: number;
  context_window: number;
}

export interface InferenceMetrics {
  tokens_per_sec_last_60s: number;
  latency_p50_ms: number;
  latency_p99_ms: number;
  total_tokens_session: number;
  cloud_fallback_count: number;
}

// ------------------------------------------------------------------
// VRAM
// ------------------------------------------------------------------
export interface VramGauge {
  gpu_id: number;
  gpu_name: string;
  total_mb: number;
  used_mb: number;
  free_mb: number;
  pct_used: number; // 0-100, derived
}

// ------------------------------------------------------------------
// Project
// ------------------------------------------------------------------
export interface ProjectSummary {
  name: string;
  path: string;
  last_opened_iso: string;
}

// ------------------------------------------------------------------
// Responses
// ------------------------------------------------------------------
export interface InterventionResponse {
  actionId: string;
  response: InterventionAction;
  payload?: unknown;
}

// ------------------------------------------------------------------
// Recent tasks
// ------------------------------------------------------------------
export interface RecentTask {
  id: string;
  title: string;
  status: 'completed' | 'interrupted' | 'failed';
  finished_iso: string;
  fsm_state_at_end: FsmState;
  duration_ms: number;
}

// ------------------------------------------------------------------
// Context budget
// ------------------------------------------------------------------
export interface ContextBudgetSlot {
  name: string;
  used_tokens: number;
  total_tokens: number;
  pct: number;
}

export interface ContextBudget {
  slots: ContextBudgetSlot[];
  total_used: number;
  total_cap: number;
}

// ------------------------------------------------------------------
// Supervision
// ------------------------------------------------------------------
export interface SupervisionEvent {
  ts: number;
  agent: AgentRole;
  event: 'STARTED' | 'RESTARTED' | 'DIED' | 'HEALTHY';
  restart_count: number;
  reason?: string;
}

// ------------------------------------------------------------------
// Heartbeat
// ------------------------------------------------------------------
export interface HeartbeatEntry {
  agent: AgentRole;
  last_ts: number;
  interval_ms: number; // expected interval
  timeout_ms: number;
  alive: boolean;
}

// ------------------------------------------------------------------
// Research tree
// ------------------------------------------------------------------
export type ResearchNodeStage =
  | 'FORMULATE'
  | 'SEARCH'
  | 'EXTRACT'
  | 'REFLECT'
  | 'CONVERGE';

export interface ResearchStageEvent {
  node_id: number;
  stage: 'formulate' | 'search' | 'extract' | 'reflect' | 'converge';
  phase: 'started' | 'completed';
  ts_ms: number;
  phi: number;
  psi: number;
  iteration: number;
  payload?: Record<string, unknown>;
}

export interface ResearchTreeNode {
  id: string;
  query: string;
  stage: ResearchNodeStage;
  children?: ResearchTreeNode[];
  source_count: number;
  converged: boolean;
  phi?: number; // convergence threshold 1
  psi?: number; // convergence threshold 2
  // B-7: fine-grained per-stage events for live progress display.
  stage_events?: ResearchStageEvent[];
}

// ------------------------------------------------------------------
// BSP (Baby Step Pipeline) superstep
// ------------------------------------------------------------------
export interface BspSuperstep {
  id: number;
  ts: number;
  baby_step_id: string;
  block: BabyStepBlock;
  duration_ms: number;
  status: 'PASS' | 'FAIL' | 'SKIP';
  notes?: string;
}

// ------------------------------------------------------------------
// Session branches
// ------------------------------------------------------------------
export type BranchStatus = 'active' | 'merged' | 'abandoned';

export interface SessionBranch {
  id: string;
  label: string;
  status: BranchStatus;
  created_ts: number;
  last_activity_ts: number;
  parent_branch_id?: string;
}

// ------------------------------------------------------------------
// Anti-spiral
// ------------------------------------------------------------------
export interface AntiSpiralCounter {
  step_id: string;
  edit_count: number;
  threshold: number;
  triggered: boolean;
  last_hash: string; // blake3 of file set
}

// ------------------------------------------------------------------
// Mistake book
// ------------------------------------------------------------------
export interface MistakeEntry {
  id: string;
  baby_step_id: string;
  hash: string; // blake3, no variable names
  pattern_id: string;
  description: string;
  severity: 'CRITICAL' | 'WARNING';
  ts: number;
  resolved: boolean;
}

// ------------------------------------------------------------------
// Interrogation tree
// ------------------------------------------------------------------
export interface InterrogationChoice {
  id: string;
  label: string;
  next_node_id?: string;
}

export interface InterrogationNode {
  id: string;
  question: string;
  choices: InterrogationChoice[];
  diagnosis?: string; // leaf node
}

// ------------------------------------------------------------------
// Failure taxonomy
// ------------------------------------------------------------------
export type FailureClassification =
  | 'LOGIC_ERROR'
  | 'TYPE_MISMATCH'
  | 'RESOURCE_EXHAUSTED'
  | 'NETWORK_TIMEOUT'
  | 'DEPENDENCY_MISSING'
  | 'PERMISSION_DENIED'
  | 'ASSERTION_FAILED'
  | 'PANIC'
  | 'UNCLASSIFIED';

export interface FailureTaxonomyResult {
  classification: FailureClassification;
  confidence: number; // 0-1
  baby_step_id: string;
  file: string;
  line?: number;
  raw_message: string;
  ts: number;
}

// ------------------------------------------------------------------
// Build / Dev servers
// ------------------------------------------------------------------
export type BuildStatus = 'draft' | 'building' | 'live' | 'error';

export interface BuildSlot {
  id: string;
  label: string;
  status: BuildStatus;
  url?: string;
  duration_ms?: number;
  started_ts?: number;
}

export interface DevServer {
  pid: number;
  port: number;
  label: string;
  status: 'running' | 'stopped' | 'error';
  started_ts: number;
  log_tail: string[];
}

// ------------------------------------------------------------------
// Project intelligence
// ------------------------------------------------------------------
export interface ProjectTier1 {
  language: string;
  build_tool: string;
  framework: string;
  test_runner: string;
  ci: string;
  linter: string;
}

export interface ProjectConvention {
  name: string;
  strength: 'strong' | 'moderate' | 'mixed';
  example: string;
}

export interface ProjectTier2 {
  conventions: ProjectConvention[];
}

export interface TelescopeSummary {
  level: 1 | 2 | 3 | 4;
  summary: string;
}

export interface ProjectTier3 {
  architecture_inference: string;
  telescope_summaries: TelescopeSummary[];
}

export interface ProjectProfile {
  tier1: ProjectTier1;
  tier2: ProjectTier2;
  tier3: ProjectTier3;
  override_file?: string;
}

// ------------------------------------------------------------------
// Feasibility
// ------------------------------------------------------------------
export type FeasibilityOutcome = 'PASS' | 'FAIL' | 'BLOCKED';

export interface FeasibilityResult {
  id: string;
  check: string;
  outcome: FeasibilityOutcome;
  /** B-21 4F: Rust `Option<String>` serializes to JSON null, never undefined.
   *  Consumers MUST use `== null` (not `=== undefined`). */
  detail: string | null;
  ts: number;
}

export interface FeasibilityReport {
  overall: FeasibilityOutcome;
  results: FeasibilityResult[];
  probed_at: number;
}

// ------------------------------------------------------------------
// Baseline harness (A-1)
// ------------------------------------------------------------------
export interface BaselineMetric {
  name: string;
  value: number;
  unit: string;
  threshold: number;
  pass: boolean;
}

export interface BaselineResults {
  metrics: BaselineMetric[];
  run_ts: number;
  deterministic: boolean;
}

// ------------------------------------------------------------------
// Dep audit
// ------------------------------------------------------------------
export interface DepAuditResult {
  crate_or_pkg: string;
  current_version: string;
  latest_version: string;
  outdated: boolean;
  security_advisory?: string;
  checked_ts: number;
}

// ------------------------------------------------------------------
// MSI (Multi-Session Isolation)
// ------------------------------------------------------------------
export interface MsiStatus {
  session_id: string;
  owned_files: string[];
  locked_files: string[];
  conflicts: string[];
  active_sessions: { id: string; task: string }[];
}

// ------------------------------------------------------------------
// Steering
// ------------------------------------------------------------------
export interface SteeringStatus {
  active: boolean;
  vector_label: string;
  last_computed_ts: number;
  magnitude: number;
  applied_count: number;
}

// ------------------------------------------------------------------
// MCP
// ------------------------------------------------------------------
export interface McpStatus {
  enabled: boolean;
  bridge_url: string;
  ping_ms?: number;
  last_ping_ts?: number;
  connected: boolean;
  tools_available: string[];
}

// ------------------------------------------------------------------
// Gate status
// ------------------------------------------------------------------
export interface GateStatus {
  mode: GateMode;
  auto_approve_threshold: number;
  pending_count: number;
  auto_advanced_count: number;
}

// ------------------------------------------------------------------
// Model routing
// ------------------------------------------------------------------
export interface ModelRouting {
  role: AgentRole;
  target: string; // e.g. "Qwen2.5-Coder-7B-Instruct", "claude-3-5-sonnet-20241022"
  fallback?: string;
}

// ------------------------------------------------------------------
// System status
// ------------------------------------------------------------------
export interface SystemStatus {
  fsm: FsmSnapshot;
  sidecar: SidecarStatus;
  vram: VramGauge[];
  gate: GateStatus;
  active_agents: AgentEntry[];
  session_id: string;
  uptime_ms: number;
}

// ------------------------------------------------------------------
// Auto-advance stats
// ------------------------------------------------------------------
export interface AutoAdvanceStats {
  total_steps: number;
  auto_advanced: number;
  intervention_required: number;
  auto_advance_rate: number; // 0-1
  last_24h_rate: number;
}

// ------------------------------------------------------------------
// Optimization proposals
// ------------------------------------------------------------------
export interface OptimizationProposal {
  id: string;
  title: string;
  description: string;
  impact: 'HIGH' | 'MEDIUM' | 'LOW';
  status: 'pending' | 'applied' | 'rejected';
  created_ts: number;
}

// ------------------------------------------------------------------
// B-23 Phase 4H Group F — self-mod proposal + MSI surfaces.
// Distinct from `OptimizationProposal` (legacy display shape) and from
// the multi-session-isolation `MsiStatus` above. These mirror the
// Tauri command return shapes for `get_pending_proposals`,
// `get_applied_proposals`, and `get_msi_status` after the Group F wiring.
// ------------------------------------------------------------------
export interface SelfModProposalRow {
  proposal_id: string;
  table: string;
  fpr_estimate?: number;
  occurrences_seen?: number;
  pipeline_mode: string;
  applied_by?: string;
  ts_ms: number;
}

export interface SelfModMsiSnapshot {
  pipeline_mode: string;
  msi_value: number;
  status: string;
  ts_ms: number;
}

// ------------------------------------------------------------------
// Tab model (workspace layout)
// ------------------------------------------------------------------
export interface TabModel {
  id: string;
  type: WindowType;
  label: string;
}

// ------------------------------------------------------------------
// Slash commands
// ------------------------------------------------------------------
export type SlashCommand =
  | '/task'
  | '/fix'
  | '/explore'
  | '/config'
  | '/status'
  | '/continue'
  | '/research'
  | '/ask';

export interface SlashCommandDef {
  command: SlashCommand;
  description: string;
  example: string;
}

// ------------------------------------------------------------------
// Web quality / scaffold
// ------------------------------------------------------------------
export interface ScaffoldTemplate {
  id: string;
  label: string;
  framework: string;
  files_count: number;
  // Phase A (s-69820): the four fields below are populated when the live
  // `list_scaffold_templates` Tauri command is reachable. Mock data leaves
  // them undefined; the drawer falls back to the legacy two-line render.
  description?: string;
  sections?: string[];
  dev_port?: number;
  build_command?: string;
  dev_command?: string;
  recommended_for?: string[];
  theme_system?: string;
  validation_commands?: string[];
  assets_count?: number;
}

/// Phase A (s-69820): apply request to the `apply_scaffold_template` Tauri
/// command. `target_dir` must be an absolute path on the host fs; the
/// command rejects non-absolute paths and pre-existing non-empty dirs.
export interface ApplyScaffoldArgs {
  template: string;
  target_dir: string;
  project_name?: string;
  description?: string;
}

export interface WebAragState {
  index_size: number;
  last_crawled_ts: number;
  pages_indexed: number;
  quality_score: number; // 0-1
}

export interface WebRulesCounts {
  t1: number;
  t2: number;
  t3: number;
}

// ------------------------------------------------------------------
// Tools / Hooks
// ------------------------------------------------------------------
export type HookType = 'PreToolUse' | 'PostToolUse' | 'Stop' | 'PreCompact';
export type HookVerdict = 'ALLOW' | 'BLOCK' | 'WARN';

export interface HookExecution {
  ts: number;
  hook: HookType;
  tool?: string;
  verdict: HookVerdict;
  reason?: string;
}

export interface ShortCoderStats {
  savings_pct: number;
  exact_hits: number;
  whitespace_hits: number;
  ellipsis_hits: number;
  total_edits: number;
}

export interface ToolFilterEntry {
  tool: string;
  allowed: boolean;
  reason?: string;
}

// ------------------------------------------------------------------
// Intent classification
// ------------------------------------------------------------------
export type IntentType =
  | 'TASK'
  | 'FIX'
  | 'EXPLORE'
  | 'EXPLORE_CODE'
  | 'RESEARCH'
  | 'INTERVENE'
  | 'STEER'
  | 'ABORT'
  | 'CONFIG'
  | 'AMBIGUOUS';

// ------------------------------------------------------------------
// Remote sidecar boot overlay
// ------------------------------------------------------------------
export type RemoteSidecarStep = 'ssh' | 'tunnel' | 'health' | 'done';
export type RemoteSidecarStepStatus = 'pending' | 'running' | 'ok' | 'error';

export interface RemoteSidecarProgress {
  step: RemoteSidecarStep;
  status: RemoteSidecarStepStatus;
  ts_ms: number;
  error?: string;
}

// ------------------------------------------------------------------
// D.4 — Obscura test fetch
// ------------------------------------------------------------------
export interface ObscuraFetchResult {
  ok: boolean;
  html?: string;
  truncated?: boolean;
  fetch_time_ms?: number;
  error?: string;
}

// ------------------------------------------------------------------
// D.5 — Live contract test streaming
// ------------------------------------------------------------------
/** Payload for `d5-progress:{requestId}` Tauri event. */
export interface D5ProgressEvent {
  line: string;
  ts_ms: number;
}

/** Payload for `d5-complete:{requestId}` Tauri event. */
export interface D5CompleteEvent {
  /** Process exit code, or null if the process could not be waited on. */
  exit_code: number | null;
  /** Set when the cargo spawn itself failed (pre-exit). */
  error?: string;
  ts_ms: number;
}

// ------------------------------------------------------------------
// Phase B/C — Web validation (lint + Lighthouse + Chromium probe)
// Mirrors elai-web-validator Rust types (lint.rs, cwv.rs).
// ------------------------------------------------------------------

/** Severity tier for one lint finding.
 *  Mirrors `LintSeverity` in lint.rs with `#[serde(rename_all = "lowercase")]`,
 *  so the wire format is the lowercase variant name. */
export type LintSeverity = 'error' | 'warning' | 'info';

/** A single lint finding emitted by ESLint, Stylelint, or axe-core.
 *  Mirrors `LintFinding` in elai-web-validator/src/lint.rs.
 *  `rule_id` is namespaced per tool (`eslint:no-undef`, `stylelint:color-hex-length`,
 *  `axe:color-contrast`) so the in-house WEB-* rule pack never collides. */
export interface LintFinding {
  rule_id: string;
  severity: LintSeverity;
  /** Path to the file the finding refers to. PathBuf serializes as a string. */
  file: string;
  /** 1-based line number, when the tool reports one. Rust `Option<u32>::None`
   *  serializes to JSON `null`, so the wire form is `number | null` — NOT
   *  `number | undefined`. Consumers must guard with `== null` (catches both)
   *  rather than `=== undefined`. */
  line?: number | null;
  /** 1-based column number, when the tool reports one. See `line` re: null. */
  column?: number | null;
  message: string;
  fixable: boolean;
}

/** UI-side bundle of lint-tool toggles. The Tauri command takes the three
 *  bools as separate parameters; this struct exists for callers that want to
 *  bundle them into a single config object before invoking. Mirrors
 *  `LintRunnerConfig` in lint.rs (skip_missing_packages has #[serde(default)]
 *  on the Rust side so it is optional in TypeScript). */
export interface LintRunnerConfig {
  eslint: boolean;
  stylelint: boolean;
  axe: boolean;
  skip_missing_packages?: boolean;
}

/** Distilled Core Web Vitals + overall performance score for one Lighthouse
 *  run. Mirrors `LighthouseReport` in elai-web-validator/src/cwv.rs. */
export interface LighthouseReport {
  /** Largest Contentful Paint, ms (lower is better). */
  lcp_ms: number;
  /** Cumulative Layout Shift, unitless (target < 0.1). */
  cls: number;
  /** Interaction-to-Next-Paint, ms (target < 200). */
  inp_ms: number;
  /** Performance score in [0, 1] (higher is better). */
  perf_score: number;
}

// B-18 — Sandbox tier status (get_sandbox_status command)

export type SandboxTier = 'A++' | 'C-Windows' | 'C-Linux' | 'None';

export interface SandboxAgentPathCount {
  role: string;
  path_count: number;
}

/** Sandbox status payload from get_sandbox_status Tauri command. */
export interface SandboxStatus {
  /** Effective isolation tier on this platform. */
  tier: SandboxTier;
  /** Number of active per-agent AppContainer profiles (MVP: 0 or 1). */
  profile_count: number;
  /** Per-role filesystem allow-list path counts (B-18 P3). */
  allowed_paths_per_agent: SandboxAgentPathCount[];
  /** Timestamp of last visibility-filter hit in ms, or null if not tracked. */
  last_visibility_filter_hit_ms: number | null;
}

// ------------------------------------------------------------------
// CLOUD AUTH (Anthropic OAuth login + cloud model picker)
// Fields use `T | null` (not `undefined`) because serde serializes Rust
// `Option<T>` as JSON `null` — see CLAUDE.md NULL_VS_UNDEFINED note.
// ------------------------------------------------------------------
export interface OAuthStatus {
  logged_in: boolean;
  display_name: string | null;
  expires_at: number | null;
  scopes: string[];
  expired: boolean;
}

export interface OAuthLoginInfo {
  auth_url: string;
  callback_port: number;
  redirect_uri: string;
  event: string;
}

export interface OAuthLoginCompletePayload {
  ok: boolean;
  error?: string;
}

export interface CloudModelInfo {
  model_id: string;
  allowlist: string[];
  default: string;
}

// B-27: cloud-mode spend summary surfaced in MonitorWindow.
// Fields mirror the snake_case shape returned by the Rust `get_cloud_spend`
// Tauri command. MonitorWindow conditionally renders rows only when
// `task_usd > 0` or `day_usd > 0` (spec D-4:308).
export interface CloudSpendInfo {
  task_usd: number;
  day_usd: number;
  cache_creation_tokens: number;
  cache_read_tokens: number;
  redaction_count: number;
}
