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
  | 'symbol_graph'
  // G-5: reviewer-cockpit Diff-Surface-to-Review (DSR) window — renders the
  // vouched/un-vouched bands + the headline DSR from review_surface_scored
  // rows. See workflow/reasoning/G-5-decisions.md.
  | 'cockpit'
  // H-10: sigma/WebGL canvas — full codebase graph with semantic zoom + LOD,
  // dead-code island overlay (topology-suspected vs coverage-confirmed), and
  // PageRank hub overlay. Reuses get_graph_view_data mode='symbol' feed.
  | 'canvas';

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
  // H-10: dead-code island overlay (HD-5). Additive Option fields; existing
  // consumers that do not inspect them are unaffected.
  // true  = topology-suspected dead (no static callers after entrypoints re-attach)
  //         OR coverage-confirmed dead (coverage_count == 0 after llvm-cov run).
  // The two cases are distinguished by coverage_count:
  //   coverage_count === 0   -> confirmed dead (amber-ghost + dashed)
  //   coverage_count === null -> topology-only suspected (amber-neutral ring)
  is_dead_code?: boolean;
  coverage_count?: number | null;
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
  // E-46 D-7 — Decomposition entropy histogram (low_entropy_decomposition SSB rows).
  // Wire-dark unless ELAI_PLANNING_DECOMPOSITION_ENABLE=1.
  | 'decomposition-entropy'
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
  | 'failure-modes'
  // E-29 — Cloud spend drawer (per-run + per-model token cost ledger,
  // backed by the cost_summary + run_cost_breakdown composite reads).
  | 'spend'
  // E-29 — Query trace drawer (sidecar /search query log + result ranking,
  // backed by query_trace SSB rows + search_result_event spans).
  | 'query'
  // E-29 — Transcript drawer (full agent message transcript for a run,
  // backed by message_log SSB rows joined with tool_call + tool_result).
  | 'transcript'
  // C-41 — Application Skills System (9 skills under .elai_cc/skills/,
  // pre-classify routing telemetry via skill_triggered + skill_executed
  // SSB rows). Wire-dark unless ELAI_APPLICATION_SKILLS_ENABLE=1.
  | 'skills'
  // F-1 / A5: per-surface VibeGateDrawer (list_vibegate_blocks command).
  // Default-ON (B1/W-2): VibeGateBlocked rows emit with no env override.
  | 'vibegate'
  // F-2a / A5: per-surface SecurityRulesPanel (get_security_blocks_recent command).
  // Default-ON (B1/W-2): SecurityRuleBlocked rows emit with no env override.
  | 'security-rules'
  // F-9 / Slice D5: DepsSecurityPanel (write_time_slop_status command).
  // Default-ON (B1/W-2): WriteTimeSlopsquatBlocked rows emit with no env override.
  | 'deps-security'
  // F-10 / Slice A5: SecurityMonotonicityDrawer (get_security_monotonicity_blocks).
  // Default-ON (W-2): SecurityClassRegressionBlocked rows emit unless the
  // ELAI_SECURITY_MONOTONICITY_DISABLE kill-switch is set.
  | 'security-monotonicity'
  // F-5 / Slice A5: StalenessQuarantineDrawer (get_staleness_quarantine).
  // No ENABLE gate (W-2): staleness_quarantine_broadcast rows emit at the
  // decay-tick boundary when a fact crosses the staleness threshold.
  | 'staleness-quarantine'
  // F-7 / Slice A5: TemplateRankingDrawer (get_template_ranking).
  // No ENABLE gate (W-2): template_utility_ranked rows emit at the try_execute
  // gate-threshold seam + the CLI. Renders smoothed utility vs raw
  // first_try_ratio side-by-side (the F-7 before/after view).
  | 'template-ranking'
  // F-13 / Slice A5: SessionArchiveDrawer (get_session_archives).
  // Producer gate ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE is default-OFF / wire-dark
  // (forensic opt-in): session_archived rows emit at the run_migration DROP
  // site when a stale-version SSB is cold-archived before the destructive DROP.
  | 'session-archive'
  // G-4 / Slice A: McpRugPullDrawer (get_mcp_rug_pulls).
  // Default-ON (W-2): mcp_rug_pull_blocked rows emit from the live MCP bridge
  // re-pin gate (DefaultMcpBridge::http_list_tools) when a reconnect re-fetch
  // from a previously-approved server no longer matches the blake3 schema pin.
  // The only opt-out is the kill-switch ELAI_MCP_REPIN_DISABLE.
  | 'mcp-rug-pull'
  // G-1 / Slice A: TaintFlowDrawer (get_taint_flows).
  // Default-ON (W-2): taint_flow rows emit from the live data-plane taint gate
  // (production_agent_rpc git_commit / edit arms) when an untrusted ingest (web
  // body / MCP tool output / external file read / sandbox stdout) flows into a
  // destructive sink and the capability gate blocks it. The only opt-out is the
  // kill-switch ELAI_TAINT_PROPAGATE_DISABLE.
  | 'taint-flow'
  // G-2 / Slice A: RraViewerPanel (get_run_artifact).
  // Default-ON (W-2): run_artifact_emitted rows emit at the `elai run` exit when
  // a Reproducible-Run Artifact (.rra) bundle is sealed (SSB action stream +
  // determinism pins + F-gate evidence manifest + blake3 root). The only opt-out
  // is the kill-switch ELAI_RRA_DISABLE.
  | 'rra-viewer'
  // G-3 / Slice A: BlastRadiusDrawer (get_blast_radius_blocks).
  // Default-ON (W-2): blast_radius_blocked rows emit from the deterministic
  // runtime gates (translate_tool_request / run_bash FS-escape arm +
  // fetch_and_extract egress arm) when a fail-closed gate refuses a
  // post-expansion FS target outside the worktree root or a denied egress host.
  // The only opt-out is the kill-switch ELAI_BLAST_RADIUS_DISABLE.
  | 'blast-radius'
  // G-8 / Slice A: AttestationDrawer (get_attestations).
  // Default-ON (W-2): provenance_attestation_emitted rows emit at the `elai run`
  // exit when a signed in-toto v1 provenance attestation is built from the sealed
  // G-2 Reproducible-Run Artifact (the F-gate verified_levels rollup re-expressed
  // + signed with a LOCAL blake3-Merkle proof, re-verifiable OFFLINE). The only
  // opt-out is the kill-switch ELAI_ATTESTATION_DISABLE.
  | 'attestation'
  // G-9 / Slice A: RedactionPanel (get_redaction_stats).
  // Default-ON (W-2): secret_redacted advisory rows emit whenever the write-time
  // secret+PII redactor fires on a typed SsbPayload free-text field at the top of
  // write_payload. The row carries the redacted field identity + stable
  // blake3-keyed token(s) + match count, NEVER the raw secret. Redaction is
  // always on (no enable flag).
  | 'redaction'
  // G-7 / Slice A: ConfidenceDrawer (get_claim_confidence + get_trifecta_quarantines).
  // claim_confidence_scored rows carry the REAL fused per-change confidence
  // feeding the conformal decision (the producer is behind the conformal triple
  // gate); trifecta_quarantined rows are the lethal-trifecta dual-LLM quarantine
  // beacon (default-ON, kill-switch ELAI_TRIFECTA_QUARANTINE_DISABLE only).
  | 'confidence'
  // G-6 / SCHEMA_VERSION 217: SafeDeltaPanel (get_safety_delta_latest).
  // safety_delta_measured rows emitted by `elai bench safedelta`; surfaces the
  // derived SAFEDELTA (gate-ON-vs-OFF marginal residual rate) + 95% bootstrap CI
  // + per-CWE delta + corpus_blake3 + pinned seed + gate_recall + runpod_pending.
  // Tier-1 = static-gate only; Tier-2 dynamic exploit-exec arm is RunPod-gated.
  | 'safe-delta'
  // I-27 / SCHEMA_VERSION 265: ConvergencePanel (get_convergence_curve_rows).
  | 'convergence'
  // G-10 / SCHEMA_VERSION 218: ModelDriftPanel (get_model_drift_status).
  // model_drift_suspected rows emitted at `elai run` exit or `elai model-drift check`.
  // Surfaces identity_key + drift kind + signals + per_family_delta +
  // affected_template_ids + confirmed_by_fingerprint (GD-5 honest badge).
  // Default-ON (W-2); kill-switch ELAI_MODEL_DRIFT_DISABLE only.
  | 'model-drift'
  | 'pagerank';

