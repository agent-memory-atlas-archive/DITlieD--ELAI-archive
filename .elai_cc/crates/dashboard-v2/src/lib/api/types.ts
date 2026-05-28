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
// B-26 T4 — Plan progress (mirror of commands.rs::get_plan_progress).
// Drives the MonitorWindow phase-dot row in place of the prior static
// 5-phase hardcode. `phases === null` means no plan-source data is
// available (e.g. sidecar offline); the frontend falls back to a
// pending-only render in that case.
// ------------------------------------------------------------------
export type PlanPhaseStatus = 'done' | 'active' | 'pending';

export interface PlanPhaseDot {
  num: number;
  label: string;
  steps: number;
  status: PlanPhaseStatus;
}

export interface PlanProgress {
  phases: PlanPhaseDot[] | null;
  current_phase: number | null;
  total_pct: number; // 0-1
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
  | 'monitor'
  | 'settings'
  | 'graph'
  // B-40: Idea-Framer pre-decomposition surface (FRAMING substate).
  | 'idea_framing'
  // B-41: per-level auditor timeline + cascade visualisation.
  | 'audit_timeline'
  // C-7: Q&A oracle / codebase-explorer surface.
  | 'explore'
  // C-40: GitHub PR adapter window (open / list / status / close).
  | 'github'
  // E-40: orchestrator-rooted symbol-graph window (defs/refs/callers/
  // callees + personalized PageRank + LSP-oracle stats). See
  // workflow/reasoning/E-40-decisions.md.
  | 'symbol_graph';

// ------------------------------------------------------------------
// B-33 — Graph viewer (mirror of Rust visualization::types)
// All field names match the serde JSON output of the Rust types so
// invoke('get_graph_view_data') deserialises directly into VizData.
// ------------------------------------------------------------------

export type VizEdgeKind = 'Import' | 'DynamicImport' | 'ReExport';

export type SymbolMode =
  | 'Full'
  | { Capped: { reason: string } }
  | { Omitted: { reason: string } };

export interface VizProject {
  name: string;
  files: number;
  edges: number;
  symbols: number | null;
  call_edges: number | null;
  built_at: number | null;
}

export interface VizFile {
  id: string;
  label: string;
  language: string;
  deps: number;
  dependents: number;
  symbol_count: number;
  // C-24 (DRIFT-7 Path A): optional decomposition-level / status fields.
  // Populated only by the decomposition view (b33_build_decomposition_view).
  // GraphWindow's cytoscape selectors use these to pick shape + color.
  level?: string;
  status?: string;
}

export interface VizFileEdge {
  source: string;
  target: string;
  kind: VizEdgeKind;
  cyclic: boolean;
}

export interface VizSymbol {
  id: string;
  file: string;
  name: string;
  qualified_name: string;
  kind: string;
  line: number;
}

export interface VizSymbolEdge {
  source: string;
  target: string;
  /** elai_knowledge::symbol_graph::types::Confidence (Local|Unique|MultipleCandidates|Unresolved) */
  confidence: 'Local' | 'Unique' | 'MultipleCandidates' | 'Unresolved';
}

export interface VizData {
  project: VizProject;
  files: VizFile[];
  file_edges: VizFileEdge[];
  symbols: VizSymbol[];
  symbol_edges: VizSymbolEdge[];
  symbols_by_file: Record<string, VizSymbol[]>;
  symbol_mode: SymbolMode;
}

export interface ImpactOverlay {
  files: string[];
  by_depth: Record<string, string[]>;
  total_files: number;
  max_depth_reached: number;
  truncated: boolean;
}

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
  | 'web-quality'
  // B-42 — Cosmos artifact tier drawer.
  | 'cosmos'
  // C-23 — Sessions drawer (BranchTree + per-branch SessionDag actions).
  | 'sessions'
  // D-1 — Reasoning template archive (MutationArchivePanel).
  | 'mutation-archive'
  // D-2 — PPM (Process Preference Model) verifier scores.
  | 'ppm-scores'
  // D-3 — multi-verifier consensus aggregator rows.
  | 'verification-consensus'
  // D-4 — tripwire stack (KL / Δρ / IQR / perplexity / DDM auto-revert).
  | 'tripwires'
  // E-30 — public-agent-incident drawer (argv-allowlist denies, trust
  // dialog defers, MCP pin refusals, lineage collisions, untrusted memory
  // writes, postmortems).
  | 'incidents'
  // D-19 — PRMBench adversarial-gaming benchmark runner.
  | 'prmbench'
  // D-23 — Sycophancy detection benchmark runner (D-14 substrate).
  | 'sycophancy-bench'
  // D-5 — BAI (Best-Arm Identification) Thompson-sampling promotion arms.
  | 'bai-arms'
  // D-6 — Held-out task store + 9-gram contamination filter.
  | 'held-out'
  // D-7 — Cross-domain transfer canary (5 orthogonal task families).
  | 'canary'
  // D-9 — L3 frozen-surface registry + audit-run timeline.
  | 'locked-keys'
  // D-10 — mode-routing predicate distribution + recent rows.
  | 'mode-routing'
  // D-21 — mode-routing coverage benchmark (macro F1 + per-mode F1 + 5x5 confusion).
  | 'mode-routing-bench'
  // D-11 — constructive-checker registry + recent results.
  | 'checker-registry'
  // D-13 — Adversary (cross-family) panel: structured counter-evidence +
  // consensus verdict transitions emitted by run_adversarial_pass.
  | 'adversary'
  // D-12 — OperationalizationCheck baby-step predicate panel: recent
  // baby_step_dispatched SSB rows grouped by parent_task_id + pass
  // rate + per-checker-kind histogram.
  | 'baby-step'
  // D-16 — Conformal calibration (UniCR) panel: tau + corpus + per-claim
  // abstain/answer histogram emitted by conformal_calibration producers.
  | 'conformal-calibration'
  // D-22 — Conformal calibration validation panel: per-bucket refusal
  // rate + UniCR distribution-shift band gate emitted by
  // `ConformalValidationRunner::run_all`.
  | 'conformal-validation'
  // D-15 — Discovery-vs-search NoveltyClass panel: recent
  // novelty_classified SSB rows + per-class distribution +
  // discovery-candidate flagging.
  | 'novelty'
  // D-18 — typed-ObservableEffect predicate panel: recent
  // observable_effect_verified SSB rows + per-sandbox-tier usage +
  // per-outcome histogram + confound-flag chips.
  | 'observable-effect'
  // D-17 — Honest-refusal SLA gauge: current rate + healthy-band +
  // last breach + sparkline of recent rates.
  | 'refusal-sla'
  // D-20 — ARC-AGI-2 OOD probe: recent arc_benchmark_completed runs +
  // verdict banner + per-run dominant mode/novelty class.
  | 'arc-bench'
  // E-46 — Hierarchical decomposition tuning (dual ledger + AGENTLESS
  // shortcut taxonomy + goal-drift canary gauge).
  | 'planning-ledger'
  // E-5 — User modeling + long-term memory drawer.
  | 'memory-facts'
  // E-1 dashboard slice — chat-memory (MemoryNotes + CoreBlock).
  | 'chat-memory'
  // E-35 — XGrammar grammar-cache visibility.
  | 'grammar-compile'
  // D-26 — Full-page archive explorer (search + lineage graph + summary).
  | 'archive-explorer'
  // D-27 — Discovery queue drawer (pending DiscoveryCandidate templates
  // awaiting adversarial verification).
  | 'discovery-queue'
  // D-27 — Full-page mode-routing + refusal-SLA dashboard view
  // (composes ModeRouting + Novelty + ConformalCalibration +
  // RefusalRateGauge panels via the get_mode_routing_refusal_combined
  // composite read).
  | 'mode-routing-dashboard'
  // E-29 — Failure modes drawer (agentic observability: tool-bypass,
  // multi-step reasoning failures, attribution-loss, sycophantic-confirm,
  // OTel span breakages emitted via AgenticObservabilityEvent SSB rows).
  | 'failure-modes';

// ------------------------------------------------------------------
// Agent roles
//
// NOTE: this TS type uses kebab-case identifiers (`'prompt-engineer'`)
// for historical reasons across 31+ usages. The Rust enum
// `ipc_types::AgentRole` uses `#[serde(rename_all = "snake_case")]`,
// so the wire format is snake_case (`'prompt_engineer'`).
//
// AgentRole intentionally stays at 8 variants because every consumer
// (ModelRouting, mock-data, SettingsWindow.modelRouting Record) was
// authored against this shape. The full 16-variant Rust enum (B-40
// idea-framer, B-41 per-level auditors) is mirrored separately by
// `RuntimeAgentRole` below for the multi-runtime adapter surface.
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

/** Full 16-variant agent role list mirroring `ipc_types::AgentRole`.
 *  Used by RuntimePanel + the `set_role_runtime` / `list_runtimes`
 *  Tauri commands. Wire format on the Rust side is snake_case
 *  (`'prompt_engineer'`); the resolver accepts both this kebab-case
 *  spelling and the canonical snake_case spelling so the dashboard
 *  can keep its historical formatting. */
export type RuntimeAgentRole =
  | AgentRole
  // B-40 pre-decomposition framing roles.
  | 'idea-framer'
  | 'idea-critic'
  // B-41 per-level auditor tiers.
  | 'auditor-concepts'
  | 'auditor-phases'
  | 'auditor-steps'
  | 'auditor-baby-steps'
  | 'auditor-baby-step-detail'
  // Plan-mode shim (sometimes surfaced separately by the orchestrator).
  | 'plan-auditor';

/** Stable iteration order matching `ipc_types::AgentRole`. RuntimePanel
 *  uses this to populate the per-role dropdown. */
export const RUNTIME_AGENT_ROLES: readonly RuntimeAgentRole[] = [
  'executor',
  'auditor',
  'researcher',
  'designer',
  'prompt-engineer',
  'context-compiler',
  'memory-writer',
  'media-validator',
  'idea-framer',
  'idea-critic',
  'auditor-concepts',
  'auditor-phases',
  'auditor-steps',
  'auditor-baby-steps',
  'auditor-baby-step-detail',
  'plan-auditor',
] as const;

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

// B-7 (SCHEMA_VERSION 100): producer-wire surfaces. Hand-authored TS
// mirrors of orchestrator/src/ssb.rs payload structs (not ts-rs — the
// ssb.rs file is hand-curated). Snake-case field names match the rmp
// serialization byte-for-byte; do NOT camelCase.
export interface PackageRefVerified {
  name: string;
  registry: string;
  version: string | null;
  verified: boolean;
}

// B-7 (SCHEMA_VERSION 100): mirrors orchestrator FailureRecordedPayload
// (which mirrors pipeline_tree_navigator::FailureRecord). Used by the
// PlanWindow per-baby-step retry pane and any future failure timeline.
export interface FailureRecord {
  baby_step_id: number;
  step_id: number;
  phase_id: number;
  concept_id: number;
  reason: string;
  attempts: number;
}

// ------------------------------------------------------------------
// Knowledge / Search
// ------------------------------------------------------------------
// C-8 Task #7: the live get_search_mode command returns the lowercase
// payload-discriminator tags emitted by the SearchModeChanged SSB row
// (mode: 'hybrid' | 'lexical' | 'semantic' | 'ann'). Legacy callers using
// the uppercase enum literals are preserved so older drawers / stubs still
// compile while migration is in progress.
export type SearchMode =
  | 'HYBRID'
  | 'SPARSE_ONLY'
  | 'BM25_ONLY'
  | 'hybrid'
  | 'lexical'
  | 'semantic'
  | 'ann';

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
// E-17 SP4: RAG retrieval-quality beacon rows.
//
// Hand-typed mirrors of the four `get_*_recent` Tauri commands in
// dashboard-v2/src-tauri/src/commands.rs. The producer (E-17 SP2)
// emits the underlying SsbPayload variants (DeepPoolCappedPayload,
// RrfQualityGateFiredPayload, HydeSkippedCodeQueryPayload,
// RrfComponentWeightsPayload) — see orchestrator/src/ssb.rs:7832-7873.
// The Tauri command reshapes each row's payload_json into the flat
// shapes below for KnowledgeDrawer consumption.
// ------------------------------------------------------------------

/** Mirror of the `get_deep_pool_capped_recent` per-row shape.
 *  Emitted once per DEEP-tier query when `knowledge::deep::deep_pool_cap`
 *  drops candidates above the per-stage cap. */
export interface DeepPoolCappedRow {
  seq: number;
  ts: number;
  capped_count: number;
  max_allowed: number;
  stage_count: number;
}

/** Mirror of the `get_rrf_quality_gate_recent` per-row shape.
 *  Emitted once per DEEP-tier query when `knowledge::deep::rrf_quality_gate`
 *  drops fused candidates below the RRF floor. */
export interface RrfQualityGateRow {
  seq: number;
  ts: number;
  dropped_count: number;
  floor: number;
  total_before: number;
}

/** Mirror of the `get_hyde_skipped_code_query_recent` per-row shape.
 *  Emitted when `knowledge::hyde::should_use_hyde` returns false on a
 *  CodeSymbol / Mixed query that would otherwise have invoked HyDE. */
export interface HydeSkippedCodeQueryRow {
  seq: number;
  ts: number;
  query_class: string | null;
  word_count: number;
  code_signal_present: boolean;
}

/** Mirror of the `get_rrf_component_weights_recent` per-row shape.
 *  Each weight is `Option<f64>` on the Rust side — `null` here means
 *  the ranker did not contribute to that fusion run (e.g. code_vec is
 *  null when ELAI_CODE_VEC is disabled or no code index is loaded). */
export interface RrfComponentWeightsRow {
  seq: number;
  ts: number;
  bm25_weight: number | null;
  code_vec_weight: number | null;
  text_vec_weight: number | null;
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

// C2: credential form field definition (mirrors Rust CredentialField struct).
export interface CredentialField {
  id: string;
  label: string;
  secret: boolean;
  default?: string;
}

// C2: intervention type string union (snake_case matches Rust serde rename_all).
export type InterventionTypeName =
  | 'approve_change'
  | 'replan_approval'
  | 'error_recovery'
  | 'request_credential';

export interface InterventionRequest {
  id: string;
  title: string;
  description: string;
  file?: string;
  diff_summary?: string;
  created_ts: number;
  // C2: present when intervention_type === 'request_credential'.
  intervention_type?: InterventionTypeName;
  credential_fields?: CredentialField[];
}

export type CardKind = 'status' | 'intervention' | 'error' | 'disambiguation' | 'steering' | 'user' | 'response' | 'code-answer' | 'credential-request';

export interface CommandCard {
  id: string;
  ts: number;
  kind: CardKind;
  payload: CardPayload;
  // E-4 chat UX critique meta. Optional; attached when the producer
  // emits a ChatUxCritiqueEvent SSB row tied to this card (wire-dark
  // unless ELAI_CHAT_UX_CRITIQUE_ENABLE=1).
  chat_ux_meta?: ChatUxCardMeta;
}

// ------------------------------------------------------------------
// E-4 chat UX critique — bubble-level overlay types.
// Mirrors crates::orchestrator::chat_ux module wire shapes
// (ConfidenceTier / EvidenceRef / ClaimWithConfidence / HonestCorrection
//  / SycophancyVerdict). The dashboard renders these inline as
// ClaimConfidenceChip + HonestCorrectionBlock + SycophancyGuardBadge.
// ------------------------------------------------------------------
export type ChatUxConfidenceTier =
  | 'verified'
  | 'read_from_source'
  | 'from_training'
  | 'guessed';

export type ChatUxEvidenceRef =
  | { kind: 'file_ref'; value: string }
  | { kind: 'ssb_seq'; value: number }
  | { kind: 'det_check'; value: string };

export interface ChatUxClaimWithConfidence {
  text: string;
  tier: ChatUxConfidenceTier;
  evidence_ref: ChatUxEvidenceRef | null;
}

export type ChatUxHonestCorrectionTrigger =
  | 'test_failed'
  | 'doc_disagreed'
  | 'user_corrected';

export interface ChatUxHonestCorrection {
  original_claim: string;
  correction_text: string;
  trigger: ChatUxHonestCorrectionTrigger;
  original_claim_blake3_hex?: string;
}

export type ChatUxSycophancyVerdict =
  | { kind: 'hold_position'; evidence_ref: ChatUxEvidenceRef }
  | { kind: 'no_evidence' };

export interface ChatUxCardMeta {
  claims?: ChatUxClaimWithConfidence[];
  correction?: ChatUxHonestCorrection;
  sycophancy?: ChatUxSycophancyVerdict;
}

export type CardPayload =
  | StatusCardPayload
  | InterventionCardPayload
  | CredentialRequestCardPayload
  | ErrorCardPayload
  | DisambiguationCardPayload
  | SteeringCardPayload
  | UserCardPayload
  | ResponseCardPayload
  | CodeAnswerCardPayload;

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

// C2: credential request card — separate CardKind so the render branch is
// unambiguous (avoids sub-branching inside the 'intervention' block).
export interface CredentialRequestCardPayload {
  kind: 'credential-request';
  request_id: string;
  description: string;
  fields: CredentialField[];
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

export interface KnowledgeHit {
  path: string;
  line_start: number;
  line_end: number;
  score: number;
  chunk_kind: string;
  snippet: string;
}

export interface CodeAnswerCardPayload {
  kind: 'code-answer';
  query: string;
  hits: KnowledgeHit[];
  status: 'ok' | 'not_indexed' | 'error';
  message?: string;
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
  /** C-22: distinct feasibility probes recorded in this view's window.
   *  Currently 0 or 1 (viewer keeps the latest report only). */
  probe_count?: number;
  /** C-22: alternative slots attempted across all stub runs. */
  alternatives_explored?: number;
  /** C-22: distinct novel steps in the latest probe report. */
  novel_step_count?: number;
  /** C-22: skip_reason from the most-recent feasibility report, if any. */
  skip_reason?: string | null;
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
  | 'MEDIA_VALIDATE'
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

/** E-15: backend selected by orchestrator::sandbox::select_backend(). */
export type SandboxBackendKind =
  | 'Firecracker'
  | 'Gvisor'
  | 'AppContainer'
  | 'None';

export interface SandboxAgentPathCount {
  role: string;
  path_count: number;
}

/** E-15: a single researcher-net egress reject sampled from the orchestrator
 *  ring buffer. `ts_ms` is wall-clock millis, `dest_ip` is the destination
 *  attempted, `reason` is the iptables rule that fired the REJECT. */
export interface SandboxResearcherNetBlockedAttempt {
  ts_ms: number;
  dest_ip: string;
  reason: string;
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
  // ---- E-15 (additive) ------------------------------------------------
  /** Sandbox backend selected at runtime. `None` when no isolation is active
   *  (e.g. browser-dev fallback). */
  backend_selected: SandboxBackendKind;
  /** FirecrackerHotPool snapshot-restore p50 in milliseconds, or null when
   *  the pool has not recorded any restores yet (cold start / KVM absent). */
  restore_p50_ms: number | null;
  /** Number of warm snapshots resident in the FirecrackerHotPool. */
  snapshot_count: number;
  /** Process-local counter of `regenerate_uniqueness_seeds` calls. */
  uniqueness_regenerations_count: number;
  /** Hostnames pinned in the researcher-net iptables OUTPUT allowlist
   *  (sourced from [researcher.net].allowed_hosts in elai_cc.toml). */
  researcher_net_allowed_hosts: string[];
  /** Sample (last 10) of researcher-net egress attempts blocked by the
   *  REJECT rule. Empty when no rejects have been recorded. */
  researcher_net_blocked_attempts: SandboxResearcherNetBlockedAttempt[];
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

// C-5 follow-on (SCHEMA_VERSION 103): latest cloud routing status surfaced
// to CloudPanel. Each field maps to the most-recent SSB row of its type
// and is `null` when no such row has been written yet.
// Wire shape matches the Rust `get_cloud_route_status` Tauri command.
export interface RouteBlockedRow {
  reason: string;
  spent_usd: number;
  cap_usd: number;
  cap_period: string;
  baby_step_id: number;
  ts_ms: number;
}
export interface WarnLimitReachedRow {
  spent_usd: number;
  cap_usd: number;
  warn_at_percent: number;
  pct_of_cap: number;
  baby_step_id: number;
  ts_ms: number;
}
export interface CostEstimateRow {
  low_usd: number;
  high_usd: number;
  model: string;
  step_count: number;
  heuristic_source: string;
  ts_ms: number;
}
export interface CloudRouteStatus {
  route_blocked: RouteBlockedRow | null;
  warn_limit_reached: WarnLimitReachedRow | null;
  cost_estimate: CostEstimateRow | null;
}

// C-28: T-1 mutation testing gauge surfaced in MonitorWindow.
// Fields mirror the snake_case shape returned by the Rust
// `get_mutation_gauge` Tauri command. MonitorWindow renders the section
// green when `score_pct >= threshold_pct`, amber when below, and a grey
// "no mutation run yet" placeholder when the value is `null` (no run
// recorded for the current session). See
// workflow/reasoning/C-28-decisions.md DECISION-D9 / DECISION-D10.
export interface MutationGaugeInfo {
  score_pct: number;
  threshold_pct: number;
  cache_hits: number;
  total_run: number;
  last_updated_ms: number;
}

// C-32: Code-Completeness Layer 2 -- skeleton-first completeness ratio
// surfaced in MonitorWindow. Polled every 5s next to the mutation gauge.
// `{ known: false }` = no skeleton flushed yet for this session (renders
// a grey "no skeleton flushed yet" placeholder). `{ known: true, ... }`
// drives the colour-coded horizontal gauge: red < 0.5, yellow 0.5..0.9,
// green >= 0.9 per workflow/reasoning/C-32-decisions.md D3 / plan §Task 6.
// Audit-C32 W2 close-out: previously returned bare null; now returns the
// envelope shape so the frontend can branch on a single `known` flag.
export type CompletenessRatioInfo =
  | { known: false }
  | {
      known: true;
      ratio: number;
      blank_count: number;
      last_path: string | null;
    };

// ------------------------------------------------------------------
// REFLECT (B-39) — multi-candidate self-reflection pipeline state.
// Fields use snake_case matching Rust struct field names verbatim.
// Nullable fields use `T | null` (not `undefined`) because serde
// serializes Rust `Option<T>` as JSON `null`.
// ------------------------------------------------------------------
export interface ReflectTriagedPayload {
  turn_id: string;
  class: string;
}

export interface ReflectCandidatePayload {
  turn_id: string;
  candidate_index: number;
  content: string;
}

export interface ReflectAuditVerdictPayload {
  turn_id: string;
  best_index: number;
  rationale: string;
}

export interface ReflectRefinedPayload {
  turn_id: string;
  total_tokens_used: number;
  synthesis: string;
}

export interface ReflectState {
  enabled: boolean;
  last_triage: ReflectTriagedPayload | null;
  last_verdict: ReflectAuditVerdictPayload | null;
  last_summary: ReflectRefinedPayload | null;
}

// ------------------------------------------------------------------
// B-24 Phase 4I Group C8 — Media validator dashboard contract.
// Mirrors src-tauri/src/state.rs MediaValidationEntry + friends.
// Snake_case keys match Tauri's auto-derived JSON.
// ------------------------------------------------------------------
export type Verdict = 'Pass' | 'Fail' | 'HumanReview' | 'Degraded';
export type ReviewDecision = 'Pass' | 'Fail' | 'RequestRevalidate';

export interface CheckResultSnapshot {
  name: string;
  pass: boolean;
  detail: string;
  latency_ms: number;
}

export interface Tier1Snapshot {
  pass: boolean;
  check_count: number;
  latency_ms: number;
  checks: CheckResultSnapshot[];
}

export interface Tier2Snapshot {
  advisory_pass: boolean;
  model_used: string;
  latency_ms: number;
  // Free-shape VLM JSON output (overall_quality, issues, confidence, reasoning).
  // `null` when the orchestrator emits Tier-2 complete without the structured
  // payload extension.
  structured_output: unknown;
}

export interface CapturedFrameSnapshot {
  adapter_name: string;
  frame_count: number;
  latency_ms: number;
  // file:// or absolute path to thumbnail. The Tauri webview may not load
  // raw file:// — the panel falls back to filename text rendering when
  // image_path is null or unreachable.
  image_path: string | null;
}

export interface IterationSnapshot {
  iteration: number;
  issue_count: number;
  converged: boolean;
  verdict: string | null;
  issues: string[];
}

// E-16 -- AXTree YAML snapshot. Folded onto MediaValidationEntry when
// the orchestrator emits a VlmImageValidationKind::AxtreeMismatch SSB
// row. Mirrors state.rs AxtreeSnapshot.
export interface AxtreeSnapshot {
  yaml_diff_pct: number;
  mismatch: boolean;
  baseline_path: string | null;
  current_path: string | null;
  threshold_pct: number | null;
}

// E-16 -- Sycophancy probe snapshot. Folded onto MediaValidationEntry
// when the orchestrator emits a (REUSED) SycophancyFlag SSB row from
// the media_validator sycophancy_probe path. Verdict label vocab MUST
// match SycophancyFlagPayload.verdict.
export interface SycophancySnapshot {
  flip_detected: boolean;
  delta: number;
  verdict: string;
}

export interface MediaValidationEntry {
  step_id: string;
  request_id: string;
  media_type: string;
  file_path: string;
  tier1: Tier1Snapshot | null;
  tier2: Tier2Snapshot | null;
  render_frames: CapturedFrameSnapshot[];
  feedback_iterations: IterationSnapshot[];
  final_verdict: string | null;
  updated_at: number;
  // E-16 fields (optional so old payloads decode cleanly).
  axtree?: AxtreeSnapshot | null;
  sycophancy?: SycophancySnapshot | null;
  swap_order_disagreement?: boolean | null;
  convergence_history?: number[];
}

// Payload of the Tauri `media_ssb` event re-broadcast by the SSB
// subscriber. Frontend listens with @tauri-apps/api/event::listen.
export interface MediaSsbEvent {
  action: string;
  step_id: string;
  seq: number;
  ts_ms: number;
}

// ------------------------------------------------------------------
// B-40 — Idea-Framer / Domain Research Presets / Critic Loop types.
// Mirror of the Rust serde-tagged types in
// elai_orchestrator::idea_framing (snake_case discriminants).
// ------------------------------------------------------------------
export type ProjectClass =
  | 'consumer_mobile_app'
  | 'web_app'
  | 'saas_dashboard'
  | 'desktop_app'
  | 'game_mod'
  | 'research_tool'
  | 'library_or_sdk'
  | 'cli_tool'
  | 'other';

export type CriticCategory =
  | 'stale'
  | 'slop'
  | 'policy'
  | 'feasibility'
  | 'market'
  | 'scope'
  | 'contradiction'
  | 'design_hard'
  | 'vague'
  | 'missing';

export type CriticSeverity = 'block' | 'warn' | 'info';

export interface CriticFinding {
  category: CriticCategory;
  severity: CriticSeverity;
  evidence_ref?: number | null;
  proposed_fix: string;
}

export interface CriticRound {
  round_index: number;
  findings: CriticFinding[];
}

export interface Tradeoff {
  name: string;
  options: string[];
  default: string;
  user_facing_question: string;
}

export interface TradeoffCard {
  tradeoffs: Tradeoff[];
}

export interface StreamSummary {
  stream_name: string;
  stream_root: number;
  convergence: string;
  converged: boolean;
}

export interface ResearchSummary {
  streams: StreamSummary[];
  degraded_streams: string[];
}

export interface KeyDecision {
  name: string;
  rationale: string;
  source_refs: string[];
}

export interface FramedIdea {
  project_class: ProjectClass;
  raw_prompt: string;
  research_summary: ResearchSummary;
  key_decisions: KeyDecision[];
  known_unknowns: string[];
  tradeoff_card: TradeoffCard;
  critic_rounds: CriticRound[];
}

/// User response to a Tradeoff (one entry per Tradeoff.name). `option`
/// must match an entry in the corresponding Tradeoff.options array.
export interface TradeoffResponse {
  tradeoff_name: string;
  option: string;
}

/// State surfaced by the Tauri `get_idea_framing_state` command. `null`
/// when no FRAMING substate is active for the current session.
export interface IdeaFramingState {
  session_id: string;
  framed_idea: FramedIdea | null;
  /// Highest round_index emitted so far (0..=3). Used by MonitorWindow
  /// for the "Critic rounds: N/3" badge.
  critic_rounds_used: number;
  /// True when an IdeaFramed row with converged=true has been emitted.
  is_converged: boolean;
  /// True when the FRAMING substate is currently active (FSM PLANNING +
  /// agent_substate=Framing).
  is_active: boolean;
}

// ------------------------------------------------------------------
// B-38 — Ambiguity gate (continuous metric)
//
// cross-lang-pair: dashboard/tauri/api
// Mirrors `get_ambiguity_state` in src-tauri/src/commands.rs which currently
// returns mock data; the real producer wires AmbiguityScorer into the gate
// in a follow-up B-N.
// ------------------------------------------------------------------
export type AmbiguityMilestone = 'initial' | 'progress' | 'refined' | 'ready';

export interface AmbiguityComponent {
  score: number;
  weight: number;
  floor: number;
}

export interface AmbiguityComponents {
  goal_clarity: AmbiguityComponent;
  constraint_clarity: AmbiguityComponent;
  success_criteria_clarity: AmbiguityComponent;
  context_clarity: AmbiguityComponent | null; // brownfield-only
}

export interface AmbiguityStreak {
  current: number;
  required: number;
}

export interface AmbiguityState {
  /// 0.0..1.0, rounded to 4 dp by the scorer.
  score: number;
  milestone: AmbiguityMilestone;
  threshold: number;
  brownfield: boolean;
  components: AmbiguityComponents;
  /// Per-dimension component names that breached their floor.
  floor_failures: string[];
  streak: AmbiguityStreak;
  qualifies_for_advance: boolean;
  /// C-2: legacy `mock` field is no longer emitted by the live backend
  /// (replaced with real SSB reads). Kept as an optional flag for
  /// backward compatibility with any cached client state — always
  /// absent in C-2+ responses.
  mock?: boolean;
}

// ------------------------------------------------------------------
// B-42 — Cosmos artifact tier (cross-project knowledge cache).
// Mirrors `elai_knowledge::CosmosArtifactStatusJson` (snake_case).
// `refreshed_at` is RFC3339 UTC or null for never-refreshed artifacts.
// `status` is the snake_case CosmosStatus discriminant.
// ------------------------------------------------------------------
export type CosmosArtifactStatus = {
  name: string;
  project_classes: string[];
  refreshed_at: string | null;
  status: 'fresh' | 'stale' | 'decayed';
  size_bytes: number;
};

// ------------------------------------------------------------------
// B-41 — Per-level auditor mode + verdict surface (mirrors
// orchestrator::auditor_mode::LevelAuditVerdict JSON encoding).
// ------------------------------------------------------------------
export type AuditorMode =
  | 'concepts'
  | 'phases'
  | 'steps'
  | 'babysteps'
  | 'babystep_detail'
  | 'plan'
  | 'production_grade';

export type AuditorTier = 'concept' | 'phase' | 'step' | 'baby_step';

export type AuditVerdict = 'allow' | 'block';

export type AuditFindingSeverity = 'BLOCK' | 'WARN' | 'INFO';

export interface AuditFinding {
  category: string;
  severity: AuditFindingSeverity;
  message: string;
  source?: string | null;
}

export interface TierNodeIdJson {
  tier: AuditorTier;
  id: number;
}

export interface LevelAuditVerdictJson {
  mode: AuditorMode;
  tier_node_id: TierNodeIdJson;
  findings: AuditFinding[];
  verdict: AuditVerdict;
  respawned_once: boolean;
  cascade_to_parent: boolean;
}

// ------------------------------------------------------------------
// B-37 Phase 4M — Multi-runtime backend abstraction.
//
// Mirrors `providers::runtime::RuntimeBackend` (snake_case via serde
// rename_all="snake_case"), `RuntimeHealth` (tag="status",
// rename_all="snake_case"), `RuntimeCapabilities`, plus the
// dashboard-side DTOs `RuntimeInfo` / `TestReport` defined in
// crates/dashboard-v2/src-tauri/src/commands.rs (B-37D wave-2A).
//
// Field names use snake_case to match the Rust serde wire format —
// every other dashboard type follows the same convention (see
// SidecarStatus, AmbiguityState, CosmosArtifactStatus).
// ------------------------------------------------------------------

/** Canonical snake_case identifier for a runtime backend. The 11
 *  variants match `RuntimeBackend::canonical_name()` in
 *  providers/src/runtime/types.rs. */
export type RuntimeBackendId =
  | 'sidecar'
  | 'anthropic_api'
  | 'claude_code'
  | 'codex_cli'
  | 'opencode'
  | 'hermes'
  | 'gemini_cli'
  | 'kiro_cli'
  | 'copilot_cli'
  | 'ollama'
  | 'vllm'
  | 'mlx'
  | 'llama_cpp'
  | 'ppm'
  | 'composer2';

/** Stable iteration order matching `RuntimeBackend::all()`. */
export const RUNTIME_BACKEND_IDS: readonly RuntimeBackendId[] = [
  'sidecar',
  'anthropic_api',
  'claude_code',
  'codex_cli',
  'opencode',
  'hermes',
  'gemini_cli',
  'kiro_cli',
  'copilot_cli',
  'ollama',
  'vllm',
  'mlx',
  'llama_cpp',
  'ppm',
  'composer2',
] as const;

/** Discriminator for the `RuntimeHealth` serde tag. */
export type RuntimeHealthStatus = 'ready' | 'degraded' | 'unavailable';

/** Mirror of `RuntimeHealth` (tag="status"). `reason` is populated
 *  only for `degraded` and `unavailable`. */
export interface RuntimeHealthJson {
  status: RuntimeHealthStatus;
  reason?: string;
}

/** Mirror of `providers::runtime::RuntimeCapabilities`. All-false is
 *  the conservative default surfaced by stub adapters. */
export interface RuntimeCapabilitiesJson {
  supports_tools: boolean;
  supports_streaming: boolean;
  supports_vision: boolean;
  max_context: number;
  max_output: number;
  json_schema_mode: boolean;
  supports_prefill: boolean;
  supports_caching: boolean;
  native_thinking_budget: boolean;
}

/** Wire shape returned by the `list_runtimes` Tauri command.
 *  `is_stub` distinguishes adapters whose `health_check` returns
 *  `Unavailable` because the underlying CLI/runtime isn't wired yet
 *  (B-37 stubs) from adapters that probed and failed at runtime. */
export interface RuntimeInfo {
  /** Display label; usually equal to `canonical_name`. */
  backend: string;
  /** Canonical snake_case identifier (RuntimeBackendId). */
  canonical_name: RuntimeBackendId;
  /** RuntimeAgentRoles currently routed to this backend. Empty when
   *  no role is mapped to it. Uses the 16-variant superset
   *  (RuntimeAgentRole) so B-40 idea-framer + B-41 per-level auditors
   *  surface alongside the historical 8 AgentRole variants. */
  roles: RuntimeAgentRole[];
  /** Last-known health probe. Stub adapters report
   *  `{ status: 'unavailable', reason: 'not implemented' }`. */
  health: RuntimeHealthJson;
  /** True when the adapter is a B-37 stub (not yet wired). */
  is_stub: boolean;
  /** Most recent probe round-trip in ms, if measured. */
  latency_ms?: number | null;
  /** Capability advertisement (optional — older builds may omit it). */
  capabilities?: RuntimeCapabilitiesJson;
}

/** Wire shape returned by the `test_runtime` Tauri command.
 *  `ok` is true only when health probed as `ready`. `error` carries
 *  the `reason` string from `degraded` / `unavailable` outcomes (and
 *  NEVER serialised SecretString — see providers/src/runtime/secret.rs). */
export interface TestReport {
  backend: RuntimeBackendId;
  ok: boolean;
  health: RuntimeHealthJson;
  latency_ms: number;
  error?: string;
}

// ------------------------------------------------------------------
// C-4 — Tauri commands orchestration + cloud spend surface.
// Mirrors SsbViewer DTOs (snake_case) + the Tauri wrapper return
// shapes in src-tauri/src/commands.rs:7745..8060. The `known` boolean
// is the convention used by phase_detail / baby_step_detail /
// action_detail wrappers so cold-start / not-found rows can be
// rendered as an empty-state placeholder without throwing.
// Rust `Option<T>` serializes as JSON `null`, not `undefined`.
// ------------------------------------------------------------------

/** Mirror of `SsbViewer::PhaseDetail` augmented with the `known`
 *  discriminator the Tauri wrapper inserts. When `known === false`
 *  only `phase_id` is meaningful; every other field is absent. */
export type PhaseDetailResponse =
  | {
      known: true;
      phase_id: string;
      description: string | null;
      expected_outcome: string | null;
      dependencies: string[];
      estimated_steps_remaining: number | null;
    }
  | {
      known: false;
      phase_id: string;
    };

/** Mirror of `SsbViewer::BabyStepDetail` augmented with the `known`
 *  discriminator. `tdd` / `acceptance` are free-shape JSON because the
 *  Rust side serializes them as `Option<serde_json::Value>`. */
export type BabyStepDetailResponse =
  | {
      known: true;
      baby_step_id: string;
      task: string;
      estimated_tokens: number;
      concept_name: string;
      phase_name: string;
      step_name: string;
      tdd: unknown | null;
      acceptance: unknown | null;
    }
  | {
      known: false;
      baby_step_id: string;
    };

/** Mirror of `SsbViewer::PhaseStatus`. Empty Vec on cold start. */
export interface PhaseProgressRow {
  phase_id: string;
  status: string;
  step_count: number;
  completed_count: number;
}

/** Mirror of `SsbViewer::SsbActionSummary`. `action_type` is the
 *  snake_case `SsbActionType` discriminant (e.g. `'phase_complete'`,
 *  `'task_submitted'`). Kept as `string` rather than a closed union
 *  because the Rust enum has 100+ variants and the dashboard renders
 *  it generically. */
export interface RecentActionRow {
  seq: number;
  timestamp: number;
  action_type: string;
  agent_id: string;
  summary: string;
}

/** Mirror of `SsbViewer::FileChange`. */
export interface FileChangeRow {
  path: string;
  action: string;
  summary: string;
}

/** Mirror of `SsbViewer::SsbActionDetail` augmented with the `known`
 *  discriminator. `payload_json` is the typed SsbPayload re-serialised
 *  as JSON (msgpack on the wire, JSON at the IPC boundary). */
export type ActionDetailResponse =
  | {
      known: true;
      seq: number;
      ts: number;
      action_type: string;
      agent_id: string;
      payload_json: unknown;
      files: string[];
    }
  | {
      known: false;
      seq: number;
    };

/** Mirror of `commands.rs::ProjectEntry`. `last_opened_ms` is epoch ms;
 *  the dashboard uses it for sort ordering (newest first). */
export interface ProjectEntry {
  name: string;
  path: string;
  last_opened_ms: number;
}

/** `list_projects` Tauri wrapper return shape. `known: false` is
 *  surfaced when the read / parse failed (vs `known: true` + empty Vec
 *  for the legitimate "no projects yet" case). */
export interface ListProjectsResponse {
  known: boolean;
  projects: ProjectEntry[];
}

/** Per-stage research history row.
 *
 *  B-7 (SCHEMA_VERSION 100): rewired to the `web_research_step` SSB rows
 *  filtered to stage-populated entries (no new SsbActionType is fabricated
 *  for per-stage history). Mirrors `WebResearchStepPayload` in
 *  orchestrator/src/ssb.rs verbatim (snake_case fields preserved). Optional
 *  fields are emitted as `""` by the producer when not populated; callers
 *  treat empty strings as "not yet reported". */
export interface ResearchNodeHistoryRow {
  agent_id: string;
  node_id: number;
  action: string;
  phi: number;
  psi: number;
  iteration: number;
  sources_fetched: number;
  sources_used: number;
  /** "reqwest" | "obscura" | "" (empty = pre-patch row). */
  fetcher?: string;
  /** "formulate" | "search" | "extract" | "reflect" | "converge".
   *  Field is omitted (undefined) when the row is a coarse, non-stage
   *  event — viewer filters those out, but the type stays defensive. */
  stage?: string;
  /** "started" | "completed". Omitted on coarse rows. */
  stage_phase?: string;
  /** Stage-specific structured data (query, n_results, etc.). */
  stage_payload?: Record<string, unknown> | null;
}

// ------------------------------------------------------------------
// B-3 — feature-flag + query-tier + rule-transition surfaces.
//
// Workspace policy (orchestrator/src/empire/types.rs:11) is hand-authored
// TS mirrors, NOT ts-rs. These interfaces mirror the corresponding Rust
// shapes verbatim (dashboard-v2/src-tauri/src/commands.rs:get_feature_flags
// / get_query_tier_stats / get_rule_transitions and
// orchestrator/src/ssb.rs::RuleTransitionPayload).
// ------------------------------------------------------------------

/** B-3 #27/#28 + V2 rebaseline. Returned by `get_feature_flags()` at
 *  bootstrap. `autonomous_mode` gates AutoAdvanceStats (Phase 4G);
 *  `self_modification` gates ProposalsPanel (Phase 4H); `dashboard_offline`
 *  mirrors the `ELAI_DASHBOARD_OFFLINE` env var so stub.ts can short-circuit
 *  Tauri invocation when set. */
export interface FeatureFlags {
  autonomous_mode: boolean;
  self_modification: boolean;
  dashboard_offline: boolean;
}

/** B-3 #3. Mirror of `orchestrator::ssb_viewer::QueryTierStats` so the
 *  KnowledgeDrawer can render its per-tier histogram. `sample_size==0`
 *  is the "no data yet" signal — until B-4 lands the producer, the
 *  Tauri command returns this shape verbatim. */
export interface QueryTierStats {
  fast_pct: number;
  standard_pct: number;
  deep_pct: number;
  avg_latency_ms_fast: number;
  avg_latency_ms_standard: number;
  avg_latency_ms_deep: number;
  sample_size: number;
}

/** B-3 #11. Per-row shape returned by `get_rule_transitions(n)`. Mirrors
 *  `orchestrator::ssb::RuleTransitionPayload` plus the `{seq, ts}` envelope
 *  added by the Tauri command. Status strings are uppercase lifecycle
 *  labels (PROPOSED | CANDIDATE | SHADOW | ACTIVE | RETIRED). */
export interface RuleTransitionRow {
  seq: number;
  ts: number;
  rule_id: string;
  from_status: string;
  to_status: string;
  reason: string;
  timestamp_ms: number;
}

/** B-3 #16 (revised — workspace policy is hand-authored TS, NOT ts-rs).
 *  Mirror of the Rust `SessionHandoffEntry` shape returned by
 *  `get_session_handoffs` so the SettingsWindow handoff list reads the
 *  right keys. The legacy V1 fields `{id, branch, ts}` are replaced by
 *  the canonical `{seq, ts_ms, winner, abandoned, trunk_snap_id,
 *  from_snap_count}` shape. */
export interface SessionHandoffEntry {
  seq: number;
  ts_ms: number;
  winner: string;
  abandoned: string[];
  trunk_snap_id: string;
  from_snap_count: number;
}

// ------------------------------------------------------------------
// D-5 — BAI Thompson-promotion row union.
//
// Source of truth: crates/dashboard-v2/src-tauri/src/commands_d5.rs.
// Hand-authored TS mirror (workspace policy — types.ts is the shared
// surface, ts-rs is NOT used in this crate per CLAUDE.md "no new
// dependencies without confirmation" + the explicit policy comment
// above on line 2339).
//
// Two row variants share the BAI surface:
//   * 'arm_updated'      — mirrors `bai_arm_updated` (SCHEMA 72). Carries
//                          the Beta(alpha, beta) posterior after a pull +
//                          the was_promoted flag.
//   * 'thompson_sampled' — D-5 closure (SCHEMA 110). Carries the Beta
//                          draw used to pick the next arm. Emitted
//                          immediately BEFORE the matching arm_updated
//                          row by `bai::emit_bai_round`.
//
// Joining the two: same `template_id` + adjacent SSB seq numbers.
// Dashboard consumers can render the (sample, posterior, promote?)
// triple in one row, or stream them as separate ticks.
// ------------------------------------------------------------------

/** D-5 (SCHEMA_VERSION 72): mirror of orchestrator BaiArmUpdatedPayload.
 *  `mean_estimate` is `alpha / (alpha + beta)` — the Beta posterior
 *  mean. Computed Tauri-side so the frontend doesn't have to repeat
 *  the divide-by-zero guard (Beta(1, 1) prior makes the denominator
 *  always positive in practice; the explicit guard keeps the contract
 *  honest). */
export interface BaiArmRow {
  template_id: string;
  alpha: number;
  beta: number;
  n_pulls: number;
  was_promoted: boolean;
  mean_estimate: number;
}

/** D-5 closure (SCHEMA_VERSION 110): mirror of orchestrator
 *  ThompsonSampledPayload. Captures the Beta draw the Thompson policy
 *  used to pick the next arm this round. `sampled_value` is always in
 *  [0.0, 1.0]; `arm_id` is the 32-char simple-hex template id matching
 *  the companion BaiArmRow on the same SSB seq. */
export interface ThompsonSampledRow {
  arm_id: string;
  sampled_value: number;
  alpha: number;
  beta: number;
}

/** D-5 closure tagged-union row: either an arm-updated tick OR a
 *  Thompson-sample tick. The dashboard BaiArmsPanel can branch on `kind`
 *  to render the appropriate column set. */
export type BaiUnionRow =
  | ({ kind: 'arm_updated' } & BaiArmRow)
  | ({ kind: 'thompson_sampled' } & ThompsonSampledRow);

// ------------------------------------------------------------------
// D-4 — tripwire stack row (mirror of Rust commands_d4::TripwireRow).
//
// Source of truth: crates/dashboard-v2/src-tauri/src/commands_d4.rs.
// Hand-authored TS mirror (workspace policy — types.ts is the shared
// surface, ts-rs is not used in this crate).
//
// kind union admits:
//   * 'fired'           — single detector firing (TripwireFired row)
//   * 'auto_reverted'   — auto-revert applied (TemplateAutoReverted)
//   * 'fired_concurrent' — D-4 GAP-2: >=2 detectors fired in the same
//                         observe() tick (TripwireFiredConcurrent —
//                         SCHEMA_VERSION 108)
//
// source mirrors `crate::tripwire::SignalSource::as_str` (D-4 GAP-1):
// 'reward' | 'action' | 'perplexity' | 'classifier' | '' (legacy rows
// pre-GAP-1 that lacked the field).
// ------------------------------------------------------------------
export type TripwireKindLabel =
  | 'kl_divergence'
  | 'correlation_drop'
  | 'iqr_outlier'
  | 'perplexity'
  | 'ddm';

export type TripwireSignalSource =
  | 'reward'
  | 'action'
  | 'perplexity'
  | 'classifier'
  | '';

export type TripwireRowKind = 'fired' | 'auto_reverted' | 'fired_concurrent';

/** D-23 — sycophancy detection benchmark row.
 *  Mirrors the orchestrator-side `SycophancyBenchRunCompletedPayload`.
 *  `verdict` is the snake-case label (`"pass"` / `"fail"`).
 *  `fixture_blake3` is the 64-hex blake3 of the canonicalized fixture
 *  corpus the run was computed against. Inconclusive rows are excluded
 *  from the detection_rate / fp_rate denominators per
 *  D-23-decisions.md DECISION-3 (3-arm ProbeVerdict collapse).
 */
export interface SycophancyBenchRow {
  seq: number;
  ts: number;
  tp: number;
  fp: number;
  tn: number;
  fn_: number;
  inconclusive: number;
  detection_rate: number;
  fp_rate: number;
  verdict: string;
  total_fixtures: number;
  fixture_blake3: string;
  runner_name: string;
}

export interface TripwireRow {
  seq: number;
  ts: number;
  kind: TripwireRowKind;
  event_id: string | null;
  tripwire_kind: string | null;
  /** D-4 GAP-1: signal-stream tag (snake_case SignalSource label).
   *  Optional on legacy rows pre-GAP-1; the producer populates it
   *  unconditionally on new rows. */
  source?: TripwireSignalSource | null;
  candidate_id: string | null;
  observed_value: number | null;
  threshold: number | null;
  fired_at_ms: number | null;
  template_id: string | null;
  parent_template_id: string | null;
  retirement_reason: string | null;
  fired_tripwire_id: string | null;
  /** D-4 GAP-2: second detector kind on concurrent-fire rows. Null on
   *  'fired' / 'auto_reverted' rows. */
  tripwire_a?: string | null;
  tripwire_b?: string | null;
  fired_within_ms?: number | null;
}

// ------------------------------------------------------------------
// D-10 — mode-routing predicate (hand-authored TS mirror of the Rust
// `GroundingMode` + `ClaimType` closed enums in
// crates/orchestrator/src/grounding_mode.rs + crates/orchestrator/src/claim.rs).
//
// The Rust enums use #[serde(rename_all = "snake_case")] so the wire
// labels are these snake_case strings; the panels render UPPERCASE
// human labels separately.
//
// ts-rs is NOT used in this codebase (per D-5 DECISION-7 + D-6
// DECISION-8); the mirror is hand-authored. If a future plan adopts
// ts-rs, regenerate from the Rust source rather than editing here.
//
// Grounding: arxiv:1908.04734 (Everitt current-RF principle, T1) — the
// orchestrator (NOT the policy model) picks the grounding mode for
// every claim. These constants are read-only on the UI side.
// ------------------------------------------------------------------

export type GroundingMode =
  | 'constructive'
  | 'falsifiability_cross_family'
  | 'conformal_refusal'
  | 'operational_deferred'
  | 'conservative_fallback';

export const GROUNDING_MODES: ReadonlyArray<GroundingMode> = [
  'constructive',
  'falsifiability_cross_family',
  'conformal_refusal',
  'operational_deferred',
  'conservative_fallback',
];

export const GROUNDING_MODE_LABELS: Readonly<Record<GroundingMode, string>> = {
  constructive: 'CONSTRUCTIVE',
  falsifiability_cross_family: 'FALSIFIABILITY (CROSS-FAMILY)',
  conformal_refusal: 'CONFORMAL REFUSAL',
  operational_deferred: 'OPERATIONAL DEFERRED',
  conservative_fallback: 'CONSERVATIVE FALLBACK',
};

export type ClaimType =
  | 'factual'
  | 'procedural'
  | 'normative'
  | 'speculative'
  | 'hypothetical'
  | 'computational'
  | 'meta_query';

export const CLAIM_TYPES: ReadonlyArray<ClaimType> = [
  'factual',
  'procedural',
  'normative',
  'speculative',
  'hypothetical',
  'computational',
  'meta_query',
];

// Shape of one `claim_routed` SSB row as returned by the Tauri
// `get_recent_claim_routings` command (mirror of Rust
// `agent_pool::ClaimRoutedRow`).
export interface ClaimRoutedRow {
  claim_id: string;
  claim_type: string;
  mode: string;
  ts_ms: number;
  ctx_summary_blake3: string;
}

// Shape of the histogram aggregation surface returned by the Tauri
// `get_recent_claim_routings` command (mirror of Rust
// `commands_d10::RecentClaimRoutings`).
export interface RecentClaimRoutings {
  rows: ClaimRoutedRow[];
  counts_by_mode: Record<string, number>;
  total: number;
}

// ------------------------------------------------------------------
// D-15 — NoveltyClass mirror (shared with NoveltyPanel.svelte +
// DiscoveryQueueDrawer.svelte).
//
// Hand-authored per the D-5/D-6/D-10/D-13 ts-rs-not-adopted
// precedent — keeping the type co-located with the rest of the
// dashboard wire surface so future drawers can `import { NoveltyClass,
// NoveltyClassifiedRow } from '$lib/api/types'` without redefining the
// shape. The Rust enum is `crate::novelty::NoveltyClass` with
// `#[serde(rename_all = "snake_case")]`; the wire labels below match
// the Rust as_str() output exactly.
//
// L3-LOCKED — 3-arm closed taxonomy per arXiv:2406.05761 (LLM Novelty
// Detection) §4 + workflow/reasoning/D-15-decisions.md DECISION-3.
// Adding a fourth bucket is a plan-level schema change.
// ------------------------------------------------------------------
export type NoveltyClass = 'search' | 'distant_search' | 'discovery_candidate';

// Shape of one `novelty_classified` SSB row as returned by the Tauri
// `get_novelty_recent` command (mirror of Rust
// `crate::novelty::NoveltyClassifiedRow`).
export interface NoveltyClassifiedRow {
  template_id: string;
  class: string;
  dkg_match_count: number;
  near_duplicate_threshold: number;
  ts_ms: number;
}

// Per-class count snapshot for the dashboard distribution chart
// (mirror of Rust `crate::novelty::NoveltyDistribution`).
export interface NoveltyDistribution {
  total: number;
  search: number;
  distant_search: number;
  discovery_candidate: number;
}

// ------------------------------------------------------------------
// D-18 closure — typed-ObservableEffect 4-variant pack.
// Mirrors crates/orchestrator/src/observable_effect.rs +
// crates/orchestrator/src/ssb.rs +
// crates/dashboard-v2/src-tauri/src/commands_d18.rs.
//
// Paper anchors (DECISION-6 substituted citations):
//   Voyager arxiv:2305.16291 §3 (skill-with-measurable-effect)
//   Eureka arxiv:2310.12931 §3.3 (component-variation Goodhart guard)
//   Coscientist arxiv:2304.05332 §4 (sandbox vs production observation)
// ------------------------------------------------------------------

/// Discriminated-union kind label for the 4 typed-ObservableEffect SSB
/// variants. Matches the snake-case wire tag emitted by
/// `SsbActionType::as_sql_str` on the Rust side.
export type ObservableEffectKind =
  | 'observable_effect_verified'
  | 'observable_effect_dispatched'
  | 'goodhart_risk_flag'
  | 'sandbox_observation'
  | 'production_observation_proposal';

/// Closed snake-case set for `ObservableEffectRow.outcome` (the legacy
/// `observable_effect_verified` outcome surface — DISTINCT from
/// `ObservationOutcome` below).
export type VerificationOutcomeLabel = 'pass' | 'fail' | 'skipped';

/// Closed snake-case set for `SandboxObservation.outcome` and
/// `ObservationContext.kind`-paired observations (Coscientist §4
/// taxonomy). DISTINCT from `VerificationOutcomeLabel`.
export type ObservationOutcomeLabel =
  | 'confirmed'
  | 'refuted'
  | 'inconclusive';

/// Closed snake-case set for `SandboxTier`.
export type SandboxTierLabel =
  | 'no_sandbox'
  | 'process_isolated'
  | 'container_isolated'
  | 'network_blocked';

/// Mirror of Rust `ObservableEffectRow` + `ObservableEffectRowDto`.
export interface ObservableEffectRow {
  effect_id: string;
  trigger_kind: string;
  observation_kind: string;
  sandbox_tier: string;
  outcome: string;
  confound_flags: string;
  elapsed_us: number;
  ts_ms: number;
}

/// Mirror of Rust `ObservableEffectDispatchedPayload` +
/// `ObservableEffectDispatchedDto`.
export interface ObservableEffectDispatched {
  effect_id: string;
  trigger: string;
  context_kind: string;
  ts_ms: number;
}

/// Mirror of Rust `GoodhartRiskFlagPayload` + `GoodhartFlagDto`.
/// Per Eureka §3.3 component-variation invariant.
export interface GoodhartFlag {
  effect_id: string;
  risk_kind: string;
  evidence: string;
  ts_ms: number;
}

/// Mirror of Rust `SandboxObservationPayload` +
/// `SandboxObservationDto`. Per Coscientist §4 sandbox taxonomy.
export interface SandboxObservation {
  effect_id: string;
  tier: string;
  outcome: string;
  evidence_count: number;
  ts_ms: number;
}

/// Mirror of Rust `ProductionObservationProposalPayload` +
/// `ProductionObservationProposalDto`. Per Coscientist §4 invariant,
/// `requires_human_review` is `true` whenever `instrument_file` is
/// `null`/`undefined`.
export interface ProductionObservationProposal {
  effect_id: string;
  instrument_file: string | null;
  requires_human_review: boolean;
  ts_ms: number;
}

/// Mirror of Rust `RecentObservableEffects`. Additive extension carries
/// the 4 new typed-ObservableEffect families alongside the legacy
/// `observable_effect_verified` rows.
export interface RecentObservableEffects {
  rows: ObservableEffectRow[];
  counts_by_outcome: Record<string, number>;
  counts_by_sandbox_tier: Record<string, number>;
  total: number;
  dispatched_rows: ObservableEffectDispatched[];
  goodhart_flags: GoodhartFlag[];
  sandbox_observations: SandboxObservation[];
  production_proposals: ProductionObservationProposal[];
}

// ------------------------------------------------------------------
// D-26 GAP-3 — flat ArchiveVerifyResult projection.
//
// Mirror of Rust `commands_d26::ArchiveVerifyStatus`. Source of truth:
// crates/dashboard-v2/src-tauri/src/commands_d26.rs::ArchiveVerifyStatus.
// Hand-authored (workspace policy — ts-rs not used in this crate).
//
// Producer: `elai archive verify` CLI driver (D-8 SCHEMA_VERSION 115).
// Consumer: ArchiveVerifyGauge.svelte (inline widget, NOT a drawer).
//
// Empty state (no archive_verify_result row yet):
//   verified=false, total_entries=0, error_count=0,
//   last_verified_ts_ms=null, errors=[]. Distinguished from a true
//   verify failure via `last_verified_ts_ms === null`.
//
// `errors` carries short kind labels (e.g. "HashMismatch",
// "BrokenParent") extracted from the externally-tagged
// `ArchiveVerifyError` enum on the wire.
// ------------------------------------------------------------------
export interface ArchiveVerifyStatus {
  verified: boolean;
  total_entries: number;
  error_count: number;
  last_verified_ts_ms: number | null;
  errors: string[];
}

// ------------------------------------------------------------------
// D-27 closure pass — additive read-only DTOs.
//
// Source of truth for SycophancyFlagRow:
//   crates/dashboard-v2/src-tauri/src/commands_d27.rs::SycophancyFlagRow
// Source of truth for CheckerResultRow + RecentCheckerResults:
//   crates/dashboard-v2/src-tauri/src/commands_d11.rs::
//     {CheckerResultRowDto, RecentCheckerResults}
//
// Hand-authored TS mirror (workspace policy — types.ts is shared
// surface, ts-rs not in use per D-5/D-6/D-26 decisions). If a future
// plan adopts ts-rs, regenerate from the Rust source rather than
// editing here.
// ------------------------------------------------------------------

/** D-27 GAP-1 — projected `sycophancy_flag` row consumed by
 *  SycophancyFlagStrip. Mirrors orchestrator-side
 *  SycophancyFlagPayload (D-14 substrate) field-wise.
 *
 *  `verdict` is one of `convergent` | `divergent_sycophancy_flag` |
 *  `inconclusive` (snake-case per
 *  `crate::sycophancy::ProbeVerdict::as_str`). `flip_detected` mirrors
 *  the Sharma §3.2 flip definition. `delta` is the string-similarity
 *  distance in [0.0, 1.0]. */
export interface SycophancyFlagRow {
  seq: number;
  ts_ms: number;
  claim_id: string;
  flip_detected: boolean;
  delta: number;
  verdict: string;
}

/** D-27 GAP-2 mirror — one `checker_result_emitted` row (D-11). */
export interface CheckerResultRow {
  checker_id: string;
  checker_kind: string;
  result_kind: string;
  claim_id: string;
  ts_ms: number;
  reason: string;
}

/** D-27 GAP-2 mirror — D-11 commands_d11::RecentCheckerResults.
 *  `counts_by_kind` is the aggregate histogram seeded with the four
 *  closed-enum result kinds (`pass` / `fail` / `inapplicable` /
 *  `error`); `rows` is the flat tail (newest-first). */
export interface RecentCheckerResults {
  rows: CheckerResultRow[];
  counts_by_kind: Record<string, number>;
  total: number;
}

// ---------------------------------------------------------------------------
// E-23 — developer-discourse positioning + UX-guard trust telemetry types.
// Backed by orchestrator::trust_telemetry::TrustReport (Rust) plus the two
// new commands get_context_state + get_trust_telemetry. Wire-dark unless
// ELAI_DEVELOPER_DISCOURSE_ENABLE=1 on the orchestrator side; the dashboard
// renders the panels regardless (empty zeroed shape when gate is off).
// ---------------------------------------------------------------------------

export interface TopicBudget {
  topic: string;
  remaining_calls: number;
  remaining_chars: number;
}

export interface ContextState {
  fsm_state: string;
  substate: string | null;
  token_budget_pct: number;
  mcp_think_topics: TopicBudget[];
  last_precompact_thresholds: number[];
  /** "passed" | "blocked" | "not_run" — see C-42 follow-up. */
  prompt_injection_check_status: string;
  /** "local" | "cloud_billed" | "" — derived from B-37 runtime_fallback. */
  runtime_choice_badge: string;
}

export interface TrustReport {
  tasks_per_hour: number;
  findings_per_task: number;
  revert_rate: number;
  ungrounded_count: number;
  stagnation_dist: Record<string, number>;
}

// ------------------------------------------------------------------
// E-39 -- cost + token economics dashboard DTOs (mirrors
// crates/dashboard-v2/src-tauri/src/commands_e39.rs).
// Wire-dark unless ELAI_COST_TOKEN_ECONOMICS_ENABLE=1 on the
// orchestrator side.
// ------------------------------------------------------------------

export interface VelocityWindowSnapshot {
  /** snake_case window label: one_min / ten_min / one_hour / twenty_four_hour */
  window: string;
  count: number;
  limit: number;
  /** "requests" | "usd_cents" */
  kind: string;
  ts_ms: number;
}

export interface VelocityCapStatus {
  windows: VelocityWindowSnapshot[];
  grand_total: number;
}

export interface CostRouterDecisionRow {
  session_id: string | null;
  task_id: string | null;
  difficulty: number;
  budget_remaining_usd: number;
  /** "local" | "sonnet" | "haiku" | "deepseek_v4_flash" | "opus" */
  chosen_backend: string;
  /** "low" | "medium" | "high" */
  criticality: string;
  rationale: string;
  ts_ms: number;
}

export interface AutomixVerdictRow {
  session_id: string | null;
  task_id: string | null;
  /** "continue" | "retry" | "escalate" */
  verdict: string;
  reason: string;
  ts_ms: number;
}

export interface FreeTierQuotaRow {
  /** Snake-case backend label: "cerebras_free" | "groq_free" | "deepseek_v4_flash" */
  backend: string;
  /** "ok" | "approaching" | "exceeded" */
  status: string;
  remaining_pct: number;
  ts_ms: number;
}
