//! Shared State Board (SSB): inter-agent communication via SQLite WAL.
//!
//! Single-writer guarantee via `tokio::mpsc` channel. All writes serialized
//! through one dedicated task. Reads served from a `deadpool-sqlite` async connection pool.
//! Append-only within a session — no UPDATE or DELETE.
//!
//! A single [`Ssb`] instance is permanently bound to one `(project_id, session_id)` pair.
//! Multi-session scenarios require separate instances.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use rusqlite::types::Value;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tokio::sync::{broadcast, mpsc, oneshot};

use elai_core::clock::ClockProvider;
use elai_reasoning::chat_reflect::{ReflectEvent, ReflectEventSink};

// ---------------------------------------------------------------------------
// Schema version — bump on any DDL change
// ---------------------------------------------------------------------------

// B-10: bumped 21 -> 22 for SsbActionType::ContextBudgetComputed +
// SsbFilteredByOverlap. Tier-4 last bumped 20 -> 21 for ExecutionModeChanged.
// B-19A: bumped 22 -> 23 for SsbActionType::HookLatencyViolation
// (per-hook-tier T1/T2/T3 latency budget breach beacon).
// batch-ful-d: bumped 23 -> 24 for research_path_chosen + grounding_cache_event CHECK variants.
// Track B (s-1430428): bumped 24 -> 25 for mcp_server_connected / mcp_tool_called /
// mcp_server_disconnected CHECK variants. Enum + from_str + payloads already landed in
// batch-ful-d; this bump completes the CHECK constraint to match the enum surface.
// B-6 (coordinated schema bump): bumped 25 -> 26 for B-4 circuit_breaker_state_change,
// B-7 bubble_up_typed / integration_checkpoint_success / dag_repair_round /
// concept_decomposition_node, and B-6's own evidence_gate_evaluated / supervision_event /
// bsp_superstep / agent_lifecycle / anti_spiral_hit. All CHECK, enum, as_sql_str,
// from_str, payloads, and SsbPayload variants land atomically in this bump.
// DESTRUCTIVE: existing v25 session DBs are DROPped on first read (no migration logic).
// B-23 Phase 4H Group A: bumped 27 -> 28 for 6 new self-mod SSB action types
// (data_table_grew, l3_proposal, convergence_stagnant, config_oscillation,
//  regression_detected, evolution_withheld). The 5 batch-3C self-mod variants
// (proposal_*, msi_update, config_archived) already shipped in v26/v27.
// B-39 chat reflect pipeline: 4 new SsbActionType variants for per-turn REFLECT events.
// B-24 Phase 4I Group C1: bumped 29 -> 30 for 6 new media-validator SSB action types
// (media_validation_started, media_tier1_complete, media_tier2_complete,
//  render_complete, visual_feedback_iteration, media_human_review_required).
// B-28 E-3 (SCHEMA_VERSION 31): bumped 30 -> 31 for SsbActionType::GenerationModeSelected.
// Single new action_type added in this bump; the other 3 focus-tunnel beacons
// (streaming_early_terminated, pipeline_pre_assembled, dependency_resolved) land
// with their own bumps in E-4..E-7 when their emitters are wired. Co-staging the
// CHECK list without emitters would create dead schema rows the writer never
// produces — declined per B-24 4I.1 precedent.
// B-28 E-4 (SCHEMA_VERSION 32): bumped 31 -> 32 for SsbActionType::StreamingEarlyTerminated.
// Single new action_type. Emitter is the post-E-7 WIRING phase that plugs the
// library-tier StreamingValidator (focus_tunnel::streaming) into run_fim_step /
// run_diff_step; E-4 only defines the library surface + the schema bump. The
// payload variant lands now (not deferred) because the StreamingValidator's
// EarlyTermWireForm projection consumes StreamingEarlyTerminatedPayload directly
// — co-staging is appropriate here even though the live emitter is deferred.
// DESTRUCTIVE: existing v31 session DBs are DROPped on first read (no migration
// logic). Matches B-23 / B-24 / B-27 / B-28 E-3 precedent.
// B-28 WIRING-1 (SCHEMA_VERSION 33 -> 34 -> 35): batched 3-variant bump for the
// remaining focus_tunnel reader-side beacons. Three serial bumps in one atomic
// commit:
//   * 32 -> 33: SsbActionType::PipelinePreAssembled (item A.1). Emitter is the
//     post-WIRING-2 hook in drive_task that calls
//     PipelinePreAssembler::on_step_outcome + spawn prepare(). Payload struct
//     PipelinePreAssembledPayload defined below. Reader aggregator
//     pre_assembly_stats_for_payloads (pre_assembly.rs:498) wires the match arm
//     in this commit.
//   * 33 -> 34: SsbActionType::DependencyResolved (item A.1). Emitter is the
//     post-execute hook scanning changed_files via plugins::import_extractor +
//     DependencyResolver::resolve. Payload struct DependencyResolvedPayload
//     defined below. Reader aggregator dependency_resolution_stats_for_payloads
//     (dep_resolver.rs:807) wires the match arm in this commit.
//   * 34 -> 35: SsbActionType::MicroStepCreated (item A.1). Emitter is the
//     SpeculativeFileRouter::route NeedsMicroStep / MicroStepCapExhausted path.
//     Payload struct MicroStepCreatedPayload defined below. Reader aggregator
//     file_router_stats_for_payloads (file_router.rs:881) wires the match arm
//     in this commit.
// DESTRUCTIVE: existing v32 session DBs are DROPped on first read (no migration
// logic). Matches B-23 / B-24 / B-27 / B-28 E-3 / E-4 precedent.
// B-30 (SCHEMA_VERSION 36): LanguageGrammarStatus action type for one-shot
// LanguageRegistry startup publication. Emitter: dkg index init after
// LanguageRegistry::new() + per-grammar register() cycle. Consumer:
// dashboard KnowledgeDrawer LANGUAGES LOADED card.
// B-29 Phase 5 (SCHEMA_VERSION 37): SymbolQuery action type — one row per
// symbol_impact / symbol_flow / symbol_context / symbol_list tool call.
// Carries SymbolQueryPayload { tool, target, hop_count, total_files, latency_ms }.
// B-32 (SCHEMA_VERSION 38): MinifiedFileChunked variant for the adaptive
// character-based chunker. Emitted by IndexingService once per minified file
// after the BM25 batch loop. Carries MinifiedFileChunkedPayload { relative_path,
// char_count, chunk_count, avg_line_length }. Defense-in-depth alongside the
// `tracing::info!(target="knowledge::chunking", ..., "minified chunked")` emit
// at bm25.rs:412/523 (kept so dev console retains visibility).
// B-33 follow-up (SCHEMA_VERSION 40): SymbolGraphUpdated variant emitted
// by IndexingService after build_symbol_graph() success. Carries
// SymbolGraphUpdatedPayload { project_id, file_count, symbol_count,
// edge_count, built_ms } for the dashboard GraphWindow refresh path.
// Replaces the 30s setInterval fallback that B-33 shipped while ssb.rs
// was locked.
// B-31.5 (SCHEMA_VERSION 39): bumped 38 -> 39 for the 5 deferred B-31 SSB
// variants — IndexCheckpoint / IndexResume / IndexCancel / LockAcquired /
// LockContested. These replace the `tracing::info!(target="ssb_pending", ...)`
// placeholders left by Cycle B in startup.rs (and the new per-batch /
// per-lock writer call-sites added to indexing_service.rs + lock.rs in
// Cycle C). 14-site lockstep documented at
// `workflow/reasoning/B-31.5-decisions.md` DECISION-B31.5-7. DESTRUCTIVE:
// existing v38 session DBs are DROPped on first read (no migration logic).
// B-33 follow-up (SCHEMA_VERSION 40): bumped 39 -> 40 for the previously
// DEFERRED SymbolGraphUpdated variant. This replaces the 30s setInterval
// fallback in dashboard-v2 GraphWindow.svelte (B-33 D3) with a real
// per-build event. Producer: IndexingService::build_index_resumable
// after build_symbol_graph() returns Ok and reports >0 files. Consumer:
// dashboard-v2 GraphWindow event listener + KnowledgeDrawer refresh.
// 7-site lockstep (enum / CHECK / as_sql_str / from_str / SsbPayload
// variant / action_type mapper / test_payload reconstructor) documented
// at workflow/reasoning/B-33-decisions.md DEFERRED items 1+2.
// B-40 (SCHEMA_VERSION 41): bumped 40 -> 41 for 5 new SSB action types
// powering the FRAMING substate (Idea-Framer + Critic Loop + Domain
// Research Presets). Variants: IdeaFramingStarted /
// ProjectClassClassified / CriticRoundComplete / IdeaFramed /
// TradeoffCardEmitted. Producers: orchestrator::idea_framing module.
// Consumers: dashboard-v2 IdeaFramingWindow + MonitorWindow critic-rounds
// badge + task_runner::derive_framing_evidence (7th AutoAdvanceEvaluator
// check, gated on FRAMING substate). DESTRUCTIVE migration per
// systemPatterns 2026-05-10 — pragma user_version mismatch drops + recreates
// the ssb table. 5x4=20 lockstep sites: enum L314, SQL CHECK L140-285,
// as_sql_str L742, ssb_action_type_from_str L889, SsbPayload variant L3068
// + action_type() match arm L3231 + 5 payload structs at L2247+. Documented
// at workflow/reasoning/B-40-decisions.md DECISION-SSB-BIJECTION.
// B-34 (SCHEMA_VERSION 42): bumped 41 -> 42 for 4 new SSB action types for
// the entry-point detection + artifact pipeline:
//   entry_points_detected  -- emitted by the entry_points tool dispatcher arm
//                              after detect_entry_points() returns.
//   artifacts_indexed      -- emitted by Phase C artifact indexer wiring.
//   artifact_searched      -- emitted by search_artifacts dispatcher arm.
//   artifact_stale         -- emitted when ArtifactIndexer detects changed files.
// Producers: production_agent_rpc.rs entry_points arm (entry_points_detected);
//            indexing_service.rs (artifacts_indexed, artifact_stale);
//            search_artifacts arm (artifact_searched).
// 4x5=20 lockstep sites: enum / CHECK / as_sql_str / from_str / SsbPayload
// (payload structs land with the indexing_service wiring in Phase C2).
// B-38 (SCHEMA_VERSION 43): bumped 42 -> 43 for 3 new SSB action types tied
// to the ambiguity-score continuous metric gate:
//   - ambiguity_scored          (one row per AmbiguityScorer.score() result)
//   - ambiguity_floor_failed    (one row per gate verdict where a floor was
//                                breached even with overall <= 0.2)
//   - ambiguity_streak_advanced (one row per qualifying-but-streak-incomplete
//                                gate verdict; carries current/required streak)
// Additive only — no existing variants renumbered or removed.
// B-42 (SCHEMA_VERSION 44): bumped 43 -> 44 for 4 new SSB action types
// tied to the cosmos tier knowledge worker:
//   - cosmos_refresh_started   (one row per CosmosWorker::tick refresh start)
//   - cosmos_artifact_updated  (one row per successful artifact refresh)
//   - cosmos_artifact_decayed  (one row when age > 3 * cadence_interval)
//   - cosmos_grounding_used    (one row per CosmosCheck verifier dispatch)
// Producers: knowledge::cosmos_worker (refresh / decay), reasoning::verification
// (grounding_used). Additive only — no existing variants renumbered.
//
// B-41 (SCHEMA_VERSION 45): bumped 44 -> 45 for 3 new SSB action types
// tied to the per-level auditor mode dispatch (Concepts/Phases/Steps/
// BabySteps/BabyStepDetail/Plan):
//   - per_level_audit_started (one row per Auditor* spawn at a tier node)
//   - per_level_audit_verdict (one row per LevelAuditVerdict produced)
//   - per_level_audit_cascade (one row when a 2nd BLOCK forces parent cascade)
// Producers: orchestrator::pipeline_tree_navigator (per-level audit hooks).
// Additive only — no existing variants renumbered.
//
// B-37C (SCHEMA_VERSION 46): bumped 45 -> 46 for 4 new SSB action types
// tied to the Phase 4M multi-runtime backend abstraction:
//   - runtime_fallback         (per-step in RuntimeOrchestrator chain walk)
//   - runtime_health_changed   (per backend health transition observed)
//   - cli_subprocess_spawned   (per spawn() in CLI adapters)
//   - cli_subprocess_killed    (per kill_and_wait() in CLI adapters)
// Producers: orchestrator::runtime_orchestrator (fallback / health),
//            providers::runtime::cli_common (subprocess spawn/kill).
// The pinned-schema-version contract tests from B-34 (idea_framing_contract /
// minified_chunked_ssb_contract) intentionally stay at 42 floor — they
// guard against regressions, not floor advancement. Additive only.
// C-1 (SCHEMA_VERSION 47): atomic bump for 7 new knowledge + reasoning
// producer variants (GrcStepEmitted, GroundingSweepResult, CsqEvaluated,
// AuditorRubberStampRejected, HydeQuery, VectorCacheHitRate, QueryTrace).
// Producers: task_runner (GRC step + grounding sweep + CSQ),
// reasoning::engine (auditor anti-rubber-stamp, surfaced at the
// PipelineGrc caller), knowledge::hyde (per-hypothesize call),
// knowledge::vector::cache (flush_hit_rate_ssb), knowledge::pipeline
// (per-query trace). See workflow/reasoning/C-1-decisions.md.
// C-2 (SCHEMA_VERSION 48): atomic bump for 10 new orchestration + cosmos +
// stagnation producer variants — 4 tree-node beacons (PhaseNodeReady /
// StepNodeReady / BabyStepReady / DagRepairEscalated) + AgentStatusChanged
// (mirror of AgentLifecycle, single `state` field per DRIFT-5) + 5
// stagnation / anti-spiral variants (StagnationSpinningDetected /
// StagnationOscillationDetected / LateralThinkingActivated /
// GoalDriftWarning / WatchdogTimeout). Producers: task_runner baby-step
// loop (PhaseNodeReady/StepNodeReady/BabyStepReady), pipeline_builder
// RepairOutcome::Escalated path (DagRepairEscalated), bsp/engine co-emit
// (AgentStatusChanged), reserved for C-10 stagnation detectors (5 anti-
// spiral variants — defined now so the SCHEMA bump is atomic).
// See workflow/reasoning/C-2-decisions.md.
// C-7 (SCHEMA_VERSION 49): atomic bump for 1 new dashboard review-surface
// variant — ChangeDecision. Emitted by the dashboard-v2 Tauri commands
// accept_change / decline_change / retry_audit each time a human reviewer
// acts on a per-file or per-hunk change row in ReviewWindow. Carries
// `decision` ("accepted" | "declined" | "retried"), `path`, optional
// `hunk_id` (None == file-level decision), and optional `note` (reviewer
// retry note). Live FileWrite rows remain the source of truth for the
// diff payload itself; ChangeDecision rows record reviewer verdicts.
// See workflow/reasoning/C-7-decisions.md.
// C-8 (SCHEMA_VERSION 50): atomic bump for 1 new viewer-stub-eradication
// variant — SearchModeChanged. Emitted by the live_cli startup-readiness
// path AND by the dashboard-v2 Tauri command `set_search_mode` whenever
// the DegradationCascade verdict changes (or a reviewer manually flips
// the mode for diagnostic purposes). Carries the new `mode` string
// ("hybrid" | "lexical" | "semantic" | "ann") plus an optional `history`
// vector capturing the trailing mode sequence so the dashboard can
// render a "last N transitions" timeline without re-querying. Bridges
// the live_cli AppState (CLI process) and dashboard-v2 AppState
// (src-tauri process) which share no in-memory state — SSB is the
// only cross-process channel (per context-package-C8.md DRIFT-I).
// NOTE: ArtifactsIndexed variant + ArtifactsIndexedPayload already exist
// from B-34 (SCHEMA 42); C-8 wires them into production_agent_rpc.rs
// without re-defining. See workflow/reasoning/C-8-decisions.md DRIFT-G.
// C-5 (SCHEMA_VERSION 51): atomic bump for 1 new cloud-mode-hardening
// variant — RedactionApplied. Emitted by `RedactingBabyStepExecutor`
// each time the privacy redactor scrubs PII or secrets from a cloud-bound
// baby-step payload. Carries `count` (number of redaction substitutions
// applied) + `fields` (which logical fields were touched: e.g.
// `["task_summary", "escalation_message"]`). Consumer: CloudPanel.svelte
// privacy-event feed (last 5 redactions surfaced via SsbViewer query).
// Companion writes: CloudSpend (existing) and the new spend-cap-enforcement
// emission paths in CloudGroundingClient + CloudModeRouter. See
// workflow/reasoning/C-5-decisions.md.
// C-13 (SCHEMA_VERSION 53): atomic bump for 1 new IPC-dispatcher
// production-swap beacon — WipSquashComplete. Emitted by the post-AUDITING
// -> MEMORIZING transition hook (`squash_wip_commits_if_any` in
// `wip_squash.rs`) AND by the `elai wip-squash` CLI subcommand whenever a
// real squash actually executed (`squashed == true`, `wip_count > 5`).
// Carries `wip_count` (the leading-WIP-commit count observed before
// squashing) + `squashed` (true ⇒ a `git reset --soft HEAD~N` + commit
// happened; false ⇒ no-op because count <= 5). Producer wiring: lands at
// the task_runner audit-loop boundary AND as a direct CLI invocation for
// the Python hook side-channel. Consumer: dashboard activity feed surfaces
// "wip-squash: 7 commits collapsed" rows. See workflow/reasoning/C-13-decisions.md.
//
// C-22 (SCHEMA_VERSION 54): atomic bump for 2 new feasibility-probe completeness
// beacons:
//   * alternative_explored — emitted once per NovelStep alternative accepted
//     by the feasibility probe. Carries step_id + alternative_index so the
//     dashboard can render per-step alternative trails. Producer wiring:
//     `crate::feasibility_probe::DefaultFeasibilityProber::probe` per
//     NovelStep iteration where an alternative was chosen.
//   * feasibility_escalated — emitted when `decide_with_mode` returns
//     `FeasibilityDecision::Escalate`. Carries the FSM-facing escalation
//     summary (reason + escalation variant tag). Distinct from the
//     pre-existing `escalation_fired` row which targets the autonomous
//     gate; this row is the feasibility-probe-specific signal.
// Both action_types are reflected in the SQL CHECK constraint, as_sql_str,
// from_str, SsbPayload variant, action_type() arm, test_payload arm, and
// all_test_payloads sample list — the atomic 9-tuple per CLAUDE.md SCHEMA
// hard rule.
//
// C-23 (SCHEMA_VERSION 55): atomic bump for 3 new SessionDag branch-UI
// beacons. Replaces the previous `Checkpoint`-row-with-discriminator
// pattern used in `merge_branch_with_ssb` so the dashboard ActivityWindow
// and SessionsDrawer surfaces can render branch operations without
// inspecting the grc_history_hash string. The pre-existing
// `branch_merge` action_type (singular form, no payload) remains as a
// no-op deprecation stub for legacy rows; new producers MUST emit the
// new typed variants.
//   * branch_forked   — one row per successful SessionDag::create_branch.
//                       Producer: dashboard `fork_session_branch` Tauri
//                       command + CLI `/session fork` (handoff.rs).
//                       Carries branch_name + tip_snap_id + parent_snap_id.
//   * branch_merged   — one row per successful merge_branch_with_ssb
//                       (replaces the legacy Checkpoint emit on Winner).
//                       Producer: SessionDag::merge_branch_with_ssb.
//                       Carries winner_name + trunk_snap_id + abandoned_names.
//   * branch_closed   — one row per successful close_branch_with_ssb.
//                       Producer: SessionDag::close_branch_with_ssb (new
//                       wrapper mirroring merge_branch_with_ssb). Carries
//                       branch_name + reason.
// All three action_types are reflected in the SQL CHECK constraint,
// as_sql_str, from_str, SsbPayload variant, action_type() arm,
// test_payload arm, and all_test_payloads sample list — the atomic
// 9-tuple per CLAUDE.md SCHEMA hard rule.
// C-10 (SCHEMA_VERSION 52): atomic bump for 3 new anti-spiral lateral
// thinking + goal-drift block beacons:
//   * lateral_thinking_succeeded — emitted when a persona switch broke
//     a stagnation loop (downstream pattern detector no longer fires
//     after the persona suggestion was consumed). Producer wiring
//     deferred to C-N; the variant is added now so the SCHEMA bump is
//     atomic with the other C-10 detection emit sites.
//   * lateral_thinking_exhausted — emitted when all 5 personas have
//     been tried within the current task window without a pattern
//     break. Carries the persona-trial count so the dashboard can
//     surface "all personas exhausted" status.
//   * goal_drift_block — emitted when the GoalDriftCanary verdict
//     crosses the Block threshold (drift_score < 0.4 jaccard
//     similarity in the plan-direction reading; see
//     workflow/reasoning/C-10-decisions.md DRIFT-3). The existing
//     `goal_drift_warning` row covers the YELLOW band; this new row
//     covers the RED band and may be wired by the gate to fail the
//     task before audit. Producer wiring lands when GoalDriftCanary
//     emits the corresponding decision branch in task_runner.
// C-25 (SCHEMA_VERSION 56): atomic bump for 5 new compiled-pipeline +
// template-crystallizer + background-task beacons. Definition-only here;
// producer wiring lands in `compiled_pipeline.rs`, `background_task.rs`,
// `pipeline_builder.rs`, and the `task_runner.rs` post-execute hook.
//   * plan_skeleton                — one row per pipeline_builder factory
//                                    entry. Carries plan_id, step_count,
//                                    concept_class, plan_skeleton_hash,
//                                    and source (Template | Llm). C-25
//                                    ships the FIRST emitter (the spec's
//                                    "plan_skeleton SSB row" was missing
//                                    before this bump — see C-25 DRIFT-D).
//   * compiled_pipeline_fired      — one row per CompiledExecutor cache HIT
//                                    followed by successful run. Carries
//                                    pipeline_id, concept_class, use_count.
//   * template_crystallized        — one row per TemplateCrystallizer
//                                    crystallization. Carries concept_class,
//                                    plan_skeleton_hash, pipeline_id,
//                                    threshold_n.
//   * background_task_started      — one row per BackgroundTaskManager
//                                    spawn_task / gated_spawn success.
//                                    Carries task_id, command_summary,
//                                    soft_timeout_secs, hard_timeout_secs.
//   * background_task_complete     — one row per harvested handle
//                                    (poll_completed or auto-detach).
//                                    Carries task_id, exit_status,
//                                    duration_secs, auto_detached,
//                                    timed_out.
// DESTRUCTIVE: existing v55 session DBs are DROPped on first open with v56;
// rebuild from scratch (no migration is provided because SSB rows are
// observational, capped at PAYLOAD_MAX_BYTES, and have no downstream
// consumers that hold long-term state).
//
// C-25 audit follow-up (SCHEMA_VERSION 57): atomic bump for the 6th
// compiled-pipeline beacon, `decompile_triggered`. WIRING_CONTRACT line 161
// of the C-25 plan listed `decompile_triggered` as a produced SSB
// action_type, but the v56 implementation routed the event through the
// pre-existing `AntiSpiralHit` payload (WARN-1 in audit-C25.md). This
// bump adds the missing variant + payload so dashboard / contract queries
// filtered on `decompile_triggered` see every decompile event. Producer
// wiring lives at `compiled_pipeline.rs::CompiledExecutor::emit_decompile_triggered`.
//   * decompile_triggered          — one row per CompiledExecutor decompile
//                                    decision (consecutive-failure threshold
//                                    or confidence floor). Carries
//                                    concept_class, plan_skeleton_hash,
//                                    failure_count, trigger_reason.
// DESTRUCTIVE: existing v56 session DBs are DROPped on first open with v57.
//
// C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
// Two new beacons added atomically (9-tuple lockstep — see
// workflow/reasoning/C-26-decisions.md):
//   * futility_detected            — same-error-class window in
//                                    NavigatorTree saturates at
//                                    futility_same_error_threshold (default
//                                    3). Carries error_class,
//                                    consecutive_count, level, escalated.
//   * futility_output_stagnation   — GoalDriftCanary similarity between
//                                    consecutive baby-step outputs exceeds
//                                    0.90. Carries similarity, threshold,
//                                    level, new_persona (Hacker for NoDrift
//                                    per C-10 D4 affinity).
// all_test_payloads().len() goes 118 -> 120 (+2 variants).
//
// C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) DeterminismConfig + T-0
// WiringContract foundation. Three new beacons emitted atomically — the
// determinism tier produces one, the wiring tier produces two. Producer
// wiring lands in `determinism.rs` (DriftComparator) + `wiring_contract.rs`
// (StubBanAuditor / DkgReachabilityAuditor / ObservableEffectRunner) +
// `cli/wiring_audit.rs` (audit driver). See workflow/reasoning/C-27-decisions.md.
//   * wiring_evidence        — one row per produces_symbol verified reachable
//                              by DkgReachabilityAuditor under a configured
//                              [wiring.entrypoints]. Carries entrypoint,
//                              produces_symbols_verified, reachable_count,
//                              audit_id.
//   * wiring_violation       — one row per StubBanAuditor /
//                              DkgReachabilityAuditor / ObservableEffectRunner
//                              violation. Carries violation_kind (StubPattern
//                              | UnreachableSymbol | BrokenObservable),
//                              file, line, pattern, audit_id.
//   * determinism_drift      — one row per DriftComparator divergence. Carries
//                              seed, run_a_hash, run_b_hash,
//                              first_divergence_index, audit_id. Identical
//                              sequences emit ZERO rows (stub_ban contract).
// all_test_payloads().len() goes 120 -> 123 (+3 variants).
//
// C-28 (SCHEMA_VERSION 60): Testing-trust Tier T-1 mutation testing gate.
// One new beacon — mutation_survived — emitted by deterministic_auditor's
// Step 6 mutation check, one row per surviving mutant. Producer wiring
// lands in `mutation_testing.rs` (RustMutationRunner / PythonMutationRunner
// + MutationResult + EquivalentMutantCache) + `deterministic_auditor.rs`
// (CheckKind::Mutation, run_all() Step 6 async bridge). See
// workflow/reasoning/C-28-decisions.md.
//   * mutation_survived       — one row per mutant whose outcome is Survived.
//                               Carries file, line, mutant_kind. Caught /
//                               Timeout / Skipped emit zero rows.
// all_test_payloads().len() goes 123 -> 124 (+1 variant).
//
// C-29 (SCHEMA_VERSION 61): Testing-trust Tier T-2 (IntentPair) + T-3
// (basic Metamorphic relations). Two new beacons emitted atomically with
// the 9-tuple lockstep — IntentConfirmed (per BabyStep T-2 confirmation,
// `pair_count >= IntentConfig.min_pairs`) + MrViolated (one row per failed
// metamorphic relation emitted from `deterministic_auditor` Step 7,
// post-Mutation). The MrViolated payload `counterexample` MUST be
// non-empty (debug_assert + runtime sentinel fallback, see
// `metamorphic_testing::MrResult::failed` + DRIFT-C29-11). Producer
// wiring lands in `intent_pair.rs` / `metamorphic_testing.rs` +
// `evidence.rs::IntentPairsGate` + `autonomous_gate.rs` auto-confirm
// branch + `deterministic_auditor.rs::check_metamorphic`. See
// workflow/reasoning/C-29-decisions.md.
//   * intent_confirmed        — one row per BabyStep that satisfies the
//                               PLANNING -> TESTING_FIRST IntentPair gate.
//                               Carries step_id, pair_count, confirmed_by
//                               ("user" | "autonomous_gate").
//   * mr_violated             — one row per `MrResult { passed: false }`.
//                               Carries step_id, relation
//                               ("identity"|"inverse"|"permutation_invariance"),
//                               counterexample (non-empty, stub_ban),
//                               sample_size.
// all_test_payloads().len() goes 124 -> 126 (+2 variants).
//
// C-30 (SCHEMA_VERSION 62): atomic bump for 5 new Surgical Mode + Oracle Mode
// pipeline beacons. Producer wiring lands in
// `surgical_decomposer.rs` (decompose + LOC gate emit), `oracle_cache.rs`
// (get/insert emit), and `cli/src/live_cli.rs` (project_detection_signal
// scan before first /task). See workflow/reasoning/C-30-decisions.md.
//   * surgical_plan_created      — one row per SurgicalDecomposer::decompose
//                                  Ok return. Carries complexity_class
//                                  (Simple|Medium|Complex), blast_radius_files,
//                                  total_loc_estimate, step_count.
//   * surgical_loc_warning       — one row per decompose call whose
//                                  AffectedSet.total_loc_estimate exceeds 500.
//                                  Carries total_loc_estimate + threshold +
//                                  mode (Interactive|Autonomous).
//   * oracle_cache_hit           — one row per OracleAnswerCache::get hit.
//                                  Carries query_hash_hex, source_tier
//                                  (T1Authoritative|T2Strong|T3Weak),
//                                  cached_age_secs.
//   * oracle_cache_miss          — one row per OracleAnswerCache::insert
//                                  (preceded by a get-returns-None). Carries
//                                  query_hash_hex + confidence.
//   * project_detection_signal   — one row per signal evaluated by the
//                                  fresh-project detector
//                                  (Cargo.toml|go.mod|pyproject.toml|
//                                  package.json|rust_file_count_ge_10).
//                                  Carries signal_name, weight
//                                  (Required|Strong|Weak), fired (bool).
// All five variants are reflected in the SQL CHECK constraint, as_sql_str,
// from_str, SsbPayload variant, action_type() arm, test_payload arm, and
// all_test_payloads sample list — the atomic 9-tuple per CLAUDE.md SCHEMA
// hard rule. all_test_payloads().len() goes 126 -> 131 (+5 variants).
//
// C-32 (SCHEMA_VERSION 63): atomic bump for 5 new Code-Completeness Layer 2
// beacons. Producer wiring lands in
// `skeleton_generator.rs` (generate + flush emit), `mini_critic.rs`
// (per-blank verdict emit), `structural_retry.rs` (per-stage attempt
// emit), and `bridge_template.rs` (deterministic bridge-file emit). See
// workflow/reasoning/C-32-decisions.md.
//   * skeleton_generated              — one row per
//                                        `SkeletonGenerator::generate_for_functions`
//                                        Ok return. Carries blank_count,
//                                        language, file_path.
//   * skeleton_flushed                — one row per
//                                        `SkeletonGenerator::flush_to_disk`
//                                        success. Carries path,
//                                        completeness_ratio.
//   * mini_critic_verdict             — one row per `MiniCritic::check`.
//                                        Carries blank_id, verdict
//                                        ("pass"|"warn"|"block"), reason,
//                                        language.
//   * structural_retry_stage_attempt  — one row per
//                                        `StructuralRetrySequencer` stage
//                                        invocation. Carries blank_id,
//                                        stage, attempt, outcome.
//   * bridge_file_generated           — one row per
//                                        `BridgeFileTemplate::write_bridge`
//                                        success. Carries kind, path,
//                                        module_count.
// All five variants are reflected in the SQL CHECK constraint, as_sql_str,
// from_str, SsbPayload variant, action_type() arm, test_payload arm, and
// all_test_payloads sample list — the atomic 9-tuple per CLAUDE.md SCHEMA
// hard rule. all_test_payloads().len() goes 131 -> 136 (+5 variants).
// DESTRUCTIVE: existing v62 session DBs are DROPped on first open with v63
// (no migration is provided because SSB rows are observational, capped at
// PAYLOAD_MAX_BYTES, and have no downstream consumers that hold long-term
// state).
// D-1 (SCHEMA_VERSION 64): bumped 63 -> 64 for 4 new SsbActionType variants
// emitting ReasoningTemplate ArchiveChain lifecycle beacons.
//   * archive_entry_added            — one row per ArchiveChain::append.
//   * archive_entry_retired          — one row per template retirement.
//   * template_promoted              — one row per BAI promotion (D-5 producer
//                                      lands in a later session; the wire
//                                      surface is added now so the schema bump
//                                      is atomic).
//   * template_candidate_rejected    — one row per Candidate rejection.
// Schema bump is destructive (existing v63 DBs are dropped on first open)
// per the existing SSB convention documented in the v63 block above.
//
// E-5 (SCHEMA_VERSION 65): bumped 64 -> 65 for 2 new SsbActionType variants
// emitting bi-temporal `memory_facts` lifecycle beacons (E-plan plan E-5
// — user modeling + long-term memory). Variants:
//   * memory_fact_extracted   — one row per FactStore::insert success
//                               (Mem0-style ADD; UPDATE/DELETE/NOOP
//                               dispatcher variants reserved for the
//                               follow-up vertical slice).
//   * memory_fact_superseded  — one row per FactStore::supersede success
//                               (Zep bi-temporal close + replacement pair).
// All 9 atomic lockstep sites are touched: enum L1773-1782, SQL CHECK arm,
// as_sql_str arm, ssb_action_type_from_str arm, SsbPayload variant,
// action_type() arm, payload struct, test_payload arm, all_test_payloads
// arm. all_test_payloads().len() goes 140 -> 142 (+2 variants). Schema
// bump is destructive (existing v64 DBs are dropped on first open) per
// the existing SSB convention documented above.
//
// D-2 (SCHEMA_VERSION 66): bumped 65 -> 66 for 2 new SsbActionType variants
// emitting PPM (Process Preference Model) verifier-side beacons (D-plan
// plan D-2 — separable improving verifier). Variants:
//   * ppm_scored           — one row per PpmClient::score success that
//                            clears the configured threshold. Carries
//                            (candidate_id, verifier_id, mode,
//                             score_mean, score_var, num_steps,
//                             latency_ms, cache_hit).
//   * ppm_gating_failed    — one row per scoring that does NOT clear the
//                            threshold. Carries (candidate_id,
//                            verifier_id, mode, score_mean, threshold,
//                            num_steps).
// all_test_payloads().len() goes 142 -> 144 (+2 variants). Schema bump
// is destructive per the SSB convention.
//
// E-1 (SCHEMA_VERSION 67): bumped 66 -> 67 for 2 new SsbActionType variants
// emitting infinite-context chat illusion beacons (E-plan plan E-1 —
// HippoRAG PPR + CoreBlock + SLEEP_COMPUTE FSM substate).
//   * chat_reflection_completed — one row per
//                                 `chat_memory_reflect::reflect()` Ok return.
//                                 Carries (note_id, note_preview,
//                                 facts_extracted, decay_rows_examined,
//                                 decay_rows_updated, core_block_bytes).
//   * memory_evolution_run      — one row per SLEEP_COMPUTE substate entry
//                                 / exit. Carries (phase, duration_ms,
//                                 notes_touched).
// all_test_payloads().len() goes 144 -> 146 (+2 variants). Coordinated
// with D-2 (which took 66 first) per the brief's last-writer convention.
//
// E-1 dashboard slice (SCHEMA_VERSION 68): bumped 67 -> 68 for 4 more
// SsbActionType variants closing the WIRING_CONTRACT gap on the MemoryNotes
// + CoreBlock surfaces (producer wiring in task_runner.rs is off-limits
// this session — emitter sites listed in followups_for_other_sessions).
//   * memory_note_inserted     — one row per `NoteStore::insert` Ok return.
//                                Carries (note_id, scope, content_blake3,
//                                link_count).
//   * memory_note_evolved      — one row per A-MEM note evolution rewrite
//                                (PPR-walker-attached link/edit pass).
//                                Carries (note_id, old_content_blake3,
//                                new_content_blake3, links_added,
//                                links_removed).
//   * memory_conflict_detected — one row per duplicate / contradiction
//                                pair the conflict adjudicator emits.
//                                Carries (fact_id_a, fact_id_b,
//                                conflict_kind: "duplicate" |
//                                "contradiction").
//   * core_block_replaced      — one row per CoreBlockStore::upsert that
//                                replaces an existing row (vs first
//                                insert). Carries (user_id, old_blake3,
//                                new_blake3, bytes).
// all_test_payloads().len() goes 146 -> 150 (+4 variants).
//
// D-3 (SCHEMA_VERSION 69): bumped 68 -> 69 for 2 new SsbActionType variants
// covering the multi-verifier consensus aggregator (see
// .elai_cc/crates/orchestrator/src/verification_result.rs + the
// run_consensus_pass producer in agent_pool.rs). Both writer-side lockstep
// arms (enum / CHECK / as_sql_str / from_str / SsbPayload variant /
// action_type() / test_payload / all_test_payloads / variant count) updated
// atomically per CLAUDE.md SCHEMA hard rule.
//   * verifier_consensus_pass    — one row per `run_consensus_pass` Ok
//                                  return with verdict = Pass. Carries
//                                  (candidate_id, template_id, verdict,
//                                  verifier_count, pass_count, rule_id).
//   * verifier_consensus_fail    — one row per `run_consensus_pass` Ok
//                                  return with verdict != Pass. Carries
//                                  (candidate_id, template_id,
//                                  failing_verifier_ids, reason).
// all_test_payloads().len() goes 150 -> 152 (+2 variants).
//
// D-4 (SCHEMA_VERSION 70): bumped 69 -> 70 for 2 new SsbActionType variants
// covering the tripwire stack (see crates/orchestrator/src/tripwire.rs).
// All 9 lockstep arms updated atomically per CLAUDE.md SCHEMA hard rule.
//   * tripwire_fired             — one row per
//                                  `TripwireMonitor::observe` firing.
//                                  Carries (event_id, tripwire_kind,
//                                  candidate_id, observed_value, threshold,
//                                  fired_at_ms).
//   * template_auto_reverted     — one row per `tripwire::auto_revert`
//                                  success. Carries (template_id,
//                                  parent_template_id, retirement_reason,
//                                  fired_tripwire_id).
// all_test_payloads().len() goes 152 -> 154 (+2 variants).
//
// D-19 (SCHEMA_VERSION 71): bumped 70 -> 71 for 1 new SsbActionType variant
// covering the PRMBench adversarial-gaming benchmark runner (see
// crates/orchestrator/src/benchmarks/prmbench_runner.rs). All 9 lockstep
// arms updated atomically per CLAUDE.md SCHEMA hard rule.
//   * prmbench_run_completed   — one row per `PRMBenchRunner::run_all`
//                                (or `run_category`) Ok return. Carries
//                                (total, accuracy, precision, recall,
//                                gating_verdict, fixture_blake3,
//                                category_count).
// all_test_payloads().len() goes 154 -> 155 (+1 variant).
//
// D-5 (SCHEMA_VERSION 72): bumped 71 -> 72 for 1 new SsbActionType variant
// covering the Beta-posterior Thompson sampling promotion round (see
// crates/orchestrator/src/bai.rs). All 9 lockstep arms updated atomically per
// CLAUDE.md SCHEMA hard rule.
//   * bai_arm_updated            — one row per `bai::emit_bai_round` posterior
//                                  update. Carries (template_id, alpha, beta,
//                                  n_pulls, mean_estimate, was_promoted).
// all_test_payloads().len() goes 155 -> 156 (+1 variant).
//
// D-6 (SCHEMA_VERSION 73): bumped 72 -> 73 for 2 new SsbActionType variants
// covering the time-windowed held-out task store (see
// crates/orchestrator/src/held_out.rs). LiveCodeBench-style
// (arxiv:2403.07974, T1) 9-gram contamination defense + eligibility window.
//   * held_out_task_added — one row per `held_out::insert_and_emit` Ok
//                           (task_id, source, content_blake3, ngram_count,
//                           ingested_ts_ms, eligibility_window_ms).
//   * held_out_task_used  — one row per `held_out::mark_used_and_emit` Ok
//                           (task_id, consumer_template_id, used_ts_ms).
// all_test_payloads().len() goes 156 -> 158 (+2 variants).
//
// D-7 (SCHEMA_VERSION 74): bumped 73 -> 74 for 2 new SsbActionType variants
// covering the cross-domain transfer canary (see
// crates/orchestrator/src/canary.rs). NanoFlux-style
// (arxiv:2509.23252, T1) defense: every L2 mutation runs through 5
// orthogonal task families so a target-family win does not silently
// regress an adjacent family. Producer is `canary::emit_canary_run`,
// invoked from the BAI promotion gate.
//   * canary_run_completed     — one row per `canary::emit_canary_run` Ok
//                                (report_id, ts_ms, overall_transfer_score,
//                                decision_kind, dominant_fail_family).
//   * canary_family_evaluated  — one row per per-family evaluator pass
//                                inside the same run (family, n, pass, fail,
//                                transfer_score, baseline_score).
// all_test_payloads().len() goes 158 -> 160 (+2 variants).
//
// D-9 (SCHEMA_VERSION 75): bumped 74 -> 75 for 2 new SsbActionType variants
// covering the L3 frozen-surface access-control auditor (see
// crates/orchestrator/src/locked_keys_auditor.rs). Everitt current-RF
// principle (arxiv:1908.04734, T1): the agent must not edit the function
// that grades it. Producer is `LockedKeysAuditor::audit_diff_ranged`,
// invoked from the `elai audit-locked-keys` CLI subcommand which is
// wired into the workspace pre-commit hook.
//   * locked_keys_audit_run         — one row per `audit_diff_ranged`
//                                     invocation (audit_id, ts_ms,
//                                     n_audited, n_violations,
//                                     registry_blake3, registry_path).
//   * locked_keys_violation_detected — one row per overlap between a
//                                     staged edit and a registry entry
//                                     (audit_id, ts_ms, attempted_edit,
//                                     attempted_lines, locked_key_path,
//                                     locked_key_signature_blake3, reason).
// all_test_payloads().len() goes 160 -> 162 (+2 variants).
//
// E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
//   * task_advanced — dual-ledger TaskLedger/ProgressLedger atomic update.
//                     Carries a [`TaskAdvancedPayload`] (task_id,
//                     parent_id, completion_pct, blocker, ts_ms).
//   * agentless_shortcut_taken — AGENTLESS leaf-shortcut elected for a
//                     task (single-file, ≤N LOC, no cross-crate / no
//                     async network). Carries an
//                     [`AgentlessShortcutTakenPayload`] (task_id, reason,
//                     ts_ms). Grounding: arxiv:2407.01489 (AGENTLESS,
//                     32.00% SWE-bench Lite at $0.70/issue).
//   * goal_drift_fired — GoalDriftCanary fired (drift > threshold).
//                     Carries a [`GoalDriftFiredPayload`]
//                     (initial_goal_blake3, drift, threshold, ts_ms).
//                     Grounding: arxiv:2412.21154 (Devin/SWE-bench
//                     task-ledger pattern).
// all_test_payloads().len() goes 162 -> 165 (+3 variants).
//
// D-10 (SCHEMA_VERSION 77): bumped 76 -> 77 for 1 new SsbActionType variant
// covering the mode-routing predicate (orchestrator picks the grounding
// mode for every novel claim — the policy model has no input). The
// router function is L3-locked per Cohen Boxed Myopic / Everitt
// current-RF (arxiv:1908.04734). Producer is `agent_pool::route_claim`,
// which wraps `grounding_mode::select_grounding_mode` and emits the
// row before any verifier runs.
//   * claim_routed — one row per claim run through the router
//                    (claim_id, claim_type, mode, ts_ms,
//                    ctx_summary_blake3). The ctx blake3 hashes the
//                    orchestrator-state snapshot the router saw, so
//                    the dashboard can detect post-hoc context drift.
// all_test_payloads().len() goes 165 -> 166 (+1 variant).
//
// D-21 (SCHEMA_VERSION 78): bumped 77 -> 78 for 1 new SsbActionType variant
// covering the mode-routing coverage benchmark. The runner replays a
// labeled fixture of synthetic claims through `select_grounding_mode`
// and reports per-mode F1 / macro F1 / 5x5 confusion. Gate: macro F1
// >= 0.85 AND every per-mode F1 >= 0.70 (no class collapse).
//   * mode_routing_bench_run_completed -- one row per
//                    `ModeRoutingBenchmark::run_all` Ok return (total,
//                    accuracy, f1_macro, gating_verdict, fixture_blake3,
//                    fixture_row_count). Producer is
//                    `benchmarks::mode_routing_runner::ModeRoutingBenchmark`;
//                    CI consumes the row's gating_verdict (and the
//                    runner's exit code) via the `elai bench mode-routing`
//                    subcommand.
// all_test_payloads().len() goes 166 -> 167 (+1 variant).
//
// D-11 (SCHEMA_VERSION 79): bumped 78 -> 79 for 1 new SsbActionType variant
// covering the constructive-checker registry (Mode A operational substrate).
// When the D-10 router selects GroundingMode::Constructive,
// `agent_pool::dispatch_constructive_checkers` runs
// `CheckerRegistry::run_all_applicable` and emits one row per result.
//   * checker_result_emitted -- one row per
//                    (checker_id, checker_kind, result_kind, claim_id, ts_ms,
//                    reason) triple returned by the registry. Producer is the
//                    orchestrator (NOT the policy model). Grounding:
//                    arxiv:2310.04406 (constructive proof checking),
//                    arxiv:2211.12588 (PoT, T1), arxiv:2402.01030 (CodeAct, T1).
// all_test_payloads().len() goes 167 -> 168 (+1 variant).
//
// D-13 (SCHEMA_VERSION 80): bumped 79 -> 80 for 2 new SsbActionType variants
// covering the adversarial-verification pass (cross-family heterogeneous
// verifier rerun). The pass calls
// `crate::adversary::Adversary::generate_counter`, then re-runs the
// verifier consensus on `(claim, counter_evidence)`. Producer is
// `crate::agent_pool::run_adversarial_pass`. Per arxiv:2403.07974 (T1)
// adversarial-verification + arxiv:2504.16760 (T1) lightweight latent
// verifiers (heterogeneous-family verification as the documented
// defense against same-family verifier-transfer failure).
//   * adversary_generated -- one row per
//                    `run_adversarial_pass` adversary-emit step.
//                    Carries an [`AdversaryGeneratedPayload`]
//                    (counter_id, claim_id, severity, model_family).
//   * consensus_reevaluated_with_counter -- one row per re-run
//                    consensus pass that includes the counter-evidence.
//                    Carries a
//                    [`ConsensusReevaluatedWithCounterPayload`]
//                    (claim_id, original_verdict, post_counter_verdict).
// all_test_payloads().len() goes 168 -> 170 (+2 variants).
//
// D-12 (SCHEMA_VERSION 81): bumped 80 -> 81 for 1 new SsbActionType variant
// covering the OperationalizationCheck baby-step predicate
// (`crate::baby_step::is_operationalized` +
// `crate::baby_step::dispatch_check_for_baby_step`). Producer is
// `crate::agent_pool::run_operational_check`, which guards the
// MEMORIZING transition: if the predicate fails or the dispatch result
// is non-Pass, the auditor BLOCKs. Per arxiv:2305.16291 Voyager (T1)
// skill-with-measurable-effect pattern and arxiv:2407.01489 AGENTLESS
// (T1) single-step verification.
//   * baby_step_dispatched -- one row per
//                    `dispatch_check_for_baby_step` outcome. Carries
//                    [`BabyStepDispatchedPayload`] (step_id,
//                    parent_task_id, checker_kinds_used, outcome,
//                    ts_ms).
// all_test_payloads().len() goes 170 -> 171 (+1 variant).
//
// D-16 (SCHEMA_VERSION 82): bumped 81 -> 82 for 2 new SsbActionType variants
// covering the UniCR conformal-refusal calibration (formal Mode E grounding,
// arXiv:2503.01345 + Vovk et al. 2005 §2.2). Producers live in
// `crate::conformal_calibration` (pure compute + emit) and
// `crate::agent_pool::dispatch_conformal_refusal` (one-shot consumer
// that wires the D-10 GroundingMode::ConformalRefusal arm to a live
// calibration corpus).
//   * conformal_tau_computed     -- one row per `compute_tau_and_emit`
//                                   Ok return. Carries
//                                   [`ConformalTauComputedPayload`]
//                                   (tau, epsilon, corpus_size, source,
//                                   ts_ms). Source is the snake-case
//                                   `CorpusSource` label.
//   * conformal_refusal_decided  -- one row per `decide_and_emit` call.
//                                   Carries
//                                   [`ConformalRefusalDecidedPayload`]
//                                   (claim_id, claim_score, tau,
//                                   decision, confidence, ts_ms).
// all_test_payloads().len() goes 171 -> 173 (+2 variants).
//
// D-15 (SCHEMA_VERSION 83): bumped 82 -> 83 for 1 new SsbActionType variant
// covering the discovery-vs-search NoveltyClass labeling pass (real-
// problem §1 Mode C honesty layer; arXiv:2406.05761 LLM novelty
// detection three-bucket taxonomy). Producer lives in
// `crate::novelty::emit_novelty_classified` (pure predicate + emit) and
// `crate::agent_pool::dispatch_novelty` (one-shot consumer reachable
// from the D-10 Constructive / OperationalFalsification arms).
//   * novelty_classified         -- one row per `emit_novelty_classified`
//                                   call. Carries
//                                   [`NoveltyClassifiedPayload`]
//                                   (template_id, class,
//                                   dkg_match_count,
//                                   near_duplicate_threshold, ts_ms).
//                                   `class` is the snake-case
//                                   `NoveltyClass` label
//                                   (`search` / `distant_search` /
//                                   `discovery_candidate`).
// all_test_payloads().len() goes 173 -> 174 (+1 variant).
//
// D-18 (SCHEMA_VERSION 84): bumped 83 -> 84 for 1 new SsbActionType
// variant covering the typed-ObservableEffect predicate (Mode D upgrade
// extension of C-22 FeasibilityProber). Plan:
// docs/thefinalglance/D-plan/D-18_observable_effect_typed.md. The full
// 4-variant pack the plan calls for (ObservableEffectDispatched +
// GoodhartRiskFlag + SandboxObservation +
// ProductionObservationProposal) is deferred to a follow-up session;
// this pass ships the minimum-viable single-variant beacon plus the
// closed-enum typed surface in `crate::observable_effect`.
//   * observable_effect_verified -- one row per
//                                   `observable_effect::verify_observable_effect`
//                                   call. Carries
//                                   [`ObservableEffectVerifiedPayload`]
//                                   (effect_id, trigger_kind,
//                                   observation_kind, sandbox_tier,
//                                   outcome, confound_flags,
//                                   elapsed_us, ts_ms). Producer:
//                                   `crates/orchestrator/src/observable_effect.rs`.
// all_test_payloads().len() goes 174 -> 175 (+1 variant).
//
// D-17 (SCHEMA_VERSION 85): bumped 84 -> 85 for 2 new SsbActionType variants
// covering the honest-refusal SLA monitoring surface. Plan:
// docs/thefinalglance/D-plan/D-17_honest_refusal_sla.md. The MVP ships
// a sliding-window `RefusalRateMonitor` that observes the
// `ConformalRefusalDecided` decisions D-16 emits, computes a
// per-snapshot refusal rate, and classifies it against an operator-
// configured healthy band. Two beacons fire from the producer:
//   * refusal_rate_snapshot   -- one row per `compute_snapshot` call
//                                from the periodic
//                                `dispatch_refusal_sla_check`
//                                dispatcher. Carries
//                                [`RefusalRateSnapshotPayload`]
//                                (refusal_rate, window_total,
//                                abstain_count, health_band, ts_ms).
//   * refusal_sla_breach      -- one row per snapshot whose
//                                `health_band != Healthy`. Carries
//                                [`RefusalSlaBreachPayload`] (rate,
//                                threshold, breach_kind, ts_ms) where
//                                `breach_kind` is `above` | `below`.
// all_test_payloads().len() goes 175 -> 177 (+2 variants). Grounding:
// arXiv:2503.01345 (UniCR honest refusal — the SLA monitors the
// formal Mode E surface D-16 ships); docs/specs/guide/real-problem-
// solving-grounded-2026-05-19.md §4.6 (refusal rate as first-class
// quality metric).
//
// D-22 (SCHEMA_VERSION 86): bumped 85 -> 86 for 1 new SsbActionType
// variant — `ConformalValidationCompleted` — covering the conformal
// calibration validation runner that gates the UniCR formal guarantee
// (arXiv:2503.01345, T1) under distribution shift. Plan:
// docs/thefinalglance/D-plan/D-22_conformal_calibration_validation.md.
// One row per `ConformalValidationRunner::run_all` Ok return. Carries
// [`ConformalValidationCompletedPayload`] (tau, epsilon,
// empirical_refusal_rate, gating_verdict, fixture_blake3, total, ts_ms).
// Producer:
// `crates/orchestrator/src/benchmarks/conformal_validation_runner.rs`.
// Consumer: `elai bench conformal` CLI exit code + dashboard
// `ConformalValidationPanel`. all_test_payloads().len() goes 177 -> 178.
//
// D-20 (SCHEMA_VERSION 87): bumped 86 -> 87 for 1 new SsbActionType
// variant — `ArcBenchmarkCompleted` — covering the ARC-AGI-2 OOD probe
// harness that gates ELAI's mode-routing + novelty-classifier honesty
// stack against ARC-style out-of-distribution claims. Plan:
// docs/thefinalglance/D-plan/D-20_arc_agi2_ood_probe.md. One row per
// `ARCAGI2Runner::run_all` Ok return. Carries
// [`ARCBenchmarkCompletedPayload`] (total, honestly_evaluated_count,
// gating_verdict, fixture_blake3, dominant_mode,
// dominant_novelty_class, ts_ms). Producer:
// `crates/orchestrator/src/benchmarks/arc_agi2_runner.rs`. Consumer:
// `elai bench arc` CLI exit code + dashboard `ARCBenchPanel`.
// TODO_DOC[D-20] — paper-ingestor follow-up to anchor against
// arXiv:2412.04261 (ARC-AGI-2) and arXiv:2601.10904 (ARC Prize 2025
// Tech Report); citations dropped from module doc per the anti-
// citation-rot rule until paper-ingestor lands a workflow/papers/
// entry. all_test_payloads().len() goes 178 -> 179.
const SCHEMA_VERSION: i64 = 87;

/// Public re-export of [`SCHEMA_VERSION`] for contract / integration tests
/// outside the orchestrator crate. The const itself stays crate-private so
/// internal callers cannot drift from the canonical value.
pub const SCHEMA_VERSION_PUBLIC: i64 = SCHEMA_VERSION;

/// Maximum serialized payload size in bytes (1 MiB).
/// `write_payload()` rejects payloads exceeding this limit with `SsbError::PayloadTooLarge`.
const PAYLOAD_MAX_BYTES: usize = 1_048_576;

const CREATE_SSB_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS ssb (
    seq         INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  TEXT NOT NULL,
    session_id  TEXT NOT NULL,
    agent_id    TEXT NOT NULL,
    superstep   INTEGER NOT NULL DEFAULT 0,
    action_type TEXT NOT NULL CHECK(action_type IN (
        'file_write', 'file_read', 'tool_result', 'test_outcome', 'finding',
        'claim', 'checkpoint', 'model_swap', 'phase_transition',
        'knowledge_degradation', 'isolation_metric', 'plan_update',
        'replan_request', 'stub_reference', 'reasoning_trace',
        'phase_complete', 'memory_monitoring', 'workload_profile',
        'death_event', 'recovery_event',
        'supervision_exhausted', 'spiral_detected',
        'context_reduction', 'branch_merge',
        'mistake_entry', 'research_step',
        'research_decompose_level', 'web_research_step',
        'preview_available',
        'intervention_request', 'intervention_response',
        'task_submitted',
        'hook_event',
        'provider_signals', 'search_trigger_fired', 'vram_tier_detected',
        'indexing_progress', 'dkg_ready',
        'tiered_analysis', 'shortcoder_compression',
        'tool_dispatch', 'validation_tier', 'failure_classified',
        'git_commit', 'daira_trace',
        'system_probe', 'feasibility_report', 'novel_step_detected', 'stub_run',
        'gate_decision', 'task_split', 'escalation_fired', 'mode_changed',
        'proposal_generated', 'proposal_applied', 'proposal_rejected',
        'msi_update', 'config_archived',
        'profile_selected', 'fsm_transition', 'micro_reflection',
        'telescope_collapse', 'config_warning',
        'constraints_injected', 'intervention_resolved',
        'dkg_querier_swapped',
        'steering_computed', 'steering_compute_failed',
        'chat_message', 'chat_response',
        'execution_mode_changed',
        -- B-10 (DECISION-C): per-dispatch ContextBudget snapshot + 3E.5
        -- SSB entry filtering result for the dashboard's per-slot bar chart.
        'context_budget_computed', 'ssb_filtered_by_overlap',
        -- B-19A (T10): hook latency budget violation beacon emitted by
        -- the dispatcher when a T1/T2/T3 hook overruns its tier budget.
        'hook_latency_violation',
        -- batch-ful-d (A.2/A.6): research path selection + grounding cache hit/miss.
        'research_path_chosen', 'grounding_cache_event',
        -- Track B (s-1430428): MCP bridge lifecycle + tool-call beacons.
        -- Enum variants McpServerConnected / McpToolCalled / McpServerDisconnected
        -- already declared at L283-291; from_str arms at L498-500.
        'mcp_server_connected', 'mcp_tool_called', 'mcp_server_disconnected',
        -- B-4: circuit-breaker state transition for embedding/reranker models.
        'circuit_breaker_state_change',
        -- B-7: bubble-up failure cascade, integration checkpoints, DAG repair, decomposition.
        'bubble_up_typed', 'integration_checkpoint_success',
        'dag_repair_round', 'concept_decomposition_node',
        -- B-6: orchestration producer wiring — evidence gates, supervision, BSP, agent lifecycle.
        'evidence_gate_evaluated', 'supervision_event',
        'bsp_superstep', 'agent_lifecycle', 'anti_spiral_hit',
        -- B-27 (SCHEMA_VERSION 27): cloud-mode dual-rate spend ledger row +
        -- one-shot escalation marker emitted when local exhausts retries
        -- and cloud_mode promotes the step. cloud_mode.rs is the sole writer.
        'cloud_spend', 'escalate_to_cloud',
        -- B-23 Phase 4H Group A (SCHEMA_VERSION 28): 6 new self-mod beacons.
        -- data_table_grew    -- Layer 1A row insert event (per spec phase-4H.txt:140)
        -- l3_proposal         -- Layer 3 structural-change proposal (HUMAN-ONLY review)
        -- convergence_stagnant-- OSC verdict (3 consecutive >=0.95 similarity)
        -- config_oscillation  -- OSC period-2 verdict (sim(N,N-2) AND sim(N-1,N-3))
        -- regression_detected -- REG cross-pipeline AC regression report
        -- evolution_withheld  -- EVO gate Withhold (config_blob never evolved)
        'data_table_grew', 'l3_proposal', 'convergence_stagnant',
        'config_oscillation', 'regression_detected', 'evolution_withheld',
        -- B-39 chat reflect pipeline: per-turn REFLECT stage beacons.
        'reflect_triaged', 'reflect_candidate', 'reflect_audit_verdict', 'reflect_refined',
        -- B-24 Phase 4I Group C1: media validator subsystem (Tier 1 / Tier 2 /
        -- Render-Capture-Evaluate / Visual-Feedback / HumanReview branch).
        -- Payloads ship in Group C4 alongside the dashboard wiring; the
        -- enum + SQL CHECK + as_sql_str arms are added now so the SCHEMA_VERSION
        -- bump (29 -> 30) is atomic.
        'media_validation_started', 'media_tier1_complete', 'media_tier2_complete',
        'render_complete', 'visual_feedback_iteration', 'media_human_review_required',
        -- B-28 E-3 (SCHEMA_VERSION 31): focus_tunnel ModeSelector beacon.
        -- generation_mode_selected — emitted once per baby-step decision in
        -- drive_task immediately before the executor dispatch. Carries the
        -- chosen GenerationMode plus the SelectionRule that fired, so the
        -- dashboard's mode_distribution surface and the auditor's
        -- first-match-wins check both read from one source.
        'generation_mode_selected',
        -- B-28 E-4 (SCHEMA_VERSION 32): focus_tunnel StreamingValidator
        -- early-termination beacon. Emitted by the post-E-7 WIRING phase
        -- whenever the library-tier validator returns ValidationDecision
        -- ::EarlyTerm. Carries the wire-form EarlyTermReason (lazy_pattern /
        -- unbalanced_braces / output_too_long / deprecated_import /
        -- unbounded_string_literal) plus the token count at termination so
        -- the dashboard can render an early-term row keyed by reason.
        'streaming_early_terminated',
        -- B-28 WIRING-1 (SCHEMA_VERSION 33): focus_tunnel
        -- PipelinePreAssembler readiness beacon. One row per
        -- PipelinePreAssembler::on_step_outcome -> prepare() invocation.
        -- Carries pass_rate_bp (basis points) + was_discarded + depth +
        -- bytes_assembled so the dashboard can render the parallelism gauge
        -- and waste-rate bar without aggregating across multiple rows.
        'pipeline_pre_assembled',
        -- B-28 WIRING-1 (SCHEMA_VERSION 34): focus_tunnel DependencyResolver
        -- decision beacon. Emitted once per ResolutionDecision after the
        -- post-execute import-scan + resolve cycle. Carries the resolved
        -- language + name + decision string (AutoApprove / QueueForHuman /
        -- Blocked) + the reason that gated the decision. install_command
        -- mirrors the AutoApprove ApplyPlan when present.
        'dependency_resolved',
        -- B-28 WIRING-1 (SCHEMA_VERSION 35): focus_tunnel SpeculativeFileRouter
        -- micro-step creation / cap-exhaustion beacon. Emitted once per
        -- NeedsMicroStep or MicroStepCapExhausted route decision. Carries
        -- the parent step id + created_path + type_hint (Module / Struct /
        -- Function / Const) + reason string (the gating cue) so the
        -- dashboard can render a per-step expansion trail.
        'micro_step_created',
        -- B-30 (SCHEMA_VERSION 36): one-shot LanguageRegistry status emitted
        -- at DKG engine startup. Carries loaded + failed grammar lists so
        -- the dashboard KnowledgeDrawer can render the LANGUAGES LOADED card.
        'language_grammar_status',
        -- B-29 Phase 5 (SCHEMA_VERSION 37): symbol_query action for per-tool
        -- symbol_impact / symbol_flow / symbol_context / symbol_list calls.
        -- Consumer: dashboard KnowledgeDrawer Impact Analysis panel + ActivityFeed.
        'symbol_query',
        -- B-32 (SCHEMA_VERSION 38): minified_file_chunked action emitted once
        -- per minified file by IndexingService after the BM25 batch loop.
        -- Producer: IndexingService::build_index_resumable post-DKG walkback.
        -- Consumer: dashboard KnowledgeDrawer minified-file surface.
        'minified_file_chunked',
        -- B-31.5 (SCHEMA_VERSION 39): the 5 deferred B-31 SSB variants.
        -- index_checkpoint  -- per-batch checkpoint emitted by build_index_resumable
        -- index_resume      -- emitted when build_index_resumable returns Resumed
        -- index_cancel      -- emitted when build_index_resumable returns Cancelled
        --                       OR when graceful_shutdown signals a project to cancel
        -- lock_acquired     -- IndexLock::try_acquire success
        -- lock_contested    -- IndexLock::try_acquire AlreadyHeld
        'index_checkpoint', 'index_resume', 'index_cancel',
        'lock_acquired', 'lock_contested',
        -- B-33 follow-up (SCHEMA_VERSION 40): SymbolGraphUpdated. One row
        -- per successful symbol-graph build emitted by IndexingService
        -- after build_pipeline::build_symbol_graph() returns Ok. Carries
        -- SymbolGraphUpdatedPayload { project_id, file_count, symbol_count,
        -- edge_count, built_ms } so the dashboard GraphWindow can refresh
        -- without polling. Replaces the 30s setInterval fallback shipped
        -- in B-33 (D3) which was DEFERRED at session time because
        -- ssb.rs was locked by s-40304.
        'symbol_graph_updated',
        -- B-40 (SCHEMA_VERSION 41): FRAMING substate beacons emitted by
        -- orchestrator::idea_framing during the pre-decomposition pass.
        -- idea_framing_started     -- one row on FRAMING substate entry
        -- project_class_classified -- one row when classify_project_class
        --                             returns; carries ProjectClass + confidence
        -- critic_round_complete    -- one row per CriticLoop round (1-3);
        --                             carries round_index + finding counts
        --                             grouped by 10 CriticCategory variants
        -- idea_framed              -- one row when frame_idea() persists
        --                             framed-idea.toml under
        --                             workflow/sessions/{sid}/idea-framing/
        -- tradeoff_card_emitted    -- one row when TradeoffCard is surfaced
        --                             to the dashboard for user response
        'idea_framing_started', 'project_class_classified',
        'critic_round_complete', 'idea_framed', 'tradeoff_card_emitted',
        -- B-34 (SCHEMA_VERSION 42): entry-point detection + artifact pipeline beacons.
        -- entry_points_detected -- one row per entry_points tool call; carries project_id,
        --                          count, and by_reason breakdown.
        -- artifacts_indexed     -- one row per ArtifactIndexer completion; carries indexed
        --                          names, up_to_date names, total_chunks, latency_ms.
        -- artifact_searched     -- one row per search_artifacts tool call; carries query,
        --                          optional artifact_name filter, result_count, latency_ms.
        -- artifact_stale        -- one row per stale artifact detection (content_hash mismatch);
        --                          carries artifact_name + changed_files list.
        'entry_points_detected', 'artifacts_indexed', 'artifact_searched', 'artifact_stale',
        -- B-38 (SCHEMA_VERSION 43): ambiguity-score continuous-metric gate beacons.
        -- ambiguity_scored          -- one row per AmbiguityScorer.score() result;
        --                              carries overall score, milestone, per-dimension
        --                              clarity scores, optional context clarity, and the
        --                              brownfield flag.
        -- ambiguity_floor_failed    -- one row per gate verdict where a per-dimension floor
        --                              was breached even with overall <= 0.2; carries the
        --                              score and the failed-dimension names.
        -- ambiguity_streak_advanced -- one row per qualifying-but-streak-incomplete gate
        --                              verdict; carries the score plus current/required
        --                              streak counts.
        'ambiguity_scored', 'ambiguity_floor_failed', 'ambiguity_streak_advanced',
        -- B-42 (SCHEMA_VERSION 44): cosmos tier knowledge worker beacons.
        -- cosmos_refresh_started   -- one row per refresh tick start (CosmosWorker)
        -- cosmos_artifact_updated  -- one row per successful refresh
        -- cosmos_artifact_decayed  -- one row when artifact exceeds 3 * cadence
        -- cosmos_grounding_used    -- one row per CosmosCheck verifier dispatch
        'cosmos_refresh_started', 'cosmos_artifact_updated',
        'cosmos_artifact_decayed', 'cosmos_grounding_used',
        -- B-41 (SCHEMA_VERSION 45): per-level auditor mode beacons.
        -- per_level_audit_started -- one row per AuditorConcepts/Phases/Steps/
        --                            BabySteps/BabyStepDetail spawn at a tier
        --                            node (mode + tier + node_id).
        -- per_level_audit_verdict -- one row per LevelAuditVerdict produced
        --                            (mode + tier + node_id + verdict +
        --                            findings_count + respawned_once +
        --                            cascade_to_parent).
        -- per_level_audit_cascade -- one row when a 2nd BLOCK at a tier sets
        --                            cascade_to_parent=true and the navigator
        --                            bubbles up to the parent decomposer
        --                            (mode + tier + node_id + parent_tier).
        'per_level_audit_started', 'per_level_audit_verdict',
        'per_level_audit_cascade',
        -- B-37C (SCHEMA_VERSION 46): multi-runtime backend abstraction beacons.
        -- runtime_fallback         -- one row per fallback chain step (primary -> next backend)
        -- runtime_health_changed   -- one row per per-backend RuntimeHealth transition
        -- cli_subprocess_spawned   -- one row per CLI adapter subprocess spawn
        -- cli_subprocess_killed    -- one row per CLI adapter subprocess kill / exit
        'runtime_fallback', 'runtime_health_changed',
        'cli_subprocess_spawned', 'cli_subprocess_killed',
        -- C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
        -- grc_step_emitted              -- per emit_reasoning call (real PipelineGrc budget)
        -- grounding_sweep_result        -- one row per ground_plan() call (task_runner)
        -- csq_evaluated                 -- one row per generate_csq() result (task_runner)
        -- auditor_rubber_stamp_rejected -- engine.rs InsufficientFindings surfaced at PipelineGrc caller
        -- hyde_query                    -- one row per HydeStage::hypothesize call
        -- vector_cache_hit_rate         -- aggregate row per VectorCache::flush_hit_rate_ssb
        -- query_trace                   -- per QueryPipeline::query_with_tier completion
        'grc_step_emitted', 'grounding_sweep_result', 'csq_evaluated',
        'auditor_rubber_stamp_rejected', 'hyde_query',
        'vector_cache_hit_rate', 'query_trace',
        -- C-2 (SCHEMA_VERSION 48): orchestration + cosmos + stagnation beacons.
        -- phase_node_ready              -- task_runner baby-step loop crosses a phase boundary
        -- step_node_ready               -- task_runner baby-step loop crosses a step boundary
        -- baby_step_ready               -- task_runner per baby-step entry
        -- dag_repair_escalated          -- pipeline_builder RepairOutcome::Escalated
        -- agent_status_changed          -- mirror of AgentLifecycle for live-agent panel
        -- stagnation_spinning_detected  -- C-10 anti-spiral detector (definition only here)
        -- stagnation_oscillation_detected -- C-10 anti-spiral detector (definition only here)
        -- lateral_thinking_activated    -- C-10 persona switch (definition only here)
        -- goal_drift_warning            -- C-10 goal-drift canary (definition only here)
        -- watchdog_timeout              -- C-10 generation watchdog (definition only here)
        'phase_node_ready', 'step_node_ready', 'baby_step_ready',
        'dag_repair_escalated', 'agent_status_changed',
        'stagnation_spinning_detected', 'stagnation_oscillation_detected',
        'lateral_thinking_activated', 'goal_drift_warning', 'watchdog_timeout',
        -- C-7 (SCHEMA_VERSION 49): reviewer verdict beacon emitted by the
        -- dashboard-v2 Tauri commands accept_change / decline_change /
        -- retry_audit. One row per reviewer action; `decision` discriminates
        -- accepted/declined/retried; `hunk_id` is null for file-level
        -- verdicts and set for per-hunk verdicts.
        'change_decision',
        -- C-8 (SCHEMA_VERSION 50): degradation-cascade verdict bridge between
        -- the live_cli AppState and the dashboard-v2 process. Emitted by the
        -- CLI at startup_readiness_check and on cascade transitions; mirrored
        -- by the dashboard-v2 Tauri set_search_mode command. The dashboard
        -- get_search_mode command reads the most-recent row (graceful empty
        -- fallback returns hybrid). mode is the resolved SearchMode label;
        -- history is the trailing N transitions so the drawer renders a
        -- timeline without N round-trips. See C-8-decisions.md DRIFT-I/J.
        'search_mode_changed',
        -- C-5 (SCHEMA_VERSION 51): cloud-mode hardening — per-call privacy
        -- redaction event emitted by RedactingBabyStepExecutor whenever the
        -- privacy_redactor scrubs PII or secrets from a cloud-bound
        -- baby-step payload. Producer: cloud_mode::RedactingBabyStepExecutor.
        -- Consumer: dashboard CloudPanel.svelte privacy-event feed. Carries
        -- redaction count + the list of logical fields touched.
        'redaction_applied',
        -- C-10 (SCHEMA_VERSION 52): three new anti-spiral lateral-thinking
        -- + goal-drift block beacons (companion to the C-2 stagnation
        -- variants already at L484-485). Definition-only here; producer
        -- wiring lands at task_runner step boundaries via
        -- anti_spiral_extended.rs.
        'lateral_thinking_succeeded', 'lateral_thinking_exhausted',
        'goal_drift_block',
        -- C-13 (SCHEMA_VERSION 53): IPC-dispatcher production-swap beacon.
        -- Emitted by `squash_wip_commits_if_any` (wip_squash.rs) at the
        -- post-AUDITING -> MEMORIZING boundary AND by the `elai wip-squash`
        -- CLI subcommand each time a leading-WIP-commit squash actually
        -- ran. Carries (wip_count, squashed). Producer wiring lives in
        -- task_runner.rs::squash_wip_commits + cli/src/wip_squash_subcommand.rs.
        'wip_squash_complete',
        -- C-22 (SCHEMA_VERSION 54): feasibility-probe completeness beacons.
        -- alternative_explored: per-NovelStep alternative emission;
        -- feasibility_escalated: FeasibilityDecision::Escalate decision emission.
        'alternative_explored', 'feasibility_escalated',
        -- C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
        -- branch_forked: one row per successful SessionDag::create_branch
        --                emit (replaces ad-hoc fork log lines).
        -- branch_merged: one row per merge_branch_with_ssb Winner — supersedes
        --                the legacy `branch_merge`/Checkpoint discriminator pair.
        -- branch_closed: one row per close_branch_with_ssb success.
        'branch_forked', 'branch_merged', 'branch_closed',
        -- C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
        -- + background-task beacons. C-25 ships the FIRST plan_skeleton
        -- emitter (no producer existed before this bump — see C-25 DRIFT-D).
        -- plan_skeleton             -- one row per pipeline_builder factory entry
        -- compiled_pipeline_fired   -- one row per CompiledExecutor cache HIT + run
        -- template_crystallized     -- one row per TemplateCrystallizer success
        -- background_task_started   -- one row per gated_spawn success
        -- background_task_complete  -- one row per harvested handle (or auto-detach)
        'plan_skeleton', 'compiled_pipeline_fired', 'template_crystallized',
        'background_task_started', 'background_task_complete',
        -- C-25 audit follow-up (SCHEMA_VERSION 57): missing 6th beacon
        -- from the C-25 WIRING_CONTRACT. Producer:
        -- compiled_pipeline::CompiledExecutor::emit_decompile_triggered.
        -- Carries concept_class, plan_skeleton_hash, failure_count,
        -- trigger_reason (snake-case discriminant: confidence | consecutive_failures).
        'decompile_triggered',
        -- C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
        -- futility_detected            -- NavigatorTree same-error-class window
        --                                 saturates (default threshold = 3).
        --                                 Carries error_class, consecutive_count,
        --                                 level (baby_step|step|phase|concept),
        --                                 escalated (bool).
        -- futility_output_stagnation   -- GoalDriftCanary similarity > 0.90
        --                                 between consecutive outputs. Carries
        --                                 similarity, threshold, level,
        --                                 new_persona (snake-case persona).
        'futility_detected', 'futility_output_stagnation',
        -- C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) DeterminismConfig
        -- + T-0 WiringContract foundation. 3 new beacons emitted atomically.
        -- wiring_evidence       -- DkgReachabilityAuditor produces_symbol verified reachable
        -- wiring_violation      -- StubBanAuditor / DkgReachabilityAuditor /
        --                          ObservableEffectRunner violation
        -- determinism_drift     -- DriftComparator divergence; zero rows on identical runs
        'wiring_evidence', 'wiring_violation', 'determinism_drift',
        -- C-28 (SCHEMA_VERSION 60): Testing-trust T-1 mutation testing gate.
        -- mutation_survived     -- One row per surviving mutant emitted from
        --                          DeterministicAuditor Step 6 (post-Build /
        --                          VersionGate / FileSizeCensus / LazyCode /
        --                          Completeness). Carries file, line, mutant_kind.
        --                          Caught / Timeout / Skipped emit ZERO rows
        --                          (stub_ban contract). The mutation_score
        --                          gate (default 60%) is enforced via a
        --                          DeterministicFinding::Block, not via
        --                          row count.
        'mutation_survived',
        -- C-29 (SCHEMA_VERSION 61): Testing-trust T-2 IntentPair gate +
        -- T-3 basic metamorphic relations.
        -- intent_confirmed      -- One row per BabyStep that passes the
        --                          PLANNING -> TESTING_FIRST IntentPair
        --                          gate (pair_count >= IntentConfig.min_pairs).
        --                          Carries step_id, pair_count, confirmed_by.
        -- mr_violated           -- One row per failed metamorphic relation
        --                          emitted by deterministic_auditor Step 7.
        --                          counterexample MUST be non-empty
        --                          (debug_assert + runtime sentinel fallback).
        'intent_confirmed', 'mr_violated',
        -- C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode pipeline beacons.
        -- surgical_plan_created      -- one row per SurgicalDecomposer::decompose
        --                               Ok return. Carries complexity_class,
        --                               blast_radius_files, total_loc_estimate,
        --                               step_count.
        -- surgical_loc_warning       -- one row per decompose call whose
        --                               total_loc_estimate exceeds 500.
        --                               Carries total_loc_estimate + threshold
        --                               + mode (interactive|autonomous).
        -- oracle_cache_hit           -- one row per OracleAnswerCache::get hit.
        --                               Carries query_hash_hex + source_tier +
        --                               cached_age_secs.
        -- oracle_cache_miss          -- one row per OracleAnswerCache::insert.
        --                               Carries query_hash_hex + confidence.
        -- project_detection_signal   -- one row per signal evaluated by the
        --                               fresh-project detector. Carries
        --                               signal_name + weight + fired.
        'surgical_plan_created', 'surgical_loc_warning',
        'oracle_cache_hit', 'oracle_cache_miss',
        'project_detection_signal',
        -- C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 beacons.
        -- skeleton_generated             -- one row per SkeletonGenerator::generate
        --                                   Ok return. Carries blank_count,
        --                                   language, file_path.
        -- skeleton_flushed               -- one row per SkeletonGenerator::flush_to_disk
        --                                   success. Carries path, completeness_ratio.
        -- mini_critic_verdict            -- one row per MiniCritic::check. Carries
        --                                   blank_id, verdict, reason, language.
        -- structural_retry_stage_attempt -- one row per StructuralRetrySequencer
        --                                   stage invocation. Carries blank_id,
        --                                   stage, attempt, outcome.
        -- bridge_file_generated          -- one row per BridgeFileTemplate::write_bridge
        --                                   success. Carries kind, path, module_count.
        'skeleton_generated', 'skeleton_flushed',
        'mini_critic_verdict',
        'structural_retry_stage_attempt',
        'bridge_file_generated',
        -- D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle beacons.
        -- archive_entry_added         -- one row per ArchiveChain::append (template_id,
        --                                content_hash, parent_hash, domain, pattern_kind,
        --                                archive_status).
        -- archive_entry_retired       -- one row per template retirement (template_id,
        --                                retired_at_ms, retirement_reason).
        -- template_promoted           -- one row per BAI promotion (template_id, stage,
        --                                pass_rate). D-5 producer lands later.
        -- template_candidate_rejected -- one row per Candidate rejection (template_id,
        --                                rejection_reason).
        'archive_entry_added', 'archive_entry_retired',
        'template_promoted', 'template_candidate_rejected',
        -- E-5 (SCHEMA_VERSION 65): bi-temporal memory_facts lifecycle beacons.
        -- memory_fact_extracted   -- one row per FactStore::insert success
        --                            (Mem0-style ADD; full ADD/UPDATE/DELETE/NOOP
        --                            dispatcher reserved for the follow-up vertical slice).
        -- memory_fact_superseded  -- one row per FactStore::supersede success
        --                            (Zep bi-temporal close + replacement pair).
        'memory_fact_extracted', 'memory_fact_superseded',
        -- D-2 (SCHEMA_VERSION 66): PPM (Process Preference Model) verifier-side
        -- beacons. ppm_scored is emitted on every PpmClient::score success that
        -- clears the configured threshold; ppm_gating_failed is emitted when
        -- the gate trips.
        'ppm_scored', 'ppm_gating_failed',
        -- E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
        -- chat_reflection_completed -- one row per reflect() Ok return.
        --                              Carries (note_id, note_preview,
        --                              facts_extracted, decay rows examined /
        --                              updated, core_block_bytes).
        -- memory_evolution_run      -- one row per SLEEP_COMPUTE substate
        --                              entry / exit. Carries (phase,
        --                              duration_ms, notes_touched).
        'chat_reflection_completed', 'memory_evolution_run',
        -- E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
        -- producer + conflict beacons.
        -- memory_note_inserted     -- one row per NoteStore::insert Ok.
        -- memory_note_evolved      -- one row per A-MEM note evolution.
        -- memory_conflict_detected -- one row per duplicate / contradiction
        --                             adjudicator emission.
        -- core_block_replaced      -- one row per CoreBlockStore::upsert
        --                             replacing an existing row.
        'memory_note_inserted', 'memory_note_evolved',
        'memory_conflict_detected', 'core_block_replaced',
        -- D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator beacons.
        -- verifier_consensus_pass -- one row per run_consensus_pass Ok with
        --                            verdict = Pass.
        -- verifier_consensus_fail -- one row per run_consensus_pass Ok with
        --                            verdict != Pass.
        'verifier_consensus_pass', 'verifier_consensus_fail',
        -- D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
        -- tripwire_fired          -- one row per TripwireMonitor::observe
        --                            firing (KL / Δρ / IQR / perplexity /
        --                            DDM).
        -- template_auto_reverted  -- one row per tripwire::auto_revert
        --                            success (template flipped to Retired
        --                            with RetirementReason::Degradation).
        'tripwire_fired', 'template_auto_reverted',
        -- D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark runner beacon.
        -- prmbench_run_completed -- one row per PRMBenchRunner::run_all (or
        --                           run_category) Ok return (total, accuracy,
        --                           precision, recall, gating_verdict,
        --                           fixture_blake3, category_count).
        'prmbench_run_completed',
        -- D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
        -- bai_arm_updated         -- one row per `bai::emit_bai_round` posterior
        --                            update (template_id, alpha, beta, n_pulls,
        --                            mean_estimate, was_promoted).
        'bai_arm_updated',
        -- D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
        -- held_out_task_added     -- one row per `held_out::insert_and_emit` Ok
        --                            (task_id, source, content_blake3,
        --                            ngram_count, ingested_ts_ms,
        --                            eligibility_window_ms).
        -- held_out_task_used      -- one row per `held_out::mark_used_and_emit`
        --                            Ok (task_id, consumer_template_id,
        --                            used_ts_ms).
        'held_out_task_added', 'held_out_task_used',
        -- D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
        -- canary_run_completed     -- one row per `canary::emit_canary_run` Ok
        --                             (report_id, ts_ms, overall_transfer_score,
        --                             decision_kind, dominant_fail_family).
        -- canary_family_evaluated  -- one row per per-family evaluator pass
        --                             inside the same run (family, n, pass,
        --                             fail, transfer_score, baseline_score).
        'canary_run_completed', 'canary_family_evaluated',
        -- D-9 (SCHEMA_VERSION 75): L3 frozen-surface access-control auditor.
        -- locked_keys_audit_run         -- one row per LockedKeysAuditor
        --                                  audit_diff_ranged() invocation
        --                                  (audit_id, ts_ms, n_audited,
        --                                  n_violations, registry_blake3,
        --                                  registry_path).
        -- locked_keys_violation_detected -- one row per registry overlap
        --                                  on a staged edit (audit_id,
        --                                  ts_ms, attempted_edit,
        --                                  attempted_lines,
        --                                  locked_key_path,
        --                                  locked_key_signature_blake3,
        --                                  reason).
        'locked_keys_audit_run', 'locked_keys_violation_detected',
        -- E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
        -- task_advanced              -- dual-ledger TaskLedger+ProgressLedger
        --                              atomic advance (task_id, parent_id,
        --                              completion_pct, status, blocker, ts_ms).
        -- agentless_shortcut_taken   -- AGENTLESS leaf-shortcut elected
        --                              (task_id, reason, ts_ms);
        --                              arxiv:2407.01489.
        -- goal_drift_fired           -- GoalDriftCanary fired
        --                              (initial_goal_blake3, drift,
        --                              threshold, step, ts_ms);
        --                              arxiv:2505.02709 / arxiv:2412.21154.
        'task_advanced', 'agentless_shortcut_taken', 'goal_drift_fired',
        -- D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
        -- claim_routed -- one row per `agent_pool::route_claim` call
        --                 (claim_id, claim_type, mode, ts_ms,
        --                 ctx_summary_blake3). Orchestrator (NOT the
        --                 policy model) is the sole emitter; the
        --                 routing function is L3-locked per Everitt
        --                 current-RF (arxiv:1908.04734).
        'claim_routed',
        -- D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
        -- mode_routing_bench_run_completed -- one row per
        --                 `ModeRoutingBenchmark::run_all` Ok return
        --                 (total, accuracy, f1_macro, gating_verdict,
        --                 fixture_blake3, fixture_row_count). The runner
        --                 replays a hand-labeled fixture through D-10's
        --                 `select_grounding_mode` and gates pass iff
        --                 macro F1 >= 0.85 AND every per-mode F1 >= 0.70.
        'mode_routing_bench_run_completed',
        -- D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
        -- checker_result_emitted -- one row per (checker_id, checker_kind,
        --                 result_kind, claim_id, ts_ms, reason) triple emitted
        --                 by `CheckerRegistry::run_all_applicable`. Producer
        --                 is the orchestrator (NOT the policy model);
        --                 grounding arxiv:2310.04406.
        'checker_result_emitted',
        -- D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
        -- adversary_generated                -- counter_id, claim_id,
        --                                       severity, model_family.
        -- consensus_reevaluated_with_counter -- claim_id, original_verdict,
        --                                       post_counter_verdict.
        'adversary_generated',
        'consensus_reevaluated_with_counter',
        -- D-12 (SCHEMA_VERSION 81): OperationalizationCheck baby-step
        -- predicate dispatch beacon. Emitted by
        -- `agent_pool::run_operational_check` after
        -- `dispatch_check_for_baby_step` returns. Per arxiv:2305.16291
        -- Voyager (T1) skill-with-measurable-effect + arxiv:2407.01489
        -- AGENTLESS (T1) single-step verification.
        'baby_step_dispatched',
        -- D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal calibration
        -- beacons. Per arXiv:2503.01345 (UniCR) + Vovk et al. 2005 §2.2.
        -- conformal_tau_computed     -- tau, epsilon, corpus_size, source,
        --                               ts_ms; emitted by
        --                               `compute_tau_and_emit`.
        -- conformal_refusal_decided  -- claim_id, claim_score, tau,
        --                               decision, confidence, ts_ms;
        --                               emitted by `decide_and_emit` and
        --                               `agent_pool::dispatch_conformal_refusal`.
        'conformal_tau_computed',
        'conformal_refusal_decided',
        -- D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass
        -- labeling pass. Per arXiv:2406.05761 (LLM novelty detection)
        -- + real-problem §1 Mode C honesty layer.
        -- novelty_classified           -- template_id, class,
        --                                 dkg_match_count,
        --                                 near_duplicate_threshold,
        --                                 ts_ms; emitted by
        --                                 `novelty::emit_novelty_classified`
        --                                 and `agent_pool::dispatch_novelty`.
        'novelty_classified',
        -- D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate
        -- beacon. Per docs/thefinalglance/D-plan/D-18_observable_effect_typed.md.
        -- observable_effect_verified -- effect_id, trigger_kind,
        --                               observation_kind, sandbox_tier,
        --                               outcome (pass|fail|skipped),
        --                               confound_flags, elapsed_us,
        --                               ts_ms; emitted by
        --                               `observable_effect::verify_observable_effect`.
        'observable_effect_verified',
        -- D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
        -- Per docs/thefinalglance/D-plan/D-17_honest_refusal_sla.md.
        -- refusal_rate_snapshot -- refusal_rate, window_total,
        --                          abstain_count, health_band
        --                          (below|healthy|above), ts_ms;
        --                          emitted by
        --                          `refusal_sla::RefusalRateMonitor::emit_snapshot`.
        -- refusal_sla_breach     -- rate, threshold, breach_kind
        --                          (above|below), ts_ms; emitted when
        --                          `sla_breach(snapshot) == true`.
        'refusal_rate_snapshot', 'refusal_sla_breach',
        -- D-22 (SCHEMA_VERSION 86): conformal calibration validation beacon.
        -- Per docs/thefinalglance/D-plan/D-22_conformal_calibration_validation.md.
        -- conformal_validation_completed -- tau, epsilon,
        --                                   empirical_refusal_rate,
        --                                   gating_verdict, fixture_blake3,
        --                                   total, ts_ms; emitted by
        --                                   `benchmarks::conformal_validation_runner::ConformalValidationRunner::run_all`
        --                                   after a successful run against
        --                                   a held-out validation fixture.
        'conformal_validation_completed',
        -- D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
        -- Per docs/thefinalglance/D-plan/D-20_arc_agi2_ood_probe.md.
        -- arc_benchmark_completed -- total, honestly_evaluated_count,
        --                            gating_verdict (honestly_evaluated|
        --                            game_with_memorization|refused),
        --                            fixture_blake3, dominant_mode,
        --                            dominant_novelty_class, ts_ms;
        --                            emitted by
        --                            `benchmarks::arc_agi2_runner::ARCAGI2Runner::run_all`
        --                            after a successful run against an
        --                            ARC-AGI-2 fixture (mini bundled or
        --                            full 50-task subset).
        'arc_benchmark_completed'
    )),
    payload     BLOB NOT NULL,
    created_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ssb_project     ON ssb(project_id);
CREATE INDEX IF NOT EXISTS idx_ssb_session     ON ssb(session_id);
CREATE INDEX IF NOT EXISTS idx_ssb_superstep   ON ssb(superstep);
CREATE INDEX IF NOT EXISTS idx_ssb_conflict    ON ssb(project_id, superstep);
CREATE INDEX IF NOT EXISTS idx_ssb_action_type ON ssb(action_type);
CREATE INDEX IF NOT EXISTS idx_ssb_agent_id    ON ssb(agent_id);
";

const CREATE_SSB_FILES_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS ssb_files (
    seq       INTEGER NOT NULL REFERENCES ssb(seq),
    file_path TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ssb_files_path ON ssb_files(file_path);
CREATE INDEX IF NOT EXISTS idx_ssb_files_seq  ON ssb_files(seq);
";

// ---------------------------------------------------------------------------
// Action type enum
// ---------------------------------------------------------------------------

/// All ~100 action types recorded in SSB entries.
/// B-19A added `HookLatencyViolation` (was 74 in B-10). The count is
/// approximate because per-batch additions land asynchronously; the
/// canonical inventory lives in [`SsbActionType::as_sql_str`] and the
/// CHECK constraint above. B-23 Phase 4H Group A is the latest tranche
/// of additions (6 self-mod beacons). B-39 adds 4 reflect variants
/// (reflect_triaged, reflect_candidate, reflect_audit_verdict, reflect_refined).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SsbActionType {
    FileWrite,
    FileRead,
    ToolResult,
    TestOutcome,
    Finding,
    Claim,
    Checkpoint,
    ModelSwap,
    PhaseTransition,
    KnowledgeDegradation,
    IsolationMetric,
    PlanUpdate,
    ReplanRequest,
    StubReference,
    ReasoningTrace,
    PhaseComplete,
    MemoryMonitoring,
    WorkloadProfile,
    DeathEvent,
    RecoveryEvent,
    SupervisionExhausted,
    SpiralDetected,
    ContextReduction,
    BranchMerge,
    MistakeEntry,
    ResearchStep,
    ResearchDecomposeLevel,
    WebResearchStep,
    PreviewAvailable,
    InterventionRequest,
    InterventionResponse,
    TaskSubmitted,
    /// Batch 1A: hook dispatcher verdict emission (T1/T2/T3 tiered analysis).
    /// Carries a [`HookEventPayload`]. Dashboard filters on the `hook_event`
    /// discriminant to render "hook · T1 BLOCK" style rows.
    HookEvent,
    /// Batch 1B: per-baby-step snapshot of sidecar extended signals
    /// (A-17 GG logprob/entropy + A-25 novelty + A-8 steering + A-9 ETC state).
    /// Carries a [`ProviderSignalsPayload`].
    ProviderSignals,
    /// Batch 1B: ETC search-trigger fired on the most recent sidecar
    /// completion (A-9). Carries a [`SearchTriggerFiredPayload`].
    SearchTriggerFired,
    /// Batch 1B: VRAM tier detected at task_runner startup (A-37). Emitted
    /// once per run. Carries a [`VramTierDetectedPayload`].
    VramTierDetected,
    /// Batch 1C (MVP audit 2026-04-18 section 4 item 6 / A-44): project
    /// indexing stage transition. Emitted by the task_runner's
    /// `maybe_build_project_index` wrapper at (Start, Complete, Failed)
    /// boundaries — intermediate `dkg_parsing` / `bm25_indexing` stages are
    /// reserved for the future incremental/streaming build. Carries an
    /// [`IndexingProgressPayload`].
    IndexingProgress,
    /// Batch 1C: one-shot DKG summary emitted once per project per runner
    /// lifetime, immediately after the first successful
    /// `IndexingService::run_full_index`. Carries a [`DkgReadyPayload`].
    DkgReady,
    /// Batch 2A: per-hook-invocation summary complementing the per-verdict
    /// `HookEvent` rows. Emitted once per T1/T2/T3 dispatch with timings and
    /// rule counts so the dashboard can surface "T1 · 3 rules / 0.4ms"
    /// without having to aggregate individual HookEvent rows. Carries a
    /// [`TieredAnalysisPayload`].
    TieredAnalysis,
    /// Batch 2A: one row per ShortCoder compression at the PostToolUse
    /// boundary. Records the byte count before/after and which rules fired
    /// so the dashboard can render "ShortCoder · 1200→320 tokens (73%)".
    /// Carries a [`ShortCoderCompressionPayload`].
    ShortCoderCompression,
    /// Batch 2B: per-dispatch summary for every `InProcessToolDispatcher`
    /// call (edit / validate / classify / git_commit / file_read / incremental).
    /// Carries a [`ToolDispatchPayload`].
    ToolDispatch,
    /// Batch 2B: per-tier validation verdict row emitted once per
    /// [`elai_core::ValidationTier`] execution. Carries a
    /// [`ValidationTierPayload`].
    ValidationTier,
    /// Batch 2B: failure taxonomy classification row emitted by
    /// `dispatch_classify_failure`. Carries a [`FailureClassifiedPayload`].
    FailureClassified,
    /// Batch 2B: git2-backed commit boundary. Carries a
    /// [`GitCommitPayload`].
    GitCommit,
    /// Batch 2B: DAIRA call-graph breadcrumb. Emitted on entry/exit of
    /// every dispatch method so the activity feed can render call graphs.
    /// Carries a [`DairaTracePayload`].
    DairaTrace,
    /// Batch 3A (Phase 4F): system probe snapshot. Emitted once per task
    /// runner lifetime and replayed on every dashboard subscribe. Carries
    /// a [`SystemProbePayload`].
    SystemProbe,
    /// Batch 3A (Phase 4F): per-concept / per-task feasibility verdict.
    /// Carries a [`FeasibilityReportPayload`].
    FeasibilityReport,
    /// Batch 3A (Phase 4F): one row per novel step detected by the
    /// feasibility prober. Carries a [`NovelStepDetectedPayload`].
    NovelStepDetected,
    /// Batch 3A (Phase 4F): one row per stub execution inside the
    /// feasibility probe. Carries a [`StubRunPayload`].
    StubRun,
    /// Batch 3B (Phase 4G): autonomous-gate decision row. Emitted on every
    /// call to [`crate::autonomous_gate::AutonomousGate::decide`]. Carries a
    /// [`GateDecisionPayload`].
    GateDecision,
    /// Batch 3B (Phase 4G): task-split announcement emitted when the
    /// autonomous gate decides to partition a task under context pressure.
    /// Carries a [`TaskSplitPayload`].
    TaskSplit,
    /// Batch 3B (Phase 4G): escalation-fired row. Carries an
    /// [`EscalationFiredPayload`] with the trigger + step_id.
    EscalationFired,
    /// Batch 3B (Phase 4G): gate-mode transition (Interactive ↔ Autonomous).
    /// Carries a [`ModeChangedPayload`].
    ModeChanged,
    /// Batch 3C (Phase 4H) — Layer 1A proposal generated. Carries a
    /// [`ProposalGeneratedPayload`].
    ProposalGenerated,
    /// Batch 3C (Phase 4H) — Layer 1A proposal applied. Carries a
    /// [`ProposalAppliedPayload`].
    ProposalApplied,
    /// Batch 3C (Phase 4H) — Layer 1A proposal rejected. Carries a
    /// [`ProposalRejectedPayload`].
    ProposalRejected,
    /// Batch 3C (Phase 4H) — per-apply MSI update. Carries a
    /// [`MsiUpdatePayload`].
    MsiUpdate,
    /// Batch 3C (Phase 4H) — archive capping event. Carries a
    /// [`ConfigArchivedPayload`].
    ConfigArchived,
    /// Batch 4C (A-18) — per-dispatch GenerationProfile selection. Carries a
    /// [`ProfileSelectedPayload`].
    ProfileSelected,
    /// Batch 4C (A-22) — FSM transition emitted on RESEARCHING entry/exit.
    /// Carries an [`FsmTransitionPayload`].
    FsmTransition,
    /// Batch 4C (A-23) — micro-reflection diagnosis on baby-step failure.
    /// Carries a [`MicroReflectionPayload`].
    MicroReflection,
    /// Batch 4C (A-46) — context telescope collapse event. Carries a
    /// [`TelescopeCollapsePayload`].
    TelescopeCollapse,
    /// Batch 4C (A-45) — config-load warning (e.g. source-tiers.toml missing).
    /// Carries a [`ConfigWarningPayload`].
    ConfigWarning,
    /// Batch-fu (FU.2 / A-7) — constitutional CONSTRAINTS block injected into
    /// the assembled PhaseSpec. Emitted once per dispatch that carried a
    /// non-empty `ProjectProfile.constraints` list. Carries a
    /// [`ConstraintsInjectedPayload`].
    ConstraintsInjected,
    /// Batch-fu (FU.5 / A-31) — resolution row emitted after an
    /// `InterventionRequest` has been answered by the dashboard via
    /// `respond_to_confirmation`. Carries an [`InterventionResolvedPayload`].
    InterventionResolved,
    /// Batch-ful-a (FUL-A.1) — live upgrade of the rule engine's DkgQuerier
    /// after the first successful `IndexingService::run_full_index`. Carries
    /// a [`DkgQuerierSwappedPayload`]. Emitted once per project per runner
    /// lifetime, immediately after the structured `IndexingProgress=complete`
    /// + `DkgReady` rows. Lets the dashboard activity feed render "DKG querier
    /// upgraded · empty → adapter" without polling the rule engine.
    DkgQuerierSwapped,
    /// Batch-ful-c (FUL-C / A-8) — freshness steering vector computed for a
    /// role. Emitted on successful `/v1/steering/compute` round-trip +
    /// cache-file write. Carries a [`SteeringComputedPayload`].
    SteeringComputed,
    /// Batch-ful-c (FUL-C / A-8) — freshness steering vector compute failed
    /// for a role (sidecar unreachable, HTTP error, validation rejected, or
    /// cache write failed). Carries a [`SteeringComputeFailedPayload`].
    SteeringComputeFailed,
    /// Batch-ful-d (FUL-D / phase-3J S9) — external MCP server connection
    /// established via the `DefaultMcpBridge`. Emitted once per successful
    /// `initialize` handshake + `tools/list` response. Carries a
    /// [`McpServerConnectedPayload`].
    McpServerConnected,
    /// Batch-ful-d (FUL-D / phase-3J S9) — per-call summary row emitted on
    /// every `tools/call` round-trip. Carries a [`McpToolCalledPayload`].
    McpToolCalled,
    /// Batch-ful-d (FUL-D / phase-3J S9) — MCP server disconnected, either
    /// by user toggle, heartbeat timeout, or peer-closed TCP. Carries a
    /// [`McpServerDisconnectedPayload`].
    McpServerDisconnected,
    /// Tier 1 CHAT window (dashboard-wiring-plan.txt §5) — user-submitted
    /// chat-surface message after client-side intent classification.
    /// Dashboard-originated row; no orchestrator consumer today (same
    /// deferral pattern as `TaskSubmitted`). Carries a [`ChatMessagePayload`].
    ChatMessage,
    /// Tier 1 CHAT window — system-emitted card response rendered in the
    /// chat message list (StatusCard / ErrorCard / DisambiguationCard /
    /// SteeringAckCard). Dashboard-originated. Carries a
    /// [`ChatResponsePayload`].
    ChatResponse,
    /// Tier 4 Settings reorg (dashboard-wiring-plan.txt §8) — execution-mode
    /// selection from the Settings window (LOCAL / CLOUD / HYBRID). Emitted
    /// by the dashboard `set_execution_mode` Tauri command. Carries a
    /// [`ExecutionModeChangedPayload`]. Dashboard-originated row; CLOUD /
    /// HYBRID dispatch paths are not yet implemented (§9 group 20) — the row
    /// is accepted but a follow-up `Finding` row is emitted to flag the gap.
    ExecutionModeChanged,
    /// B-10 (DECISION-C): per-dispatch ContextBudget snapshot. Emitted after
    /// `ContextBudget::compute_with_profile` succeeds at every
    /// `build_dispatch_context` callsite so the dashboard's per-slot bar
    /// chart can render real allocation values matching the active tier
    /// profile (e.g. 12.5/66.7/12.5/8.3 for 7B-12GB). Carries a
    /// [`ContextBudgetComputedPayload`]. Schema version 22.
    ContextBudgetComputed,
    /// B-10 / 3E.5 (DECISION-C): SSB entry filtering summary emitted when
    /// the SsbEntries slot truncation drops candidates below
    /// [`crate::context_budget::FILE_OVERLAP_THRESHOLD`]. Carries a
    /// [`SsbFilteredByOverlapPayload`] with `entries_total` / `entries_kept`
    /// / `overlap_threshold`. Schema version 22.
    SsbFilteredByOverlap,
    /// B-19A (T10) — emitted by the hook dispatcher whenever a T1/T2/T3
    /// hook overruns its tier latency budget
    /// (T1 <= 1ms, T2 <= 10ms, T3 <= 5ms). Carries a
    /// [`HookLatencyViolationPayload`]. Schema version 23.
    HookLatencyViolation,
    /// Track A.2 — research synthesis path selection. Emitted once per
    /// `phase_f_synthesize` call. Carries a [`ResearchPathChosenPayload`].
    ResearchPathChosen,
    /// Track A.6 — grounding cache hit or miss. Emitted by `ground_or_cached`
    /// on each call. Carries a [`GroundingCacheEventPayload`].
    GroundingCacheEvent,
    /// B-4: circuit-breaker state transition for embedding/reranker models.
    /// Emitted by CrossEncoderReranker + OrtVectorModule on CLOSED->OPEN or HALF_OPEN transitions.
    /// Dashboard get_embedding_breakers reads the latest row per `model` field.
    /// Carries a [`CircuitBreakerStateChangePayload`].
    CircuitBreakerStateChange,
    /// B-7: one row per BubbleUpEvent push in handle_failure() / bubble_up_on_*_integration_fail().
    /// Carries a [`BubbleUpTypedPayload`] with from/to level, action variant, and reason.
    BubbleUpTyped,
    /// B-7: emitted at all 4 DecompositionLevel checkpoints by IntegrationCheckpointEmitter.
    /// Carries a [`IntegrationCheckpointSuccessPayload`] with level, node_id, timing, success flag.
    IntegrationCheckpointSuccess,
    /// B-7: one row per RepairAction emitted by DagRepairLoop::repair() via DagRepairEmitter.
    /// Carries a [`DagRepairRoundPayload`] with action kind, round number, and reason string.
    DagRepairRound,
    /// B-7: one row per concept/phase/step/baby_step built inside research_decompose().
    /// Carries a [`ConceptDecompositionNodePayload`] with node_kind, node_id, parent_id, name.
    ConceptDecompositionNode,
    /// B-6: emitted after each EvidenceGate::check() call in the FSM transition path.
    /// Carries an [`EvidenceGateEvaluatedPayload`] with gate_id, verdict, and ts_ms.
    EvidenceGateEvaluated,
    /// B-6: emitted by the supervision layer on each restart or kill event.
    /// Carries a [`SupervisionEventPayload`] with agent_id, event kind, and attempt count.
    SupervisionEvent,
    /// B-6: one row per completed BSP superstep written by the BSP engine.
    /// Carries a [`BspSuperstepPayload`] with snapshot_id, ts_ms, and channels_count.
    BspSuperstep,
    /// B-6: agent lifecycle state change (spawned / ready / exited).
    /// Carries an [`AgentLifecyclePayload`] with agent_id, role, state, and optional death_class.
    AgentLifecycle,
    /// B-6: anti-spiral escalation signal — step hash repeated past threshold.
    /// Carries an [`AntiSpiralHitPayload`] with agent_id, step_hash, and count.
    AntiSpiralHit,
    /// B-27: per-call cloud-mode spend ledger row. Carries a [`CloudSpendPayload`]
    /// with provider, model, dual-rate token counts, and computed cost_usd.
    /// One row per AnthropicClient::send_message round-trip routed through
    /// cloud_mode.rs. Read by `get_cloud_spend` Tauri command for the
    /// MonitorWindow spend gauges (spec D-4:307).
    CloudSpend,
    /// B-27: one-shot marker emitted when cloud_mode.rs promotes a baby-step
    /// from local execution to cloud after the local model exhausts retries
    /// or trips an explicit escalation rule. Carries an [`EscalateToCloudPayload`].
    EscalateToCloud,
    /// B-23 Phase 4H Group A — Layer 1A row insert event. Carries a
    /// [`DataTableGrewPayload`]. Distinct from `ConfigArchived` (capping) and
    /// `ProposalApplied` (the action) — `DataTableGrew` records the actual
    /// table-row growth so the dashboard can render "deprecated_patterns +1
    /// (cargo, blake2->blake3)" without aggregating apply rows.
    DataTableGrew,
    /// B-23 Phase 4H — Layer 3 structural-change proposal emitted for HUMAN
    /// review only. Carries an [`L3ProposalPayload`]. Self-mod agent NEVER
    /// applies these; the dashboard surfaces them for an operator to approve.
    L3Proposal,
    /// B-23 Phase 4H 4H.7B (OSC) — emitted when ConfigVariantSimilarity over
    /// the last `stagnation_window` consecutive variants is >= 0.95 for that
    /// PipelineMode. Carries a [`ConvergenceStagnantPayload`].
    ConvergenceStagnant,
    /// B-23 Phase 4H 4H.7B (OSC) — period-2 oscillation detected
    /// (sim(N,N-2) AND sim(N-1,N-3) both >= 0.95). Carries a
    /// [`ConfigOscillationPayload`].
    ConfigOscillation,
    /// B-23 Phase 4H 4H.7C (REG) — cross-pipeline AC regression detected
    /// (AC failed in latest pipeline, passed in some prior pipeline).
    /// Carries a [`RegressionDetectedPayload`].
    RegressionDetected,
    /// B-23 Phase 4H 4H.7D (EVO) — evolution gate Withhold result emitted
    /// when the latest N variants never structurally evolved (similarity >=
    /// 0.95 across all consecutive pairs). Carries an
    /// [`EvolutionWithheldPayload`]. Forces MSI verdict to STABILIZING for
    /// the affected mode.
    EvolutionWithheld,
    // B-39 chat reflect pipeline: 4 new SsbActionType variants for per-turn REFLECT events.
    /// B-39: TRIAGE stage verdict. Carries a [`ReflectTriagedPayload`].
    ReflectTriaged,
    /// B-39: REASONER candidate emitted. Carries a [`ReflectCandidatePayload`].
    ReflectCandidate,
    /// B-39: AUDITOR best-of-N verdict. Carries a [`ReflectAuditVerdictPayload`].
    ReflectAuditVerdict,
    /// B-39: SYNTHESIZE final synthesis. Carries a [`ReflectRefinedPayload`].
    ReflectRefined,
    // B-24 Phase 4I Group C1: media validator beacons. Payloads land in
    // Group C4 with the dashboard surface; the enum surface is added now
    // so the schema bump is atomic and downstream readers (SsbViewer)
    // can already match on the variant.
    /// B-24: media validation started for a media artifact. Carries
    /// [`MediaValidationStartedPayload`] (Group C4).
    MediaValidationStarted,
    /// B-24: Tier-1 deterministic checks complete. Carries
    /// [`MediaTier1CompletePayload`] (Group C4).
    MediaTier1Complete,
    /// B-24: Tier-2 (VLM) advisory complete. Carries
    /// [`MediaTier2CompletePayload`] (Group C4).
    MediaTier2Complete,
    /// B-24: render adapter finished producing a captured frame set.
    /// Carries [`RenderCompletePayload`] (Group C4).
    RenderComplete,
    /// B-24: one iteration of the visual-feedback loop completed.
    /// Carries [`VisualFeedbackIterationPayload`] (Group C4).
    VisualFeedbackIteration,
    /// B-24: validator escalates to a human-review verdict. Carries
    /// [`MediaHumanReviewRequiredPayload`] (Group C4).
    MediaHumanReviewRequired,
    /// B-28 E-3 (SCHEMA_VERSION 31): focus-tunnel ModeSelector decision
    /// beacon. Emitted once per baby-step in `drive_task` immediately
    /// before the executor dispatch. Carries a
    /// [`GenerationModeSelectedPayload`] with the chosen
    /// `GenerationMode`, the `SelectionRule` that fired, and the active
    /// execution-mode tag — feeds the dashboard `mode_distribution`
    /// surface and the auditor's first-match-wins assertion.
    GenerationModeSelected,
    /// B-28 E-4 (SCHEMA_VERSION 32): focus-tunnel StreamingValidator
    /// early-termination beacon. Emitted by the post-E-7 WIRING phase
    /// whenever the synchronous library-tier validator returns
    /// `ValidationDecision::EarlyTerm`. Carries a
    /// [`StreamingEarlyTerminatedPayload`] with the wire-form
    /// `EarlyTermReason`, the token count at termination, and the active
    /// generation `mode` — feeds the dashboard's "early-term" surface and
    /// the auditor's reason-distribution check. The library validator
    /// itself does NOT emit the row; the wiring layer translates
    /// `EarlyTermWireForm` into this payload.
    StreamingEarlyTerminated,
    /// B-28 WIRING-1 (SCHEMA_VERSION 33): focus-tunnel PipelinePreAssembler
    /// readiness beacon. One row per `PipelinePreAssembler::on_step_outcome`
    /// + `prepare()` invocation. Carries a [`PipelinePreAssembledPayload`]
    /// with `pass_rate_bp` (basis points) + `was_discarded` + `depth_at_time`
    /// + `bytes_assembled` so the dashboard can render the parallelism gauge
    /// and waste-rate bar without aggregating across multiple rows.
    PipelinePreAssembled,
    /// B-28 WIRING-1 (SCHEMA_VERSION 34): focus-tunnel DependencyResolver
    /// decision beacon. Emitted once per `ResolutionDecision` after the
    /// post-execute import-scan + resolve cycle. Carries a
    /// [`DependencyResolvedPayload`] with the wire-form decision string
    /// (`auto_approve` / `queue_for_human` / `blocked`) and gating reason.
    DependencyResolved,
    /// B-28 WIRING-1 (SCHEMA_VERSION 35): focus-tunnel SpeculativeFileRouter
    /// micro-step beacon. Emitted once per `NeedsMicroStep` or
    /// `MicroStepCapExhausted` route decision. Carries a
    /// [`MicroStepCreatedPayload`] with the parent step id + created path +
    /// type_hint (Module / Struct / Function / Const) + reason string.
    MicroStepCreated,
    /// B-30 (SCHEMA_VERSION 36): one-shot LanguageRegistry status emitted at
    /// DKG engine startup, immediately after the per-grammar `catch_unwind`
    /// registration cycle completes. Carries a
    /// [`LanguageGrammarStatusPayload`] with the full loaded + failed lists.
    /// Producers: dkg index init (LanguageRegistry::new() + register() loop).
    /// Consumers: dashboard KnowledgeDrawer LANGUAGES LOADED card.
    LanguageGrammarStatus,
    /// B-29 Phase 5 (SCHEMA_VERSION 37): one row per symbol_* tool call.
    /// Carries a [`SymbolQueryPayload`] with the tool name, target, hop count,
    /// total files reached, and latency in ms.
    /// Producers: translate_tool_request() for symbol_impact / symbol_flow /
    /// symbol_context / symbol_list tool IDs.
    /// Consumers: dashboard KnowledgeDrawer Impact Analysis panel + ActivityFeed.
    SymbolQuery,
    /// B-32 (SCHEMA_VERSION 38): one row per minified file detected during
    /// indexing. Emitted by [`crate::indexing_service::IndexingService::build_index_resumable`]
    /// after the BM25 batch loop completes (post-DKG walkback over indexed
    /// candidates). Carries a [`MinifiedFileChunkedPayload`] with the
    /// relative path, byte count, chunk count, and computed average line
    /// length. Defense-in-depth alongside the bm25.rs `tracing::info!`
    /// emit — the SSB row persists for dashboard surfacing, the tracing
    /// log keeps dev-console visibility.
    MinifiedFileChunked,
    /// B-31.5 (SCHEMA_VERSION 39): per-batch checkpoint emitted by
    /// [`crate::indexing_service::IndexingService::build_index_resumable`].
    /// Carries an [`IndexCheckpointPayload`] with the project_id, current
    /// batch counters, file count, and last_checkpoint_at. Replaces the
    /// `tracing::info!(target="ssb_pending", ...)` placeholders from
    /// Cycle B.
    IndexCheckpoint,
    /// B-31.5 (SCHEMA_VERSION 39): emitted by `build_index_resumable` when it
    /// returns [`crate::indexing_service::IndexResult::Resumed`], or by
    /// [`crate::startup::auto_resume_indexed_projects`] when a resumed run
    /// completes. Carries an [`IndexResumePayload`] with the project_id,
    /// `resumed_from_batch`, and `files_skipped` counters.
    IndexResume,
    /// B-31.5 (SCHEMA_VERSION 39): emitted by `build_index_resumable` when it
    /// returns [`crate::indexing_service::IndexResult::Cancelled`], or by
    /// [`crate::startup::graceful_shutdown`] when it requests cancellation
    /// of an in-progress project. Carries an [`IndexCancelPayload`] with
    /// the project_id and partial-state counters.
    IndexCancel,
    /// B-31.5 (SCHEMA_VERSION 39): emitted by `IndexLock::try_acquire` (when
    /// invoked through the optional SSB-writer path) on successful
    /// acquisition. Carries a [`LockAcquiredPayload`] with the project_id,
    /// scope, and `holder_pid`.
    LockAcquired,
    /// B-31.5 (SCHEMA_VERSION 39): emitted by `IndexLock::try_acquire` (when
    /// invoked through the optional SSB-writer path) on
    /// [`crate::lock::IndexLockError::AlreadyHeld`]. Carries a
    /// [`LockContestedPayload`] mirroring the AlreadyHeld error payload —
    /// the contender pid is the current process; `holder_pid` is the live
    /// holder.
    LockContested,
    /// B-33 follow-up (SCHEMA_VERSION 40): one row per successful symbol-graph
    /// build emitted by [`crate::indexing_service::IndexingService::build_index_resumable`]
    /// after [`elai_knowledge::symbol_graph::build_pipeline::build_symbol_graph`]
    /// returns Ok with a non-empty file count. Carries a
    /// [`SymbolGraphUpdatedPayload`] with the project id, file/symbol/edge
    /// counts, and the build wall-clock millisecond timestamp.
    /// Producers: IndexingService post-DKG symbol-graph build branch.
    /// Consumers: dashboard-v2 GraphWindow event listener (replaces the
    /// 30s polling fallback shipped in B-33 D3).
    SymbolGraphUpdated,
    /// B-40 (SCHEMA_VERSION 41): emitted once when the FRAMING substate is
    /// entered (PLANNING + agent_substate=Framing). Carries an
    /// [`IdeaFramingStartedPayload`] with the raw user prompt + session
    /// timestamp. Producer: [`crate::idea_framing::frame_idea`].
    /// Consumer: dashboard-v2 IdeaFramingWindow.svelte initial render.
    IdeaFramingStarted,
    /// B-40 (SCHEMA_VERSION 41): emitted by
    /// [`crate::idea_framing::classify_project_class`] after the regex
    /// classifier returns a [`crate::idea_framing::ProjectClass`].
    /// Carries a [`ProjectClassClassifiedPayload`] with the snake_case class
    /// name + matched_pattern hint (None for `Other`).
    ProjectClassClassified,
    /// B-40 (SCHEMA_VERSION 41): emitted by
    /// [`crate::idea_framing::run_critic_loop`] at the end of each round
    /// (1..=3). Carries a [`CriticRoundCompletePayload`] with round_index
    /// + per-category finding counts + total BLOCK/WARN/INFO tallies +
    /// converged flag (true when zero BLOCK findings remain).
    CriticRoundComplete,
    /// B-40 (SCHEMA_VERSION 41): emitted by
    /// [`crate::idea_framing::frame_idea`] after persisting framed-idea.toml.
    /// Carries an [`IdeaFramedPayload`] with the artifact path + project_class
    /// + rounds_used + tradeoff_count. Consumed by the 7th AutoAdvanceEvaluator
    /// evidence check `derive_framing_evidence`.
    IdeaFramed,
    /// B-40 (SCHEMA_VERSION 41): emitted by
    /// [`crate::idea_framing::frame_idea`] when the TradeoffCard is ready
    /// for user response (surfaced to dashboard IdeaFramingWindow).
    /// Carries a [`TradeoffCardEmittedPayload`] with tradeoff names +
    /// option counts.
    TradeoffCardEmitted,
    /// B-34 (SCHEMA_VERSION 42): one row per `entry_points` tool call.
    /// Carries an [`EntryPointsDetectedPayload`] with the project_id, total
    /// count, and per-reason breakdown so the dashboard can render an
    /// entry-point summary card without re-running detection.
    EntryPointsDetected,
    /// B-34 (SCHEMA_VERSION 42): one row per ArtifactIndexer completion.
    /// Carries an [`ArtifactsIndexedPayload`] with indexed/up_to_date artifact
    /// names, total_chunks, and latency_ms. Produced by Phase C indexer wiring.
    ArtifactsIndexed,
    /// B-34 (SCHEMA_VERSION 42): one row per `search_artifacts` tool call.
    /// Carries an [`ArtifactSearchedPayload`] with the query, optional
    /// artifact_name filter, result_count, and latency_ms.
    ArtifactSearched,
    /// B-34 (SCHEMA_VERSION 42): one row per stale artifact detected by the
    /// ArtifactIndexer content-hash check. Carries an [`ArtifactStalePayload`]
    /// with artifact_name and the list of changed files.
    ArtifactStale,
    /// B-38 (SCHEMA_VERSION 43): one row per `AmbiguityScorer.score()`
    /// completion. Carries an [`AmbiguityScoredPayload`].
    AmbiguityScored,
    /// B-38 (SCHEMA_VERSION 43): one row per GroundingSweepGate verdict
    /// where one or more per-dimension floors were breached. Carries an
    /// [`AmbiguityFloorFailedPayload`].
    AmbiguityFloorFailed,
    /// B-38 (SCHEMA_VERSION 43): one row per qualifying-but-streak-incomplete
    /// gate verdict. Carries an [`AmbiguityStreakAdvancedPayload`].
    AmbiguityStreakAdvanced,
    /// B-42 (SCHEMA_VERSION 44): one row per cosmos refresh-loop tick start.
    /// Carries a [`CosmosRefreshStartedPayload`].
    CosmosRefreshStarted,
    /// B-42 (SCHEMA_VERSION 44): one row per successful cosmos artifact refresh.
    /// Carries a [`CosmosArtifactUpdatedPayload`].
    CosmosArtifactUpdated,
    /// B-42 (SCHEMA_VERSION 44): one row when a cosmos artifact exceeds
    /// 3 * cadence_interval since last refresh. Carries a
    /// [`CosmosArtifactDecayedPayload`].
    CosmosArtifactDecayed,
    /// B-42 (SCHEMA_VERSION 44): one row per `CosmosCheck` verifier dispatch.
    /// Carries a [`CosmosGroundingUsedPayload`].
    CosmosGroundingUsed,
    /// B-41 (SCHEMA_VERSION 45): per-level auditor mode spawn at a tier node.
    /// Carries a [`PerLevelAuditStartedPayload`].
    PerLevelAuditStarted,
    /// B-41 (SCHEMA_VERSION 45): per-level auditor verdict produced.
    /// Carries a [`PerLevelAuditVerdictPayload`].
    PerLevelAuditVerdict,
    /// B-41 (SCHEMA_VERSION 45): per-level auditor cascade (2nd BLOCK at a
    /// tier forces parent-tier respawn). Carries a [`PerLevelAuditCascadePayload`].
    PerLevelAuditCascade,
    /// B-37C (SCHEMA_VERSION 46): one row per runtime fallback chain step
    /// emitted by the `RuntimeOrchestrator` when a primary backend is
    /// unavailable / fails. Orthogonal to `ModelSwap` (which records model
    /// substitution within a backend). Carries a [`RuntimeFallbackPayload`].
    RuntimeFallback,
    /// B-37C (SCHEMA_VERSION 46): one row per per-backend `RuntimeHealth`
    /// transition observed by `RuntimeOrchestrator::health_snapshot`.
    /// Carries a [`RuntimeHealthChangedPayload`].
    RuntimeHealthChanged,
    /// B-37C (SCHEMA_VERSION 46): one row per CLI adapter subprocess spawn
    /// (B-37B cli_common::spawn). Carries a [`CliSubprocessSpawnedPayload`]
    /// with the redacted argv (secrets stripped).
    CliSubprocessSpawned,
    /// B-37C (SCHEMA_VERSION 46): one row per CLI adapter subprocess kill
    /// (B-37B cli_common::kill_and_wait, idle timeout, or natural exit).
    /// Carries a [`CliSubprocessKilledPayload`] with exit code + reason.
    CliSubprocessKilled,
    /// C-1 (SCHEMA_VERSION 47): one row per `emit_reasoning` call when a
    /// `PipelineGrc` handle was plumbed through `ResolvedConfig`. Replaces
    /// the synthetic-constant `ReasoningTrace` row with real
    /// `budget_remaining` (from `BasicGrcEngine::get_budget`) and
    /// `agent_id` (from the engine's `AgentGrcConfig.role`). Carries a
    /// [`GrcStepEmittedPayload`].
    GrcStepEmitted,
    /// C-1 (SCHEMA_VERSION 47): one row per `PipelineGrc::ground_plan`
    /// call in `task_runner::drive_task_inner`. Emitted ONCE per call
    /// (not per gate iteration) so the dashboard renders one sweep row
    /// per plan grounding pass. Carries a [`GroundingSweepResultPayload`].
    GroundingSweepResult,
    /// C-1 (SCHEMA_VERSION 47): one row per `PipelineGrc::generate_csq`
    /// completion in `task_runner::drive_task_inner`. Replaces the
    /// TODO(B-future) comment at the pre-tool CSQ branch. stub-ban-allow: historical reference, not a stub
    /// Carries a [`CsqEvaluatedPayload`].
    CsqEvaluated,
    /// C-1 (SCHEMA_VERSION 47): one row per `BasicGrcEngine::advance`
    /// returning `GrcError::InsufficientFindings` (auditor attempted to
    /// Converge with zero `FINDING:` rows). Surfaced at the `PipelineGrc`
    /// caller in `task_runner`, NOT inside `engine.rs`, because the
    /// engine has no SSB hook (GAP-2 in explorer-C1.md). Carries an
    /// [`AuditorRubberStampRejectedPayload`].
    AuditorRubberStampRejected,
    /// C-1 (SCHEMA_VERSION 47): one row per `HydeStage::hypothesize`
    /// call. `cache_hit=true` rows carry latency_ms of the cache lookup
    /// only; `cache_hit=false` rows carry total sidecar + cache-insert
    /// latency. Carries a [`HydeQueryPayload`].
    HydeQuery,
    /// C-1 (SCHEMA_VERSION 47): aggregate vector-cache hit/miss snapshot
    /// emitted by `VectorCache::flush_hit_rate_ssb` after a query
    /// pipeline run. Hits/misses counters reset atomically per flush.
    /// Carries a [`VectorCacheHitRatePayload`].
    VectorCacheHitRate,
    /// C-1 (SCHEMA_VERSION 47): per-query trace emitted at the end of
    /// `QueryPipeline::query_with_tier`. Pairs with the existing
    /// `tracing::debug!("query_trace")` event but persists into SSB so
    /// the dashboard recentTraces panel can read live rows. Carries a
    /// [`QueryTracePayload`].
    QueryTrace,
    /// C-2 (SCHEMA_VERSION 48): emitted by the task_runner baby-step loop
    /// the first time `nav.next_baby_step` returns a baby-step whose
    /// `tree.location_index[bs_id].phase_id` differs from the previously
    /// seen phase id. Carries a [`PhaseNodeReadyPayload`].
    PhaseNodeReady,
    /// C-2 (SCHEMA_VERSION 48): emitted by the task_runner baby-step loop
    /// when the resolved `tree.location_index[bs_id].step_id` differs from
    /// the previously seen step id. Carries a [`StepNodeReadyPayload`].
    StepNodeReady,
    /// C-2 (SCHEMA_VERSION 48): emitted once per baby-step consumption in
    /// the task_runner loop. Carries a [`BabyStepReadyPayload`].
    BabyStepReady,
    /// C-2 (SCHEMA_VERSION 48): emitted by `pipeline_builder` on the
    /// `RepairOutcome::Escalated` path when the DAG repair loop exhausts
    /// `T_DAG = 10` retries. Carries a [`DagRepairEscalatedPayload`].
    DagRepairEscalated,
    /// C-2 (SCHEMA_VERSION 48): mirror of [`Self::AgentLifecycle`] surfaced
    /// for the dashboard `AGENTS` panel. Single `state` field (no
    /// old_lifecycle); consumers derive transitions by reading the ordered
    /// stream per agent_id (DRIFT-5 in context-package-C2.md). Carries an
    /// [`AgentStatusChangedPayload`].
    AgentStatusChanged,
    /// C-2 (SCHEMA_VERSION 48): C-10 anti-spiral detector — spinning pattern
    /// (same step hash 3+ times). Variant defined now so the SCHEMA bump
    /// is atomic; producer wiring lands in C-10. Carries a
    /// [`StagnationSpinningDetectedPayload`].
    StagnationSpinningDetected,
    /// C-2 (SCHEMA_VERSION 48): C-10 anti-spiral detector — period-2
    /// oscillation pattern. Definition-only in C-2; producer wiring in
    /// C-10. Carries a [`StagnationOscillationDetectedPayload`].
    StagnationOscillationDetected,
    /// C-2 (SCHEMA_VERSION 48): C-10 persona switch — lateral thinking
    /// persona activated to break a stagnation loop. Definition-only in
    /// C-2; producer wiring in C-10. Carries a
    /// [`LateralThinkingActivatedPayload`].
    LateralThinkingActivated,
    /// C-2 (SCHEMA_VERSION 48): C-10 goal-drift canary verdict. Emitted
    /// when the canary detects that the active baby-step has drifted from
    /// the original task intent. Definition-only in C-2; producer wiring
    /// in C-10. Carries a [`GoalDriftWarningPayload`].
    GoalDriftWarning,
    /// C-2 (SCHEMA_VERSION 48): C-10 generation watchdog timeout. Emitted
    /// when an agent exceeds its per-step wall-clock budget without
    /// producing observable output. Definition-only in C-2; producer
    /// wiring in C-10. Carries a [`WatchdogTimeoutPayload`].
    WatchdogTimeout,
    /// C-7 (SCHEMA_VERSION 49): reviewer verdict on a proposed change. Emitted
    /// by the dashboard-v2 Tauri commands `accept_change` / `decline_change` /
    /// `retry_audit` each time a human reviewer acts on a change row in
    /// ReviewWindow. Carries a [`ChangeDecisionPayload`].
    ChangeDecision,
    /// C-8 (SCHEMA_VERSION 50): DegradationCascade verdict bridge.
    /// Emitted by the live_cli startup-readiness path AND by the
    /// dashboard-v2 Tauri command `set_search_mode`. Carries a
    /// [`SearchModeChangedPayload`]. The dashboard `get_search_mode`
    /// command reads the latest row of this type (graceful empty
    /// fallback to "hybrid"). Cross-process SSB bridge between the
    /// CLI AppState and the dashboard process (no shared memory).
    SearchModeChanged,
    /// C-5 (SCHEMA_VERSION 51): per-call privacy redaction event.
    /// Emitted by [`crate::cloud_mode::RedactingBabyStepExecutor`] each
    /// time the privacy redactor scrubs PII or secrets from a
    /// cloud-bound baby-step payload. Carries a
    /// [`RedactionAppliedPayload`] with `count` (number of substitutions
    /// applied across all redacted fields) and `fields` (the logical
    /// field names touched, e.g. `["task_summary", "escalation_message"]`).
    /// Consumer: dashboard CloudPanel.svelte privacy-event feed (last 5
    /// redactions). The row is best-effort: a write failure never aborts
    /// the baby-step execution.
    RedactionApplied,
    /// C-10 (SCHEMA_VERSION 52): lateral-thinking persona switch
    /// succeeded — emitted when a previously-active stagnation pattern
    /// no longer fires after a persona suggestion was consumed.
    /// Carries a [`LateralThinkingSucceededPayload`]. Producer wiring:
    /// task_runner step-boundary detector (anti_spiral_extended.rs).
    LateralThinkingSucceeded,
    /// C-10 (SCHEMA_VERSION 52): lateral-thinking exhausted — all 5
    /// personas have been tried within the current task window without
    /// a pattern break. Carries a [`LateralThinkingExhaustedPayload`].
    LateralThinkingExhausted,
    /// C-10 (SCHEMA_VERSION 52): goal-drift Block verdict — emitted
    /// when the GoalDriftCanary verdict crosses the Block threshold.
    /// Companion to [`Self::GoalDriftWarning`] (Yellow band). Carries a
    /// [`GoalDriftBlockPayload`].
    GoalDriftBlock,
    /// C-13 (SCHEMA_VERSION 53): WIP-commit squash completion beacon.
    /// Emitted by `crate::wip_squash::squash_wip_commits_if_any` at the
    /// post-AUDITING -> MEMORIZING boundary AND by the `elai wip-squash`
    /// CLI subcommand. Carries `wip_count` (leading-WIP-commit count
    /// observed) + `squashed` (true ⇒ `git reset --soft HEAD~N` + commit
    /// fired; false ⇒ no-op because count <= 5). Producer wiring:
    /// task_runner audit-loop boundary + cli/wip_squash_subcommand.rs.
    /// Consumer: dashboard activity feed.
    WipSquashComplete,
    /// C-22 (SCHEMA_VERSION 54): per-NovelStep alternative-explored row.
    /// Emitted by `DefaultFeasibilityProber::probe` once per NovelStep
    /// alternative attempted, carrying step_id + alternative_index.
    /// Consumer: dashboard FeasibilityPanel alternative trail.
    AlternativeExplored,
    /// C-22 (SCHEMA_VERSION 54): feasibility-probe escalation row.
    /// Emitted when `decide_with_mode` returns
    /// `FeasibilityDecision::Escalate`. Carries reason + variant tag.
    /// Distinct from the gate-level `escalation_fired` row.
    FeasibilityEscalated,
    /// C-23 (SCHEMA_VERSION 55): SessionDag branch fork beacon. Emitted
    /// at the orchestrator level whenever `SessionDag::create_branch`
    /// returns Ok via the new `create_branch_with_ssb` wrapper (mirror
    /// of `merge_branch_with_ssb`). Also emitted by the dashboard-v2
    /// `fork_session_branch` Tauri command which opens the DAG per call.
    /// Carries [`BranchForkedPayload`] (branch_name + tip_snap_id +
    /// parent_snap_id).
    BranchForked,
    /// C-23 (SCHEMA_VERSION 55): SessionDag branch merge beacon.
    /// Supersedes the legacy Checkpoint-with-discriminator emit at
    /// `merge_branch_with_ssb`. Carries [`BranchMergedPayload`]
    /// (winner_name + trunk_snap_id + abandoned_names).
    BranchMerged,
    /// C-23 (SCHEMA_VERSION 55): SessionDag branch close beacon. Emitted
    /// by the new `close_branch_with_ssb` wrapper. Carries
    /// [`BranchClosedPayload`] (branch_name + reason).
    BranchClosed,
    /// C-25 (SCHEMA_VERSION 56): plan-skeleton emission row. Emitted by
    /// `pipeline_builder::PipelineBuilder::build_with_emitter` ONCE per
    /// factory entry. Carries [`PlanSkeletonPayload`] (plan_id, step_count,
    /// concept_class, plan_skeleton_hash, source). C-25 is the FIRST emitter
    /// of this row (see C-25 DRIFT-D — no producer existed pre-bump).
    PlanSkeleton,
    /// C-25 (SCHEMA_VERSION 56): compiled-pipeline cache-hit + run beacon.
    /// Emitted by `compiled_pipeline::CompiledExecutor::try_execute` on
    /// successful pipeline-cache resolution and run. Carries
    /// [`CompiledPipelineFiredPayload`] (pipeline_id, concept_class,
    /// use_count, plan_skeleton_hash).
    CompiledPipelineFired,
    /// C-25 (SCHEMA_VERSION 56): template-crystallizer success beacon.
    /// Emitted by `compiled_pipeline::TemplateCrystallizer::check_crystallization`
    /// when the threshold (N=3 consecutive successes) is met and a new
    /// CompiledPipeline is crystallized. Carries
    /// [`TemplateCrystallizedPayload`] (concept_class, plan_skeleton_hash,
    /// pipeline_id, threshold_n).
    TemplateCrystallized,
    /// C-25 (SCHEMA_VERSION 56): background-task spawn beacon. Emitted by
    /// `background_task::BackgroundTaskManager::gated_spawn` on a
    /// successful (autonomous_mode=true) spawn. Carries
    /// [`BackgroundTaskStartedPayload`] (task_id, command_summary,
    /// soft_timeout_secs, hard_timeout_secs).
    BackgroundTaskStarted,
    /// C-25 (SCHEMA_VERSION 56): background-task completion beacon.
    /// Emitted by `background_task::BackgroundTaskManager::poll_completed`
    /// when a handle finishes (normal exit) AND by `enforce_timeouts`
    /// when a handle is auto-detached at the soft boundary. Carries
    /// [`BackgroundTaskCompletePayload`] (task_id, exit_status,
    /// duration_secs, auto_detached, timed_out).
    BackgroundTaskComplete,
    /// C-25 audit follow-up (SCHEMA_VERSION 57): compiled-pipeline
    /// decompile beacon. Emitted by
    /// `compiled_pipeline::CompiledExecutor::emit_decompile_triggered`
    /// whenever a cache HIT is rejected via the confidence floor or the
    /// consecutive-failure threshold. Carries
    /// [`DecompileTriggeredPayload`] (concept_class, plan_skeleton_hash,
    /// failure_count, trigger_reason). Closes the WARN-1 gap from
    /// audit-C25.md where decompile events were routed through
    /// `AntiSpiralHit` and dashboard filters keyed on `decompile_triggered`
    /// missed every decompile.
    DecompileTriggered,
    /// C-26 (SCHEMA_VERSION 58): NavigatorTree futility — same-error-class
    /// window saturates at `futility_same_error_threshold` (default 3).
    /// Carries [`FutilityDetectedPayload`].
    FutilityDetected,
    /// C-26 (SCHEMA_VERSION 58): NavigatorTree futility — output-similarity
    /// path. Triggered when GoalDriftCanary computes a Jaccard similarity
    /// > 0.90 between two consecutive baby-step outputs. Carries
    /// [`FutilityOutputStagnationPayload`] (includes the persona switch
    /// label per C-10 D4 affinity table).
    FutilityOutputStagnation,
    /// C-27 (SCHEMA_VERSION 59): Testing-trust T-0 — DkgReachabilityAuditor
    /// successfully verified that a produces_symbol declared in a
    /// WIRING_CONTRACT block is reachable from a configured
    /// [wiring.entrypoints]. Carries [`WiringEvidencePayload`].
    WiringEvidence,
    /// C-27 (SCHEMA_VERSION 59): Testing-trust T-0 — any of StubBanAuditor /
    /// DkgReachabilityAuditor / ObservableEffectRunner reported a violation.
    /// Carries [`WiringViolationPayload`] (violation_kind discriminates the
    /// three branches: StubPattern | UnreachableSymbol | BrokenObservable).
    WiringViolation,
    /// C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) — DriftComparator
    /// observed divergent ActionTypeSequence at index N between two
    /// nominally-equivalent runs (same seed, same hardware_class). Carries
    /// [`DeterminismDriftPayload`]. Identical sequences emit zero rows
    /// (stub_ban contract: zero false positives).
    DeterminismDrift,
    /// C-28 (SCHEMA_VERSION 60): Testing-trust T-1 mutation testing —
    /// `MutationRunner` produced a `MutantOutcome::Survived` for a generated
    /// mutant. Emitted once per surviving mutant by the Step 6
    /// `DeterministicAuditor` mutation check (runs only after Build /
    /// VersionGate / FileSizeCensus / LazyCode / Completeness all pass).
    /// Carries [`MutationSurvivedPayload`] (file, line, mutant_kind).
    /// Caught / Timeout / Skipped outcomes emit ZERO rows; the
    /// `mutation_score` gate is enforced via a separate
    /// `DeterministicFinding::Block` finding.
    MutationSurvived,
    /// C-29 (SCHEMA_VERSION 61): Testing-trust Tier T-2 — emitted when a
    /// `BabyStepSpec.intent_pairs` set crosses the
    /// `IntentConfig.min_pairs` threshold (default 3). The
    /// `IntentPairsGate` (PLANNING -> TESTING_FIRST) emits one row per
    /// confirmation; the autonomous gate auto-confirm branch emits with
    /// `confirmed_by="autonomous_gate"`. Carries
    /// [`IntentConfirmedPayload`].
    IntentConfirmed,
    /// C-29 (SCHEMA_VERSION 61): Testing-trust Tier T-3 — `MetamorphicTestRunner`
    /// observed a relation violation (one of identity / inverse /
    /// permutation_invariance). Emitted by `deterministic_auditor` Step 7
    /// (post-Mutation). Carries [`MrViolatedPayload`] with a non-empty
    /// `counterexample` (debug_assert + runtime sentinel fallback, see
    /// `metamorphic_testing::MrResult::failed` + DRIFT-C29-11). MrViolated
    /// is NOT a standalone Block — score aggregation deferred to T-4 per
    /// C-29 plan §13(c).
    MrViolated,
    /// C-30 (SCHEMA_VERSION 62): Surgical Mode — one row per successful
    /// `SurgicalDecomposer::decompose` return. Carries a
    /// [`SurgicalPlanCreatedPayload`].
    SurgicalPlanCreated,
    /// C-30 (SCHEMA_VERSION 62): Surgical Mode — one row per decompose call
    /// whose AffectedSet.total_loc_estimate exceeds 500. Carries a
    /// [`SurgicalLocWarningPayload`] (loc + threshold + mode).
    SurgicalLocWarning,
    /// C-30 (SCHEMA_VERSION 62): Oracle Mode — one row per
    /// `OracleAnswerCache::get` hit. Carries an [`OracleCacheHitPayload`]
    /// (query_hash_hex + source_tier + cached_age_secs).
    OracleCacheHit,
    /// C-30 (SCHEMA_VERSION 62): Oracle Mode — one row per
    /// `OracleAnswerCache::insert` (preceded by a cache miss). Carries an
    /// [`OracleCacheMissPayload`] (query_hash_hex + confidence).
    OracleCacheMiss,
    /// C-30 (SCHEMA_VERSION 62): one row per signal evaluated by the
    /// fresh-project detector inside the first-task gate. Carries a
    /// [`ProjectDetectionSignalPayload`] (signal_name + weight + fired).
    ProjectDetectionSignal,
    /// C-32 (SCHEMA_VERSION 63): one row per
    /// `SkeletonGenerator::generate_for_functions` Ok return. Carries a
    /// [`SkeletonGeneratedPayload`] (blank_count, language, file_path).
    SkeletonGenerated,
    /// C-32 (SCHEMA_VERSION 63): one row per
    /// `SkeletonGenerator::flush_to_disk` success. Carries a
    /// [`SkeletonFlushedPayload`] (path, completeness_ratio).
    SkeletonFlushed,
    /// C-32 (SCHEMA_VERSION 63): one row per `MiniCritic::check`. Carries a
    /// [`MiniCriticVerdictPayload`] (blank_id, verdict, reason, language).
    MiniCriticVerdict,
    /// C-32 (SCHEMA_VERSION 63): one row per
    /// `StructuralRetrySequencer` stage invocation. Carries a
    /// [`StructuralRetryStageAttemptPayload`] (blank_id, stage, attempt,
    /// outcome).
    StructuralRetryStageAttempt,
    /// C-32 (SCHEMA_VERSION 63): one row per
    /// `BridgeFileTemplate::write_bridge` success. Carries a
    /// [`BridgeFileGeneratedPayload`] (kind, path, module_count).
    BridgeFileGenerated,
    /// D-1 (SCHEMA_VERSION 64): one row per [`crate::archive_chain::ArchiveChain::append`]
    /// call. Carries an [`ArchiveEntryAddedPayload`] (template_id, content_hash,
    /// parent_hash, domain, pattern_kind, archive_status).
    ArchiveEntryAdded,
    /// D-1 (SCHEMA_VERSION 64): one row per template retirement transition.
    /// Carries an [`ArchiveEntryRetiredPayload`] (template_id, retired_at_ms,
    /// retirement_reason).
    ArchiveEntryRetired,
    /// D-1 (SCHEMA_VERSION 64): one row per BAI Thompson promotion (D-5
    /// producer lands later; wire surface added now for atomic schema bump).
    /// Carries a [`TemplatePromotedPayload`] (template_id, stage, pass_rate).
    TemplatePromoted,
    /// D-1 (SCHEMA_VERSION 64): one row per Candidate rejection. Carries a
    /// [`TemplateCandidateRejectedPayload`] (template_id, rejection_reason).
    TemplateCandidateRejected,
    /// E-5 (SCHEMA_VERSION 65): one row per Mem0-style extraction emitting
    /// a new currently-valid `memory_facts` row (the producer wrapper in
    /// the orchestrator's per-turn extraction hook emits this row right
    /// after `FactStore::insert` returns Ok). Carries a
    /// [`MemoryFactExtractedPayload`] (fact_id, fact_type, scope,
    /// trust_score, key).
    MemoryFactExtracted,
    /// E-5 (SCHEMA_VERSION 65): one row per `FactStore::supersede` Ok return —
    /// emitted after the bi-temporal close + new-row insert pair completes
    /// inside a single sqlite transaction. Carries a
    /// [`MemoryFactSupersededPayload`] (old_fact_id, new_fact_id, valid_to_ms).
    MemoryFactSuperseded,
    /// D-2 (SCHEMA_VERSION 66): one row per
    /// [`crate::ppm_client::PpmClient::score`] call that clears the
    /// configured `min_score_threshold`. Carries a [`PpmScoredPayload`]
    /// (candidate_id, verifier_id, mode, score_mean, score_var, num_steps,
    /// latency_ms, cache_hit).
    PpmScored,
    /// D-2 (SCHEMA_VERSION 66): one row per PpmClient::score call whose
    /// `score_mean` is below the configured threshold. Carries a
    /// [`PpmGatingFailedPayload`] (candidate_id, verifier_id, mode,
    /// score_mean, threshold, num_steps).
    PpmGatingFailed,
    /// E-1 (SCHEMA_VERSION 67): one row per
    /// `elai_reasoning::chat_memory_reflect::reflect()` Ok return —
    /// emitted by the orchestrator's chat-turn-committed hook right
    /// after the reflect pass completes. Carries a
    /// [`ChatReflectionCompletedPayload`] (note_id, note_preview,
    /// facts_extracted, decay_rows_examined, decay_rows_updated,
    /// core_block_bytes).
    ChatReflectionCompleted,
    /// E-1 (SCHEMA_VERSION 67): one row per SLEEP_COMPUTE substate
    /// entry / exit. Carries a [`MemoryEvolutionRunPayload`] (phase
    /// `"start"` | `"end"`, duration_ms, notes_touched).
    MemoryEvolutionRun,
    /// E-1 dashboard slice (SCHEMA_VERSION 68): one row per
    /// `elai_memory::NoteStore::insert` Ok return — emitted by the
    /// orchestrator's chat-turn-committed hook (producer wiring lives in
    /// task_runner.rs and is owned by another session). Carries a
    /// [`MemoryNoteInsertedPayload`] (note_id, scope, content_blake3,
    /// link_count).
    MemoryNoteInserted,
    /// E-1 dashboard slice (SCHEMA_VERSION 68): one row per A-MEM note
    /// evolution rewrite (PPR-walker-attached link/edit pass). Carries a
    /// [`MemoryNoteEvolvedPayload`] (note_id, old_content_blake3,
    /// new_content_blake3, links_added, links_removed).
    MemoryNoteEvolved,
    /// E-1 dashboard slice (SCHEMA_VERSION 68): one row per duplicate /
    /// contradiction pair the conflict adjudicator emits. Carries a
    /// [`MemoryConflictDetectedPayload`] (fact_id_a, fact_id_b,
    /// conflict_kind: `"duplicate"` | `"contradiction"`).
    MemoryConflictDetected,
    /// E-1 dashboard slice (SCHEMA_VERSION 68): one row per
    /// `CoreBlockStore::upsert` that replaces an existing row (vs first
    /// insert). Carries a [`CoreBlockReplacedPayload`] (user_id,
    /// old_blake3, new_blake3, bytes).
    CoreBlockReplaced,
    /// D-3 (SCHEMA_VERSION 69): one row per
    /// [`crate::agent_pool::run_consensus_pass`] Ok return whose verdict is
    /// `Pass`. Carries a [`VerifierConsensusPassPayload`] (candidate_id,
    /// template_id, verdict, verifier_count, pass_count, rule_id).
    VerifierConsensusPass,
    /// D-3 (SCHEMA_VERSION 69): one row per
    /// [`crate::agent_pool::run_consensus_pass`] Ok return whose verdict is
    /// `Fail` / `Inconclusive`. Carries a [`VerifierConsensusFailPayload`]
    /// (candidate_id, template_id, failing_verifier_ids, reason).
    VerifierConsensusFail,
    /// D-4 (SCHEMA_VERSION 70): one row per
    /// [`crate::tripwire::TripwireMonitor::observe`] firing. Carries a
    /// [`TripwireFiredPayload`] (event_id, tripwire_kind, candidate_id,
    /// observed_value, threshold, fired_at_ms).
    TripwireFired,
    /// D-4 (SCHEMA_VERSION 70): one row per
    /// [`crate::tripwire::auto_revert`] success. Carries a
    /// [`TemplateAutoRevertedPayload`] (template_id, parent_template_id,
    /// retirement_reason, fired_tripwire_id).
    TemplateAutoReverted,
    /// D-19 (SCHEMA_VERSION 71): one row per
    /// [`crate::benchmarks::prmbench_runner::PRMBenchRunner::run_all`] (or
    /// [`crate::benchmarks::prmbench_runner::PRMBenchRunner::run_category`])
    /// Ok return. Carries a [`PrmBenchRunCompletedPayload`] (total,
    /// accuracy, precision, recall, gating_verdict, fixture_blake3,
    /// category_count).
    PrmBenchRunCompleted,
    /// D-5 (SCHEMA_VERSION 72): one row per `bai::emit_bai_round` posterior
    /// update. Producer is [`crate::bai::emit_bai_round`]; the value carries
    /// the BAI arm state after the Beta(alpha, beta) update and a
    /// `was_promoted` flag mirroring whether the Thompson sample fired the
    /// promotion rule on this round. See workflow/reasoning/D-5-decisions.md.
    /// Carries a [`BaiArmUpdatedPayload`] (template_id, alpha, beta, n_pulls,
    /// mean_estimate, was_promoted).
    BaiArmUpdated,
    /// D-6 (SCHEMA_VERSION 73): one row per `held_out::insert_and_emit` Ok
    /// return — i.e. a fresh held-out task that passed the 9-gram
    /// contamination filter and was inserted into the store. Producer is
    /// [`crate::held_out::insert_and_emit`]. Carries a
    /// [`HeldOutTaskAddedPayload`] (task_id, source, content_blake3,
    /// ngram_count, ingested_ts_ms, eligibility_window_ms).
    HeldOutTaskAdded,
    /// D-6 (SCHEMA_VERSION 73): one row per `held_out::mark_used_and_emit`
    /// Ok return — i.e. a held-out task consumed by a BAI eval round.
    /// Producer is [`crate::held_out::mark_used_and_emit`] (called from
    /// `agent_pool::sample_held_out_task`). Carries a
    /// [`HeldOutTaskUsedPayload`] (task_id, consumer_template_id,
    /// used_ts_ms).
    HeldOutTaskUsed,
    /// D-7 (SCHEMA_VERSION 74): one row per `canary::emit_canary_run` Ok
    /// return — i.e. a completed cross-domain transfer canary report.
    /// Producer is [`crate::canary::emit_canary_run`], invoked by the BAI
    /// promotion gate. Carries a [`CanaryRunCompletedPayload`] (report_id,
    /// ts_ms, overall_transfer_score, decision_kind,
    /// dominant_fail_family).
    CanaryRunCompleted,
    /// D-7 (SCHEMA_VERSION 74): one row per per-family evaluator pass
    /// inside a canary run — emitted alongside [`Self::CanaryRunCompleted`]
    /// so the dashboard can render per-family deltas without parsing the
    /// run's `per_family` array. Carries a
    /// [`CanaryFamilyEvaluatedPayload`] (report_id, family, n_evaluated,
    /// pass_count, fail_count, transfer_score, baseline_score).
    CanaryFamilyEvaluated,
    /// D-9 — one row per `LockedKeysAuditor::audit_diff_ranged`
    /// invocation. Carries a [`LockedKeysAuditRunPayload`] (audit_id,
    /// ts_ms, n_audited, n_violations, registry_blake3,
    /// registry_path). The dashboard `LockedKeysPanel` joins these
    /// run rows to per-violation [`LockedKeysViolationDetectedPayload`]
    /// rows on `audit_id`. Producer:
    /// `crates/orchestrator/src/locked_keys_auditor.rs`. Grounding:
    /// arxiv:1908.04734 (Everitt current-RF principle).
    LockedKeysAuditRun,
    /// D-9 — one row per overlap between a staged edit and a
    /// LockedKeysRegistry entry. Carries a
    /// [`LockedKeysViolationDetectedPayload`] (audit_id, ts_ms,
    /// attempted_edit, attempted_lines, locked_key_path,
    /// locked_key_signature_blake3, reason). Producer:
    /// `crates/orchestrator/src/locked_keys_auditor.rs`.
    LockedKeysViolationDetected,
    /// E-46 (SCHEMA_VERSION 76) — dual-ledger atomic advancement. One row
    /// per `DualLedger::advance_task` call: TaskLedger.status +
    /// ProgressLedger.completion_pct/blockers updated together. Carries a
    /// [`TaskAdvancedPayload`]. Producer:
    /// `crates/orchestrator/src/dual_ledger.rs`. Grounding:
    /// arxiv:2412.21154 (Devin/SWE-bench task-ledger pattern).
    TaskAdvanced,
    /// E-46 (SCHEMA_VERSION 76) — AGENTLESS leaf-shortcut elected for a
    /// task (single-file change, ≤N LOC, no cross-crate, no async
    /// network). Carries an [`AgentlessShortcutTakenPayload`]. Producer:
    /// `crates/orchestrator/src/agentless_shortcut.rs`. Grounding:
    /// arxiv:2407.01489 (AGENTLESS, 32.00% SWE-bench Lite at $0.70/issue).
    AgentlessShortcutTaken,
    /// E-46 (SCHEMA_VERSION 76) — `GoalDriftCanary::fired()` evaluated
    /// true (drift > threshold). Carries a [`GoalDriftFiredPayload`].
    /// Producer: `crates/orchestrator/src/goal_drift_canary.rs`.
    /// Grounding: arxiv:2505.02709 (goal-drift PRM supervision).
    GoalDriftFired,
    /// D-10 (SCHEMA_VERSION 77) — one row per `agent_pool::route_claim`
    /// call. Carries a [`ClaimRoutedPayload`] (claim_id, claim_type,
    /// mode, ts_ms, ctx_summary_blake3). Producer:
    /// `crates/orchestrator/src/agent_pool.rs::route_claim`, which
    /// wraps the pure predicate
    /// `crates/orchestrator/src/grounding_mode.rs::select_grounding_mode`.
    /// Grounding: arxiv:1908.04734 (Everitt current-RF principle).
    ClaimRouted,
    /// D-21 (SCHEMA_VERSION 78) — one row per
    /// `crates/orchestrator/src/benchmarks/mode_routing_runner.rs::ModeRoutingBenchmark::run_all`
    /// Ok return. Carries a [`ModeRoutingBenchRunCompletedPayload`]
    /// (total, accuracy, f1_macro, gating_verdict, fixture_blake3,
    /// fixture_row_count). The runner gates pass iff macro F1 >= 0.85
    /// AND every per-mode F1 >= 0.70 (no class collapse). CI consumes
    /// the row's gating_verdict via the `elai bench mode-routing`
    /// subcommand exit code.
    ModeRoutingBenchRunCompleted,
    /// D-11 (SCHEMA_VERSION 79) — one row per
    /// `CheckerRegistry::run_all_applicable` result triple emitted by
    /// `crates/orchestrator/src/agent_pool.rs::dispatch_constructive_checkers`.
    /// Carries a [`CheckerResultEmittedPayload`] (checker_id,
    /// checker_kind, result_kind, claim_id, ts_ms, reason). Producer is
    /// the orchestrator (NOT the policy model). Grounding:
    /// arxiv:2310.04406 (constructive proof checking), arxiv:2211.12588
    /// (PoT, T1), arxiv:2402.01030 (CodeAct, T1).
    CheckerResultEmitted,
    /// D-13 (SCHEMA_VERSION 80) — one row per
    /// `agent_pool::run_adversarial_pass` adversary-emit step. Carries
    /// an [`AdversaryGeneratedPayload`] (counter_id, claim_id,
    /// severity, model_family). Producer:
    /// `crates/orchestrator/src/agent_pool.rs::run_adversarial_pass`.
    /// Grounding: arxiv:2403.07974 (T1) adversarial verification.
    AdversaryGenerated,
    /// D-13 (SCHEMA_VERSION 80) — one row per re-run verifier
    /// consensus that includes adversary counter-evidence. Carries a
    /// [`ConsensusReevaluatedWithCounterPayload`] (claim_id,
    /// original_verdict, post_counter_verdict). Producer:
    /// `crates/orchestrator/src/agent_pool.rs::run_adversarial_pass`.
    /// Grounding: arxiv:2504.16760 (T1) lightweight latent verifiers
    /// (heterogeneous-family verification).
    ConsensusReevaluatedWithCounter,
    /// D-12 (SCHEMA_VERSION 81) — one row per
    /// `agent_pool::run_operational_check` invocation. Carries a
    /// [`BabyStepDispatchedPayload`] (step_id, parent_task_id,
    /// checker_kinds_used, outcome, ts_ms). Producer:
    /// `crates/orchestrator/src/agent_pool.rs::run_operational_check`.
    /// Grounding: arxiv:2305.16291 Voyager (T1)
    /// skill-with-measurable-effect; arxiv:2407.01489 AGENTLESS (T1)
    /// single-step verification.
    BabyStepDispatched,
    /// D-16 (SCHEMA_VERSION 82) — one row per
    /// `conformal_calibration::compute_tau_and_emit` Ok return. Carries
    /// a [`ConformalTauComputedPayload`] (tau, epsilon, corpus_size,
    /// source, ts_ms). Producer:
    /// `crates/orchestrator/src/conformal_calibration.rs`. Grounding:
    /// arXiv:2503.01345 (UniCR) + Vovk et al. 2005 §2.2 quantile-based
    /// conformal predictor.
    ConformalTauComputed,
    /// D-16 (SCHEMA_VERSION 82) — one row per
    /// `conformal_calibration::decide_and_emit` call (and by extension
    /// every `agent_pool::dispatch_conformal_refusal` invocation).
    /// Carries a [`ConformalRefusalDecidedPayload`] (claim_id,
    /// claim_score, tau, decision, confidence, ts_ms). Producer:
    /// `crates/orchestrator/src/conformal_calibration.rs`. Grounding:
    /// arXiv:2503.01345 (UniCR) — formal Mode E refusal rule.
    ConformalRefusalDecided,
    /// D-15 (SCHEMA_VERSION 83) — one row per
    /// `novelty::emit_novelty_classified` call (and by extension every
    /// `agent_pool::dispatch_novelty` invocation). Carries a
    /// [`NoveltyClassifiedPayload`] (template_id, class,
    /// dkg_match_count, near_duplicate_threshold, ts_ms). Producer:
    /// `crates/orchestrator/src/novelty.rs`. Grounding:
    /// arXiv:2406.05761 (LLM novelty detection three-bucket taxonomy);
    /// docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
    /// §1 Mode C + §4.8.
    NoveltyClassified,
    /// D-18 (SCHEMA_VERSION 84) — one row per
    /// `observable_effect::verify_observable_effect` call. Carries a
    /// [`ObservableEffectVerifiedPayload`] (effect_id, trigger_kind,
    /// observation_kind, sandbox_tier, outcome, confound_flags,
    /// elapsed_us, ts_ms). Producer:
    /// `crates/orchestrator/src/observable_effect.rs`. Plan:
    /// docs/thefinalglance/D-plan/D-18_observable_effect_typed.md
    /// (orthogonality + sandbox-vs-production-labeling + remaining
    /// 3-variant pack from the plan deferred to a follow-up session).
    ObservableEffectVerified,
    /// D-17 (SCHEMA_VERSION 85) — one row per
    /// `refusal_sla::RefusalRateMonitor::compute_snapshot` call (and by
    /// extension every `agent_pool::dispatch_refusal_sla_check`
    /// invocation). Carries a [`RefusalRateSnapshotPayload`]
    /// (refusal_rate, window_total, abstain_count, health_band, ts_ms).
    /// Producer: `crates/orchestrator/src/refusal_sla.rs`. Grounding:
    /// arXiv:2503.01345 (UniCR honest refusal); docs/specs/guide/
    /// real-problem-solving-grounded-2026-05-19.md §4.6.
    RefusalRateSnapshot,
    /// D-17 (SCHEMA_VERSION 85) — one row per snapshot whose
    /// `HealthBand != Healthy`. Carries a [`RefusalSlaBreachPayload`]
    /// (rate, threshold, breach_kind, ts_ms). `breach_kind` is the
    /// snake-case `above` | `below` label. Producer:
    /// `crates/orchestrator/src/refusal_sla.rs::sla_breach` emission
    /// path.
    RefusalSlaBreach,
    /// D-22 (SCHEMA_VERSION 86) — one row per
    /// [`crate::benchmarks::conformal_validation_runner::ConformalValidationRunner::run_all`]
    /// Ok return. Carries a [`ConformalValidationCompletedPayload`]
    /// (tau, epsilon, empirical_refusal_rate, gating_verdict,
    /// fixture_blake3, total, ts_ms). Gates the UniCR distribution-free
    /// guarantee (arXiv:2503.01345, T1) on a held-out validation fixture
    /// split into in_domain / near_ood / far_ood buckets. Producer:
    /// `crates/orchestrator/src/benchmarks/conformal_validation_runner.rs`.
    /// Plan: docs/thefinalglance/D-plan/D-22_conformal_calibration_validation.md.
    ConformalValidationCompleted,
    /// D-20 (SCHEMA_VERSION 87) — one row per
    /// [`crate::benchmarks::arc_agi2_runner::ARCAGI2Runner::run_all`]
    /// Ok return. Carries an [`ARCBenchmarkCompletedPayload`] (total,
    /// honestly_evaluated_count, gating_verdict, fixture_blake3,
    /// dominant_mode, dominant_novelty_class, ts_ms). Gates ELAI's
    /// mode-routing + novelty-classifier honesty stack against
    /// ARC-AGI-2-style out-of-distribution claims. Producer:
    /// `crates/orchestrator/src/benchmarks/arc_agi2_runner.rs`. Plan:
    /// docs/thefinalglance/D-plan/D-20_arc_agi2_ood_probe.md.
    ArcBenchmarkCompleted,
}

impl SsbActionType {
    /// SQL-compatible string for the CHECK constraint.
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::FileWrite => "file_write",
            Self::FileRead => "file_read",
            Self::ToolResult => "tool_result",
            Self::TestOutcome => "test_outcome",
            Self::Finding => "finding",
            Self::Claim => "claim",
            Self::Checkpoint => "checkpoint",
            Self::ModelSwap => "model_swap",
            Self::PhaseTransition => "phase_transition",
            Self::KnowledgeDegradation => "knowledge_degradation",
            Self::IsolationMetric => "isolation_metric",
            Self::PlanUpdate => "plan_update",
            Self::ReplanRequest => "replan_request",
            Self::StubReference => "stub_reference",
            Self::ReasoningTrace => "reasoning_trace",
            Self::PhaseComplete => "phase_complete",
            Self::MemoryMonitoring => "memory_monitoring",
            Self::WorkloadProfile => "workload_profile",
            Self::DeathEvent => "death_event",
            Self::RecoveryEvent => "recovery_event",
            Self::SupervisionExhausted => "supervision_exhausted",
            Self::SpiralDetected => "spiral_detected",
            Self::ContextReduction => "context_reduction",
            Self::BranchMerge => "branch_merge",
            Self::MistakeEntry => "mistake_entry",
            Self::ResearchStep => "research_step",
            Self::ResearchDecomposeLevel => "research_decompose_level",
            Self::WebResearchStep => "web_research_step",
            Self::PreviewAvailable => "preview_available",
            Self::InterventionRequest => "intervention_request",
            Self::InterventionResponse => "intervention_response",
            Self::TaskSubmitted => "task_submitted",
            Self::HookEvent => "hook_event",
            Self::ProviderSignals => "provider_signals",
            Self::SearchTriggerFired => "search_trigger_fired",
            Self::VramTierDetected => "vram_tier_detected",
            Self::IndexingProgress => "indexing_progress",
            Self::DkgReady => "dkg_ready",
            Self::TieredAnalysis => "tiered_analysis",
            Self::ShortCoderCompression => "shortcoder_compression",
            Self::ToolDispatch => "tool_dispatch",
            Self::ValidationTier => "validation_tier",
            Self::FailureClassified => "failure_classified",
            Self::GitCommit => "git_commit",
            Self::DairaTrace => "daira_trace",
            Self::SystemProbe => "system_probe",
            Self::FeasibilityReport => "feasibility_report",
            Self::NovelStepDetected => "novel_step_detected",
            Self::StubRun => "stub_run",
            Self::GateDecision => "gate_decision",
            Self::TaskSplit => "task_split",
            Self::EscalationFired => "escalation_fired",
            Self::ModeChanged => "mode_changed",
            Self::ProposalGenerated => "proposal_generated",
            Self::ProposalApplied => "proposal_applied",
            Self::ProposalRejected => "proposal_rejected",
            Self::MsiUpdate => "msi_update",
            Self::ConfigArchived => "config_archived",
            Self::ProfileSelected => "profile_selected",
            Self::FsmTransition => "fsm_transition",
            Self::MicroReflection => "micro_reflection",
            Self::TelescopeCollapse => "telescope_collapse",
            Self::ConfigWarning => "config_warning",
            Self::ConstraintsInjected => "constraints_injected",
            Self::InterventionResolved => "intervention_resolved",
            Self::DkgQuerierSwapped => "dkg_querier_swapped",
            Self::SteeringComputed => "steering_computed",
            Self::SteeringComputeFailed => "steering_compute_failed",
            Self::McpServerConnected => "mcp_server_connected",
            Self::McpToolCalled => "mcp_tool_called",
            Self::McpServerDisconnected => "mcp_server_disconnected",
            Self::ChatMessage => "chat_message",
            Self::ChatResponse => "chat_response",
            Self::ExecutionModeChanged => "execution_mode_changed",
            Self::ContextBudgetComputed => "context_budget_computed",
            Self::SsbFilteredByOverlap => "ssb_filtered_by_overlap",
            Self::HookLatencyViolation => "hook_latency_violation",
            Self::ResearchPathChosen => "research_path_chosen",
            Self::GroundingCacheEvent => "grounding_cache_event",
            Self::CircuitBreakerStateChange => "circuit_breaker_state_change",
            Self::BubbleUpTyped => "bubble_up_typed",
            Self::IntegrationCheckpointSuccess => "integration_checkpoint_success",
            Self::DagRepairRound => "dag_repair_round",
            Self::ConceptDecompositionNode => "concept_decomposition_node",
            // B-6: orchestration producer wiring.
            Self::EvidenceGateEvaluated => "evidence_gate_evaluated",
            Self::SupervisionEvent => "supervision_event",
            Self::BspSuperstep => "bsp_superstep",
            Self::AgentLifecycle => "agent_lifecycle",
            Self::AntiSpiralHit => "anti_spiral_hit",
            Self::CloudSpend => "cloud_spend",
            Self::EscalateToCloud => "escalate_to_cloud",
            // B-23 Phase 4H Group A.
            Self::DataTableGrew => "data_table_grew",
            Self::L3Proposal => "l3_proposal",
            Self::ConvergenceStagnant => "convergence_stagnant",
            Self::ConfigOscillation => "config_oscillation",
            Self::RegressionDetected => "regression_detected",
            Self::EvolutionWithheld => "evolution_withheld",
            // B-39 chat reflect pipeline.
            Self::ReflectTriaged => "reflect_triaged",
            Self::ReflectCandidate => "reflect_candidate",
            Self::ReflectAuditVerdict => "reflect_audit_verdict",
            Self::ReflectRefined => "reflect_refined",
            // B-24 Phase 4I Group C1: media validator beacons.
            Self::MediaValidationStarted => "media_validation_started",
            Self::MediaTier1Complete => "media_tier1_complete",
            Self::MediaTier2Complete => "media_tier2_complete",
            Self::RenderComplete => "render_complete",
            Self::VisualFeedbackIteration => "visual_feedback_iteration",
            Self::MediaHumanReviewRequired => "media_human_review_required",
            // B-28 E-3 (SCHEMA_VERSION 31): focus-tunnel ModeSelector beacon.
            Self::GenerationModeSelected => "generation_mode_selected",
            // B-28 E-4 (SCHEMA_VERSION 32): focus-tunnel streaming early-term beacon.
            Self::StreamingEarlyTerminated => "streaming_early_terminated",
            // B-28 WIRING-1 (SCHEMA_VERSION 33): pre-assembly readiness beacon.
            Self::PipelinePreAssembled => "pipeline_pre_assembled",
            // B-28 WIRING-1 (SCHEMA_VERSION 34): dependency-resolver decision beacon.
            Self::DependencyResolved => "dependency_resolved",
            // B-28 WIRING-1 (SCHEMA_VERSION 35): micro-step creation beacon.
            Self::MicroStepCreated => "micro_step_created",
            // B-30 (SCHEMA_VERSION 36): language grammar registry status.
            Self::LanguageGrammarStatus => "language_grammar_status",
            // B-29 Phase 5 (SCHEMA_VERSION 37): symbol-graph tool call row.
            Self::SymbolQuery => "symbol_query",
            // B-32 (SCHEMA_VERSION 38): minified-file chunking SSB row.
            Self::MinifiedFileChunked => "minified_file_chunked",
            // B-31.5 (SCHEMA_VERSION 39): 5 deferred B-31 SSB variants.
            Self::IndexCheckpoint => "index_checkpoint",
            Self::IndexResume => "index_resume",
            Self::IndexCancel => "index_cancel",
            Self::LockAcquired => "lock_acquired",
            Self::LockContested => "lock_contested",
            // B-33 follow-up (SCHEMA_VERSION 40): symbol-graph build event row.
            Self::SymbolGraphUpdated => "symbol_graph_updated",
            // B-40 (SCHEMA_VERSION 41): FRAMING substate beacons.
            Self::IdeaFramingStarted => "idea_framing_started",
            Self::ProjectClassClassified => "project_class_classified",
            Self::CriticRoundComplete => "critic_round_complete",
            Self::IdeaFramed => "idea_framed",
            Self::TradeoffCardEmitted => "tradeoff_card_emitted",
            // B-34 (SCHEMA_VERSION 42)
            Self::EntryPointsDetected => "entry_points_detected",
            Self::ArtifactsIndexed => "artifacts_indexed",
            Self::ArtifactSearched => "artifact_searched",
            Self::ArtifactStale => "artifact_stale",
            // B-38 (SCHEMA_VERSION 43): ambiguity gate beacons.
            Self::AmbiguityScored => "ambiguity_scored",
            Self::AmbiguityFloorFailed => "ambiguity_floor_failed",
            Self::AmbiguityStreakAdvanced => "ambiguity_streak_advanced",
            // B-42 (SCHEMA_VERSION 44): cosmos tier knowledge worker beacons.
            Self::CosmosRefreshStarted => "cosmos_refresh_started",
            Self::CosmosArtifactUpdated => "cosmos_artifact_updated",
            Self::CosmosArtifactDecayed => "cosmos_artifact_decayed",
            Self::CosmosGroundingUsed => "cosmos_grounding_used",
            // B-41 (SCHEMA_VERSION 45): per-level auditor mode beacons.
            Self::PerLevelAuditStarted => "per_level_audit_started",
            Self::PerLevelAuditVerdict => "per_level_audit_verdict",
            Self::PerLevelAuditCascade => "per_level_audit_cascade",
            // B-37C (SCHEMA_VERSION 46): multi-runtime backend beacons.
            Self::RuntimeFallback => "runtime_fallback",
            Self::RuntimeHealthChanged => "runtime_health_changed",
            Self::CliSubprocessSpawned => "cli_subprocess_spawned",
            Self::CliSubprocessKilled => "cli_subprocess_killed",
            // C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
            Self::GrcStepEmitted => "grc_step_emitted",
            Self::GroundingSweepResult => "grounding_sweep_result",
            Self::CsqEvaluated => "csq_evaluated",
            Self::AuditorRubberStampRejected => "auditor_rubber_stamp_rejected",
            Self::HydeQuery => "hyde_query",
            Self::VectorCacheHitRate => "vector_cache_hit_rate",
            Self::QueryTrace => "query_trace",
            // C-2 (SCHEMA_VERSION 48): orchestration + cosmos + stagnation beacons.
            Self::PhaseNodeReady => "phase_node_ready",
            Self::StepNodeReady => "step_node_ready",
            Self::BabyStepReady => "baby_step_ready",
            Self::DagRepairEscalated => "dag_repair_escalated",
            Self::AgentStatusChanged => "agent_status_changed",
            Self::StagnationSpinningDetected => "stagnation_spinning_detected",
            Self::StagnationOscillationDetected => "stagnation_oscillation_detected",
            Self::LateralThinkingActivated => "lateral_thinking_activated",
            Self::GoalDriftWarning => "goal_drift_warning",
            Self::WatchdogTimeout => "watchdog_timeout",
            // C-7 (SCHEMA_VERSION 49): reviewer verdict beacon.
            Self::ChangeDecision => "change_decision",
            // C-8 (SCHEMA_VERSION 50): search-mode cross-process bridge.
            Self::SearchModeChanged => "search_mode_changed",
            // C-5 (SCHEMA_VERSION 51): privacy-redaction event.
            Self::RedactionApplied => "redaction_applied",
            // C-10 (SCHEMA_VERSION 52): anti-spiral lateral-thinking +
            // goal-drift Block beacons.
            Self::LateralThinkingSucceeded => "lateral_thinking_succeeded",
            Self::LateralThinkingExhausted => "lateral_thinking_exhausted",
            Self::GoalDriftBlock => "goal_drift_block",
            // C-13 (SCHEMA_VERSION 53): IPC-dispatcher production-swap
            // beacon — WIP commit squash completion.
            Self::WipSquashComplete => "wip_squash_complete",
            // C-22 (SCHEMA_VERSION 54): feasibility-probe completeness beacons.
            Self::AlternativeExplored => "alternative_explored",
            Self::FeasibilityEscalated => "feasibility_escalated",
            // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
            Self::BranchForked => "branch_forked",
            Self::BranchMerged => "branch_merged",
            Self::BranchClosed => "branch_closed",
            // C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
            // + background-task beacons.
            Self::PlanSkeleton => "plan_skeleton",
            Self::CompiledPipelineFired => "compiled_pipeline_fired",
            Self::TemplateCrystallized => "template_crystallized",
            Self::BackgroundTaskStarted => "background_task_started",
            Self::BackgroundTaskComplete => "background_task_complete",
            Self::DecompileTriggered => "decompile_triggered",
            // C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
            Self::FutilityDetected => "futility_detected",
            Self::FutilityOutputStagnation => "futility_output_stagnation",
            // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons.
            Self::WiringEvidence => "wiring_evidence",
            Self::WiringViolation => "wiring_violation",
            Self::DeterminismDrift => "determinism_drift",
            // C-28 (SCHEMA_VERSION 60): Testing-trust T-1 mutation testing.
            Self::MutationSurvived => "mutation_survived",
            // C-29 (SCHEMA_VERSION 61): Testing-trust T-2 + T-3 beacons.
            Self::IntentConfirmed => "intent_confirmed",
            Self::MrViolated => "mr_violated",
            // C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode beacons.
            Self::SurgicalPlanCreated => "surgical_plan_created",
            Self::SurgicalLocWarning => "surgical_loc_warning",
            Self::OracleCacheHit => "oracle_cache_hit",
            Self::OracleCacheMiss => "oracle_cache_miss",
            Self::ProjectDetectionSignal => "project_detection_signal",
            // C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 beacons.
            Self::SkeletonGenerated => "skeleton_generated",
            Self::SkeletonFlushed => "skeleton_flushed",
            Self::MiniCriticVerdict => "mini_critic_verdict",
            Self::StructuralRetryStageAttempt => "structural_retry_stage_attempt",
            Self::BridgeFileGenerated => "bridge_file_generated",
            // D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle.
            Self::ArchiveEntryAdded => "archive_entry_added",
            Self::ArchiveEntryRetired => "archive_entry_retired",
            Self::TemplatePromoted => "template_promoted",
            Self::TemplateCandidateRejected => "template_candidate_rejected",
            // E-5 (SCHEMA_VERSION 65): memory_facts bi-temporal lifecycle beacons.
            Self::MemoryFactExtracted => "memory_fact_extracted",
            Self::MemoryFactSuperseded => "memory_fact_superseded",
            // D-2 (SCHEMA_VERSION 66): PPM verifier beacons.
            Self::PpmScored => "ppm_scored",
            Self::PpmGatingFailed => "ppm_gating_failed",
            // E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
            Self::ChatReflectionCompleted => "chat_reflection_completed",
            Self::MemoryEvolutionRun => "memory_evolution_run",
            // E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
            // producer + conflict beacons.
            Self::MemoryNoteInserted => "memory_note_inserted",
            Self::MemoryNoteEvolved => "memory_note_evolved",
            Self::MemoryConflictDetected => "memory_conflict_detected",
            Self::CoreBlockReplaced => "core_block_replaced",
            // D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator.
            Self::VerifierConsensusPass => "verifier_consensus_pass",
            Self::VerifierConsensusFail => "verifier_consensus_fail",
            // D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
            Self::TripwireFired => "tripwire_fired",
            Self::TemplateAutoReverted => "template_auto_reverted",
            // D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark beacon.
            Self::PrmBenchRunCompleted => "prmbench_run_completed",
            // D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
            Self::BaiArmUpdated => "bai_arm_updated",
            // D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
            Self::HeldOutTaskAdded => "held_out_task_added",
            Self::HeldOutTaskUsed => "held_out_task_used",
            // D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
            Self::CanaryRunCompleted => "canary_run_completed",
            Self::CanaryFamilyEvaluated => "canary_family_evaluated",
            Self::LockedKeysAuditRun => "locked_keys_audit_run",
            Self::LockedKeysViolationDetected => "locked_keys_violation_detected",
            // E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
            Self::TaskAdvanced => "task_advanced",
            Self::AgentlessShortcutTaken => "agentless_shortcut_taken",
            Self::GoalDriftFired => "goal_drift_fired",
            // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
            Self::ClaimRouted => "claim_routed",
            // D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
            Self::ModeRoutingBenchRunCompleted => "mode_routing_bench_run_completed",
            // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
            Self::CheckerResultEmitted => "checker_result_emitted",
            // D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
            Self::AdversaryGenerated => "adversary_generated",
            Self::ConsensusReevaluatedWithCounter => "consensus_reevaluated_with_counter",
            // D-12 (SCHEMA_VERSION 81): OperationalizationCheck beacon.
            Self::BabyStepDispatched => "baby_step_dispatched",
            // D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal beacons.
            Self::ConformalTauComputed => "conformal_tau_computed",
            Self::ConformalRefusalDecided => "conformal_refusal_decided",
            // D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass beacon.
            Self::NoveltyClassified => "novelty_classified",
            // D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate beacon.
            Self::ObservableEffectVerified => "observable_effect_verified",
            // D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
            Self::RefusalRateSnapshot => "refusal_rate_snapshot",
            Self::RefusalSlaBreach => "refusal_sla_breach",
            // D-22 (SCHEMA_VERSION 86): conformal-calibration validation beacon.
            Self::ConformalValidationCompleted => "conformal_validation_completed",
            // D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
            Self::ArcBenchmarkCompleted => "arc_benchmark_completed",
        }
    }
}

/// Public wrapper around the internal string-to-enum map.
/// Used by read-side consumers (e.g. [`crate::ssb_viewer::SsbViewer`]).
pub fn ssb_action_type_from_str_public(s: &str) -> Result<SsbActionType, SsbError> {
    ssb_action_type_from_str(s)
}

fn ssb_action_type_from_str(s: &str) -> Result<SsbActionType, SsbError> {
    match s {
        "file_write" => Ok(SsbActionType::FileWrite),
        "file_read" => Ok(SsbActionType::FileRead),
        "tool_result" => Ok(SsbActionType::ToolResult),
        "test_outcome" => Ok(SsbActionType::TestOutcome),
        "finding" => Ok(SsbActionType::Finding),
        "claim" => Ok(SsbActionType::Claim),
        "checkpoint" => Ok(SsbActionType::Checkpoint),
        "model_swap" => Ok(SsbActionType::ModelSwap),
        "phase_transition" => Ok(SsbActionType::PhaseTransition),
        "knowledge_degradation" => Ok(SsbActionType::KnowledgeDegradation),
        "isolation_metric" => Ok(SsbActionType::IsolationMetric),
        "plan_update" => Ok(SsbActionType::PlanUpdate),
        "replan_request" => Ok(SsbActionType::ReplanRequest),
        "stub_reference" => Ok(SsbActionType::StubReference),
        "reasoning_trace" => Ok(SsbActionType::ReasoningTrace),
        "phase_complete" => Ok(SsbActionType::PhaseComplete),
        "memory_monitoring" => Ok(SsbActionType::MemoryMonitoring),
        "workload_profile" => Ok(SsbActionType::WorkloadProfile),
        "death_event" => Ok(SsbActionType::DeathEvent),
        "recovery_event" => Ok(SsbActionType::RecoveryEvent),
        "supervision_exhausted" => Ok(SsbActionType::SupervisionExhausted),
        "spiral_detected" => Ok(SsbActionType::SpiralDetected),
        "context_reduction" => Ok(SsbActionType::ContextReduction),
        "branch_merge" => Ok(SsbActionType::BranchMerge),
        "mistake_entry" => Ok(SsbActionType::MistakeEntry),
        "research_step" => Ok(SsbActionType::ResearchStep),
        "research_decompose_level" => Ok(SsbActionType::ResearchDecomposeLevel),
        "web_research_step" => Ok(SsbActionType::WebResearchStep),
        "preview_available" => Ok(SsbActionType::PreviewAvailable),
        "intervention_request" => Ok(SsbActionType::InterventionRequest),
        "intervention_response" => Ok(SsbActionType::InterventionResponse),
        "task_submitted" => Ok(SsbActionType::TaskSubmitted),
        "hook_event" => Ok(SsbActionType::HookEvent),
        "provider_signals" => Ok(SsbActionType::ProviderSignals),
        "search_trigger_fired" => Ok(SsbActionType::SearchTriggerFired),
        "vram_tier_detected" => Ok(SsbActionType::VramTierDetected),
        "indexing_progress" => Ok(SsbActionType::IndexingProgress),
        "dkg_ready" => Ok(SsbActionType::DkgReady),
        "tiered_analysis" => Ok(SsbActionType::TieredAnalysis),
        "shortcoder_compression" => Ok(SsbActionType::ShortCoderCompression),
        "tool_dispatch" => Ok(SsbActionType::ToolDispatch),
        "validation_tier" => Ok(SsbActionType::ValidationTier),
        "failure_classified" => Ok(SsbActionType::FailureClassified),
        "git_commit" => Ok(SsbActionType::GitCommit),
        "daira_trace" => Ok(SsbActionType::DairaTrace),
        "system_probe" => Ok(SsbActionType::SystemProbe),
        "feasibility_report" => Ok(SsbActionType::FeasibilityReport),
        "novel_step_detected" => Ok(SsbActionType::NovelStepDetected),
        "stub_run" => Ok(SsbActionType::StubRun),
        "gate_decision" => Ok(SsbActionType::GateDecision),
        "task_split" => Ok(SsbActionType::TaskSplit),
        "escalation_fired" => Ok(SsbActionType::EscalationFired),
        "mode_changed" => Ok(SsbActionType::ModeChanged),
        "proposal_generated" => Ok(SsbActionType::ProposalGenerated),
        "proposal_applied" => Ok(SsbActionType::ProposalApplied),
        "proposal_rejected" => Ok(SsbActionType::ProposalRejected),
        "msi_update" => Ok(SsbActionType::MsiUpdate),
        "config_archived" => Ok(SsbActionType::ConfigArchived),
        "profile_selected" => Ok(SsbActionType::ProfileSelected),
        "fsm_transition" => Ok(SsbActionType::FsmTransition),
        "micro_reflection" => Ok(SsbActionType::MicroReflection),
        "telescope_collapse" => Ok(SsbActionType::TelescopeCollapse),
        "config_warning" => Ok(SsbActionType::ConfigWarning),
        "constraints_injected" => Ok(SsbActionType::ConstraintsInjected),
        "intervention_resolved" => Ok(SsbActionType::InterventionResolved),
        "dkg_querier_swapped" => Ok(SsbActionType::DkgQuerierSwapped),
        "steering_computed" => Ok(SsbActionType::SteeringComputed),
        "steering_compute_failed" => Ok(SsbActionType::SteeringComputeFailed),
        "mcp_server_connected" => Ok(SsbActionType::McpServerConnected),
        "mcp_tool_called" => Ok(SsbActionType::McpToolCalled),
        "mcp_server_disconnected" => Ok(SsbActionType::McpServerDisconnected),
        "chat_message" => Ok(SsbActionType::ChatMessage),
        "chat_response" => Ok(SsbActionType::ChatResponse),
        "execution_mode_changed" => Ok(SsbActionType::ExecutionModeChanged),
        "context_budget_computed" => Ok(SsbActionType::ContextBudgetComputed),
        "ssb_filtered_by_overlap" => Ok(SsbActionType::SsbFilteredByOverlap),
        "hook_latency_violation" => Ok(SsbActionType::HookLatencyViolation),
        "research_path_chosen" => Ok(SsbActionType::ResearchPathChosen),
        "grounding_cache_event" => Ok(SsbActionType::GroundingCacheEvent),
        // B-4 (v26 bump completed).
        "circuit_breaker_state_change" => Ok(SsbActionType::CircuitBreakerStateChange),
        // B-7 (v26 bump completed).
        "bubble_up_typed" => Ok(SsbActionType::BubbleUpTyped),
        "integration_checkpoint_success" => Ok(SsbActionType::IntegrationCheckpointSuccess),
        "dag_repair_round" => Ok(SsbActionType::DagRepairRound),
        "concept_decomposition_node" => Ok(SsbActionType::ConceptDecompositionNode),
        // B-6: orchestration producer wiring.
        "evidence_gate_evaluated" => Ok(SsbActionType::EvidenceGateEvaluated),
        "supervision_event" => Ok(SsbActionType::SupervisionEvent),
        "bsp_superstep" => Ok(SsbActionType::BspSuperstep),
        "agent_lifecycle" => Ok(SsbActionType::AgentLifecycle),
        "anti_spiral_hit" => Ok(SsbActionType::AntiSpiralHit),
        // B-27 cloud-mode action types.
        "cloud_spend" => Ok(SsbActionType::CloudSpend),
        "escalate_to_cloud" => Ok(SsbActionType::EscalateToCloud),
        // B-23 Phase 4H Group A self-mod action types.
        "data_table_grew" => Ok(SsbActionType::DataTableGrew),
        "l3_proposal" => Ok(SsbActionType::L3Proposal),
        "convergence_stagnant" => Ok(SsbActionType::ConvergenceStagnant),
        "config_oscillation" => Ok(SsbActionType::ConfigOscillation),
        "regression_detected" => Ok(SsbActionType::RegressionDetected),
        "evolution_withheld" => Ok(SsbActionType::EvolutionWithheld),
        // B-39 chat reflect pipeline.
        "reflect_triaged" => Ok(SsbActionType::ReflectTriaged),
        "reflect_candidate" => Ok(SsbActionType::ReflectCandidate),
        "reflect_audit_verdict" => Ok(SsbActionType::ReflectAuditVerdict),
        "reflect_refined" => Ok(SsbActionType::ReflectRefined),
        // B-24 Phase 4I Group C1: media validator beacons.
        "media_validation_started" => Ok(SsbActionType::MediaValidationStarted),
        "media_tier1_complete" => Ok(SsbActionType::MediaTier1Complete),
        "media_tier2_complete" => Ok(SsbActionType::MediaTier2Complete),
        "render_complete" => Ok(SsbActionType::RenderComplete),
        "visual_feedback_iteration" => Ok(SsbActionType::VisualFeedbackIteration),
        "media_human_review_required" => Ok(SsbActionType::MediaHumanReviewRequired),
        // B-28 E-3 (SCHEMA_VERSION 31).
        "generation_mode_selected" => Ok(SsbActionType::GenerationModeSelected),
        // B-28 E-4 (SCHEMA_VERSION 32).
        "streaming_early_terminated" => Ok(SsbActionType::StreamingEarlyTerminated),
        // B-28 WIRING-1 (SCHEMA_VERSION 33 -> 35).
        "pipeline_pre_assembled" => Ok(SsbActionType::PipelinePreAssembled),
        "dependency_resolved" => Ok(SsbActionType::DependencyResolved),
        "micro_step_created" => Ok(SsbActionType::MicroStepCreated),
        // B-30 (SCHEMA_VERSION 36): language grammar registry status.
        "language_grammar_status" => Ok(SsbActionType::LanguageGrammarStatus),
        // B-29 Phase 5 (SCHEMA_VERSION 37): symbol-graph tool call row.
        "symbol_query" => Ok(SsbActionType::SymbolQuery),
        // B-32 (SCHEMA_VERSION 38): minified-file chunking SSB row.
        "minified_file_chunked" => Ok(SsbActionType::MinifiedFileChunked),
        // B-31.5 (SCHEMA_VERSION 39): 5 deferred B-31 SSB variants.
        "index_checkpoint" => Ok(SsbActionType::IndexCheckpoint),
        "index_resume" => Ok(SsbActionType::IndexResume),
        "index_cancel" => Ok(SsbActionType::IndexCancel),
        "lock_acquired" => Ok(SsbActionType::LockAcquired),
        "lock_contested" => Ok(SsbActionType::LockContested),
        // B-33 follow-up (SCHEMA_VERSION 40): symbol-graph build event row.
        "symbol_graph_updated" => Ok(SsbActionType::SymbolGraphUpdated),
        // B-40 (SCHEMA_VERSION 41): FRAMING substate beacons.
        "idea_framing_started" => Ok(SsbActionType::IdeaFramingStarted),
        "project_class_classified" => Ok(SsbActionType::ProjectClassClassified),
        "critic_round_complete" => Ok(SsbActionType::CriticRoundComplete),
        "idea_framed" => Ok(SsbActionType::IdeaFramed),
        "tradeoff_card_emitted" => Ok(SsbActionType::TradeoffCardEmitted),
        // B-34 (SCHEMA_VERSION 42): entry-point detection + artifact pipeline beacons.
        "entry_points_detected" => Ok(SsbActionType::EntryPointsDetected),
        "artifacts_indexed"     => Ok(SsbActionType::ArtifactsIndexed),
        "artifact_searched"     => Ok(SsbActionType::ArtifactSearched),
        "artifact_stale"        => Ok(SsbActionType::ArtifactStale),
        // B-38 (SCHEMA_VERSION 43): ambiguity gate beacons.
        "ambiguity_scored"            => Ok(SsbActionType::AmbiguityScored),
        "ambiguity_floor_failed"      => Ok(SsbActionType::AmbiguityFloorFailed),
        "ambiguity_streak_advanced"   => Ok(SsbActionType::AmbiguityStreakAdvanced),
        // B-42 (SCHEMA_VERSION 44): cosmos tier knowledge worker beacons.
        "cosmos_refresh_started"  => Ok(SsbActionType::CosmosRefreshStarted),
        "cosmos_artifact_updated" => Ok(SsbActionType::CosmosArtifactUpdated),
        "cosmos_artifact_decayed" => Ok(SsbActionType::CosmosArtifactDecayed),
        "cosmos_grounding_used"   => Ok(SsbActionType::CosmosGroundingUsed),
        // B-41 (SCHEMA_VERSION 45): per-level auditor mode beacons.
        "per_level_audit_started" => Ok(SsbActionType::PerLevelAuditStarted),
        "per_level_audit_verdict" => Ok(SsbActionType::PerLevelAuditVerdict),
        "per_level_audit_cascade" => Ok(SsbActionType::PerLevelAuditCascade),
        // B-37C (SCHEMA_VERSION 46): multi-runtime backend beacons.
        "runtime_fallback"        => Ok(SsbActionType::RuntimeFallback),
        "runtime_health_changed"  => Ok(SsbActionType::RuntimeHealthChanged),
        "cli_subprocess_spawned"  => Ok(SsbActionType::CliSubprocessSpawned),
        "cli_subprocess_killed"   => Ok(SsbActionType::CliSubprocessKilled),
        // C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
        "grc_step_emitted"              => Ok(SsbActionType::GrcStepEmitted),
        "grounding_sweep_result"        => Ok(SsbActionType::GroundingSweepResult),
        "csq_evaluated"                 => Ok(SsbActionType::CsqEvaluated),
        "auditor_rubber_stamp_rejected" => Ok(SsbActionType::AuditorRubberStampRejected),
        "hyde_query"                    => Ok(SsbActionType::HydeQuery),
        "vector_cache_hit_rate"         => Ok(SsbActionType::VectorCacheHitRate),
        "query_trace"                   => Ok(SsbActionType::QueryTrace),
        // C-2 (SCHEMA_VERSION 48): orchestration + cosmos + stagnation beacons.
        "phase_node_ready"               => Ok(SsbActionType::PhaseNodeReady),
        "step_node_ready"                => Ok(SsbActionType::StepNodeReady),
        "baby_step_ready"                => Ok(SsbActionType::BabyStepReady),
        "dag_repair_escalated"           => Ok(SsbActionType::DagRepairEscalated),
        "agent_status_changed"           => Ok(SsbActionType::AgentStatusChanged),
        "stagnation_spinning_detected"   => Ok(SsbActionType::StagnationSpinningDetected),
        "stagnation_oscillation_detected"=> Ok(SsbActionType::StagnationOscillationDetected),
        "lateral_thinking_activated"     => Ok(SsbActionType::LateralThinkingActivated),
        "goal_drift_warning"             => Ok(SsbActionType::GoalDriftWarning),
        "watchdog_timeout"               => Ok(SsbActionType::WatchdogTimeout),
        // C-7 (SCHEMA_VERSION 49): reviewer verdict beacon.
        "change_decision"                => Ok(SsbActionType::ChangeDecision),
        // C-8 (SCHEMA_VERSION 50): search-mode cross-process bridge.
        "search_mode_changed"            => Ok(SsbActionType::SearchModeChanged),
        // C-5 (SCHEMA_VERSION 51): privacy-redaction event.
        "redaction_applied"              => Ok(SsbActionType::RedactionApplied),
        // C-10 (SCHEMA_VERSION 52): anti-spiral lateral-thinking +
        // goal-drift Block beacons.
        "lateral_thinking_succeeded"     => Ok(SsbActionType::LateralThinkingSucceeded),
        "lateral_thinking_exhausted"     => Ok(SsbActionType::LateralThinkingExhausted),
        "goal_drift_block"               => Ok(SsbActionType::GoalDriftBlock),
        // C-13 (SCHEMA_VERSION 53): IPC-dispatcher production-swap beacon.
        "wip_squash_complete"            => Ok(SsbActionType::WipSquashComplete),
        // C-22 (SCHEMA_VERSION 54): feasibility-probe completeness beacons.
        "alternative_explored"           => Ok(SsbActionType::AlternativeExplored),
        "feasibility_escalated"          => Ok(SsbActionType::FeasibilityEscalated),
        // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
        "branch_forked"                  => Ok(SsbActionType::BranchForked),
        "branch_merged"                  => Ok(SsbActionType::BranchMerged),
        "branch_closed"                  => Ok(SsbActionType::BranchClosed),
        // C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
        // + background-task beacons.
        "plan_skeleton"                  => Ok(SsbActionType::PlanSkeleton),
        "compiled_pipeline_fired"        => Ok(SsbActionType::CompiledPipelineFired),
        "template_crystallized"          => Ok(SsbActionType::TemplateCrystallized),
        "background_task_started"        => Ok(SsbActionType::BackgroundTaskStarted),
        "background_task_complete"       => Ok(SsbActionType::BackgroundTaskComplete),
        "decompile_triggered"            => Ok(SsbActionType::DecompileTriggered),
        // C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
        "futility_detected"              => Ok(SsbActionType::FutilityDetected),
        "futility_output_stagnation"     => Ok(SsbActionType::FutilityOutputStagnation),
        // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons.
        "wiring_evidence"                => Ok(SsbActionType::WiringEvidence),
        "wiring_violation"               => Ok(SsbActionType::WiringViolation),
        "determinism_drift"              => Ok(SsbActionType::DeterminismDrift),
        // C-28 (SCHEMA_VERSION 60): Testing-trust T-1 mutation testing.
        "mutation_survived"              => Ok(SsbActionType::MutationSurvived),
        // C-29 (SCHEMA_VERSION 61): Testing-trust T-2 + T-3 beacons.
        "intent_confirmed"               => Ok(SsbActionType::IntentConfirmed),
        "mr_violated"                    => Ok(SsbActionType::MrViolated),
        // C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode beacons.
        "surgical_plan_created"          => Ok(SsbActionType::SurgicalPlanCreated),
        "surgical_loc_warning"           => Ok(SsbActionType::SurgicalLocWarning),
        "oracle_cache_hit"               => Ok(SsbActionType::OracleCacheHit),
        "oracle_cache_miss"              => Ok(SsbActionType::OracleCacheMiss),
        "project_detection_signal"       => Ok(SsbActionType::ProjectDetectionSignal),
        // C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 beacons.
        "skeleton_generated"             => Ok(SsbActionType::SkeletonGenerated),
        "skeleton_flushed"               => Ok(SsbActionType::SkeletonFlushed),
        "mini_critic_verdict"            => Ok(SsbActionType::MiniCriticVerdict),
        "structural_retry_stage_attempt" => Ok(SsbActionType::StructuralRetryStageAttempt),
        "bridge_file_generated"          => Ok(SsbActionType::BridgeFileGenerated),
        // D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle.
        "archive_entry_added"            => Ok(SsbActionType::ArchiveEntryAdded),
        "archive_entry_retired"          => Ok(SsbActionType::ArchiveEntryRetired),
        "template_promoted"              => Ok(SsbActionType::TemplatePromoted),
        "template_candidate_rejected"    => Ok(SsbActionType::TemplateCandidateRejected),
        // E-5 (SCHEMA_VERSION 65): memory_facts bi-temporal lifecycle beacons.
        "memory_fact_extracted"          => Ok(SsbActionType::MemoryFactExtracted),
        "memory_fact_superseded"         => Ok(SsbActionType::MemoryFactSuperseded),
        // D-2 (SCHEMA_VERSION 66): PPM verifier beacons.
        "ppm_scored"                     => Ok(SsbActionType::PpmScored),
        "ppm_gating_failed"              => Ok(SsbActionType::PpmGatingFailed),
        // E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
        "chat_reflection_completed"      => Ok(SsbActionType::ChatReflectionCompleted),
        "memory_evolution_run"           => Ok(SsbActionType::MemoryEvolutionRun),
        // E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
        // producer + conflict beacons.
        "memory_note_inserted"           => Ok(SsbActionType::MemoryNoteInserted),
        "memory_note_evolved"            => Ok(SsbActionType::MemoryNoteEvolved),
        "memory_conflict_detected"       => Ok(SsbActionType::MemoryConflictDetected),
        "core_block_replaced"            => Ok(SsbActionType::CoreBlockReplaced),
        // D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator.
        "verifier_consensus_pass"        => Ok(SsbActionType::VerifierConsensusPass),
        "verifier_consensus_fail"        => Ok(SsbActionType::VerifierConsensusFail),
        // D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
        "tripwire_fired"                 => Ok(SsbActionType::TripwireFired),
        "template_auto_reverted"         => Ok(SsbActionType::TemplateAutoReverted),
        // D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark beacon.
        "prmbench_run_completed"         => Ok(SsbActionType::PrmBenchRunCompleted),
        // D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
        "bai_arm_updated"                => Ok(SsbActionType::BaiArmUpdated),
        // D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
        "held_out_task_added"            => Ok(SsbActionType::HeldOutTaskAdded),
        "held_out_task_used"             => Ok(SsbActionType::HeldOutTaskUsed),
        // D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
        "canary_run_completed"           => Ok(SsbActionType::CanaryRunCompleted),
        "canary_family_evaluated"        => Ok(SsbActionType::CanaryFamilyEvaluated),
        "locked_keys_audit_run"          => Ok(SsbActionType::LockedKeysAuditRun),
        "locked_keys_violation_detected" => Ok(SsbActionType::LockedKeysViolationDetected),
        // E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
        "task_advanced"                  => Ok(SsbActionType::TaskAdvanced),
        "agentless_shortcut_taken"       => Ok(SsbActionType::AgentlessShortcutTaken),
        "goal_drift_fired"               => Ok(SsbActionType::GoalDriftFired),
        // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
        "claim_routed"                   => Ok(SsbActionType::ClaimRouted),
        // D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
        "mode_routing_bench_run_completed" => Ok(SsbActionType::ModeRoutingBenchRunCompleted),
        // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
        "checker_result_emitted"         => Ok(SsbActionType::CheckerResultEmitted),
        // D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
        "adversary_generated"            => Ok(SsbActionType::AdversaryGenerated),
        "consensus_reevaluated_with_counter" => {
            Ok(SsbActionType::ConsensusReevaluatedWithCounter)
        }
        // D-12 (SCHEMA_VERSION 81): OperationalizationCheck beacon.
        "baby_step_dispatched" => Ok(SsbActionType::BabyStepDispatched),
        // D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal beacons.
        "conformal_tau_computed" => Ok(SsbActionType::ConformalTauComputed),
        "conformal_refusal_decided" => Ok(SsbActionType::ConformalRefusalDecided),
        // D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass beacon.
        "novelty_classified" => Ok(SsbActionType::NoveltyClassified),
        // D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate beacon.
        "observable_effect_verified" => Ok(SsbActionType::ObservableEffectVerified),
        // D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
        "refusal_rate_snapshot" => Ok(SsbActionType::RefusalRateSnapshot),
        "refusal_sla_breach" => Ok(SsbActionType::RefusalSlaBreach),
        // D-22 (SCHEMA_VERSION 86): conformal-calibration validation beacon.
        "conformal_validation_completed" => Ok(SsbActionType::ConformalValidationCompleted),
        // D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
        "arc_benchmark_completed" => Ok(SsbActionType::ArcBenchmarkCompleted),
        other => Err(SsbError::UnknownActionType(other.to_owned())),
    }
}

/// Validates that the deserialized payload matches the declared action_type.
/// This prevents silent corruption where the action_type string doesn't match
/// the actual payload struct type.
fn validate_payload_matches_action_type(
    payload_bytes: &[u8],
    declared_action_type: SsbActionType,
) -> Result<(), SsbError> {
    // Deserialize the payload to determine its actual type
    let payload: SsbPayload = rmp_serde::from_slice(payload_bytes)?;
    let actual_action_type = payload.action_type();

    if actual_action_type != declared_action_type {
        return Err(SsbError::ActionTypeMismatch {
            declared: declared_action_type.as_sql_str().to_owned(),
            actual: actual_action_type.as_sql_str().to_owned(),
        });
    }

    Ok(())
}

/// Computes cognitive isolation metric for a phase (orchestrator.txt S7A).
/// Formula: sum(agent_grc_tokens_this_phase) / model.max_context
///
/// This quantifies context that would have polluted the orchestrator if agents
/// were in-process. High values (>1.0) indicate the task is only feasible due
/// to agent isolation.
///
/// # Arguments
/// * `agent_grc_tokens` - Total GRC tokens generated by all agents in the phase
/// * `model_max_context` - Maximum context window of the model (e.g., 131072 for Gemma4 E2B)
///
/// # Returns
/// Ratio of context savings. Values >1.0 mean isolation saved more than one full context window.
pub fn compute_isolation_metric(agent_grc_tokens: u64, model_max_context: u64) -> f64 {
    if model_max_context == 0 {
        return 0.0;
    }
    agent_grc_tokens as f64 / model_max_context as f64
}

// ---------------------------------------------------------------------------
// Typed payload structs (3B.4)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWritePayload {
    pub path: String,
    pub size_bytes: u64,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileReadPayload {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResultPayload {
    pub tool_id: String,
    pub success: bool,
    pub output_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestOutcomePayload {
    pub test_name: String,
    pub passed: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingPayload {
    pub severity: String,
    pub category: String,
    pub message: String,
    pub file_path: Option<String>,
    pub line: Option<u32>,
    /// Batch 2B: taxonomy label attached by `dispatch_classify_failure`.
    /// Backward-compatible: unset → behaves exactly like pre-2B findings.
    /// The dashboard reads this via the new `FailureClassified` SSB row —
    /// this field is an in-Finding convenience so a single row carries
    /// both the pipeline emit-site AND the classification when we have it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_category: Option<String>,
    /// Batch 4A (A-41): which deterministic audit step produced this Finding.
    /// One of `"build" | "version_gate" | "file_size" | "lazy_code" | "completeness"`.
    /// `None` for findings emitted from non-audit sources (backward compatible).
    /// The dashboard labels audit findings by step when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_step: Option<String>,
    /// Batch 4A (A-36): DKG few-shot exemplars attached to a Finding.
    /// Populated by Tier 3 PI when DKG similarity yields k<=3 nodes.
    /// `None` when DKG is empty or disabled (backward compatible).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub few_shot_examples: Option<Vec<FewShotExample>>,
}

/// Batch 4A (A-36): a single few-shot exemplar pulled from DKG similarity.
///
/// Each exemplar is a short in-project reference the dashboard can render under
/// the Finding as a collapsible sub-block (max 3 per Finding, 2 lines each).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FewShotExample {
    pub symbol_name: String,
    pub file_path: String,
    /// 1-based line number inside `file_path`.
    pub line: u32,
    /// Short snippet (<=2 lines) taken from the DKG node signature.
    pub snippet: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimPayload {
    pub claim_text: String,
    pub grounding_source: String,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointPayload {
    pub agent_state_hash: String,
    pub grc_history_hash: String,
    pub agent_id: String,
    pub step_index: u64,
    pub file_sizes: Vec<(String, u64)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelSwapPayload {
    pub from_model: String,
    pub to_model: String,
    pub gpu_id: u8,
    pub swap_duration_ms: u64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseTransitionPayload {
    pub from_state: String,
    pub to_state: String,
    pub evidence_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeDegradationPayload {
    pub component: String,
    pub old_mode: String,
    pub new_mode: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsolationMetricPayload {
    pub agent_id: String,
    pub phase_id: String,
    pub context_savings_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanUpdatePayload {
    pub phase_id: String,
    pub version: String,
    pub change_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplanRequestPayload {
    pub agent_id: String,
    pub severity: String, // "MINOR" or "MAJOR"
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StubReferencePayload {
    pub phase_id: String,
    pub step_id: String,
    pub stub_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningTracePayload {
    pub agent_id: String,
    pub step_index: u32,
    pub step_type: String,
    pub content_summary: String,
    pub grounding_source: String,
    pub grounding_level: String,
    pub confidence: f64,
    pub elapsed_ms: u64,
    pub topic: String,
    pub budget_remaining: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseCompletePayload {
    pub phase_id: String,
    pub agent_id: String,
    pub fsm_state: String,
    pub evidence_artifacts: Vec<String>,
    pub commit_hashes: Vec<String>,
    pub test_outcomes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryMonitoringPayload {
    pub channel_id: String,
    pub deep_size_bytes: u64,
    pub threshold_exceeded: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeathEventPayload {
    pub agent_id: String,
    pub classification: String,
    pub prior_step_index: i64,
    pub checkpoint_path: String,
    pub had_checkpoint: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryEventPayload {
    pub agent_id: String,
    pub classification: String,
    pub recovered_from_checkpoint: bool,
    pub degraded: bool,
    pub first_grounding_source: String,
    pub prior_step_index: i64,
    pub recovery_step_index: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupervisionExhaustedPayload {
    pub agent_id: String,
    pub phase_id: String,
    pub restart_count: u32,
    pub max_restarts: u32,
    pub restart_history: Vec<i64>,
    pub escalation_action: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpiralDetectedPayload {
    pub agent_id: String,
    pub phase_id: String,
    pub spiral_depth: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextReductionPayload {
    pub level: u8,
    pub before_tokens: usize,
    pub after_tokens: usize,
    pub slots_affected: Vec<u8>,
}

/// B-10 (DECISION-C) — per-dispatch ContextBudget snapshot. Emitted once per
/// successful `ContextBudget::compute_with_profile` call so the dashboard
/// per-slot bar chart can render real allocation values matching the active
/// model tier (e.g. 12.5/66.7/12.5/8.3 weighting for 7B-12GB).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextBudgetComputedPayload {
    /// Tier label (e.g. "micro_7b_12gb", "micro_3b_12gb", "large_32b_48gb",
    /// "micro_8gb"). Sourced from the
    /// `ModelCapabilityProfile.model_name` so the dashboard can group by tier.
    pub tier: String,
    /// Raw model_max_context fed into `compute_with_profile` before
    /// effective_ceiling clamping (caller's pre-tier value).
    pub raw_max: usize,
    /// Per-slot weights snapshot (8 slots, scaled to 0..=10000 basis points).
    pub slots: Vec<u64>,
    /// Per-slot allocated_tokens after compute_with_profile (8 entries).
    pub allocations: Vec<usize>,
    /// Number of adaptive-reduction levels fired (0 = Normal, up to 4 = Terminal).
    pub adapt_levels_fired: u8,
}

/// B-10 / 3E.5 (DECISION-C) — SSB entry filtering summary emitted whenever
/// the SsbEntries slot truncation drops candidates scoring below
/// [`crate::context_budget::FILE_OVERLAP_THRESHOLD`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SsbFilteredByOverlapPayload {
    pub entries_total: usize,
    pub entries_kept: usize,
    pub overlap_threshold: f32,
}

/// B-19A (T10) — hook latency budget violation. Emitted by the dispatcher
/// whenever a T1/T2/T3 hook tier overruns its budget
/// (T1 <= 1ms, T2 <= 10ms, T3 <= 5ms). Used by the dashboard to surface
/// rule-engine slowness on the activity feed. The dispatcher emits at most
/// one of these per tier per dispatch; an absent row means the tier ran
/// within budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookLatencyViolationPayload {
    /// Hook point name — `"PreToolUse"`, `"PostToolUse"`, `"Stop"`,
    /// or `"PreCompact"`.
    pub hook_name: String,
    /// Tier label — `"T1"`, `"T2"`, `"T3"`, or `"Composite"`.
    pub tier: String,
    /// The configured budget in milliseconds for this tier.
    pub budget_ms: u64,
    /// The measured wall-clock latency in milliseconds.
    pub actual_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchMergePayload {
    pub winner_name: String,
    pub abandoned_names: Vec<String>,
    pub trunk_snap_id: String,
    pub from_snap_ids: Vec<String>,
}

/// SSB payload for MistakeBook entries (A-20). Wraps `elai_core::MistakeEntry`
/// for SSB storage with baby-step scoping via `baby_step_id`.
///
/// Tombstone support: when `cleared = true` this record is a clear-on-success
/// marker. All `MistakeEntry` rows for `baby_step_id` written before this
/// tombstone's `seq` are logically deleted. `query_by_baby_step` filters them
/// out automatically. This preserves SSB append-only invariant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MistakeEntryPayload {
    /// Identifies which baby step this mistake belongs to.
    pub baby_step_id: String,
    /// The structured mistake record (from elai_core).
    pub entry: elai_core::MistakeEntry,
    /// When true: this is a tombstone. All prior entries for `baby_step_id`
    /// are logically cleared. Default false for normal entries.
    #[serde(default)]
    pub cleared: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkloadProfilePayload {
    pub agent_role: String, // Serialized as snake_case string from AgentRole enum
    pub phase_id: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub latency_ms: u64,
    pub vram_peak_mb: u32,
    pub model_id: String,
    pub gpu_id: u8,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchStepPayload {
    pub agent_id: String,
    pub node_id: u64,
    pub action: String,
    pub phi: f64,
    pub psi: f64,
    pub iteration: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebResearchStepPayload {
    pub agent_id: String,
    pub node_id: u64,
    pub action: String,
    pub phi: f64,
    pub psi: f64,
    pub iteration: u32,
    pub sources_fetched: u32,
    pub sources_used: u32,
    /// Track A.4: which HTTP fetcher backend served this iteration.
    /// "reqwest" | "obscura" | "" (empty = pre-patch row, default via serde).
    #[serde(default)]
    pub fetcher: String,
    // B-7: fine-grained per-stage fields. All `#[serde(default)]` so existing
    // rows (stage=null) deserialize without error. No schema version bump:
    // payload is stored as an opaque msgpack blob; DB schema unchanged.
    // stage: "formulate"|"search"|"extract"|"reflect"|"converge"|"" (empty = coarse row)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub stage: String,
    // stage_phase: "started"|"completed"|"" (empty = coarse row)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub stage_phase: String,
    // stage_payload: stage-specific structured data (query, n_results, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_payload: Option<serde_json::Value>,
}

/// Track A.2: research synthesis path selection.
///
/// Emitted once per `phase_f_synthesize` call with the chosen synthesis path
/// ("llm" or "deterministic_fallback") and the reason for that choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchPathChosenPayload {
    /// "llm" | "deterministic_fallback"
    pub path: String,
    /// "sentinel" | "empty_response" | "no_citation_markers" | "grounding_error" | "no_claims" | "llm"
    pub reason: String,
    /// Wall-clock ms since UNIX_EPOCH at emission time.
    pub ts_ms: u64,
}

/// Track A.6: grounding cache hit or miss event.
///
/// Emitted by `ground_or_cached` on each call so the dashboard can aggregate
/// cache efficiency per research session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundingCacheEventPayload {
    /// "hit" | "miss"
    pub kind: String,
    /// First 8 hex chars of the blake3 hash of the cache key.
    pub key_fingerprint: String,
    /// ResearchLevel name: "Idea" | "Concept" | "Phase" | "Step"
    pub level: String,
    /// Microseconds elapsed from call entry to return.
    pub elapsed_us: u64,
}

/// B-4: circuit-breaker state transition for an embedding or reranker model.
/// Emitted by CrossEncoderReranker + OrtVectorModule on state changes.
/// Dashboard get_embedding_breakers queries latest row per `model`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircuitBreakerStateChangePayload {
    /// Model identifier: "graphcodebert" | "granite-embed" | "ms-marco-l12" | "ms-marco-l6" | "splade-v3"
    pub model: String,
    /// New state: "CLOSED" | "OPEN" | "HALF_OPEN"
    pub state: String,
    /// Previous state: "CLOSED" | "OPEN" | "HALF_OPEN"
    pub previous_state: String,
    /// Number of consecutive failures at transition time.
    pub failures: u32,
    /// Rolling success rate [0.0, 1.0] at transition time.
    pub success_rate: f64,
    /// Unix epoch ms of the failure that triggered this transition (None if no failure).
    pub last_failure_ts: Option<i64>,
}

// ---------------------------------------------------------------------------
// B-7 SCHEMA payload structs — deferred (v25->v26 bump coordinated with B-6/B-9)
// ---------------------------------------------------------------------------

/// B-7: one row per BubbleUpEvent pushed in handle_failure() or bubble_up_on_*_integration_fail().
/// `from_level` == `to_level` for threshold-miss events; differs on cascade triggers.
/// `action_kind` is the BubbleUpAction discriminant as a snake_case string:
///   "research_alternative" | "redecompose_step" | "replan_phase" | "replan_concept"
///   | "diagnose_and_retry" | "escalate"
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BubbleUpTypedPayload {
    pub from_level: String,
    pub to_level: String,
    pub action_kind: String,
    /// Compact context string (baby_step_id, step_id, phase_id, concept_id, or escalate context).
    pub reason: String,
    pub failure_count: u32,
    pub threshold: u32,
}

/// B-7: emitted by run_integration_checkpoint() for all 4 DecompositionLevels.
/// `level` is the DecompositionLevel discriminant: "BabyStep" | "Step" | "Phase" | "Concept".
/// This structured variant replaces the generic Finding row emitted by B-20A task_runner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationCheckpointSuccessPayload {
    pub level: String,
    pub node_id: u64,
    pub wiring_target: String,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub duration_ms: u64,
}

/// B-7: one row per RepairAction emitted by DagRepairLoop::repair().
/// `action_kind` is "reorder" | "substitute".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagRepairRoundPayload {
    pub action_kind: String,
    pub round: u32,
    pub reason: String,
}

/// B-7: one row per node built during research_decompose().
/// `node_kind` is "concept" | "phase" | "step" | "baby_step".
/// `parent_id` is 0 for root concept nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptDecompositionNodePayload {
    pub node_kind: String,
    pub node_id: u64,
    pub parent_id: u64,
    pub name: String,
}

// ---------------------------------------------------------------------------
// B-6 payload structs
// ---------------------------------------------------------------------------

/// B-6: emitted after each EvidenceGate::check() call in the FSM grounding-sweep path.
/// `verdict` is "pass" | "block". `gate_id` is the gate's canonical snake_case name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceGateEvaluatedPayload {
    /// Canonical snake_case gate name, e.g. "knowledge_readiness_gate".
    pub gate_id: String,
    /// "pass" or "block".
    pub verdict: String,
    /// Unix epoch ms at evaluation time.
    pub ts_ms: i64,
}

/// B-6: emitted by the supervision layer on each agent restart or kill event.
/// `event` is "restart" | "kill" | "spiral_detected".
/// `attempt` is the 1-based restart count within the current phase window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupervisionEventPayload {
    pub agent_id: String,
    /// "restart" | "kill" | "spiral_detected"
    pub event: String,
    /// 1-based restart count within the current phase window.
    pub attempt: u32,
}

/// B-6: one row per completed BSP superstep, emitted by the BSP engine after
/// evidence-gate passes and checkpoint is saved.
/// `snapshot_id` is "{session_id}:superstep:{n}".
/// `channels_count` is the number of active channel topics at barrier exit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BspSuperstepPayload {
    /// "{session_id}:superstep:{n}" uniquely identifies the snapshot.
    pub snapshot_id: String,
    /// Unix epoch ms at superstep completion.
    pub ts_ms: i64,
    /// Number of active channel topics at barrier exit.
    pub channels_count: u32,
}

/// B-6: agent lifecycle state change emitted by the BSP engine at spawn and death sites.
/// `state` is "spawned" | "ready" | "exited".
/// `death_class` is present only when `state == "exited"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentLifecyclePayload {
    pub agent_id: String,
    /// AgentRole discriminant in snake_case (e.g. "executor", "auditor").
    pub role: String,
    /// "spawned" | "ready" | "exited"
    pub state: String,
    /// Present when state == "exited": "clean_exit" | "crash" | "timeout" | "budget_exceeded".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_class: Option<String>,
}

/// B-6: anti-spiral escalation signal — the same step hash has appeared `count` times
/// within the current phase, triggering the escalation path in the SpiralTracker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AntiSpiralHitPayload {
    pub agent_id: String,
    /// Blake3 hex hash of the step content (or step_id string).
    pub step_hash: String,
    /// Number of times this hash was seen in the current phase window.
    pub count: u32,
}

/// B-27: per-call spend ledger row.
///
/// Emitted by cloud_mode.rs once per [`crate::providers::AnthropicClient::send_message`]
/// round-trip. The row is the SOLE write side of the cloud spend ledger:
/// the dashboard's `get_cloud_spend` Tauri command reads back via
/// `Ssb::query_by_type(SsbActionType::CloudSpend)` and aggregates per-task
/// (last entry) + per-day (sum of today's entries).
///
/// Cost is computed by [`crate::cloud_mode::SpendAccumulator`] using
/// per-model named pricing constants and a dual-rate formula
/// (input_tokens × INPUT_PRICE + output_tokens × OUTPUT_PRICE), accumulated
/// across all baby-steps of the task. This matches spec D-4:303-305 and
/// supersedes B-27's original phase_count × avg_tokens approximation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudSpendPayload {
    pub provider: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_creation_tokens: u32,
    pub cache_read_tokens: u32,
    pub cost_usd: f64,
    pub task_id: String,
    pub ts_ms: i64,
}

/// B-27: one-shot escalation marker.
///
/// Emitted by cloud_mode.rs the first time a baby-step is promoted from
/// local execution to cloud — typically because the local model exhausted
/// its retry budget, or an explicit escalation rule fired. Provides an
/// audit trail for "why did this step go to the cloud?" without consumers
/// having to reconstruct the decision from per-call signals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscalateToCloudPayload {
    pub step_id: String,
    pub trigger_reason: String,
    /// Reasoning layer that ran locally before escalating (1-5 per
    /// cloud-mode-design.txt REASONING LAYER COLLAPSE :140-152).
    pub layer_failed: u8,
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// B-23 Phase 4H Group A — self-modification beacons (6 payload structs).
// Spec: docs/specs/guide/steps/phase-4H.txt + B-23 plan.
// ---------------------------------------------------------------------------

/// `data_table_grew` — Layer 1A applied a row to one of the four bounded
/// data tables (deprecated_patterns / failure_taxonomy / lazy_code_regex /
/// source_tiers). Distinct from `proposal_applied` (the action) — this row
/// records the resulting table-row growth so the dashboard can render
/// "deprecated_patterns +1 (cargo, blake2 -> blake3)".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataTableGrewPayload {
    pub proposal_id: String,
    pub table: String,           // snake_case table name (matches DataTable::as_str)
    pub pipeline_mode: String,   // surgical | greenfield | oracle
    pub ecosystem: Option<String>,
    pub language: Option<String>,
    pub rowcount_after: u64,
    pub ts_ms: i64,
}

/// `l3_proposal` — Layer 3 structural-change proposal emitted for HUMAN
/// review only. Self-mod NEVER applies these. The dashboard surfaces them
/// for an operator approve / reject decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L3ProposalPayload {
    pub proposal_id: String,
    pub change_type: String,      // matches StructuralChangeType::as_str
    pub description: String,
    pub rationale: String,
    pub ts_ms: i64,
}

/// `convergence_stagnant` — OSC verdict (4H.7B). Latest `window` consecutive
/// ConfigVariants per `pipeline_mode` all crossed the similarity threshold.
/// Triggers exploration-bonus boost in next parent_selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvergenceStagnantPayload {
    pub pipeline_mode: String,
    pub window: u32,             // consecutive variants measured
    pub similarity: f32,         // latest similarity (>= 0.95)
    pub variant_id: String,      // latest variant id
    pub ts_ms: i64,
}

/// `config_oscillation` — OSC period-2 verdict (4H.7B). Both half-periods
/// passed the similarity threshold (sim(N,N-2) AND sim(N-1,N-3) >= 0.95).
/// Triggers AllowedConfigKey runtime-exclusion freeze for differing keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigOscillationPayload {
    pub pipeline_mode: String,
    pub period: u32,             // currently always 2
    pub frozen_keys: Vec<String>,
    pub freeze_pipelines: u32,   // pipelines for which the freeze holds
    pub ts_ms: i64,
}

/// `regression_detected` — REG cross-pipeline AC regression report (4H.7C).
/// Distinct from `pass_to_fail` in TestOutcomeReport: REG spans the entire
/// pipeline history per AC, not a single eval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegressionDetectedPayload {
    pub pipeline_id: String,
    pub pipeline_mode: String,
    pub regressed_ac_indices: Vec<u32>,
    pub max_consecutive_failures: u32,
    pub ts_ms: i64,
}

/// `evolution_withheld` — EVO gate Withhold result (4H.7D). The latest N
/// variants never structurally evolved (similarity >= 0.95 across every
/// consecutive pair). Forces MSI verdict to STABILIZING for the mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvolutionWithheldPayload {
    pub pipeline_mode: String,
    pub identical_streak: u32,
    pub reason: String,          // human-readable Withhold reason
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// B-39 chat reflect pipeline — per-turn REFLECT stage payload structs.
// Spec: B-39 plan; trait elai_reasoning::chat_reflect::ReflectEventSink.
// ---------------------------------------------------------------------------

/// B-39: TRIAGE stage verdict for a chat turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReflectTriagedPayload {
    pub turn_id: String,
    pub class: String,
}

/// B-39: single REASONER candidate emitted during best-of-N generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReflectCandidatePayload {
    pub turn_id: String,
    pub candidate_index: u32,
    pub content: String,
}

/// B-39: AUDITOR best-of-N verdict selecting the winning candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReflectAuditVerdictPayload {
    pub turn_id: String,
    pub best_index: u32,
    pub rationale: String,
}

/// B-39: SYNTHESIZE final output after the full REFLECT pipeline completes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReflectRefinedPayload {
    pub turn_id: String,
    pub total_tokens_used: u32,
    pub synthesis: String,
}

// ---------------------------------------------------------------------------
// B-24 Phase 4I Group C1: media validator payloads.
//
// Minimal structs landed in C1 so the SsbActionType <-> SsbPayload mapping
// is total. Group C4 expands each payload with the full per-step metadata
// (Tier1/Tier2 results, captured-frame paths, iteration history, ...).
// Keeping the surface narrow now means the schema bump is reversible if a
// downstream change requires renaming a field — Group C4 owns the field
// stability contract; C1 only owns the action-type names.
// ---------------------------------------------------------------------------

/// B-24: media validation started for a single media artifact. Emitted on
/// `MediaValidator::validate` entry. `step_id` ties the row to the
/// pipeline step that produced the artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MediaValidationStartedPayload {
    pub step_id: String,
    pub media_type: String,
    pub file_path: String,
}

/// B-24: Tier-1 deterministic checks complete. The full per-check
/// breakdown is added in Group C4; for now we ship `pass`+`latency_ms`
/// so the dashboard can aggregate without a payload re-serialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MediaTier1CompletePayload {
    pub step_id: String,
    pub pass: bool,
    pub check_count: u32,
    pub latency_ms: u64,
}

/// B-24: Tier-2 (VLM) advisory complete.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MediaTier2CompletePayload {
    pub step_id: String,
    pub advisory_pass: bool,
    pub model_used: String,
    pub latency_ms: u64,
}

/// B-24: render adapter (web / blender / animation / ui / comfyui)
/// finished producing a captured-frame set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RenderCompletePayload {
    pub step_id: String,
    pub adapter_name: String,
    pub frame_count: u32,
    pub latency_ms: u64,
}

/// B-24: one iteration of the visual-feedback loop completed.
/// `converged` is true when the issues set stabilized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VisualFeedbackIterationPayload {
    pub step_id: String,
    pub iteration: u32,
    pub issue_count: u32,
    pub converged: bool,
}

/// B-24: HumanReview branch — validator could not auto-decide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MediaHumanReviewRequiredPayload {
    pub step_id: String,
    pub reason: String,
}

/// WEB.3: emitted when a dev-server check passes and a preview is available.
/// Matches the spec contract from mvp-implementation-order.txt WEB.3:
///   payload = { url: "http://localhost:{port}", step_id, slot: "live" }
/// Additional fields (`agent_id`, `build_success`, `dev_server_ok`) carry
/// orchestrator/observability context for SSB consumers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewAvailablePayload {
    pub url: String,
    pub step_id: String,
    pub slot: String,
    pub agent_id: String,
    pub build_success: bool,
    pub dev_server_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchDecomposeLevelPayload {
    pub agent_id: String,
    pub level: u32,
    pub concept_count: usize,
    pub phase_count: usize,
}

// ---------------------------------------------------------------------------
// Intervention payloads (4K.1 — dashboard human-in-the-loop gates)
// ---------------------------------------------------------------------------

/// One field in a credential request form.
///
/// `id` becomes the env-var name (uppercased, sanitized to `[A-Z0-9_]+`).
/// `secret` controls whether the dashboard renders the input as type="password".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialField {
    /// Identifier for the field; becomes the env-var name after sanitization.
    pub id: String,
    /// Human-readable label shown in the credential form.
    pub label: String,
    /// When true the dashboard renders the input as a masked password field.
    pub secret: bool,
    /// Optional pre-filled default value shown in the form.
    pub default: Option<String>,
}

/// Category of human intervention being requested (4K.1).
///
/// `RequestCredential` carries the form fields the dashboard must render.
/// Adding this variant requires removing `Copy` from the enum because
/// `Vec<CredentialField>` is not `Copy`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionType {
    ApproveChange,
    ReplanApproval,
    ErrorRecovery,
    /// C2: request the user to supply one or more credential values.
    RequestCredential { fields: Vec<CredentialField> },
}

/// One option presented to the user when an intervention is raised.
///
/// The `Retry(String)` variant carries a freeform message; `Keyword` variants
/// are unit. `#[serde(untagged)]` renders enum-less JSON: `"accept"`,
/// `"decline"`, `"skip"`, or `{"retry": "message"}`, or
/// `{"credentials": {"KEY": "value", ...}}` — matching what the dashboard
/// frontend emits.
///
/// `Credentials` is the C2 variant: carries an `InterventionCredentials` whose
/// `credentials` map is keyed by `CredentialField.id`. The untagged representation
/// is `{"credentials": {"KEY": "VALUE"}}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InterventionOption {
    /// C2: credential form submission from the dashboard.
    Credentials(InterventionCredentials),
    Retry(InterventionRetry),
    Keyword(InterventionKeyword),
}

/// Keyword-form intervention option (`"accept"`, `"decline"`, `"skip"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionKeyword {
    Accept,
    Decline,
    Skip,
}

/// Structured form of the `retry` option.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionRetry {
    pub retry: String,
}

/// C2: structured form of the `credentials` option.
///
/// Serializes as `{"credentials": {"KEY": "VALUE", ...}}` so the untagged
/// `InterventionOption` enum can distinguish it from `Keyword` and `Retry`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionCredentials {
    pub credentials: BTreeMap<String, String>,
}

/// Short diff summary attached to an intervention request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffSummary {
    pub files_changed: u32,
    pub lines_added: u32,
    pub lines_removed: u32,
}

/// Payload for `intervention_request` actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterventionRequestPayload {
    pub step_id: String,
    pub intervention_type: InterventionType,
    pub description: String,
    pub diff_summary: Option<DiffSummary>,
    pub options: Vec<InterventionOption>,
}

/// Payload for `intervention_response` actions.
///
/// `request_id` references the `seq` of the original `intervention_request` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterventionResponsePayload {
    pub request_id: i64,
    pub chosen: InterventionOption,
    pub user_note: Option<String>,
}

/// Payload for `task_submitted` actions (4K.3 — dashboard MVP).
///
/// Written by the dashboard process when the user submits a task and consumed
/// by the orchestrator's task_runner (patch-1.30 commit 8ce26a0). The action
/// is recorded in the SSB so downstream consumers (dashboard activity feed,
/// replay tools) can observe the full task lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskSubmittedPayload {
    pub task: String,
    pub project_id: String,
    pub submitted_at_ms: i64,
}

/// Payload for `chat_message` actions (Tier 1 CHAT window).
///
/// Written by the dashboard's `submit_chat_message` Tauri command after the
/// client-side `intentRouter` (or the backend `classify_intent` fallback)
/// classifies the user's input. `intent` is the routed category (TASK / FIX /
/// EXPLORE / CONFIG / STATUS / CONTINUE / RESEARCH / INTERVENE / STEER /
/// ABORT / MODE / MODEL / AMBIGUOUS). No orchestrator consumer yet — the row
/// lives purely for chat history replay (same deferral pattern as
/// `TaskSubmittedPayload`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessagePayload {
    pub session_id: String,
    pub text: String,
    pub intent: String,
    pub project_id: String,
    pub submitted_at_ms: i64,
}

/// Payload for `chat_response` actions (Tier 1 CHAT window).
///
/// Written by the dashboard after rendering a system card back to the user
/// (StatusCard, ErrorCard, DisambiguationCard, SteeringAckCard). `card_kind`
/// is one of "status" | "error" | "disambiguation" | "steering_ack". `body`
/// is the rendered card text. Row is append-only, no ACK flow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatResponsePayload {
    pub session_id: String,
    pub card_kind: String,
    pub body: String,
    pub project_id: String,
    pub emitted_at_ms: i64,
}

/// Payload for `execution_mode_changed` actions (Tier 4 Settings reorg,
/// dashboard-wiring-plan.txt §8).
///
/// Written by the dashboard `set_execution_mode` Tauri command whenever the
/// user toggles the Execution section radio (LOCAL / CLOUD / HYBRID).
/// `from_mode` and `to_mode` are the validated mode strings (the command
/// rejects any value outside the fixed set before writing). `project_id`
/// identifies the active project at the time of the change. The CLOUD and
/// HYBRID branches of the dispatch path are not yet implemented (§9 group
/// 20); the dashboard emits a follow-up `Finding` row to flag the gap when
/// the user selects one of those modes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionModeChangedPayload {
    pub from_mode: String,
    pub to_mode: String,
    pub project_id: String,
    pub changed_at_ms: i64,
}

/// Batch 1B — per-baby-step snapshot of sidecar extended signals.
///
/// Written by the task_runner after every sidecar-backed dispatch (see
/// `provider_signals::ProviderSignalReader`). Fields map 1:1 to the reader's
/// trait methods; all are `Option` so rows are emitted even for partially-
/// populated providers (e.g. a sidecar that emits GG but no novelty yet).
///
/// The dashboard filters the activity feed on `provider_signals` action_type
/// and renders "GG · entropy 2.4" style rows. High-entropy thresholds (>3.5
/// warn, <1.0 confident) are resolved on the frontend (ActivityEntry.svelte).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderSignalsPayload {
    pub gg_mean_logprob: Option<f64>,
    pub gg_min_logprob: Option<f64>,
    pub entropy_mean: Option<f64>,
    pub entropy_max: Option<f64>,
    pub novelty_cosine: Option<f64>,
    pub steering_vector_active: bool,
    pub etc_disabled: bool,
    pub step_id: String,
}

/// Batch 1B — payload for the A-9 search-trigger SSB row.
///
/// Emitted only when [`provider_signals::ProviderSignalReader::read_last_search_trigger`]
/// returned `Some` on the last sidecar completion — NOT per-dispatch. Carries
/// the `reason` reported by the sidecar ETC monitor plus a threshold label so
/// the dashboard can render "search trigger · low_confidence" style rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchTriggerFiredPayload {
    pub reason: String,
    pub threshold_hit: String,
    pub step_id: String,
}

/// Batch 1B — payload for the A-37 VRAM-tier-detected SSB row.
///
/// Emitted exactly once at task_runner startup per `drive_task` host.
/// `tier` is one of `"8GB"` / `"12GB"` / `"16GB"` / `"24GB"` / `"48GB"` /
/// `"cloud"`. `detected_via` is one of `"nvml"` / `"env"` / `"default"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VramTierDetectedPayload {
    pub tier: String,
    pub detected_via: String,
    pub free_mb: Option<u32>,
}

/// Batch 1C — payload for the project-indexing stage transitions.
///
/// Emitted by the task_runner's `maybe_build_project_index` wrapper at
/// boundaries of the `IndexingService::run_full_index` call. Currently two
/// stages materialise per successful run (`start`, `complete`) plus one on
/// failure (`failed`); the `dkg_parsing` and `bm25_indexing` stages are
/// reserved for the future incremental/streaming build surface that the
/// indexing service will expose once tree-sitter and tantivy publish
/// per-file callbacks.
///
/// The `files_total` field is advisory — it may be zero when the runner
/// cannot cheaply enumerate the project (e.g. very large repos). The
/// frontend renders "42/150 files" only when `files_total > 0`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexingProgressPayload {
    /// One of `"start"` / `"dkg_parsing"` / `"bm25_indexing"` / `"complete"`
    /// / `"failed"`. Stable set; see contract
    /// `task_runner_indexing_service_contract::indexing_stage_labels_are_stable`.
    pub stage: String,
    /// Project identifier — matches the `project_id` on `TaskSubmittedPayload`.
    pub project_id: String,
    /// Number of files processed so far. Zero at `start`.
    pub files_processed: u32,
    /// Estimated total file count. Zero when unknown.
    pub files_total: u32,
    /// Milliseconds elapsed since the `start` emission. Zero at `start`.
    pub elapsed_ms: u64,
    /// Optional human-readable status message (e.g. the error string on
    /// `failed`). Typically `None` on success stages.
    pub message: Option<String>,
    /// Track A.1: machine-readable error kind for the "failed" stage
    /// (e.g. "bm25_error", "dkg_error"). `None` on non-failure stages.
    /// Backward-compatible: old rows without this field deserialize to `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_kind: Option<String>,
    /// Track A.1: full error message string for the "failed" stage.
    /// Duplicates `message` with a semantically distinct name so dashboards
    /// can branch on the typed field rather than parsing `message`.
    /// Backward-compatible: old rows without this field deserialize to `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// B-32 (SCHEMA_VERSION 38) — payload for the
/// [`SsbActionType::MinifiedFileChunked`] one-row-per-minified-file emission.
///
/// Emitted by [`crate::indexing_service::IndexingService::build_index_resumable`]
/// after the BM25 batch loop completes, once per file whose content satisfies
/// `elai_knowledge::chunking::detect::is_minified` (average line length >
/// `MAX_AVG_LINE_LENGTH = 500` chars). Mirrors the tracing line emitted at
/// `bm25.rs:412/523` so the dashboard can surface minified files (vendored
/// bundles, generated protobuf go, minified CSS) without having to parse
/// tracing output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinifiedFileChunkedPayload {
    /// Path relative to the project root, e.g. `vendor/bundle.min.js`.
    /// Stable across machines (no absolute prefix).
    pub relative_path: String,
    /// Raw byte length of the file content. Persisted as `u64` to match the
    /// `metadata.indexed_files` column shape; minified bundles routinely
    /// exceed `u32::MAX` chars after concatenation.
    pub char_count: u64,
    /// Number of chunks emitted by the character-based chunker (does NOT
    /// include the zero-orphan `ChunkKind::File` chunk that bm25 always
    /// appends).
    pub chunk_count: u32,
    /// Computed `content.len() / line_count` average. The detector's
    /// threshold is `MAX_AVG_LINE_LENGTH = 500`; values below 500 will never
    /// produce this row.
    pub avg_line_length: u32,
}

/// B-33 follow-up (SCHEMA_VERSION 40) — payload for the
/// [`SsbActionType::SymbolGraphUpdated`] one-row-per-build emission.
///
/// Emitted by [`crate::indexing_service::IndexingService::build_index_resumable`]
/// after [`elai_knowledge::symbol_graph::build_pipeline::build_symbol_graph`]
/// returns Ok with `BuildSummary { files > 0, ... }`. Replaces the 30s
/// `setInterval` polling fallback that B-33 dashboard-v2 GraphWindow shipped
/// while `ssb.rs` was locked. Consumers (GraphWindow event listener,
/// KnowledgeDrawer refresh hook) drive UI invalidation on receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolGraphUpdatedPayload {
    /// BLAKE3-derived per-project identifier (matches
    /// `metadata::project_id_from_path`).
    pub project_id: String,
    /// Number of source files indexed by the symbol-graph build.
    /// `u32` is sufficient for any project size we ship to (>4G files
    /// would imply a multi-TB corpus).
    pub file_count: u32,
    /// Total number of `SymbolNode` rows persisted by the build.
    /// `u64` matches the `metadata.symbol_count` column shape.
    pub symbol_count: u64,
    /// Total number of `SymbolEdge` rows persisted by the build.
    /// `u64` matches the `metadata.edge_count` column shape.
    pub edge_count: u64,
    /// Wall-clock UTC milliseconds when the build completed (sourced from
    /// the injected `ClockProvider`). Used by the dashboard for "fetched
    /// X ms ago" timestamps.
    pub built_ms: i64,
    /// C-24 (DRIFT-4): `Some(true)` when the producer used the incremental
    /// `update_changed_files_symbol_graph` pathway (i.e. a resume after a
    /// prior successful index). `Some(false)` or `None` when a full
    /// `build_symbol_graph` rebuild ran (cold start / above-threshold
    /// change set). Additive `Option<bool>` so existing rmp-serde fixtures
    /// continue to decode — same precedent as C-17 DECISION-8 for
    /// `IndexCheckpointPayload::batch_duration_ms`. No SCHEMA_VERSION bump.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_incremental: Option<bool>,
}

// ---------------------------------------------------------------------------
// B-40 (SCHEMA_VERSION 41) — FRAMING substate payloads
// ---------------------------------------------------------------------------

/// B-40 (SCHEMA_VERSION 41) — payload for [`SsbActionType::IdeaFramingStarted`].
///
/// Emitted once when the FRAMING substate is entered. `prompt` is the raw
/// user input (trimmed). `session_id` is provided redundantly so dashboard
/// consumers that subscribe to a global SSB stream can scope per-task without
/// reading the SsbEntry envelope. `started_at_ms` is the wall-clock unix-ms
/// timestamp sourced from the injected [`elai_core::ClockProvider`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdeaFramingStartedPayload {
    pub session_id: String,
    pub prompt: String,
    pub started_at_ms: i64,
}

/// B-40 (SCHEMA_VERSION 41) — payload for [`SsbActionType::ProjectClassClassified`].
///
/// `project_class` is the snake_case discriminant of
/// [`crate::idea_framing::ProjectClass`] (one of 9 variants). `matched_pattern`
/// records the regex / keyword that fired in the classifier; `None` when the
/// fallback `Other` arm matched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectClassClassifiedPayload {
    pub session_id: String,
    pub project_class: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_pattern: Option<String>,
}

/// B-40 (SCHEMA_VERSION 41) — payload for [`SsbActionType::CriticRoundComplete`].
///
/// Emitted at the end of each [`crate::idea_framing::run_critic_loop`] round.
/// `round_index` is 1-based (rounds 1..=3). `category_counts` maps each of the
/// 10 [`crate::idea_framing::CriticCategory`] discriminants (snake_case) to
/// the number of findings emitted in that round. `block_count` / `warn_count` /
/// `info_count` are the per-severity totals. `converged` is true when
/// `block_count == 0` (loop exits early).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriticRoundCompletePayload {
    pub session_id: String,
    pub round_index: u32,
    pub block_count: u32,
    pub warn_count: u32,
    pub info_count: u32,
    pub converged: bool,
    /// snake_case CriticCategory name → finding count for that round.
    pub category_counts: Vec<(String, u32)>,
}

/// B-40 (SCHEMA_VERSION 41) — payload for [`SsbActionType::IdeaFramed`].
///
/// Emitted by [`crate::idea_framing::frame_idea`] after framed-idea.toml is
/// persisted. `artifact_path` is the absolute path
/// `workflow/sessions/{sid}/idea-framing/framed-idea.toml`. `rounds_used` is
/// the count of critic rounds actually executed (1..=3). `tradeoff_count` is
/// the number of tradeoffs on the TradeoffCard. Consumed by
/// [`crate::task_runner::derive_framing_evidence`] (the 7th
/// AutoAdvanceEvaluator check, gated on FRAMING substate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdeaFramedPayload {
    pub session_id: String,
    pub artifact_path: String,
    pub project_class: String,
    pub rounds_used: u32,
    pub tradeoff_count: u32,
    /// True when the critic loop converged (zero BLOCK findings) within
    /// `rounds_used` rounds. False indicates an idea_framing_block
    /// condition (FRAMING substate refuses to advance).
    pub converged: bool,
}

// ---------------------------------------------------------------------------
// B-34 (SCHEMA_VERSION 42) payload structs
// ---------------------------------------------------------------------------

/// B-34 — payload for [`SsbActionType::EntryPointsDetected`].
///
/// Emitted by the `entry_points` dispatcher arm after `detect_entry_points`
/// returns. Lets the dashboard render an entry-point summary card without
/// re-running detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPointsDetectedPayload {
    pub project_id: String,
    /// Total detected entry points (including tests if include_tests=true).
    pub count: u32,
    /// Per-reason breakdown (e.g. "orphan" -> 3, "well_known:main" -> 2).
    pub by_reason: std::collections::BTreeMap<String, u32>,
}

/// B-34 — payload for [`SsbActionType::ArtifactsIndexed`].
///
/// Emitted by the Phase C artifact indexer wiring after a complete
/// (re)indexing run finishes. Carries the partition of artifacts that were
/// freshly indexed vs. already up-to-date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactsIndexedPayload {
    pub project_id: String,
    /// Artifact names freshly (re)indexed in this run.
    pub indexed: Vec<String>,
    /// Artifact names whose content hash matched — skipped as up-to-date.
    pub up_to_date: Vec<String>,
    /// Total chunk count across all indexed artifacts after this run.
    pub total_chunks: usize,
    /// Wall-clock time for the full indexing run in milliseconds.
    pub latency_ms: u64,
}

/// B-34 — payload for [`SsbActionType::ArtifactSearched`].
///
/// Emitted by the `search_artifacts` dispatcher arm after each query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSearchedPayload {
    pub project_id: String,
    /// Optional artifact name filter applied to the query (None = all).
    pub artifact_name: Option<String>,
    /// The natural-language query string.
    pub query: String,
    /// Number of results returned (after limit clamping).
    pub result_count: usize,
    /// Wall-clock query latency in milliseconds.
    pub latency_ms: u64,
}

/// B-34 — payload for [`SsbActionType::ArtifactStale`].
///
/// Emitted when the ArtifactIndexer detects that an artifact's content has
/// changed (blake3 hash mismatch). Triggers a dashboard staleness badge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactStalePayload {
    pub project_id: String,
    pub artifact_name: String,
    /// Repo-relative paths of files whose content hash changed.
    pub changed_files: Vec<String>,
}

/// B-40 (SCHEMA_VERSION 41) — payload for [`SsbActionType::TradeoffCardEmitted`].
///
/// Emitted by [`crate::idea_framing::frame_idea`] when the TradeoffCard is
/// surfaced to the dashboard. `tradeoff_names` is the ordered list of
/// `Tradeoff.name` strings; `option_counts` is the parallel ordered list of
/// per-tradeoff option-list lengths (for dashboard validation that responses
/// reference valid options).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeoffCardEmittedPayload {
    pub session_id: String,
    pub tradeoff_names: Vec<String>,
    pub option_counts: Vec<u32>,
}

/// B-31.5 (SCHEMA_VERSION 39) — payload for [`SsbActionType::IndexCheckpoint`].
///
/// Emitted by [`crate::indexing_service::IndexingService::build_index_resumable`]
/// after each successful batch is persisted via `ProjectMetadataStore::save`.
/// `batches_total` is `None` when the work queue was empty up-front (no
/// candidates) or could not be sized cheaply.
///
/// C-17 (DRIFT-I): `batch_duration_ms` carries the per-batch wall-clock
/// elapsed in milliseconds, sourced from an OsClock-backed `Instant::now()`
/// taken at batch entry. Added as `Option<u64>` so older readers tolerate
/// payloads emitted without the field (back-compat under rmp-serde's named
/// encoding — no SCHEMA_VERSION bump required for the additive optional
/// field). Consumers parse p50/p99 across recent rows for the scale TOMLs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexCheckpointPayload {
    /// BLAKE3-derived per-project identifier (matches
    /// `metadata::project_id_from_path`).
    pub project_id: String,
    /// Number of completed batches at the time of this checkpoint row.
    pub batches_processed: u32,
    /// Total batch count when computable. `None` when `total_files == 0`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batches_total: Option<u32>,
    /// Cumulative count of files actually indexed (i.e. handed to
    /// `Bm25Index::index_file`) since the run started. Skipped/resumed files
    /// are NOT counted here.
    pub files_processed: u64,
    /// Unix-seconds timestamp of the checkpoint, sourced from the injected
    /// [`elai_core::ClockProvider`].
    pub last_checkpoint_at: i64,
    /// C-17 (DRIFT-I): wall-clock duration of the batch that just completed,
    /// in milliseconds, measured via `clock.instant_now()`. Optional for
    /// back-compat with pre-C-17 emitters; readers that need real p50/p99
    /// timing skip rows where this is `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_duration_ms: Option<u64>,
}

/// B-31.5 (SCHEMA_VERSION 39) — payload for [`SsbActionType::IndexResume`].
///
/// Emitted in two contexts:
/// * `build_index_resumable` returning [`crate::indexing_service::IndexResult::Resumed`].
/// * `auto_resume_indexed_projects` after a resumed run completes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexResumePayload {
    /// BLAKE3-derived per-project identifier.
    pub project_id: String,
    /// Value of `batches_processed` at the moment the run resumed.
    pub resumed_from_batch: u32,
    /// Number of candidate files whose stored BLAKE3 hash matched the
    /// on-disk content and were therefore skipped.
    pub files_skipped: u64,
}

/// B-31.5 (SCHEMA_VERSION 39) — payload for [`SsbActionType::IndexCancel`].
///
/// Emitted when the in-process cancellation flag is observed by
/// `build_index_resumable`, OR when `graceful_shutdown` signals a project
/// during the SHUTTING_DOWN cancellation pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexCancelPayload {
    /// BLAKE3-derived per-project identifier.
    pub project_id: String,
    /// Number of completed + checkpointed batches at abort time.
    pub batches_completed: u32,
    /// Cumulative count of files indexed before the abort.
    pub files_indexed: u64,
}

/// B-31.5 (SCHEMA_VERSION 39) — payload for [`SsbActionType::LockAcquired`].
///
/// Emitted by `IndexLock::try_acquire` when the process-global SSB writer
/// has been installed via `lock::install_ssb_writer`. The `scope` field
/// uses the same lowercase string as `LockScope::as_str` (`"index"` or
/// `"watch"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockAcquiredPayload {
    /// BLAKE3-derived per-project identifier.
    pub project_id: String,
    /// Lockscope discriminant string — `"index"` or `"watch"`.
    pub scope: String,
    /// OS process ID of the holder (== this process at acquire time).
    pub holder_pid: u32,
}

/// B-31.5 (SCHEMA_VERSION 39) — payload for [`SsbActionType::LockContested`].
///
/// Emitted on
/// [`crate::lock::IndexLockError::AlreadyHeld`] when the process-global
/// SSB writer is installed. `holder_pid` is the LIVE holder (not the
/// contender — the contender is the process emitting the row).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockContestedPayload {
    /// BLAKE3-derived per-project identifier.
    pub project_id: String,
    /// Lockscope discriminant string — `"index"` or `"watch"`.
    pub scope: String,
    /// OS process ID of the live holder that blocked acquisition.
    pub holder_pid: u32,
}

/// Batch 1C — payload for the one-shot DKG-ready announcement.
///
/// Emitted once per project per runner lifetime, immediately after the
/// first successful `IndexingService::run_full_index`. The dashboard's rule
/// engine may use this row as a signal to hot-swap the `EmptyDkgQuerier`
/// stub for a real `DkgQuerierAdapter`; see
/// `docs/mvp-wiring-audit-2026-04-18.md` section 3 A-48.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgReadyPayload {
    /// Project identifier — matches the `project_id` on `TaskSubmittedPayload`.
    pub project_id: String,
    /// DKG node count after the build completes.
    pub node_count: u32,
    /// DKG edge count after the build completes.
    pub edge_count: u32,
    /// Languages detected during the build (e.g. `["rust", "ts"]`). May be
    /// empty when the DKG engine does not expose per-language stats.
    pub languages: Vec<String>,
}

/// Batch 1A — payload for the hook dispatcher's SSB emissions.
///
/// Written by `crate::hook_dispatcher::emit_hook_events_to_ssb` on every
/// non-Allow verdict produced by the T1/T2/T3 tiered analysis (plus a
/// Composite Allow breadcrumb per run). The dashboard filters the activity
/// feed by `hook_event` action_type and renders "hook · T1 BLOCK" style
/// rows keyed on `tier` + `verdict`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookEventPayload {
    /// `"T1"` / `"T2"` / `"T3"` / `"Composite"` — matches `HookTier::as_str`.
    pub tier: String,
    /// `"Allow"` / `"Warn"` / `"Block"` — matches `HookVerdictKind::as_str`.
    pub verdict: String,
    /// Originating rule identifier (or predicate hash for pattern hooks).
    /// `None` for Allow breadcrumbs and dispatcher-internal messages.
    pub rule_id: Option<String>,
    /// Stable category tag; set to `HOOK_EVENT_CATEGORY` for every emission
    /// so downstream filters do not need to introspect `rule_id`.
    pub category: String,
    pub message: String,
    /// Baby-step identifier (typically `"baby_step_<N>"`) or `"task_<seq>"`
    /// for Stop / PreCompact events. Lets the dashboard correlate a hook
    /// verdict with the step it gated.
    pub step_id: String,
    /// `"PreToolUse"` / `"PostToolUse"` / `"Stop"` / `"PreCompact"`.
    pub hook_point: String,
}

/// Batch 2A — payload for a per-invocation tiered-analysis summary row.
///
/// Written once per T1/T2/T3 dispatch in addition to the per-verdict
/// [`HookEventPayload`] rows. Lets the dashboard render "T1 · 3 rules /
/// 0.4ms" without having to aggregate the individual hook_event rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TieredAnalysisPayload {
    /// `"T1"` / `"T2"` / `"T3"` — matches `elai_orchestrator::hook_dispatcher::HookTier::as_str`.
    pub tier: String,
    /// `"Allow"` / `"Warn"` / `"Block"` — matches `HookVerdictKind::as_str`.
    pub verdict: String,
    /// Number of rules / AST checks / patterns that produced a non-Allow
    /// verdict during this invocation.
    pub rule_count_fired: u32,
    /// Wall-clock elapsed time for this tier only, in microseconds.
    pub elapsed_us: u64,
    /// Baby-step identifier (typically `"baby_step_<N>"`).
    pub step_id: String,
    /// High-level category string (e.g. `"structural"`, `"accessibility_hint"`,
    /// `"naming_convention"`, `"todo_public_api"` for T3 advisories;
    /// `"hook_event"` for T1/T2 composite rows).
    pub category: String,
}

/// Batch 2A — payload for a ShortCoder compression row.
///
/// Emitted once per PostToolUse invocation when the ShortCoder simplifies
/// an [`AgentResponse`] before the auditor reads it. Approximate token
/// counts use the 4-chars-per-token heuristic that matches the rest of
/// the codebase (see `output_postprocessor::CHARS_PER_TOKEN`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortCoderCompressionPayload {
    /// Approximate input token count (bytes / 4).
    pub tokens_in: u32,
    /// Approximate output token count (bytes / 4).
    pub tokens_out: u32,
    /// Percent reduction in [0, 100]. `0` means no reduction (or expansion).
    pub reduction_pct: u32,
    /// Baby-step identifier.
    pub step_id: String,
    /// Stable rule labels that fired (e.g. `["collapse_bodies",
    /// "strip_line_comments"]`). Max 10 — capped to prevent runaway size.
    pub rules_applied: Vec<String>,
}

// ---------------------------------------------------------------------------
// Batch 2B — ToolDispatcher payloads
// ---------------------------------------------------------------------------

/// Batch 2B — per-dispatch summary row. Emitted once per call to each of
/// the six `ToolDispatcher` methods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDispatchPayload {
    /// One of `"edit" | "validate" | "classify_failure" | "git_commit"
    /// | "file_read_outline" | "incremental_compile"`.
    pub tool: String,
    /// Short ASCII summary of the arguments (no binary bytes). E.g.
    /// `"path=src/lib.rs tier=exact"` for an edit row.
    pub args_summary: String,
    pub elapsed_ms: u32,
    pub step_id: String,
    /// `"success"` / `"error"` / `"skipped"` / `"not_implemented"`.
    pub outcome: String,
    /// Populated when `outcome == "error"` AND the classifier recognised
    /// the failure category.
    pub error_category: Option<String>,
}

/// Batch 2B — per-tier validation verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationTierPayload {
    /// One of `"determinism" | "quality_ast" | "unit" | "integration" | "stress"`.
    pub tier: String,
    pub passed: bool,
    pub elapsed_ms: u32,
    pub rules_fired: u32,
    pub step_id: String,
}

/// Batch 2B — failure classification emitted by `dispatch_classify_failure`.
/// Confidence is stored as u16 in basis points (0..=10000) to keep the
/// payload `Eq` for msgpack-serialised equality tests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureClassifiedPayload {
    pub category: String,
    /// Confidence × 10000 (basis points). 10000 = 1.0, 7000 = 0.7.
    pub confidence_bp: u16,
    pub step_id: String,
    /// One of `"agent" | "validator" | "deterministic_audit"`.
    pub source: String,
}

/// Batch 2B — real git2 commit boundary row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitCommitPayload {
    pub commit_sha: String,
    pub branch: String,
    pub files_count: u32,
    pub step_id: String,
    pub message: String,
}

/// B-19 — DAIRA 15-marker vocabulary for the `phase` field of
/// [`DairaTracePayload`]. Kept as a `&str` slice (not an enum) so existing
/// call sites that emit legacy values ("enter"/"exit") continue to compile
/// without modification while the frontend and SSB viewer can validate
/// against this canonical list.
///
/// Marker semantics:
/// - `Call`          — function entry (legacy: "enter")
/// - `Return`        — function exit clean (legacy: "exit" / "success")
/// - `Mutation`      — state-changing side effect (e.g. file write)
/// - `Branch`        — conditional path taken
/// - `Error`         — error bubble (legacy: "error")
/// - `Finding`       — auditor BLOCK emit
/// - `Read`          — file / db read
/// - `Write`         — file / db write
/// - `ToolCallStart` — tool dispatch entry
/// - `ToolCallEnd`   — tool dispatch exit
/// - `HookPre`       — hook point firing before action
/// - `HookPost`      — hook point firing after action
/// - `Timeout`       — operation exceeded latency budget
/// - `Escalate`      — escalation signal needed
/// - `Converge`      — spiral convergence / resolution
pub const DAIRA_MARKERS: &[&str] = &[
    "Call",
    "Return",
    "Mutation",
    "Branch",
    "Error",
    "Finding",
    "Read",
    "Write",
    "ToolCallStart",
    "ToolCallEnd",
    "HookPre",
    "HookPost",
    "Timeout",
    "Escalate",
    "Converge",
];

/// Batch 2B — DAIRA call-graph breadcrumb. Emitted on entry AND exit of
/// every dispatch method so the frontend can assemble a call graph keyed
/// on `(phase, tool)` without having to deserialise the per-dispatch row.
///
/// B-19: `phase` now accepts the 15-marker DAIRA vocabulary defined in
/// [`DAIRA_MARKERS`] in addition to the legacy "enter"/"exit" strings.
/// Legacy values remain valid and are NOT rejected — this is a superset
/// expansion, not a breaking change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DairaTracePayload {
    /// DAIRA phase marker — see [`DAIRA_MARKERS`] for the canonical 15-value
    /// vocabulary. Legacy values `"enter"` / `"exit"` remain accepted.
    pub phase: String,
    /// Tool name — same tag set as [`ToolDispatchPayload::tool`].
    pub tool: String,
    /// Blake3 short hash (8 bytes hex) of the args summary. Lets the
    /// frontend correlate enter/exit pairs without needing stable IDs.
    pub args_hash: String,
    pub elapsed_us: u64,
    /// On `exit`: `"success" | "error" | "skipped" | "not_implemented"`.
    /// On `enter`: `"started"`.
    pub outcome: String,
    pub step_id: String,
}

// ---------------------------------------------------------------------------
// Batch 3A (Phase 4F) — SystemProbe / FeasibilityReport payloads
// ---------------------------------------------------------------------------

/// Batch 3A — snapshot of the [`crate::system_probe::SystemProbeResult`] at
/// task_runner startup. Emitted once per runner lifetime. `vram_*` fields are
/// `Option<u32>` so env-fallback probes with no GPU data still serialise
/// cleanly. `detection_method` / `tier` are stable string labels matching
/// `DetectionMethod::as_str()` and the `VramTierDetectedPayload` tier set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemProbePayload {
    pub gpu_name: Option<String>,
    pub vram_total_mb: Option<u32>,
    pub vram_free_mb: Option<u32>,
    pub ram_total_mb: u64,
    pub ram_free_mb: u64,
    pub disk_total_mb: u64,
    pub disk_free_mb: u64,
    /// `"shell_out"` / `"env_fallback"` / `"unavailable"`.
    pub detection_method: String,
    /// `"8GB"` / `"12GB"` / `"16GB"` / `"24GB"` / `"48GB"` / `"cloud"`.
    pub tier: String,
}

/// Batch 3A — per-concept / per-task feasibility summary. Percent values are
/// encoded as basis-point u16 (0..=10000) so the payload stays `Eq`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeasibilityReportPayload {
    /// `"feasible"` / `"marginal"` / `"infeasible"`.
    pub task_feasibility: String,
    /// Populated when the probe was skipped (e.g. `"no_novel_steps_margins_ok"`).
    pub skip_reason: Option<String>,
    pub novel_step_count: u32,
    pub escalations_count: u32,
    /// VRAM margin × 100 as u16 (bp). 2000 = 20.00%.
    pub vram_margin_bp: u16,
    pub ram_margin_bp: u16,
    pub disk_margin_bp: u16,
}

/// Batch 3A — one row per novel step detected by the feasibility probe.
/// Emitted inline with the enclosing [`FeasibilityReportPayload`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NovelStepDetectedPayload {
    pub step_id: String,
    pub reason: String,
    pub alternative_count: u32,
}

/// Batch 3A — one row per stub execution inside the feasibility probe.
/// Emitted per novel step + per baby-step probe invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StubRunPayload {
    pub step_id: String,
    pub passed: bool,
    pub elapsed_ms: u32,
}

/// Batch 3B (Phase 4G) — one row per [`crate::autonomous_gate::AutonomousGate::decide`]
/// call. `decision` is the [`crate::autonomous_gate::AutonomyDecision`]
/// variant name (`"auto_advance"` / `"request_human"` / `"split_task"` /
/// `"substitute_model"` / `"escalate"`). `risk_level` is populated on
/// RequestHuman; `reason` on RequestHuman / Escalate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateDecisionPayload {
    pub decision: String,
    pub risk_level: Option<String>,
    pub reason: Option<String>,
    pub evidence_summary: String,
    pub step_id: String,
}

/// Batch 3B (Phase 4G) — task-split announcement. Emitted when the gate
/// decides the current task must be split under context pressure.
///
/// Batch FUL-B extends the payload with three optional fields describing
/// the real splitter's output (human-readable summaries, splitter kind,
/// token estimate). All three carry `#[serde(default)]` so rows serialized
/// before FUL-B still deserialize into `None` / `None` / `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskSplitPayload {
    pub original_task_id: String,
    pub sub_task_ids: Vec<String>,
    pub depth: u8,
    /// Batch FUL-B: one-line summary per sub-task, aligned positionally
    /// with `sub_task_ids`. `None` when the deterministic-fallback path
    /// fired (Batch 3B semantics, no splitter plugged).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_task_summaries: Option<Vec<String>>,
    /// Batch FUL-B: name of the splitter that produced the sub-tasks.
    /// Canonical values: `"phase_sizing"` (real splitter) or
    /// `"deterministic_fallback"` (Batch 3B fallback).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub splitter: Option<String>,
    /// Batch FUL-B: token estimate that triggered the split. Populated
    /// only when `splitter = Some("phase_sizing")`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complexity_estimate: Option<u32>,
}

/// Batch 3B (Phase 4G) — escalation trigger row. `trigger` is the
/// [`crate::autonomous_gate::EscalationTrigger::as_str`] label. `details` is
/// free-form (e.g. the AntiSpiralDetector depth).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscalationFiredPayload {
    pub trigger: String,
    pub step_id: String,
    pub details: String,
}

/// Batch 3B (Phase 4G) — gate-mode change. Lower-cased (`"interactive"` /
/// `"autonomous"`) to match the `/mode` slash-command syntax.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeChangedPayload {
    pub from: String,
    pub to: String,
}

/// Batch 3C (Phase 4H) — Layer 1A proposal generated row. Emitted once per
/// `process_outcome` proposal that passes the FPR + security-sensitive gates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposalGeneratedPayload {
    pub proposal_id: String,
    pub table: String,
    pub new_row_summary: String,
    pub fpr_estimate: f32,
    pub source: String,
}

/// Batch 3C (Phase 4H) — Layer 1A proposal applied row. `commit_sha` is
/// `None` when git2 wiring is absent or the commit failed (non-fatal).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalAppliedPayload {
    pub proposal_id: String,
    pub table: String,
    pub applied_by: String,
    pub commit_sha: Option<String>,
}

/// Batch 3C (Phase 4H) — Layer 1A proposal rejected row. Emitted both at
/// automatic rejection time (FPR/security) and from the dashboard reject
/// command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalRejectedPayload {
    pub proposal_id: String,
    pub table: String,
    pub reason: String,
}

/// Batch 3C (Phase 4H) — MSI update row. Emitted on every apply. `status` is
/// one of `"IMPROVING"` / `"STABILIZING"` / `"DIVERGING"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsiUpdatePayload {
    pub pipeline_mode: String,
    pub msi_value: f32,
    pub status: String,
}

/// Batch 3C (Phase 4H) — config archive cap event. `dropped_id` is
/// `Some(_)` when the 51st apply bumped an old entry out; `None` otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigArchivedPayload {
    pub pipeline_mode: String,
    pub archived_count: u32,
    pub dropped_id: Option<String>,
}

/// Batch 4C (A-18) — per-dispatch GenerationProfile selection.
///
/// Emitted by [`crate::profile_selector::GenerationProfileSelector`]
/// implementations whenever the task runner chooses a generation profile for
/// the next sidecar dispatch. `profile` is the [`elai_orchestrator::context_budget::GenerationProfile`]
/// `as_str` form (e.g. `"FirstAttempt"`, `"RepairRetry"`). `role` is a free-form
/// string label for the dispatch role (`"executor"`, `"auditor"`, etc.).
/// `fsm_state` is the [`crate::fsm::FsmState`] rendered via its `Display` impl.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSelectedPayload {
    pub profile: String,
    pub role: String,
    pub fsm_state: String,
    pub step_id: String,
}

/// Batch 4C (A-22) — FSM transition emitted on RESEARCHING entry/exit.
///
/// Emitted by `drive_web_research_task` (and any future RESEARCHING caller) at
/// state boundary. Both `from` and `to` are the `Display`-rendered FSM state
/// names (e.g. `"IDLE"`, `"RESEARCHING"`, `"EXECUTING"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FsmTransitionPayload {
    pub from: String,
    pub to: String,
    pub reason: String,
}

/// Batch 4C (A-23) — micro-reflection diagnosis row.
///
/// Emitted after a baby-step failure once the sidecar-backed 500-token
/// reflection returns. `root_cause` is truncated to 100 chars by the frontend
/// for the activity feed; full text remains in the SSB for detail view.
/// `confidence` is the reflection's self-reported 0.0..=1.0 score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MicroReflectionPayload {
    pub step_id: String,
    pub root_cause: String,
    pub proposed_fix: String,
    pub confidence: f32,
}

/// Batch 4C (A-46) — ContextTelescope collapse event.
///
/// Emitted by [`crate::telescope_caller::invoke_telescope`] each time a
/// repair-context build trims layers to fit under a token budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelescopeCollapsePayload {
    pub from_tokens: u32,
    pub to_tokens: u32,
    pub layers_collapsed: u32,
    pub step_id: String,
}

/// Batch 4C (A-45) — config-load warning.
///
/// Emitted when a non-fatal config file lookup fails (e.g. missing
/// `source-tiers.toml` at dashboard startup) and the runtime falls back to a
/// safe default. `config` is a short identifier (e.g. `"source-tiers.toml"`);
/// `reason` is a short classifier (e.g. `"file_not_found"`, `"parse_error"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigWarningPayload {
    pub config: String,
    pub reason: String,
    pub detail: String,
}

/// Batch-fu (FU.2 / A-7) — constitutional CONSTRAINTS block injection row.
///
/// Emitted by the task_runner's `build_dispatch_context` call site each time a
/// dispatch is assembled with a non-empty `ProjectProfile.constraints` list.
/// `constraint_count` is the number of bullets rendered into the block;
/// `step_id` is the baby-step identifier so the dashboard can correlate the
/// row with a specific dispatch. `project_id` is denormalised onto the payload
/// in addition to the SSB `project_id` column so subscribe-only consumers that
/// only see the payload can still filter by project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstraintsInjectedPayload {
    pub project_id: String,
    pub constraint_count: u32,
    pub step_id: String,
}

/// Batch-fu (FU.5 / A-31) — intervention resolution row.
///
/// Emitted by the task_runner after a matching `InterventionResponse` arrives
/// on the SSB broadcast channel for a pending IRREVERSIBLE-confirmation
/// `InterventionRequest`. Pairs with the request by `intervention_id` so the
/// dashboard can close out a request-row once resolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionResolvedPayload {
    pub intervention_id: String,
    pub approved: bool,
    pub elapsed_ms: u32,
}

/// Batch-ful-a (FUL-A.1) — DkgQuerier upgrade event.
///
/// Emitted by `task_runner::drive_task` immediately after a fresh build of the
/// DKG database succeeds (`IndexingProgress=complete` + `DkgReady`). The rule
/// engine's previously-plugged `EmptyDkgQuerier` is replaced in-place with a
/// `DkgQuerierAdapter` via the `HotSwappableDkgQuerier` wrapper so every
/// subsequent rule evaluation sees the live DKG without requiring a dashboard
/// relaunch. Exactly one row per project per runner lifetime — the dashboard
/// surfaces this as "DKG querier upgraded · empty → adapter".
///
/// `from` / `to` are tags (not full type names) so the wire format remains
/// small and stable. Current values: `"empty"` (EmptyDkgQuerier) /
/// `"adapter"` (DkgQuerierAdapter). Additional tags may be added as the
/// querier hierarchy grows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgQuerierSwappedPayload {
    /// Project identifier — matches the `project_id` on `TaskSubmittedPayload`.
    pub project_id: String,
    /// Short tag for the previous querier implementation (e.g. `"empty"`).
    pub from: String,
    /// Short tag for the newly-installed querier implementation (e.g.
    /// `"adapter"`).
    pub to: String,
}

/// Batch-ful-c (FUL-C / A-8) — steering vector computed payload.
///
/// Emitted by `SteeringVectorEngine::compute_vector_via_sidecar_role` after
/// a successful sidecar round-trip AND cache-file write. Carries the role
/// label, final vector dim, L2 norm, elapsed wall-clock time in ms, and the
/// cache path (stringified for JSON round-trip friendliness).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteeringComputedPayload {
    /// Role label the vector was computed for (e.g. `"executor"`).
    pub role: String,
    /// Dimension of the resulting vector (typically = model hidden_dim).
    pub vector_dim: u32,
    /// L2 norm of the vector. Zero norm implies the contrast pair produced
    /// no difference; the helper rejects these before emitting this row so
    /// callers can treat `norm > 0` as an invariant.
    pub norm: f32,
    /// Wall-clock time elapsed in milliseconds from request send to cache
    /// write completion.
    pub elapsed_ms: u32,
    /// Absolute filesystem path of the persisted cache bin. Stringified so
    /// the msgpack payload stays platform-portable.
    pub cache_path: String,
}

/// Batch-ful-c (FUL-C / A-8) — steering vector compute failure payload.
///
/// Emitted on every non-success path: sidecar connection refused, HTTP error,
/// empty pairs rejected by the Python side, cache write failure, etc. The
/// `reason` string is a short classifier token (`"sidecar_unreachable"`,
/// `"http_error"`, `"zero_magnitude"`, `"cache_write_failed"`) suffixed with
/// the human detail where available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteeringComputeFailedPayload {
    pub role: String,
    pub reason: String,
}

/// Batch-ful-d (FUL-D / phase-3J S9) — MCP server connected payload.
///
/// Emitted by the dashboard `DefaultMcpBridge` after a successful JSON-RPC
/// `initialize` + `tools/list` round-trip. The `tool_count` is informational
/// only (for "mcp · connected · 7 tools" dashboard rows) and may be zero if
/// the server exposes resources but no tools.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerConnectedPayload {
    /// URL the bridge connected to (e.g. `http://localhost:8200` or
    /// `stdio://elai-mcp-stub`). Useful for the dashboard to display.
    pub server_url: String,
    /// Number of tools advertised by the server's `tools/list` response.
    pub tool_count: u32,
}

/// Batch-ful-d (FUL-D / phase-3J S9) — per-call MCP tool invocation row.
///
/// Emitted on every `tools/call` round-trip (success or error). `outcome` is
/// a short classifier token — `"success"`, `"error"`, `"security_blocked"`,
/// `"timeout"` — so the dashboard can colour the row without parsing the
/// detail string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpToolCalledPayload {
    pub tool_name: String,
    pub server_url: String,
    pub elapsed_ms: u32,
    pub outcome: String,
}

/// Batch-ful-d (FUL-D / phase-3J S9) — MCP server disconnected payload.
///
/// Emitted on every disconnect path: explicit user toggle, heartbeat
/// timeout, JSON-RPC protocol error, network drop. `reason` is a short
/// classifier (`"user_disabled"`, `"heartbeat_timeout"`, `"protocol_error"`,
/// `"connection_closed"`) optionally suffixed with the human detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerDisconnectedPayload {
    pub server_url: String,
    pub reason: String,
}

/// B-28 E-3 (SCHEMA_VERSION 31): focus-tunnel ModeSelector decision row.
///
/// Emitted by `task_runner::drive_task` immediately before the executor
/// dispatch at the `baby_step_executor.execute(exec_ctx)` callsite. One
/// row per baby step. Stays a flat record of strings/bool (no nested enums)
/// so the dashboard payload_json bridge can render rows without importing
/// focus_tunnel symbols.
///
/// `mode`, `rule_fired`, `exec_mode` are SSB wire-form strings produced via
/// `crate::focus_tunnel::generation_mode_str` / `selection_rule_str` /
/// `exec_mode_str` so the names stay locked. The reader-side aggregation
/// helper `crate::focus_tunnel::mode_distribution_for_payloads` parses
/// `mode` back into a [`crate::focus_tunnel::GenerationMode`] enum value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationModeSelectedPayload {
    /// `"baby_step_{id}"` stringified at emit time so the dashboard can
    /// correlate this row with other beacons keyed by the same step id.
    pub step_id: String,
    /// One of `"fim"` / `"diff"` / `"structured"` (snake_case wire form
    /// matching `GenerationMode` serde).
    pub mode: String,
    /// One of `"cloud_override"` / `"create_file_override"` /
    /// `"fim_all_conditions_met"` / `"multi_region_diff"` /
    /// `"structured_fallback"` / `"hinted_by_preferred_mode"`.
    pub rule_fired: String,
    /// Static label naming the rule body (`"rule_1_cloud_execution"`, ...).
    pub reason: String,
    /// The advisory `preferred_mode` hint present on the step at decision
    /// time, if any.
    pub preferred_was: Option<String>,
    /// Snapshot of `ModelFeatures.supports_fim` at decision time. Lets the
    /// auditor verify that Rule 3 never fires when supports_fim is false.
    pub supports_fim: bool,
    /// One of `"local"` / `"cloud"` / `"auto"` per
    /// `crate::cloud_mode::ExecMode`. Distinct from the
    /// `execution_mode_changed` LOCAL/CLOUD/HYBRID dashboard surface — this
    /// is the routing-side enum, not the settings-window one.
    pub exec_mode: String,
}

/// B-28 E-4 (SCHEMA_VERSION 32): focus-tunnel StreamingValidator early-term row.
///
/// Emitted by the post-E-7 WIRING phase whenever the synchronous
/// library-tier validator (`crate::focus_tunnel::streaming::StreamingValidator`)
/// returns `ValidationDecision::EarlyTerm` during a streaming generation
/// inside `run_fim_step` / `run_diff_step`. The validator itself is
/// I/O-free; the WIRING phase translates `EarlyTermWireForm` into this
/// payload and writes it via the SsbWriter.
///
/// Flat record of strings + numbers (no nested enums) so the dashboard
/// `payload_json` bridge can render rows without importing focus_tunnel
/// symbols. `reason` is the wire-form `EarlyTermReason` (lazy_pattern /
/// unbalanced_braces / output_too_long / deprecated_import /
/// unbounded_string_literal); `reason_detail` carries the structured detail
/// (pattern_id for LazyPattern; depth for UnbalancedBraces; etc).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamingEarlyTerminatedPayload {
    /// `"baby_step_{id}"` stringified at emit time so the dashboard can
    /// correlate this row with its originating GenerationModeSelected row
    /// and BabyStepSpec.
    pub step_id: String,
    /// One of `"fim"` / `"diff"` / `"structured"` (snake_case wire form
    /// matching `GenerationMode` serde). Captured at validator construction
    /// time so the row remains consistent if the validator outlives the
    /// step's selection record.
    pub mode: String,
    /// Wire-form `EarlyTermReason`. See `EarlyTermReason::ssb_wire_str`.
    pub reason: String,
    /// Structured detail string — `pattern_id` for LazyPattern;
    /// `depth=N` for UnbalancedBraces; `expected=N;actual=M` for
    /// OutputTooLong; `<name>-><replacement>` for DeprecatedImport;
    /// `opener_token=N` for UnboundedStringLiteral. None when the variant
    /// carries no structured detail.
    pub reason_detail: Option<String>,
    /// Token count at the moment of termination — proxy for "tokens we
    /// stopped emitting" telemetry.
    pub token_count_at_term: u32,
    /// Total tokens received from the sidecar before the terminate
    /// decision. Equal to `token_count_at_term` in the E-4 design (no
    /// in-flight buffer beyond the trigger); reserved for future overlap
    /// semantics where the validator could buffer ahead.
    pub total_tokens_received: u32,
    /// `"rust"` / `"python"` / `"generic"` — informational, drives the
    /// pattern set the validator applied.
    pub language_hint: String,
}

/// B-28 WIRING-1 (SCHEMA_VERSION 33): focus-tunnel PipelinePreAssembler row.
///
/// Emitted by the post-execute hook in `task_runner::drive_task` after
/// `PipelinePreAssembler::on_step_outcome(step_idx, passed, &target_files)`
/// fires. Carries the rolling pass-rate observed at decision time (in basis
/// points, 0..=10_000) and whether the pre-assembled context was discarded
/// because the next step touched different files.
///
/// Flat record of primitives (no nested enums) so the dashboard
/// `payload_json` bridge can render rows without importing focus_tunnel
/// symbols. `was_discarded == true` and `bytes_assembled == 0` mark a
/// canceled pre-assembly (the rolling-window discard path).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelinePreAssembledPayload {
    /// Zero-based index of the step whose pre-assembly produced this row.
    pub step_idx: u64,
    /// Rolling pass-rate at decision time, in basis points (0..=10_000).
    /// Stored as integer to avoid float drift across crates and SSB
    /// payload_json bridges.
    pub pass_rate_bp: u16,
    /// True when the pre-assembled context was discarded (rolling-window
    /// adaptive disable or per-step target-file change).
    pub was_discarded: bool,
    /// In-flight depth at decision time. PipelinePreAssembler caps at
    /// `MAX_DEPTH = 1` today; widened to u8 for future relaxation.
    pub depth_at_time: u8,
    /// Bytes assembled into the readiness payload (0 when discarded).
    pub bytes_assembled: u64,
}

/// B-28 WIRING-1 (SCHEMA_VERSION 34): focus-tunnel DependencyResolver row.
///
/// Emitted once per `ResolutionDecision` after the post-execute import-scan
/// + resolve cycle. Carries flat strings (decision and reason use String
/// mirrors instead of nested enums, matching the
/// `ExecutionModeChangedPayload` precedent — keeps the focus_tunnel ↔ ssb
/// dependency direction unidirectional).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyResolvedPayload {
    /// Wire-form import language: `"rust"` / `"python"` / `"typescript"` /
    /// `"go"` / `"javascript"` (matches `ImportLanguage::wire_str`).
    pub language: String,
    /// Dependency name as it appears in the import statement.
    pub name: String,
    /// Wire-form decision: `"auto_approve"` / `"queue_for_human"` /
    /// `"blocked"`.
    pub decision: String,
    /// Wire-form reason gating the decision (e.g. `"allowlist_match"`,
    /// `"not_allowlisted"`, `"anti_stale_waiver"`, `"registry_not_found"`).
    pub reason: String,
    /// Install command for AutoApprove decisions. `None` for QueueForHuman
    /// / Blocked or when the resolver did not propose an install plan.
    pub install_command: Option<String>,
}

/// B-28 WIRING-1 (SCHEMA_VERSION 35): focus-tunnel SpeculativeFileRouter row.
///
/// Emitted once per `RouteDecision::NeedsMicroStep` or
/// `RouteDecision::MicroStepCapExhausted`. `reason == "cap_exhausted"` is
/// the dashboard-side signal for the cap-exhausted branch; any other
/// `reason` value (e.g. `"undefined_symbol"`) maps to NeedsMicroStep
/// semantics. `count` is 0 on cap-exhausted rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicroStepCreatedPayload {
    /// Baby-step id of the step that triggered the micro-step (the parent).
    pub parent_step_id: u64,
    /// Path of the created micro-step target file.
    pub created_path: String,
    /// Wire-form symbol kind: `"Module"` / `"Struct"` / `"Function"` /
    /// `"Const"`. Mirrors `MicroStepType` as a String to keep the focus_tunnel
    /// ↔ ssb dep direction unidirectional.
    pub type_hint: String,
    /// Reason string. Common values: `"undefined_symbol"` (NeedsMicroStep),
    /// `"cap_exhausted"` (MicroStepCapExhausted), `"parse_skipped"`.
    pub reason: String,
    /// Number of micro-steps this row represents (1 for NeedsMicroStep,
    /// 0 for MicroStepCapExhausted).
    pub count: u8,
}

/// B-30 (SCHEMA_VERSION 36): one-shot LanguageRegistry status row.
///
/// Emitted once at DKG engine startup, after the per-grammar `catch_unwind`
/// registration cycle completes. The `loaded` list contains language IDs of
/// successfully initialized grammars in alphabetical order. The `failed` list
/// contains `(language_id, error_message)` pairs in alphabetical order of
/// language ID.
///
/// Both lists are deterministically sorted — derived from a `BTreeMap` source —
/// so downstream consumers can rely on stable ordering without sorting themselves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageGrammarStatusPayload {
    /// Language IDs successfully loaded, in alphabetical order.
    pub loaded: Vec<String>,
    /// Language IDs that failed to load, paired with the error message,
    /// in alphabetical order of language ID.
    pub failed: Vec<(String, String)>,
}

impl From<elai_knowledge::dkg::LanguageRegistryStatus> for LanguageGrammarStatusPayload {
    fn from(s: elai_knowledge::dkg::LanguageRegistryStatus) -> Self {
        Self { loaded: s.loaded, failed: s.failed }
    }
}

/// B-29 Phase 5 (SCHEMA_VERSION 37): payload for one symbol_* tool call.
///
/// Emitted by `translate_tool_request` in `production_agent_rpc.rs` for
/// `symbol_impact`, `symbol_flow`, `symbol_context`, and `symbol_list` tool IDs.
/// Consumed by the dashboard KnowledgeDrawer Impact Analysis panel and
/// the ActivityFeed via `SsbViewer::latest_action_of_type("symbol_query")`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolQueryPayload {
    /// The tool that was invoked ("symbol_impact" / "symbol_flow" / etc.).
    pub tool: String,
    /// The primary target string (file path or symbol name for impact/context;
    /// entrypoint_id for flow; optional file for list).
    pub target: String,
    /// Depth parameter from the request (0 if not applicable).
    pub hop_count: u32,
    /// Number of files in the result set (total_files for impact;
    /// symbols.len() for context/list; depth for flow).
    pub total_files: u32,
    /// Wall-clock latency for the traversal in milliseconds.
    pub latency_ms: u32,
}

// ---------------------------------------------------------------------------
// SsbPayload enum (typed wrapper over all action-type payloads)
// ---------------------------------------------------------------------------

/// Typed payload enum. Each variant maps 1:1 to an [`SsbActionType`] (31 total).
/// Serialized to msgpack via `rmp_serde` before storage.
///
/// All variants are schema-committed (77 total post-B-19A — was 76 in B-10,
/// 74 in Tier 4). B-19A adds `HookLatencyViolation`.
/// Variants without a current production emitter — `Claim`, `ModelSwap`,
/// `KnowledgeDegradation`, `IsolationMetric`, `BranchMerge`, `SpiralDetected`,
/// `SupervisionExhausted`, `InterventionRequest` — are forward-compat for
/// phases 3F / 4G / 4H / wire-antispiral / newphase-4G. Do not remove: on-disk
/// SSB files and the dashboard `payload_json` pipeline depend on the
/// discriminant set staying stable. `SpiralDetected` has a scheduled emitter
/// in Session wire-antispiral; `InterventionRequest` in Session newphase-4G
/// (AutonomousGate). B-10 added `ContextBudgetComputed` + `SsbFilteredByOverlap`
/// for the per-slot budget bar chart and 3E.5 entry-filter transparency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SsbPayload {
    FileWrite(FileWritePayload),
    FileRead(FileReadPayload),
    ToolResult(ToolResultPayload),
    TestOutcome(TestOutcomePayload),
    Finding(FindingPayload),
    Claim(ClaimPayload),
    Checkpoint(CheckpointPayload),
    ModelSwap(ModelSwapPayload),
    PhaseTransition(PhaseTransitionPayload),
    KnowledgeDegradation(KnowledgeDegradationPayload),
    IsolationMetric(IsolationMetricPayload),
    PlanUpdate(PlanUpdatePayload),
    ReplanRequest(ReplanRequestPayload),
    StubReference(StubReferencePayload),
    ReasoningTrace(ReasoningTracePayload),
    PhaseComplete(PhaseCompletePayload),
    MemoryMonitoring(MemoryMonitoringPayload),
    WorkloadProfile(WorkloadProfilePayload),
    DeathEvent(DeathEventPayload),
    RecoveryEvent(RecoveryEventPayload),
    SupervisionExhausted(SupervisionExhaustedPayload),
    SpiralDetected(SpiralDetectedPayload),
    ContextReduction(ContextReductionPayload),
    BranchMerge(BranchMergePayload),
    MistakeEntry(MistakeEntryPayload),
    ResearchStep(ResearchStepPayload),
    ResearchDecomposeLevel(ResearchDecomposeLevelPayload),
    WebResearchStep(WebResearchStepPayload),
    PreviewAvailable(PreviewAvailablePayload),
    InterventionRequest(InterventionRequestPayload),
    InterventionResponse(InterventionResponsePayload),
    TaskSubmitted(TaskSubmittedPayload),
    HookEvent(HookEventPayload),
    ProviderSignals(ProviderSignalsPayload),
    SearchTriggerFired(SearchTriggerFiredPayload),
    VramTierDetected(VramTierDetectedPayload),
    IndexingProgress(IndexingProgressPayload),
    DkgReady(DkgReadyPayload),
    TieredAnalysis(TieredAnalysisPayload),
    ShortCoderCompression(ShortCoderCompressionPayload),
    ToolDispatch(ToolDispatchPayload),
    ValidationTier(ValidationTierPayload),
    FailureClassified(FailureClassifiedPayload),
    GitCommit(GitCommitPayload),
    DairaTrace(DairaTracePayload),
    SystemProbe(SystemProbePayload),
    FeasibilityReport(FeasibilityReportPayload),
    NovelStepDetected(NovelStepDetectedPayload),
    StubRun(StubRunPayload),
    GateDecision(GateDecisionPayload),
    TaskSplit(TaskSplitPayload),
    EscalationFired(EscalationFiredPayload),
    ModeChanged(ModeChangedPayload),
    // Batch 3C (Phase 4H) — Layer 1A.
    ProposalGenerated(ProposalGeneratedPayload),
    ProposalApplied(ProposalAppliedPayload),
    ProposalRejected(ProposalRejectedPayload),
    MsiUpdate(MsiUpdatePayload),
    ConfigArchived(ConfigArchivedPayload),
    // Batch 4C — scattered A-step wiring.
    ProfileSelected(ProfileSelectedPayload),
    FsmTransition(FsmTransitionPayload),
    MicroReflection(MicroReflectionPayload),
    TelescopeCollapse(TelescopeCollapsePayload),
    ConfigWarning(ConfigWarningPayload),
    // Batch-fu — cross-cutting follow-up wiring.
    ConstraintsInjected(ConstraintsInjectedPayload),
    InterventionResolved(InterventionResolvedPayload),
    // Batch-ful-a — DkgQuerierAdapter hot-swap notification.
    DkgQuerierSwapped(DkgQuerierSwappedPayload),
    // Batch-ful-c — A-8 freshness steering vector lifecycle.
    SteeringComputed(SteeringComputedPayload),
    SteeringComputeFailed(SteeringComputeFailedPayload),
    // Batch-ful-d — external MCP server lifecycle + per-call rows.
    McpServerConnected(McpServerConnectedPayload),
    McpToolCalled(McpToolCalledPayload),
    McpServerDisconnected(McpServerDisconnectedPayload),
    // Tier 1 CHAT window — dashboard-originated message and card rows.
    ChatMessage(ChatMessagePayload),
    ChatResponse(ChatResponsePayload),
    // Tier 4 Settings reorg (dashboard-wiring-plan.txt §8) — execution-mode
    // selection from the Settings window (LOCAL / CLOUD / HYBRID).
    ExecutionModeChanged(ExecutionModeChangedPayload),
    // B-10 (DECISION-C) — per-dispatch ContextBudget snapshot for the
    // dashboard's per-slot bar chart, and 3E.5 SSB entry filtering summary.
    ContextBudgetComputed(ContextBudgetComputedPayload),
    SsbFilteredByOverlap(SsbFilteredByOverlapPayload),
    // B-19A (T10) — per-tier hook latency budget violation beacon.
    HookLatencyViolation(HookLatencyViolationPayload),
    // Track A.2 — synthesis path selection per phase_f_synthesize call.
    ResearchPathChosen(ResearchPathChosenPayload),
    // Track A.6 — grounding cache hit/miss event per ground_or_cached call.
    GroundingCacheEvent(GroundingCacheEventPayload),
    // B-4: circuit-breaker state transition for embedding/reranker models.
    CircuitBreakerStateChange(CircuitBreakerStateChangePayload),
    // B-7: bubble-up cascade, integration checkpoints, DAG repair, decomposition nodes.
    BubbleUpTyped(BubbleUpTypedPayload),
    IntegrationCheckpointSuccess(IntegrationCheckpointSuccessPayload),
    DagRepairRound(DagRepairRoundPayload),
    ConceptDecompositionNode(ConceptDecompositionNodePayload),
    // B-6: orchestration producer wiring.
    EvidenceGateEvaluated(EvidenceGateEvaluatedPayload),
    SupervisionEvent(SupervisionEventPayload),
    BspSuperstep(BspSuperstepPayload),
    AgentLifecycle(AgentLifecyclePayload),
    AntiSpiralHit(AntiSpiralHitPayload),
    // B-27: cloud-mode spend ledger + escalation marker.
    CloudSpend(CloudSpendPayload),
    EscalateToCloud(EscalateToCloudPayload),
    // B-23 Phase 4H Group A: self-mod beacons (Layer 1A growth + Layer 3 +
    // OSC + REG + EVO). Five batch-3C variants (proposal_*, msi_update,
    // config_archived) shipped earlier and live above.
    DataTableGrew(DataTableGrewPayload),
    L3Proposal(L3ProposalPayload),
    ConvergenceStagnant(ConvergenceStagnantPayload),
    ConfigOscillation(ConfigOscillationPayload),
    RegressionDetected(RegressionDetectedPayload),
    EvolutionWithheld(EvolutionWithheldPayload),
    // B-39 chat reflect pipeline: per-turn REFLECT stage beacons.
    ReflectTriaged(ReflectTriagedPayload),
    ReflectCandidate(ReflectCandidatePayload),
    ReflectAuditVerdict(ReflectAuditVerdictPayload),
    ReflectRefined(ReflectRefinedPayload),
    // B-24 Phase 4I Group C1: media validator beacons.
    MediaValidationStarted(MediaValidationStartedPayload),
    MediaTier1Complete(MediaTier1CompletePayload),
    MediaTier2Complete(MediaTier2CompletePayload),
    RenderComplete(RenderCompletePayload),
    VisualFeedbackIteration(VisualFeedbackIterationPayload),
    MediaHumanReviewRequired(MediaHumanReviewRequiredPayload),
    // B-28 E-3 (SCHEMA_VERSION 31): focus-tunnel ModeSelector decision row.
    GenerationModeSelected(GenerationModeSelectedPayload),
    // B-28 E-4 (SCHEMA_VERSION 32): focus-tunnel StreamingValidator early-term row.
    StreamingEarlyTerminated(StreamingEarlyTerminatedPayload),
    // B-28 WIRING-1 (SCHEMA_VERSION 33): pre-assembly readiness row.
    PipelinePreAssembled(PipelinePreAssembledPayload),
    // B-28 WIRING-1 (SCHEMA_VERSION 34): dependency resolver decision row.
    DependencyResolved(DependencyResolvedPayload),
    // B-28 WIRING-1 (SCHEMA_VERSION 35): speculative file-router micro-step row.
    MicroStepCreated(MicroStepCreatedPayload),
    // B-30 (SCHEMA_VERSION 36): one-shot language grammar registry status row.
    LanguageGrammarStatus(LanguageGrammarStatusPayload),
    // B-29 Phase 5 (SCHEMA_VERSION 37): one row per symbol_* tool call.
    SymbolQuery(SymbolQueryPayload),
    // B-32 (SCHEMA_VERSION 38): one row per minified file detected by the
    // adaptive character chunker.
    MinifiedFileChunked(MinifiedFileChunkedPayload),
    // B-31.5 (SCHEMA_VERSION 39): 5 deferred B-31 SSB variants — replaces
    // the `tracing::info!(target="ssb_pending", ...)` placeholders from
    // Cycle B.
    IndexCheckpoint(IndexCheckpointPayload),
    IndexResume(IndexResumePayload),
    IndexCancel(IndexCancelPayload),
    LockAcquired(LockAcquiredPayload),
    LockContested(LockContestedPayload),
    // B-33 follow-up (SCHEMA_VERSION 40): symbol-graph build event row.
    SymbolGraphUpdated(SymbolGraphUpdatedPayload),
    // B-40 (SCHEMA_VERSION 41): FRAMING substate beacons.
    IdeaFramingStarted(IdeaFramingStartedPayload),
    ProjectClassClassified(ProjectClassClassifiedPayload),
    CriticRoundComplete(CriticRoundCompletePayload),
    IdeaFramed(IdeaFramedPayload),
    TradeoffCardEmitted(TradeoffCardEmittedPayload),
    // B-34 (SCHEMA_VERSION 42): entry-point detection + artifact pipeline beacons.
    EntryPointsDetected(EntryPointsDetectedPayload),
    ArtifactsIndexed(ArtifactsIndexedPayload),
    ArtifactSearched(ArtifactSearchedPayload),
    ArtifactStale(ArtifactStalePayload),
    // B-38 (SCHEMA_VERSION 43): ambiguity gate beacons.
    AmbiguityScored(AmbiguityScoredPayload),
    AmbiguityFloorFailed(AmbiguityFloorFailedPayload),
    AmbiguityStreakAdvanced(AmbiguityStreakAdvancedPayload),
    // B-42 (SCHEMA_VERSION 44): cosmos tier knowledge worker beacons.
    CosmosRefreshStarted(CosmosRefreshStartedPayload),
    CosmosArtifactUpdated(CosmosArtifactUpdatedPayload),
    CosmosArtifactDecayed(CosmosArtifactDecayedPayload),
    CosmosGroundingUsed(CosmosGroundingUsedPayload),
    // B-41 (SCHEMA_VERSION 45): per-level auditor mode beacons.
    PerLevelAuditStarted(PerLevelAuditStartedPayload),
    PerLevelAuditVerdict(PerLevelAuditVerdictPayload),
    PerLevelAuditCascade(PerLevelAuditCascadePayload),
    // B-37C (SCHEMA_VERSION 46): multi-runtime backend beacons.
    RuntimeFallback(RuntimeFallbackPayload),
    RuntimeHealthChanged(RuntimeHealthChangedPayload),
    CliSubprocessSpawned(CliSubprocessSpawnedPayload),
    CliSubprocessKilled(CliSubprocessKilledPayload),
    // C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
    GrcStepEmitted(GrcStepEmittedPayload),
    GroundingSweepResult(GroundingSweepResultPayload),
    CsqEvaluated(CsqEvaluatedPayload),
    AuditorRubberStampRejected(AuditorRubberStampRejectedPayload),
    HydeQuery(HydeQueryPayload),
    VectorCacheHitRate(VectorCacheHitRatePayload),
    QueryTrace(QueryTracePayload),
    // C-2 (SCHEMA_VERSION 48): orchestration + cosmos + stagnation beacons.
    PhaseNodeReady(PhaseNodeReadyPayload),
    StepNodeReady(StepNodeReadyPayload),
    BabyStepReady(BabyStepReadyPayload),
    DagRepairEscalated(DagRepairEscalatedPayload),
    AgentStatusChanged(AgentStatusChangedPayload),
    StagnationSpinningDetected(StagnationSpinningDetectedPayload),
    StagnationOscillationDetected(StagnationOscillationDetectedPayload),
    LateralThinkingActivated(LateralThinkingActivatedPayload),
    GoalDriftWarning(GoalDriftWarningPayload),
    WatchdogTimeout(WatchdogTimeoutPayload),
    // C-7 (SCHEMA_VERSION 49): reviewer verdict beacon.
    ChangeDecision(ChangeDecisionPayload),
    // C-8 (SCHEMA_VERSION 50): search-mode cross-process bridge.
    SearchModeChanged(SearchModeChangedPayload),
    // C-5 (SCHEMA_VERSION 51): privacy-redaction event.
    RedactionApplied(RedactionAppliedPayload),
    // C-10 (SCHEMA_VERSION 52): anti-spiral lateral-thinking + goal-drift Block.
    LateralThinkingSucceeded(LateralThinkingSucceededPayload),
    LateralThinkingExhausted(LateralThinkingExhaustedPayload),
    GoalDriftBlock(GoalDriftBlockPayload),
    // C-13 (SCHEMA_VERSION 53): WIP commit squash completion.
    WipSquashComplete(WipSquashCompletePayload),
    // C-22 (SCHEMA_VERSION 54): feasibility-probe completeness beacons.
    AlternativeExplored(AlternativeExploredPayload),
    FeasibilityEscalated(FeasibilityEscalatedPayload),
    // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
    BranchForked(BranchForkedPayload),
    BranchMerged(BranchMergedPayload),
    BranchClosed(BranchClosedPayload),
    // C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
    // + background-task beacons. Payload structs live below.
    PlanSkeleton(PlanSkeletonPayload),
    CompiledPipelineFired(CompiledPipelineFiredPayload),
    TemplateCrystallized(TemplateCrystallizedPayload),
    BackgroundTaskStarted(BackgroundTaskStartedPayload),
    BackgroundTaskComplete(BackgroundTaskCompletePayload),
    DecompileTriggered(DecompileTriggeredPayload),
    // C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
    FutilityDetected(FutilityDetectedPayload),
    FutilityOutputStagnation(FutilityOutputStagnationPayload),
    // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons. Payload
    // structs live below near the RedactionAppliedPayload region.
    WiringEvidence(WiringEvidencePayload),
    WiringViolation(WiringViolationPayload),
    DeterminismDrift(DeterminismDriftPayload),
    // C-28 (SCHEMA_VERSION 60): Testing-trust T-1 mutation testing.
    MutationSurvived(MutationSurvivedPayload),
    // C-29 (SCHEMA_VERSION 61): Testing-trust T-2 (IntentPair) + T-3
    // (Metamorphic). Payload structs live below near MutationSurvivedPayload.
    IntentConfirmed(IntentConfirmedPayload),
    MrViolated(MrViolatedPayload),
    // C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode pipeline beacons.
    SurgicalPlanCreated(SurgicalPlanCreatedPayload),
    SurgicalLocWarning(SurgicalLocWarningPayload),
    OracleCacheHit(OracleCacheHitPayload),
    OracleCacheMiss(OracleCacheMissPayload),
    ProjectDetectionSignal(ProjectDetectionSignalPayload),
    // C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 beacons. Payload
    // structs live below the C-30 region.
    SkeletonGenerated(SkeletonGeneratedPayload),
    SkeletonFlushed(SkeletonFlushedPayload),
    MiniCriticVerdict(MiniCriticVerdictPayload),
    StructuralRetryStageAttempt(StructuralRetryStageAttemptPayload),
    BridgeFileGenerated(BridgeFileGeneratedPayload),
    // D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle.
    ArchiveEntryAdded(ArchiveEntryAddedPayload),
    ArchiveEntryRetired(ArchiveEntryRetiredPayload),
    TemplatePromoted(TemplatePromotedPayload),
    TemplateCandidateRejected(TemplateCandidateRejectedPayload),
    // E-5 (SCHEMA_VERSION 65): memory_facts bi-temporal lifecycle beacons.
    MemoryFactExtracted(MemoryFactExtractedPayload),
    MemoryFactSuperseded(MemoryFactSupersededPayload),
    // D-2 (SCHEMA_VERSION 66): PPM verifier beacons.
    PpmScored(PpmScoredPayload),
    PpmGatingFailed(PpmGatingFailedPayload),
    // E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
    ChatReflectionCompleted(ChatReflectionCompletedPayload),
    MemoryEvolutionRun(MemoryEvolutionRunPayload),
    // E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
    // producer + conflict beacons.
    MemoryNoteInserted(MemoryNoteInsertedPayload),
    MemoryNoteEvolved(MemoryNoteEvolvedPayload),
    MemoryConflictDetected(MemoryConflictDetectedPayload),
    CoreBlockReplaced(CoreBlockReplacedPayload),
    // D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator.
    VerifierConsensusPass(VerifierConsensusPassPayload),
    VerifierConsensusFail(VerifierConsensusFailPayload),
    // D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
    TripwireFired(TripwireFiredPayload),
    TemplateAutoReverted(TemplateAutoRevertedPayload),
    // D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark runner.
    PrmBenchRunCompleted(PrmBenchRunCompletedPayload),
    // D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
    BaiArmUpdated(BaiArmUpdatedPayload),
    // D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
    HeldOutTaskAdded(HeldOutTaskAddedPayload),
    HeldOutTaskUsed(HeldOutTaskUsedPayload),
    // D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
    CanaryRunCompleted(CanaryRunCompletedPayload),
    CanaryFamilyEvaluated(CanaryFamilyEvaluatedPayload),
    LockedKeysAuditRun(LockedKeysAuditRunPayload),
    LockedKeysViolationDetected(LockedKeysViolationDetectedPayload),
    // E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
    TaskAdvanced(TaskAdvancedPayload),
    AgentlessShortcutTaken(AgentlessShortcutTakenPayload),
    GoalDriftFired(GoalDriftFiredPayload),
    // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
    ClaimRouted(ClaimRoutedPayload),
    // D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
    ModeRoutingBenchRunCompleted(ModeRoutingBenchRunCompletedPayload),
    // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
    CheckerResultEmitted(CheckerResultEmittedPayload),
    // D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
    AdversaryGenerated(AdversaryGeneratedPayload),
    ConsensusReevaluatedWithCounter(ConsensusReevaluatedWithCounterPayload),
    // D-12 (SCHEMA_VERSION 81): OperationalizationCheck baby-step
    // predicate dispatch beacon.
    BabyStepDispatched(BabyStepDispatchedPayload),
    // D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal calibration beacons.
    ConformalTauComputed(ConformalTauComputedPayload),
    ConformalRefusalDecided(ConformalRefusalDecidedPayload),
    // D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass beacon.
    NoveltyClassified(NoveltyClassifiedPayload),
    // D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate beacon.
    ObservableEffectVerified(ObservableEffectVerifiedPayload),
    // D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
    RefusalRateSnapshot(RefusalRateSnapshotPayload),
    RefusalSlaBreach(RefusalSlaBreachPayload),
    // D-22 (SCHEMA_VERSION 86): conformal-calibration validation beacon.
    ConformalValidationCompleted(ConformalValidationCompletedPayload),
    // D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
    ArcBenchmarkCompleted(ARCBenchmarkCompletedPayload),
}

impl SsbPayload {
    /// Returns the [`SsbActionType`] corresponding to this payload variant.
    /// This is the single source of truth for the discriminant→action_type mapping.
    pub fn action_type(&self) -> SsbActionType {
        match self {
            Self::FileWrite(_) => SsbActionType::FileWrite,
            Self::FileRead(_) => SsbActionType::FileRead,
            Self::ToolResult(_) => SsbActionType::ToolResult,
            Self::TestOutcome(_) => SsbActionType::TestOutcome,
            Self::Finding(_) => SsbActionType::Finding,
            Self::Claim(_) => SsbActionType::Claim,
            Self::Checkpoint(_) => SsbActionType::Checkpoint,
            Self::ModelSwap(_) => SsbActionType::ModelSwap,
            Self::PhaseTransition(_) => SsbActionType::PhaseTransition,
            Self::KnowledgeDegradation(_) => SsbActionType::KnowledgeDegradation,
            Self::IsolationMetric(_) => SsbActionType::IsolationMetric,
            Self::PlanUpdate(_) => SsbActionType::PlanUpdate,
            Self::ReplanRequest(_) => SsbActionType::ReplanRequest,
            Self::StubReference(_) => SsbActionType::StubReference,
            Self::ReasoningTrace(_) => SsbActionType::ReasoningTrace,
            Self::PhaseComplete(_) => SsbActionType::PhaseComplete,
            Self::MemoryMonitoring(_) => SsbActionType::MemoryMonitoring,
            Self::WorkloadProfile(_) => SsbActionType::WorkloadProfile,
            Self::DeathEvent(_) => SsbActionType::DeathEvent,
            Self::RecoveryEvent(_) => SsbActionType::RecoveryEvent,
            Self::SupervisionExhausted(_) => SsbActionType::SupervisionExhausted,
            Self::SpiralDetected(_) => SsbActionType::SpiralDetected,
            Self::ContextReduction(_) => SsbActionType::ContextReduction,
            Self::BranchMerge(_) => SsbActionType::BranchMerge,
            Self::MistakeEntry(_) => SsbActionType::MistakeEntry,
            Self::ResearchStep(_) => SsbActionType::ResearchStep,
            Self::ResearchDecomposeLevel(_) => SsbActionType::ResearchDecomposeLevel,
            Self::WebResearchStep(_) => SsbActionType::WebResearchStep,
            Self::PreviewAvailable(_) => SsbActionType::PreviewAvailable,
            Self::InterventionRequest(_) => SsbActionType::InterventionRequest,
            Self::InterventionResponse(_) => SsbActionType::InterventionResponse,
            Self::TaskSubmitted(_) => SsbActionType::TaskSubmitted,
            Self::HookEvent(_) => SsbActionType::HookEvent,
            Self::ProviderSignals(_) => SsbActionType::ProviderSignals,
            Self::SearchTriggerFired(_) => SsbActionType::SearchTriggerFired,
            Self::VramTierDetected(_) => SsbActionType::VramTierDetected,
            Self::IndexingProgress(_) => SsbActionType::IndexingProgress,
            Self::DkgReady(_) => SsbActionType::DkgReady,
            Self::TieredAnalysis(_) => SsbActionType::TieredAnalysis,
            Self::ShortCoderCompression(_) => SsbActionType::ShortCoderCompression,
            Self::ToolDispatch(_) => SsbActionType::ToolDispatch,
            Self::ValidationTier(_) => SsbActionType::ValidationTier,
            Self::FailureClassified(_) => SsbActionType::FailureClassified,
            Self::GitCommit(_) => SsbActionType::GitCommit,
            Self::DairaTrace(_) => SsbActionType::DairaTrace,
            Self::SystemProbe(_) => SsbActionType::SystemProbe,
            Self::FeasibilityReport(_) => SsbActionType::FeasibilityReport,
            Self::NovelStepDetected(_) => SsbActionType::NovelStepDetected,
            Self::StubRun(_) => SsbActionType::StubRun,
            Self::GateDecision(_) => SsbActionType::GateDecision,
            Self::TaskSplit(_) => SsbActionType::TaskSplit,
            Self::EscalationFired(_) => SsbActionType::EscalationFired,
            Self::ModeChanged(_) => SsbActionType::ModeChanged,
            Self::ProposalGenerated(_) => SsbActionType::ProposalGenerated,
            Self::ProposalApplied(_) => SsbActionType::ProposalApplied,
            Self::ProposalRejected(_) => SsbActionType::ProposalRejected,
            Self::MsiUpdate(_) => SsbActionType::MsiUpdate,
            Self::ConfigArchived(_) => SsbActionType::ConfigArchived,
            Self::ProfileSelected(_) => SsbActionType::ProfileSelected,
            Self::FsmTransition(_) => SsbActionType::FsmTransition,
            Self::MicroReflection(_) => SsbActionType::MicroReflection,
            Self::TelescopeCollapse(_) => SsbActionType::TelescopeCollapse,
            Self::ConfigWarning(_) => SsbActionType::ConfigWarning,
            Self::ConstraintsInjected(_) => SsbActionType::ConstraintsInjected,
            Self::InterventionResolved(_) => SsbActionType::InterventionResolved,
            Self::DkgQuerierSwapped(_) => SsbActionType::DkgQuerierSwapped,
            Self::SteeringComputed(_) => SsbActionType::SteeringComputed,
            Self::SteeringComputeFailed(_) => SsbActionType::SteeringComputeFailed,
            Self::McpServerConnected(_) => SsbActionType::McpServerConnected,
            Self::McpToolCalled(_) => SsbActionType::McpToolCalled,
            Self::McpServerDisconnected(_) => SsbActionType::McpServerDisconnected,
            Self::ChatMessage(_) => SsbActionType::ChatMessage,
            Self::ChatResponse(_) => SsbActionType::ChatResponse,
            Self::ExecutionModeChanged(_) => SsbActionType::ExecutionModeChanged,
            Self::ContextBudgetComputed(_) => SsbActionType::ContextBudgetComputed,
            Self::SsbFilteredByOverlap(_) => SsbActionType::SsbFilteredByOverlap,
            Self::HookLatencyViolation(_) => SsbActionType::HookLatencyViolation,
            Self::ResearchPathChosen(_) => SsbActionType::ResearchPathChosen,
            Self::GroundingCacheEvent(_) => SsbActionType::GroundingCacheEvent,
            Self::CircuitBreakerStateChange(_) => SsbActionType::CircuitBreakerStateChange,
            Self::BubbleUpTyped(_) => SsbActionType::BubbleUpTyped,
            Self::IntegrationCheckpointSuccess(_) => SsbActionType::IntegrationCheckpointSuccess,
            Self::DagRepairRound(_) => SsbActionType::DagRepairRound,
            Self::ConceptDecompositionNode(_) => SsbActionType::ConceptDecompositionNode,
            // B-6: orchestration producer wiring.
            Self::EvidenceGateEvaluated(_) => SsbActionType::EvidenceGateEvaluated,
            Self::SupervisionEvent(_) => SsbActionType::SupervisionEvent,
            Self::BspSuperstep(_) => SsbActionType::BspSuperstep,
            Self::AgentLifecycle(_) => SsbActionType::AgentLifecycle,
            Self::AntiSpiralHit(_) => SsbActionType::AntiSpiralHit,
            Self::CloudSpend(_) => SsbActionType::CloudSpend,
            Self::EscalateToCloud(_) => SsbActionType::EscalateToCloud,
            // B-23 Phase 4H Group A.
            Self::DataTableGrew(_) => SsbActionType::DataTableGrew,
            Self::L3Proposal(_) => SsbActionType::L3Proposal,
            Self::ConvergenceStagnant(_) => SsbActionType::ConvergenceStagnant,
            Self::ConfigOscillation(_) => SsbActionType::ConfigOscillation,
            Self::RegressionDetected(_) => SsbActionType::RegressionDetected,
            Self::EvolutionWithheld(_) => SsbActionType::EvolutionWithheld,
            // B-39 chat reflect pipeline.
            Self::ReflectTriaged(_) => SsbActionType::ReflectTriaged,
            Self::ReflectCandidate(_) => SsbActionType::ReflectCandidate,
            Self::ReflectAuditVerdict(_) => SsbActionType::ReflectAuditVerdict,
            Self::ReflectRefined(_) => SsbActionType::ReflectRefined,
            // B-24 Phase 4I Group C1: media validator beacons.
            Self::MediaValidationStarted(_) => SsbActionType::MediaValidationStarted,
            Self::MediaTier1Complete(_) => SsbActionType::MediaTier1Complete,
            Self::MediaTier2Complete(_) => SsbActionType::MediaTier2Complete,
            Self::RenderComplete(_) => SsbActionType::RenderComplete,
            Self::VisualFeedbackIteration(_) => SsbActionType::VisualFeedbackIteration,
            Self::MediaHumanReviewRequired(_) => SsbActionType::MediaHumanReviewRequired,
            // B-28 E-3.
            Self::GenerationModeSelected(_) => SsbActionType::GenerationModeSelected,
            // B-28 E-4.
            Self::StreamingEarlyTerminated(_) => SsbActionType::StreamingEarlyTerminated,
            // B-28 WIRING-1 (3 new variants).
            Self::PipelinePreAssembled(_) => SsbActionType::PipelinePreAssembled,
            Self::DependencyResolved(_) => SsbActionType::DependencyResolved,
            Self::MicroStepCreated(_) => SsbActionType::MicroStepCreated,
            // B-30 (SCHEMA_VERSION 36): language grammar registry status.
            Self::LanguageGrammarStatus(_) => SsbActionType::LanguageGrammarStatus,
            // B-29 Phase 5 (SCHEMA_VERSION 37): symbol-graph tool call row.
            Self::SymbolQuery(_) => SsbActionType::SymbolQuery,
            // B-32 (SCHEMA_VERSION 38): minified-file chunking row.
            Self::MinifiedFileChunked(_) => SsbActionType::MinifiedFileChunked,
            // B-31.5 (SCHEMA_VERSION 39): 5 deferred B-31 SSB variants.
            Self::IndexCheckpoint(_) => SsbActionType::IndexCheckpoint,
            Self::IndexResume(_) => SsbActionType::IndexResume,
            Self::IndexCancel(_) => SsbActionType::IndexCancel,
            Self::LockAcquired(_) => SsbActionType::LockAcquired,
            Self::LockContested(_) => SsbActionType::LockContested,
            // B-33 follow-up (SCHEMA_VERSION 40): symbol-graph build event row.
            Self::SymbolGraphUpdated(_) => SsbActionType::SymbolGraphUpdated,
            // B-40 (SCHEMA_VERSION 41): FRAMING substate beacons.
            Self::IdeaFramingStarted(_) => SsbActionType::IdeaFramingStarted,
            Self::ProjectClassClassified(_) => SsbActionType::ProjectClassClassified,
            Self::CriticRoundComplete(_) => SsbActionType::CriticRoundComplete,
            Self::IdeaFramed(_) => SsbActionType::IdeaFramed,
            Self::TradeoffCardEmitted(_) => SsbActionType::TradeoffCardEmitted,
            // B-34 (SCHEMA_VERSION 42): entry-point detection + artifact pipeline beacons.
            Self::EntryPointsDetected(_) => SsbActionType::EntryPointsDetected,
            Self::ArtifactsIndexed(_) => SsbActionType::ArtifactsIndexed,
            Self::ArtifactSearched(_) => SsbActionType::ArtifactSearched,
            Self::ArtifactStale(_) => SsbActionType::ArtifactStale,
            // B-38 (SCHEMA_VERSION 43): ambiguity gate beacons.
            Self::AmbiguityScored(_) => SsbActionType::AmbiguityScored,
            Self::AmbiguityFloorFailed(_) => SsbActionType::AmbiguityFloorFailed,
            Self::AmbiguityStreakAdvanced(_) => SsbActionType::AmbiguityStreakAdvanced,
            // B-42 (SCHEMA_VERSION 44): cosmos tier knowledge worker beacons.
            Self::CosmosRefreshStarted(_) => SsbActionType::CosmosRefreshStarted,
            Self::CosmosArtifactUpdated(_) => SsbActionType::CosmosArtifactUpdated,
            Self::CosmosArtifactDecayed(_) => SsbActionType::CosmosArtifactDecayed,
            Self::CosmosGroundingUsed(_) => SsbActionType::CosmosGroundingUsed,
            // B-41 (SCHEMA_VERSION 45): per-level auditor mode beacons.
            Self::PerLevelAuditStarted(_) => SsbActionType::PerLevelAuditStarted,
            Self::PerLevelAuditVerdict(_) => SsbActionType::PerLevelAuditVerdict,
            Self::PerLevelAuditCascade(_) => SsbActionType::PerLevelAuditCascade,
            // B-37C (SCHEMA_VERSION 46): multi-runtime backend beacons.
            Self::RuntimeFallback(_) => SsbActionType::RuntimeFallback,
            Self::RuntimeHealthChanged(_) => SsbActionType::RuntimeHealthChanged,
            Self::CliSubprocessSpawned(_) => SsbActionType::CliSubprocessSpawned,
            Self::CliSubprocessKilled(_) => SsbActionType::CliSubprocessKilled,
            // C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
            Self::GrcStepEmitted(_) => SsbActionType::GrcStepEmitted,
            Self::GroundingSweepResult(_) => SsbActionType::GroundingSweepResult,
            Self::CsqEvaluated(_) => SsbActionType::CsqEvaluated,
            Self::AuditorRubberStampRejected(_) => SsbActionType::AuditorRubberStampRejected,
            Self::HydeQuery(_) => SsbActionType::HydeQuery,
            Self::VectorCacheHitRate(_) => SsbActionType::VectorCacheHitRate,
            Self::QueryTrace(_) => SsbActionType::QueryTrace,
            // C-2 (SCHEMA_VERSION 48): orchestration + cosmos + stagnation beacons.
            Self::PhaseNodeReady(_) => SsbActionType::PhaseNodeReady,
            Self::StepNodeReady(_) => SsbActionType::StepNodeReady,
            Self::BabyStepReady(_) => SsbActionType::BabyStepReady,
            Self::DagRepairEscalated(_) => SsbActionType::DagRepairEscalated,
            Self::AgentStatusChanged(_) => SsbActionType::AgentStatusChanged,
            Self::StagnationSpinningDetected(_) => SsbActionType::StagnationSpinningDetected,
            Self::StagnationOscillationDetected(_) => SsbActionType::StagnationOscillationDetected,
            Self::LateralThinkingActivated(_) => SsbActionType::LateralThinkingActivated,
            Self::GoalDriftWarning(_) => SsbActionType::GoalDriftWarning,
            Self::WatchdogTimeout(_) => SsbActionType::WatchdogTimeout,
            // C-7 (SCHEMA_VERSION 49): reviewer verdict beacon.
            Self::ChangeDecision(_) => SsbActionType::ChangeDecision,
            // C-8 (SCHEMA_VERSION 50): search-mode cross-process bridge.
            Self::SearchModeChanged(_) => SsbActionType::SearchModeChanged,
            // C-5 (SCHEMA_VERSION 51): privacy-redaction event.
            Self::RedactionApplied(_) => SsbActionType::RedactionApplied,
            // C-10 (SCHEMA_VERSION 52): anti-spiral lateral-thinking + goal-drift Block.
            Self::LateralThinkingSucceeded(_) => SsbActionType::LateralThinkingSucceeded,
            Self::LateralThinkingExhausted(_) => SsbActionType::LateralThinkingExhausted,
            Self::GoalDriftBlock(_) => SsbActionType::GoalDriftBlock,
            // C-13 (SCHEMA_VERSION 53): WIP commit squash completion.
            Self::WipSquashComplete(_) => SsbActionType::WipSquashComplete,
            // C-22 (SCHEMA_VERSION 54): feasibility-probe completeness beacons.
            Self::AlternativeExplored(_) => SsbActionType::AlternativeExplored,
            Self::FeasibilityEscalated(_) => SsbActionType::FeasibilityEscalated,
            // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
            Self::BranchForked(_) => SsbActionType::BranchForked,
            Self::BranchMerged(_) => SsbActionType::BranchMerged,
            Self::BranchClosed(_) => SsbActionType::BranchClosed,
            // C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
            // + background-task beacons.
            Self::PlanSkeleton(_) => SsbActionType::PlanSkeleton,
            Self::CompiledPipelineFired(_) => SsbActionType::CompiledPipelineFired,
            Self::TemplateCrystallized(_) => SsbActionType::TemplateCrystallized,
            Self::BackgroundTaskStarted(_) => SsbActionType::BackgroundTaskStarted,
            Self::BackgroundTaskComplete(_) => SsbActionType::BackgroundTaskComplete,
            Self::DecompileTriggered(_) => SsbActionType::DecompileTriggered,
            // C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
            Self::FutilityDetected(_) => SsbActionType::FutilityDetected,
            Self::FutilityOutputStagnation(_) => SsbActionType::FutilityOutputStagnation,
            // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons.
            Self::WiringEvidence(_) => SsbActionType::WiringEvidence,
            Self::WiringViolation(_) => SsbActionType::WiringViolation,
            Self::DeterminismDrift(_) => SsbActionType::DeterminismDrift,
            // C-28 (SCHEMA_VERSION 60): Testing-trust T-1 mutation testing.
            Self::MutationSurvived(_) => SsbActionType::MutationSurvived,
            // C-29 (SCHEMA_VERSION 61): Testing-trust T-2 + T-3 beacons.
            Self::IntentConfirmed(_) => SsbActionType::IntentConfirmed,
            Self::MrViolated(_) => SsbActionType::MrViolated,
            // C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode beacons.
            Self::SurgicalPlanCreated(_) => SsbActionType::SurgicalPlanCreated,
            Self::SurgicalLocWarning(_) => SsbActionType::SurgicalLocWarning,
            Self::OracleCacheHit(_) => SsbActionType::OracleCacheHit,
            Self::OracleCacheMiss(_) => SsbActionType::OracleCacheMiss,
            Self::ProjectDetectionSignal(_) => SsbActionType::ProjectDetectionSignal,
            // C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 beacons.
            Self::SkeletonGenerated(_) => SsbActionType::SkeletonGenerated,
            Self::SkeletonFlushed(_) => SsbActionType::SkeletonFlushed,
            Self::MiniCriticVerdict(_) => SsbActionType::MiniCriticVerdict,
            Self::StructuralRetryStageAttempt(_) => SsbActionType::StructuralRetryStageAttempt,
            Self::BridgeFileGenerated(_) => SsbActionType::BridgeFileGenerated,
            // D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle.
            Self::ArchiveEntryAdded(_) => SsbActionType::ArchiveEntryAdded,
            Self::ArchiveEntryRetired(_) => SsbActionType::ArchiveEntryRetired,
            Self::TemplatePromoted(_) => SsbActionType::TemplatePromoted,
            Self::TemplateCandidateRejected(_) => SsbActionType::TemplateCandidateRejected,
            // E-5 (SCHEMA_VERSION 65): memory_facts bi-temporal lifecycle beacons.
            Self::MemoryFactExtracted(_) => SsbActionType::MemoryFactExtracted,
            Self::MemoryFactSuperseded(_) => SsbActionType::MemoryFactSuperseded,
            // D-2 (SCHEMA_VERSION 66): PPM verifier beacons.
            Self::PpmScored(_) => SsbActionType::PpmScored,
            Self::PpmGatingFailed(_) => SsbActionType::PpmGatingFailed,
            // E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
            Self::ChatReflectionCompleted(_) => SsbActionType::ChatReflectionCompleted,
            Self::MemoryEvolutionRun(_) => SsbActionType::MemoryEvolutionRun,
            // E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
            // producer + conflict beacons.
            Self::MemoryNoteInserted(_) => SsbActionType::MemoryNoteInserted,
            Self::MemoryNoteEvolved(_) => SsbActionType::MemoryNoteEvolved,
            Self::MemoryConflictDetected(_) => SsbActionType::MemoryConflictDetected,
            Self::CoreBlockReplaced(_) => SsbActionType::CoreBlockReplaced,
            // D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator.
            Self::VerifierConsensusPass(_) => SsbActionType::VerifierConsensusPass,
            Self::VerifierConsensusFail(_) => SsbActionType::VerifierConsensusFail,
            // D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
            Self::TripwireFired(_) => SsbActionType::TripwireFired,
            Self::TemplateAutoReverted(_) => SsbActionType::TemplateAutoReverted,
            // D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark beacon.
            Self::PrmBenchRunCompleted(_) => SsbActionType::PrmBenchRunCompleted,
            // D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
            Self::BaiArmUpdated(_) => SsbActionType::BaiArmUpdated,
            // D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
            Self::HeldOutTaskAdded(_) => SsbActionType::HeldOutTaskAdded,
            Self::HeldOutTaskUsed(_) => SsbActionType::HeldOutTaskUsed,
            // D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
            Self::CanaryRunCompleted(_) => SsbActionType::CanaryRunCompleted,
            Self::CanaryFamilyEvaluated(_) => SsbActionType::CanaryFamilyEvaluated,
            Self::LockedKeysAuditRun(_) => SsbActionType::LockedKeysAuditRun,
            Self::LockedKeysViolationDetected(_) => {
                SsbActionType::LockedKeysViolationDetected
            }
            // E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
            Self::TaskAdvanced(_) => SsbActionType::TaskAdvanced,
            Self::AgentlessShortcutTaken(_) => SsbActionType::AgentlessShortcutTaken,
            Self::GoalDriftFired(_) => SsbActionType::GoalDriftFired,
            // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
            Self::ClaimRouted(_) => SsbActionType::ClaimRouted,
            // D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
            Self::ModeRoutingBenchRunCompleted(_) => {
                SsbActionType::ModeRoutingBenchRunCompleted
            }
            // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
            Self::CheckerResultEmitted(_) => SsbActionType::CheckerResultEmitted,
            // D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
            Self::AdversaryGenerated(_) => SsbActionType::AdversaryGenerated,
            Self::ConsensusReevaluatedWithCounter(_) => {
                SsbActionType::ConsensusReevaluatedWithCounter
            }
            // D-12 (SCHEMA_VERSION 81): OperationalizationCheck beacon.
            Self::BabyStepDispatched(_) => SsbActionType::BabyStepDispatched,
            // D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal beacons.
            Self::ConformalTauComputed(_) => SsbActionType::ConformalTauComputed,
            Self::ConformalRefusalDecided(_) => SsbActionType::ConformalRefusalDecided,
            // D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass beacon.
            Self::NoveltyClassified(_) => SsbActionType::NoveltyClassified,
            // D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate beacon.
            Self::ObservableEffectVerified(_) => SsbActionType::ObservableEffectVerified,
            // D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
            Self::RefusalRateSnapshot(_) => SsbActionType::RefusalRateSnapshot,
            Self::RefusalSlaBreach(_) => SsbActionType::RefusalSlaBreach,
            // D-22 (SCHEMA_VERSION 86): conformal-calibration validation beacon.
            Self::ConformalValidationCompleted(_) => {
                SsbActionType::ConformalValidationCompleted
            }
            // D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
            Self::ArcBenchmarkCompleted(_) => SsbActionType::ArcBenchmarkCompleted,
        }
    }
}

impl Ssb {
    /// Current wall-clock time in milliseconds since UNIX_EPOCH, sourced from
    /// the injected [`ClockProvider`]. Used by callers (e.g. the dashboard
    /// process) that must avoid raw `std::time::SystemTime` per the entropy
    /// quarantine rule.
    ///
    /// Saturation semantics (intentional, not silent error swallowing):
    /// * `duration_since(UNIX_EPOCH)` only fails on a system clock set before
    ///   1970-01-01; we treat that as t=0 since SSB timestamps are advisory
    ///   and we do not want a panic at the IPC boundary.
    /// * `i64::try_from(u128)` only fails past year 292278994; we saturate
    ///   to `i64::MAX` because no caller in 2026 should ever see overflow,
    ///   and a saturated max is harmless for monotonic comparisons.
    pub fn now_ms_via_clock(clock: &Arc<dyn ClockProvider>) -> i64 {
        let ms_u128 = clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis();
        i64::try_from(ms_u128).unwrap_or(i64::MAX)
    }
}

// ---------------------------------------------------------------------------
// SsbTraceable trait (orchestrator-local, no cross-crate dep)
// ---------------------------------------------------------------------------

/// Trait for types that can produce an [`SsbPayload`] for recording in the SSB.
/// Defined in `orchestrator` only — never in `reasoning` or `tools` crates.
pub trait SsbTraceable {
    fn to_ssb_payload(&self) -> SsbPayload;
}

impl SsbTraceable for ReasoningTracePayload {
    fn to_ssb_payload(&self) -> SsbPayload {
        SsbPayload::ReasoningTrace(self.clone())
    }
}

// ---------------------------------------------------------------------------
// Entry + file entry structs
// ---------------------------------------------------------------------------

/// A single SSB row, broadcast to subscribers after commit.
#[derive(Debug, Clone)]
pub struct SsbEntry {
    pub seq: i64,
    pub project_id: String,
    pub session_id: String,
    pub agent_id: String,
    pub superstep: i64,
    pub action_type: SsbActionType,
    /// `Arc<[u8]>` for O(1) clone on broadcast (tokio::broadcast requires Clone).
    pub payload: Arc<[u8]>,
    pub created_at: i64,
}

/// Junction table row linking an SSB entry to a file path.
#[derive(Debug, Clone)]
pub struct SsbFileEntry {
    pub seq: i64,
    pub file_path: String,
}

// ---------------------------------------------------------------------------
// Write request (3B.2)
// ---------------------------------------------------------------------------

/// Request sent to the single-writer loop via mpsc channel.
///
/// `project_id` and `session_id` are injected by the writer (bound at `Ssb`
/// construction). Callers provide only per-write fields.
pub struct SsbWriteRequest {
    pub action_type: SsbActionType,
    pub agent_id: String,
    pub superstep: i64,
    pub payload: Vec<u8>,
    pub files: Vec<String>,
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for SSB database and connection pool.
#[derive(Clone)]
pub struct SsbConfig {
    pub db_path: std::path::PathBuf,
    /// Read pool size. Defaults to `max(available_parallelism * 2, 4)`.
    pub pool_size: Option<u32>,
    pub project_id: String,
    pub session_id: String,
}

impl SsbConfig {
    fn effective_pool_size(&self) -> u32 {
        self.pool_size.unwrap_or_else(|| {
            let cpus = std::thread::available_parallelism()
                .map(|n| u32::try_from(n.get()).unwrap_or(u32::MAX))
                .unwrap_or(2);
            cpus.saturating_mul(2).max(4)
        })
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by SSB operations.
#[derive(Debug, thiserror::Error)]
pub enum SsbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("connection pool error: {0}")]
    Pool(String),

    #[error("ssb has been shut down")]
    Shutdown,

    #[error("write channel closed unexpectedly")]
    ChannelClosed,

    #[error("unknown SSB action type: {0}")]
    UnknownActionType(String),

    #[error("msgpack serialization error: {0}")]
    Serialization(#[from] rmp_serde::encode::Error),

    #[error("msgpack deserialization error: {0}")]
    Deserialization(#[from] rmp_serde::decode::Error),

    #[error("payload exceeds maximum size of {PAYLOAD_MAX_BYTES} bytes")]
    PayloadTooLarge,

    #[error("action_type '{declared}' does not match deserialized payload type '{actual}'")]
    ActionTypeMismatch { declared: String, actual: String },
}

// ---------------------------------------------------------------------------
// Async connection pool (deadpool managed, rusqlite 0.39)
// ---------------------------------------------------------------------------

/// Applies per-connection pragmas on pool connection creation.
/// `foreign_keys` is per-connection and must be set on every connection.
///
/// NOTE: This intentionally includes `load_module(conn)` for rarray support,
/// which the writer-side `apply_pragmas` does NOT include (rarray is only
/// needed for read-path queries like query_by_file and query_conflict).
fn apply_pool_pragmas(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Order matters: journal_mode first, then other pragmas.
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "wal_autocheckpoint", 1000)?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "cache_size", -64000)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    rusqlite::vtab::array::load_module(conn)?;
    Ok(())
}

/// Deadpool manager for rusqlite connections.
/// Opens connections in a blocking task and applies pragmas on creation.
struct SqliteManager {
    db_path: String,
}

impl deadpool::managed::Manager for SqliteManager {
    type Type = Connection;
    type Error = SsbError;

    async fn create(&self) -> Result<Connection, SsbError> {
        let path = self.db_path.clone();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&path)?;
            apply_pool_pragmas(&conn)?;
            Ok(conn)
        })
        .await
        .map_err(|e| SsbError::Pool(format!("spawn_blocking join error: {e}")))?
    }

    async fn recycle(
        &self,
        conn: &mut Connection,
        _metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<SsbError> {
        // Clear any stale implicit transaction from an interrupted read path.
        conn.execute_batch("ROLLBACK").ok();
        // Validate the connection is still usable with a cheap query.
        conn.execute_batch("SELECT 1")
            .map_err(|e| deadpool::managed::RecycleError::Backend(SsbError::Sqlite(e)))?;
        Ok(())
    }
}

/// Type alias for the deadpool-managed rusqlite pool.
type SsbPool = deadpool::managed::Pool<SqliteManager>;

// ---------------------------------------------------------------------------
// Reader trait
// ---------------------------------------------------------------------------

/// Read-only queries against the SSB.
pub trait SsbReader {
    fn query_by_session(&self, session_id: &str) -> Result<Vec<SsbEntry>, SsbError>;
    fn query_by_file(&self, file_paths: &[&str]) -> Result<Vec<SsbEntry>, SsbError>;
    fn query_by_type(&self, action_type: SsbActionType) -> Result<Vec<SsbEntry>, SsbError>;
    fn query_by_superstep(&self, superstep: i64) -> Result<Vec<SsbEntry>, SsbError>;
    /// Returns agent_ids that touched any of `file_paths` in the given `superstep`,
    /// excluding `session_id` (self). Uses JOIN + rarray for performance.
    fn query_conflict(
        &self,
        file_paths: &[&str],
        session_id: &str,
        superstep: i64,
    ) -> Result<Vec<String>, SsbError>;
    fn last_seq(&self) -> Result<Option<i64>, SsbError>;
    fn query_by_seq(&self, seq: i64) -> Result<Option<SsbEntry>, SsbError>;

    // Query patterns for new action types (3J.12)
    /// Query reasoning_trace entries for a specific agent, ordered by seq.
    fn query_reasoning_trace(&self, agent_id: &str) -> Result<Vec<SsbEntry>, SsbError>;
    /// Query phase_complete entries for a specific phase.
    fn query_phase_complete(&self, phase_id: &str) -> Result<Vec<SsbEntry>, SsbError>;
    /// Query isolation_metric entries grouped by phase.
    fn query_isolation_metrics(&self) -> Result<Vec<SsbEntry>, SsbError>;
}

// ---------------------------------------------------------------------------
// Internal message for write loop
// ---------------------------------------------------------------------------

type WriteResponse = oneshot::Sender<Result<i64, SsbError>>;

struct WriteMessage {
    request: SsbWriteRequest,
    respond: WriteResponse,
}

// ---------------------------------------------------------------------------
// Ssb (main handle)
// ---------------------------------------------------------------------------

/// Shared State Board: single-writer SQLite with broadcast notifications.
#[allow(dead_code)] // read_pool used by tests and SsbReader impl (3B.3)
pub struct Ssb {
    write_tx: Option<mpsc::UnboundedSender<WriteMessage>>,
    broadcast_tx: broadcast::Sender<SsbEntry>,
    read_pool: SsbPool,
    project_id: String,
    session_id: String,
    /// C-5: canonical DB path captured at construction so callers (e.g.
    /// task_runner cloud-mode hook) can open a sibling read-only
    /// `SsbViewer` without re-plumbing the SsbConfig through every
    /// hand-off site.
    db_path: std::path::PathBuf,
    shutdown: Arc<AtomicBool>,
    _write_task: tokio::task::JoinHandle<()>,
}

impl Ssb {
    /// Create a new SSB instance. Opens (or creates) the database, applies
    /// WAL pragmas, runs migrations, and spawns the single-writer task.
    pub fn new(
        config: SsbConfig,
        clock: Arc<dyn ClockProvider>,
    ) -> Result<Self, SsbError> {
        // --- Writer connection (dedicated, not pooled) ---
        let writer_conn = Connection::open(&config.db_path)?;
        apply_pragmas(&writer_conn)?;
        run_migration(&writer_conn)?;

        #[cfg(debug_assertions)]
        writer_conn.update_hook(Some(
            |action: rusqlite::hooks::Action, _db: &str, table: &str, _rowid: i64| {
                if (table == "ssb" || table == "ssb_files")
                    && (action == rusqlite::hooks::Action::SQLITE_UPDATE
                        || action == rusqlite::hooks::Action::SQLITE_DELETE)
                {
                    panic!(
                        "SSB append-only violation: attempted {:?} on table {}",
                        action, table
                    );
                }
            },
        ))?;

        // --- Read pool (deadpool managed, async-native) ---
        let db_path_str = config.db_path.to_string_lossy().to_string();
        let manager = SqliteManager { db_path: db_path_str };
        let read_pool = SsbPool::builder(manager)
            // NARROWING-OK: u32 pool size always fits usize on supported 32/64-bit targets.
            .max_size(config.effective_pool_size() as usize)
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .map_err(|e| SsbError::Pool(format!("pool build error: {e}")))?;

        // --- Channels ---
        let (write_tx, write_rx) = mpsc::unbounded_channel::<WriteMessage>();
        let (broadcast_tx, _) = broadcast::channel::<SsbEntry>(1024);

        let project_id = config.project_id.clone();
        let session_id = config.session_id.clone();
        let shutdown = Arc::new(AtomicBool::new(false));

        let bc_tx = broadcast_tx.clone();
        let pid = project_id.clone();
        let sid = session_id.clone();
        let sd = shutdown.clone();

        let write_task = tokio::spawn(write_loop(
            writer_conn, write_rx, bc_tx, clock, pid, sid, sd,
        ));

        Ok(Self {
            write_tx: Some(write_tx),
            broadcast_tx,
            read_pool,
            project_id,
            session_id,
            db_path: config.db_path.clone(),
            shutdown,
            _write_task: write_task,
        })
    }

    /// C-5 — canonical filesystem path of the underlying SQLite DB.
    /// Captured at construction; consumers (e.g. task_runner cloud-mode
    /// hook) open a sibling read-only [`crate::ssb_viewer::SsbViewer`]
    /// against this path without having to thread the original
    /// [`SsbConfig`] through every wiring hand-off.
    #[must_use]
    pub fn db_path(&self) -> &std::path::Path {
        &self.db_path
    }

    /// Submit a write request to the single-writer loop.
    ///
    /// # IPC sync contract (WARN-6)
    ///
    /// Callers **must** `.await` this method and receive the `seq` confirmation
    /// **before** dispatching any `ToolResult` to the agent. This guarantees
    /// the SSB entry is durable before the agent sees the result.
    ///
    /// # Type validation (3J.12)
    ///
    /// The payload bytes are deserialized and validated against the declared
    /// `action_type`. If they don't match, the write is rejected with
    /// `SsbError::ActionTypeMismatch`. This prevents silent corruption.
    pub async fn write(&self, request: SsbWriteRequest) -> Result<i64, SsbError> {
        // Validate that payload matches declared action_type (write-time validation)
        validate_payload_matches_action_type(&request.payload, request.action_type)?;

        let tx = self.write_tx.as_ref().ok_or(SsbError::Shutdown)?;
        if self.shutdown.load(Ordering::Acquire) {
            return Err(SsbError::Shutdown);
        }
        let (respond, rx) = oneshot::channel();
        tx.send(WriteMessage { request, respond })
            .map_err(|_| SsbError::Shutdown)?;
        rx.await.map_err(|_| SsbError::ChannelClosed)?
    }

    /// Submit a typed payload write. Derives `action_type` from the enum variant,
    /// serializes via msgpack, enforces the size guard, and delegates to [`write()`].
    pub async fn write_payload(
        &self,
        agent_id: &str,
        superstep: i64,
        payload: SsbPayload,
        files: Vec<String>,
    ) -> Result<i64, SsbError> {
        let action_type = payload.action_type();
        let bytes = rmp_serde::to_vec_named(&payload)?;
        if bytes.len() > PAYLOAD_MAX_BYTES {
            return Err(SsbError::PayloadTooLarge);
        }
        self.write(SsbWriteRequest {
            action_type,
            agent_id: agent_id.to_owned(),
            superstep,
            payload: bytes,
            files,
        })
        .await
    }

    /// Subscribe to broadcast notifications of committed entries.
    /// Slow subscribers that fall behind 1024 entries will receive `Lagged`.
    pub fn subscribe(&self) -> broadcast::Receiver<SsbEntry> {
        self.broadcast_tx.subscribe()
    }

    /// Gracefully shut down the write loop. Idempotent.
    ///
    /// Drops the mpsc sender so the write loop drains remaining messages
    /// and exits. Pending `write()` callers whose oneshot hasn't been
    /// answered yet will receive `Err(SsbError::ChannelClosed)`.
    pub fn shutdown(&mut self) {
        if !self.shutdown.swap(true, Ordering::Release) {
            self.write_tx.take();
        }
    }

    /// Project ID this SSB is bound to.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Session ID this SSB is bound to.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

// ---------------------------------------------------------------------------
// WAL pragma application (writer connection)
// ---------------------------------------------------------------------------

fn apply_pragmas(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "wal_autocheckpoint", 1000)?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "cache_size", -64000)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Migration
// ---------------------------------------------------------------------------

fn run_migration(conn: &Connection) -> Result<(), rusqlite::Error> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version == SCHEMA_VERSION {
        return Ok(());
    }
    // SSB is session-scoped. Stale schema → drop and recreate (data loss
    // is acceptable for old sessions).
    if version > 0 {
        conn.execute_batch("DROP TABLE IF EXISTS ssb_files; DROP TABLE IF EXISTS ssb;")?;
    }
    conn.execute_batch(CREATE_SSB_TABLE)?;
    conn.execute_batch(CREATE_SSB_FILES_TABLE)?;
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Write loop (3B.2 — single-writer task)
// ---------------------------------------------------------------------------

async fn write_loop(
    conn: Connection,
    mut rx: mpsc::UnboundedReceiver<WriteMessage>,
    broadcast_tx: broadcast::Sender<SsbEntry>,
    clock: Arc<dyn ClockProvider>,
    project_id: String,
    session_id: String,
    shutdown: Arc<AtomicBool>,
) {
    while let Some(msg) = rx.recv().await {
        let result = execute_write(
            &conn,
            &msg.request,
            &clock,
            &project_id,
            &session_id,
        );

        match result {
            Ok((seq, created_at)) => {
                let entry = SsbEntry {
                    seq,
                    project_id: project_id.clone(),
                    session_id: session_id.clone(),
                    agent_id: msg.request.agent_id.clone(),
                    superstep: msg.request.superstep,
                    action_type: msg.request.action_type,
                    payload: Arc::from(msg.request.payload.as_slice()),
                    created_at,
                };
                // Respond BEFORE broadcast (WARN-6 IPC sync): caller must
                // have seq confirmation before any subscriber can act.
                let _ = msg.respond.send(Ok(seq));
                // Broadcast after commit + after respond. No receivers = fine.
                let _ = broadcast_tx.send(entry);
            }
            Err(e) => {
                let _ = msg.respond.send(Err(e));
            }
        }
    }
    // Channel closed (shutdown or all senders dropped). Mark shutdown.
    shutdown.store(true, Ordering::Release);
}

/// Execute a single write within a BEGIN IMMEDIATE transaction.
fn execute_write(
    conn: &Connection,
    request: &SsbWriteRequest,
    clock: &Arc<dyn ClockProvider>,
    project_id: &str,
    session_id: &str,
) -> Result<(i64, i64), SsbError> {
    let created_at = i64::try_from(
        clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs(),
    )
    .unwrap_or(i64::MAX);

    // BEGIN IMMEDIATE acquires write lock early — avoids SQLITE_BUSY at commit.
    conn.execute("BEGIN IMMEDIATE", [])?;

    let insert_result = (|| -> Result<i64, rusqlite::Error> {
        conn.execute(
            "INSERT INTO ssb (project_id, session_id, agent_id, superstep, action_type, payload, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                project_id,
                session_id,
                request.agent_id,
                request.superstep,
                request.action_type.as_sql_str(),
                request.payload,
                created_at,
            ],
        )?;
        let seq = conn.last_insert_rowid();

        for file_path in &request.files {
            conn.execute(
                "INSERT INTO ssb_files (seq, file_path) VALUES (?1, ?2)",
                params![seq, file_path],
            )?;
        }
        Ok(seq)
    })();

    match insert_result {
        Ok(seq) => {
            // COMMIT can fail (disk full, I/O error). If it does, we must
            // ROLLBACK to avoid leaving the connection stuck in an open
            // transaction (rusqlite#366).
            if let Err(e) = conn.execute_batch("COMMIT") {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(SsbError::Sqlite(e));
            }
            Ok((seq, created_at))
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(SsbError::Sqlite(e))
        }
    }
}

// ---------------------------------------------------------------------------
// Row mapper
// ---------------------------------------------------------------------------

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<SsbEntry> {
    let at_str: String = row.get(5)?;
    let action_type = ssb_action_type_from_str(&at_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let payload_bytes: Vec<u8> = row.get(6)?;
    Ok(SsbEntry {
        seq: row.get(0)?,
        project_id: row.get(1)?,
        session_id: row.get(2)?,
        agent_id: row.get(3)?,
        superstep: row.get(4)?,
        action_type,
        payload: Arc::from(payload_bytes.as_slice()),
        created_at: row.get(7)?,
    })
}

// ---------------------------------------------------------------------------
// Sync→async bridge for deadpool reads
// ---------------------------------------------------------------------------

impl Ssb {
    /// Run a closure on a read-pool connection, bridging deadpool's async API
    /// to the sync SsbReader trait. Uses `block_in_place` to avoid blocking the
    /// tokio runtime when called from an async context.
    fn with_read_conn<F, T>(&self, f: F) -> Result<T, SsbError>
    where
        F: FnOnce(&Connection) -> Result<T, SsbError> + Send + 'static,
        T: Send + 'static,
    {
        let pool = self.read_pool.clone();
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => match handle.runtime_flavor() {
                tokio::runtime::RuntimeFlavor::MultiThread => {
                    // Multi-thread runtime: block_in_place is safe.
                    tokio::task::block_in_place(|| {
                        handle.block_on(async move {
                            let obj = pool
                                .get()
                                .await
                                .map_err(|e| SsbError::Pool(format!("{e}")))?;
                            f(&obj)
                        })
                    })
                }
                _ => {
                    // CurrentThread runtime context (e.g. rt.block_on in
                    // try_save_snapshot): block_in_place panics here, and
                    // re-entering the same runtime deadlocks. Spawn an OS
                    // thread with its own transient runtime to drive the work.
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .map_err(|e| SsbError::Pool(format!("rt build: {e}")))?;
                        rt.block_on(async move {
                            let obj = pool
                                .get()
                                .await
                                .map_err(|e| SsbError::Pool(format!("{e}")))?;
                            f(&obj)
                        })
                    })
                    .join()
                    .map_err(|_| SsbError::Pool("read worker panicked".into()))?
                }
            },
            Err(_) => {
                // No async context: spin transient current-thread runtime.
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| SsbError::Pool(format!("rt build: {e}")))?;
                rt.block_on(async move {
                    let obj = pool
                        .get()
                        .await
                        .map_err(|e| SsbError::Pool(format!("{e}")))?;
                    f(&obj)
                })
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SsbReader implementation
// ---------------------------------------------------------------------------

impl SsbReader for Ssb {
    fn query_by_session(&self, session_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
        let sid = session_id.to_owned();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE session_id = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(params![sid, pid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_by_file(&self, file_paths: &[&str]) -> Result<Vec<SsbEntry>, SsbError> {
        if file_paths.is_empty() {
            return Ok(vec![]);
        }
        let paths: Vec<String> = file_paths.iter().map(|p| (*p).to_owned()).collect();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let values: Vec<Value> = paths.iter().map(|p| Value::Text(p.clone())).collect();
            let ptr = Rc::new(values);
            let mut stmt = conn.prepare(
                "SELECT DISTINCT ssb.seq, ssb.project_id, ssb.session_id, ssb.agent_id, \
                 ssb.superstep, ssb.action_type, ssb.payload, ssb.created_at \
                 FROM ssb JOIN ssb_files ON ssb_files.seq = ssb.seq \
                 WHERE ssb_files.file_path IN (SELECT value FROM rarray(?1)) \
                   AND ssb.project_id = ?2",
            )?;
            let rows = stmt.query_map(params![ptr, pid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_by_type(&self, action_type: SsbActionType) -> Result<Vec<SsbEntry>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE action_type = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(params![action_type.as_sql_str(), pid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_by_superstep(&self, superstep: i64) -> Result<Vec<SsbEntry>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE superstep = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(params![superstep, pid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_conflict(
        &self,
        file_paths: &[&str],
        session_id: &str,
        superstep: i64,
    ) -> Result<Vec<String>, SsbError> {
        if file_paths.is_empty() {
            return Ok(vec![]);
        }
        let paths: Vec<String> = file_paths.iter().map(|p| (*p).to_owned()).collect();
        let sid = session_id.to_owned();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let values: Vec<Value> = paths.iter().map(|p| Value::Text(p.clone())).collect();
            let ptr = Rc::new(values);
            let mut stmt = conn.prepare(
                "SELECT DISTINCT ssb.agent_id \
                 FROM ssb_files \
                 JOIN ssb ON ssb_files.seq = ssb.seq \
                 WHERE ssb_files.file_path IN (SELECT value FROM rarray(?1)) \
                   AND ssb.project_id = ?2 \
                   AND ssb.session_id != ?3 \
                   AND ssb.superstep = ?4",
            )?;
            let rows = stmt.query_map(
                params![ptr, pid, sid, superstep],
                |row| row.get::<_, String>(0),
            )?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn last_seq(&self) -> Result<Option<i64>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare("SELECT MAX(seq) FROM ssb WHERE project_id = ?1")?;
            let result: Option<i64> = stmt.query_row(params![pid], |row| row.get(0))?;
            Ok(result)
        })
    }

    fn query_by_seq(&self, seq: i64) -> Result<Option<SsbEntry>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE seq = ?1 AND project_id = ?2 LIMIT 1",
            )?;
            let mut rows = stmt.query_map(params![seq, pid], row_to_entry)?;
            match rows.next() {
                Some(row) => Ok(Some(row.map_err(SsbError::Sqlite)?)),
                None => Ok(None),
            }
        })
    }

    fn query_reasoning_trace(&self, agent_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
        let aid = agent_id.to_owned();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE action_type = ?1 AND agent_id = ?2 AND project_id = ?3 \
                 ORDER BY seq ASC",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::ReasoningTrace.as_sql_str(), aid, pid],
                row_to_entry,
            )?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_phase_complete(&self, phase_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
        let phase = phase_id.to_owned();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE action_type = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::PhaseComplete.as_sql_str(), pid],
                row_to_entry,
            )?;
            let all_entries: Vec<SsbEntry> = rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)?;

            // Filter by phase_id from payload (since phase_id is in the payload, not a separate column)
            let filtered = all_entries
                .into_iter()
                .filter(|entry| {
                    if let Ok(payload) = rmp_serde::from_slice::<PhaseCompletePayload>(&entry.payload) {
                        payload.phase_id == phase
                    } else {
                        false
                    }
                })
                .collect();
            Ok(filtered)
        })
    }

    fn query_isolation_metrics(&self) -> Result<Vec<SsbEntry>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE action_type = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::IsolationMetric.as_sql_str(), pid],
                row_to_entry,
            )?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }
}

// ---------------------------------------------------------------------------
// Canonical SSB path helpers
// ---------------------------------------------------------------------------

/// Subdirectory (relative to a project/base directory) that holds the
/// managed sessions state, including the single SSB SQLite database.
// B-1: canonical SSB sessions subdir now under .elai_cc/sessions.
// SSB_SESSIONS_SUBDIR kept for source-compat (external callers); value updated.
pub const SSB_SESSIONS_SUBDIR: &str = ".elai_cc/sessions";

/// File name of the canonical SSB SQLite database inside
/// [`SSB_SESSIONS_SUBDIR`].
pub const SSB_DB_FILE_NAME: &str = "ssb.db";

/// Canonical SSB database path for the given project/base directory.
///
/// The CLI's sidecar bootstrap uses `<session-persistence-dir>/ssb.db`, and
/// managed session persistence files live under `<base>/.elai_cc/sessions/`
/// (see `session_control::managed_sessions_dir_for`). The dashboard must
/// open the SSB at the **same** path so it and a running CLI REPL share
/// telemetry instead of silently creating a second, empty database.
///
/// Returns `<base_dir>/.elai_cc/sessions/ssb.db` as a pure path computation
/// (no filesystem side effects — callers choose whether to create the
/// parent directory or let `Ssb::new`/`SsbViewer::open` do it).
pub fn default_ssb_path_for_base(base_dir: impl AsRef<std::path::Path>) -> std::path::PathBuf {
    base_dir
        .as_ref()
        .join(SSB_SESSIONS_SUBDIR)
        .join(SSB_DB_FILE_NAME)
}

/// Canonical SSB database path rooted at the current working directory.
///
/// Convenience wrapper for callers (dashboard binary, CLI) that treat CWD
/// as the project root. Returns `Err` only if `std::env::current_dir` fails.
pub fn default_ssb_path() -> std::io::Result<std::path::PathBuf> {
    Ok(default_ssb_path_for_base(std::env::current_dir()?))
}

// ---------------------------------------------------------------------------
// B-39: ReflectEventSink bridge — translates reasoning crate events to SSB rows.
// ---------------------------------------------------------------------------

impl ReflectEventSink for Ssb {
    /// Translates each [`ReflectEvent`] variant into the corresponding
    /// [`SsbActionType`] + [`SsbPayload`] and appends to the SSB.
    ///
    /// Write is fire-and-forget via `tokio::spawn`: the pipeline never awaits
    /// SSB persistence. Write failures are logged at WARN level and silently
    /// discarded — the pipeline must never propagate SSB errors to the caller.
    fn emit(&self, event: ReflectEvent) {
        let (action_type, payload) = match event {
            ReflectEvent::Triaged { turn_id, class } => (
                SsbActionType::ReflectTriaged,
                SsbPayload::ReflectTriaged(ReflectTriagedPayload {
                    turn_id,
                    class: class.as_wire_str().to_owned(),
                }),
            ),
            ReflectEvent::Candidate {
                turn_id,
                candidate_index,
                content,
            } => (
                SsbActionType::ReflectCandidate,
                SsbPayload::ReflectCandidate(ReflectCandidatePayload {
                    turn_id,
                    candidate_index,
                    content,
                }),
            ),
            ReflectEvent::AuditVerdict {
                turn_id,
                best_index,
                rationale,
            } => (
                SsbActionType::ReflectAuditVerdict,
                SsbPayload::ReflectAuditVerdict(ReflectAuditVerdictPayload {
                    turn_id,
                    best_index,
                    rationale,
                }),
            ),
            ReflectEvent::Refined {
                turn_id,
                total_tokens_used,
                synthesis,
            } => (
                SsbActionType::ReflectRefined,
                SsbPayload::ReflectRefined(ReflectRefinedPayload {
                    turn_id,
                    total_tokens_used,
                    synthesis,
                }),
            ),
        };

        // Serialize payload eagerly on the caller's thread; spawn write async.
        let bytes = match rmp_serde::to_vec_named(&payload) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(
                    target: "orchestrator::ssb",
                    error = %e,
                    action_type = action_type.as_sql_str(),
                    "ReflectEventSink: payload serialization failed — dropping event"
                );
                return;
            }
        };

        if bytes.len() > PAYLOAD_MAX_BYTES {
            tracing::warn!(
                target: "orchestrator::ssb",
                action_type = action_type.as_sql_str(),
                size = bytes.len(),
                max = PAYLOAD_MAX_BYTES,
                "ReflectEventSink: payload exceeds size limit — dropping event"
            );
            return;
        }

        let write_tx = match &self.write_tx {
            Some(tx) => tx.clone(),
            None => {
                tracing::warn!(
                    target: "orchestrator::ssb",
                    action_type = action_type.as_sql_str(),
                    "ReflectEventSink: SSB is shut down — dropping event"
                );
                return;
            }
        };

        let request = SsbWriteRequest {
            action_type,
            // Reflect events are emitted by the CLI chat loop; agent_id is
            // not meaningful here, so we use a stable sentinel.
            agent_id: "reflect_pipeline".to_owned(),
            superstep: 0,
            payload: bytes,
            files: vec![],
        };

        let (respond, _rx) = tokio::sync::oneshot::channel();
        if let Err(e) = write_tx.send(WriteMessage { request, respond }) {
            tracing::warn!(
                target: "orchestrator::ssb",
                error = %e,
                action_type = action_type.as_sql_str(),
                "ReflectEventSink: write channel send failed — dropping event"
            );
        }
        // _rx dropped intentionally: fire-and-forget; no await on the response.
    }
}

// ---------------------------------------------------------------------------
// B-38 (SCHEMA_VERSION 43) payload structs — appended at end of payloads region.
// Additive only. Existing payload structs are not renumbered or modified.
// ---------------------------------------------------------------------------

/// B-38 — payload for [`SsbActionType::AmbiguityScored`].
///
/// Emitted by the GroundingSweepGate when a non-`None` `AmbiguityScore` is
/// supplied via `TransitionContext.ambiguity_score`. Mirrors the
/// `AmbiguityScore.breakdown` shape but flattened to avoid pulling the
/// `elai-reasoning` crate type into the SSB schema surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmbiguityScoredPayload {
    /// Overall ambiguity score, in `[0.0, 1.0]`, rounded to 4 dp.
    pub score: f32,
    /// Milestone label (`"initial"`, `"progress"`, `"refined"`, `"ready"`).
    pub milestone: String,
    pub goal: f32,
    pub constraint: f32,
    pub success: f32,
    /// `None` for greenfield; `Some(_)` for brownfield.
    pub context: Option<f32>,
    pub brownfield: bool,
}

/// B-38 — payload for [`SsbActionType::AmbiguityFloorFailed`].
///
/// Emitted by the GroundingSweepGate when one or more per-dimension floors
/// were breached even when `overall_score <= AMBIGUITY_THRESHOLD`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmbiguityFloorFailedPayload {
    pub score: f32,
    /// Component names that fell below their floor (e.g. `"goal_clarity"`).
    pub failed_dimensions: Vec<String>,
}

/// B-38 — payload for [`SsbActionType::AmbiguityStreakAdvanced`].
///
/// Emitted by the GroundingSweepGate when the score qualifies but the
/// auto-complete streak hasn't yet reached the required count. The gate
/// itself returns BLOCK on this case; the row is purely informational so
/// the dashboard can show 1/2 → 2/2 progress.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmbiguityStreakAdvancedPayload {
    pub score: f32,
    /// The streak count INCLUDING this snapshot (e.g. 1 means the snapshot
    /// is the first ready one in a row).
    pub current_streak: u32,
    /// Required streak count (mirrors
    /// [`elai_reasoning::AUTO_COMPLETE_STREAK_REQUIRED`] cast to u32).
    pub required: u32,
}

/// B-42 — payload for [`SsbActionType::CosmosRefreshStarted`].
///
/// Emitted by `knowledge::cosmos_worker::CosmosWorker::tick` at the start
/// of a refresh sweep. Carries the count of artifacts queued for refresh
/// and the manifest size so the dashboard can render a "N/M refreshing"
/// progress card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmosRefreshStartedPayload {
    /// Number of artifacts identified as stale/missing for this refresh sweep.
    pub queued: u32,
    /// Total number of artifacts in the manifest.
    pub total: u32,
}

/// B-42 — payload for [`SsbActionType::CosmosArtifactUpdated`].
///
/// Emitted by `knowledge::cosmos_worker::CosmosWorker` after a successful
/// artifact refresh (network fetch + body hash + manifest write).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmosArtifactUpdatedPayload {
    /// Logical artifact name (e.g. `"saas_pricing_patterns"`).
    pub name: String,
    /// blake3 content hash of the refreshed body.
    pub blake3: String,
    /// Body length in bytes.
    pub body_bytes: u32,
    /// Wall-clock latency in milliseconds for the refresh operation.
    pub latency_ms: u32,
}

/// B-42 — payload for [`SsbActionType::CosmosArtifactDecayed`].
///
/// Emitted when a `cosmos_get(name)` call surfaces an artifact whose age
/// exceeds `3 * cadence_interval`. The artifact is still served; the
/// row is informational so the dashboard can render a DECAYED badge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmosArtifactDecayedPayload {
    /// Logical artifact name.
    pub name: String,
    /// Age in seconds at the time of the decay observation.
    pub age_secs: u64,
    /// Configured cadence (`"daily"`, `"weekly"`, `"monthly"`, `"quarterly"`).
    pub cadence: String,
}

/// B-42 — payload for [`SsbActionType::CosmosGroundingUsed`].
///
/// Emitted by `reasoning::verification::LocalVerifier` whenever a
/// `VerificationMethod::CosmosCheck` dispatch resolves a claim against
/// the cosmos store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmosGroundingUsedPayload {
    /// Claim category (e.g. `"design"`, `"ip_boundary"`, `"monetization"`).
    pub claim_category: String,
    /// Logical artifact name resolved from the lookup.
    pub artifact_name: String,
    /// True when the lookup returned a Fresh-or-Stale artifact (i.e. used
    /// for grounding). False when the lookup returned no matching artifact.
    pub matched: bool,
}

/// B-41 — payload for [`SsbActionType::PerLevelAuditStarted`].
///
/// Emitted by the pipeline-tree navigator immediately before spawning an
/// Auditor* agent at a tier node (Concepts/Phases/Steps/BabySteps/BabyStepDetail).
/// The `mode` and `tier` fields are the snake_case labels from
/// `auditor_mode::AuditorMode::as_str()` and `auditor_mode::TierNodeId::tier_str()`
/// — kept as strings so this payload stays decoupled from the auditor_mode
/// module (avoids a cyclic dep in payload deserialization).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerLevelAuditStartedPayload {
    /// Auditor mode label: `"concepts" | "phases" | "steps" | "babysteps" |
    /// "babystep_detail" | "plan"`.
    pub mode: String,
    /// Tier label: `"concept" | "phase" | "step" | "baby_step"`.
    pub tier: String,
    /// Underlying tier node id (ConceptId/PhaseId/StepId/BabyStepId raw value).
    pub node_id: u64,
}

/// B-41 — payload for [`SsbActionType::PerLevelAuditVerdict`].
///
/// Emitted by the pipeline-tree navigator after an Auditor* agent returns a
/// `LevelAuditVerdict`. `findings_count` is the length of the verdict's
/// `findings` vec (full Finding list is persisted as Markdown by
/// `auditor_mode::persist_verdict`; SSB carries only the metadata so payload
/// size stays bounded).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerLevelAuditVerdictPayload {
    pub mode: String,
    pub tier: String,
    pub node_id: u64,
    /// `"allow" | "block"`.
    pub verdict: String,
    pub findings_count: u32,
    pub respawned_once: bool,
    pub cascade_to_parent: bool,
}

/// B-41 — payload for [`SsbActionType::PerLevelAuditCascade`].
///
/// Emitted when a 2nd BLOCK verdict at a tier sets `cascade_to_parent=true`
/// and the navigator bubbles up to the parent decomposer. `parent_tier` is
/// the immediate parent in the decomposition hierarchy
/// (concept ← phase ← step ← baby_step ← baby_step_detail-cascades-to-baby_step).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerLevelAuditCascadePayload {
    pub mode: String,
    pub tier: String,
    pub node_id: u64,
    /// Tier label of the parent that was bubbled up to.
    pub parent_tier: String,
}

/// B-37C — payload for [`SsbActionType::RuntimeFallback`].
///
/// Emitted by `runtime_orchestrator::RuntimeOrchestrator::try_complete`
/// once per fallback chain step. `role` is the agent role being served,
/// `primary` the originally-configured backend, `attempted` the ordered
/// list of backends tried so far (including the one this row records),
/// `succeeded` is `Some(backend)` on a successful step or `None` for a
/// failure step that triggers chain advancement. `reason` is a short
/// human-readable explanation surfaced from the underlying
/// `RuntimeAdapterError` or success message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeFallbackPayload {
    pub role: elai_ipc_types::AgentRole,
    pub primary: elai_providers::runtime::RuntimeBackend,
    pub attempted: Vec<elai_providers::runtime::RuntimeBackend>,
    pub succeeded: Option<elai_providers::runtime::RuntimeBackend>,
    pub reason: String,
}

/// B-37C — payload for [`SsbActionType::RuntimeHealthChanged`].
///
/// Emitted by `runtime_orchestrator::RuntimeOrchestrator::health_snapshot`
/// when a backend's `RuntimeHealth` transitions to a different variant
/// vs the previously cached snapshot. Backed by the `RuntimeHealth`
/// serde shape (tag = "status") so dashboard renderers can decode
/// without reaching into the providers crate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeHealthChangedPayload {
    pub backend: elai_providers::runtime::RuntimeBackend,
    pub prev: elai_providers::runtime::RuntimeHealth,
    pub next: elai_providers::runtime::RuntimeHealth,
}

/// B-37C — payload for [`SsbActionType::CliSubprocessSpawned`].
///
/// Emitted by CLI adapters (B-37B `cli_common::spawn`) on every
/// `tokio::process::Command::spawn` call. `argv_redacted` MUST have
/// secret arguments stripped — see `cli_common::redact_argv` — before
/// the row is written; the writer is the last line of defence against
/// accidental secret leakage into the SSB store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliSubprocessSpawnedPayload {
    pub backend: elai_providers::runtime::RuntimeBackend,
    pub pid: u32,
    pub argv_redacted: Vec<String>,
}

/// B-37C — payload for [`SsbActionType::CliSubprocessKilled`].
///
/// Emitted by CLI adapters on subprocess termination — either
/// natural exit, `cli_common::kill_and_wait` (idle/startup timeout),
/// or user-driven cancel. `exit_code` is None when the process was
/// signalled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliSubprocessKilledPayload {
    pub backend: elai_providers::runtime::RuntimeBackend,
    pub pid: u32,
    pub exit_code: Option<i32>,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer payloads
// ---------------------------------------------------------------------------

/// C-1 — payload for [`SsbActionType::GrcStepEmitted`].
///
/// Emitted by `task_runner::emit_reasoning` when a `PipelineGrc` handle is
/// plumbed through `ResolvedConfig`. `agent_id` is sourced from the
/// engine's `AgentGrcConfig.role` (NOT the synthetic `RUNNER_AGENT_ID`
/// constant). `budget_remaining` is the per-topic
/// `GrcBudget::remaining_calls()` value. `elapsed_ms` is the wall-clock
/// time since the runner entered the GRC step (caller supplies; clock is
/// the runner's `ClockProvider`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrcStepEmittedPayload {
    pub agent_id: String,
    pub topic: String,
    pub step_type: String,
    pub grounding_level: String,
    pub confidence: f64,
    pub budget_remaining: u32,
    pub elapsed_ms: u64,
}

/// C-1 — payload for [`SsbActionType::GroundingSweepResult`].
///
/// Emitted ONCE per `PipelineGrc::ground_plan` call. Fields:
/// * `ungrounded` — count of claims that remain ungrounded after sweep.
/// * `verify_needed` — count of `training:`-prefixed claims (subset of
///   `ungrounded`) that require manual verification.
/// * `grounded` — count of claims that did verify (
///   `claims.len() - ungrounded.len()`).
/// * `top_claim` — first ungrounded claim text, used by the dashboard
///   GroundingSweepCard for an at-a-glance preview. `None` when all
///   claims are grounded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundingSweepResultPayload {
    pub ungrounded: u32,
    pub verify_needed: u32,
    pub grounded: u32,
    pub top_claim: Option<String>,
}

/// C-1 — payload for [`SsbActionType::CsqEvaluated`].
///
/// Emitted by `task_runner` immediately after `PipelineGrc::generate_csq`
/// returns. `severity` mirrors `CsqSeverity::Critical` / `Warning`
/// (the worst severity among the checks, or `"None"` when the result
/// contains no checks). `guard_present` reflects whether every critical
/// check has `guard_added=true`. `blocked` is `true` when the runner's
/// `hook_pre_blocked` flag was set as a result of the verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CsqEvaluatedPayload {
    pub severity: String,
    pub guard_present: bool,
    pub blocked: bool,
    pub check_count: u32,
}

/// C-1 — payload for [`SsbActionType::AuditorRubberStampRejected`].
///
/// Emitted at the `PipelineGrc` caller when `BasicGrcEngine::advance`
/// returns `GrcError::InsufficientFindings`. `role` is the auditor's
/// `AgentGrcConfig.role` (always `"auditor"` today). `finding_count` is
/// the observed count of `FINDING:` rows in the topic state;
/// `expected_min` is the gate threshold (currently `1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditorRubberStampRejectedPayload {
    pub role: String,
    pub finding_count: u32,
    pub expected_min: u32,
}

/// C-1 — payload for [`SsbActionType::HydeQuery`].
///
/// Emitted by `HydeStage::hypothesize` per call. `query_hash` is the
/// hex-encoded blake3 hash of the query bytes (full 32 bytes / 64 hex
/// chars). `hypothesis_len` is `0` when no hypothesis was produced
/// (short query / breaker open / sidecar error). `cache_hit=true`
/// indicates the cache TTL window was within bounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HydeQueryPayload {
    pub query_hash: String,
    pub hypothesis_len: u32,
    pub cache_hit: bool,
    pub latency_ms: u64,
}

/// C-1 — payload for [`SsbActionType::VectorCacheHitRate`].
///
/// Aggregate hits/misses snapshot flushed by
/// `VectorCache::flush_hit_rate_ssb`. `hit_rate_pct` ranges
/// `0.0..=100.0`. The producer skips emission when `hits + misses == 0`,
/// so every row carries non-zero traffic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorCacheHitRatePayload {
    pub session_id: String,
    pub hits: u32,
    pub misses: u32,
    pub hit_rate_pct: f64,
}

/// C-1 — payload for [`SsbActionType::QueryTrace`].
///
/// Per-query trace emitted at the end of
/// `QueryPipeline::query_with_tier`. Pairs with the existing
/// `tracing::debug!("query_trace")` line — both fire on every query;
/// the SSB row is the persisted form the dashboard reads via
/// `get_query_trace_recent`. `query_id` is the UUID stringified from
/// `QueryTrace::query_id`. `tier` is one of `"FAST" / "STANDARD" /
/// "DEEP" / "AUTO"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryTracePayload {
    pub query_id: String,
    pub tier: String,
    pub stage_count: u32,
    pub result_count: u32,
    pub total_latency_ms: u64,
}

// ---------------------------------------------------------------------------
// C-2 — 10 NEW payload structs (orchestration + cosmos + stagnation).
// Defined here so the SCHEMA_VERSION 47 -> 48 bump is atomic for every
// 9-tuple-lockstep row (CHECK / enum / as_sql_str / from_str / SsbPayload /
// action_type / test_payload / all_test_payloads / count assert).
// ---------------------------------------------------------------------------

/// C-2 — payload for [`SsbActionType::PhaseNodeReady`].
///
/// Emitted by the task_runner baby-step loop the first time
/// `nav.next_baby_step` returns a baby-step whose
/// `tree.location_index[bs_id].phase_id` differs from the previously
/// observed phase. `depth` mirrors the navigator's
/// `DecompositionLevel::Phase` depth (always `1` today; reserved for
/// future N-level decompositions). `parent_concept_id` is the
/// containing concept (always present per location_index invariant).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseNodeReadyPayload {
    pub phase_id: u64,
    pub concept_id: u64,
    pub label: String,
    pub depth: u32,
}

/// C-2 — payload for [`SsbActionType::StepNodeReady`].
///
/// Emitted when the task_runner baby-step loop observes a step-id
/// change against the previous iteration. `parent_phase_id` and
/// `parent_concept_id` are the containing decomposition ancestors so
/// the dashboard can render the row inside the correct tree column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepNodeReadyPayload {
    pub step_id: u64,
    pub phase_id: u64,
    pub concept_id: u64,
    pub label: String,
    pub depth: u32,
}

/// C-2 — payload for [`SsbActionType::BabyStepReady`].
///
/// Emitted once per baby-step consumption in the task_runner loop
/// (matches the cadence of `emit_reasoning`). Carries the full
/// (concept, phase, step, baby_step) ancestry plus the truncated task
/// label so the dashboard BubbleUpStrip can show a per-baby-step
/// bubble without joining additional rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BabyStepReadyPayload {
    pub baby_step_id: u64,
    pub step_id: u64,
    pub phase_id: u64,
    pub concept_id: u64,
    pub label: String,
    pub depth: u32,
}

/// C-2 — payload for [`SsbActionType::DagRepairEscalated`].
///
/// Emitted from `pipeline_builder` when `DagRepairLoop::repair`
/// returns `RepairOutcome::Escalated` after `T_DAG = 10` retries. The
/// row distinguishes "successful repair after N rounds"
/// ([`SsbActionType::DagRepairRound`]) from "gave up after N rounds"
/// so the dashboard can surface a distinct ESCALATED badge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagRepairEscalatedPayload {
    /// Number of rounds the repair loop executed before escalating.
    pub rounds: u32,
    /// Count of validation issues that remained unrepaired.
    pub remaining_issues: u32,
    /// Short summary of the last validation error (first remaining issue's
    /// `message`). Empty when no remaining issues were attached.
    pub last_error: String,
}

/// C-2 — payload for [`SsbActionType::AgentStatusChanged`].
///
/// Sibling row to [`AgentLifecyclePayload`], emitted alongside every
/// AgentLifecycle write in `bsp/engine`. Single `state` field per
/// DRIFT-5 in context-package-C2.md (no `old_lifecycle`): consumers
/// derive transitions by reading the ordered stream per `agent_id`.
/// `timestamp_ms` is sourced from the engine's `ClockProvider` (never
/// `SystemTime::now` directly) per ENTROPY_QUARANTINE.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentStatusChangedPayload {
    pub agent_id: String,
    /// AgentRole discriminant in snake_case (mirrors `AgentLifecyclePayload.role`).
    pub role: String,
    /// `"spawned" | "ready" | "exited"` — mirrors `AgentLifecyclePayload.state`.
    pub state: String,
    /// Present when `state == "exited"`: `"clean_exit" | "crash" | "timeout" |
    /// "budget_exceeded"`. Mirrors `AgentLifecyclePayload.death_class`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_class: Option<String>,
    /// Wall-clock millisecond timestamp sourced from the engine clock.
    pub timestamp_ms: u64,
}

/// C-2 — payload for [`SsbActionType::StagnationSpinningDetected`].
///
/// Defined now so the SCHEMA bump is atomic; producer wiring lands in
/// C-10 (Stagnation Detector). Carries the detector verdict shape used
/// by [`crate::anti_spiral::SpiralTracker`] when the same step_hash
/// repeats 3+ times within the current phase window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagnationSpinningDetectedPayload {
    pub agent_id: String,
    pub pattern: String,
    pub iteration: u32,
    pub detail: String,
}

/// C-2 — payload for [`SsbActionType::StagnationOscillationDetected`].
///
/// Defined now; producer wiring lands in C-10. Period-2 oscillation
/// pattern (e.g. step_hash sequence A B A B A).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagnationOscillationDetectedPayload {
    pub agent_id: String,
    pub pattern: String,
    pub iteration: u32,
    pub detail: String,
}

/// C-2 — payload for [`SsbActionType::LateralThinkingActivated`].
///
/// Defined now; producer wiring lands in C-10 (persona switcher).
/// Carries the persona label that was activated to break a stagnation
/// loop (`"hacker" | "researcher" | "simplifier" | "architect" |
/// "contrarian"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LateralThinkingActivatedPayload {
    pub agent_id: String,
    pub persona: String,
    pub iteration: u32,
    pub detail: String,
}

/// C-2 — payload for [`SsbActionType::GoalDriftWarning`].
///
/// Defined now; producer wiring lands in C-10 (Goal Drift Canary).
/// Emitted when the canary detects that the active baby-step's task
/// summary has diverged from the original task intent by more than
/// `drift_score` cosine distance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalDriftWarningPayload {
    pub agent_id: String,
    pub drift_score: f32,
    pub baby_step_id: u64,
    pub detail: String,
}

/// C-2 — payload for [`SsbActionType::WatchdogTimeout`].
///
/// Defined now; producer wiring lands in C-10 (Generation Watchdog).
/// Emitted when an agent exceeds its per-step wall-clock budget
/// without producing observable output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchdogTimeoutPayload {
    pub agent_id: String,
    pub elapsed_ms: u64,
    pub budget_ms: u64,
    pub detail: String,
}

// ---------------------------------------------------------------------------
// C-10 (SCHEMA_VERSION 52) payload structs — anti-spiral lateral-thinking +
// goal-drift Block. Producer wiring lands in anti_spiral_extended.rs +
// task_runner step-boundary loop.
// ---------------------------------------------------------------------------

/// C-10 — payload for [`SsbActionType::LateralThinkingSucceeded`].
///
/// Emitted when a previously-active stagnation pattern stops firing
/// after a persona suggestion was consumed. Carries the persona that
/// broke the loop plus the iteration count at success.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LateralThinkingSucceededPayload {
    pub agent_id: String,
    pub persona: String,
    pub iteration: u32,
    pub detail: String,
}

/// C-10 — payload for [`SsbActionType::LateralThinkingExhausted`].
///
/// Emitted when all 5 personas have been tried within the current task
/// window without breaking the stagnation pattern. Carries the count of
/// personas attempted (always 5 in the current implementation; field
/// kept for forward-compat if the persona roster expands).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LateralThinkingExhaustedPayload {
    pub agent_id: String,
    pub personas_tried: u32,
    pub iteration: u32,
    pub detail: String,
}

/// C-10 — payload for [`SsbActionType::GoalDriftBlock`].
///
/// Companion to [`GoalDriftWarningPayload`] for the Block verdict band.
/// `drift_score` carries the Jaccard-similarity value in the plan-direction
/// reading (see workflow/reasoning/C-10-decisions.md DRIFT-3). Lower
/// score = more drift; `drift_score < 0.4` triggers this Block row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalDriftBlockPayload {
    pub agent_id: String,
    pub drift_score: f32,
    pub baby_step_id: u64,
    pub detail: String,
}

// ---------------------------------------------------------------------------
// C-13 (SCHEMA_VERSION 53) payload struct — WIP commit squash completion.
// Producer wiring lands in `crate::wip_squash::squash_wip_commits_if_any`
// + the `elai wip-squash` CLI subcommand. Emitted unconditionally after a
// squash attempt regardless of the no-op vs. squashed outcome so the
// dashboard can render both rows.
// ---------------------------------------------------------------------------

/// C-13 — payload for [`SsbActionType::WipSquashComplete`].
///
/// Emitted after the leading-WIP-commit squash heuristic runs. `wip_count`
/// is the count of consecutive leading commits whose first message line
/// matches `/^[Ww][Ii][Pp]/`. `squashed` records whether a real squash
/// actually executed:
///   * `squashed == true`  ⇒ a `git reset --soft HEAD~{wip_count}` plus
///     a `git commit --no-verify -m "phase: squash {wip_count} WIP commits"`
///     fired; the working tree's HEAD now sits on a single squash commit.
///   * `squashed == false` ⇒ no-op (wip_count <= 5 or no WIP prefix found).
///
/// `step_id` is the baby-step identifier that triggered the squash; empty
/// when invoked directly via the `elai wip-squash` CLI subcommand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WipSquashCompletePayload {
    pub wip_count: u32,
    pub squashed: bool,
    pub step_id: String,
}

// ---------------------------------------------------------------------------
// C-22 (SCHEMA_VERSION 54) payload structs — feasibility-probe completeness
// ---------------------------------------------------------------------------

/// C-22 — payload for [`SsbActionType::AlternativeExplored`].
///
/// Emitted by `DefaultFeasibilityProber::probe` per NovelStep alternative
/// attempted (i.e. each branch the prober tried in the alternatives chain).
/// `alternative_index` is 0-indexed (0 = first attempt, the original step).
/// Up to `MAX_ALTERNATIVES_PER_STEP` (3) rows per NovelStep.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlternativeExploredPayload {
    pub step_id: String,
    pub alternative_index: u8,
    pub accepted: bool,
}

/// C-22 — payload for [`SsbActionType::FeasibilityEscalated`].
///
/// Emitted by the feasibility-probe escalation path when
/// `decide_with_mode` returns `FeasibilityDecision::Escalate` (Interactive
/// mode) or `Replan` (Autonomous mode). `variant` is the
/// `FeasibilityEscalation` discriminant tag in snake_case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeasibilityEscalatedPayload {
    pub reason: String,
    pub variant: String,
    pub step_id: String,
}

// ---------------------------------------------------------------------------
// C-23 (SCHEMA_VERSION 55) payload structs — SessionDag branch-UI beacons.
// ---------------------------------------------------------------------------

/// C-23 — payload for [`SsbActionType::BranchForked`].
///
/// Emitted at the orchestrator level (via `create_branch_with_ssb`) and by
/// the dashboard-v2 `fork_session_branch` Tauri command on successful
/// `SessionDag::create_branch`. `branch_name` is the user-facing branch
/// label (matches `BranchRef::name`). `tip_snap_id` is the freshly-created
/// snapshot id at the branch tip. `parent_snap_id` is the snapshot the
/// branch was forked from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchForkedPayload {
    pub branch_name: String,
    pub tip_snap_id: String,
    pub parent_snap_id: String,
}

/// C-23 — payload for [`SsbActionType::BranchMerged`].
///
/// Emitted by `SessionDag::merge_branch_with_ssb` on a successful
/// `MergeResult::Winner`. Supersedes the legacy `Checkpoint` row with a
/// `branch_merge:winner=...;abandoned=...` discriminator string (the
/// Checkpoint row was indistinguishable from a snapshot-save row in the
/// dashboard activity feed).
///
/// `winner_name` is the branch that survived the merge into trunk.
/// `trunk_snap_id` is the trunk snapshot id post-merge. `abandoned_names`
/// is the set of non-winner active branches at the time of the merge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchMergedPayload {
    pub winner_name: String,
    pub trunk_snap_id: String,
    pub abandoned_names: Vec<String>,
}

/// C-23 — payload for [`SsbActionType::BranchClosed`].
///
/// Emitted by `SessionDag::close_branch_with_ssb` on a successful
/// `close_branch` call. `reason` carries a short free-text explanation
/// (e.g. `"user-requested"`, `"dashboard:close_session_branch"`) so the
/// dashboard activity feed can render context without inspecting other
/// rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchClosedPayload {
    pub branch_name: String,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// C-25 (SCHEMA_VERSION 56) payload structs — compiled-pipeline +
// template-crystallizer + background-task beacons.
// ---------------------------------------------------------------------------

/// C-25 — source discriminator for [`PlanSkeletonPayload::source`].
///
/// Distinguishes the two pipeline-builder factory paths:
///   * `Llm`      — the standard LLM-decomposition path (existing behaviour).
///   * `Template` — the C-25 `CompiledExecutor` cache hit short-circuit.
/// Tagged via `#[serde(rename_all = "snake_case")]` so the wire form is
/// `"llm"` / `"template"` for dashboard/CLI parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanSkeletonSource {
    Llm,
    Template,
}

/// C-25 — payload for [`SsbActionType::PlanSkeleton`].
///
/// Emitted by `pipeline_builder::PipelineBuilder::build_with_emitter` on
/// every successful factory entry. `source` flips to
/// [`PlanSkeletonSource::Template`] when the C-25 `CompiledExecutor`
/// returns a cache hit; otherwise it is [`PlanSkeletonSource::Llm`]. C-25
/// ships the FIRST emitter for this row (see DRIFT-D in
/// `workflow/sessions/s-25672/context-package-C25.md`).
///
/// `plan_skeleton_hash` is a blake3-truncated-u64 over the spec name +
/// concatenated step names (stable across rustc versions — DefaultHasher
/// is NOT used, per C-25 Q5 resolution).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanSkeletonPayload {
    pub plan_id: String,
    pub step_count: u32,
    pub concept_class: String,
    pub plan_skeleton_hash: u64,
    pub source: PlanSkeletonSource,
}

/// C-25 — payload for [`SsbActionType::CompiledPipelineFired`].
///
/// Emitted by `compiled_pipeline::CompiledExecutor::try_execute` on a
/// successful cache hit followed by a successful run. `use_count` is the
/// `CompiledPipeline.use_count` AFTER the increment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledPipelineFiredPayload {
    pub pipeline_id: String,
    pub concept_class: String,
    pub plan_skeleton_hash: u64,
    pub use_count: u32,
}

/// C-25 — payload for [`SsbActionType::TemplateCrystallized`].
///
/// Emitted by `compiled_pipeline::TemplateCrystallizer::check_crystallization`
/// when the N-consecutive-success threshold (default N=3) is met and a new
/// CompiledPipeline is materialized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateCrystallizedPayload {
    pub concept_class: String,
    pub plan_skeleton_hash: u64,
    pub pipeline_id: String,
    pub threshold_n: u32,
}

/// C-25 — payload for [`SsbActionType::BackgroundTaskStarted`].
///
/// Emitted by `background_task::BackgroundTaskManager::gated_spawn` on a
/// successful (autonomous_mode=true) spawn. `command_summary` is the
/// caller-supplied free-text command label (already truncated to fit
/// PAYLOAD_MAX_BYTES). `soft_timeout_secs` is the auto-detach boundary;
/// `hard_timeout_secs` is the kill boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundTaskStartedPayload {
    pub task_id: String,
    pub command_summary: String,
    pub soft_timeout_secs: u32,
    pub hard_timeout_secs: u32,
}

/// C-25 — payload for [`SsbActionType::BackgroundTaskComplete`].
///
/// Emitted by `background_task::BackgroundTaskManager::poll_completed`
/// (normal exit path) AND `enforce_timeouts` (auto-detach path). `auto_detached`
/// is `true` when the manager dropped the JoinHandle at the soft boundary;
/// `timed_out` is `true` when the hard boundary was exceeded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundTaskCompletePayload {
    pub task_id: String,
    pub exit_status: i32,
    pub duration_secs: u32,
    pub auto_detached: bool,
    pub timed_out: bool,
}

/// C-25 audit follow-up — trigger reason for [`DecompileTriggeredPayload`].
///
/// `Confidence` fires when the cached pipeline's `first_try_success_rate`
/// is below `COMPILED_EXECUTOR_CONFIDENCE_THRESHOLD`.
/// `ConsecutiveFailures` fires when `record_failure` was called at least
/// `COMPILED_EXECUTOR_FAILURE_THRESHOLD` times since the last
/// `record_success` reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecompileTriggerReason {
    Confidence,
    ConsecutiveFailures,
}

/// C-25 audit follow-up — payload for [`SsbActionType::DecompileTriggered`].
///
/// Emitted by `compiled_pipeline::CompiledExecutor::emit_decompile_triggered`
/// whenever a cache HIT is rejected via the confidence floor or the
/// consecutive-failure threshold. `failure_count` is the observed
/// consecutive-failure counter at the time the row was emitted (0 for the
/// confidence-floor path, which does not require accumulated failures).
///
/// Closes WARN-1 from `workflow/sessions/s-25672/audit-C25.md` where the
/// C-25 v56 implementation routed this event through `AntiSpiralHit` and
/// dashboards / contract tests filtered on `decompile_triggered` saw
/// zero rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecompileTriggeredPayload {
    pub concept_class: String,
    pub plan_skeleton_hash: u64,
    pub failure_count: u32,
    pub trigger_reason: DecompileTriggerReason,
}

// ---------------------------------------------------------------------------
// C-26 (SCHEMA_VERSION 58) payload structs — constrained-decoding + futility
// ---------------------------------------------------------------------------

/// C-26 — payload for [`SsbActionType::FutilityDetected`].
///
/// Emitted by `pipeline_tree_navigator::NavigatorTree::record_baby_step_outcome`
/// when the per-baby-step same-error-class window saturates at the
/// `RetryStrategy.futility_same_error_threshold` (default 3). The producer
/// also issues a `BubbleUpAction::Escalate` and increments the cascade
/// counter at the surrounding level (matches the C-26 plan §Task 3 path A).
///
/// Fields:
///   - `error_class`         snake_case discriminant (e.g. `"compile_error"`,
///                           `"constrained_decode_reject"`); free-form
///                           string for forward-compat.
///   - `consecutive_count`   number of consecutive same-class failures at
///                           the moment the futility row was emitted.
///                           Equal to the threshold the navigator policy
///                           set.
///   - `level`               `"baby_step" | "step" | "phase" | "concept"`.
///   - `escalated`           `true` when the navigator also issued a
///                           `BubbleUpAction::Escalate` row in the same
///                           cycle. C-26 plan path A always sets `true`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FutilityDetectedPayload {
    pub error_class: String,
    pub consecutive_count: u32,
    pub level: String,
    pub escalated: bool,
}

/// C-26 — payload for [`SsbActionType::FutilityOutputStagnation`].
///
/// Emitted by `pipeline_tree_navigator::NavigatorTree::record_baby_step_outcome`
/// when `GoalDriftCanary::compute(prev_output, curr_output) > 0.90` between
/// two consecutive outputs of the same baby step. The producer also sets
/// `TreeNavigatorState.suggested_persona` via
/// `suggest_persona_for_pattern(StagnationPattern::NoDrift)`, which per
/// C-10 D4 always yields `ThinkingPersona::Hacker` (NOT Researcher).
///
/// Fields:
///   - `similarity`   Jaccard similarity in `[0.0, 1.0]`.
///   - `threshold`    the policy threshold (default `0.90`).
///   - `level`        `"baby_step" | "step" | "phase" | "concept"`.
///   - `new_persona`  snake_case persona label set by the producer (see
///                    `ThinkingPersona::as_str()` — defaults to `"hacker"`
///                    for NoDrift). `None` when C-10 is unavailable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FutilityOutputStagnationPayload {
    pub similarity: f32,
    pub threshold: f32,
    pub level: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_persona: Option<String>,
}

// ---------------------------------------------------------------------------
// C-7 (SCHEMA_VERSION 49) payload struct
// ---------------------------------------------------------------------------

/// C-7 — payload for [`SsbActionType::ChangeDecision`].
///
/// Emitted by the dashboard-v2 Tauri commands `accept_change` /
/// `decline_change` / `retry_audit` each time a human reviewer acts on
/// a per-file or per-hunk change row in ReviewWindow. The live diff
/// remains carried by [`SsbActionType::FileWrite`]; this row records
/// only the reviewer verdict so the dashboard can re-render verdict
/// state on reload + downstream consumers can audit reviewer actions.
///
/// `decision` discriminator (string for forward-compat — adding a new
/// verdict tag does not require a schema bump):
///   - `"accepted"` — emitted by `accept_change`
///   - `"declined"` — emitted by `decline_change`
///   - `"retried"`  — emitted by `retry_audit`
///
/// `hunk_id` is `None` for file-level verdicts and `Some(id)` for
/// per-hunk verdicts. The id is opaque to the orchestrator; the
/// dashboard derives it from the FileWrite diff (typically the
/// `@@ -N,M +P,Q @@` header content hash).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeDecisionPayload {
    pub decision: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hunk_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

// ---------------------------------------------------------------------------
// C-8 (SCHEMA_VERSION 50) payload struct
// ---------------------------------------------------------------------------

/// C-8 — payload for [`SsbActionType::SearchModeChanged`].
///
/// Emitted by the CLI startup-readiness path and by the dashboard-v2
/// Tauri command `set_search_mode` whenever the resolved retrieval mode
/// transitions. The dashboard `get_search_mode` command reads the latest
/// row of this type (graceful empty fallback returns `"hybrid"`). This
/// is the SSB-row bridge between the live_cli AppState (CLI process)
/// and the dashboard-v2 AppState (src-tauri process) — the two share no
/// in-memory state, so the canonical channel is the SSB DB.
///
/// `mode` discriminator (lowercase string for forward-compat — adding a
/// new SearchMode tag does not require a schema bump):
///   - `"hybrid"`   — bm25 + vector + reranker (default)
///   - `"lexical"`  — bm25 only (e.g. embedder offline)
///   - `"semantic"` — vector only (e.g. bm25 corrupted)
///   - `"ann"`      — ANN approximate-NN fallback
///
/// `history` is the trailing sequence of mode tags (most-recent last),
/// so the dashboard can render a "last N transitions" timeline without
/// N round-trips. May be empty on the first transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchModeChangedPayload {
    pub mode: String,
    #[serde(default)]
    pub history: Vec<String>,
}

/// C-5 — payload for [`SsbActionType::RedactionApplied`].
///
/// Emitted by [`crate::cloud_mode::RedactingBabyStepExecutor`] each
/// time the privacy redactor scrubs PII or secrets from a cloud-bound
/// baby-step payload. Best-effort: a write failure never aborts the
/// baby-step execution.
///
/// Fields:
///   - `count`: total number of redaction substitutions applied across
///     every redacted field on this baby-step (sum of per-field deltas).
///   - `fields`: logical field names that were touched. Today this is
///     either `["task_summary"]` or `["task_summary", "escalation_message"]`
///     depending on whether the baby-step carried an escalation message.
///     Forward-compatible: new redactable fields will simply append.
///   - `baby_step_id`: numeric id from
///     [`crate::task_runner::BabyStepExecutionCtx::baby_step_id`] so the
///     dashboard CloudPanel privacy-event feed can correlate each row
///     with its source step.
///   - `ts_ms`: wall-clock millisecond timestamp at redaction time. Set
///     by the producer (RedactingBabyStepExecutor) using its injected
///     clock so deterministic tests stay stable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionAppliedPayload {
    pub count: u64,
    #[serde(default)]
    pub fields: Vec<String>,
    pub baby_step_id: u64,
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// C-27 (SCHEMA_VERSION 59) — Testing-trust T(-1) + T-0 payloads.
// ---------------------------------------------------------------------------

/// C-27 — kind discriminator for [`WiringViolationPayload`].
///
/// `StubPattern`        — StubBanAuditor matched a forbidden regex in a
///                        production-path source file (todo!(),
///                        unimplemented!(), `Ok(Default::default())` exact-line,
///                        "deferred", "stub").
/// `UnreachableSymbol`  — DkgReachabilityAuditor could not BFS from any
///                        configured entrypoint to the declared
///                        produces_symbol.
/// `BrokenObservable`   — ObservableEffectRunner timed out waiting for the
///                        declared expect_ssb / expect_artifact row (no panic,
///                        violation row emitted instead).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WiringViolationKind {
    StubPattern,
    UnreachableSymbol,
    BrokenObservable,
}

/// C-27 — payload for [`SsbActionType::WiringEvidence`].
///
/// Emitted by `wiring_contract::DkgReachabilityAuditor` once per
/// produces_symbol declared in a WIRING_CONTRACT block that the auditor
/// verified is reachable (BFS) from at least one configured
/// `[wiring.entrypoints]` symbol path. Zero rows when the contract declares
/// no produces_symbols.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiringEvidencePayload {
    /// The configured entrypoint that reached the verified symbols.
    pub entrypoint: String,
    /// Symbols from `produces_symbols` that BFS proved reachable.
    #[serde(default)]
    pub produces_symbols_verified: Vec<String>,
    /// Total reachable count under this entrypoint (>= verified.len()).
    pub reachable_count: u32,
    /// Per-`elai wiring audit` invocation correlator.
    pub audit_id: uuid::Uuid,
}

/// C-27 — payload for [`SsbActionType::WiringViolation`].
///
/// Emitted by any of the three T-0 auditors when a wiring contract clause is
/// breached. `violation_kind` discriminates the three branches (StubPattern
/// | UnreachableSymbol | BrokenObservable). `file` + `line` carry the
/// location of the breach (file path for StubPattern; declaring contract
/// source for the other two). `pattern` is the breached pattern string
/// (forbidden regex literal for StubPattern; symbol name for
/// UnreachableSymbol; observable trigger name for BrokenObservable).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiringViolationPayload {
    pub violation_kind: WiringViolationKind,
    pub file: String,
    pub line: u32,
    pub pattern: String,
    pub audit_id: uuid::Uuid,
}

/// C-27 — payload for [`SsbActionType::DeterminismDrift`].
///
/// Emitted by `determinism::DriftComparator::compare` when two
/// `ActionTypeSequence`s recorded from nominally-equivalent runs diverge.
/// `seed` is the shared `ELAI_SEED` under test; `run_a_hash` /
/// `run_b_hash` are blake3 hashes of the joined snake-case action_type
/// stream up to (but not including) `first_divergence_index`. Identical
/// sequences emit zero rows (contract: no false positives).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismDriftPayload {
    pub seed: u64,
    pub run_a_hash: String,
    pub run_b_hash: String,
    pub first_divergence_index: u32,
    pub audit_id: uuid::Uuid,
}

// ---------------------------------------------------------------------------
// C-28 (SCHEMA_VERSION 60) — Testing-trust T-1 mutation testing payload.
// ---------------------------------------------------------------------------

/// C-28 — payload for [`SsbActionType::MutationSurvived`].
///
/// Emitted by `deterministic_auditor` Step 6 (mutation check) once per
/// mutant whose `MutantOutcome` is `Survived`. Caught / Timeout / Skipped
/// outcomes emit ZERO rows (stub_ban contract: no spam on healthy runs).
/// `file` is the source path relative to project_root; `line` is the
/// 1-based source line targeted by the mutation; `mutant_kind` is a
/// stable snake_case identifier emitted by the underlying tool (e.g.
/// `replace_+_with_-` for cargo-mutants binary-op replacements, or
/// `delete_line` for mutmut statement deletions).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationSurvivedPayload {
    pub file: String,
    pub line: u32,
    pub mutant_kind: String,
}

// ---------------------------------------------------------------------------
// C-29 (SCHEMA_VERSION 61) — Testing-trust T-2 (IntentPair) + T-3 (Metamorphic)
// payloads. See workflow/reasoning/C-29-decisions.md.
// ---------------------------------------------------------------------------

/// C-29 — payload for [`SsbActionType::IntentConfirmed`].
///
/// Emitted when a `BabyStepSpec.intent_pairs` set passes the
/// `IntentConfig.min_pairs` threshold at the PLANNING -> TESTING_FIRST
/// gate (`IntentPairsGate` in `evidence.rs`). `confirmed_by` is one of
/// `"user"` (dashboard Confirm button) or `"autonomous_gate"` (auto-confirm
/// branch). `pair_count` is the count of populated pairs at confirmation
/// time and must be >= the active `min_pairs` (the gate guarantees this).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentConfirmedPayload {
    pub step_id: u64,
    pub pair_count: u8,
    pub confirmed_by: String,
}

/// C-29 — payload for [`SsbActionType::MrViolated`].
///
/// Emitted by `deterministic_auditor` Step 7 when a metamorphic relation
/// returned `MrResult { passed: false }`. `relation` is one of
/// `"identity"` / `"inverse"` / `"permutation_invariance"`. `counterexample`
/// MUST be non-empty — `MrResult::failed` debug_asserts non-emptiness and
/// substitutes `"<empty>"` as a runtime sentinel so the SSB writer never
/// emits a row that violates the stub_ban contract (DRIFT-C29-11).
/// `sample_size` is the number of inputs the relation was evaluated over.
/// MrViolated is NOT a standalone Block — score aggregation deferred to
/// T-4 per C-29 plan §13(c).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MrViolatedPayload {
    pub step_id: u64,
    pub relation: String,
    pub counterexample: String,
    pub sample_size: u32,
}

// ---------------------------------------------------------------------------
// C-30 (SCHEMA_VERSION 62) — Surgical Mode + Oracle Mode pipeline payloads.
// See workflow/reasoning/C-30-decisions.md.
// ---------------------------------------------------------------------------

/// C-30 — payload for [`SsbActionType::SurgicalPlanCreated`].
///
/// Emitted once per successful `SurgicalDecomposer::decompose` return. The
/// payload pins the canonical SurgicalPlan shape so the dashboard
/// SurgicalPanel and downstream auditors can reconstruct the plan summary
/// without re-running the DKG traversal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurgicalPlanCreatedPayload {
    /// Stable identifier for the originating `/task` invocation
    /// (`task_<unix_secs>_<short_hash>` per surgical_decomposer.rs).
    pub task_id: String,
    /// Lowercase identifier: `"simple"` | `"medium"` | `"complex"`.
    pub complexity_class: String,
    /// Total files in `AffectedSet.direct_files + dependent_files`.
    pub blast_radius_files: u32,
    /// Sum of `(end_line - start_line + 1)` across all affected DKG nodes.
    pub total_loc_estimate: u32,
    /// Number of `SurgicalStep` entries (capped per complexity class).
    pub step_count: u32,
    /// Hex-encoded blake3 hash of the `RegressionGate.must_pass` vec
    /// (deterministic for identical must_pass content).
    pub regression_gate_snapshot_hash: String,
}

/// C-30 — payload for [`SsbActionType::SurgicalLocWarning`].
///
/// Emitted when `AffectedSet.total_loc_estimate` exceeds the
/// `SURGICAL_LOC_WARN_THRESHOLD` (500). Carries the ExecutionMode so the
/// dashboard can render distinct treatments for Interactive (user must
/// confirm) vs Autonomous (may escalate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurgicalLocWarningPayload {
    pub task_id: String,
    pub total_loc_estimate: u32,
    pub threshold: u32,
    /// `"interactive"` | `"autonomous"`.
    pub mode: String,
}

/// C-30 — payload for [`SsbActionType::OracleCacheHit`].
///
/// Emitted when `OracleAnswerCache::get(&key, now)` returns `Some(&entry)`
/// with `now - entry.cached_at < ttl_for_tier(entry.source_tier)`. The
/// `query_hash_hex` is the 64-char blake3 hex of the cache key's query bytes
/// (the project_hash component is kept private to avoid leaking path info
/// via the SSB stream).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleCacheHitPayload {
    pub query_hash_hex: String,
    /// Lowercase tier identifier: `"t1_authoritative"` | `"t2_strong"` |
    /// `"t3_weak"`. T4 / Unknown never cache, so they cannot appear here.
    pub source_tier: String,
    /// Age of the cached entry in seconds at lookup time.
    pub cached_age_secs: u64,
}

/// C-30 — payload for [`SsbActionType::OracleCacheMiss`].
///
/// Emitted alongside `OracleAnswerCache::insert(...)` after a get-returns-None
/// has been followed by a successful `run_research_pipeline` call. `confidence`
/// is the lowercase serde form of `AnswerConfidence` (`"high"` | `"medium"`
/// | `"low"` | `"unknown"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleCacheMissPayload {
    pub query_hash_hex: String,
    pub confidence: String,
}

/// C-30 — payload for [`SsbActionType::ProjectDetectionSignal`].
///
/// Emitted once per signal evaluated by the fresh-project detector when
/// `IndexingService::indexes_exist()` returns `false` at the first `/task`
/// invocation. `weight` mirrors the `SignalWeight` enum
/// (`"required"` | `"strong"` | `"weak"`). `fired = true` means the signal
/// matched (e.g. `Cargo.toml` present, or `rust_file_count >= 10`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDetectionSignalPayload {
    pub signal_name: String,
    pub weight: String,
    pub fired: bool,
}

// ---------------------------------------------------------------------------
// C-32 (SCHEMA_VERSION 63) — Code-Completeness Layer 2 payloads.
// See workflow/reasoning/C-32-decisions.md.
// ---------------------------------------------------------------------------

/// C-32 — payload for [`SsbActionType::SkeletonGenerated`].
///
/// Emitted once per successful `SkeletonGenerator::generate_for_functions`
/// return. The orchestrator pre-computes the file shape (function
/// signatures + BLANK_N body placeholders) and surfaces this row so the
/// dashboard can render "skeleton with N blanks" before any fill begins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkeletonGeneratedPayload {
    pub blank_count: u32,
    /// Lowercase language tag (`"rust"` | `"python"` | ...).
    pub language: String,
    /// Target path (UTF-8 lossy; absolute or workspace-relative).
    pub file_path: String,
}

/// C-32 — payload for [`SsbActionType::SkeletonFlushed`].
///
/// Emitted once per `SkeletonGenerator::flush_to_disk` success. The fill
/// loop performs ZERO disk writes; this single `fs::write` is the only
/// disk-side effect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletonFlushedPayload {
    pub path: String,
    pub completeness_ratio: f32,
}

/// C-32 — payload for [`SsbActionType::MiniCriticVerdict`].
///
/// Emitted once per `MiniCritic::check`. Carries the verdict tri-state
/// (`"pass"`/`"warn"`/`"block"`) and the short reason tag the dashboard
/// ReasoningDrawer renders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiniCriticVerdictPayload {
    pub blank_id: u8,
    pub verdict: String,
    pub reason: String,
    pub language: String,
}

/// C-32 — payload for [`SsbActionType::StructuralRetryStageAttempt`].
///
/// Emitted once per StructuralRetrySequencer stage invocation. The
/// `stage` field is one of `"decompose"`, `"template"`, `"few_shot"`,
/// `"best_of_n"`. The `outcome` field is the lowercase variant tag from
/// `StructuralRetryOutcome` (or `"converged"` / `"exhausted"` /
/// `"accept_and_complete"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralRetryStageAttemptPayload {
    pub blank_id: u8,
    pub stage: String,
    pub attempt: u32,
    pub outcome: String,
}

/// C-32 — payload for [`SsbActionType::BridgeFileGenerated`].
///
/// Emitted once per `BridgeFileTemplate::write_bridge` success. Carries
/// the bridge kind, the written path, and the module count assembled into
/// the bridge file body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeFileGeneratedPayload {
    /// Lowercase kind tag (`"mod_rs_aggregator"` | `"init_py_barrel"` |
    /// `"index_ts_barrel"` | `"go_package_doc"`).
    pub kind: String,
    pub path: String,
    pub module_count: u32,
}

// ---------------------------------------------------------------------------
// D-1 (SCHEMA_VERSION 64) — ReasoningTemplate ArchiveChain lifecycle payloads.
// ---------------------------------------------------------------------------

/// D-1 — payload for [`SsbActionType::ArchiveEntryAdded`].
///
/// Emitted once per [`crate::archive_chain::ArchiveChain::append`] success.
/// All hash strings are 64-char lowercase hex (Blake3); enum strings are
/// snake_case matching the wire labels defined on
/// `reasoning_template::{Domain,PatternKind,ArchiveStatus}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveEntryAddedPayload {
    pub template_id: String,
    pub content_hash: String,
    pub parent_hash: Option<String>,
    pub domain: String,
    pub pattern_kind: String,
    pub archive_status: String,
}

/// D-1 — payload for [`SsbActionType::ArchiveEntryRetired`].
///
/// Emitted once when a template transitions to `Retired`. `retired_at_ms` is
/// unix-millis from the injected [`elai_core::clock::ClockProvider`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveEntryRetiredPayload {
    pub template_id: String,
    pub retired_at_ms: i64,
    pub retirement_reason: String,
}

/// D-1 — payload for [`SsbActionType::TemplatePromoted`].
///
/// D-5 BAI Thompson promotion is the producer (lands in a later session);
/// the wire surface is added now so the schema bump is atomic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplatePromotedPayload {
    pub template_id: String,
    pub stage: String,
    pub pass_rate: f64,
}

/// D-1 — payload for [`SsbActionType::TemplateCandidateRejected`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateCandidateRejectedPayload {
    pub template_id: String,
    pub rejection_reason: String,
}

/// E-5 — payload for [`SsbActionType::MemoryFactExtracted`].
///
/// Emitted by the orchestrator's per-turn extraction hook right after
/// [`elai_memory::FactStore::insert`] returns Ok. Carries the row's
/// stable `fact_id` plus the indexed columns the dashboard MemoryDrawer
/// renders without re-querying the on-disk store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryFactExtractedPayload {
    /// UUIDv4 hex of the new row.
    pub fact_id: String,
    /// Snake-case [`elai_memory::FactType`] tag.
    pub fact_type: String,
    /// Snake-case [`elai_memory::Scope`] tag (`project` | `global`).
    pub scope: String,
    /// Optional project id (None for `scope=global`).
    pub project_id: Option<String>,
    /// User id (`"default"` on single-user installs).
    pub user_id: String,
    /// Stable key used for duplicate detection / UPDATE dispatch.
    pub key: String,
    /// Persisted trust score at row creation.
    pub trust_score: f32,
}

/// E-5 — payload for [`SsbActionType::MemoryFactSuperseded`].
///
/// Emitted by the orchestrator's update dispatcher right after
/// [`elai_memory::FactStore::supersede`] returns Ok. Both `fact_id`s are
/// surfaced so the dashboard MemoryDrawer can draw the supersession edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryFactSupersededPayload {
    /// `fact_id` of the row that has just been closed (`valid_to_ms != null`).
    pub old_fact_id: String,
    /// `fact_id` of the new currently-valid row.
    pub new_fact_id: String,
    /// `valid_to_ms` written on the old row.
    pub valid_to_ms: u64,
}

/// D-2 — payload for [`SsbActionType::PpmScored`].
///
/// Emitted by `crate::ppm_client::emit_ppm_score` after every successful
/// PPM scoring call that clears the configured threshold. `mode` is the
/// snake_case form of [`crate::sidecar_ppm_hook::PpmMode`]
/// (`"math"` / `"code"`). `score_mean` / `score_var` are widened to f64
/// for the SSB wire format (intrinsic values originate as f32 from the
/// sidecar; widening is lossless).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PpmScoredPayload {
    pub candidate_id: String,
    pub verifier_id: String,
    pub mode: String,
    pub score_mean: f64,
    pub score_var: f64,
    pub num_steps: u32,
    pub latency_ms: u64,
    pub cache_hit: bool,
}

/// D-2 — payload for [`SsbActionType::PpmGatingFailed`].
///
/// Emitted by `crate::ppm_client::emit_ppm_score` when the PPM `score_mean`
/// is below the configured `threshold`. Carries both numbers so dashboards
/// can render the gap without re-loading the threshold config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PpmGatingFailedPayload {
    pub candidate_id: String,
    pub verifier_id: String,
    pub mode: String,
    pub score_mean: f64,
    pub threshold: f64,
    pub num_steps: u32,
}

/// E-1 — payload for [`SsbActionType::ChatReflectionCompleted`].
///
/// Emitted by the orchestrator's chat-turn-committed hook right after
/// `elai_reasoning::chat_memory_reflect::reflect` returns Ok. Carries
/// the counts the dashboard ChatMemoryDrawer renders without
/// re-querying the underlying stores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatReflectionCompletedPayload {
    /// UUIDv4 hex of the new `memory_notes` row.
    pub note_id: String,
    /// First ~80 chars of the note body (for the SSB log view).
    pub note_preview: String,
    /// Number of [`elai_memory::ExtractedFact`] candidates the heuristic
    /// emitted in the same reflection pass.
    pub facts_extracted: u32,
    /// `decay_tick` rows examined.
    pub decay_rows_examined: u32,
    /// `decay_tick` rows whose persisted trust score actually moved.
    pub decay_rows_updated: u32,
    /// Byte-size of the CoreBlock body after the post-reflection
    /// append (head-truncated to [`elai_memory::CORE_BLOCK_MAX_BYTES`]).
    pub core_block_bytes: u32,
}

/// E-1 — payload for [`SsbActionType::MemoryEvolutionRun`].
///
/// Emitted on every IDLE -> SLEEP_COMPUTE substate entry AND the
/// matching SLEEP_COMPUTE -> IDLE exit. `phase` is `"start"` or
/// `"end"`; `duration_ms` is `0` on `"start"` and the elapsed time
/// from start to end on `"end"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryEvolutionRunPayload {
    /// `"start"` | `"end"`.
    pub phase: String,
    /// Elapsed ms from the matching `"start"` row (0 on `"start"`).
    pub duration_ms: u64,
    /// Number of `memory_notes` rows whose `last_accessed_ms` the
    /// PPR walker bumped during this evolution pass.
    pub notes_touched: u32,
}

/// E-1 dashboard slice — payload for [`SsbActionType::MemoryNoteInserted`].
///
/// Emitted by the orchestrator chat-turn-committed hook right after
/// `elai_memory::NoteStore::insert` returns Ok. The producer wiring
/// itself lives in `orchestrator/src/task_runner.rs` and is owned by
/// another orchestration session; this payload is shipped now so the
/// dashboard surface can subscribe without further schema churn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryNoteInsertedPayload {
    /// UUIDv4 hex of the inserted `memory_notes` row.
    pub note_id: String,
    /// `"session"` | `"project"` | `"global"`.
    pub scope: String,
    /// BLAKE3 hex of the note body (so the dashboard can dedupe across
    /// reflection passes without re-fetching the row).
    pub content_blake3: String,
    /// Number of Zettelkasten links the note carries at insert time
    /// (PPR walker adjacency seed count).
    pub link_count: u32,
}

/// E-1 dashboard slice — payload for [`SsbActionType::MemoryNoteEvolved`].
///
/// Emitted by the A-MEM note evolution rewrite pass — the PPR-walker
/// attached link/edit step described in arXiv:2502.12110 §3.3.
/// Producer wiring is in the same task_runner.rs hook as the insert
/// path; carries both old and new content hashes so the dashboard can
/// render a diff-friendly evolution history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryNoteEvolvedPayload {
    /// UUIDv4 hex of the evolved `memory_notes` row.
    pub note_id: String,
    /// BLAKE3 hex of the previous body (pre-evolution).
    pub old_content_blake3: String,
    /// BLAKE3 hex of the new body (post-evolution).
    pub new_content_blake3: String,
    /// New `note_id` references added to the row's `links` array.
    pub links_added: u32,
    /// `note_id` references removed from the row's `links` array.
    pub links_removed: u32,
}

/// E-1 dashboard slice — payload for [`SsbActionType::MemoryConflictDetected`].
///
/// Emitted by the conflict adjudicator when two `memory_facts` /
/// `memory_notes` rows describe overlapping or contradictory state.
/// `conflict_kind` is the snake_case discriminator — current vocabulary
/// is `"duplicate"` | `"contradiction"`; future kinds (e.g.
/// `"version_drift"`) can land without a schema bump because the field
/// is a free-form string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryConflictDetectedPayload {
    /// `fact_id` (or `note_id`) of the older row.
    pub fact_id_a: String,
    /// `fact_id` (or `note_id`) of the newer row.
    pub fact_id_b: String,
    /// `"duplicate"` | `"contradiction"` (snake_case discriminator).
    pub conflict_kind: String,
}

/// E-1 dashboard slice — payload for [`SsbActionType::CoreBlockReplaced`].
///
/// Emitted on every `CoreBlockStore::upsert` that REPLACES an existing
/// row (the first-insert path emits a separate beacon family — this
/// variant strictly records edits to an already-pinned core block, so
/// the dashboard's drift / edit-count indicator stays accurate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreBlockReplacedPayload {
    /// Owner of the core block (single-user installs use `"default"`).
    pub user_id: String,
    /// BLAKE3 hex of the previous body.
    pub old_blake3: String,
    /// BLAKE3 hex of the new body.
    pub new_blake3: String,
    /// Byte size of the new body (always <= CORE_BLOCK_MAX_BYTES).
    pub bytes: u32,
}

/// D-3 — payload for [`SsbActionType::VerifierConsensusPass`].
///
/// Emitted by `crate::agent_pool::run_consensus_pass` after the aggregator
/// returns `AggregateVerdict::Pass`. `verdict` is the snake-case verdict
/// label (always `"pass"` here; carried as a String so dashboard rendering
/// is uniform with the failure-row schema). `rule_id` is the stable
/// snake_case `ConsensusRule::rule_id()` label so downstream consumers
/// don't need to re-decode the rule body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierConsensusPassPayload {
    pub candidate_id: String,
    pub template_id: String,
    pub verdict: String,
    pub verifier_count: u32,
    pub pass_count: u32,
    pub rule_id: String,
}

/// D-3 — payload for [`SsbActionType::VerifierConsensusFail`].
///
/// Emitted by `crate::agent_pool::run_consensus_pass` when the aggregator
/// returns `AggregateVerdict::Fail` or `Inconclusive`. `failing_verifier_ids`
/// is non-empty for `Fail`; for `Inconclusive` it is empty and `reason`
/// carries the inconclusive-detail string (e.g. `"missing_kinds=ppm"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierConsensusFailPayload {
    pub candidate_id: String,
    pub template_id: String,
    pub failing_verifier_ids: Vec<String>,
    pub reason: String,
}

/// D-4 — payload for [`SsbActionType::TripwireFired`].
///
/// Emitted by `crate::tripwire::TripwireMonitor::observe` when one of the
/// five detectors (KL / Δρ / IQR / perplexity / DDM) crosses its threshold.
/// `event_id` is a blake3-derived 32-hex stable id so dashboard consumers
/// can dedupe by id rather than seq.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TripwireFiredPayload {
    pub event_id: String,
    pub tripwire_kind: String,
    pub candidate_id: String,
    pub observed_value: f64,
    pub threshold: f64,
    pub fired_at_ms: i64,
}

/// D-4 — payload for [`SsbActionType::TemplateAutoReverted`].
///
/// Emitted by `crate::tripwire::auto_revert` after a tripwire firing flips
/// the matching `ArchiveEntry.status` to `Retired` with
/// `RetirementReason::Degradation`. `parent_template_id` is the lineage
/// target the orchestrator switches to (None when the retired template was
/// a root).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateAutoRevertedPayload {
    pub template_id: String,
    pub parent_template_id: Option<String>,
    pub retirement_reason: String,
    pub fired_tripwire_id: String,
}

/// D-19 — payload for [`SsbActionType::PrmBenchRunCompleted`].
///
/// Emitted by
/// [`crate::benchmarks::prmbench_runner::PRMBenchRunner::run_all`] (and
/// [`crate::benchmarks::prmbench_runner::PRMBenchRunner::run_category`])
/// after a PRMBench fixture has been scored end-to-end. `gating_verdict`
/// is the snake_case form of
/// [`crate::benchmarks::prmbench_runner::GatingVerdict`]
/// (`"pass"` / `"fail"`). `fixture_blake3` is the 64-hex content hash of
/// the JSONL fixture the run was computed against (anti-stale: pins
/// dashboard rows to a specific dataset revision).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrmBenchRunCompletedPayload {
    /// Total fixture rows the runner consumed (includes `insufficient`
    /// rows where the PPM hook returned an error).
    pub total: u32,
    /// `(tp + tn) / scored` aggregate accuracy.
    pub accuracy: f64,
    /// `tp / (tp + fp)` aggregate precision.
    pub precision: f64,
    /// `tp / (tp + fn)` aggregate recall.
    pub recall: f64,
    /// Aggregate verdict — `"pass"` iff every per-category verdict
    /// passed AND at least one row was scored.
    pub gating_verdict: String,
    /// 64-hex blake3 of the fixture file contents.
    pub fixture_blake3: String,
    /// Number of distinct categories surfaced in the per-category
    /// breakdown (matches `report.per_category.len()`).
    pub category_count: u32,
}

/// D-5 — payload for [`SsbActionType::BaiArmUpdated`].
///
/// Emitted once per `bai::emit_bai_round` posterior update. Carries the BAI
/// arm state after the Beta(alpha, beta) update + the `was_promoted` flag
/// mirroring whether the Thompson sample fired the promotion rule on this
/// round (so dashboard consumers can render the running mean + last-action
/// without a second lookup). `template_id` is 32-char simple-hex
/// (uuid::Uuid::simple). Promote/reject decisions emit a follow-up
/// `template_promoted` / `template_candidate_rejected` SSB row from the same
/// producer.
///
/// Fields ordered alpha-first/beta-second to match the conjugate Beta
/// notation in arXiv:2104.05593 §2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaiArmUpdatedPayload {
    /// Hex (32-char simple) of the candidate template id.
    pub template_id: String,
    /// Beta-distribution alpha (= pass_count + 1 with the Beta(1, 1) prior).
    pub alpha: f64,
    /// Beta-distribution beta (= fail_count + 1 with the Beta(1, 1) prior).
    pub beta: f64,
    /// Cumulative pulls observed on this arm (== alpha + beta - 2).
    pub n_pulls: u32,
    /// Posterior mean = alpha / (alpha + beta).
    pub mean_estimate: f64,
    /// Whether the Thompson sample picked this arm AND the posterior cleared
    /// the promotion threshold on this round.
    pub was_promoted: bool,
}

/// D-6 — payload for [`SsbActionType::HeldOutTaskAdded`].
///
/// Emitted once per `held_out::insert_and_emit` Ok return — a fresh
/// held-out task that passed the LiveCodeBench-style 9-gram contamination
/// filter and was inserted into the rusqlite-backed store. The dashboard
/// `HeldOutPanel` reads these rows to render the "recently ingested"
/// timeline + per-source counters. The `source` label is the snake-case
/// string from [`crate::held_out::TaskSource::label`] so the dashboard
/// can group without re-deserializing the full enum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldOutTaskAddedPayload {
    /// Simple-hex uuid of the held-out task (32 chars).
    pub task_id: String,
    /// Snake-case source label (`github_issue` / `benchmark_fixture` /
    /// `synthetic`).
    pub source: String,
    /// Blake3 (64-char hex) of the normalized task body.
    pub content_blake3: String,
    /// Number of 9-gram entries in the task's signature. Used by the
    /// dashboard to flag tasks below the n-gram threshold (too short to
    /// participate in the contamination filter).
    pub ngram_count: u32,
    /// Wall-clock UNIX ms when the task was ingested.
    pub ingested_ts_ms: i64,
    /// Time-window (ms) before the task becomes eligible for BAI eval
    /// consumption.
    pub eligibility_window_ms: i64,
}

/// D-6 — payload for [`SsbActionType::HeldOutTaskUsed`].
///
/// Emitted once per `held_out::mark_used_and_emit` Ok return — a held-out
/// task consumed by a BAI eval round. Carries the consumer template id so
/// the dashboard can link held-out usage to the BAI arm that consumed it
/// (cross-referenced with the `bai_arm_updated` row from the same round).
/// `consumer_template_id` is the empty string when the consumer is the
/// pre-promotion scaffold (no template yet).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldOutTaskUsedPayload {
    /// Simple-hex uuid of the held-out task that was consumed.
    pub task_id: String,
    /// Simple-hex uuid of the consumer template (or empty string for the
    /// pre-promotion scaffold).
    pub consumer_template_id: String,
    /// Wall-clock UNIX ms when the task was consumed.
    pub used_ts_ms: i64,
}

/// D-7 — payload for [`SsbActionType::CanaryRunCompleted`].
///
/// Emitted once per `canary::emit_canary_run` Ok return — a completed
/// cross-domain transfer canary report. The dashboard `CanaryPanel`
/// joins these rows to the per-family
/// [`CanaryFamilyEvaluatedPayload`] rows (matched on `report_id`) to
/// render the per-family deltas + overall transfer score timeline.
/// NanoFlux-style (arxiv:2509.23252, T1) defense: target-family wins
/// must not silently regress adjacent families.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryRunCompletedPayload {
    /// Blake3 (64-char hex) of the serialized report body. Deterministic
    /// identifier the per-family rows reference via `report_id` so the
    /// dashboard can join without scanning the SSB tail twice.
    pub report_id: String,
    /// Wall-clock UNIX ms when the run completed.
    pub ts_ms: i64,
    /// Mean transfer score across all evaluated families on this run.
    /// Range nominally `[-1.0, +1.0]` — positive means the candidate
    /// outperformed the baseline on average, negative the opposite.
    pub overall_transfer_score: f64,
    /// Snake-case decision kind: `pass` / `fail` / `marginal_pass`.
    pub decision_kind: String,
    /// Empty string when the decision is not `fail`. When `fail`, this
    /// carries the snake-case family label that drove the failure (the
    /// single dominant regressor — the largest negative `delta` family).
    pub dominant_fail_family: String,
}

/// D-9 — payload for [`SsbActionType::LockedKeysAuditRun`].
///
/// Emitted once per `LockedKeysAuditor::audit_diff_ranged` invocation
/// (success or violation; the audit ran, the row is the durable
/// signal). `registry_blake3` is the blake3 hex of the registry's
/// canonical content so dashboard consumers can detect drift between
/// runs without re-reading the TOML.
///
/// Grounding: arxiv:1908.04734 (Everitt current-RF principle); see
/// docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedKeysAuditRunPayload {
    /// Per-audit-run correlator. The matching
    /// [`LockedKeysViolationDetectedPayload`] rows carry the same id
    /// so the dashboard joins on it.
    pub audit_id: uuid::Uuid,
    /// Wall-clock UNIX ms when the audit ran.
    pub ts_ms: i64,
    /// Number of staged file/range pairs the auditor inspected.
    pub n_audited: u32,
    /// Number of violations emitted for this run (0 on a clean audit).
    pub n_violations: u32,
    /// 64-char lowercase hex blake3 of the registry's canonical
    /// content. Stable across formatter-only changes that don't alter
    /// path / line_range / reason.
    pub registry_blake3: String,
    /// Path to the registry file the auditor loaded (string for
    /// portability across the IPC boundary).
    pub registry_path: String,
}

/// D-9 — payload for [`SsbActionType::LockedKeysViolationDetected`].
///
/// Emitted once per overlap between a staged edit and a registry
/// entry. `audit_id` matches the parent [`LockedKeysAuditRunPayload`]
/// row so the dashboard renders the full audit context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedKeysViolationDetectedPayload {
    /// Matches the parent [`LockedKeysAuditRunPayload::audit_id`].
    pub audit_id: uuid::Uuid,
    /// Wall-clock UNIX ms when the violation was raised.
    pub ts_ms: i64,
    /// Workspace-relative path of the staged edit that touched a
    /// locked surface (forward-slash normalised).
    pub attempted_edit: String,
    /// Inclusive line range of the staged edit as `[start, end]`.
    /// Length always 2. Vec form keeps the IPC schema simple.
    pub attempted_lines: Vec<u32>,
    /// Forward-slash normalised path of the registry entry that
    /// matched.
    pub locked_key_path: String,
    /// 64-char lowercase hex blake3 of the registry entry's canonical
    /// content. Allows the dashboard to detect that a violation was
    /// raised against a since-modified registry row.
    pub locked_key_signature_blake3: String,
    /// Human-readable reason copied from the matched registry entry.
    pub reason: String,
}

/// D-7 — payload for [`SsbActionType::CanaryFamilyEvaluated`].
///
/// Emitted once per family inside a single canary run (so a 5-family
/// canary emits 5 of these rows plus 1
/// [`CanaryRunCompletedPayload`]). Matched to the parent run via
/// `report_id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryFamilyEvaluatedPayload {
    /// Matches the parent [`CanaryRunCompletedPayload::report_id`].
    pub report_id: String,
    /// Snake-case family label (`code_fix` / `math_proof` /
    /// `prose_reason` / `planning_decomposition` / `tool_use`).
    pub family: String,
    /// How many held-out tasks were drawn from this family on this run.
    pub n_evaluated: u32,
    /// Candidate pass count.
    pub pass_count: u32,
    /// Candidate fail count.
    pub fail_count: u32,
    /// Candidate pass-rate minus baseline pass-rate. Positive means the
    /// candidate beat the baseline; negative is regression. Bounded by
    /// `[-1.0, +1.0]`.
    pub transfer_score: f64,
    /// Baseline pass-rate (the reference template's pass-rate on the
    /// same held-out subset). Provided alongside the delta so the
    /// dashboard can render both bars without recomputing.
    pub baseline_score: f64,
}

/// E-46 — payload for [`SsbActionType::TaskAdvanced`].
///
/// One row per `DualLedger::advance_task` call. TaskLedger.status and
/// ProgressLedger.completion_pct + blockers are updated atomically and
/// the row carries the post-update view so dashboard consumers can
/// render the dual-ledger state without re-reading the ledger.
///
/// Grounding: arxiv:2412.21154 (Devin task-ledger pattern; SWE-bench
/// 13.86% solve rate driven by explicit task-state tracking).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAdvancedPayload {
    /// Task id (string for IPC portability — the orchestrator uses a
    /// `TaskId` newtype internally).
    pub task_id: String,
    /// Optional parent task id; empty string when the task is a root.
    pub parent_id: String,
    /// Post-update completion percent in `[0.0, 100.0]`.
    pub completion_pct: f64,
    /// Snake-case status label: `pending` / `in_progress` / `complete`
    /// / `blocked`.
    pub status: String,
    /// Newly-added blocker on this advance (empty when none).
    pub blocker: String,
    /// Wall-clock UNIX ms when the advance was recorded.
    pub ts_ms: i64,
}

/// E-46 — payload for [`SsbActionType::AgentlessShortcutTaken`].
///
/// Emitted when `agentless_shortcut::shortcut_eligible` returns
/// `Eligibility::Yes` for a task AND the orchestrator elects to take
/// the shortcut (i.e. skip spawning a sub-agent). The `reason` carries
/// the canonical eligibility justification (single-file / loc / no
/// cross-crate / no async net) so the dashboard can render the
/// shortcut taxonomy.
///
/// Grounding: arxiv:2407.01489 (AGENTLESS, 32.00% SWE-bench Lite at
/// $0.70/issue — non-agentic 3-phase localize/repair/validate beats
/// every open-source agentic system at that time).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentlessShortcutTakenPayload {
    /// Task id that elected the shortcut.
    pub task_id: String,
    /// Snake-case reason label: `single_file_small` / `single_file` /
    /// `no_cross_crate` etc. Used by the dashboard to render shortcut
    /// taxonomy histograms.
    pub reason: String,
    /// Wall-clock UNIX ms when the shortcut was taken.
    pub ts_ms: i64,
}

/// E-46 — payload for [`SsbActionType::GoalDriftFired`].
///
/// Emitted when `GoalDriftCanary::fired()` evaluates true: drift
/// between the initial-goal token-set and the current summary
/// token-set exceeded the configured threshold. The
/// `initial_goal_blake3` lets dashboard consumers join consecutive
/// firings to a single canary instance.
///
/// Grounding: arxiv:2505.02709 (goal-drift PRM supervision); see also
/// D-7 [`CanaryFamily::PlanningDecomposition`] for the orthogonal
/// canary surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalDriftFiredPayload {
    /// 64-char lowercase hex blake3 of the canonical initial goal
    /// string. Stable across firings so the dashboard joins them.
    pub initial_goal_blake3: String,
    /// Measured drift in `[0.0, 1.0]` — `1.0` means orthogonal token
    /// sets, `0.0` means identical (Jaccard-distance variant used by
    /// `measure_drift`).
    pub drift: f64,
    /// Configured drift threshold the canary was checked against.
    pub threshold: f64,
    /// Step index at which the firing was observed (0-indexed within
    /// the parent decomposition walk).
    pub step: u32,
    /// Wall-clock UNIX ms when the firing was recorded.
    pub ts_ms: i64,
}

/// D-10 — payload for [`SsbActionType::ClaimRouted`].
///
/// One row per `agent_pool::route_claim` call. The orchestrator (not
/// the policy model) decides the [`crate::grounding_mode::GroundingMode`]
/// via the pure predicate
/// [`crate::grounding_mode::select_grounding_mode`]; this row is the
/// durable audit signal that the predicate ran and what it produced.
///
/// `ctx_summary_blake3` is the lowercase 64-char hex blake3 of the
/// canonical wire-form of the `OrchContext` snapshot the router saw
/// — so the dashboard can detect post-hoc context drift between
/// routing and dispatch.
///
/// Grounding:
///   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
///     §1 (5-mode taxonomy) + §3.2 (routing predicate).
///   * arxiv:1908.04734 (Everitt current-RF principle, T1).
///   * arxiv:1906.01820 (Hubinger mesa-optimization, T1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimRoutedPayload {
    /// Stable per-claim identifier (uuid string in production; any
    /// non-empty stable string in tests).
    pub claim_id: String,
    /// Snake-case [`crate::claim::ClaimType`] label — matches the
    /// type's serde tag so the dashboard can deserialize without a
    /// separate mapping.
    pub claim_type: String,
    /// Snake-case [`crate::grounding_mode::GroundingMode`] label.
    pub mode: String,
    /// Wall-clock UNIX ms when routing completed.
    pub ts_ms: i64,
    /// 64-char lowercase hex blake3 of the canonical wire-form of
    /// the `OrchContext` snapshot used at routing time. Empty string
    /// is permitted but indicates a hashing failure on the producer
    /// side (logged via `tracing::warn`).
    pub ctx_summary_blake3: String,
}

/// D-21 — payload for [`SsbActionType::ModeRoutingBenchRunCompleted`].
///
/// Emitted by
/// [`crate::benchmarks::mode_routing_runner::ModeRoutingBenchmark::run_all`]
/// after a mode-routing fixture has been scored end-to-end. The runner
/// replays a hand-labeled JSONL fixture through D-10's pure predicate
/// [`crate::grounding_mode::select_grounding_mode`] and reports macro F1
/// + per-mode F1 + a 5x5 confusion matrix. `gating_verdict` is the
/// snake-case form of
/// [`crate::benchmarks::prmbench_runner::GatingVerdict`] (`"pass"` iff
/// macro F1 >= 0.85 AND every per-mode F1 >= 0.70). `fixture_blake3`
/// pins the dashboard row to a dataset revision.
///
/// Grounding:
///   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §3.2
///     (routing predicate baseline).
///   * D-21 plan ACCEPTANCE: macro F1 >= 0.85 + per-mode F1 >= 0.70.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModeRoutingBenchRunCompletedPayload {
    /// Total fixture rows the runner consumed.
    pub total: u32,
    /// `(sum_per_mode_TP) / total_predictions` aggregate accuracy.
    pub accuracy: f64,
    /// Macro-averaged F1 across the 5 [`crate::grounding_mode::GroundingMode`]
    /// variants. Modes with zero ground-truth AND zero predictions
    /// contribute `0.0` (defined behaviour — keeps macro F1
    /// monotonically degraded by missing-class coverage).
    pub f1_macro: f64,
    /// Aggregate verdict — `"pass"` iff `f1_macro >= 0.85` AND every
    /// per-mode F1 is `>= 0.70`. `"fail"` otherwise.
    pub gating_verdict: String,
    /// 64-hex blake3 of the fixture file contents.
    pub fixture_blake3: String,
    /// Number of distinct rows actually scored (== `total` minus any
    /// invalid-label rows that were rejected at load time).
    pub fixture_row_count: u32,
}

/// D-11 — payload for [`SsbActionType::CheckerResultEmitted`].
///
/// Emitted once per `(checker_id, kind, result)` triple returned by
/// [`crate::checker_registry::CheckerRegistry::run_all_applicable`].
/// The orchestrator (NOT the policy model) is the sole producer; the
/// dispatch path is `crate::agent_pool::dispatch_constructive_checkers`,
/// which is invoked when the D-10 router selects
/// [`crate::grounding_mode::GroundingMode::Constructive`].
///
/// Field semantics:
///   * `checker_id` — stable id the orchestrator assigned at register
///     time (`"regex-foo"`, `"sympy-default"`, etc).
///   * `checker_kind` — snake-case [`crate::checker_registry::CheckerKind`]
///     label (one of: `sympy`, `z3`, `compile_rust`, `regex_match`,
///     `json_schema_validate`, `numeric_range`).
///   * `result_kind` — `pass | fail | inapplicable | error`.
///   * `claim_id` — stable per-claim identifier.
///   * `ts_ms` — wall-clock UNIX ms at result-emit time.
///   * `reason` — bounded human-readable text for `fail`,
///     `inapplicable`, or `error`; empty string for `pass`.
///
/// Grounding:
///   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §4.2
///     (checker layout) + §1 (Mode A constructive grading).
///   * arxiv:2310.04406 (constructive proof checking).
///   * arxiv:2211.12588 (Program-of-Thought, T1, TMLR 2023).
///   * arxiv:2402.01030 (CodeAct, T1, ICML 2024).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckerResultEmittedPayload {
    /// Stable id assigned at register time.
    pub checker_id: String,
    /// Snake-case [`crate::checker_registry::CheckerKind`] label.
    pub checker_kind: String,
    /// `pass | fail | inapplicable | error`.
    pub result_kind: String,
    /// Stable per-claim identifier the registry dispatched against.
    pub claim_id: String,
    /// Wall-clock UNIX ms at result-emit time.
    pub ts_ms: i64,
    /// Bounded human-readable explanation (truncated to 2048 chars by
    /// the producer). Empty string for `pass`.
    pub reason: String,
}

/// D-13 — payload for [`SsbActionType::AdversaryGenerated`].
///
/// Emitted by
/// [`crate::agent_pool::run_adversarial_pass`] after an
/// [`crate::adversary::Adversary::generate_counter`] call returns Ok.
/// `severity` is the structured counter's invalidation strength on
/// `[0.0, 1.0]`; `model_family` is the snake-case
/// [`crate::adversary::ModelFamily`] label the adversary emitted from
/// (cross-family check enforced at the consensus call-site, not on
/// this row).
///
/// Grounding:
///   * arxiv:2403.07974 (T1) adversarial verification.
///   * arxiv:2504.16760 (T1) lightweight latent verifiers
///     (heterogeneous-family verification).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdversaryGeneratedPayload {
    /// Stable per-counter id (typically blake3-derived).
    pub counter_id: String,
    /// `claim_id` of the original [`crate::claim::Claim`] this counter
    /// was generated against.
    pub claim_id: String,
    /// `[0.0, 1.0]` invalidation severity. `0.0` = "claim robust"
    /// path; `>= severity_threshold` = caller treats the original
    /// claim as invalidated.
    pub severity: f64,
    /// Snake-case [`crate::adversary::ModelFamily`] label.
    pub model_family: String,
}

/// D-13 — payload for [`SsbActionType::ConsensusReevaluatedWithCounter`].
///
/// Emitted by
/// [`crate::agent_pool::run_adversarial_pass`] after the verifier
/// consensus has been re-run on `(claim, counter_evidence)`. Both
/// verdict fields carry the snake-case
/// [`crate::verification_result::AggregateVerdict::as_str`] label so
/// dashboards can render the verdict transition without re-decoding
/// the rule body. `claim_id` is the original claim, NOT the counter
/// (the consensus is being re-evaluated AGAINST the original claim in
/// the presence of new counter-evidence).
///
/// Grounding:
///   * arxiv:2403.07974 (T1) adversarial verification.
///   * arxiv:2504.16760 (T1) heterogeneous-family verifier transfer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusReevaluatedWithCounterPayload {
    /// Stable claim id the verdict was re-computed against.
    pub claim_id: String,
    /// Verdict of the FIRST consensus pass (pre-adversary). One of
    /// `"pass" | "fail" | "inconclusive"`.
    pub original_verdict: String,
    /// Verdict of the consensus pass that consumed the adversary
    /// counter-evidence. Same label-space as `original_verdict`.
    pub post_counter_verdict: String,
}

/// D-12 — payload for [`SsbActionType::BabyStepDispatched`].
///
/// Emitted once per [`crate::baby_step::dispatch_check_for_baby_step`]
/// outcome by [`crate::agent_pool::run_operational_check`]. The row is
/// the durable audit signal the dashboard's `BabyStepPanel` renders;
/// the auditor consumes the `outcome` field to decide whether to allow
/// the MEMORIZING transition.
///
/// Field semantics:
///   * `step_id` — stable per-baby-step identifier.
///   * `parent_task_id` — parent task / phase id the baby step belongs
///     to (so the panel can group dispatches per parent).
///   * `checker_kinds_used` — snake_case
///     [`crate::checker_registry::CheckerKind`] labels for every
///     checker the dispatch fired. Empty when no checker ran (the
///     dispatch returned `NotOperationalized` or `Inapplicable`).
///   * `outcome` — one of `pass | fail | inapplicable`.
///   * `ts_ms` — wall-clock UNIX ms at dispatch-emit time.
///
/// Grounding:
///   * arxiv:2305.16291 Voyager (T1) — skill-with-measurable-effect.
///   * arxiv:2407.01489 AGENTLESS (T1) — single-step verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BabyStepDispatchedPayload {
    pub step_id: String,
    pub parent_task_id: String,
    pub checker_kinds_used: Vec<String>,
    /// `pass | fail | inapplicable`.
    pub outcome: String,
    pub ts_ms: i64,
}

/// D-16 — payload for [`SsbActionType::ConformalTauComputed`].
///
/// Emitted once per
/// [`crate::conformal_calibration::compute_tau_and_emit`] Ok return.
/// The row is the durable audit trail the dashboard
/// `ConformalCalibrationPanel` joins against the per-claim
/// [`ConformalRefusalDecidedPayload`] rows.
///
/// Grounding:
///   * arXiv:2503.01345 — UniCR (Unified Conformal Refusal).
///   * Vovk et al. *Algorithmic Learning in a Random World* (2005)
///     §2.2 — quantile-based conformal predictor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalTauComputedPayload {
    /// The threshold `tau` returned by
    /// [`crate::conformal_calibration::compute_tau`]; non-conformity
    /// scores above `tau` map to `RefusalDecision::Abstain`.
    pub tau: f64,
    /// UniCR operating point — default `0.1` per arXiv:2503.01345 §6.
    pub epsilon: f64,
    /// Number of tuples in the corpus used to compute `tau`.
    pub corpus_size: u32,
    /// Snake-case [`crate::conformal_calibration::CorpusSource`] label.
    pub source: String,
    /// Wall-clock UNIX ms at compute-emit time.
    pub ts_ms: i64,
}

/// D-16 — payload for [`SsbActionType::ConformalRefusalDecided`].
///
/// Emitted once per
/// [`crate::conformal_calibration::decide_and_emit`] call (and by
/// extension every [`crate::agent_pool::dispatch_conformal_refusal`]
/// invocation). Carries enough state for the dashboard to render the
/// per-claim abstain/answer histogram without re-decoding the tau row.
///
/// Field semantics:
///   * `claim_id` — the originating Claim id (joins against the D-10
///     `claim_routed` rows).
///   * `claim_score` — the composite confidence the orchestrator
///     supplied; in `[0.0, 1.0]`.
///   * `tau` — the calibration threshold the decision was taken
///     against.
///   * `decision` — one of `abstain | answer`.
///   * `confidence` — `claim_score` if `decision == "answer"`, else
///     `0.0` (Abstain carries no confidence by construction).
///   * `ts_ms` — wall-clock UNIX ms at decide-emit time.
///
/// Grounding: arXiv:2503.01345 (UniCR) — formal Mode E refusal rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalRefusalDecidedPayload {
    pub claim_id: String,
    pub claim_score: f64,
    pub tau: f64,
    pub decision: String,
    pub confidence: f64,
    pub ts_ms: i64,
}

/// D-17 — payload for [`SsbActionType::RefusalRateSnapshot`].
///
/// Emitted once per
/// [`crate::refusal_sla::RefusalRateMonitor::emit_snapshot`] call (and by
/// extension every [`crate::agent_pool::dispatch_refusal_sla_check`]
/// invocation). The row carries the per-window aggregate the dashboard
/// `RefusalRateGauge` renders.
///
/// Field semantics:
///   * `refusal_rate` — `abstain_count / window_total` clamped to
///     `[0.0, 1.0]`. `0.0` when `window_total == 0` (the no-data /
///     fail-safe band).
///   * `window_total` — number of [`crate::conformal_calibration::RefusalDecision`]
///     observations currently inside the sliding window.
///   * `abstain_count` — number of `Abstain` decisions inside the window.
///   * `health_band` — snake-case [`crate::refusal_sla::HealthBand`]
///     discriminant: `below` | `healthy` | `above`.
///   * `ts_ms` — wall-clock UNIX ms at snapshot-emit time.
///
/// Grounding: arXiv:2503.01345 (UniCR honest refusal); docs/specs/guide/
/// real-problem-solving-grounded-2026-05-19.md §4.6 (refusal rate as a
/// first-class quality metric).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefusalRateSnapshotPayload {
    pub refusal_rate: f64,
    pub window_total: u32,
    pub abstain_count: u32,
    pub health_band: String,
    pub ts_ms: i64,
}

/// D-17 — payload for [`SsbActionType::RefusalSlaBreach`].
///
/// Emitted by [`crate::refusal_sla::RefusalRateMonitor::emit_snapshot`]
/// whenever the computed snapshot's `health_band` is `Below` or
/// `Above`. Carries the offending rate + the closest healthy-band
/// boundary the rate crossed + the snake-case breach kind. The
/// dashboard surfaces this beacon as the last-breach chip on the
/// `RefusalRateGauge`.
///
/// Field semantics:
///   * `rate` — the offending refusal rate `[0.0, 1.0]`.
///   * `threshold` — the boundary the rate crossed (`healthy_band_lo`
///     for `Below`, `healthy_band_hi` for `Above`).
///   * `breach_kind` — `above` | `below` (snake-case).
///   * `ts_ms` — wall-clock UNIX ms at breach-emit time.
///
/// Grounding: arXiv:2503.01345 (UniCR honest refusal — formal Mode E
/// surface the SLA monitors).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefusalSlaBreachPayload {
    pub rate: f64,
    pub threshold: f64,
    pub breach_kind: String,
    pub ts_ms: i64,
}

/// D-22 — payload for
/// [`SsbActionType::ConformalValidationCompleted`].
///
/// Emitted once per
/// [`crate::benchmarks::conformal_validation_runner::ConformalValidationRunner::run_all`]
/// Ok return. The row is the durable audit trail the dashboard
/// `ConformalValidationPanel` renders to surface whether the UniCR
/// formal distribution-free guarantee (arXiv:2503.01345, T1) still
/// holds against the held-out fixture's in_domain / near_ood / far_ood
/// buckets.
///
/// Field semantics:
///   * `tau` — threshold computed from the calibration corpus at this
///     run's operating point.
///   * `epsilon` — the operating point the calibration corpus was tau'd
///     against; defaults to
///     [`crate::conformal_calibration::DEFAULT_EPSILON`].
///   * `empirical_refusal_rate` — `total_refused / total` measured on the
///     validation fixture.
///   * `gating_verdict` — snake-case [`crate::benchmarks::prmbench_runner::GatingVerdict`]
///     label (`"pass"` iff every bucket landed in-band AND every bucket
///     had `>= min_rows_per_bucket` rows; `"fail"` otherwise).
///   * `fixture_blake3` — 64-hex blake3 of the validation fixture file
///     contents, so dashboards can detect stale runs against a moved
///     fixture.
///   * `total` — number of validation rows the runner consumed.
///   * `ts_ms` — wall-clock UNIX ms at run completion.
///
/// Grounding: arXiv:2503.01345 (UniCR formal guarantee) + Vovk et al.
/// 2005 §2.2 (quantile-based conformal predictor); D-22 plan
/// `docs/thefinalglance/D-plan/D-22_conformal_calibration_validation.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalValidationCompletedPayload {
    pub tau: f64,
    pub epsilon: f64,
    pub empirical_refusal_rate: f64,
    pub gating_verdict: String,
    pub fixture_blake3: String,
    pub total: u32,
    pub ts_ms: i64,
}

/// D-20 — payload for [`SsbActionType::ArcBenchmarkCompleted`].
///
/// Emitted once per
/// [`crate::benchmarks::arc_agi2_runner::ARCAGI2Runner::run_all`] Ok
/// return. The row is the durable audit trail the dashboard
/// `ARCBenchPanel` renders to surface whether ELAI's mode-routing +
/// novelty-classifier honesty stack behaves honestly under ARC-AGI-2
/// out-of-distribution claims.
///
/// Field semantics:
///   * `total` — number of fixture rows the runner consumed.
///   * `honestly_evaluated_count` — rows that landed on the
///     `(DiscoveryCandidate class, non-Constructive mode)` pair (the
///     "honest evaluation" predicate).
///   * `gating_verdict` — snake-case
///     [`crate::benchmarks::arc_agi2_runner::ARCVerdict`] label
///     (`honestly_evaluated` | `game_with_memorization` | `refused`).
///   * `fixture_blake3` — 64-hex blake3 of the fixture file bytes
///     (anti-stale audit trail).
///   * `dominant_mode` — snake-case
///     [`crate::grounding_mode::GroundingMode`] wire label for the
///     modal mode in the per-run distribution.
///   * `dominant_novelty_class` — snake-case
///     [`crate::novelty::NoveltyClass`] wire label for the modal
///     novelty class in the per-run distribution.
///   * `ts_ms` — wall-clock UNIX ms at run completion.
///
/// Grounding: TODO_DOC[D-20] — paper-ingestor follow-up to anchor
/// against ARC-AGI-2 dataset description (arXiv:2412.04261, candidate)
/// + ARC Prize 2025 Technical Report (arXiv:2601.10904, candidate).
/// Citations dropped per anti-citation-rot rule until paper-ingestor
/// lands the workflow/papers/ interface map. Plan:
/// `docs/thefinalglance/D-plan/D-20_arc_agi2_ood_probe.md`.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ARCBenchmarkCompletedPayload {
    pub total: u32,
    pub honestly_evaluated_count: u32,
    pub gating_verdict: String,
    pub fixture_blake3: String,
    pub dominant_mode: String,
    pub dominant_novelty_class: String,
    pub ts_ms: i64,
}

/// D-15 — payload for [`SsbActionType::NoveltyClassified`].
///
/// Emitted once per [`crate::novelty::emit_novelty_classified`] call
/// (and by extension every [`crate::agent_pool::dispatch_novelty`]
/// invocation). The row is the durable audit trail the dashboard
/// `NoveltyPanel` reads to render the per-template class label, the
/// recent-classifications list, and the per-class distribution chart.
///
/// Field semantics:
///   * `template_id` — hex `TemplateId` of the classified
///     `ReasoningTemplate` (joins against D-1 ArchiveChain rows).
///   * `class` — one of `search | distant_search | discovery_candidate`
///     (snake-case [`crate::novelty::NoveltyClass`] label).
///   * `dkg_match_count` — number of DKG primitives the template body
///     references that exist in the active symbol graph.
///   * `near_duplicate_threshold` — cosine similarity in `[0.0, 1.0]`
///     against the closest archived template body.
///   * `ts_ms` — wall-clock UNIX ms at classify-emit time.
///
/// Grounding:
///   * arXiv:2406.05761 — *LLM Novelty Detection*: three-bucket
///     classification (familiar / recombination / out-of-distribution).
///   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
///     §1 Mode C (lines 68-89) + §4.8 (lines 310-328).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoveltyClassifiedPayload {
    pub template_id: String,
    pub class: String,
    pub dkg_match_count: u32,
    pub near_duplicate_threshold: f64,
    pub ts_ms: i64,
}

/// D-18 — payload for [`SsbActionType::ObservableEffectVerified`].
///
/// Emitted once per
/// [`crate::observable_effect::verify_observable_effect`] call. The row
/// is the durable audit trail the dashboard
/// `ObservableEffectPanel.svelte` renders.
///
/// Field semantics:
///   * `effect_id` — stable per-effect identifier (snake-case ASCII).
///     Used to join successive runs of the same effect against each
///     other for time-dependent confound flagging.
///   * `trigger_kind` — snake-case
///     [`crate::observable_effect::TriggerSpec`] kind label
///     (`test_command | file_presence | http_endpoint_get |
///     file_contents | stdout_contains`).
///   * `observation_kind` — snake-case
///     [`crate::observable_effect::TypedObservation`] kind label
///     (`string_match | file_content | json_shape | http_status |
///     exit_code`).
///   * `sandbox_tier` — snake-case
///     [`crate::observable_effect::SandboxTier`] label
///     (`no_sandbox | process_isolated | container_isolated |
///     network_blocked`).
///   * `outcome` — `pass | fail | skipped`. `Skipped` covers executor
///     errors + mis-authored trigger/observation pairings; it is NOT a
///     product failure.
///   * `confound_flags` — comma-separated list of set
///     [`crate::observable_effect::ConfoundFlags`] labels
///     (`environment_dependent`, `time_dependent`, `order_dependent`,
///     `nondeterministic`). Empty string when no flag is set.
///   * `elapsed_us` — verifier wall-clock microseconds from start to
///     SSB-emit.
///   * `ts_ms` — wall-clock UNIX ms at verify-emit time.
///
/// Grounding: TODO_DOC[D-18]: paper-ingestor follow-up to anchor the
/// typed-effect / sandbox-observation discipline (candidate anchors:
/// Voyager arxiv:2305.16291 §3 skill-with-measurable-effect;
/// Coscientist arxiv:2304.05332 §4 sandbox vs production observation).
/// The orchestrator brief's arXiv:2412.16367 citation was rejected
/// (WebSearch confirmed it is "A Layered Swarm Optimization Method
/// for Fitting Battery Thermal Runaway Models" — unrelated to typed
/// test predicates).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservableEffectVerifiedPayload {
    pub effect_id: String,
    pub trigger_kind: String,
    pub observation_kind: String,
    pub sandbox_tier: String,
    pub outcome: String,
    pub confound_flags: String,
    pub elapsed_us: u64,
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::SeededClock;
    use std::time::{Duration, SystemTime};
    use tempfile::TempDir;

    fn test_config(dir: &TempDir) -> SsbConfig {
        SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "test-project".into(),
            session_id: "test-session".into(),
        }
    }

    fn test_clock() -> Arc<dyn ClockProvider> {
        Arc::new(SeededClock::new())
    }

    /// Generate a valid serialized payload for any action type.
    /// Used by tests that don't care about payload contents but need
    /// valid msgpack bytes to pass write-time validation (3J.12).
    fn test_payload(action_type: SsbActionType) -> Vec<u8> {
        let payload = match action_type {
            SsbActionType::FileWrite => SsbPayload::FileWrite(FileWritePayload {
                path: "test.rs".into(), size_bytes: 0, hash: "h".into(),
            }),
            SsbActionType::FileRead => SsbPayload::FileRead(FileReadPayload {
                path: "test.rs".into(),
            }),
            SsbActionType::ToolResult => SsbPayload::ToolResult(ToolResultPayload {
                tool_id: "t".into(), success: true, output_summary: "ok".into(),
            }),
            SsbActionType::TestOutcome => SsbPayload::TestOutcome(TestOutcomePayload {
                test_name: "t".into(), passed: true, duration_ms: 1,
            }),
            SsbActionType::Finding => SsbPayload::Finding(FindingPayload {
                severity: "INFO".into(), category: "c".into(), message: "m".into(),
                file_path: None, line: None, failure_category: None,
                audit_step: None, few_shot_examples: None,
            }),
            SsbActionType::Claim => SsbPayload::Claim(ClaimPayload {
                claim_text: "c".into(), grounding_source: "s".into(), verified: false,
            }),
            SsbActionType::Checkpoint => SsbPayload::Checkpoint(CheckpointPayload {
                agent_state_hash: "h".into(), grc_history_hash: "g".into(),
                agent_id: "a".into(), step_index: 0, file_sizes: vec![],
            }),
            SsbActionType::ModelSwap => SsbPayload::ModelSwap(ModelSwapPayload {
                from_model: "a".into(), to_model: "b".into(), gpu_id: 0,
                swap_duration_ms: 1, reason: "r".into(),
            }),
            SsbActionType::PhaseTransition => SsbPayload::PhaseTransition(PhaseTransitionPayload {
                from_state: "a".into(), to_state: "b".into(), evidence_summary: "e".into(),
            }),
            SsbActionType::KnowledgeDegradation => SsbPayload::KnowledgeDegradation(KnowledgeDegradationPayload {
                component: "c".into(), old_mode: "a".into(), new_mode: "b".into(), reason: "r".into(),
            }),
            SsbActionType::IsolationMetric => SsbPayload::IsolationMetric(IsolationMetricPayload {
                agent_id: "a".into(), phase_id: "p".into(), context_savings_ratio: 0.5,
            }),
            SsbActionType::PlanUpdate => SsbPayload::PlanUpdate(PlanUpdatePayload {
                phase_id: "p".into(), version: "v".into(), change_summary: "s".into(),
            }),
            SsbActionType::ReplanRequest => SsbPayload::ReplanRequest(ReplanRequestPayload {
                agent_id: "a".into(), severity: "MINOR".into(), evidence: "e".into(),
            }),
            SsbActionType::StubReference => SsbPayload::StubReference(StubReferencePayload {
                phase_id: "p".into(), step_id: "s".into(), stub_hash: "h".into(),
            }),
            SsbActionType::ReasoningTrace => SsbPayload::ReasoningTrace(ReasoningTracePayload {
                agent_id: "a".into(), step_index: 0, step_type: "observe".into(),
                content_summary: "c".into(), grounding_source: "s".into(),
                grounding_level: "verified".into(), confidence: 0.9,
                elapsed_ms: 1, topic: "t".into(), budget_remaining: 5,
            }),
            SsbActionType::PhaseComplete => SsbPayload::PhaseComplete(PhaseCompletePayload {
                phase_id: "p".into(), agent_id: "a".into(), fsm_state: "IDLE".into(),
                evidence_artifacts: vec![], commit_hashes: vec![], test_outcomes: vec![],
            }),
            SsbActionType::MemoryMonitoring => SsbPayload::MemoryMonitoring(MemoryMonitoringPayload {
                channel_id: "c".into(), deep_size_bytes: 0, threshold_exceeded: false,
            }),
            SsbActionType::WorkloadProfile => SsbPayload::WorkloadProfile(WorkloadProfilePayload {
                agent_role: "coder".into(), phase_id: "p".into(), tokens_in: 0,
                tokens_out: 0, latency_ms: 0, vram_peak_mb: 0,
                model_id: "m".into(), gpu_id: 0, timestamp: 0,
            }),
            SsbActionType::DeathEvent => SsbPayload::DeathEvent(DeathEventPayload {
                agent_id: "a".into(), classification: "c".into(),
                prior_step_index: 0, checkpoint_path: "p".into(), had_checkpoint: false,
            }),
            SsbActionType::RecoveryEvent => SsbPayload::RecoveryEvent(RecoveryEventPayload {
                agent_id: "a".into(), classification: "c".into(),
                recovered_from_checkpoint: false, degraded: false,
                first_grounding_source: "s".into(), prior_step_index: 0, recovery_step_index: 1,
            }),
            SsbActionType::SupervisionExhausted => SsbPayload::SupervisionExhausted(SupervisionExhaustedPayload {
                agent_id: "a".into(), phase_id: "p".into(), restart_count: 0,
                max_restarts: 3, restart_history: vec![], escalation_action: "stop".into(),
            }),
            SsbActionType::SpiralDetected => SsbPayload::SpiralDetected(SpiralDetectedPayload {
                agent_id: "a".into(), phase_id: "p".into(), spiral_depth: 1,
            }),
            SsbActionType::ContextReduction => SsbPayload::ContextReduction(ContextReductionPayload {
                level: 1, before_tokens: 100, after_tokens: 50, slots_affected: vec![1],
            }),
            SsbActionType::BranchMerge => SsbPayload::BranchMerge(BranchMergePayload {
                winner_name: "w".into(), abandoned_names: vec![],
                trunk_snap_id: "t".into(), from_snap_ids: vec![],
            }),
            SsbActionType::MistakeEntry => SsbPayload::MistakeEntry(MistakeEntryPayload {
                baby_step_id: "step-1".into(),
                entry: elai_core::MistakeEntry {
                    attempt_number: 1,
                    diff_hash: elai_core::DiffHash::from_diff("test"),
                    failure_category: elai_core::FailureCategory::Unknown,
                    faulty_line: None, root_cause_json: None,
                    action_taken: "test".into(), timestamp: 0,
                },
                cleared: false,
            }),
            SsbActionType::ResearchStep => SsbPayload::ResearchStep(ResearchStepPayload {
                agent_id: "a".into(), node_id: 0, action: "expand".into(),
                phi: 0.5, psi: 0.5, iteration: 1,
            }),
            SsbActionType::ResearchDecomposeLevel => SsbPayload::ResearchDecomposeLevel(ResearchDecomposeLevelPayload {
                agent_id: "a".into(), level: 0, concept_count: 3, phase_count: 5,
            }),
            SsbActionType::WebResearchStep => SsbPayload::WebResearchStep(WebResearchStepPayload {
                agent_id: "a".into(), node_id: 0, action: "search".into(),
                phi: 0.5, psi: 0.4, iteration: 1, sources_fetched: 3, sources_used: 2,
                fetcher: "reqwest".into(),
                stage: String::new(), stage_phase: String::new(), stage_payload: None,
            }),
            SsbActionType::PreviewAvailable => SsbPayload::PreviewAvailable(PreviewAvailablePayload {
                url: "http://localhost:5173".into(), step_id: "step-1".into(),
                slot: "live".into(), agent_id: "a".into(),
                build_success: true, dev_server_ok: true,
            }),
            SsbActionType::InterventionRequest => SsbPayload::InterventionRequest(InterventionRequestPayload {
                step_id: "step-1".into(),
                intervention_type: InterventionType::ApproveChange,
                description: "approve?".into(),
                diff_summary: None,
                options: vec![
                    InterventionOption::Keyword(InterventionKeyword::Accept),
                    InterventionOption::Keyword(InterventionKeyword::Decline),
                ],
            }),
            SsbActionType::TaskSubmitted => SsbPayload::TaskSubmitted(TaskSubmittedPayload {
                task: "do the thing".into(),
                project_id: "p".into(),
                submitted_at_ms: 0,
            }),
            SsbActionType::InterventionResponse => SsbPayload::InterventionResponse(InterventionResponsePayload {
                request_id: 1,
                chosen: InterventionOption::Keyword(InterventionKeyword::Accept),
                user_note: None,
            }),
            SsbActionType::HookEvent => SsbPayload::HookEvent(HookEventPayload {
                tier: "T1".into(),
                verdict: "Allow".into(),
                rule_id: None,
                category: "hook_event".into(),
                message: "allow".into(),
                step_id: "step-1".into(),
                hook_point: "PreToolUse".into(),
            }),
            SsbActionType::ProviderSignals => SsbPayload::ProviderSignals(ProviderSignalsPayload {
                gg_mean_logprob: Some(-1.2),
                gg_min_logprob: Some(-3.5),
                entropy_mean: Some(2.1),
                entropy_max: Some(4.8),
                novelty_cosine: Some(0.72),
                steering_vector_active: false,
                etc_disabled: true,
                step_id: "step-1".into(),
            }),
            SsbActionType::SearchTriggerFired => SsbPayload::SearchTriggerFired(SearchTriggerFiredPayload {
                reason: "import".into(),
                threshold_hit: "trigger_token_idx=42".into(),
                step_id: "step-1".into(),
            }),
            SsbActionType::VramTierDetected => SsbPayload::VramTierDetected(VramTierDetectedPayload {
                tier: "12GB".into(),
                detected_via: "nvml".into(),
                free_mb: Some(1800),
            }),
            SsbActionType::IndexingProgress => SsbPayload::IndexingProgress(IndexingProgressPayload {
                stage: "start".into(),
                project_id: "p".into(),
                files_processed: 0,
                files_total: 0,
                elapsed_ms: 0,
                message: None,
                error_kind: None,
                error_message: None,
            }),
            SsbActionType::DkgReady => SsbPayload::DkgReady(DkgReadyPayload {
                project_id: "p".into(),
                node_count: 0,
                edge_count: 0,
                languages: vec![],
            }),
            SsbActionType::TieredAnalysis => SsbPayload::TieredAnalysis(TieredAnalysisPayload {
                tier: "T1".into(),
                verdict: "Allow".into(),
                rule_count_fired: 0,
                elapsed_us: 0,
                step_id: "step-1".into(),
                category: "hook_event".into(),
            }),
            SsbActionType::ShortCoderCompression => {
                SsbPayload::ShortCoderCompression(ShortCoderCompressionPayload {
                    tokens_in: 0,
                    tokens_out: 0,
                    reduction_pct: 0,
                    step_id: "step-1".into(),
                    rules_applied: vec![],
                })
            }
            SsbActionType::ToolDispatch => SsbPayload::ToolDispatch(ToolDispatchPayload {
                tool: "edit".into(),
                args_summary: "path=src/foo.rs tier=exact".into(),
                elapsed_ms: 3,
                step_id: "step-1".into(),
                outcome: "success".into(),
                error_category: None,
            }),
            SsbActionType::ValidationTier => {
                SsbPayload::ValidationTier(ValidationTierPayload {
                    tier: "determinism".into(),
                    passed: true,
                    elapsed_ms: 1200,
                    rules_fired: 1,
                    step_id: "step-1".into(),
                })
            }
            SsbActionType::FailureClassified => {
                SsbPayload::FailureClassified(FailureClassifiedPayload {
                    category: "COMPILATION_ERROR".into(),
                    confidence_bp: 10000,
                    step_id: "step-1".into(),
                    source: "validator".into(),
                })
            }
            SsbActionType::GitCommit => SsbPayload::GitCommit(GitCommitPayload {
                commit_sha: "abc123def456".into(),
                branch: "elai/plan-demo".into(),
                files_count: 2,
                step_id: "step-1".into(),
                message: "batch-2b: test commit".into(),
            }),
            SsbActionType::DairaTrace => SsbPayload::DairaTrace(DairaTracePayload {
                phase: "enter".into(),
                tool: "edit".into(),
                args_hash: "deadbeef".into(),
                elapsed_us: 0,
                outcome: "started".into(),
                step_id: "step-1".into(),
            }),
            SsbActionType::SystemProbe => SsbPayload::SystemProbe(SystemProbePayload {
                gpu_name: Some("test".into()),
                vram_total_mb: Some(12_288),
                vram_free_mb: Some(6_144),
                ram_total_mb: 32_000,
                ram_free_mb: 16_000,
                disk_total_mb: 1_000_000,
                disk_free_mb: 500_000,
                detection_method: "shell_out".into(),
                tier: "12GB".into(),
            }),
            SsbActionType::FeasibilityReport => {
                SsbPayload::FeasibilityReport(FeasibilityReportPayload {
                    task_feasibility: "feasible".into(),
                    skip_reason: None,
                    novel_step_count: 0,
                    escalations_count: 0,
                    vram_margin_bp: 5_000,
                    ram_margin_bp: 5_000,
                    disk_margin_bp: 5_000,
                })
            }
            SsbActionType::NovelStepDetected => {
                SsbPayload::NovelStepDetected(NovelStepDetectedPayload {
                    step_id: "step-1".into(),
                    reason: "unverified".into(),
                    alternative_count: 0,
                })
            }
            SsbActionType::StubRun => SsbPayload::StubRun(StubRunPayload {
                step_id: "step-1".into(),
                passed: true,
                elapsed_ms: 12,
            }),
            SsbActionType::GateDecision => SsbPayload::GateDecision(GateDecisionPayload {
                decision: "auto_advance".into(),
                risk_level: Some("LOW".into()),
                reason: None,
                evidence_summary: "kr=1 zu=1 tfc=1 tf=1 zb=1 dcg=1".into(),
                step_id: "step-1".into(),
            }),
            SsbActionType::TaskSplit => SsbPayload::TaskSplit(TaskSplitPayload {
                original_task_id: "task-1".into(),
                sub_task_ids: vec!["task-1.1".into(), "task-1.2".into()],
                depth: 1,
                sub_task_summaries: None,
                splitter: None,
                complexity_estimate: None,
            }),
            SsbActionType::EscalationFired => SsbPayload::EscalationFired(EscalationFiredPayload {
                trigger: "anti_spiral".into(),
                step_id: "step-1".into(),
                details: "spiral_depth=2".into(),
            }),
            SsbActionType::ModeChanged => SsbPayload::ModeChanged(ModeChangedPayload {
                from: "interactive".into(),
                to: "autonomous".into(),
            }),
            // Batch 3C (Phase 4H) — Layer 1A.
            SsbActionType::ProposalGenerated => SsbPayload::ProposalGenerated(ProposalGeneratedPayload {
                proposal_id: "pp-0000-1".into(),
                table: "lazy_code_regex".into(),
                new_row_summary: "todo!".into(),
                fpr_estimate: 0.0,
                source: "auto_from_outcome".into(),
            }),
            SsbActionType::ProposalApplied => SsbPayload::ProposalApplied(ProposalAppliedPayload {
                proposal_id: "pp-0000-1".into(),
                table: "lazy_code_regex".into(),
                applied_by: "layer1a".into(),
                commit_sha: None,
            }),
            SsbActionType::ProposalRejected => SsbPayload::ProposalRejected(ProposalRejectedPayload {
                proposal_id: "pp-0000-1".into(),
                table: "lazy_code_regex".into(),
                reason: "fpr too high".into(),
            }),
            SsbActionType::MsiUpdate => SsbPayload::MsiUpdate(MsiUpdatePayload {
                pipeline_mode: "surgical".into(),
                msi_value: 0.5,
                status: "IMPROVING".into(),
            }),
            SsbActionType::ConfigArchived => SsbPayload::ConfigArchived(ConfigArchivedPayload {
                pipeline_mode: "surgical".into(),
                archived_count: 1,
                dropped_id: None,
            }),
            // Batch 4C.
            SsbActionType::ProfileSelected => SsbPayload::ProfileSelected(ProfileSelectedPayload {
                profile: "FirstAttempt".into(),
                role: "executor".into(),
                fsm_state: "EXECUTING".into(),
                step_id: "step-1".into(),
            }),
            SsbActionType::FsmTransition => SsbPayload::FsmTransition(FsmTransitionPayload {
                from: "IDLE".into(),
                to: "RESEARCHING".into(),
                reason: "web_research_task".into(),
            }),
            SsbActionType::MicroReflection => SsbPayload::MicroReflection(MicroReflectionPayload {
                step_id: "step-1".into(),
                root_cause: "missing import".into(),
                proposed_fix: "add use crate::foo".into(),
                confidence: 0.8,
            }),
            SsbActionType::TelescopeCollapse => {
                SsbPayload::TelescopeCollapse(TelescopeCollapsePayload {
                    from_tokens: 1800,
                    to_tokens: 1400,
                    layers_collapsed: 1,
                    step_id: "step-1".into(),
                })
            }
            SsbActionType::ConfigWarning => SsbPayload::ConfigWarning(ConfigWarningPayload {
                config: "source-tiers.toml".into(),
                reason: "file_not_found".into(),
                detail: "path=/missing".into(),
            }),
            // Batch-fu.
            SsbActionType::ConstraintsInjected => {
                SsbPayload::ConstraintsInjected(ConstraintsInjectedPayload {
                    project_id: "proj-test".into(),
                    constraint_count: 2,
                    step_id: "step-1".into(),
                })
            }
            SsbActionType::InterventionResolved => {
                SsbPayload::InterventionResolved(InterventionResolvedPayload {
                    intervention_id: "iv-0000-0001".into(),
                    approved: true,
                    elapsed_ms: 250,
                })
            }
            SsbActionType::DkgQuerierSwapped => {
                SsbPayload::DkgQuerierSwapped(DkgQuerierSwappedPayload {
                    project_id: "p".into(),
                    from: "empty".into(),
                    to: "adapter".into(),
                })
            }
            SsbActionType::SteeringComputed => {
                SsbPayload::SteeringComputed(SteeringComputedPayload {
                    role: "executor".into(),
                    vector_dim: 2048,
                    norm: 12.34,
                    elapsed_ms: 150,
                    cache_path: "/tmp/.elai_cc/steering/p/executor.bin".into(),
                })
            }
            SsbActionType::SteeringComputeFailed => {
                SsbPayload::SteeringComputeFailed(SteeringComputeFailedPayload {
                    role: "executor".into(),
                    reason: "sidecar_unreachable".into(),
                })
            }
            SsbActionType::McpServerConnected => {
                SsbPayload::McpServerConnected(McpServerConnectedPayload {
                    server_url: "http://localhost:8200".into(),
                    tool_count: 3,
                })
            }
            SsbActionType::McpToolCalled => {
                SsbPayload::McpToolCalled(McpToolCalledPayload {
                    tool_name: "echo".into(),
                    server_url: "http://localhost:8200".into(),
                    elapsed_ms: 42,
                    outcome: "success".into(),
                })
            }
            SsbActionType::McpServerDisconnected => {
                SsbPayload::McpServerDisconnected(McpServerDisconnectedPayload {
                    server_url: "http://localhost:8200".into(),
                    reason: "user_disabled".into(),
                })
            }
            SsbActionType::ChatMessage => SsbPayload::ChatMessage(ChatMessagePayload {
                session_id: "s-test".into(),
                text: "hello".into(),
                intent: "TASK".into(),
                project_id: "p".into(),
                submitted_at_ms: 0,
            }),
            SsbActionType::ChatResponse => SsbPayload::ChatResponse(ChatResponsePayload {
                session_id: "s-test".into(),
                card_kind: "status".into(),
                body: "ok".into(),
                project_id: "p".into(),
                emitted_at_ms: 0,
            }),
            SsbActionType::ExecutionModeChanged => SsbPayload::ExecutionModeChanged(ExecutionModeChangedPayload {
                from_mode: "LOCAL".into(),
                to_mode: "CLOUD".into(),
                project_id: "p".into(),
                changed_at_ms: 0,
            }),
            SsbActionType::ContextBudgetComputed => {
                SsbPayload::ContextBudgetComputed(ContextBudgetComputedPayload {
                    tier: "micro_7b_12gb".into(),
                    raw_max: 25000,
                    slots: vec![0; 8],
                    allocations: vec![330, 400, 800, 1500, 8000, 1500, 1000, 512],
                    adapt_levels_fired: 0,
                })
            }
            SsbActionType::SsbFilteredByOverlap => {
                SsbPayload::SsbFilteredByOverlap(SsbFilteredByOverlapPayload {
                    entries_total: 10,
                    entries_kept: 4,
                    overlap_threshold: 0.10,
                })
            }
            SsbActionType::HookLatencyViolation => {
                SsbPayload::HookLatencyViolation(HookLatencyViolationPayload {
                    hook_name: "PreToolUse".into(),
                    tier: "T1".into(),
                    budget_ms: 1,
                    actual_ms: 12,
                })
            }
            SsbActionType::ResearchPathChosen => {
                SsbPayload::ResearchPathChosen(ResearchPathChosenPayload {
                    path: "llm".into(),
                    reason: "llm".into(),
                    ts_ms: 0,
                })
            }
            SsbActionType::GroundingCacheEvent => {
                SsbPayload::GroundingCacheEvent(GroundingCacheEventPayload {
                    kind: "miss".into(),
                    key_fingerprint: "00000000".into(),
                    level: "Idea".into(),
                    elapsed_us: 0,
                })
            }
            // B-4 (v26 bump completed).
            SsbActionType::CircuitBreakerStateChange => {
                SsbPayload::CircuitBreakerStateChange(CircuitBreakerStateChangePayload {
                    model: "graphcodebert".into(),
                    state: "CLOSED".into(),
                    previous_state: "CLOSED".into(),
                    failures: 0,
                    success_rate: 1.0,
                    last_failure_ts: None,
                })
            }
            // B-7 variants.
            SsbActionType::BubbleUpTyped => SsbPayload::BubbleUpTyped(BubbleUpTypedPayload {
                from_level: "BabyStep".into(),
                to_level: "Step".into(),
                action_kind: "diagnose_and_retry".into(),
                reason: "test".into(),
                failure_count: 1,
                threshold: 3,
            }),
            SsbActionType::IntegrationCheckpointSuccess => {
                SsbPayload::IntegrationCheckpointSuccess(IntegrationCheckpointSuccessPayload {
                    level: "BabyStep".into(),
                    node_id: 0,
                    wiring_target: "test-target".into(),
                    success: true,
                    failure_reason: None,
                    duration_ms: 1,
                })
            }
            SsbActionType::DagRepairRound => SsbPayload::DagRepairRound(DagRepairRoundPayload {
                action_kind: "reorder".into(),
                round: 1,
                reason: "test".into(),
            }),
            SsbActionType::ConceptDecompositionNode => {
                SsbPayload::ConceptDecompositionNode(ConceptDecompositionNodePayload {
                    node_kind: "concept".into(),
                    node_id: 0,
                    parent_id: 0,
                    name: "test-concept".into(),
                })
            }
            // B-6 variants.
            SsbActionType::EvidenceGateEvaluated => {
                SsbPayload::EvidenceGateEvaluated(EvidenceGateEvaluatedPayload {
                    gate_id: "knowledge_readiness_gate".into(),
                    verdict: "pass".into(),
                    ts_ms: 0,
                })
            }
            SsbActionType::SupervisionEvent => {
                SsbPayload::SupervisionEvent(SupervisionEventPayload {
                    agent_id: "a".into(),
                    event: "restart".into(),
                    attempt: 1,
                })
            }
            SsbActionType::BspSuperstep => SsbPayload::BspSuperstep(BspSuperstepPayload {
                snapshot_id: "s-test:superstep:1".into(),
                ts_ms: 0,
                channels_count: 1,
            }),
            SsbActionType::AgentLifecycle => SsbPayload::AgentLifecycle(AgentLifecyclePayload {
                agent_id: "a".into(),
                role: "executor".into(),
                state: "spawned".into(),
                death_class: None,
            }),
            SsbActionType::AntiSpiralHit => SsbPayload::AntiSpiralHit(AntiSpiralHitPayload {
                agent_id: "a".into(),
                step_hash: "0000000000000000000000000000000000000000000000000000000000000000".into(),
                count: 4,
            }),
            SsbActionType::CloudSpend => SsbPayload::CloudSpend(CloudSpendPayload {
                provider: "anthropic".into(),
                model: "claude-sonnet-4-6".into(),
                input_tokens: 10,
                output_tokens: 2,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                cost_usd: 0.000_06,
                task_id: "t".into(),
                ts_ms: 0,
            }),
            SsbActionType::EscalateToCloud => SsbPayload::EscalateToCloud(EscalateToCloudPayload {
                step_id: "step-1".into(),
                trigger_reason: "test".into(),
                layer_failed: 1,
                ts_ms: 0,
            }),
            // B-23 Phase 4H Group A.
            SsbActionType::DataTableGrew => SsbPayload::DataTableGrew(DataTableGrewPayload {
                proposal_id: "prop-1".into(),
                table: "deprecated_patterns".into(),
                pipeline_mode: "surgical".into(),
                ecosystem: Some("cargo".into()),
                language: Some("Rust".into()),
                rowcount_after: 12,
                ts_ms: 0,
            }),
            SsbActionType::L3Proposal => SsbPayload::L3Proposal(L3ProposalPayload {
                proposal_id: "l3-1".into(),
                change_type: "allowed_config_key_addition".into(),
                description: "add EtcThreshold variant".into(),
                rationale: "test".into(),
                ts_ms: 0,
            }),
            SsbActionType::ConvergenceStagnant => {
                SsbPayload::ConvergenceStagnant(ConvergenceStagnantPayload {
                    pipeline_mode: "surgical".into(),
                    window: 3,
                    similarity: 0.96,
                    variant_id: "v-7".into(),
                    ts_ms: 0,
                })
            }
            SsbActionType::ConfigOscillation => {
                SsbPayload::ConfigOscillation(ConfigOscillationPayload {
                    pipeline_mode: "surgical".into(),
                    period: 2,
                    frozen_keys: vec!["etc_threshold".into()],
                    freeze_pipelines: 5,
                    ts_ms: 0,
                })
            }
            SsbActionType::RegressionDetected => {
                SsbPayload::RegressionDetected(RegressionDetectedPayload {
                    pipeline_id: "pipe-99".into(),
                    pipeline_mode: "greenfield".into(),
                    regressed_ac_indices: vec![0, 3, 7],
                    max_consecutive_failures: 2,
                    ts_ms: 0,
                })
            }
            SsbActionType::EvolutionWithheld => {
                SsbPayload::EvolutionWithheld(EvolutionWithheldPayload {
                    pipeline_mode: "surgical".into(),
                    identical_streak: 5,
                    reason: "ontology never evolved".into(),
                    ts_ms: 0,
                })
            }
            // B-39 chat reflect pipeline.
            SsbActionType::ReflectTriaged => SsbPayload::ReflectTriaged(ReflectTriagedPayload {
                turn_id: "turn-1".into(),
                class: "problem".into(),
            }),
            SsbActionType::ReflectCandidate => SsbPayload::ReflectCandidate(ReflectCandidatePayload {
                turn_id: "turn-1".into(),
                candidate_index: 0,
                content: "test-content".into(),
            }),
            SsbActionType::ReflectAuditVerdict => SsbPayload::ReflectAuditVerdict(ReflectAuditVerdictPayload {
                turn_id: "turn-1".into(),
                best_index: 0,
                rationale: "most direct".into(),
            }),
            SsbActionType::ReflectRefined => SsbPayload::ReflectRefined(ReflectRefinedPayload {
                turn_id: "turn-1".into(),
                total_tokens_used: 0,
                synthesis: "test-synthesis".into(),
            }),
            // B-24 Phase 4I Group C1: media validator beacons (test fixtures).
            SsbActionType::MediaValidationStarted => {
                SsbPayload::MediaValidationStarted(MediaValidationStartedPayload {
                    step_id: "step-1".into(),
                    media_type: "image".into(),
                    file_path: "/tmp/x.png".into(),
                })
            }
            SsbActionType::MediaTier1Complete => {
                SsbPayload::MediaTier1Complete(MediaTier1CompletePayload {
                    step_id: "step-1".into(),
                    pass: true,
                    check_count: 0,
                    latency_ms: 0,
                })
            }
            SsbActionType::MediaTier2Complete => {
                SsbPayload::MediaTier2Complete(MediaTier2CompletePayload {
                    step_id: "step-1".into(),
                    advisory_pass: true,
                    model_used: "mock-vlm".into(),
                    latency_ms: 0,
                })
            }
            SsbActionType::RenderComplete => SsbPayload::RenderComplete(RenderCompletePayload {
                step_id: "step-1".into(),
                adapter_name: "web".into(),
                frame_count: 1,
                latency_ms: 0,
            }),
            SsbActionType::VisualFeedbackIteration => {
                SsbPayload::VisualFeedbackIteration(VisualFeedbackIterationPayload {
                    step_id: "step-1".into(),
                    iteration: 1,
                    issue_count: 0,
                    converged: true,
                })
            }
            SsbActionType::MediaHumanReviewRequired => {
                SsbPayload::MediaHumanReviewRequired(MediaHumanReviewRequiredPayload {
                    step_id: "step-1".into(),
                    reason: "test-reason".into(),
                })
            }
            SsbActionType::GenerationModeSelected => {
                SsbPayload::GenerationModeSelected(GenerationModeSelectedPayload {
                    step_id: "baby_step_1".into(),
                    mode: "structured".into(),
                    rule_fired: "structured_fallback".into(),
                    reason: "rule_5_structured_fallback".into(),
                    preferred_was: None,
                    supports_fim: false,
                    exec_mode: "local".into(),
                })
            }
            SsbActionType::StreamingEarlyTerminated => {
                SsbPayload::StreamingEarlyTerminated(StreamingEarlyTerminatedPayload {
                    step_id: "baby_step_1".into(),
                    mode: "fim".into(),
                    reason: "lazy_pattern".into(),
                    reason_detail: Some("rust_todo_macro".into()),
                    token_count_at_term: 17,
                    total_tokens_received: 17,
                    language_hint: "rust".into(),
                })
            }
            SsbActionType::PipelinePreAssembled => {
                SsbPayload::PipelinePreAssembled(PipelinePreAssembledPayload {
                    step_idx: 0,
                    pass_rate_bp: 10_000,
                    was_discarded: false,
                    depth_at_time: 1,
                    bytes_assembled: 1024,
                })
            }
            SsbActionType::DependencyResolved => {
                SsbPayload::DependencyResolved(DependencyResolvedPayload {
                    language: "rust".into(),
                    name: "tokio".into(),
                    decision: "auto_approve".into(),
                    reason: "allowlist_match".into(),
                    install_command: Some("cargo add tokio".into()),
                })
            }
            SsbActionType::MicroStepCreated => {
                SsbPayload::MicroStepCreated(MicroStepCreatedPayload {
                    parent_step_id: 1,
                    created_path: "crates/foo/src/bar.rs".into(),
                    type_hint: "Module".into(),
                    reason: "undefined_symbol".into(),
                    count: 1,
                })
            }
            // B-30 (SCHEMA_VERSION 36): language grammar registry status.
            SsbActionType::LanguageGrammarStatus => {
                SsbPayload::LanguageGrammarStatus(LanguageGrammarStatusPayload {
                    loaded: vec!["go".into(), "python".into(), "rust".into()],
                    failed: vec![],
                })
            }
            // B-29 Phase 5 (SCHEMA_VERSION 37): symbol-graph tool call row.
            SsbActionType::SymbolQuery => {
                SsbPayload::SymbolQuery(SymbolQueryPayload {
                    tool: "symbol_impact".into(),
                    target: "src/lib.rs".into(),
                    hop_count: 2,
                    total_files: 3,
                    latency_ms: 5,
                })
            }
            // B-32 (SCHEMA_VERSION 38): minified-file chunking row.
            SsbActionType::MinifiedFileChunked => {
                SsbPayload::MinifiedFileChunked(MinifiedFileChunkedPayload {
                    relative_path: "vendor/bundle.min.js".into(),
                    char_count: 50_000,
                    chunk_count: 25,
                    avg_line_length: 50_000,
                })
            }
            // B-31.5 (SCHEMA_VERSION 39): 5 deferred B-31 SSB variants.
            SsbActionType::IndexCheckpoint => {
                SsbPayload::IndexCheckpoint(IndexCheckpointPayload {
                    project_id: "p".into(),
                    batches_processed: 1,
                    batches_total: Some(4),
                    files_processed: 10,
                    last_checkpoint_at: 0,
                    batch_duration_ms: Some(120),
                })
            }
            SsbActionType::IndexResume => SsbPayload::IndexResume(IndexResumePayload {
                project_id: "p".into(),
                resumed_from_batch: 1,
                files_skipped: 20,
            }),
            SsbActionType::IndexCancel => SsbPayload::IndexCancel(IndexCancelPayload {
                project_id: "p".into(),
                batches_completed: 1,
                files_indexed: 5,
            }),
            SsbActionType::LockAcquired => SsbPayload::LockAcquired(LockAcquiredPayload {
                project_id: "p".into(),
                scope: "index".into(),
                holder_pid: 12345,
            }),
            SsbActionType::LockContested => SsbPayload::LockContested(LockContestedPayload {
                project_id: "p".into(),
                scope: "index".into(),
                holder_pid: 12345,
            }),
            // B-33 follow-up (SCHEMA_VERSION 40): symbol-graph build event row.
            SsbActionType::SymbolGraphUpdated => {
                SsbPayload::SymbolGraphUpdated(SymbolGraphUpdatedPayload {
                    project_id: "p".into(),
                    file_count: 16,
                    symbol_count: 1_024,
                    edge_count: 4_096,
                    built_ms: 0,
                    // C-24 (DRIFT-4): default None (additive field; full-build
                    // path leaves the flag unset).
                    is_incremental: None,
                })
            }
            // B-40 (SCHEMA_VERSION 41): FRAMING substate beacons.
            SsbActionType::IdeaFramingStarted => SsbPayload::IdeaFramingStarted(
                IdeaFramingStartedPayload {
                    session_id: "s".into(),
                    prompt: "p".into(),
                    started_at_ms: 0,
                },
            ),
            SsbActionType::ProjectClassClassified => SsbPayload::ProjectClassClassified(
                ProjectClassClassifiedPayload {
                    session_id: "s".into(),
                    project_class: "other".into(),
                    matched_pattern: None,
                },
            ),
            SsbActionType::CriticRoundComplete => SsbPayload::CriticRoundComplete(
                CriticRoundCompletePayload {
                    session_id: "s".into(),
                    round_index: 1,
                    block_count: 0,
                    warn_count: 0,
                    info_count: 0,
                    converged: true,
                    category_counts: Vec::new(),
                },
            ),
            SsbActionType::IdeaFramed => SsbPayload::IdeaFramed(IdeaFramedPayload {
                session_id: "s".into(),
                artifact_path: "framed-idea.toml".into(),
                project_class: "other".into(),
                rounds_used: 1,
                tradeoff_count: 2,
                converged: true,
            }),
            SsbActionType::TradeoffCardEmitted => SsbPayload::TradeoffCardEmitted(
                TradeoffCardEmittedPayload {
                    session_id: "s".into(),
                    tradeoff_names: Vec::new(),
                    option_counts: Vec::new(),
                },
            ),
            // B-34 (SCHEMA_VERSION 42)
            SsbActionType::EntryPointsDetected => SsbPayload::EntryPointsDetected(
                EntryPointsDetectedPayload {
                    project_id: "p".into(),
                    count: 0,
                    by_reason: std::collections::BTreeMap::new(),
                },
            ),
            SsbActionType::ArtifactsIndexed => SsbPayload::ArtifactsIndexed(
                ArtifactsIndexedPayload {
                    project_id: "p".into(),
                    indexed: Vec::new(),
                    up_to_date: Vec::new(),
                    total_chunks: 0,
                    latency_ms: 0,
                },
            ),
            SsbActionType::ArtifactSearched => SsbPayload::ArtifactSearched(
                ArtifactSearchedPayload {
                    project_id: "p".into(),
                    artifact_name: None,
                    query: "q".into(),
                    result_count: 0,
                    latency_ms: 0,
                },
            ),
            SsbActionType::ArtifactStale => SsbPayload::ArtifactStale(
                ArtifactStalePayload {
                    project_id: "p".into(),
                    artifact_name: "a".into(),
                    changed_files: Vec::new(),
                },
            ),
            // B-38 (SCHEMA_VERSION 43)
            SsbActionType::AmbiguityScored => SsbPayload::AmbiguityScored(
                AmbiguityScoredPayload {
                    score: 0.15,
                    milestone: "ready".into(),
                    goal: 0.9,
                    constraint: 0.85,
                    success: 0.85,
                    context: None,
                    brownfield: false,
                },
            ),
            SsbActionType::AmbiguityFloorFailed => SsbPayload::AmbiguityFloorFailed(
                AmbiguityFloorFailedPayload {
                    score: 0.1,
                    failed_dimensions: vec!["goal_clarity".into()],
                },
            ),
            SsbActionType::AmbiguityStreakAdvanced => SsbPayload::AmbiguityStreakAdvanced(
                AmbiguityStreakAdvancedPayload {
                    score: 0.18,
                    current_streak: 1,
                    required: 2,
                },
            ),
            // B-42 (SCHEMA_VERSION 44): cosmos tier knowledge worker beacons.
            SsbActionType::CosmosRefreshStarted => SsbPayload::CosmosRefreshStarted(
                CosmosRefreshStartedPayload { queued: 0, total: 0 },
            ),
            SsbActionType::CosmosArtifactUpdated => SsbPayload::CosmosArtifactUpdated(
                CosmosArtifactUpdatedPayload {
                    name: "a".into(),
                    blake3: "0".into(),
                    body_bytes: 0,
                    latency_ms: 0,
                },
            ),
            SsbActionType::CosmosArtifactDecayed => SsbPayload::CosmosArtifactDecayed(
                CosmosArtifactDecayedPayload {
                    name: "a".into(),
                    age_secs: 0,
                    cadence: "daily".into(),
                },
            ),
            SsbActionType::CosmosGroundingUsed => SsbPayload::CosmosGroundingUsed(
                CosmosGroundingUsedPayload {
                    claim_category: "design".into(),
                    artifact_name: "a".into(),
                    matched: true,
                },
            ),
            // B-41 (SCHEMA_VERSION 45): per-level auditor mode beacons.
            SsbActionType::PerLevelAuditStarted => SsbPayload::PerLevelAuditStarted(
                PerLevelAuditStartedPayload {
                    mode: "concepts".into(),
                    tier: "concept".into(),
                    node_id: 0,
                },
            ),
            SsbActionType::PerLevelAuditVerdict => SsbPayload::PerLevelAuditVerdict(
                PerLevelAuditVerdictPayload {
                    mode: "concepts".into(),
                    tier: "concept".into(),
                    node_id: 0,
                    verdict: "allow".into(),
                    findings_count: 0,
                    respawned_once: false,
                    cascade_to_parent: false,
                },
            ),
            SsbActionType::PerLevelAuditCascade => SsbPayload::PerLevelAuditCascade(
                PerLevelAuditCascadePayload {
                    mode: "babystep_detail".into(),
                    tier: "baby_step".into(),
                    node_id: 0,
                    parent_tier: "baby_step".into(),
                },
            ),
            // B-37C (SCHEMA_VERSION 46): multi-runtime backend beacons.
            SsbActionType::RuntimeFallback => SsbPayload::RuntimeFallback(
                RuntimeFallbackPayload {
                    role: elai_ipc_types::AgentRole::Executor,
                    primary: elai_providers::runtime::RuntimeBackend::Sidecar,
                    attempted: vec![
                        elai_providers::runtime::RuntimeBackend::Sidecar,
                        elai_providers::runtime::RuntimeBackend::AnthropicApi,
                    ],
                    succeeded: Some(elai_providers::runtime::RuntimeBackend::AnthropicApi),
                    reason: "sidecar unavailable, anthropic_api succeeded".into(),
                },
            ),
            SsbActionType::RuntimeHealthChanged => SsbPayload::RuntimeHealthChanged(
                RuntimeHealthChangedPayload {
                    backend: elai_providers::runtime::RuntimeBackend::Sidecar,
                    prev: elai_providers::runtime::RuntimeHealth::Unavailable {
                        reason: "first observation".into(),
                    },
                    next: elai_providers::runtime::RuntimeHealth::Ready,
                },
            ),
            SsbActionType::CliSubprocessSpawned => SsbPayload::CliSubprocessSpawned(
                CliSubprocessSpawnedPayload {
                    backend: elai_providers::runtime::RuntimeBackend::ClaudeCode,
                    pid: 12345,
                    argv_redacted: vec!["claude".into(), "-p".into(), "[REDACTED]".into()],
                },
            ),
            SsbActionType::CliSubprocessKilled => SsbPayload::CliSubprocessKilled(
                CliSubprocessKilledPayload {
                    backend: elai_providers::runtime::RuntimeBackend::ClaudeCode,
                    pid: 12345,
                    exit_code: Some(0),
                    reason: "normal exit".into(),
                },
            ),
            // C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
            SsbActionType::GrcStepEmitted => SsbPayload::GrcStepEmitted(GrcStepEmittedPayload {
                agent_id: "executor".into(),
                topic: "task_runner".into(),
                step_type: "observe".into(),
                grounding_level: "high".into(),
                confidence: 0.85,
                budget_remaining: 5,
                elapsed_ms: 12,
            }),
            SsbActionType::GroundingSweepResult => {
                SsbPayload::GroundingSweepResult(GroundingSweepResultPayload {
                    ungrounded: 1,
                    verify_needed: 0,
                    grounded: 3,
                    top_claim: Some("training:placeholder".into()),
                })
            }
            SsbActionType::CsqEvaluated => SsbPayload::CsqEvaluated(CsqEvaluatedPayload {
                severity: "Warning".into(),
                guard_present: true,
                blocked: false,
                check_count: 2,
            }),
            SsbActionType::AuditorRubberStampRejected => SsbPayload::AuditorRubberStampRejected(
                AuditorRubberStampRejectedPayload {
                    role: "auditor".into(),
                    finding_count: 0,
                    expected_min: 1,
                },
            ),
            SsbActionType::HydeQuery => SsbPayload::HydeQuery(HydeQueryPayload {
                query_hash: "abc123".into(),
                hypothesis_len: 128,
                cache_hit: false,
                latency_ms: 12,
            }),
            SsbActionType::VectorCacheHitRate => {
                SsbPayload::VectorCacheHitRate(VectorCacheHitRatePayload {
                    session_id: "test-session".into(),
                    hits: 4,
                    misses: 1,
                    hit_rate_pct: 80.0,
                })
            }
            SsbActionType::QueryTrace => SsbPayload::QueryTrace(QueryTracePayload {
                query_id: "11111111-2222-3333-4444-555555555555".into(),
                tier: "STANDARD".into(),
                stage_count: 9,
                result_count: 12,
                total_latency_ms: 245,
            }),
            // C-2 (SCHEMA_VERSION 48): orchestration + cosmos + stagnation beacons.
            SsbActionType::PhaseNodeReady => SsbPayload::PhaseNodeReady(PhaseNodeReadyPayload {
                phase_id: 7,
                concept_id: 3,
                label: "phase-test".into(),
                depth: 1,
            }),
            SsbActionType::StepNodeReady => SsbPayload::StepNodeReady(StepNodeReadyPayload {
                step_id: 12,
                phase_id: 7,
                concept_id: 3,
                label: "step-test".into(),
                depth: 2,
            }),
            SsbActionType::BabyStepReady => SsbPayload::BabyStepReady(BabyStepReadyPayload {
                baby_step_id: 42,
                step_id: 12,
                phase_id: 7,
                concept_id: 3,
                label: "baby-step-test".into(),
                depth: 3,
            }),
            SsbActionType::DagRepairEscalated => SsbPayload::DagRepairEscalated(
                DagRepairEscalatedPayload {
                    rounds: 10,
                    remaining_issues: 2,
                    last_error: "cycle persists after repair".into(),
                },
            ),
            SsbActionType::AgentStatusChanged => SsbPayload::AgentStatusChanged(
                AgentStatusChangedPayload {
                    agent_id: "executor-1".into(),
                    role: "executor".into(),
                    state: "spawned".into(),
                    death_class: None,
                    timestamp_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::StagnationSpinningDetected => SsbPayload::StagnationSpinningDetected(
                StagnationSpinningDetectedPayload {
                    agent_id: "executor-1".into(),
                    pattern: "spinning".into(),
                    iteration: 3,
                    detail: "step_hash repeated 3 times".into(),
                },
            ),
            SsbActionType::StagnationOscillationDetected => {
                SsbPayload::StagnationOscillationDetected(
                    StagnationOscillationDetectedPayload {
                        agent_id: "executor-1".into(),
                        pattern: "oscillation".into(),
                        iteration: 4,
                        detail: "period-2 A-B-A-B".into(),
                    },
                )
            }
            SsbActionType::LateralThinkingActivated => SsbPayload::LateralThinkingActivated(
                LateralThinkingActivatedPayload {
                    agent_id: "executor-1".into(),
                    persona: "contrarian".into(),
                    iteration: 5,
                    detail: "switched after spinning".into(),
                },
            ),
            SsbActionType::GoalDriftWarning => SsbPayload::GoalDriftWarning(
                GoalDriftWarningPayload {
                    agent_id: "executor-1".into(),
                    drift_score: 0.42,
                    baby_step_id: 17,
                    detail: "task drifted from intent".into(),
                },
            ),
            SsbActionType::WatchdogTimeout => SsbPayload::WatchdogTimeout(WatchdogTimeoutPayload {
                agent_id: "executor-1".into(),
                elapsed_ms: 30_000,
                budget_ms: 20_000,
                detail: "no observable output".into(),
            }),
            // C-10 (SCHEMA_VERSION 52) — lateral-thinking + goal-drift Block.
            SsbActionType::LateralThinkingSucceeded => SsbPayload::LateralThinkingSucceeded(
                LateralThinkingSucceededPayload {
                    agent_id: "executor-1".into(),
                    persona: "hacker".into(),
                    iteration: 6,
                    detail: "pattern broken after hacker persona".into(),
                },
            ),
            SsbActionType::LateralThinkingExhausted => SsbPayload::LateralThinkingExhausted(
                LateralThinkingExhaustedPayload {
                    agent_id: "executor-1".into(),
                    personas_tried: 5,
                    iteration: 10,
                    detail: "all personas tried; pattern persists".into(),
                },
            ),
            SsbActionType::GoalDriftBlock => SsbPayload::GoalDriftBlock(GoalDriftBlockPayload {
                agent_id: "executor-1".into(),
                drift_score: 0.32,
                baby_step_id: 19,
                detail: "drift_score below block threshold".into(),
            }),
            // C-7 (SCHEMA_VERSION 49): reviewer verdict beacon.
            SsbActionType::ChangeDecision => SsbPayload::ChangeDecision(ChangeDecisionPayload {
                decision: "accepted".into(),
                path: "src/foo.rs".into(),
                hunk_id: Some("hunk-1".into()),
                note: None,
            }),
            // C-8 (SCHEMA_VERSION 50): search-mode cross-process bridge.
            SsbActionType::SearchModeChanged => SsbPayload::SearchModeChanged(
                SearchModeChangedPayload {
                    mode: "hybrid".into(),
                    history: vec!["lexical".into(), "hybrid".into()],
                },
            ),
            // C-5 (SCHEMA_VERSION 51): privacy-redaction event.
            SsbActionType::RedactionApplied => SsbPayload::RedactionApplied(
                RedactionAppliedPayload {
                    count: 3,
                    fields: vec!["task_summary".into(), "escalation_message".into()],
                    baby_step_id: 17,
                    ts_ms: 0,
                },
            ),
            // C-13 (SCHEMA_VERSION 53): WIP commit squash completion.
            SsbActionType::WipSquashComplete => SsbPayload::WipSquashComplete(
                WipSquashCompletePayload {
                    wip_count: 7,
                    squashed: true,
                    step_id: "step-42".into(),
                },
            ),
            // C-22 (SCHEMA_VERSION 54): feasibility-probe completeness beacons.
            SsbActionType::AlternativeExplored => SsbPayload::AlternativeExplored(
                AlternativeExploredPayload {
                    step_id: "step-c22".into(),
                    alternative_index: 1,
                    accepted: true,
                },
            ),
            SsbActionType::FeasibilityEscalated => SsbPayload::FeasibilityEscalated(
                FeasibilityEscalatedPayload {
                    reason: "vram_margin_breached".into(),
                    variant: "resource_margin_breached".into(),
                    step_id: "step-c22".into(),
                },
            ),
            // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
            SsbActionType::BranchForked => SsbPayload::BranchForked(BranchForkedPayload {
                branch_name: "feature-x".into(),
                tip_snap_id: "snap-c23-tip".into(),
                parent_snap_id: "snap-c23-parent".into(),
            }),
            SsbActionType::BranchMerged => SsbPayload::BranchMerged(BranchMergedPayload {
                winner_name: "feature-x".into(),
                trunk_snap_id: "snap-c23-trunk".into(),
                abandoned_names: vec!["experiment-y".into()],
            }),
            SsbActionType::BranchClosed => SsbPayload::BranchClosed(BranchClosedPayload {
                branch_name: "experiment-y".into(),
                reason: "user-requested".into(),
            }),
            // C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
            // + background-task beacons.
            SsbActionType::PlanSkeleton => SsbPayload::PlanSkeleton(PlanSkeletonPayload {
                plan_id: "plan-c25-test".into(),
                step_count: 3,
                concept_class: "Counter".into(),
                plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                source: PlanSkeletonSource::Llm,
            }),
            SsbActionType::CompiledPipelineFired => SsbPayload::CompiledPipelineFired(
                CompiledPipelineFiredPayload {
                    pipeline_id: "pl-c25-test".into(),
                    concept_class: "Counter".into(),
                    plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                    use_count: 4,
                },
            ),
            SsbActionType::TemplateCrystallized => SsbPayload::TemplateCrystallized(
                TemplateCrystallizedPayload {
                    concept_class: "Counter".into(),
                    plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                    pipeline_id: "pl-c25-test".into(),
                    threshold_n: 3,
                },
            ),
            SsbActionType::BackgroundTaskStarted => SsbPayload::BackgroundTaskStarted(
                BackgroundTaskStartedPayload {
                    task_id: "bg-c25-test".into(),
                    command_summary: "grep -r foo /large".into(),
                    soft_timeout_secs: 30,
                    hard_timeout_secs: 600,
                },
            ),
            SsbActionType::BackgroundTaskComplete => SsbPayload::BackgroundTaskComplete(
                BackgroundTaskCompletePayload {
                    task_id: "bg-c25-test".into(),
                    exit_status: 0,
                    duration_secs: 12,
                    auto_detached: false,
                    timed_out: false,
                },
            ),
            // C-25 audit follow-up (SCHEMA_VERSION 57): missing 6th
            // compiled-pipeline beacon (WARN-1 in audit-C25.md).
            SsbActionType::DecompileTriggered => SsbPayload::DecompileTriggered(
                DecompileTriggeredPayload {
                    concept_class: "Counter".into(),
                    plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                    failure_count: 2,
                    trigger_reason: DecompileTriggerReason::ConsecutiveFailures,
                },
            ),
            // C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
            SsbActionType::FutilityDetected => SsbPayload::FutilityDetected(
                FutilityDetectedPayload {
                    error_class: "constrained_decode_reject".into(),
                    consecutive_count: 3,
                    level: "baby_step".into(),
                    escalated: true,
                },
            ),
            SsbActionType::FutilityOutputStagnation => SsbPayload::FutilityOutputStagnation(
                FutilityOutputStagnationPayload {
                    similarity: 0.95,
                    threshold: 0.90,
                    level: "baby_step".into(),
                    new_persona: Some("hacker".into()),
                },
            ),
            // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons.
            // audit_id = Uuid::nil() keeps the test fixture inside the entropy
            // quarantine — see C-15 D7 + DRIFT-11.
            SsbActionType::WiringEvidence => SsbPayload::WiringEvidence(
                WiringEvidencePayload {
                    entrypoint: "crates/cli/src/main.rs::main".into(),
                    produces_symbols_verified: vec!["DeterminismConfig".into()],
                    reachable_count: 1,
                    audit_id: uuid::Uuid::nil(),
                },
            ),
            SsbActionType::WiringViolation => SsbPayload::WiringViolation(
                WiringViolationPayload {
                    violation_kind: WiringViolationKind::StubPattern,
                    file: "crates/orchestrator/src/example.rs".into(),
                    line: 42,
                    pattern: "todo!()".into(),
                    audit_id: uuid::Uuid::nil(),
                },
            ),
            SsbActionType::DeterminismDrift => SsbPayload::DeterminismDrift(
                DeterminismDriftPayload {
                    seed: 42,
                    run_a_hash: "blake3:a".into(),
                    run_b_hash: "blake3:b".into(),
                    first_divergence_index: 7,
                    audit_id: uuid::Uuid::nil(),
                },
            ),
            // C-28 (SCHEMA_VERSION 60): T-1 mutation testing — one row per
            // surviving mutant. Fixture pins file/line/mutant_kind for the
            // SQL CHECK roundtrip test (test_typed_payloads).
            SsbActionType::MutationSurvived => SsbPayload::MutationSurvived(
                MutationSurvivedPayload {
                    file: "crates/orchestrator/src/example.rs".into(),
                    line: 7,
                    mutant_kind: "replace_+_with_-".into(),
                },
            ),
            // C-29 (SCHEMA_VERSION 61): T-2 IntentPair confirmation +
            // T-3 metamorphic-relation violation. Both fixtures pin
            // canonical payload shapes for the test_typed_payloads
            // roundtrip; MrViolated.counterexample MUST be non-empty
            // (stub_ban contract).
            SsbActionType::IntentConfirmed => SsbPayload::IntentConfirmed(
                IntentConfirmedPayload {
                    step_id: 1,
                    pair_count: 3,
                    confirmed_by: "user".into(),
                },
            ),
            SsbActionType::MrViolated => SsbPayload::MrViolated(MrViolatedPayload {
                step_id: 1,
                relation: "inverse".into(),
                counterexample: "sample_index=0 input_hex=01 encoded_hex=ff decoded_hex=01".into(),
                sample_size: 1,
            }),
            // C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode payloads.
            SsbActionType::SurgicalPlanCreated => SsbPayload::SurgicalPlanCreated(
                SurgicalPlanCreatedPayload {
                    task_id: "task_1700000000_c30test".into(),
                    complexity_class: "simple".into(),
                    blast_radius_files: 2,
                    total_loc_estimate: 150,
                    step_count: 1,
                    regression_gate_snapshot_hash: "blake3:c30aaaaa".into(),
                },
            ),
            SsbActionType::SurgicalLocWarning => SsbPayload::SurgicalLocWarning(
                SurgicalLocWarningPayload {
                    task_id: "task_1700000000_c30test".into(),
                    total_loc_estimate: 600,
                    threshold: 500,
                    mode: "interactive".into(),
                },
            ),
            SsbActionType::OracleCacheHit => SsbPayload::OracleCacheHit(OracleCacheHitPayload {
                query_hash_hex: "00".repeat(32),
                source_tier: "t1_authoritative".into(),
                cached_age_secs: 60,
            }),
            SsbActionType::OracleCacheMiss => SsbPayload::OracleCacheMiss(OracleCacheMissPayload {
                query_hash_hex: "00".repeat(32),
                confidence: "high".into(),
            }),
            SsbActionType::ProjectDetectionSignal => SsbPayload::ProjectDetectionSignal(
                ProjectDetectionSignalPayload {
                    signal_name: "cargo_toml".into(),
                    weight: "required".into(),
                    fired: true,
                },
            ),
            // C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 payloads.
            SsbActionType::SkeletonGenerated => SsbPayload::SkeletonGenerated(
                SkeletonGeneratedPayload {
                    blank_count: 4,
                    language: "rust".into(),
                    file_path: "src/lib.rs".into(),
                },
            ),
            SsbActionType::SkeletonFlushed => SsbPayload::SkeletonFlushed(
                SkeletonFlushedPayload {
                    path: "src/lib.rs".into(),
                    completeness_ratio: 1.0,
                },
            ),
            SsbActionType::MiniCriticVerdict => SsbPayload::MiniCriticVerdict(
                MiniCriticVerdictPayload {
                    blank_id: 1,
                    verdict: "pass".into(),
                    reason: "ok".into(),
                    language: "rust".into(),
                },
            ),
            SsbActionType::StructuralRetryStageAttempt => SsbPayload::StructuralRetryStageAttempt(
                StructuralRetryStageAttemptPayload {
                    blank_id: 1,
                    stage: "template".into(),
                    attempt: 1,
                    outcome: "converged".into(),
                },
            ),
            SsbActionType::BridgeFileGenerated => SsbPayload::BridgeFileGenerated(
                BridgeFileGeneratedPayload {
                    kind: "mod_rs_aggregator".into(),
                    path: "src/mod.rs".into(),
                    module_count: 3,
                },
            ),
            // D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle.
            SsbActionType::ArchiveEntryAdded => SsbPayload::ArchiveEntryAdded(
                ArchiveEntryAddedPayload {
                    template_id: "00000000000000000000000000000001".into(),
                    content_hash: "ff".repeat(32),
                    parent_hash: None,
                    domain: "code".into(),
                    pattern_kind: "decomposition".into(),
                    archive_status: "frozen".into(),
                },
            ),
            SsbActionType::ArchiveEntryRetired => SsbPayload::ArchiveEntryRetired(
                ArchiveEntryRetiredPayload {
                    template_id: "00000000000000000000000000000001".into(),
                    retired_at_ms: 1_700_000_000_000,
                    retirement_reason: "degradation".into(),
                },
            ),
            SsbActionType::TemplatePromoted => SsbPayload::TemplatePromoted(
                TemplatePromotedPayload {
                    template_id: "00000000000000000000000000000001".into(),
                    stage: "stage_50".into(),
                    pass_rate: 0.94,
                },
            ),
            SsbActionType::TemplateCandidateRejected => SsbPayload::TemplateCandidateRejected(
                TemplateCandidateRejectedPayload {
                    template_id: "00000000000000000000000000000001".into(),
                    rejection_reason: "below_floor".into(),
                },
            ),
            // E-5 (SCHEMA_VERSION 65): memory_facts bi-temporal lifecycle beacons.
            SsbActionType::MemoryFactExtracted => SsbPayload::MemoryFactExtracted(
                MemoryFactExtractedPayload {
                    fact_id: "fact-test-extract".into(),
                    fact_type: "preference".into(),
                    scope: "project".into(),
                    project_id: Some("proj-test".into()),
                    user_id: "default".into(),
                    key: "prefers_tracing_over_log".into(),
                    trust_score: 0.9,
                },
            ),
            SsbActionType::MemoryFactSuperseded => SsbPayload::MemoryFactSuperseded(
                MemoryFactSupersededPayload {
                    old_fact_id: "fact-test-old".into(),
                    new_fact_id: "fact-test-new".into(),
                    valid_to_ms: 1_700_000_000_000,
                },
            ),
            // D-2 (SCHEMA_VERSION 66): PPM verifier beacons.
            SsbActionType::PpmScored => SsbPayload::PpmScored(PpmScoredPayload {
                candidate_id: "cand-test-1".into(),
                verifier_id: "ppm-test".into(),
                mode: "math".into(),
                score_mean: 0.91,
                score_var: 0.0004,
                num_steps: 3,
                latency_ms: 412,
                cache_hit: false,
            }),
            SsbActionType::PpmGatingFailed => SsbPayload::PpmGatingFailed(
                PpmGatingFailedPayload {
                    candidate_id: "cand-test-2".into(),
                    verifier_id: "ppm-test".into(),
                    mode: "math".into(),
                    score_mean: 0.42,
                    threshold: 0.7,
                    num_steps: 2,
                },
            ),
            // E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
            SsbActionType::ChatReflectionCompleted => SsbPayload::ChatReflectionCompleted(
                ChatReflectionCompletedPayload {
                    note_id: "note-test-reflect".into(),
                    note_preview: "U: i prefer blake3...".into(),
                    facts_extracted: 1,
                    decay_rows_examined: 4,
                    decay_rows_updated: 1,
                    core_block_bytes: 256,
                },
            ),
            SsbActionType::MemoryEvolutionRun => SsbPayload::MemoryEvolutionRun(
                MemoryEvolutionRunPayload {
                    phase: "end".into(),
                    duration_ms: 1_240,
                    notes_touched: 8,
                },
            ),
            // E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
            // producer + conflict beacons.
            SsbActionType::MemoryNoteInserted => SsbPayload::MemoryNoteInserted(
                MemoryNoteInsertedPayload {
                    note_id: "note-test-insert".into(),
                    scope: "session".into(),
                    content_blake3: "0".repeat(64),
                    link_count: 2,
                },
            ),
            SsbActionType::MemoryNoteEvolved => SsbPayload::MemoryNoteEvolved(
                MemoryNoteEvolvedPayload {
                    note_id: "note-test-evolve".into(),
                    old_content_blake3: "1".repeat(64),
                    new_content_blake3: "2".repeat(64),
                    links_added: 1,
                    links_removed: 0,
                },
            ),
            SsbActionType::MemoryConflictDetected => SsbPayload::MemoryConflictDetected(
                MemoryConflictDetectedPayload {
                    fact_id_a: "fact-test-a".into(),
                    fact_id_b: "fact-test-b".into(),
                    conflict_kind: "duplicate".into(),
                },
            ),
            SsbActionType::CoreBlockReplaced => SsbPayload::CoreBlockReplaced(
                CoreBlockReplacedPayload {
                    user_id: "default".into(),
                    old_blake3: "3".repeat(64),
                    new_blake3: "4".repeat(64),
                    bytes: 512,
                },
            ),
            // D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator.
            SsbActionType::VerifierConsensusPass => SsbPayload::VerifierConsensusPass(
                VerifierConsensusPassPayload {
                    candidate_id: "cand-test-pass".into(),
                    template_id: "tpl-test-1".into(),
                    verdict: "pass".into(),
                    verifier_count: 3,
                    pass_count: 2,
                    rule_id: "k_of_n".into(),
                },
            ),
            SsbActionType::VerifierConsensusFail => SsbPayload::VerifierConsensusFail(
                VerifierConsensusFailPayload {
                    candidate_id: "cand-test-fail".into(),
                    template_id: "tpl-test-1".into(),
                    failing_verifier_ids: vec!["v-intrinsic-1".into(), "v-det-1".into()],
                    reason: "fail".into(),
                },
            ),
            // D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
            SsbActionType::TripwireFired => SsbPayload::TripwireFired(TripwireFiredPayload {
                event_id: "00112233445566778899aabbccddeeff".into(),
                tripwire_kind: "iqr_outlier".into(),
                candidate_id: "cand-test-trip".into(),
                observed_value: 42.0,
                threshold: 3.0,
                fired_at_ms: 1_700_000_000_000,
            }),
            SsbActionType::TemplateAutoReverted => {
                SsbPayload::TemplateAutoReverted(TemplateAutoRevertedPayload {
                    template_id: "tpl-test-auto-1".into(),
                    parent_template_id: Some("tpl-test-parent".into()),
                    retirement_reason: "degradation".into(),
                    fired_tripwire_id: "00112233445566778899aabbccddeeff".into(),
                })
            }
            // D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark beacon.
            SsbActionType::PrmBenchRunCompleted => {
                SsbPayload::PrmBenchRunCompleted(PrmBenchRunCompletedPayload {
                    total: 20,
                    accuracy: 0.75,
                    precision: 0.80,
                    recall: 0.70,
                    gating_verdict: "pass".into(),
                    fixture_blake3: "a".repeat(64),
                    category_count: 2,
                })
            }
            // D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
            SsbActionType::BaiArmUpdated => SsbPayload::BaiArmUpdated(BaiArmUpdatedPayload {
                template_id: "11112222333344445555666677778888".into(),
                alpha: 9.0,
                beta: 2.0,
                n_pulls: 9,
                mean_estimate: 0.818,
                was_promoted: true,
            }),
            // D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
            SsbActionType::HeldOutTaskAdded => {
                SsbPayload::HeldOutTaskAdded(HeldOutTaskAddedPayload {
                    task_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                    source: "synthetic".into(),
                    content_blake3: "b".repeat(64),
                    ngram_count: 12,
                    ingested_ts_ms: 1_700_000_000_000,
                    eligibility_window_ms: 0,
                })
            }
            SsbActionType::HeldOutTaskUsed => SsbPayload::HeldOutTaskUsed(HeldOutTaskUsedPayload {
                task_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                consumer_template_id: "11112222333344445555666677778888".into(),
                used_ts_ms: 1_700_000_005_000,
            }),
            // D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
            SsbActionType::CanaryRunCompleted => {
                SsbPayload::CanaryRunCompleted(CanaryRunCompletedPayload {
                    report_id: "c".repeat(64),
                    ts_ms: 1_700_000_010_000,
                    overall_transfer_score: 0.04,
                    decision_kind: "pass".into(),
                    dominant_fail_family: String::new(),
                })
            }
            SsbActionType::CanaryFamilyEvaluated => {
                SsbPayload::CanaryFamilyEvaluated(CanaryFamilyEvaluatedPayload {
                    report_id: "c".repeat(64),
                    family: "code_fix".into(),
                    n_evaluated: 5,
                    pass_count: 4,
                    fail_count: 1,
                    transfer_score: 0.10,
                    baseline_score: 0.70,
                })
            }
            // D-9 (SCHEMA_VERSION 75): L3 frozen-surface auditor beacons.
            SsbActionType::LockedKeysAuditRun => {
                SsbPayload::LockedKeysAuditRun(LockedKeysAuditRunPayload {
                    audit_id: uuid::Uuid::nil(),
                    ts_ms: 1_700_000_020_000,
                    n_audited: 2,
                    n_violations: 1,
                    registry_blake3: "0".repeat(64),
                    registry_path: "scripts/locked_keys_registry.toml".into(),
                })
            }
            SsbActionType::LockedKeysViolationDetected => {
                SsbPayload::LockedKeysViolationDetected(
                    LockedKeysViolationDetectedPayload {
                        audit_id: uuid::Uuid::nil(),
                        ts_ms: 1_700_000_020_001,
                        attempted_edit:
                            "crates/orchestrator/src/tripwire.rs".into(),
                        attempted_lines: vec![0, u32::MAX],
                        locked_key_path:
                            "crates/orchestrator/src/tripwire.rs".into(),
                        locked_key_signature_blake3: "0".repeat(64),
                        reason: "tripwire detection per arxiv:1908.04734".into(),
                    },
                )
            }
            // E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
            SsbActionType::TaskAdvanced => SsbPayload::TaskAdvanced(TaskAdvancedPayload {
                task_id: "task-1".into(),
                parent_id: String::new(),
                completion_pct: 50.0,
                status: "in_progress".into(),
                blocker: String::new(),
                ts_ms: 1_700_000_030_000,
            }),
            SsbActionType::AgentlessShortcutTaken => {
                SsbPayload::AgentlessShortcutTaken(AgentlessShortcutTakenPayload {
                    task_id: "task-1".into(),
                    reason: "single_file_small".into(),
                    ts_ms: 1_700_000_030_001,
                })
            }
            SsbActionType::GoalDriftFired => SsbPayload::GoalDriftFired(GoalDriftFiredPayload {
                initial_goal_blake3: "d".repeat(64),
                drift: 0.42,
                threshold: 0.3,
                step: 3,
                ts_ms: 1_700_000_030_002,
            }),
            // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
            SsbActionType::ClaimRouted => SsbPayload::ClaimRouted(ClaimRoutedPayload {
                claim_id: "claim-d10-test".into(),
                claim_type: "factual".into(),
                mode: "constructive".into(),
                ts_ms: 1_700_000_040_000,
                ctx_summary_blake3: "e".repeat(64),
            }),
            // D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
            SsbActionType::ModeRoutingBenchRunCompleted => {
                SsbPayload::ModeRoutingBenchRunCompleted(
                    ModeRoutingBenchRunCompletedPayload {
                        total: 25,
                        accuracy: 0.92,
                        f1_macro: 0.88,
                        gating_verdict: "pass".into(),
                        fixture_blake3: "f".repeat(64),
                        fixture_row_count: 25,
                    },
                )
            }
            // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
            SsbActionType::CheckerResultEmitted => SsbPayload::CheckerResultEmitted(
                CheckerResultEmittedPayload {
                    checker_id: "regex-default".into(),
                    checker_kind: "regex_match".into(),
                    result_kind: "pass".into(),
                    claim_id: "claim-d11-test".into(),
                    ts_ms: 1_700_000_050_000,
                    reason: String::new(),
                },
            ),
            // D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
            SsbActionType::AdversaryGenerated => SsbPayload::AdversaryGenerated(
                AdversaryGeneratedPayload {
                    counter_id: "ctr-d13-test".into(),
                    claim_id: "claim-d13-test".into(),
                    severity: 0.75,
                    model_family: "anthropic".into(),
                },
            ),
            SsbActionType::ConsensusReevaluatedWithCounter => {
                SsbPayload::ConsensusReevaluatedWithCounter(
                    ConsensusReevaluatedWithCounterPayload {
                        claim_id: "claim-d13-test".into(),
                        original_verdict: "pass".into(),
                        post_counter_verdict: "fail".into(),
                    },
                )
            }
            // D-12 (SCHEMA_VERSION 81): OperationalizationCheck beacon.
            SsbActionType::BabyStepDispatched => SsbPayload::BabyStepDispatched(
                BabyStepDispatchedPayload {
                    step_id: "bs-d12-test".into(),
                    parent_task_id: "task-d12-test".into(),
                    checker_kinds_used: vec!["numeric_range".into()],
                    outcome: "pass".into(),
                    ts_ms: 1_700_000_060_000,
                },
            ),
            // D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal beacons.
            SsbActionType::ConformalTauComputed => SsbPayload::ConformalTauComputed(
                ConformalTauComputedPayload {
                    tau: 0.8,
                    epsilon: 0.1,
                    corpus_size: 40,
                    source: "held_out".into(),
                    ts_ms: 1_700_000_070_000,
                },
            ),
            SsbActionType::ConformalRefusalDecided => SsbPayload::ConformalRefusalDecided(
                ConformalRefusalDecidedPayload {
                    claim_id: "claim-d16-test".into(),
                    claim_score: 0.05,
                    tau: 0.8,
                    decision: "abstain".into(),
                    confidence: 0.0,
                    ts_ms: 1_700_000_071_000,
                },
            ),
            // D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass beacon.
            SsbActionType::NoveltyClassified => SsbPayload::NoveltyClassified(
                NoveltyClassifiedPayload {
                    template_id: "tpl-d15-test".into(),
                    class: "discovery_candidate".into(),
                    dkg_match_count: 0,
                    near_duplicate_threshold: 0.05,
                    ts_ms: 1_700_000_080_000,
                },
            ),
            // D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate beacon.
            SsbActionType::ObservableEffectVerified => SsbPayload::ObservableEffectVerified(
                ObservableEffectVerifiedPayload {
                    effect_id: "eff-d18-test".into(),
                    trigger_kind: "test_command".into(),
                    observation_kind: "string_match".into(),
                    sandbox_tier: "no_sandbox".into(),
                    outcome: "pass".into(),
                    confound_flags: String::new(),
                    elapsed_us: 1234,
                    ts_ms: 1_700_000_090_000,
                },
            ),
            // D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
            SsbActionType::RefusalRateSnapshot => SsbPayload::RefusalRateSnapshot(
                RefusalRateSnapshotPayload {
                    refusal_rate: 0.5,
                    window_total: 100,
                    abstain_count: 50,
                    health_band: "above".into(),
                    ts_ms: 1_700_000_100_000,
                },
            ),
            SsbActionType::RefusalSlaBreach => SsbPayload::RefusalSlaBreach(
                RefusalSlaBreachPayload {
                    rate: 0.5,
                    threshold: 0.2,
                    breach_kind: "above".into(),
                    ts_ms: 1_700_000_101_000,
                },
            ),
            // D-22 (SCHEMA_VERSION 86): conformal-calibration validation beacon.
            SsbActionType::ConformalValidationCompleted => {
                SsbPayload::ConformalValidationCompleted(
                    ConformalValidationCompletedPayload {
                        tau: 0.4,
                        epsilon: 0.1,
                        empirical_refusal_rate: 0.08,
                        gating_verdict: "pass".into(),
                        fixture_blake3: "f".repeat(64),
                        total: 36,
                        ts_ms: 1_700_000_110_000,
                    },
                )
            }
            // D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
            SsbActionType::ArcBenchmarkCompleted => SsbPayload::ArcBenchmarkCompleted(
                ARCBenchmarkCompletedPayload {
                    total: 24,
                    honestly_evaluated_count: 11,
                    gating_verdict: "honestly_evaluated".into(),
                    fixture_blake3: "a".repeat(64),
                    dominant_mode: "operational_deferred".into(),
                    dominant_novelty_class: "discovery_candidate".into(),
                    ts_ms: 1_700_000_120_000,
                },
            ),
        };
        rmp_serde::to_vec_named(&payload).expect("test payload serialization")
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_single_writer() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let mut seqs = Vec::new();
        for i in 0..10 {
            let seq = ssb
                .write(SsbWriteRequest {
                    action_type: SsbActionType::FileWrite,
                    agent_id: format!("agent-{i}"),
                    superstep: 0,
                    payload: test_payload(SsbActionType::FileWrite),
                    files: vec![format!("src/file_{i}.rs")],
                })
                .await
                .expect("write");
            seqs.push(seq);
        }

        // All seqs must be monotonically increasing with no gaps.
        for window in seqs.windows(2) {
            assert_eq!(window[1], window[0] + 1, "seqs must be sequential");
        }

        ssb.shutdown();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_write_after_shutdown() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Write one entry successfully.
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::Checkpoint,
            agent_id: "agent-1".into(),
            superstep: 0,
            payload: test_payload(SsbActionType::Checkpoint),
            files: vec![],
        })
        .await
        .expect("write before shutdown");

        ssb.shutdown();

        // Write after shutdown must fail.
        let result = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::Checkpoint,
                agent_id: "agent-1".into(),
                superstep: 0,
                payload: test_payload(SsbActionType::Checkpoint),
                files: vec![],
            })
            .await;

        assert!(result.is_err(), "write after shutdown must fail");
        match result.unwrap_err() {
            SsbError::Shutdown => (),
            other => panic!("expected Shutdown error, got: {other}"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_concurrent_writers() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Arc::new(
            Ssb::new(test_config(&dir), test_clock()).expect("ssb init"),
        );

        // Spawn 10 concurrent write tasks — no Mutex needed, write() takes &self
        // and the mpsc channel serializes writes internally.
        let mut handles = Vec::new();
        for i in 0..10 {
            let ssb = ssb.clone();
            handles.push(tokio::spawn(async move {
                ssb.write(SsbWriteRequest {
                    action_type: SsbActionType::ToolResult,
                    agent_id: format!("agent-{i}"),
                    superstep: 0,
                    payload: test_payload(SsbActionType::ToolResult),
                    files: vec![format!("src/mod_{i}.rs")],
                })
                .await
            }));
        }

        let mut seqs: Vec<i64> = Vec::new();
        for h in handles {
            let seq = h.await.expect("join").expect("write");
            seqs.push(seq);
        }

        seqs.sort();
        // All 10 seqs present, sequential, no gaps.
        assert_eq!(seqs.len(), 10);
        for window in seqs.windows(2) {
            assert_eq!(window[1], window[0] + 1, "seqs must be sequential");
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_broadcast_after_commit() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let mut rx = ssb.subscribe();

        let finding_bytes = test_payload(SsbActionType::Finding);
        let seq = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::Finding,
                agent_id: "agent-x".into(),
                superstep: 1,
                payload: finding_bytes.clone(),
                files: vec!["src/main.rs".into()],
            })
            .await
            .expect("write");

        let entry = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("timeout")
            .expect("recv");

        assert_eq!(entry.seq, seq);
        assert_eq!(entry.agent_id, "agent-x");
        assert_eq!(entry.superstep, 1);
        assert_eq!(entry.action_type, SsbActionType::Finding);
        assert_eq!(&*entry.payload, &*finding_bytes);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_action_type_mismatch_rejected() {
        // Adversarial test (3J.12): verify that action_type string mismatch
        // with payload struct is rejected at write time, not silently corrupted.
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Create a FileWrite payload
        let file_write_payload = SsbPayload::FileWrite(FileWritePayload {
            path: "test.rs".to_string(),
            size_bytes: 100,
            hash: "abc123".to_string(),
        });
        let payload_bytes = rmp_serde::to_vec_named(&file_write_payload).expect("serialize");

        // Try to write it with a different action_type (ToolResult)
        let result = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::ToolResult, // Mismatch!
                agent_id: "agent-1".into(),
                superstep: 0,
                payload: payload_bytes,
                files: vec![],
            })
            .await;

        assert!(result.is_err(), "action_type mismatch must be rejected");
        match result.unwrap_err() {
            SsbError::ActionTypeMismatch { declared, actual } => {
                assert_eq!(declared, "tool_result");
                assert_eq!(actual, "file_write");
            }
            other => panic!("expected ActionTypeMismatch error, got: {other}"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_reasoning_trace_monotonic_step_index() {
        // Adversarial test (3J.12): verify reasoning_trace entries can be queried
        // and are ordered by seq (monotonic per agent per phase).
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let agent_id = "test-agent";

        // Write multiple reasoning_trace entries
        for i in 0..5 {
            let payload = SsbPayload::ReasoningTrace(ReasoningTracePayload {
                agent_id: agent_id.to_string(),
                step_index: i,
                step_type: "observe".to_string(),
                content_summary: format!("step {}", i),
                grounding_source: "test".to_string(),
                grounding_level: "HIGH".to_string(),
                confidence: 0.9,
                elapsed_ms: 100,
                topic: "test-topic".to_string(),
                budget_remaining: 10,
            });
            let payload_bytes = rmp_serde::to_vec_named(&payload).expect("serialize");
            ssb.write(SsbWriteRequest {
                action_type: SsbActionType::ReasoningTrace,
                agent_id: agent_id.to_string(),
                superstep: 0,
                payload: payload_bytes,
                files: vec![],
            })
            .await
            .expect("write");
        }

        // Query reasoning_trace entries for the agent
        let entries = ssb
            .query_reasoning_trace(agent_id)
            .expect("query should succeed");

        assert_eq!(entries.len(), 5, "should have 5 entries");

        // Verify they are ordered by seq (monotonically increasing)
        for window in entries.windows(2) {
            assert!(window[1].seq > window[0].seq, "seq must be monotonically increasing");
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_created_at_uses_clock_provider() {
        let clock = Arc::new(SeededClock::with_epoch(
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000),
        ));
        let dir = TempDir::new().expect("tempdir");
        let config = SsbConfig {
            db_path: dir.path().join("clock_test.db"),
            pool_size: Some(2),
            project_id: "proj".into(),
            session_id: "sess".into(),
        };
        let ssb = Ssb::new(config, clock.clone()).expect("ssb init");

        let mut rx = ssb.subscribe();
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::Claim,
            agent_id: "agent-clock".into(),
            superstep: 0,
            payload: test_payload(SsbActionType::Claim),
            files: vec![],
        })
        .await
        .expect("write");

        let entry = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("timeout")
            .expect("recv");

        // SeededClock epoch is 1_000_000 secs past UNIX_EPOCH.
        assert_eq!(entry.created_at, 1_000_000);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_wal_pragma_applied() {
        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let _ssb = Ssb::new(config.clone(), test_clock()).expect("ssb init");

        // Verify via a direct connection that WAL is active.
        let conn = Connection::open(&config.db_path).expect("open");
        let journal_mode: String =
            conn.pragma_query_value(None, "journal_mode", |row| row.get(0))
                .expect("pragma");
        assert_eq!(journal_mode.to_lowercase(), "wal");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_pool_connection_has_foreign_keys() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let obj = ssb.read_pool.get().await.expect("pool get");
        let fk: i64 = obj
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .expect("pragma");
        assert_eq!(fk, 1, "foreign_keys must be ON for pool connections");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_migration_from_v0() {
        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("migrate_test.db");

        // Create a V0 database (no tables, user_version=0).
        {
            let conn = Connection::open(&db_path).expect("open");
            conn.pragma_update(None, "user_version", 0).expect("pragma");
        }

        // SSB should create tables.
        let config = SsbConfig {
            db_path,
            pool_size: Some(2),
            project_id: "proj".into(),
            session_id: "sess".into(),
        };
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");

        // Write should succeed (tables exist).
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::FileRead,
            agent_id: "agent-m".into(),
            superstep: 0,
            payload: test_payload(SsbActionType::FileRead),
            files: vec![],
        })
        .await
        .expect("write after migration");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_shutdown_idempotent() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        ssb.shutdown();
        ssb.shutdown(); // Second call must not panic.
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_ssb_files_inserted() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let seq = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::FileWrite,
                agent_id: "agent-f".into(),
                superstep: 0,
                payload: test_payload(SsbActionType::FileWrite),
                files: vec!["src/a.rs".into(), "src/b.rs".into()],
            })
            .await
            .expect("write");

        // Verify via read pool.
        let obj = ssb.read_pool.get().await.expect("pool");
        let mut stmt = obj
            .prepare("SELECT file_path FROM ssb_files WHERE seq = ?1 ORDER BY file_path")
            .expect("prepare");
        let paths: Vec<String> = stmt
            .query_map(params![seq], |row| row.get(0))
            .expect("query")
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(paths, vec!["src/a.rs", "src/b.rs"]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_all_action_types_insert() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let types = [
            SsbActionType::FileWrite,
            SsbActionType::FileRead,
            SsbActionType::ToolResult,
            SsbActionType::TestOutcome,
            SsbActionType::Finding,
            SsbActionType::Claim,
            SsbActionType::Checkpoint,
            SsbActionType::ModelSwap,
            SsbActionType::PhaseTransition,
            SsbActionType::KnowledgeDegradation,
            SsbActionType::IsolationMetric,
            SsbActionType::PlanUpdate,
            SsbActionType::ReplanRequest,
            SsbActionType::StubReference,
            SsbActionType::ReasoningTrace,
            SsbActionType::PhaseComplete,
            SsbActionType::MemoryMonitoring,
            SsbActionType::DeathEvent,
            SsbActionType::RecoveryEvent,
            SsbActionType::SupervisionExhausted,
            SsbActionType::SpiralDetected,
            SsbActionType::ContextReduction,
            SsbActionType::BranchMerge,
            SsbActionType::WorkloadProfile,
            SsbActionType::MistakeEntry,
            SsbActionType::ResearchStep,
            SsbActionType::ResearchDecomposeLevel,
            SsbActionType::WebResearchStep,
            SsbActionType::PreviewAvailable,
            SsbActionType::InterventionRequest,
            SsbActionType::InterventionResponse,
            SsbActionType::TaskSubmitted,
            SsbActionType::ExecutionModeChanged,
            SsbActionType::ContextBudgetComputed,
            SsbActionType::SsbFilteredByOverlap,
            SsbActionType::HookLatencyViolation,
        ];

        for (i, at) in types.iter().enumerate() {
            ssb.write(SsbWriteRequest {
                action_type: *at,
                agent_id: format!("agent-{i}"),
                superstep: 0,
                payload: test_payload(*at),
                files: vec![],
            })
            .await
            .unwrap_or_else(|e| panic!("write {at:?} failed: {e}"));
        }

        // Verify count.
        let obj = ssb.read_pool.get().await.expect("pool");
        let count: i64 = obj
            .query_row("SELECT COUNT(*) FROM ssb", [], |row| row.get(0))
            .expect("count");
        // B-10 (DECISION-C): subset count is 35 (Tier-4 added 1, B-10 added 2).
        // B-19A: subset count is 36 (B-19A added HookLatencyViolation).
        // The assertion grows as variants are added to the test's `types` vec
        // above; both must move in lockstep.
        assert_eq!(count, 36, "must have inserted all 36 types in this test list");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_10k_sequential_writes() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let bulk_payload = test_payload(SsbActionType::ToolResult);
        let mut prev_seq = 0i64;
        for i in 0..10_000 {
            let seq = ssb
                .write(SsbWriteRequest {
                    action_type: SsbActionType::ToolResult,
                    agent_id: "agent-bulk".into(),
                    superstep: 0,
                    payload: bulk_payload.clone(),
                    files: vec![],
                })
                .await
                .expect("write");

            if i > 0 {
                assert_eq!(seq, prev_seq + 1, "seq gap at write {i}");
            }
            prev_seq = seq;
        }

        ssb.shutdown();
    }

    // -----------------------------------------------------------------------
    // 3B.3 — Read-path tests
    // -----------------------------------------------------------------------

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_read_pool() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Write 3 entries with different action types
        for (i, at) in [
            SsbActionType::FileWrite,
            SsbActionType::Finding,
            SsbActionType::FileWrite,
        ]
        .iter()
        .enumerate()
        {
            ssb.write(SsbWriteRequest {
                action_type: *at,
                agent_id: format!("agent-{i}"),
                superstep: 0,
                payload: test_payload(*at),
                files: vec![format!("src/f{i}.rs")],
            })
            .await
            .expect("write");
        }

        // query_by_session
        let rows = ssb.query_by_session("test-session").expect("by_session");
        assert_eq!(rows.len(), 3, "query_by_session should return 3 entries");

        // query_by_type
        let rows = ssb
            .query_by_type(SsbActionType::FileWrite)
            .expect("by_type");
        assert_eq!(rows.len(), 2, "query_by_type FileWrite should return 2");

        // query_by_superstep
        let rows = ssb.query_by_superstep(0).expect("by_superstep");
        assert_eq!(rows.len(), 3, "query_by_superstep(0) should return 3");

        // query_by_file (uses rarray)
        let rows = ssb
            .query_by_file(&["src/f0.rs", "src/f2.rs"])
            .expect("by_file");
        assert_eq!(rows.len(), 2, "query_by_file should return 2 entries");

        // query_by_file with empty slice
        let rows = ssb.query_by_file(&[]).expect("by_file_empty");
        assert_eq!(rows.len(), 0, "query_by_file([]) should return 0");

        // last_seq
        let seq = ssb.last_seq().expect("last_seq");
        assert_eq!(seq, Some(3), "last_seq should be 3 after 3 writes");

        // Verify round-trip of action_type through ssb_action_type_from_str
        let finding = ssb
            .query_by_type(SsbActionType::Finding)
            .expect("finding");
        assert_eq!(finding.len(), 1);
        assert_eq!(finding[0].action_type, SsbActionType::Finding);

        ssb.shutdown();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_conflict_detection() {
        let dir = TempDir::new().expect("tempdir");
        let config = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "proj-1".into(),
            session_id: "session-A".into(),
        };
        let mut ssb = Ssb::new(config, test_clock()).expect("ssb init");

        // session-A writes to f1.rs and f2.rs
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::FileWrite,
            agent_id: "agent-A".into(),
            superstep: 1,
            payload: test_payload(SsbActionType::FileWrite),
            files: vec!["src/f1.rs".into(), "src/f2.rs".into()],
        })
        .await
        .expect("write A");

        // Test-only backdoor: direct SQL insert bypasses write_loop and
        // update_hook (which is on writer_conn, not pool connections).
        {
            let obj = ssb.read_pool.get().await.expect("pool conn");
            obj.execute(
                "INSERT INTO ssb (project_id, session_id, agent_id, superstep, action_type, payload, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params!["proj-1", "session-B", "agent-B", 1i64, "file_write", &[2u8][..], 100i64],
            )
            .expect("direct insert ssb");
            let seq = obj.last_insert_rowid();
            obj.execute(
                "INSERT INTO ssb_files (seq, file_path) VALUES (?1, ?2)",
                params![seq, "src/f1.rs"],
            )
            .expect("direct insert ssb_files");
        }

        // query_conflict: session-A asks who else touched f1.rs in superstep 1
        let conflicts = ssb
            .query_conflict(&["src/f1.rs"], "session-A", 1)
            .expect("conflict");
        assert_eq!(conflicts, vec!["agent-B"], "should detect agent-B");

        // f2.rs was only touched by session-A, no conflict for session-B asking
        let conflicts = ssb
            .query_conflict(&["src/f2.rs"], "session-B", 1)
            .expect("conflict f2");
        assert_eq!(conflicts, vec!["agent-A"], "should detect agent-A for f2");

        // No conflict for untouched file
        let conflicts = ssb
            .query_conflict(&["src/untouched.rs"], "session-A", 1)
            .expect("no conflict");
        assert!(conflicts.is_empty(), "no conflict for untouched file");

        // Empty file_paths → empty result
        let conflicts = ssb
            .query_conflict(&[], "session-A", 1)
            .expect("empty paths");
        assert!(conflicts.is_empty(), "empty paths = empty result");

        ssb.shutdown();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_conflict_cross_project() {
        let dir = TempDir::new().expect("tempdir");

        // Two SSB instances with different project IDs sharing the same DB
        let config_a = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "proj-alpha".into(),
            session_id: "session-1".into(),
        };
        let config_b = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "proj-beta".into(),
            session_id: "session-2".into(),
        };
        let mut ssb_a = Ssb::new(config_a, test_clock()).expect("ssb_a init");
        let mut ssb_b = Ssb::new(config_b, test_clock()).expect("ssb_b init");

        // proj-alpha writes to shared.rs
        ssb_a
            .write(SsbWriteRequest {
                action_type: SsbActionType::FileWrite,
                agent_id: "agent-alpha".into(),
                superstep: 0,
                payload: test_payload(SsbActionType::FileWrite),
                files: vec!["src/shared.rs".into()],
            })
            .await
            .expect("write alpha");

        // proj-beta writes to shared.rs
        ssb_b
            .write(SsbWriteRequest {
                action_type: SsbActionType::FileWrite,
                agent_id: "agent-beta".into(),
                superstep: 0,
                payload: test_payload(SsbActionType::FileWrite),
                files: vec!["src/shared.rs".into()],
            })
            .await
            .expect("write beta");

        // Cross-project isolation: all read methods must be project-scoped.

        // query_conflict: proj-alpha should NOT see proj-beta's writes
        let conflicts = ssb_a
            .query_conflict(&["src/shared.rs"], "session-1", 0)
            .expect("cross-project conflict");
        assert!(
            conflicts.is_empty(),
            "cross-project should not detect conflict, got: {conflicts:?}"
        );

        // query_by_session: ssb_a should only see its own session's entries
        let rows = ssb_a
            .query_by_session("session-1")
            .expect("by_session alpha");
        assert_eq!(rows.len(), 1, "alpha should see only its own entry");
        assert_eq!(rows[0].project_id, "proj-alpha");

        // query_by_file: ssb_a should only see alpha's write to shared.rs
        let rows = ssb_a
            .query_by_file(&["src/shared.rs"])
            .expect("by_file alpha");
        assert_eq!(rows.len(), 1, "alpha should see only its own file entry");
        assert_eq!(rows[0].agent_id, "agent-alpha");

        // query_by_type: ssb_a should only see its own FileWrite
        let rows = ssb_a
            .query_by_type(SsbActionType::FileWrite)
            .expect("by_type alpha");
        assert_eq!(rows.len(), 1, "alpha should see only its own type entry");

        // query_by_superstep: ssb_a should only see its own superstep 0
        let rows = ssb_a
            .query_by_superstep(0)
            .expect("by_superstep alpha");
        assert_eq!(rows.len(), 1, "alpha should see only its own superstep entry");

        // last_seq: ssb_a should see only its own max seq
        let alpha_seq = ssb_a.last_seq().expect("last_seq alpha");
        let beta_seq = ssb_b.last_seq().expect("last_seq beta");
        assert!(alpha_seq.is_some());
        assert!(beta_seq.is_some());
        // Both wrote 1 entry each. If last_seq were global, alpha would see beta's seq.
        // Since project-scoped, each sees only its own.
        assert_ne!(
            alpha_seq, beta_seq,
            "project-scoped last_seq should differ (different seqs per project)"
        );

        ssb_a.shutdown();
        ssb_b.shutdown();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_append_only() {
        use std::sync::atomic::AtomicU32;

        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Write one entry so we have a row to attempt mutation on
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::Checkpoint,
            agent_id: "agent-test".into(),
            superstep: 0,
            payload: test_payload(SsbActionType::Checkpoint),
            files: vec![],
        })
        .await
        .expect("write");

        // Use a flag-based hook instead of panic (panics in C FFI callbacks abort).
        // This tests the hook LOGIC (detection of UPDATE/DELETE), not the panic path.
        let violation_count = Arc::new(AtomicU32::new(0));
        let vc = violation_count.clone();

        let db_path = dir.path().join("test_ssb.db");
        let direct_conn = Connection::open(&db_path).expect("direct conn");
        direct_conn
            .update_hook(Some(
                move |action: rusqlite::hooks::Action, _db: &str, table: &str, _rowid: i64| {
                    if (table == "ssb" || table == "ssb_files")
                        && (action == rusqlite::hooks::Action::SQLITE_UPDATE
                            || action == rusqlite::hooks::Action::SQLITE_DELETE)
                    {
                        vc.fetch_add(1, Ordering::Relaxed);
                    }
                },
            ))
            .expect("set hook");

        // INSERT should NOT trigger the violation counter
        direct_conn
            .execute(
                "INSERT INTO ssb (project_id, session_id, agent_id, superstep, action_type, payload, created_at) \
                 VALUES ('p', 's', 'a', 0, 'checkpoint', X'01', 100)",
                [],
            )
            .expect("insert ok");
        assert_eq!(
            violation_count.load(Ordering::Relaxed),
            0,
            "INSERT should not trigger append-only hook"
        );

        // UPDATE should trigger
        direct_conn
            .execute("UPDATE ssb SET agent_id = 'hacked' WHERE seq = 1", [])
            .expect("update executes");
        assert_eq!(
            violation_count.load(Ordering::Relaxed),
            1,
            "UPDATE should trigger append-only hook"
        );

        // DELETE should also trigger
        direct_conn
            .execute("DELETE FROM ssb WHERE seq = 2", [])
            .expect("delete executes");
        assert_eq!(
            violation_count.load(Ordering::Relaxed),
            2,
            "DELETE should trigger append-only hook"
        );

        // Verify the production code installs the hook on writer (debug mode).
        // The Ssb writer_conn has the panic-based hook. We test its presence
        // indirectly: if write() succeeds (INSERT), the hook didn't fire.
        let seq = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::Checkpoint,
                agent_id: "agent-verify".into(),
                superstep: 0,
                payload: test_payload(SsbActionType::Checkpoint),
                files: vec![],
            })
            .await
            .expect("write after hook installed");
        assert!(seq > 0, "write should succeed (INSERT allowed by hook)");

        ssb.shutdown();
    }

    // -----------------------------------------------------------------------
    // 3B.4 — Typed payload tests
    // -----------------------------------------------------------------------

    fn all_test_payloads() -> Vec<SsbPayload> {
        vec![
            SsbPayload::FileWrite(FileWritePayload {
                path: "src/main.rs".into(),
                size_bytes: 1024,
                hash: "abc123".into(),
            }),
            SsbPayload::FileRead(FileReadPayload {
                path: "src/lib.rs".into(),
            }),
            SsbPayload::ToolResult(ToolResultPayload {
                tool_id: "grep".into(),
                success: true,
                output_summary: "found 3 matches".into(),
            }),
            SsbPayload::TestOutcome(TestOutcomePayload {
                test_name: "test_foo".into(),
                passed: true,
                duration_ms: 42,
            }),
            SsbPayload::Finding(FindingPayload {
                severity: "high".into(),
                category: "security".into(),
                message: "SQL injection".into(),
                file_path: Some("src/db.rs".into()),
                line: Some(99),
                failure_category: None,
                audit_step: None,
                few_shot_examples: None,
            }),
            SsbPayload::Claim(ClaimPayload {
                claim_text: "blake3 is faster".into(),
                grounding_source: "benchmark".into(),
                verified: true,
            }),
            SsbPayload::Checkpoint(CheckpointPayload {
                agent_state_hash: "deadbeef".into(),
                grc_history_hash: "cafebabe".into(),
                agent_id: "agent-001".into(),
                step_index: 5,
                file_sizes: vec![("state.msgpack".into(), 1024)],
            }),
            SsbPayload::ModelSwap(ModelSwapPayload {
                from_model: "sonnet".into(),
                to_model: "opus".into(),
                gpu_id: 0,
                swap_duration_ms: 150,
                reason: "complex task".into(),
            }),
            SsbPayload::PhaseTransition(PhaseTransitionPayload {
                from_state: "PLANNING".into(),
                to_state: "EXECUTING".into(),
                evidence_summary: "all tests pass".into(),
            }),
            SsbPayload::KnowledgeDegradation(KnowledgeDegradationPayload {
                component: "compaction".into(),
                old_mode: "full".into(),
                new_mode: "summary".into(),
                reason: "memory pressure".into(),
            }),
            SsbPayload::IsolationMetric(IsolationMetricPayload {
                agent_id: "agent-001".into(),
                phase_id: "3B".into(),
                context_savings_ratio: 2.5,
            }),
            SsbPayload::PlanUpdate(PlanUpdatePayload {
                phase_id: "3B".into(),
                version: "v1.2".into(),
                change_summary: "added step 4".into(),
            }),
            SsbPayload::ReplanRequest(ReplanRequestPayload {
                agent_id: "agent-0".into(),
                severity: "MAJOR".into(),
                evidence: "scope change detected".into(),
            }),
            SsbPayload::StubReference(StubReferencePayload {
                phase_id: "3B.4".into(),
                step_id: "step_1".into(),
                stub_hash: "abc123".into(),
            }),
            SsbPayload::ReasoningTrace(ReasoningTracePayload {
                agent_id: "agent-001".into(),
                step_index: 0,
                step_type: "observe".into(),
                content_summary: "read file".into(),
                grounding_source: "file:src/main.rs".into(),
                grounding_level: "HIGH".into(),
                confidence: 0.9,
                elapsed_ms: 100,
                topic: "debug".into(),
                budget_remaining: 10,
            }),
            SsbPayload::PhaseComplete(PhaseCompletePayload {
                phase_id: "3B.4".into(),
                agent_id: "agent-main".into(),
                fsm_state: "COMPLETE".into(),
                evidence_artifacts: vec!["test-results.json".into()],
                commit_hashes: vec!["abc1234".into()],
                test_outcomes: vec!["20 passed, 0 failed".into()],
            }),
            SsbPayload::MemoryMonitoring(MemoryMonitoringPayload {
                channel_id: "heap_mb".into(),
                deep_size_bytes: 512000000,
                threshold_exceeded: false,
            }),
            SsbPayload::DeathEvent(DeathEventPayload {
                agent_id: "agent-dead".into(),
                classification: "crash".into(),
                prior_step_index: 10,
                checkpoint_path: "sessions/default/agents/abc/abc".into(),
                had_checkpoint: true,
            }),
            SsbPayload::RecoveryEvent(RecoveryEventPayload {
                agent_id: "agent-dead".into(),
                classification: "crash".into(),
                recovered_from_checkpoint: true,
                degraded: false,
                first_grounding_source: "CHECKPOINT:sessions/default/agents/abc/abc".into(),
                prior_step_index: 10,
                recovery_step_index: 11,
            }),
            SsbPayload::SupervisionExhausted(SupervisionExhaustedPayload {
                agent_id: "agent-exhausted".into(),
                phase_id: "phase-3C".into(),
                restart_count: 4,
                max_restarts: 3,
                restart_history: vec![1, 5, 9, 13],
                escalation_action: "terminate".into(),
            }),
            SsbPayload::SpiralDetected(SpiralDetectedPayload {
                agent_id: "agent-spiral".into(),
                phase_id: "phase-3C".into(),
                spiral_depth: 5,
            }),
            SsbPayload::ContextReduction(ContextReductionPayload {
                level: 1,
                before_tokens: 10000,
                after_tokens: 8000,
                slots_affected: vec![5, 6],
            }),
            SsbPayload::BranchMerge(BranchMergePayload {
                winner_name: "explore-1".into(),
                abandoned_names: vec!["explore-2".into()],
                trunk_snap_id: "snap-merge-001".into(),
                from_snap_ids: vec!["snap-tip-1".into(), "snap-tip-2".into()],
            }),
            SsbPayload::WorkloadProfile(WorkloadProfilePayload {
                agent_role: "executor".into(),
                phase_id: "3B".into(),
                tokens_in: 1000,
                tokens_out: 500,
                latency_ms: 150,
                vram_peak_mb: 512,
                model_id: "gemma4:e2b".into(),
                gpu_id: 0,
                timestamp: 1700000000,
            }),
            SsbPayload::MistakeEntry(MistakeEntryPayload {
                baby_step_id: "step-1".into(),
                entry: elai_core::MistakeEntry {
                    attempt_number: 1,
                    diff_hash: elai_core::DiffHash::from_diff("bad-patch"),
                    failure_category: elai_core::FailureCategory::CompileError,
                    faulty_line: Some(42),
                    root_cause_json: Some("missing semicolon".into()),
                    action_taken: "templated".into(),
                    timestamp: 1700000000,
                },
                cleared: false,
            }),
            SsbPayload::ResearchStep(ResearchStepPayload {
                agent_id: "researcher-0".into(),
                node_id: 42,
                action: "expand".into(),
                phi: 0.75,
                psi: 0.80,
                iteration: 3,
            }),
            SsbPayload::ResearchDecomposeLevel(ResearchDecomposeLevelPayload {
                agent_id: "decomposer-0".into(),
                level: 2,
                concept_count: 4,
                phase_count: 8,
            }),
            SsbPayload::WebResearchStep(WebResearchStepPayload {
                agent_id: "web-researcher-0".into(),
                node_id: 1,
                action: "search".into(),
                phi: 0.6,
                psi: 0.5,
                iteration: 2,
                sources_fetched: 5,
                sources_used: 3,
                fetcher: "reqwest".into(),
                stage: String::new(), stage_phase: String::new(), stage_payload: None,
            }),
            SsbPayload::PreviewAvailable(PreviewAvailablePayload {
                url: "http://localhost:5173".into(),
                step_id: "step-WEB-3".into(),
                slot: "live".into(),
                agent_id: "previewer-0".into(),
                build_success: true,
                dev_server_ok: true,
            }),
            SsbPayload::InterventionRequest(InterventionRequestPayload {
                step_id: "step-4K-1".into(),
                intervention_type: InterventionType::ApproveChange,
                description: "approve change?".into(),
                diff_summary: Some(DiffSummary {
                    files_changed: 2,
                    lines_added: 42,
                    lines_removed: 7,
                }),
                options: vec![
                    InterventionOption::Keyword(InterventionKeyword::Accept),
                    InterventionOption::Keyword(InterventionKeyword::Decline),
                    InterventionOption::Retry(InterventionRetry {
                        retry: "more evidence".into(),
                    }),
                ],
            }),
            SsbPayload::InterventionResponse(InterventionResponsePayload {
                request_id: 42,
                chosen: InterventionOption::Keyword(InterventionKeyword::Accept),
                user_note: Some("looks good".into()),
            }),
            SsbPayload::TaskSubmitted(TaskSubmittedPayload {
                task: "ship the dashboard MVP".into(),
                project_id: "dash-mvp".into(),
                submitted_at_ms: 1_713_300_000_000,
            }),
            SsbPayload::ChatMessage(ChatMessagePayload {
                session_id: "s-dash".into(),
                text: "/task rebuild the dashboard".into(),
                intent: "TASK".into(),
                project_id: "dash-mvp".into(),
                submitted_at_ms: 1_713_300_000_100,
            }),
            SsbPayload::ChatResponse(ChatResponsePayload {
                session_id: "s-dash".into(),
                card_kind: "status".into(),
                body: "routed to TASK".into(),
                project_id: "dash-mvp".into(),
                emitted_at_ms: 1_713_300_000_150,
            }),
            SsbPayload::ExecutionModeChanged(ExecutionModeChangedPayload {
                from_mode: "LOCAL".into(),
                to_mode: "HYBRID".into(),
                project_id: "dash-mvp".into(),
                changed_at_ms: 1_713_300_000_200,
            }),
            // B-10 (DECISION-C) — 2 new variants for the dashboard's per-slot
            // budget bar chart and SSB entry filter transparency.
            SsbPayload::ContextBudgetComputed(ContextBudgetComputedPayload {
                tier: "micro_7b_12gb".into(),
                raw_max: 25_000,
                slots: vec![0, 0, 0, 1250, 6670, 1250, 830, 0],
                allocations: vec![330, 400, 800, 1500, 8000, 1500, 1000, 512],
                adapt_levels_fired: 0,
            }),
            SsbPayload::SsbFilteredByOverlap(SsbFilteredByOverlapPayload {
                entries_total: 12,
                entries_kept: 4,
                overlap_threshold: 0.10,
            }),
            // B-39 chat reflect pipeline.
            SsbPayload::ReflectTriaged(ReflectTriagedPayload {
                turn_id: "turn-1".into(),
                class: "problem".into(),
            }),
            SsbPayload::ReflectCandidate(ReflectCandidatePayload {
                turn_id: "turn-1".into(),
                candidate_index: 0,
                content: "test-content".into(),
            }),
            SsbPayload::ReflectAuditVerdict(ReflectAuditVerdictPayload {
                turn_id: "turn-1".into(),
                best_index: 0,
                rationale: "most direct".into(),
            }),
            SsbPayload::ReflectRefined(ReflectRefinedPayload {
                turn_id: "turn-1".into(),
                total_tokens_used: 0,
                synthesis: "test-synthesis".into(),
            }),
            // B-24 Phase 4I Group C1: media validator beacons.
            SsbPayload::MediaValidationStarted(MediaValidationStartedPayload {
                step_id: "step-1".into(),
                media_type: "image".into(),
                file_path: "/tmp/x.png".into(),
            }),
            SsbPayload::MediaTier1Complete(MediaTier1CompletePayload {
                step_id: "step-1".into(),
                pass: true,
                check_count: 0,
                latency_ms: 0,
            }),
            SsbPayload::MediaTier2Complete(MediaTier2CompletePayload {
                step_id: "step-1".into(),
                advisory_pass: true,
                model_used: "mock-vlm".into(),
                latency_ms: 0,
            }),
            SsbPayload::RenderComplete(RenderCompletePayload {
                step_id: "step-1".into(),
                adapter_name: "web".into(),
                frame_count: 1,
                latency_ms: 0,
            }),
            SsbPayload::VisualFeedbackIteration(VisualFeedbackIterationPayload {
                step_id: "step-1".into(),
                iteration: 1,
                issue_count: 0,
                converged: true,
            }),
            SsbPayload::MediaHumanReviewRequired(MediaHumanReviewRequiredPayload {
                step_id: "step-1".into(),
                reason: "test-reason".into(),
            }),
            SsbPayload::GenerationModeSelected(GenerationModeSelectedPayload {
                step_id: "baby_step_99".into(),
                mode: "fim".into(),
                rule_fired: "fim_all_conditions_met".into(),
                reason: "rule_3_fim_eligible".into(),
                preferred_was: None,
                supports_fim: true,
                exec_mode: "local".into(),
            }),
            // B-28 E-4 (SCHEMA_VERSION 32) — focus-tunnel StreamingValidator early-term.
            SsbPayload::StreamingEarlyTerminated(StreamingEarlyTerminatedPayload {
                step_id: "baby_step_77".into(),
                mode: "fim".into(),
                reason: "lazy_pattern".into(),
                reason_detail: Some("rust_todo_macro".into()),
                token_count_at_term: 17,
                total_tokens_received: 17,
                language_hint: "rust".into(),
            }),
            // B-28 WIRING-1 (SCHEMA_VERSION 33) — pre-assembly readiness row.
            SsbPayload::PipelinePreAssembled(PipelinePreAssembledPayload {
                step_idx: 5,
                pass_rate_bp: 8500,
                was_discarded: false,
                depth_at_time: 1,
                bytes_assembled: 4096,
            }),
            // B-28 WIRING-1 (SCHEMA_VERSION 34) — dep-resolver decision row.
            SsbPayload::DependencyResolved(DependencyResolvedPayload {
                language: "rust".into(),
                name: "tokio".into(),
                decision: "auto_approve".into(),
                reason: "allowlist_match".into(),
                install_command: Some("cargo add tokio".into()),
            }),
            // B-28 WIRING-1 (SCHEMA_VERSION 35) — file-router micro-step row.
            SsbPayload::MicroStepCreated(MicroStepCreatedPayload {
                parent_step_id: 11,
                created_path: "crates/foo/src/bar.rs".into(),
                type_hint: "Module".into(),
                reason: "undefined_symbol".into(),
                count: 1,
            }),
            // B-30 (SCHEMA_VERSION 36) — language grammar registry status row.
            SsbPayload::LanguageGrammarStatus(LanguageGrammarStatusPayload {
                loaded: vec!["go".into(), "python".into(), "rust".into()],
                failed: vec![("kotlin".into(), "ABI mismatch".into())],
            }),
            // B-29 Phase 5 (SCHEMA_VERSION 37) — symbol-graph tool call row.
            SsbPayload::SymbolQuery(SymbolQueryPayload {
                tool: "symbol_impact".into(),
                target: "crates/knowledge/src/symbol_graph/traversal.rs".into(),
                hop_count: 2,
                total_files: 7,
                latency_ms: 3,
            }),
            // B-31.5 (SCHEMA_VERSION 39) — 5 deferred B-31 SSB variants.
            SsbPayload::IndexCheckpoint(IndexCheckpointPayload {
                project_id: "abc123def4567890".into(),
                batches_processed: 2,
                batches_total: Some(4),
                files_processed: 100,
                last_checkpoint_at: 1_700_000_000,
                batch_duration_ms: Some(250),
            }),
            SsbPayload::IndexResume(IndexResumePayload {
                project_id: "abc123def4567890".into(),
                resumed_from_batch: 1,
                files_skipped: 20,
            }),
            SsbPayload::IndexCancel(IndexCancelPayload {
                project_id: "abc123def4567890".into(),
                batches_completed: 1,
                files_indexed: 50,
            }),
            SsbPayload::LockAcquired(LockAcquiredPayload {
                project_id: "abc123def4567890".into(),
                scope: "index".into(),
                holder_pid: 12345,
            }),
            SsbPayload::LockContested(LockContestedPayload {
                project_id: "abc123def4567890".into(),
                scope: "index".into(),
                holder_pid: 99999,
            }),
            // B-33 follow-up (SCHEMA_VERSION 40) — symbol-graph build summary row.
            SsbPayload::SymbolGraphUpdated(SymbolGraphUpdatedPayload {
                project_id: "abc123def4567890".into(),
                file_count: 42,
                symbol_count: 1234,
                edge_count: 5678,
                built_ms: 1_700_000_000_000,
                // C-24 (DRIFT-4): exercise the additive Option<bool> field
                // explicitly in the determinism vector so rmp-serde encoding
                // round-trips Some(true) too (the field is skipped when
                // None per #[serde(skip_serializing_if]).
                is_incremental: Some(true),
            }),
            // B-40 (SCHEMA_VERSION 41) — 5 FRAMING substate beacons.
            SsbPayload::IdeaFramingStarted(IdeaFramingStartedPayload {
                session_id: "s-test".into(),
                prompt: "build a todo app".into(),
                started_at_ms: 1_700_000_000_000,
            }),
            SsbPayload::ProjectClassClassified(ProjectClassClassifiedPayload {
                session_id: "s-test".into(),
                project_class: "saas".into(),
                matched_pattern: Some("react+postgres".into()),
            }),
            SsbPayload::CriticRoundComplete(CriticRoundCompletePayload {
                session_id: "s-test".into(),
                round_index: 1,
                block_count: 0,
                warn_count: 2,
                info_count: 3,
                converged: true,
                category_counts: vec![("clarity".into(), 1), ("scope".into(), 1)],
            }),
            SsbPayload::IdeaFramed(IdeaFramedPayload {
                session_id: "s-test".into(),
                artifact_path: "workflow/sessions/s-test/idea.md".into(),
                project_class: "saas".into(),
                rounds_used: 2,
                tradeoff_count: 3,
                converged: true,
            }),
            SsbPayload::TradeoffCardEmitted(TradeoffCardEmittedPayload {
                session_id: "s-test".into(),
                tradeoff_names: vec!["db".into(), "auth".into()],
                option_counts: vec![3, 2],
            }),
            // B-34 (SCHEMA_VERSION 42) — entry-point + artifact pipeline beacons.
            SsbPayload::EntryPointsDetected(EntryPointsDetectedPayload {
                project_id: "abc123def4567890".into(),
                count: 3,
                by_reason: {
                    let mut m = std::collections::BTreeMap::new();
                    m.insert("orphan".into(), 1);
                    m.insert("well_known:main".into(), 2);
                    m
                },
            }),
            SsbPayload::ArtifactsIndexed(ArtifactsIndexedPayload {
                project_id: "abc123def4567890".into(),
                indexed: vec!["readme".into()],
                up_to_date: vec!["license".into()],
                total_chunks: 12,
                latency_ms: 45,
            }),
            SsbPayload::ArtifactSearched(ArtifactSearchedPayload {
                project_id: "abc123def4567890".into(),
                artifact_name: Some("readme".into()),
                query: "install".into(),
                result_count: 4,
                latency_ms: 7,
            }),
            SsbPayload::ArtifactStale(ArtifactStalePayload {
                project_id: "abc123def4567890".into(),
                artifact_name: "readme".into(),
                changed_files: vec!["README.md".into()],
            }),
            // B-38 (SCHEMA_VERSION 43) — ambiguity continuous-metric beacons.
            SsbPayload::AmbiguityScored(AmbiguityScoredPayload {
                score: 0.42,
                milestone: "progress".into(),
                goal: 0.5,
                constraint: 0.4,
                success: 0.4,
                context: Some(0.3),
                brownfield: true,
            }),
            SsbPayload::AmbiguityFloorFailed(AmbiguityFloorFailedPayload {
                score: 0.2,
                failed_dimensions: vec!["goal_clarity".into()],
            }),
            SsbPayload::AmbiguityStreakAdvanced(AmbiguityStreakAdvancedPayload {
                score: 0.9,
                current_streak: 2,
                required: 3,
            }),
            // B-42 (SCHEMA_VERSION 44) — cosmos tier knowledge worker beacons.
            SsbPayload::CosmosRefreshStarted(CosmosRefreshStartedPayload {
                queued: 3,
                total: 12,
            }),
            SsbPayload::CosmosArtifactUpdated(CosmosArtifactUpdatedPayload {
                name: "saas_pricing_patterns".into(),
                blake3: "f00dfeedcafebabe".into(),
                body_bytes: 4096,
                latency_ms: 120,
            }),
            SsbPayload::CosmosArtifactDecayed(CosmosArtifactDecayedPayload {
                name: "saas_pricing_patterns".into(),
                age_secs: 86_400,
                cadence: "daily".into(),
            }),
            SsbPayload::CosmosGroundingUsed(CosmosGroundingUsedPayload {
                claim_category: "monetization".into(),
                artifact_name: "saas_pricing_patterns".into(),
                matched: true,
            }),
            // B-41 (SCHEMA_VERSION 45) — per-level auditor mode beacons.
            SsbPayload::PerLevelAuditStarted(PerLevelAuditStartedPayload {
                mode: "concepts".into(),
                tier: "concept".into(),
                node_id: 1,
            }),
            SsbPayload::PerLevelAuditVerdict(PerLevelAuditVerdictPayload {
                mode: "phases".into(),
                tier: "phase".into(),
                node_id: 2,
                verdict: "allow".into(),
                findings_count: 0,
                respawned_once: false,
                cascade_to_parent: false,
            }),
            SsbPayload::PerLevelAuditCascade(PerLevelAuditCascadePayload {
                mode: "babystep_detail".into(),
                tier: "baby_step".into(),
                node_id: 42,
                parent_tier: "baby_step".into(),
            }),
            // B-37C (SCHEMA_VERSION 46) — multi-runtime backend beacons.
            SsbPayload::RuntimeFallback(RuntimeFallbackPayload {
                role: elai_ipc_types::AgentRole::Executor,
                primary: elai_providers::runtime::RuntimeBackend::Sidecar,
                attempted: vec![
                    elai_providers::runtime::RuntimeBackend::Sidecar,
                    elai_providers::runtime::RuntimeBackend::AnthropicApi,
                ],
                succeeded: Some(elai_providers::runtime::RuntimeBackend::AnthropicApi),
                reason: "sidecar unavailable, fell through to anthropic_api".into(),
            }),
            SsbPayload::RuntimeHealthChanged(RuntimeHealthChangedPayload {
                backend: elai_providers::runtime::RuntimeBackend::Sidecar,
                prev: elai_providers::runtime::RuntimeHealth::Unavailable {
                    reason: "first observation".into(),
                },
                next: elai_providers::runtime::RuntimeHealth::Ready,
            }),
            SsbPayload::CliSubprocessSpawned(CliSubprocessSpawnedPayload {
                backend: elai_providers::runtime::RuntimeBackend::ClaudeCode,
                pid: 12345,
                argv_redacted: vec!["claude".into(), "-p".into(), "[REDACTED]".into()],
            }),
            SsbPayload::CliSubprocessKilled(CliSubprocessKilledPayload {
                backend: elai_providers::runtime::RuntimeBackend::ClaudeCode,
                pid: 12345,
                exit_code: Some(0),
                reason: "normal exit".into(),
            }),
            // C-1 (SCHEMA_VERSION 47) — 7 knowledge + reasoning producer payloads.
            SsbPayload::GrcStepEmitted(GrcStepEmittedPayload {
                agent_id: "executor".into(),
                topic: "task_runner".into(),
                step_type: "observe".into(),
                grounding_level: "high".into(),
                confidence: 0.85,
                budget_remaining: 5,
                elapsed_ms: 12,
            }),
            SsbPayload::GroundingSweepResult(GroundingSweepResultPayload {
                ungrounded: 1,
                verify_needed: 0,
                grounded: 3,
                top_claim: Some("training:placeholder".into()),
            }),
            SsbPayload::CsqEvaluated(CsqEvaluatedPayload {
                severity: "Critical".into(),
                guard_present: false,
                blocked: true,
                check_count: 3,
            }),
            SsbPayload::AuditorRubberStampRejected(AuditorRubberStampRejectedPayload {
                role: "auditor".into(),
                finding_count: 0,
                expected_min: 1,
            }),
            SsbPayload::HydeQuery(HydeQueryPayload {
                query_hash: "abc123".into(),
                hypothesis_len: 128,
                cache_hit: false,
                latency_ms: 12,
            }),
            SsbPayload::VectorCacheHitRate(VectorCacheHitRatePayload {
                session_id: "test-session".into(),
                hits: 7,
                misses: 3,
                hit_rate_pct: 70.0,
            }),
            SsbPayload::QueryTrace(QueryTracePayload {
                query_id: "11111111-2222-3333-4444-555555555555".into(),
                tier: "STANDARD".into(),
                stage_count: 9,
                result_count: 12,
                total_latency_ms: 245,
            }),
            // C-2 (SCHEMA_VERSION 48) — 10 orchestration + cosmos + stagnation payloads.
            SsbPayload::PhaseNodeReady(PhaseNodeReadyPayload {
                phase_id: 7,
                concept_id: 3,
                label: "phase-test".into(),
                depth: 1,
            }),
            SsbPayload::StepNodeReady(StepNodeReadyPayload {
                step_id: 12,
                phase_id: 7,
                concept_id: 3,
                label: "step-test".into(),
                depth: 2,
            }),
            SsbPayload::BabyStepReady(BabyStepReadyPayload {
                baby_step_id: 42,
                step_id: 12,
                phase_id: 7,
                concept_id: 3,
                label: "baby-step-test".into(),
                depth: 3,
            }),
            SsbPayload::DagRepairEscalated(DagRepairEscalatedPayload {
                rounds: 10,
                remaining_issues: 2,
                last_error: "cycle persists after repair".into(),
            }),
            SsbPayload::AgentStatusChanged(AgentStatusChangedPayload {
                agent_id: "executor-1".into(),
                role: "executor".into(),
                state: "spawned".into(),
                death_class: None,
                timestamp_ms: 1_700_000_000_000,
            }),
            SsbPayload::StagnationSpinningDetected(StagnationSpinningDetectedPayload {
                agent_id: "executor-1".into(),
                pattern: "spinning".into(),
                iteration: 3,
                detail: "step_hash repeated 3 times".into(),
            }),
            SsbPayload::StagnationOscillationDetected(StagnationOscillationDetectedPayload {
                agent_id: "executor-1".into(),
                pattern: "oscillation".into(),
                iteration: 4,
                detail: "period-2 A-B-A-B".into(),
            }),
            SsbPayload::LateralThinkingActivated(LateralThinkingActivatedPayload {
                agent_id: "executor-1".into(),
                persona: "contrarian".into(),
                iteration: 5,
                detail: "switched after spinning".into(),
            }),
            SsbPayload::GoalDriftWarning(GoalDriftWarningPayload {
                agent_id: "executor-1".into(),
                drift_score: 0.42,
                baby_step_id: 17,
                detail: "task drifted from intent".into(),
            }),
            SsbPayload::WatchdogTimeout(WatchdogTimeoutPayload {
                agent_id: "executor-1".into(),
                elapsed_ms: 30_000,
                budget_ms: 20_000,
                detail: "no observable output".into(),
            }),
            // C-7 (SCHEMA_VERSION 49) — reviewer verdict beacon.
            SsbPayload::ChangeDecision(ChangeDecisionPayload {
                decision: "accepted".into(),
                path: "src/foo.rs".into(),
                hunk_id: Some("hunk-1".into()),
                note: None,
            }),
            // C-8 (SCHEMA_VERSION 50) — search-mode cross-process bridge.
            SsbPayload::SearchModeChanged(SearchModeChangedPayload {
                mode: "hybrid".into(),
                history: vec!["lexical".into(), "hybrid".into()],
            }),
            // C-5 (SCHEMA_VERSION 51) — privacy-redaction event.
            SsbPayload::RedactionApplied(RedactionAppliedPayload {
                count: 3,
                fields: vec!["task_summary".into(), "escalation_message".into()],
                baby_step_id: 17,
                ts_ms: 0,
            }),
            // C-10 (SCHEMA_VERSION 52) — lateral-thinking + goal-drift Block.
            SsbPayload::LateralThinkingSucceeded(LateralThinkingSucceededPayload {
                agent_id: "executor-1".into(),
                persona: "hacker".into(),
                iteration: 6,
                detail: "pattern broken after hacker persona".into(),
            }),
            SsbPayload::LateralThinkingExhausted(LateralThinkingExhaustedPayload {
                agent_id: "executor-1".into(),
                personas_tried: 5,
                iteration: 10,
                detail: "all personas tried; pattern persists".into(),
            }),
            SsbPayload::GoalDriftBlock(GoalDriftBlockPayload {
                agent_id: "executor-1".into(),
                drift_score: 0.32,
                baby_step_id: 19,
                detail: "drift_score below block threshold".into(),
            }),
            SsbPayload::WipSquashComplete(WipSquashCompletePayload {
                wip_count: 7,
                squashed: true,
                step_id: "step-42".into(),
            }),
            SsbPayload::AlternativeExplored(AlternativeExploredPayload {
                step_id: "step-c22".into(),
                alternative_index: 1,
                accepted: true,
            }),
            SsbPayload::FeasibilityEscalated(FeasibilityEscalatedPayload {
                reason: "vram_margin_breached".into(),
                variant: "resource_margin_breached".into(),
                step_id: "step-c22".into(),
            }),
            // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
            SsbPayload::BranchForked(BranchForkedPayload {
                branch_name: "feature-x".into(),
                tip_snap_id: "snap-c23-tip".into(),
                parent_snap_id: "snap-c23-parent".into(),
            }),
            SsbPayload::BranchMerged(BranchMergedPayload {
                winner_name: "feature-x".into(),
                trunk_snap_id: "snap-c23-trunk".into(),
                abandoned_names: vec!["experiment-y".into()],
            }),
            SsbPayload::BranchClosed(BranchClosedPayload {
                branch_name: "experiment-y".into(),
                reason: "user-requested".into(),
            }),
            // C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
            // + background-task beacons.
            SsbPayload::PlanSkeleton(PlanSkeletonPayload {
                plan_id: "plan-c25-test".into(),
                step_count: 3,
                concept_class: "Counter".into(),
                plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                source: PlanSkeletonSource::Llm,
            }),
            SsbPayload::CompiledPipelineFired(CompiledPipelineFiredPayload {
                pipeline_id: "pl-c25-test".into(),
                concept_class: "Counter".into(),
                plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                use_count: 4,
            }),
            SsbPayload::TemplateCrystallized(TemplateCrystallizedPayload {
                concept_class: "Counter".into(),
                plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                pipeline_id: "pl-c25-test".into(),
                threshold_n: 3,
            }),
            SsbPayload::BackgroundTaskStarted(BackgroundTaskStartedPayload {
                task_id: "bg-c25-test".into(),
                command_summary: "grep -r foo /large".into(),
                soft_timeout_secs: 30,
                hard_timeout_secs: 600,
            }),
            SsbPayload::BackgroundTaskComplete(BackgroundTaskCompletePayload {
                task_id: "bg-c25-test".into(),
                exit_status: 0,
                duration_secs: 12,
                auto_detached: false,
                timed_out: false,
            }),
            // C-25 audit follow-up (SCHEMA_VERSION 57): missing 6th
            // compiled-pipeline beacon (WARN-1 in audit-C25.md).
            SsbPayload::DecompileTriggered(DecompileTriggeredPayload {
                concept_class: "Counter".into(),
                plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                failure_count: 2,
                trigger_reason: DecompileTriggerReason::ConsecutiveFailures,
            }),
            // C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
            SsbPayload::FutilityDetected(FutilityDetectedPayload {
                error_class: "constrained_decode_reject".into(),
                consecutive_count: 3,
                level: "baby_step".into(),
                escalated: true,
            }),
            SsbPayload::FutilityOutputStagnation(FutilityOutputStagnationPayload {
                similarity: 0.95,
                threshold: 0.90,
                level: "baby_step".into(),
                new_persona: Some("hacker".into()),
            }),
            // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons.
            SsbPayload::WiringEvidence(WiringEvidencePayload {
                entrypoint: "crates/cli/src/main.rs::main".into(),
                produces_symbols_verified: vec![
                    "DeterminismConfig".into(),
                    "WiringContract".into(),
                ],
                reachable_count: 2,
                audit_id: uuid::Uuid::nil(),
            }),
            SsbPayload::WiringViolation(WiringViolationPayload {
                violation_kind: WiringViolationKind::StubPattern,
                file: "crates/orchestrator/src/example.rs".into(),
                line: 99,
                pattern: "todo!()".into(),
                audit_id: uuid::Uuid::nil(),
            }),
            SsbPayload::DeterminismDrift(DeterminismDriftPayload {
                seed: 42,
                run_a_hash: "blake3:aaaa".into(),
                run_b_hash: "blake3:bbbb".into(),
                first_divergence_index: 13,
                audit_id: uuid::Uuid::nil(),
            }),
            // C-28 (SCHEMA_VERSION 60): T-1 mutation testing.
            SsbPayload::MutationSurvived(MutationSurvivedPayload {
                file: "crates/orchestrator/src/example.rs".into(),
                line: 7,
                mutant_kind: "replace_+_with_-".into(),
            }),
            // C-29 (SCHEMA_VERSION 61): T-2 IntentPair + T-3 Metamorphic.
            SsbPayload::IntentConfirmed(IntentConfirmedPayload {
                step_id: 1,
                pair_count: 3,
                confirmed_by: "user".into(),
            }),
            SsbPayload::MrViolated(MrViolatedPayload {
                step_id: 1,
                relation: "inverse".into(),
                counterexample: "sample_index=0 input_hex=01 encoded_hex=ff decoded_hex=01"
                    .into(),
                sample_size: 1,
            }),
            // C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode payloads.
            SsbPayload::SurgicalPlanCreated(SurgicalPlanCreatedPayload {
                task_id: "task_1700000000_c30test".into(),
                complexity_class: "simple".into(),
                blast_radius_files: 2,
                total_loc_estimate: 150,
                step_count: 1,
                regression_gate_snapshot_hash: "blake3:c30aaaaa".into(),
            }),
            SsbPayload::SurgicalLocWarning(SurgicalLocWarningPayload {
                task_id: "task_1700000000_c30test".into(),
                total_loc_estimate: 600,
                threshold: 500,
                mode: "interactive".into(),
            }),
            SsbPayload::OracleCacheHit(OracleCacheHitPayload {
                query_hash_hex: "00".repeat(32),
                source_tier: "t1_authoritative".into(),
                cached_age_secs: 60,
            }),
            SsbPayload::OracleCacheMiss(OracleCacheMissPayload {
                query_hash_hex: "00".repeat(32),
                confidence: "high".into(),
            }),
            SsbPayload::ProjectDetectionSignal(ProjectDetectionSignalPayload {
                signal_name: "cargo_toml".into(),
                weight: "required".into(),
                fired: true,
            }),
            // C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 payloads.
            SsbPayload::SkeletonGenerated(SkeletonGeneratedPayload {
                blank_count: 4,
                language: "rust".into(),
                file_path: "src/lib.rs".into(),
            }),
            SsbPayload::SkeletonFlushed(SkeletonFlushedPayload {
                path: "src/lib.rs".into(),
                completeness_ratio: 1.0,
            }),
            SsbPayload::MiniCriticVerdict(MiniCriticVerdictPayload {
                blank_id: 1,
                verdict: "pass".into(),
                reason: "ok".into(),
                language: "rust".into(),
            }),
            SsbPayload::StructuralRetryStageAttempt(StructuralRetryStageAttemptPayload {
                blank_id: 1,
                stage: "template".into(),
                attempt: 1,
                outcome: "converged".into(),
            }),
            SsbPayload::BridgeFileGenerated(BridgeFileGeneratedPayload {
                kind: "mod_rs_aggregator".into(),
                path: "src/mod.rs".into(),
                module_count: 3,
            }),
            // D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle.
            SsbPayload::ArchiveEntryAdded(ArchiveEntryAddedPayload {
                template_id: "00000000000000000000000000000001".into(),
                content_hash: "ff".repeat(32),
                parent_hash: None,
                domain: "code".into(),
                pattern_kind: "decomposition".into(),
                archive_status: "frozen".into(),
            }),
            SsbPayload::ArchiveEntryRetired(ArchiveEntryRetiredPayload {
                template_id: "00000000000000000000000000000001".into(),
                retired_at_ms: 1_700_000_000_000,
                retirement_reason: "degradation".into(),
            }),
            SsbPayload::TemplatePromoted(TemplatePromotedPayload {
                template_id: "00000000000000000000000000000001".into(),
                stage: "stage_50".into(),
                pass_rate: 0.94,
            }),
            SsbPayload::TemplateCandidateRejected(TemplateCandidateRejectedPayload {
                template_id: "00000000000000000000000000000001".into(),
                rejection_reason: "below_floor".into(),
            }),
            // E-5 (SCHEMA_VERSION 65): memory_facts bi-temporal lifecycle beacons.
            SsbPayload::MemoryFactExtracted(MemoryFactExtractedPayload {
                fact_id: "fact-test-extract".into(),
                fact_type: "preference".into(),
                scope: "project".into(),
                project_id: Some("proj-test".into()),
                user_id: "default".into(),
                key: "prefers_tracing_over_log".into(),
                trust_score: 0.9,
            }),
            SsbPayload::MemoryFactSuperseded(MemoryFactSupersededPayload {
                old_fact_id: "fact-test-old".into(),
                new_fact_id: "fact-test-new".into(),
                valid_to_ms: 1_700_000_000_000,
            }),
            // D-2 (SCHEMA_VERSION 66): PPM verifier beacons.
            SsbPayload::PpmScored(PpmScoredPayload {
                candidate_id: "cand-test-1".into(),
                verifier_id: "ppm-test".into(),
                mode: "math".into(),
                score_mean: 0.91,
                score_var: 0.0004,
                num_steps: 3,
                latency_ms: 412,
                cache_hit: false,
            }),
            SsbPayload::PpmGatingFailed(PpmGatingFailedPayload {
                candidate_id: "cand-test-2".into(),
                verifier_id: "ppm-test".into(),
                mode: "math".into(),
                score_mean: 0.42,
                threshold: 0.7,
                num_steps: 2,
            }),
            // E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
            SsbPayload::ChatReflectionCompleted(ChatReflectionCompletedPayload {
                note_id: "note-test-reflect".into(),
                note_preview: "U: i prefer blake3...".into(),
                facts_extracted: 1,
                decay_rows_examined: 4,
                decay_rows_updated: 1,
                core_block_bytes: 256,
            }),
            SsbPayload::MemoryEvolutionRun(MemoryEvolutionRunPayload {
                phase: "end".into(),
                duration_ms: 1_240,
                notes_touched: 8,
            }),
            // E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
            // producer + conflict beacons.
            SsbPayload::MemoryNoteInserted(MemoryNoteInsertedPayload {
                note_id: "note-test-insert".into(),
                scope: "session".into(),
                content_blake3: "0".repeat(64),
                link_count: 2,
            }),
            SsbPayload::MemoryNoteEvolved(MemoryNoteEvolvedPayload {
                note_id: "note-test-evolve".into(),
                old_content_blake3: "1".repeat(64),
                new_content_blake3: "2".repeat(64),
                links_added: 1,
                links_removed: 0,
            }),
            SsbPayload::MemoryConflictDetected(MemoryConflictDetectedPayload {
                fact_id_a: "fact-test-a".into(),
                fact_id_b: "fact-test-b".into(),
                conflict_kind: "duplicate".into(),
            }),
            SsbPayload::CoreBlockReplaced(CoreBlockReplacedPayload {
                user_id: "default".into(),
                old_blake3: "3".repeat(64),
                new_blake3: "4".repeat(64),
                bytes: 512,
            }),
            // D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator.
            SsbPayload::VerifierConsensusPass(VerifierConsensusPassPayload {
                candidate_id: "cand-test-pass".into(),
                template_id: "tpl-test-1".into(),
                verdict: "pass".into(),
                verifier_count: 3,
                pass_count: 2,
                rule_id: "k_of_n".into(),
            }),
            SsbPayload::VerifierConsensusFail(VerifierConsensusFailPayload {
                candidate_id: "cand-test-fail".into(),
                template_id: "tpl-test-1".into(),
                failing_verifier_ids: vec!["v-intrinsic-1".into(), "v-det-1".into()],
                reason: "fail".into(),
            }),
            // D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
            SsbPayload::TripwireFired(TripwireFiredPayload {
                event_id: "00112233445566778899aabbccddeeff".into(),
                tripwire_kind: "iqr_outlier".into(),
                candidate_id: "cand-test-trip".into(),
                observed_value: 42.0,
                threshold: 3.0,
                fired_at_ms: 1_700_000_000_000,
            }),
            SsbPayload::TemplateAutoReverted(TemplateAutoRevertedPayload {
                template_id: "tpl-test-auto-1".into(),
                parent_template_id: Some("tpl-test-parent".into()),
                retirement_reason: "degradation".into(),
                fired_tripwire_id: "00112233445566778899aabbccddeeff".into(),
            }),
            // D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark beacon.
            SsbPayload::PrmBenchRunCompleted(PrmBenchRunCompletedPayload {
                total: 20,
                accuracy: 0.75,
                precision: 0.80,
                recall: 0.70,
                gating_verdict: "pass".into(),
                fixture_blake3: "a".repeat(64),
                category_count: 2,
            }),
            // D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
            SsbPayload::BaiArmUpdated(BaiArmUpdatedPayload {
                template_id: "11112222333344445555666677778888".into(),
                alpha: 9.0,
                beta: 2.0,
                n_pulls: 9,
                mean_estimate: 0.818,
                was_promoted: true,
            }),
            // D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
            SsbPayload::HeldOutTaskAdded(HeldOutTaskAddedPayload {
                task_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                source: "synthetic".into(),
                content_blake3: "b".repeat(64),
                ngram_count: 12,
                ingested_ts_ms: 1_700_000_000_000,
                eligibility_window_ms: 0,
            }),
            SsbPayload::HeldOutTaskUsed(HeldOutTaskUsedPayload {
                task_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                consumer_template_id: "11112222333344445555666677778888".into(),
                used_ts_ms: 1_700_000_005_000,
            }),
            // D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
            SsbPayload::CanaryRunCompleted(CanaryRunCompletedPayload {
                report_id: "c".repeat(64),
                ts_ms: 1_700_000_010_000,
                overall_transfer_score: 0.04,
                decision_kind: "pass".into(),
                dominant_fail_family: String::new(),
            }),
            SsbPayload::CanaryFamilyEvaluated(CanaryFamilyEvaluatedPayload {
                report_id: "c".repeat(64),
                family: "code_fix".into(),
                n_evaluated: 5,
                pass_count: 4,
                fail_count: 1,
                transfer_score: 0.10,
                baseline_score: 0.70,
            }),
            // D-9 (SCHEMA_VERSION 75): L3 frozen-surface auditor beacons.
            SsbPayload::LockedKeysAuditRun(LockedKeysAuditRunPayload {
                audit_id: uuid::Uuid::nil(),
                ts_ms: 1_700_000_020_000,
                n_audited: 2,
                n_violations: 1,
                registry_blake3: "0".repeat(64),
                registry_path: "scripts/locked_keys_registry.toml".into(),
            }),
            SsbPayload::LockedKeysViolationDetected(LockedKeysViolationDetectedPayload {
                audit_id: uuid::Uuid::nil(),
                ts_ms: 1_700_000_020_001,
                attempted_edit: "crates/orchestrator/src/tripwire.rs".into(),
                attempted_lines: vec![0, u32::MAX],
                locked_key_path: "crates/orchestrator/src/tripwire.rs".into(),
                locked_key_signature_blake3: "0".repeat(64),
                reason: "tripwire detection per arxiv:1908.04734".into(),
            }),
            // E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
            SsbPayload::TaskAdvanced(TaskAdvancedPayload {
                task_id: "task-1".into(),
                parent_id: String::new(),
                completion_pct: 50.0,
                status: "in_progress".into(),
                blocker: String::new(),
                ts_ms: 1_700_000_030_000,
            }),
            SsbPayload::AgentlessShortcutTaken(AgentlessShortcutTakenPayload {
                task_id: "task-1".into(),
                reason: "single_file_small".into(),
                ts_ms: 1_700_000_030_001,
            }),
            SsbPayload::GoalDriftFired(GoalDriftFiredPayload {
                initial_goal_blake3: "d".repeat(64),
                drift: 0.42,
                threshold: 0.3,
                step: 3,
                ts_ms: 1_700_000_030_002,
            }),
            // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
            SsbPayload::ClaimRouted(ClaimRoutedPayload {
                claim_id: "claim-d10-test".into(),
                claim_type: "factual".into(),
                mode: "constructive".into(),
                ts_ms: 1_700_000_040_000,
                ctx_summary_blake3: "e".repeat(64),
            }),
            // D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
            SsbPayload::ModeRoutingBenchRunCompleted(ModeRoutingBenchRunCompletedPayload {
                total: 25,
                accuracy: 0.92,
                f1_macro: 0.88,
                gating_verdict: "pass".into(),
                fixture_blake3: "f".repeat(64),
                fixture_row_count: 25,
            }),
            // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
            SsbPayload::CheckerResultEmitted(CheckerResultEmittedPayload {
                checker_id: "regex-default".into(),
                checker_kind: "regex_match".into(),
                result_kind: "pass".into(),
                claim_id: "claim-d11-test".into(),
                ts_ms: 1_700_000_050_000,
                reason: String::new(),
            }),
            // D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
            SsbPayload::AdversaryGenerated(AdversaryGeneratedPayload {
                counter_id: "ctr-d13-test".into(),
                claim_id: "claim-d13-test".into(),
                severity: 0.75,
                model_family: "anthropic".into(),
            }),
            SsbPayload::ConsensusReevaluatedWithCounter(
                ConsensusReevaluatedWithCounterPayload {
                    claim_id: "claim-d13-test".into(),
                    original_verdict: "pass".into(),
                    post_counter_verdict: "fail".into(),
                },
            ),
            // D-12 (SCHEMA_VERSION 81): OperationalizationCheck beacon.
            SsbPayload::BabyStepDispatched(BabyStepDispatchedPayload {
                step_id: "bs-d12-test".into(),
                parent_task_id: "task-d12-test".into(),
                checker_kinds_used: vec!["numeric_range".into()],
                outcome: "pass".into(),
                ts_ms: 1_700_000_060_000,
            }),
            // D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal beacons.
            SsbPayload::ConformalTauComputed(ConformalTauComputedPayload {
                tau: 0.8,
                epsilon: 0.1,
                corpus_size: 40,
                source: "held_out".into(),
                ts_ms: 1_700_000_070_000,
            }),
            SsbPayload::ConformalRefusalDecided(ConformalRefusalDecidedPayload {
                claim_id: "claim-d16-test".into(),
                claim_score: 0.05,
                tau: 0.8,
                decision: "abstain".into(),
                confidence: 0.0,
                ts_ms: 1_700_000_071_000,
            }),
            // D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass beacon.
            SsbPayload::NoveltyClassified(NoveltyClassifiedPayload {
                template_id: "tpl-d15-test".into(),
                class: "discovery_candidate".into(),
                dkg_match_count: 0,
                near_duplicate_threshold: 0.05,
                ts_ms: 1_700_000_080_000,
            }),
            // D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate beacon.
            SsbPayload::ObservableEffectVerified(ObservableEffectVerifiedPayload {
                effect_id: "eff-d18-test".into(),
                trigger_kind: "test_command".into(),
                observation_kind: "string_match".into(),
                sandbox_tier: "no_sandbox".into(),
                outcome: "pass".into(),
                confound_flags: String::new(),
                elapsed_us: 1234,
                ts_ms: 1_700_000_090_000,
            }),
            // D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
            SsbPayload::RefusalRateSnapshot(RefusalRateSnapshotPayload {
                refusal_rate: 0.5,
                window_total: 100,
                abstain_count: 50,
                health_band: "above".into(),
                ts_ms: 1_700_000_100_000,
            }),
            SsbPayload::RefusalSlaBreach(RefusalSlaBreachPayload {
                rate: 0.5,
                threshold: 0.2,
                breach_kind: "above".into(),
                ts_ms: 1_700_000_101_000,
            }),
            // D-22 (SCHEMA_VERSION 86): conformal-calibration validation beacon.
            SsbPayload::ConformalValidationCompleted(ConformalValidationCompletedPayload {
                tau: 0.4,
                epsilon: 0.1,
                empirical_refusal_rate: 0.08,
                gating_verdict: "pass".into(),
                fixture_blake3: "f".repeat(64),
                total: 36,
                ts_ms: 1_700_000_110_000,
            }),
            // D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
            SsbPayload::ArcBenchmarkCompleted(ARCBenchmarkCompletedPayload {
                total: 24,
                honestly_evaluated_count: 11,
                gating_verdict: "honestly_evaluated".into(),
                fixture_blake3: "a".repeat(64),
                dominant_mode: "operational_deferred".into(),
                dominant_novelty_class: "discovery_candidate".into(),
                ts_ms: 1_700_000_120_000,
            }),
        ]
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_typed_payloads() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let payloads = all_test_payloads();
        assert_eq!(
            payloads.len(),
            179,
            "must cover original 32 + 2 CHAT + 1 Tier 4 EXECUTION_MODE_CHANGED + 2 B-10 + 4 B-39 + 6 B-24 + 1 B-28 E-3 + 1 B-28 E-4 + 3 B-28 WIRING-1 + 1 B-30 + 1 B-29 + 5 B-31.5 + 1 B-33-followup + 5 B-40 + 4 B-34 + 3 B-38 + 4 B-42 + 3 B-41 + 4 B-37C + 7 C-1 + 10 C-2 + 1 C-7 + 1 C-8 + 1 C-5 + 3 C-10 + 1 C-13 + 2 C-22 + 3 C-23 + 5 C-25 + 1 C-25-audit + 2 C-26 + 3 C-27 + 1 C-28 + 2 C-29 + 5 C-30 + 5 C-32 + 4 D-1 + 2 E-5 + 2 D-2 + 2 E-1 + 4 E-1 dashboard slice + 2 D-3 + 2 D-4 + 1 D-19 + 1 D-5 + 2 D-6 + 2 D-7 + 2 D-9 + 3 E-46 + 1 D-10 (ClaimRouted) + 1 D-21 (ModeRoutingBenchRunCompleted) + 1 D-11 (CheckerResultEmitted) + 2 D-13 (AdversaryGenerated + ConsensusReevaluatedWithCounter) + 1 D-12 (BabyStepDispatched) + 2 D-16 (ConformalTauComputed + ConformalRefusalDecided) + 1 D-15 (NoveltyClassified) + 1 D-18 (ObservableEffectVerified) + 2 D-17 (RefusalRateSnapshot + RefusalSlaBreach) + 1 D-22 (ConformalValidationCompleted) + 1 D-20 (ARCBenchmarkCompleted)",
        );

        for (i, payload) in payloads.iter().enumerate() {
            let expected_action = payload.action_type();
            let seq = ssb
                .write_payload(
                    &format!("agent-{i}"),
                    i as i64,
                    payload.clone(),
                    vec![format!("file-{i}.rs")],
                )
                .await
                .unwrap_or_else(|e| panic!("write_payload variant {i} failed: {e}"));
            assert!(seq > 0, "seq must be positive for variant {i}");

            // Read back via query_by_type and verify round-trip
            let entries = ssb
                .query_by_type(expected_action)
                .unwrap_or_else(|e| panic!("query_by_type {expected_action:?} failed: {e}"));
            assert!(
                !entries.is_empty(),
                "query_by_type should return entries for {expected_action:?}"
            );
            let entry = entries.last().expect("at least one entry");
            let deserialized: SsbPayload =
                rmp_serde::from_slice(&entry.payload)
                    .unwrap_or_else(|e| panic!("deserialize variant {i} failed: {e}"));
            assert_eq!(
                &deserialized, payload,
                "round-trip mismatch for variant {i}"
            );
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_payload_size_limit() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Build a payload that serializes to > 1 MiB
        let big_content = "x".repeat(2 * 1024 * 1024);
        let big_payload = SsbPayload::ReasoningTrace(ReasoningTracePayload {
            agent_id: "agent-big".into(),
            step_index: 0,
            step_type: "observe".into(),
            content_summary: big_content,
            grounding_source: "g".into(),
            grounding_level: "verified".into(),
            confidence: 0.9,
            elapsed_ms: 100,
            topic: "t".into(),
            budget_remaining: 5,
        });

        let result = ssb.write_payload("agent-big", 0, big_payload, vec![]).await;
        assert!(result.is_err(), "oversized payload must be rejected");
        match result.unwrap_err() {
            SsbError::PayloadTooLarge => {}
            other => panic!("expected PayloadTooLarge, got: {other}"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_trace_integration() {
        let trace = ReasoningTracePayload {
            agent_id: "agent-trace".into(),
            step_index: 1,
            step_type: "hypothesize".into(),
            content_summary: "maybe X".into(),
            grounding_source: "test:unit".into(),
            grounding_level: "verified".into(),
            confidence: 0.8,
            elapsed_ms: 50,
            topic: "debug".into(),
            budget_remaining: 4,
        };

        let payload = trace.to_ssb_payload();
        match &payload {
            SsbPayload::ReasoningTrace(inner) => {
                assert_eq!(inner.step_type, "hypothesize");
                assert_eq!(inner.content_summary, "maybe X");
            }
            other => panic!("expected ReasoningTrace variant, got: {other:?}"),
        }
        assert_eq!(payload.action_type(), SsbActionType::ReasoningTrace);
    }

    // Type mismatch is impossible at the type level: write_payload() derives
    // action_type from the SsbPayload enum variant via action_type(). There is
    // no runtime string matching and no way to pass a mismatched pair. This is
    // a compile-time guarantee, not a runtime check.

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_payload_empty_vecs() {
        // PhaseCompletePayload with empty Vec fields must not error
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let payload = SsbPayload::PhaseComplete(PhaseCompletePayload {
            phase_id: "3B.4".into(),
            agent_id: "a".into(),
            fsm_state: "COMPLETE".into(),
            evidence_artifacts: vec![],
            commit_hashes: vec![],
            test_outcomes: vec![],
        });
        let seq = ssb
            .write_payload("agent-empty", 0, payload, vec![])
            .await
            .expect("empty vecs must be accepted");
        assert!(seq > 0);
    }

    // ---------------------------------------------------------------------
    // C-18 phase-3B backfill — payload-mismatch + write-sequence determinism
    // ---------------------------------------------------------------------

    /// `validate_payload_matches_action_type` (ssb.rs:1660) rejects a write
    /// whose declared `action_type` does not match the actual payload variant
    /// — already covered for FileWrite/ToolResult at ssb.rs:7298.
    ///
    /// This C-18 addition locks the CheckpointPayload -> FileWritePayload
    /// direction (phase-3B MISSING). Writing a Checkpoint variant under the
    /// `FileWrite` action_type must yield `Err(SsbError::ActionTypeMismatch)`
    /// — not a panic, not an Ok.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_payload_mismatch_checkpoint_as_file_write() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // 1. Build a CheckpointPayload and serialise it. Field values are
        //    irrelevant — only the variant discriminant matters for the
        //    mismatch check.
        let checkpoint_payload = SsbPayload::Checkpoint(CheckpointPayload {
            agent_state_hash: "h".into(),
            grc_history_hash: "g".into(),
            agent_id: "a".into(),
            step_index: 0,
            file_sizes: vec![],
        });
        let payload_bytes =
            rmp_serde::to_vec_named(&checkpoint_payload).expect("serialize checkpoint");

        // 2. Attempt to write the bytes under the wrong action_type.
        let result = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::FileWrite, // <- mismatch with Checkpoint payload
                agent_id: "agent-mismatch".into(),
                superstep: 0,
                payload: payload_bytes,
                files: vec![],
            })
            .await;

        // 3. Assert Err(ActionTypeMismatch) with the right declared/actual.
        assert!(result.is_err(), "action_type mismatch must be rejected");
        match result.expect_err("just asserted is_err") {
            SsbError::ActionTypeMismatch { declared, actual } => {
                assert_eq!(declared, "file_write");
                assert_eq!(actual, "checkpoint");
            }
            other => panic!("expected SsbError::ActionTypeMismatch; got {other}"),
        }
    }

    /// Sequence-counter determinism contract (phase-3B MISSING).
    ///
    /// With a `SeededClock` (fixed epoch) and 5 sequential writes, the
    /// returned `seq` values must be exactly 1..=5 (monotonic, starting at 1).
    /// On a second run against a fresh in-memory database with the same
    /// `SeededClock`, the persisted payload bytes for each row must be
    /// byte-identical to the first run — locking out clock-derived
    /// non-determinism in the encoded payload.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_write_sequence_determinism() {
        fn five_writes_payload(i: u32) -> SsbPayload {
            SsbPayload::FileWrite(FileWritePayload {
                path: format!("seqdet_{i}.rs"),
                size_bytes: 16,
                hash: format!("h{i}"),
            })
        }

        async fn run_once() -> (Vec<i64>, Vec<Vec<u8>>) {
            let dir = TempDir::new().expect("tempdir");
            let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
            let ssb = Ssb::new(test_config(&dir), Arc::clone(&clock))
                .expect("ssb init for determinism run");

            let mut seqs = Vec::with_capacity(5);
            for i in 1u32..=5 {
                let seq = ssb
                    .write_payload(
                        "agent-determinism",
                        0,
                        five_writes_payload(i),
                        vec![],
                    )
                    .await
                    .expect("write_payload must succeed for determinism run");
                seqs.push(seq);
            }

            // Read back the payload bytes for each row in seq order.
            // The async write path completes the SQL INSERT before
            // `write_payload` returns the seq (see Ssb::write_loop), so a
            // subsequent SsbReader::query_by_seq is guaranteed to find it.
            // `SsbEntry.payload` is `Arc<[u8]>` — convert to Vec<u8> so the
            // two run results can be compared element-wise without exposing
            // an Arc pointer-equality false-positive.
            let mut payloads: Vec<Vec<u8>> = Vec::with_capacity(5);
            for seq in &seqs {
                let entry = SsbReader::query_by_seq(&ssb, *seq)
                    .expect("query_by_seq must succeed for determinism run")
                    .expect("row must exist after write");
                payloads.push(entry.payload.to_vec());
            }
            (seqs, payloads)
        }

        let (seqs_a, payloads_a) = run_once().await;
        let (seqs_b, payloads_b) = run_once().await;

        assert_eq!(seqs_a, vec![1i64, 2, 3, 4, 5], "first run seqs must be 1..=5");
        assert_eq!(seqs_b, vec![1i64, 2, 3, 4, 5], "second run seqs must be 1..=5");
        assert_eq!(
            payloads_a, payloads_b,
            "payload bytes must be byte-identical across two SeededClock runs"
        );
    }

    mod proptest_payloads {
        use super::*;
        use proptest::prelude::*;

        fn arb_payload() -> impl Strategy<Value = SsbPayload> {
            prop_oneof![
                ("[a-z]{1,20}", 0u64..10000, "[a-f0-9]{8}")
                    .prop_map(|(path, size_bytes, hash)| SsbPayload::FileWrite(
                        FileWritePayload { path, size_bytes, hash }
                    )),
                "[a-z]{1,20}".prop_map(|path| SsbPayload::FileRead(FileReadPayload { path })),
                ("[a-z]{1,10}", any::<bool>(), "[a-z ]{0,30}")
                    .prop_map(|(tool_id, success, output_summary)| SsbPayload::ToolResult(
                        ToolResultPayload { tool_id, success, output_summary }
                    )),
                ("[a-z_]{1,20}", any::<bool>(), 0u64..100000)
                    .prop_map(|(test_name, passed, duration_ms)| SsbPayload::TestOutcome(
                        TestOutcomePayload { test_name, passed, duration_ms }
                    )),
                ("[a-z]{1,10}", "[a-z]{1,10}", "[a-z ]{1,30}", any::<Option<String>>(), any::<Option<u32>>())
                    .prop_map(|(severity, category, message, file_path, line)| SsbPayload::Finding(
                        FindingPayload { severity, category, message, file_path, line, failure_category: None, audit_step: None, few_shot_examples: None }
                    )),
                ("[a-z ]{1,30}", "[a-z]{1,10}", any::<bool>())
                    .prop_map(|(claim_text, grounding_source, verified)| SsbPayload::Claim(
                        ClaimPayload { claim_text, grounding_source, verified }
                    )),
                ("[a-f0-9]{16}", "[a-f0-9]{16}", "[a-z0-9-]{1,20}", any::<u64>())
                    .prop_map(|(agent_state_hash, grc_history_hash, agent_id, step_index)| SsbPayload::Checkpoint(
                        CheckpointPayload { agent_state_hash, grc_history_hash, agent_id, step_index, file_sizes: vec![] }
                    )),
                ("[a-z]{1,10}", "[a-z]{1,10}", any::<u8>(), any::<u64>(), "[a-z ]{1,20}")
                    .prop_map(|(from_model, to_model, gpu_id, swap_duration_ms, reason)| SsbPayload::ModelSwap(
                        ModelSwapPayload { from_model, to_model, gpu_id, swap_duration_ms, reason }
                    )),
                ("[A-Z]{1,10}", "[A-Z]{1,10}", "[a-z ]{1,30}")
                    .prop_map(|(from_state, to_state, evidence_summary)| SsbPayload::PhaseTransition(
                        PhaseTransitionPayload { from_state, to_state, evidence_summary }
                    )),
                ("[a-z]{1,10}", "[a-z]{1,10}", "[a-z]{1,10}", "[a-z ]{1,20}")
                    .prop_map(|(component, old_mode, new_mode, reason)| SsbPayload::KnowledgeDegradation(
                        KnowledgeDegradationPayload { component, old_mode, new_mode, reason }
                    )),
                ("[a-z0-9-]{1,20}", "[0-9A-Z.]{1,10}", any::<f64>().prop_filter("finite", |v| v.is_finite()))
                    .prop_map(|(agent_id, phase_id, context_savings_ratio)| SsbPayload::IsolationMetric(
                        IsolationMetricPayload { agent_id, phase_id, context_savings_ratio }
                    )),
                ("[0-9A-Z.]{1,10}", "[0-9.]{1,5}", "[a-z ]{1,30}")
                    .prop_map(|(phase_id, version, change_summary)| SsbPayload::PlanUpdate(
                        PlanUpdatePayload { phase_id, version, change_summary }
                    )),
                ("[a-z0-9-]{1,20}", "(MINOR|MAJOR)", "[a-z ]{1,30}")
                    .prop_map(|(agent_id, severity, evidence)| SsbPayload::ReplanRequest(
                        ReplanRequestPayload { agent_id, severity, evidence }
                    )),
                ("[0-9A-Z.]{1,10}", "[a-z_]{1,10}", "[a-f0-9]{8}")
                    .prop_map(|(phase_id, step_id, stub_hash)| SsbPayload::StubReference(
                        StubReferencePayload { phase_id, step_id, stub_hash }
                    )),
                ("[a-z0-9-]{1,20}", any::<u32>(), "[a-z]{1,10}", "[a-z ]{1,30}", "[a-z:]{1,20}",
                 "(HIGH|MEDIUM|LOW)", any::<f64>().prop_filter("finite", |v| v.is_finite()),
                 any::<u64>(), "[a-z]{1,20}", any::<u32>())
                    .prop_map(|(agent_id, step_index, step_type, content_summary, grounding_source,
                                 grounding_level, confidence, elapsed_ms, topic, budget_remaining)| SsbPayload::ReasoningTrace(
                        ReasoningTracePayload { agent_id, step_index, step_type, content_summary, grounding_source,
                                                 grounding_level, confidence, elapsed_ms, topic, budget_remaining }
                    )),
                ("[0-9A-Z.]{1,10}", "[a-z-]{1,10}", "[A-Z]{1,10}",
                 proptest::collection::vec("[a-z.]{1,10}", 0..3),
                 proptest::collection::vec("[a-f0-9]{7}", 0..3),
                 proptest::collection::vec("[a-z ]{1,20}", 0..3))
                    .prop_map(|(phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes)|
                        SsbPayload::PhaseComplete(PhaseCompletePayload {
                            phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes,
                        })
                    ),
                ("[a-z_]{1,15}", any::<u64>(), any::<bool>())
                    .prop_map(|(channel_id, deep_size_bytes, threshold_exceeded)| SsbPayload::MemoryMonitoring(
                        MemoryMonitoringPayload { channel_id, deep_size_bytes, threshold_exceeded }
                    )),
                (0u8..5, 0usize..200000, 0usize..200000,
                 proptest::collection::vec(0u8..8, 0..4))
                    .prop_map(|(level, before_tokens, after_tokens, slots_affected)| SsbPayload::ContextReduction(
                        ContextReductionPayload { level, before_tokens, after_tokens, slots_affected }
                    )),
                ("[a-z-]{1,20}",
                 proptest::collection::vec("[a-z-]{1,20}", 0..3),
                 "[a-f0-9]{8}",
                 proptest::collection::vec("[a-f0-9]{8}", 1..4))
                    .prop_map(|(winner_name, abandoned_names, trunk_snap_id, from_snap_ids)| SsbPayload::BranchMerge(
                        BranchMergePayload { winner_name, abandoned_names, trunk_snap_id, from_snap_ids }
                    )),
                ("[a-z0-9-]{1,20}", 1u32..10, "[a-z_]{1,30}", any::<bool>())
                    .prop_map(|(baby_step_id, attempt, action_taken, has_line)| SsbPayload::MistakeEntry(
                        MistakeEntryPayload {
                            baby_step_id,
                            entry: elai_core::MistakeEntry {
                                attempt_number: attempt,
                                diff_hash: elai_core::DiffHash::from_diff(&action_taken),
                                failure_category: elai_core::FailureCategory::CompileError,
                                faulty_line: if has_line { Some(42) } else { None },
                                root_cause_json: None,
                                action_taken,
                                timestamp: 0,
                            },
                            cleared: false,
                        }
                    )),
                ("[a-z0-9-]{1,20}", 0u32..4, 0usize..100, 0usize..100)
                    .prop_map(|(agent_id, level, concept_count, phase_count)| SsbPayload::ResearchDecomposeLevel(
                        ResearchDecomposeLevelPayload { agent_id, level, concept_count, phase_count }
                    )),
                ("[a-z0-9-]{1,20}", 0u64..1000, "[a-z]{1,10}",
                 any::<f64>().prop_filter("finite", |v| v.is_finite()),
                 any::<f64>().prop_filter("finite", |v| v.is_finite()),
                 0u32..100, 0u32..50, 0u32..50)
                    .prop_map(|(agent_id, node_id, action, phi, psi, iteration, sources_fetched, sources_used)|
                        SsbPayload::WebResearchStep(WebResearchStepPayload {
                            agent_id, node_id, action, phi, psi, iteration, sources_fetched, sources_used,
                            fetcher: "reqwest".to_owned(),
                            stage: String::new(),
                            stage_phase: String::new(),
                            stage_payload: None,
                        })
                    ),
                ("http://localhost:[1-9][0-9]{3,4}", "[a-z0-9-]{1,20}",
                 "(live|draft|previous)", "[a-z0-9-]{1,20}",
                 any::<bool>(), any::<bool>())
                    .prop_map(|(url, step_id, slot, agent_id, build_success, dev_server_ok)|
                        SsbPayload::PreviewAvailable(PreviewAvailablePayload {
                            url, step_id, slot, agent_id, build_success, dev_server_ok,
                        })
                    ),
                ("[a-z0-9-]{1,20}", "[a-z ]{1,20}")
                    .prop_map(|(step_id, description)| SsbPayload::InterventionRequest(
                        InterventionRequestPayload {
                            step_id,
                            intervention_type: InterventionType::ApproveChange,
                            description,
                            diff_summary: None,
                            options: vec![InterventionOption::Keyword(InterventionKeyword::Accept)],
                        }
                    )),
                (0i64..10_000, any::<Option<String>>())
                    .prop_map(|(request_id, user_note)| SsbPayload::InterventionResponse(
                        InterventionResponsePayload {
                            request_id,
                            chosen: InterventionOption::Keyword(InterventionKeyword::Accept),
                            user_note,
                        }
                    )),
                ("[a-z ]{1,40}", "[a-z0-9-]{1,20}", 0i64..1_000_000_000_000)
                    .prop_map(|(task, project_id, submitted_at_ms)| SsbPayload::TaskSubmitted(
                        TaskSubmittedPayload { task, project_id, submitted_at_ms }
                    )),
                ("[a-z0-9-]{1,20}", "[a-z/ ]{1,40}", "[A-Z_]{3,12}",
                 "[a-z0-9-]{1,20}", 0i64..1_000_000_000_000)
                    .prop_map(|(session_id, text, intent, project_id, submitted_at_ms)|
                        SsbPayload::ChatMessage(ChatMessagePayload {
                            session_id, text, intent, project_id, submitted_at_ms,
                        })
                    ),
                ("[a-z0-9-]{1,20}", "(status|error|disambiguation|steering_ack)",
                 "[a-z ]{1,40}", "[a-z0-9-]{1,20}", 0i64..1_000_000_000_000)
                    .prop_map(|(session_id, card_kind, body, project_id, emitted_at_ms)|
                        SsbPayload::ChatResponse(ChatResponsePayload {
                            session_id, card_kind, body, project_id, emitted_at_ms,
                        })
                    ),
                ("(LOCAL|CLOUD|HYBRID)", "(LOCAL|CLOUD|HYBRID)",
                 "[a-z0-9-]{1,20}", 0i64..1_000_000_000_000)
                    .prop_map(|(from_mode, to_mode, project_id, changed_at_ms)|
                        SsbPayload::ExecutionModeChanged(ExecutionModeChangedPayload {
                            from_mode, to_mode, project_id, changed_at_ms,
                        })
                    ),
                // B-10 (DECISION-C) — ContextBudgetComputed + SsbFilteredByOverlap
                // arbitraries. tier kept short; slots/allocations 8-element vec.
                ("[a-z0-9_]{1,30}", 0usize..200000,
                 proptest::collection::vec(0u64..10000, 8..=8),
                 proptest::collection::vec(0usize..50000, 8..=8),
                 0u8..=4)
                    .prop_map(|(tier, raw_max, slots, allocations, adapt_levels_fired)|
                        SsbPayload::ContextBudgetComputed(ContextBudgetComputedPayload {
                            tier, raw_max, slots, allocations, adapt_levels_fired,
                        })
                    ),
                (0usize..1000, 0usize..1000, 0.0f32..1.0)
                    .prop_map(|(entries_total, entries_kept, overlap_threshold)|
                        SsbPayload::SsbFilteredByOverlap(SsbFilteredByOverlapPayload {
                            entries_total, entries_kept, overlap_threshold,
                        })
                    ),
                // B-19A (T10) — hook latency violation arbitrary. hook_name
                // / tier kept short ASCII to mirror the discriminant strings.
                ("(PreToolUse|PostToolUse|Stop|PreCompact)", "(T1|T2|T3|Composite)",
                 0u64..1000, 0u64..10000)
                    .prop_map(|(hook_name, tier, budget_ms, actual_ms)|
                        SsbPayload::HookLatencyViolation(HookLatencyViolationPayload {
                            hook_name: hook_name.to_string(),
                            tier: tier.to_string(),
                            budget_ms,
                            actual_ms,
                        })
                    ),
                // B-39 chat reflect pipeline arbitraries.
                ("[a-z0-9-]{1,20}", "(chat|problem|code_fix)")
                    .prop_map(|(turn_id, class)|
                        SsbPayload::ReflectTriaged(ReflectTriagedPayload {
                            turn_id,
                            class: class.to_string(),
                        })
                    ),
                ("[a-z0-9-]{1,20}", 0u32..1024, "[a-z ]{1,40}")
                    .prop_map(|(turn_id, candidate_index, content)|
                        SsbPayload::ReflectCandidate(ReflectCandidatePayload {
                            turn_id,
                            candidate_index,
                            content,
                        })
                    ),
                ("[a-z0-9-]{1,20}", 0u32..1024, "[a-z ]{1,40}")
                    .prop_map(|(turn_id, best_index, rationale)|
                        SsbPayload::ReflectAuditVerdict(ReflectAuditVerdictPayload {
                            turn_id,
                            best_index,
                            rationale,
                        })
                    ),
                ("[a-z0-9-]{1,20}", 0u32..1024, "[a-z ]{1,40}")
                    .prop_map(|(turn_id, total_tokens_used, synthesis)|
                        SsbPayload::ReflectRefined(ReflectRefinedPayload {
                            turn_id,
                            total_tokens_used,
                            synthesis,
                        })
                    ),
                // B-31.5 (SCHEMA_VERSION 39) — 5 deferred B-31 SSB variants.
                ("[a-f0-9]{16}", 0u32..1000, any::<Option<u32>>(), 0u64..100_000, 0i64..2_000_000_000, any::<Option<u64>>())
                    .prop_map(|(project_id, batches_processed, batches_total, files_processed, last_checkpoint_at, batch_duration_ms)|
                        SsbPayload::IndexCheckpoint(IndexCheckpointPayload {
                            project_id,
                            batches_processed,
                            batches_total,
                            files_processed,
                            last_checkpoint_at,
                            batch_duration_ms,
                        })
                    ),
                ("[a-f0-9]{16}", 0u32..1000, 0u64..100_000)
                    .prop_map(|(project_id, resumed_from_batch, files_skipped)|
                        SsbPayload::IndexResume(IndexResumePayload {
                            project_id,
                            resumed_from_batch,
                            files_skipped,
                        })
                    ),
                ("[a-f0-9]{16}", 0u32..1000, 0u64..100_000)
                    .prop_map(|(project_id, batches_completed, files_indexed)|
                        SsbPayload::IndexCancel(IndexCancelPayload {
                            project_id,
                            batches_completed,
                            files_indexed,
                        })
                    ),
                ("[a-f0-9]{16}", "(index|watch)", 1u32..1_000_000)
                    .prop_map(|(project_id, scope, holder_pid)|
                        SsbPayload::LockAcquired(LockAcquiredPayload {
                            project_id,
                            scope: scope.to_string(),
                            holder_pid,
                        })
                    ),
                ("[a-f0-9]{16}", "(index|watch)", 1u32..1_000_000)
                    .prop_map(|(project_id, scope, holder_pid)|
                        SsbPayload::LockContested(LockContestedPayload {
                            project_id,
                            scope: scope.to_string(),
                            holder_pid,
                        })
                    ),
            ]
        }

        proptest! {
            #[test]
            fn test_payload_round_trip(payload in arb_payload()) {
                let bytes = rmp_serde::to_vec_named(&payload)
                    .expect("serialize must succeed");
                let decoded: SsbPayload = rmp_serde::from_slice(&bytes)
                    .expect("deserialize must succeed");
                prop_assert_eq!(&decoded, &payload);
            }
        }
    }
}