// ------------------------------------------------------------------
// E-46 D-7 — Decomposition entropy histogram DTOs.
// Mirror the Tauri command surface in commands_e46.rs::EntropyHistogramRow.
// ------------------------------------------------------------------

/** One low_entropy_decomposition SSB row projected for the histogram. */
export interface EntropyHistogramRow {
  parent_node_key: string;
  /** H_children / H_parent, in [0.0, 1.0]. */
  entropy_ratio: number;
  h_parent: number;
  h_children: number;
  threshold: number;
  ts_ms: number;
}

// ------------------------------------------------------------------
// C-41 — Application Skills System DTOs.
// Mirror the Tauri command surface in commands_c41.rs (camelCase on
// the wire). The dashboard does NOT mint skills; it only inspects
// the 9 canonical scaffolds under .elai_cc/skills/.
// ------------------------------------------------------------------

export interface SkillManifest {
  name: string;
  description: string;
  type: string;
  trigger_examples: string[];
  estimated_minutes: number;
  tags: string[];
  version: string;
  scope: string;
}

export interface SkillRoutingEvent {
  timestamp_ms: number;
  action_type: string;
  skill_name_hash: number;
  score: number;
  top_k_rank: number;
  fell_through: boolean;
}

// ------------------------------------------------------------------
// E-25 Tranche N — Compaction-Ladder dashboard surface.
//
// Mirrors the two read-only Tauri commands in
// crates/dashboard-v2/src-tauri/src/commands_e25.rs. The producer is
// the orchestrator's task_runner PreCompact branch, gated by
// ELAI_E25_COMPACTION_LADDER_ENABLE. Wire-dark by default.
// ------------------------------------------------------------------

export interface CompactionLadderStateDto {
  enabled: boolean;
  default_target_tokens: number;
  tiers: string[];
}

export interface CompactionEventDto {
  timestamp_ms: number;
  session_id: string | null;
  tier: string;
  tier_ordinal: number;
  bytes_freed: number;
  budget_before: number;
  budget_after: number;
  target_budget: number;
  wall_ms: number;
}

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

// B-5: one row from the full CSQ verdict log (`get_csq_log` -> SSB
// csq_evaluated rows, every verdict regardless of whether it gated a baby
// step). The command reshapes CsqEvaluatedPayload; `conclusion` carries the
// raw severity label string (e.g. "Critical" / "Warning" / "Info"),
// `guard_present` is the STOP-Verified guard-token presence flag, and
// `blocked` is true when the verdict gated the step (Critical without guard).
export interface CsqLogRow {
  csq_id: string;
  seq: number;
  ts_ms: number;
  conclusion: string | null;
  guard_present: boolean;
  blocked: boolean;
  check_count: number;
}

// B-5: one per-role GRC budget config row (`get_agent_grc_config` ->
// compile-time `default_grc_config` table). `role` doubles as the GRC topic
// key (AgentGrcConfig carries no separate topic_prefix field).
export interface GrcConfigRow {
  role: string;
  max_calls: number;
  max_chars: number;
  topic_prefix: string;
}

// B-5 GAP-1: one row from the anti-rubber-stamp rejection log
// (`get_auditor_rubber_stamp_rejections` -> SSB
// `auditor_rubber_stamp_rejected` rows). The ReasoningDrawer renders
// "Auditor: N rubber stamps rejected this session" from the array
// length. `role` is the auditor role string (always "auditor" today),
// `finding_count` is the observed FINDING count, `expected_min` is the
// gate threshold that was not met.
export interface AuditorRubberStampRejectionRow {
  seq: number;
  ts_ms: number;
  role: string;
  finding_count: number;
  expected_min: number;
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

// B-17 Task 2: per-gate aggregated verdict counts.
export interface GateVerdictCount {
  gate_id: string;
  pass_count: number;
  block_count: number;
  total: number;
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

// B-7 STEP 11: BubbleUpEvent typed surface — mirrors the backend
// BubbleUpTypedPayload (ssb/payloads.rs). `from_level` and `to_level` are
// DecompositionLevel Debug-format strings: "BabyStep" | "Step" | "Phase"
// | "Concept". `action_kind` is the BubbleUpAction discriminant:
// "research_alternative" | "redecompose_step" | "replan_phase" |
// "replan_concept" | "diagnose_and_retry" | "escalate".
// When from_level == to_level the event is a same-level threshold miss;
// when they differ it is a cascade lift (e.g. BabyStep → Step).
export interface BubbleUpEvent {
  from_level: string;
  to_level: string;
  action_kind: string;
  reason: string;
  failure_count: number;
  threshold: number;
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

// B-7 STEP 7: validation error history entry derived from dag_repair_round
// SSB rows. `kind` is the repair action ("reorder" | "substitute"), `round`
// is the repair-loop iteration, `message` is the validation error description.
export interface ValidationErrorEntry {
  kind: string;
  round: number;
  message: string;
}

// B-7 STEP 7: repair log entry — one per dag_repair_round or
// dag_repair_escalated SSB row. `escalated` is true when T_DAG=10 was
// exhausted (dag_repair_escalated row).
export interface RepairLogEntry {
  action_kind: string; // "reorder" | "substitute" | "escalated"
  round: number;
  reason: string;
  escalated: boolean;
  remaining_issues?: number;
}

// B-7 STEP 7: full ExecutablePipeline DAG response from
// get_executable_pipeline Tauri command. Wraps the DAG nodes, validation
// error history, and repair log into one structured object for the
// PlanWindow detail-pane DAG visualizer.
export interface ExecutablePipelineDag {
  nodes: PipelineNode[];
  validation_errors: ValidationErrorEntry[];
  repair_log: RepairLogEntry[];
}

// B-7 STEP 6: mirrors orchestrator DagRepairRoundPayload (ssb/payloads.rs).
// Snake-case field names match the rmp serialization byte-for-byte.
export interface DagRepairEvent {
  action_kind: string; // "reorder" | "substitute"
  round: number;
  reason: string;
}

// B-7 STEP 6: mirrors orchestrator DagRepairEscalatedPayload (ssb/payloads.rs).
// Emitted when T_DAG=10 repair rounds are exhausted without convergence.
export interface DagRepairEscalated {
  rounds: number;
  remaining_issues: number;
  last_error: string;
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

// B-7 Task #8: mirrors orchestrator TaskSplitPayload (extended with
// phase_id + complexity_reason for PhaseSizingSplitter context). Only
// rows with splitter = "phase_sizing" and phase_id != null are returned
// by get_phase_sized_splits; the other optional fields stay for forward
// compatibility with the full payload shape.
export interface PhaseSizedSplit {
  original_task_id: string;
  sub_task_ids: string[];
  depth: number;
  sub_task_summaries: string[] | null;
  splitter: string | null;
  complexity_estimate: number | null;
  phase_id: number | null;
  complexity_reason: string | null;
}

// B-7 Step 9: mirrors orchestrator TreePositionChangedPayload (ssb/payloads.rs).
// Emitted once per DepthFirstNavigator advance() / handle_failure transition.
// current_node is a snake-case node id (e.g. "bs-3", "step-1"); empty when the
// traversal is exhausted. visited/blocked/skipped carry BabyStepId.0 values.
export interface TreePosition {
  current_node: string;
  visited: number[];
  blocked: number[];
  skipped: number[];
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

// B-4: one persisted `hyde_query` SSB row (HydeStage::hypothesize). The
// SSB payload stores the query-bytes blake3 hash + hypothesis CHAR LENGTH
// (not the raw hypothesis text) + cache-hit flag + latency. The
// KnowledgeDrawer QueryTrace inspector renders these as a hypothesis
// preview strip.
export interface HydeQueryRow {
  seq: number;
  ts: number;
  query_hash: string;
  hypothesis_len: number;
  cache_hit: boolean;
  latency_ms: number;
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
  // B-4: count of files the DKG parser failed to ingest on the last index.
  // Surfaced as "X files indexed, Y failed". See get_dkg_stats SSB-FIDELITY
  // note — the current DkgReady SSB row does not yet persist this count, so
  // the command emits 0 until the producer-side payload field is added.
  failed_files: number;
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
  // B-14 STEP 5: +N -M lines per dashboard-design.txt RESPONSE TYPES
  // "What changed (file path, +N -M lines)". Optional — populated by
  // the producer when the diff line counts are known.
  lines_added?: number;
  lines_removed?: number;
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
  // B-14 STEP 5: "What ELAI will do about it (retry strategy, escalation)"
  // per dashboard-design.txt RESPONSE TYPES Error card. Optional —
  // populated by the producer when a retry/escalation plan is known.
  retry_strategy?: string;
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
  // B-14 STEP 5: "Applied to: [current task / all future steps]"
  // per dashboard-design.txt RESPONSE TYPES Steering acknowledgment.
  // Optional — populated by the producer when the scope is known.
  applied_to?: string;
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

// I-2: the 4 arms of the BranchWorkspaceEvent SSB action_type
// ("branch_workspace_event"), mirroring the Rust BranchWorkspaceEventKind
// (orchestrator/src/ssb.rs). The BranchTree surface badges branches with the
// latest workspace event. cross_lang_diff.py keeps this in sync with the Rust
// enum (W-4).
export type BranchWorkspaceEventKind =
  | 'Spawned'
  | 'Merged'
  | 'EscapeBlocked'
  | 'ConflictEscalated';

export interface SessionBranch {
  id: string;
  label: string;
  status: BranchStatus;
  created_ts: number;
  last_activity_ts: number;
  parent_branch_id?: string;
  // I-2: latest CoW workspace barrier event for this branch (badge surface).
  // Optional + additive: existing consumers of SessionBranch are unaffected and
  // the existing BranchStatus field is unchanged.
  workspace_event_kind?: BranchWorkspaceEventKind;
}

// I-2: one decoded BranchWorkspaceEvent SSB row for the BranchTree workspace
// badge surface (C-23). Mirrors orchestrator::ssb::BranchWorkspaceEventPayload.
export interface BranchWorkspaceEventRow {
  branch_id: string;
  kind: BranchWorkspaceEventKind;
  // blake3 hex-prefix of the branch (or merge-target) root path; an identifier.
  worktree_root_blake3: string;
  // For EscapeBlocked/ConflictEscalated: the offending/conflicting path.
  conflict_path?: string;
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
// F-8 / Slice A5 — SCF symbolic-conflict feed. One row per held-pending
// self-mod config variant the SCF symbolic detector flagged against an
// active locked-key invariant (`self_mod_safety::LOCKED_KEYS`). POLICY is
// WARN-then-escalate, never hard fail-closed — these are held-pending
// variants awaiting human escalation, NOT rejections.
//
// Mirrors `orchestrator::ssb_viewer::SymbolicConflictRow` field-for-field
// (serde serializes the Rust field names verbatim over the IPC boundary).
// ------------------------------------------------------------------
export interface SymbolicConflict {
  /** SSB sequence number (newest-first ordering key). */
  seq: number;
  /** Row creation wall-clock ms. */
  ts_ms: number;
  /** blake3 hex-prefix of the conflict tuple (kind + key + candidate_id). */
  fingerprint: string;
  /** Conflict class / symbolic tag. */
  kind: string;
  /** The locked invariant key the variant contradicts. */
  key: string;
  /** Human-readable contradiction detail (proposed vs locked invariant). */
  detail: string;
  /** Stable id of the flagged-and-held-pending config variant. */
  candidate_id: string;
  /** FSM agent sub-state at flag time. */
  fsm_substate: string;
}

// ------------------------------------------------------------------
// F-11 — config-DGM admit-loop feed row. Each entry is one candidate routed
// through the four-guard chain (tripwire blake3 fingerprint, trap-pattern
// scorer, is_security_sensitive, Two-Gate): Admitted (passed all guards +
// inserted), RejectedByGuard (a guard fired — guard_rejected_by names it), or
// LoopClosed (select_parent ran after insert_variant). Wire-dark producer (the
// darwin-loop CLI, gated by feature darwinian-evolve + ELAI_DARWINIAN_LOOP_-
// ENABLE=1); empty in steady state.
//
// Mirrors `orchestrator::ssb_viewer::DarwinianAdmitRow` field-for-field (serde
// serializes the Rust field names verbatim over the IPC boundary).
// ------------------------------------------------------------------
export interface DarwinianAdmit {
  /** SSB sequence number (newest-first ordering key). */
  seq: number;
  /** Row creation wall-clock ms. */
  ts_ms: number;
  /** Discriminator: 'admitted' | 'rejected_by_guard' | 'loop_closed'. */
  kind: string;
  /** Stable id of the candidate config variant the loop evaluated. */
  variant_id: string;
  /** Parent variant id the candidate mutated from (null for a root seed). */
  parent_id: string | null;
  /** Real evaluation score in [0, 1] (the Two-Gate margin basis). */
  alpha: number;
  /** Firing guard on 'rejected_by_guard' (tripwire/trap/security/two_gate); null otherwise. */
  guard_rejected_by: string | null;
  /** blake3 hex-prefix (16 chars) of the candidate config_blob. */
  candidate_blake3: string;
}

// ------------------------------------------------------------------
// H-43 — execution-grounded DGM fitness feed row. Each entry is one
// FitnessEvaluated SSB row emitted per scored candidate in the DGM
// admit loop (darwin_admit_loop::run_one_generation). Wire-dark producer
// (gated by darwinian-evolve feature + ELAI_DARWINIAN_LOOP_ENABLE=1);
// empty in steady state.
//
// Kind discriminator: 'execution_pass_rate' | 'dual_agreement' | 'fitness_regression'.
// fitness_regression means the candidate scored below its parent —
// the promotion gate will reject it (negative margin).
//
// Mirrors FitnessEvaluatedPayload field-for-field (serde serializes the
// Rust field names verbatim over the IPC boundary).
// ------------------------------------------------------------------
export interface FitnessEvaluated {
  /** SSB sequence number (newest-first ordering key). */
  seq: number;
  /** Row creation wall-clock ms. */
  ts_ms: number;
  /** Discriminator: 'execution_pass_rate' | 'dual_agreement' | 'fitness_regression'. */
  kind: string;
  /** Fraction of frozen corpus tests the candidate passed [0, 1]. */
  pass_rate: number;
  /** Dual-execution output-consensus mass (CodeT) [0, 1]. */
  dual_agreement: number;
  /** Number of frozen corpus test cases evaluated. */
  tests_total: number;
  /** Weighted composite fitness = 0.7 * pass_rate + 0.3 * dual_agreement. */
  fitness: number;
  /** ELAI_SEED at evaluation time (GD-2 reproducibility). */
  seed: number;
  /** blake3 hex-prefix (16 chars) of the candidate config_blob. */
  candidate_blake3: string;
}

// ------------------------------------------------------------------
// H-46 — CMP parent-selection feed. Each entry is one CladeSelected SSB row
// emitted by config_archive::select_parent_with_cmp during a DGM generation.
// Wire-dark producer (gated by darwinian-evolve feature +
// ELAI_DARWINIAN_LOOP_ENABLE=1); empty in steady state.
//
// key_used discriminator: 'clade_metaproductivity' | 'immediate_score_fallback'.
// clade_size==0 means ImmediateScoreFallback (no-lineage cold-start guard).
//
// Mirrors `orchestrator::ssb_viewer::CladeSelectionRow` field-for-field (serde
// serializes the Rust field names verbatim over the IPC boundary).
// ------------------------------------------------------------------
export interface CladeSelection {
  /** SSB sequence number (newest-first ordering key). */
  seq: number;
  /** Row creation wall-clock ms. */
  ts_ms: number;
  /** Selection key: 'clade_metaproductivity' | 'immediate_score_fallback'. */
  kind: string;
  /** Stable id of the selected parent variant. */
  node_id: string;
  /** Clade-Metaproductivity aggregate (mean execution-grounded delta). */
  cmp: number;
  /** Count of descendants that contributed to cmp (0 = fallback). */
  clade_size: number;
  /** Immediate sigmoid score of the selected node's own alpha. */
  immediate_score: number;
  /** Snake_case wire string of the selection key used. */
  key_used: string;
}

// ------------------------------------------------------------------
// F-12 — semantic-contradiction feed. Each pair is one
// memory_conflict_detected SSB row whose conflict_kind is
// "semantic_contradiction" (the deterministic-NLI second pass in the live
// memory tick). The InvalidatedOlder rows additionally carry a soft-closed
// older fact row in the FactStore; SurfaceToUser rows leave both valid.
//
// Mirrors `commands_e5::SemanticContradictionDto` field-for-field (serde
// serializes the Rust field names verbatim over the IPC boundary).
// ------------------------------------------------------------------
export interface SemanticContradiction {
  /** Older row id (fact_id_a). */
  fact_id_a: string;
  /** Newer row id (fact_id_b). */
  fact_id_b: string;
  /** Free-form conflict kind (always "semantic_contradiction" here). */
  conflict_kind: string;
  /** SSB sequence number (newest-first ordering key). */
  seq: number;
  /** Row creation wall-clock ms. */
  ts_ms: number;
}

export interface SemanticContradictionsResponse {
  /** Whether an SSB viewer was attached (false in browser-dev / no DB). */
  viewer_attached: boolean;
  /** The semantic-contradiction pairs, newest first. */
  pairs: SemanticContradiction[];
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
  // I-10: cache-economics planner fields. Absent (undefined) when no
  // CacheEconomicsEvent rows exist yet (fresh install / no cloud calls).
  ttl_tier?: string;
  breakeven_reads?: number;
  guard_trips_today?: number;
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
 *  from_snap_count}` shape.
 *
 *  B-13-POLISH-1 finding: this type currently has NO frontend consumer.
 *  The frontend consumes handoff data indirectly via
 *  `get_session_branches` (commands.rs), which wraps each handoff row
 *  into a `SessionBranch` object `{id, label, status, created_ts,
 *  last_activity_ts, parent_branch_id}`. This type is retained as
 *  documentation of the canonical Rust shape; remove it only if the
 *  SettingsWindow is confirmed to never render raw handoff rows. */
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
//
// FINAL TRANCHE B4 / D-15 -- `candidate_status` carries the optional
// lifecycle stage ("queued" / "ratified" / "rejected") for discovery
// candidates; absent for legacy rows + non-DiscoveryCandidate classes.
// The dashboard NoveltyPanel renders a small badge per row when set.
export type DiscoveryCandidateStatus = 'queued' | 'ratified' | 'rejected';
export interface NoveltyClassifiedRow {
  template_id: string;
  class: string;
  dkg_match_count: number;
  near_duplicate_threshold: number;
  ts_ms: number;
  candidate_status?: DiscoveryCandidateStatus | null;
}

// Per-class count snapshot for the dashboard distribution chart
// (mirror of Rust `crate::novelty::NoveltyDistribution`).
export interface NoveltyDistribution {
  total: number;
  search: number;
  distant_search: number;
  discovery_candidate: number;
}

// Aggregated `get_novelty_recent` response (mirror of Rust
// `commands_d15::RecentNovelty`). The Tauri command joins the most-recent
// rows with a per-class distribution snapshot computed over the same
// window so the consumer can render histograms + per-row badges without
// re-walking JS.
export interface RecentNovelty {
  rows: NoveltyClassifiedRow[];
  distribution: NoveltyDistribution;
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
// H-41 — checker-coverage + gap-backlog types (SCHEMA_VERSION 219).
// Source of truth: commands_h41::{CheckerGapBacklogRowDto, CoverageReportDto}.
// Wire-dark: ELAI_CHECKER_GAP_ENABLE=1 required on the writer side.
// ---------------------------------------------------------------------------

/** H-41 mirror of commands_h41::CheckerGapBacklogRowDto.
 *  One ranked backlog entry: how many times a claim-class x taxon pair
 *  emitted a CheckerGapDetected row. HIGH-priority (priority=2) rows are
 *  NP-complete / undecidable — never-LLM-grade (Xu arXiv:2401.11817 RD-3). */
export interface CheckerGapBacklogRow {
  claim_type: string;
  /** Xu Table-3 complexity taxon string (e.g. "np_complete_or_combinatorial"). */
  complexity_taxon: string;
  count: number;
  /** 1 = normal, 2 = HIGH (NP-complete / undecidable). */
  priority: number;
}

/** H-41 mirror of commands_h41::CoverageReportDto.
 *  Live Mode-A coverage fraction for a session. Recomputable from the
 *  SSB journal under ELAI_SEED (GD-2). */
export interface CoverageReport {
  mode_a_fraction: number;
  constructive_count: number;
  total_factual_computational: number;
  session_id: string;
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
  /** E-3 4B — voice-section role (planner / coder / critic / default). */
  current_role: string | null;
  /** E-3 4B — underlying backend name; hidden by default in the chat UI. */
  current_runtime: string | null;
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

// ------------------------------------------------------------------
// F-1 / A5: per-surface DTOs for the split DrawerKey surfaces.
// Mirrors commands_vibegate.rs::VibeGateRow and
// commands_f2.rs::SecurityRuleRow (Rust source of truth).
// Field `ts` (not `ts_ms`) matches the Rust serde output.
// ------------------------------------------------------------------

/** One VibeGateBlocked row for the VibeGateDrawer (DrawerKey vibegate). */
export interface VibeGateRow {
  seq: number;
  ts: number;
  /** "CodeVuln" | "Secret" | "DepHallucinated" | "Skill" */
  kind: string;
  rule_id: string | null;
  cwe: string | null;
  file_path: string;
  source_blake3: string;
  citation: string | null;
  fsm_state: string | null;
}

// ------------------------------------------------------------------
// F-4 Slice B-5 — TestIntegrityEvent DTO (AuditTimelineWindow fabrication badge).
//
// Mirrors Rust `TestIntegrityEventJson` in commands.rs:
//   test_name: String, passed: bool, has_stdout_hash: bool, kind: String
// kind is one of "verified" | "fabrication_blocked" (sql_str arms of
// TestIntegrityEventKind). Cross-lang drift gate: cross_lang_diff.py.
// ------------------------------------------------------------------
export interface TestIntegrityEventJson {
  test_name: string;
  passed: boolean;
  has_stdout_hash: boolean;
  kind: 'verified' | 'fabrication_blocked';
}

/** One SecurityRuleBlocked flat-beacon row for the SecurityRulesPanel (DrawerKey security-rules). */
export interface SecurityRuleRow {
  seq: number;
  ts: number;
  rule_id: string;
  cwe: string;
  file_path: string;
  language: string;
  candidate_blake3: string;
}

/**
 * F-2b / A5 -- the static-gate insecure-rate keystone for the
 * SecurityRulesPanel before/after tile. Mirrors the Rust `InsecureRateRow`
 * in commands_f2.rs (field names exact). These are the REAL ELAI static
 * write-gate (`evaluate_vibegate`) measurements over a SeCodePLT-structured
 * CWE corpus; DISTINCT from the SeCodePLT dynamic secure-coding rate.
 */
export interface InsecureRateRow {
  suite: string;
  model_id: string;
  n_samples: number;
  insecure_rate_before: number;
  insecure_rate_after: number;
  gate_recall: number;
  ts: number;
}

// ------------------------------------------------------------------
// F-9 / Slice D5 -- DepsSecurityPanel DTOs.
// Mirrors Rust structs in commands_f9.rs (camelCase on the wire via
// serde defaults). cross-lang-pair: dashboard/tauri/api.
// ------------------------------------------------------------------

/** One WriteTimeSlopsquatBlocked SSB row for the DepsSecurityPanel (DrawerKey deps-security). */
export interface WriteTimeSlopRow {
  seq: number;
  ts_ms: number;
  /** blake3 64-bit prefix of the dependency name (never the raw name). */
  dep_name_hash: number;
  /** Package ecosystem: "cargo" | "pypi" | "npm". */
  ecosystem: string;
  /** Verdict kind label: "not_found" | "too_new". */
  verdict_kind: string;
  /** Package age in days (only present for the "too_new" arm). */
  registry_age_days: number | null;
}

/** Per-verdict-kind block count for the DepsSecurityPanel summary header. */
export interface WriteTimeSlopCount {
  verdict_kind: string;
  count: number;
}

/** Top-level DTO returned by write_time_slop_status (DrawerKey deps-security). */
export interface WriteTimeSlopStatusDto {
  /** True unless ELAI_HOOKS_DISABLED is set (global hook dispatcher kill-switch). */
  env_active: boolean;
  /** Total number of WriteTimeSlopsquatBlocked rows in the query window. */
  total_blocked: number;
  /** Per-verdict-kind breakdown from the recent rows (sorted by count desc). */
  counts_by_verdict: WriteTimeSlopCount[];
  /** Most recent rows, newest first, capped at 50. */
  recent_rows: WriteTimeSlopRow[];
}

/**
 * One security_class_regression_blocked SSB row for the
 * SecurityMonotonicityDrawer (DrawerKey security-monotonicity, F-10 / A5).
 */
export interface SecurityMonotonicityRow {
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts: number;
  /** Production leg: "refine" | "auto_advance" | "self_mod" (LoopKind label). */
  loop_kind: string;
  /** Stable identifier of the iteration-loop instance. */
  loop_id: string;
  /** Security-class labels newly introduced by this iteration (the regression set). */
  introduced_classes: string[];
  /** Security-class labels present in the prior iteration (the baseline). */
  prior_classes: string[];
  /** blake3 hex-prefix of the regressing candidate edit blob (never raw source). */
  delta_blake3: string;
  /** Snake-case language label (e.g. "rust" | "python" | "typescript"). */
  lang: string;
}

/**
 * Response envelope returned by get_security_monotonicity_blocks
 * (DrawerKey security-monotonicity, F-10 / A5).
 */
export interface SecurityMonotonicityResponse {
  /** Most recent rows, newest first, capped at 50. */
  rows: SecurityMonotonicityRow[];
  /**
   * True when the guard is ENABLED (default-ON; false only when the
   * ELAI_SECURITY_MONOTONICITY_DISABLE kill-switch is set).
   */
  env_active: boolean;
}

/**
 * One SkillLibraryEvent SSB row for the SkillsPanel Trust-Ladder timeline
 * (F-6 / A5). Merges the three arms (skill_acquired / skill_trust_promoted /
 * skill_demoted_stale) into a single newest-first stream.
 */
export interface SkillLibraryRow {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts: number;
  /** Stable id of the learned skill. */
  skill_id: string;
  /** Tier AFTER the event (snake-case wire label: "s1".."s4"). */
  tier: string;
  /** Tier BEFORE the event for a transition; null for an acquisition. */
  prior_tier: string | null;
  /** Fire-point: "acquired" | "trust_promoted" | "demoted_stale". */
  kind: string;
  /** Cumulative successful invocations at event time. */
  invocations: number;
  /** Failure rate (failures / total) at event time. */
  failure_rate: number;
}

/**
 * One rung of the derived current-state Trust-Ladder (F-6 / A5): the tier each
 * skill currently sits at, folded from the newest event per skill_id.
 */
export interface SkillLadderEntry {
  /** Stable id of the learned skill. */
  skill_id: string;
  /** Current tier (snake-case wire label) per the newest event. */
  tier: string;
  /** Cumulative successful invocations at the newest event. */
  invocations: number;
  /** Failure rate at the newest event. */
  failure_rate: number;
  /** SSB row timestamp (epoch millis) of the newest event. */
  ts: number;
}

/**
 * Response envelope returned by get_skill_library (F-6 / A5). Carries the
 * merged newest-first event timeline and the derived per-skill ladder snapshot.
 */
export interface SkillLibraryResponse {
  /** Most recent SkillLibraryEvent rows across all arms, newest first. */
  events: SkillLibraryRow[];
  /** Derived per-skill current-tier snapshot, sorted by skill id. */
  ladder: SkillLadderEntry[];
}

/**
 * One staleness_quarantine_broadcast SSB row for the StalenessQuarantineDrawer
 * (DrawerKey staleness-quarantine, F-5 / A5). Emitted at the orchestrator/CLI
 * memory-decay-tick boundary when a fact crosses the staleness threshold; the
 * cascade demotes downstream DKG concepts to concept_stale=2 + a non-destructive
 * staleness_quarantined marker edge + flags Cosmos artifacts.
 */
export interface StalenessQuarantineRow {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts: number;
  /** Stable id of the quarantined fact. */
  fact_id: string;
  /** Decay staleness ratio at quarantine time (0.0 fresh .. 1.0+ fully stale). */
  staleness_ratio: number;
  /** Count of DKG concepts demoted to concept_stale=2 by this cascade. */
  dependents_demoted: number;
  /** Count of Cosmos artifacts flagged stale by this cascade. */
  cosmos_artifacts_flagged: number;
  /** Queue records DROPPED-OLDEST due to capacity overflow at emit time. */
  dropped_count: number;
}

/**
 * Response envelope returned by get_staleness_quarantine (F-5 / A5). Carries the
 * newest-first staleness_quarantine_broadcast row stream.
 */
export interface StalenessQuarantineResponse {
  /** Most recent rows, newest first, capped at 50. */
  rows: StalenessQuarantineRow[];
}

/**
 * One template_utility_ranked SSB row for the TemplateRankingDrawer
 * (DrawerKey template-ranking, F-7 / A5). Emitted at the
 * CompiledExecutor::try_execute gate-threshold seam and by the
 * `elai template-ranking list` CLI when a crystallized-pipeline template's
 * Laplace-smoothed outcome utility is computed. The drawer renders `utility`
 * (smoothed, "after") next to `first_try_ratio` (raw recorded baseline,
 * "before") side-by-side. A 1/1 template scores utility ~= 0.667 while its
 * first_try_ratio is 1.0; the two converge as sample_n grows.
 */
export interface TemplateRankingRow {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts: number;
  /** Concept-class key the crystallized template is bucketed under. */
  concept_class: string;
  /** Stable hash of the plan skeleton (the other half of the cache key). */
  plan_skeleton_hash: number;
  /** Laplace-smoothed outcome utility (ALPHA=BETA=1.0); the "after" value. */
  utility: number;
  /** Raw recorded baseline (persisted first_try_success_rate); the "before". */
  first_try_ratio: number;
  /** Sample size (total firings = persisted use_count) backing the utility. */
  sample_n: number;
  /** Stable id of the crystallized pipeline template whose utility was ranked. */
  chosen_pipeline_id: string;
}

/**
 * Response envelope returned by get_template_ranking (F-7 / A5). Carries the
 * newest-first template_utility_ranked row stream.
 */
export interface TemplateRankingResponse {
  /** Most recent rows, newest first, capped at 50. */
  rows: TemplateRankingRow[];
}

/**
 * One session_archived SSB row for the SessionArchiveDrawer
 * (DrawerKey session-archive, F-13 / A5). Emitted at the run_migration
 * destructive-DROP site (the Ssb::new production open path) when a stale-version
 * DB is opened with ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE set; the stale ssb.db is
 * cold-archived to workflow/sessions/{id}/archive/ssb-v{N}.sqlite BEFORE the
 * DROP. Field names mirror the Rust `SessionArchiveRow` verbatim (serde).
 */
export interface SessionArchive {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts_ms: number;
  /** The session whose stale-version store was archived. */
  session_id: string;
  /** The on-disk user_version the store carried BEFORE the bump. */
  stale_version: number;
  /** Relative path of the archive file under the per-session workflow dir. */
  archive_path: string;
  /** blake3 hex-prefix of the archived bytes (integrity check). */
  manifest_blake3: string;
  /** Which store: "ssb" (destructive journal) or "session_dag" (precautionary). */
  store_kind: string;
}

/**
 * G-4 / Slice A: one mcp_rug_pull_blocked SSB row for the McpRugPullDrawer
 * (DrawerKey mcp-rug-pull, command get_mcp_rug_pulls). Emitted by the live MCP
 * bridge re-pin gate (DefaultMcpBridge::http_list_tools) when a reconnect-time
 * tools/list re-fetch from a previously-approved server no longer matches the
 * blake3 schema pin. The changed tool(s) are blocked fail-closed until operator
 * re-approval. Field names mirror the Rust `McpRugPullRow` verbatim (serde).
 */
export interface McpRugPull {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts_ms: number;
  /** The MCP server whose re-fetched tool list no longer matched the pin. */
  server_url: string;
  /** blake3 schema hash at pin time (before the rug-pull). */
  schema_blake3_before: string;
  /** blake3 schema hash of the rug-pulled re-fetch (after). */
  schema_blake3_after: string;
  /** Names of the tools whose description/presence changed (line-jump set). */
  changed_tools: string[];
  /** Count of per-tool description swaps (the line-jump signal). */
  description_diff_count: number;
  /** The production entrypoint that re-fetched and tripped the gate. */
  ingest_origin: string;
}

/**
 * G-1 / Slice A: one taint_flow SSB row for the TaintFlowDrawer (DrawerKey
 * taint-flow, command get_taint_flows). Emitted by the live data-plane taint
 * gate (production_agent_rpc git_commit / edit arms) when an untrusted ingest
 * (web body / MCP tool output / external file read / sandbox stdout) flows into
 * a destructive sink and the allow_tool_with_origin capability gate REJECTS the
 * call. Field names mirror the Rust `TaintFlowRow` verbatim (serde).
 */
export interface TaintFlow {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts_ms: number;
  /** Ingest Kind label: "web" / "tool_output" / "file" / "sandbox_output". */
  kind: string;
  /** blake3 hex-prefix of the ingest origin (URL / server+tool / path / id). */
  origin_blake3: string;
  /** The joined trust label that reached the sink ("untrusted"). */
  trust_level: string;
  /** The destructive sink tool name that was blocked ("git_commit" / "edit"). */
  sink_tool: string;
  /** Whether the sink was blocked (true for an emitted taint_flow row). */
  blocked: boolean;
  /** The task / turn id the flow belongs to (dashboard correlation). */
  task_id: string;
}

/**
 * G-2 / Slice A: one run_artifact_emitted SSB row for the RraViewerPanel
 * (DrawerKey rra-viewer, command get_run_artifact). Emitted at the `elai run`
 * exit when a Reproducible-Run Artifact (.rra) bundle is sealed: the blake3 root
 * over the canonical bundle bytes, the honest per-lane guarantee level (never
 * "reproducible_tokens" for a cloud lane, GD-5), the F-gate evidence-manifest
 * entry count, and the pinned ELAI_SEED. Field names mirror the Rust
 * `RunArtifactRow` verbatim (serde).
 */
export interface RunArtifact {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts_ms: number;
  /** blake3 hex-prefix of the canonical .rra bundle bytes (the root). */
  root_blake3: string;
  /**
   * Honest per-lane guarantee label: "reproducible_action_stream" /
   * "verify_divergence" (never "reproducible_tokens" for a cloud lane).
   */
  guarantee_level: string;
  /** Number of F-gate verdict-hash entries in the evidence manifest. */
  gate_count: number;
  /** The pinned ELAI_SEED the action stream re-derives under. */
  seed: number;
  /** The session whose stream was bundled. */
  session_id: string;
  /**
   * I-45: 16-hex blake3 objective hash sealed at run time.
   * null / undefined on LEGACY receipts (pre-I-45 runs without IdeaFramingStarted).
   */
  objective_hash?: string | null;
  /**
   * J-14: effective tier verdict (snake_case TierVerdictLabel) joined from the
   * latest `TierEvaluated` ReproPackEvent row for this session_id.
   * null / undefined when no tier verdict row exists (LEGACY receipts or verify
   * path SSB emit still DARK).
   */
  tier_effective?: string | null;
  /**
   * J-14: requested tier (snake_case); null / undefined when no tier row exists.
   */
  tier_requested?: string | null;
  /**
   * J-14: recorded downgrades as already-formatted "from->to:reason" strings
   * from the latest `TierEvaluated` row. Empty when no tier row or no
   * downgrades were recorded.
   */
  downgrades?: string[];
  /**
   * J-14: first divergent position (0-based) from the latest
   * `FirstDivergenceLocated` row for this session_id. null / undefined when no
   * divergence row exists (T0 action-faithful or LEGACY).
   */
  divergence_position?: number | null;
  /**
   * J-14: expected value (hex) at the divergence position. null / undefined
   * when no divergence row exists.
   */
  divergence_expected?: string | null;
  /**
   * J-14: actual value (hex) at the divergence position. null / undefined
   * when no divergence row exists.
   */
  divergence_actual?: string | null;
}

/**
 * G-3 / Slice A: one blast_radius_blocked SSB row for the BlastRadiusDrawer
 * (DrawerKey blast-radius, command get_blast_radius_blocks). Emitted by the
 * deterministic runtime blast-radius gates when a fail-closed gate refuses an
 * operation: either a shell-EXPANDED filesystem-mutating target that resolved
 * OUTSIDE the per-run worktree root (kind = "fs_escape", from
 * translate_tool_request / run_bash) or an outbound egress host that carried a
 * NUL/control char or resolved outside the pinned allow-set (kind =
 * "egress_denied", from fetch_and_extract). The pre/post-expansion command pair
 * is the #10077 forensics fix. Field names mirror the Rust `BlastRadiusBlockRow`
 * verbatim (serde).
 */
export interface BlastRadiusBlock {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts_ms: number;
  /** Boundary-crossed label: "fs_escape" / "egress_denied". */
  kind: string;
  /** The original, un-expanded command/argv the agent issued. */
  pre_expansion_command: string;
  /**
   * The resolved/canonicalised target (FS) or rejected host+ip (egress) — what
   * would actually have executed/connected (the #10077 forensics fix).
   */
  post_expansion_command: string;
  /** The specific resolved FS target path, or the rejected egress host. */
  target_or_host: string;
  /**
   * blake3 hex-prefix of the canonical worktree root (identifier, not a path).
   * Empty for the egress arm.
   */
  worktree_root_blake3: string;
  /** G-1 provenance trust label join when present; "" for a pure-containment block. */
  trust_label: string;
}

/**
 * G-8 / Slice A: one provenance_attestation_emitted SSB row for the
 * AttestationDrawer (DrawerKey attestation, command get_attestations). Emitted at
 * the `elai run` exit when a signed in-toto v1 provenance attestation is built
 * from the sealed G-2 Reproducible-Run Artifact: the F-gate verified_levels
 * rollup re-expressed as a custom in-toto predicate + signed with a LOCAL
 * blake3-Merkle proof so it re-verifies OFFLINE without re-running ELAI
 * (trust-without-re-running, SLSA-VSA). Field names mirror the Rust
 * `AttestationRow` verbatim (serde).
 */
export interface Attestation {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts_ms: number;
  /** The run whose Reproducible-Run Artifact was attested. */
  session_id: string;
  /**
   * The in-toto subject digest: the blake3-Merkle root over the predicate (the
   * value the git trailer carries; `elai verify-attestation` recomputes it).
   */
  subject_blake3: string;
  /** The G-2 bundle root the attestation chains to (the run-artifact anchor). */
  run_artifact_root: string;
  /** Count of F-gate verdicts in the SLSA-VSA verified_levels rollup. */
  verified_levels: number;
  /**
   * true iff every present verified-level passed (every present manifest entry
   * IS a passed gate; a gate-off state leaves no entry).
   */
  all_passed: boolean;
  /** The honest signing-model label (default "local_blake3_merkle"). */
  signing_model: string;
}

/**
 * G-9 / Slice A: one secret_redacted SSB row for the RedactionPanel (DrawerKey
 * redaction, command get_redaction_stats). Emitted whenever the write-time
 * secret+PII redactor fires on a typed SsbPayload free-text field at the top of
 * Ssb::write_payload (BEFORE msgpack serialization; the append-only invariant
 * forces write-time). The row carries the redacted field identity + the stable
 * blake3-keyed token(s) + the match count + the per-kind (label, count) summary,
 * NEVER the raw secret. Field names mirror the Rust `RedactionStatRow` verbatim
 * (serde). `kind_counts` is a list of [label, count] tuples.
 */
export interface RedactionStat {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts_ms: number;
  /** The run whose write the redaction fired on. */
  session_id: string;
  /** The redacted SsbPayload field identity (e.g. "idea_framing_started.prompt"). */
  field: string;
  /** The stable blake3-keyed redaction token(s) that replaced the match(es). */
  tokens: string[];
  /** Total number of redactions that fired on the write. */
  match_count: number;
  /** Sorted [kind_label, count] tuples across the hits (no secret value). */
  kind_counts: [string, number][];
}

/**
 * G-7 / Slice A: one claim_confidence_scored SSB row for the ConfidenceDrawer
 * (DrawerKey confidence, command get_claim_confidence). Emitted per live
 * per-change conformal decision carrying the REAL fused claim_score (log-prob
 * LOCAL-only + self-consistency + exec-agreement components) that replaced the
 * task_runner step-count placeholder. Field names mirror the Rust
 * `ClaimConfidenceRow` verbatim (serde). `logprob_component` is null on the
 * cloud lane (Anthropic returns no token log-probs — GD-5).
 */
export interface ClaimConfidence {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts_ms: number;
  /** Synthetic per-claim id the decision was keyed on. */
  claim_id: string;
  /** The calibrated fused confidence in [0,1] fed to decide(). */
  claim_score: number;
  /** (a) Calibrated log-prob component; null on the cloud lane. */
  logprob_component: number | null;
  /** (b) Self-consistency agreement component in [0,1]. */
  self_consistency_component: number;
  /** (c) Oracle-free test execution-agreement component in [0,1]. */
  exec_agreement_component: number;
  /** The lane the score was computed on ("local" | "cloud"). */
  lane: string;
  /** The conformal tau threshold the decision used. */
  tau: number;
  /** The decide() verdict wire label ("abstain" | "answer"). */
  conformal_verdict: string;
  /** Whether the decision abstained. */
  abstained: boolean;
}

/**
 * G-7 / Slice A: one trifecta_quarantined SSB row for the ConfidenceDrawer
 * (DrawerKey confidence, command get_trifecta_quarantines). Emitted per tool-call
 * chokepoint where the secret-present + untrusted-span + egress legs co-occur and
 * the untrusted span was held back from the tool-caller. Field names mirror the
 * Rust `TrifectaQuarantineRow` verbatim (serde). NEVER the raw secret or span.
 */
export interface TrifectaQuarantine {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts_ms: number;
  /** Synthetic task id the quarantine fired on. */
  task_id: string;
  /** blake3 prefix of the untrusted origin (NEVER the raw span). */
  untrusted_origin_blake3: string;
  /** The egress / destructive sink tool that was blocked. */
  sink_tool: string;
  /** (a) A secret pattern matched in the live context (G-9 detector). */
  secret_present: boolean;
  /** (b) The pending span's provenance was Untrusted (G-1 taint). */
  untrusted_span: boolean;
  /** (c) The pending tool was an egress channel (capability gate subset). */
  egress_available: boolean;
  /** Whether the span was quarantined. */
  quarantined: boolean;
  /** The no-tool sub-LLM verdict label. */
  sub_llm_verdict: string;
}

/**
 * G-5 / Slice A: one review_surface_scored SSB row for the CockpitWindow
 * (WindowType 'cockpit', command get_review_surface). Emitted by the
 * `elai review --cockpit` Diff-Surface-to-Review (DSR) producer: the DSR ratio
 * (un-vouched changed lines / total changed lines, recomputed byte-identically
 * from the pinned git-diff + the seq-ASC SSB stream, GD-2), the vouched /
 * un-vouched / total changed-line counts, and the pinned diff-base commit. The
 * window renders the headline DSR + the vouched/un-vouched split. Field names
 * mirror the Rust `ReviewSurfaceRow` verbatim (serde).
 */
export interface ReviewSurface {
  /** SSB monotonic sequence number (newest first ordering key). */
  seq: number;
  /** SSB row timestamp (epoch millis). */
  ts_ms: number;
  /** The session whose working-tree changes were scored. */
  session_id: string;
  /** Diff-Surface-to-Review ratio in [0.0, 1.0]. */
  dsr: number;
  /** DSR denominator: total added + removed changed lines across all hunks. */
  total_changed_lines: number;
  /** DSR numerator: changed lines in un-vouched hunks (need full attention). */
  attention_lines: number;
  /** Count of vouched hunks (clean/passing deterministic signal). */
  vouched_hunks: number;
  /** Count of un-vouched hunks (need full attention). */
  unvouched_hunks: number;
  /** The deterministically resolved diff-base commit SHA the score used. */
  base_commit: string;
  /**
   * G-5 honesty fix: whether the working-tree diff was actually obtained.
   * `false` ONLY on a git error (absent / not a repo / non-zero exit /
   * unresolvable base); in that case the DSR/counts are NOT a review verdict and
   * the cockpit renders a "diff unavailable" banner, NOT a clean DSR=0. `true`
   * for both a real diff AND a genuinely empty diff (honest "no changes").
   */
  diff_available: boolean;
  /**
   * G-5 honesty fix: the git error reason when `diff_available === false`;
   * empty string otherwise.
   */
  diff_unavailable_reason: string;
}

// ------------------------------------------------------------------
// G-6 (SCHEMA_VERSION 217) — SafeDeltaPanel DTO.
// Mirrors SafetyDeltaLatest in commands_g6.rs (camelCase on the wire).
// ------------------------------------------------------------------

/** The most recent `safety_delta_measured` SSB row surfaced by the SafeDeltaPanel. */
export interface SafetyDeltaLatest {
  /** Suite label from the underlying SecCodePlt report. */
  suite: string;
  /** Candidate-generation provenance label. */
  model_id: string;
  /** Derived SAFEDELTA = insecure_rate_before - insecure_rate_after. */
  safety_delta: number;
  /** Seeded bootstrap-resample 95% CI lower bound (GD-2). */
  ci_low: number;
  /** Seeded bootstrap-resample 95% CI upper bound (GD-2). */
  ci_high: number;
  /** Number of candidate completions evaluated. */
  n_tasks: number;
  /** Deterministic RNG seed (ELAI_SEED, default 0). */
  seed: number;
  /** blake3 hex of the corpus blob (provenance; anti-gaming). */
  corpus_blake3: string;
  /** Derived gate recall: (before - after) / before. */
  gate_recall: number;
  /**
   * `true` = Tier-1 static-gate only; the live dynamic exploit-exec arm
   * (BaxBench-392 arXiv:2502.11844, SUSVIBES-200 arXiv:2512.03262) is
   * separately RunPod-gated. Never claim significance for Tier-1 (GD-5).
   */
  runpod_pending: boolean;
  /** SSB row timestamp (unix milliseconds). */
  ts: number;
}

// ------------------------------------------------------------------
// I-38 (SCHEMA_VERSION 235) — QdeltaPanel DTO.
// Mirrors QdeltaLatest in commands_i38.rs (snake_case on the wire).
// ------------------------------------------------------------------

/** The most recent `qdelta_measured` SSB row surfaced by the QdeltaPanel. */
export interface QdeltaLatest {
  /** Suite label (e.g. "qdelta_static_fixture_v1"). */
  suite: string;
  /** Candidate-generation provenance label. */
  model_id: string;
  /** Number of task pairs evaluated. */
  n_tasks: number;
  /** Number of bootstrap seeds used. */
  seed_count: number;
  /** Whether the evaluation was paired (always true after floor_check). */
  paired: boolean;
  /** Defective-diff rate with gates ON. */
  on_defective_rate: number;
  /** Defective-diff rate with gates OFF. */
  off_defective_rate: number;
  /** QDELTA = off_defective_rate - on_defective_rate (positive = gates help). */
  delta: number;
  /** Paired bootstrap 95% CI lower bound. */
  ci_low: number;
  /** Paired bootstrap 95% CI upper bound. */
  ci_high: number;
  /** Cost per task in USD (informational). */
  dollar_per_task: number;
  /** Contamination probe result string: "not_run" | "honestly_evaluated" | "game_with_memorization". */
  contamination_probe_result: string;
  /** Mutation survival rate per task (I-20 stub; 0.0 until I-20 ships). */
  surviving_mutants_per_task: number;
  /** blake3 hex of the corpus blob (provenance; anti-gaming). */
  corpus_blake3: string;
  /**
   * `true` = Tier-1 fixture only; live-model leg is RunPod-gated (D-026).
   * Never claim significance for Tier-1.
   */
  runpod_pending: boolean;
  /** SSB row timestamp (unix milliseconds). */
  ts: number;
}

// ------------------------------------------------------------------
// G-10 (SCHEMA_VERSION 218) -- ModelDriftPanel DTO.
// Mirrors ModelDriftStatus in commands_g10.rs (camelCase on the wire).
// ------------------------------------------------------------------

/** The most recent `model_drift_suspected` SSB row surfaced by the ModelDriftPanel. */
export interface ModelDriftStatus {
  /** blake3 identity key of the (model_pin, system_fingerprint, lane) tuple. */
  identity_key: string;
  /** Primary drift signal label: fingerprint_changed | canary_regressed | malformed_output_spike. */
  kind: string;
  /** All drift signal labels that fired (ordered: FingerprintChanged first if present). */
  signals: string[];
  /**
   * Per-family regression delta: [family_label, transfer_score] pairs.
   * Negative transfer_score means regression; threshold is DEFAULT_FAIL_BAND (0.05).
   */
  per_family_delta: [string, number][];
  /**
   * Domain labels of regressed families (for review routing surface, DD5).
   * Format: "domain:<family_label>" (e.g. "domain:code_fix").
   */
  affected_template_ids: string[];
  /**
   * `true` ONLY for LOCAL lane + changed system fingerprint (GD-5 honest ceiling).
   * NEVER `true` for CLOUD lane — cloud drift is behavioral-suspected only.
   * The panel renders a FINGERPRINT-CONFIRMED / BEHAVIORAL-SUSPECTED badge.
   */
  confirmed_by_fingerprint: boolean;
  /** Wall-clock ms at which the stored last-good baseline was captured. */
  baseline_captured_at: number;
  /** SSB row timestamp (unix milliseconds). */
  ts: number;
}

// ---------------------------------------------------------------------------
// H-13 canvas op types
// ---------------------------------------------------------------------------

/**
 * H-13: 10-arm CanvasOpKind discriminator mirror.
 * Wire labels are snake_case matching CanvasOpKind::as_str() on the Rust side.
 */
export type CanvasOpKind =
  | 'node_created'
  | 'edge_connected'
  | 'node_deleted'
  | 'edge_deleted'
  | 'annotation_added'
  | 'label_edited'
  | 'group_created'
  | 'node_forked'
  | 'screenshot_captured'
  | 'layout_changed';

/**
 * H-13: payload for a canvas_op_applied SSB row.
 * Mirrors Rust CanvasOpPayload (ssb.rs) + CanvasOpDto (commands_h13.rs).
 * Consumed by CanvasPanel.svelte to mutate the graphology graph live.
 */
export interface CanvasOpPayload {
  /** Op kind discriminator (10-arm). */
  kind: CanvasOpKind;
  /** Primary node id affected (empty string for edge/annotation ops). */
  node_id: string;
  /** Flexible per-op JSON payload (matches CanvasCreate/Connect/etc. input schemas). */
  op_data: Record<string, unknown>;
  /** Canvas mode at emission time (e.g. "arch", "code"). */
  mode: string;
  /** Canonical task id. */
  task_id: string;
  /** Session id for correlation. */
  session_id: string;
  /** SSB row timestamp (unix ms). */
  ts: number;
}

// ---------------------------------------------------------------------------
// I-15 — oracle cache source discriminator
// ---------------------------------------------------------------------------

/** I-15: source discriminator surfaced on OracleCacheHit / OracleCacheMiss rows.
 *  "blake3_exact"   = in-session exact match (zero ONNX embedding calls).
 *  "semantic_vector" = full vector pipeline ran (miss path).
 *  null             = legacy C-30 row without I-15 field.
 */
export type OracleCacheSource = "blake3_exact" | "semantic_vector" | null;

// ------------------------------------------------------------------
// I-33 (SCHEMA_VERSION 240) -- GovernanceMetricsChip DTO.
// Mirrors GovernanceMetricsPayload in commands_i33.rs (snake_case on wire).
// ------------------------------------------------------------------

/** Governance metrics aggregated from recent governance_primitive_fired SSB rows. */
export interface GovernanceMetrics {
  /** CDL: Cosmetic-Deadlock rate = CeflExternalized count / total. */
  cdl: number;
  /** DIU: Deferral Info Utilisation = I6qRejected count / total. */
  diu: number;
  /** FSR: Framing Success Rate = clean CommitRevealSealed / total CommitRevealSealed. */
  fsr: number;
  /** Total number of governance_primitive_fired rows sampled. */
  row_count: number;
}

// ------------------------------------------------------------------
// I-12 (SCHEMA_VERSION 251) -- PlaybookEvent DTO.
// Mirrors PlaybookEventPayload / PlaybookEventKind in rule_curator.rs.
// action_type = "playbook_event" (Pattern-2, single SQL variant, 4-arm Kind).
// ------------------------------------------------------------------

/** I-12: 4-arm kind discriminator for PlaybookEvent SSB rows. */
export type PlaybookEventKind = "Promoted" | "Demoted" | "ValidationRun" | "Exported";

/** I-12: payload of a playbook_event SSB row. */
export interface PlaybookEventPayload {
  rule_id: string;
  kind: PlaybookEventKind;
  /** SSB seq IDs of the evidence rows that grounded this event. */
  outcome_row_ids: string[];
  /** TPR - FPR lift estimate; null when not yet computed (ValidationRun / Demoted). */
  lift: number | null;
  /** Number of regressions detected during this validation run. */
  regression_count: number;
  /** Number of SSB evidence rows counted for this rule_id. */
  evidence_count: number;
  /** Unix milliseconds of the event. */
  ts_ms: number;
}

// ------------------------------------------------------------------
// I-36 (SCHEMA_VERSION 268) -- CompiledVariant badge chip DTO.
// Mirrors CompiledVariantRow in commands_i36.rs (snake_case on wire).
// action_type = "prompt_compiled" (Pattern-2, 3-arm Kind).
// ------------------------------------------------------------------

/** I-36: one row of the compiled-prompt badge in SettingsWindow. */
export interface CompiledVariantRow {
  /** Event kind: "prompt_compiled" | "transfer_gap_measured" | "prompt_compiled_rolledback". */
  kind: string;
  /** Snake-case role name (e.g. "executor", "researcher"). */
  role: string;
  /** Backend name (e.g. "anthropic_api", "sidecar"). */
  backend: string;
  /** First 16 hex chars of the blake3 variant hash. */
  variant_hash: string;
  /** compiled_pass_rate - source_pass_rate delta (0.0 for Compiled rows). */
  fixture_delta: number;
  /** SSB row timestamp (unix milliseconds). */
  ts: number;
}
