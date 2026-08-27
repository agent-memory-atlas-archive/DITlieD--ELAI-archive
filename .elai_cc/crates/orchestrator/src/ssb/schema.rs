#![allow(unused_imports)]
// SSB schema: version bump history + CREATE TABLE / CHECK constraint SQL.
// Tier-B Move 1 split from ssb.rs (behavior-preserving code movement, df18539).

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

// E-25 (SCHEMA_VERSION 172) -- claude-code-architecture catalog payload
// type used in `SsbPayload::ClaudeCodeArchitectureEvent`. The catalog enum
// + kind discriminator are imported inside the `tests` module where the
// sample-payload constructors live.
use crate::architecture::claude_code_arch::ClaudeCodeArchitectureEventPayload;

// E-29 Tier-B (SCHEMA_VERSION 173) -- agentic-observability event payload
// type used in `SsbPayload::AgenticObservabilityEvent`. The payload + 4-
// arm kind discriminator live inline below (`AgenticObservabilityEventPayload`
// + `AgenticObservabilityEventKind`) since the observability subtree
// (crate::observability) consumes the payload to build the row but does
// not own the wire-format definition. Reuses crate::mast::MastMode so
// the MastFailureClassified arm carries the canonical 14-variant tag
// without duplicating the taxonomy.
use crate::mast::MastMode;

use super::*;
use crate::ssb::{action_type::*, payloads::*, store::*};

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
// C-35 (SCHEMA_VERSION 88): bumped 87 -> 88 for 4 new SsbActionType variants
// covering the tool-attention dynamic gating layer (C-35 plan).
// tool_gated, tool_eager, tool_lazy_resolved, tool_denied -- emitted by
// plugins::mcp::gate::filter_for_state and ::lazy::LazySchemaCache on every
// prompt assembly and lazy schema resolution. Each row carries tool_name,
// fsm_state, prompt_tokens_saved. Producer: plugins/src/mcp/gate.rs + lazy.rs.
// all_test_payloads().len() goes 179 -> 183.
// E-27 (SCHEMA_VERSION 89): atomic bump for 3 new quantization-backend beacons.
//   - quant_backend_selected      -- emitted by sidecar InferenceEngine.__init__ on
//                                    every model load; carries backend, kv_dtype,
//                                    compute_capability, marlin_enabled.
//   - kv_cache_dtype_software_sim -- emitted when FP8 KV runs in software-sim on
//                                    Ampere; carries throughput_cost_pct (~5%).
//   - awq_checkpoint_built        -- emitted by scripts/build_awq.py on success.
// Producers: python/elai_inference/server.py + scripts/build_awq.py.
// Additive only -- no existing variants renumbered.
// C-39 (SCHEMA_VERSION 90): atomic bump for 3 new CLI-via-MCP dispatch beacons.
//   - cli_mcp_dispatch       -- emitted per tool call via --via-mcp. Carries tool_name, port.
//   - cli_mcp_server_started -- emitted on CLI co-process auto-launch. Carries port.
//   - cli_mcp_server_stopped -- emitted on CLI co-process exit. Carries port.
// Producer: crates/cli/src/run_subcommand.rs (McpClientDispatch).
// all_test_payloads().len() goes 186 -> 189.
// E-15 (SCHEMA_VERSION 91): atomic bump for 6 new sandbox hardening beacons.
// all_test_payloads().len() goes 189 -> 195.
// E-24 (SCHEMA_VERSION 92): atomic bump for 6 new MCP hardening beacons.
//   - mcp_schema_pin_violation    -- emitted when blake3 hash of tools/list mismatches pinned hash.
//   - mcp_sanitization_rule_applied -- emitted per Trail-of-Bits line-jumping rule match.
//   - mcp_budget_enforced         -- emitted when tools dropped to fit token budget.
//   - mcp_legacy_sse_transport    -- startup warning when SSE-only transport used.
//   - mcp_llm_guard_blocked       -- emitted when LLM-guard denies an MCP tool call.
//   - anthropic_gateway_detected  -- startup warning when ANTHROPIC_BASE_URL is a gateway URL.
// all_test_payloads().len() goes 195 -> 201.
//
// C-48 (SCHEMA_VERSION 93): atomic bump for 3 new SafeHarbor memory-trust beacons.
//   - memory_tier_assigned    -- emitted when a MemoryFact is stored with its TrustTier.
//   - memory_query_filtered   -- emitted when a retrieval query excludes rows by TierCeiling.
//   - safety_critical_blocked -- emitted when a T2/T3 row is blocked from a safety-critical context.
// all_test_payloads().len() goes 201 -> 204.
//
// E-30 (SCHEMA_VERSION 94): atomic bump for 8 new public-incident hardening beacons.
//   - argv_destructive_denied         -- emitted when check_argv() returns Deny.
//   - argv_approval_required          -- emitted when check_argv() returns RequireHumanApproval.
//   - trust_dialog_hook_deferred      -- emitted when ensure_session_trust() defers hook load (CVE-2025-59536).
//   - trust_dialog_env_deferred       -- emitted when ensure_session_trust() blocks env-redirect (CVE-2026-21852).
//   - mcp_pin_version_refused         -- emitted when an MCP server version is refused (exact-pin policy).
//   - auditor_lineage_collision        -- emitted when auditor/executor share the same lineage group.
//   - untrusted_memory_write_blocked  -- emitted when an untrusted TextSource attempts a memory write.
//   - incident_record_created         -- emitted when an incident postmortem entry is stored.
// all_test_payloads().len() goes 204 -> 212.
//
// E-32 (SCHEMA_VERSION 95): atomic bump for 4 new context-window extension beacons.
//   - fitm_calibration_applied    -- emitted by FoundInTheMiddleCalibrator on each long-context
//                                    request (prompt_len > 8K) when ELAI_FITM_ENABLED=1.
//                                    Carries prompt_len, dummy_len, applied_ms.
//                                    Producer: python/elai_inference/server.py::FoundInTheMiddleCalibrator.
//   - deep_mode_activated         -- emitted at sidecar boot when ELAI_DEEP_MODE=1 activates
//                                    Qwen2.5-7B-Instruct-1M at 64K cap.
//                                    Carries model_id, ctx_cap, vram_gb.
//                                    Producer: python/elai_inference/server.py::activate_deep_mode.
//   - deep_mode_escalation_chosen -- emitted by deep_mode_escalation_hook when SessionDag handoff
//                                    prefers local deep mode over cloud escalation.
//                                    Carries reason, files_referenced.
//                                    Producer: crates/providers/src/runtime/escalation_policy.rs.
//   - yarn_refused                -- emitted at sidecar boot when ELAI_USE_YARN=1 is refused
//                                    for Qwen2.5-Coder models (15-20% short-prompt quality drop
//                                    per vllm issue 18728; arxiv:2601.15300).
//                                    Producer: python/elai_inference/server.py::refuse_yarn_at_boot.
// all_test_payloads().len() goes 212 -> 216.
// Producers: python/elai_inference/server.py + escalation_policy.rs.
//
// E-51 (SCHEMA_VERSION 96): atomic bump for 7 new edge-inference beacons.
//   - mlx_subprocess_spawned      -- emitted by MlxAdapter on vllm-mlx process spawn.
//   - mlx_subprocess_killed       -- emitted by MlxAdapter on vllm-mlx process kill.
//   - llamacpp_subprocess_spawned -- emitted by LlamaCppAdapter on llama-server spawn.
//   - llamacpp_subprocess_killed  -- emitted by LlamaCppAdapter on llama-server kill.
//   - mlx_apple_platform_detected -- emitted once per session on Apple Silicon detection.
//   - llamacpp_ep_detected        -- emitted once per session on EP detection.
//   - runtime_chain_built         -- emitted by CLI bootstrap on every chain construction.
// all_test_payloads().len() goes 216 -> 223.
//
// E-17 SP2 (SCHEMA_VERSION 97): atomic bump for 4 new RAG-retrieval-quality
// beacons emitted by the knowledge crate (knowledge::pipeline::rrf_rerank
// + knowledge::hyde::should_use_hyde gate).
//   - deep_pool_capped         -- emitted once per DEEP-tier query when the
//                                 deep candidate pool cap drops one or more
//                                 input candidates (default cap = 100, see
//                                 knowledge::deep::DEEP_CAP_DEFAULT).
//   - rrf_quality_gate_fired   -- emitted once per DEEP-tier query when the
//                                 RRF quality gate drops candidates below
//                                 knowledge::deep::RRF_FLOOR_DEFAULT (0.005).
//   - hyde_skipped_code_query  -- emitted when knowledge::hyde::should_use_hyde
//                                 returns false on a CodeSymbol / Mixed query.
//   - rrf_component_weights    -- emitted once per query with the bm25 /
//                                 code_vec / text_vec RRF weights active.
// all_test_payloads().len() goes 223 -> 227.
//
// B-3 (SCHEMA_VERSION 98): atomic bump for 4 new SsbActionType variants
// closing the AntiStalenessDrawer "no data yet" stubs + RulesDrawer
// rule-transition surface (per B-3 #6/#7/#8/#11 DEAD UI INVENTORY).
//   - dep_audit_completed        -- one row per dep-audit run (B-23 producer)
//   - baseline_run_completed     -- one row per baseline-harness run (B-4 producer)
//   - deterministic_audit_run    -- one row per deterministic-audit gate run (B-15 producer)
//   - rule_transition            -- one row per rule lifecycle status change
//                                   (PROPOSED->CANDIDATE->SHADOW->ACTIVE->RETIRED);
//                                   sibling to the persistent `rule_transitions`
//                                   SQLite table at rules/src/schema.rs:101 so
//                                   the dashboard can subscribe without polling.
// all_test_payloads().len() goes 227 -> 231.
//
// B-4 (SCHEMA_VERSION 99): bumped 98 -> 99 for SsbActionType::CanaryDrift,
// the canary drift snapshot row emitted by the periodic cascade evaluator
// in cli::live_cli. The dashboard's `get_canary_drift` Tauri command reads
// the latest row so SettingsWindow can render the per-startup cosine vs
// the persisted reference. all_test_payloads().len() goes 231 -> 232.
//
// B-7 (SCHEMA_VERSION 100): batched 4-variant bump 99 -> 100 for
// B-7 producer-side wiring closure. New variants:
//   - TreePositionChanged: per DepthFirstNavigator advance() cursor update
//     ({current_node, visited[], blocked[], skipped[]}).
//   - FailureRecorded: per FailureRecord insert in handle_failure()
//     ({baby_step_id, step_id, phase_id, concept_id, reason, attempts}).
//   - PackageRefVerified: per PackageRef verify_packages() outcome
//     ({name, registry, version, verified}).
//   - TechStackDecided: per TechStackDecision creation
//     ({component, chosen, alternatives, source}).
// TaskSplitPayload extended with optional `phase_id` + `complexity_reason`
// fields (#[serde(default)]) to carry PhaseSizingSplitter context without
// forking a new variant (per extends-vs-forks gotchas.md:90). research_node
// _stage handled by extending the SsbViewer reader over existing
// `web_research_step` rows (stage + stage_phase fields already populated
// by SsbWebResearchStepEmitter::emit_stage at task_runner.rs:2044) —
// no new SsbActionType needed.
// all_test_payloads().len() goes 232 -> 236 (+4 variants).
//
// B-19 (SCHEMA_VERSION 101): single-variant bump 100 -> 101 for
// SsbActionType::MarkerReport — per-agent-execution DAIRA marker aggregation
// row complementing the per-event DairaTrace stream. Plan B-19 task #9.
// The dispatcher emits one MarkerReport per agent boundary summarising the
// count of each of the 15 DAIRA_MARKERS phases observed in that window so
// the dashboard can render a single aggregated row instead of replaying the
// raw per-event trace. action_type_lock.golden bumps 288 -> 289 in lockstep
// (action_type_lock.rs TOTAL_VARIANTS const + ALL_VARIANTS array).
//
// B-25 (SCHEMA_VERSION 102): atomic 5-variant bump 101 -> 102 for the
// Phase 4J Empire integration. New variants:
//   - EmpireConnected            -- factory-brain handshake completes
//                                   (carries brain_id + protocol_version)
//   - EmpireDisconnected         -- bridge transitions to Disconnected
//                                   (carries brain_id + reason)
//   - EmpireEventReceived        -- Empire-side event surfaces on the bus
//                                   (carries source + event_type + payload)
//   - EmpireCapabilityInvoked    -- ElaiNeuron capability dispatch attempt
//                                   (carries capability + caller + allowed)
//   - EmpireRepairProposed       -- repair pipeline surfaces a proposal
//                                   (carries issue + target_path + fix_kind)
// Producer: orchestrator::empire::bridge::EmpireBridge::emit_action when
// the EMPIRE-PENDING activation flip lands. Consumer: dashboard
// EmpireBridgePanel (live event stream + capability log + repair pane).
// all_test_payloads().len() goes 237 -> 242 (+5 variants).
// action_type_lock.rs TOTAL_VARIANTS bumps 298 -> 303 in lockstep.
//
// C-5 follow-on (SCHEMA_VERSION 103, +3 variants): RouteBlocked +
// WarnLimitReached + CostEstimate. Atomic landing:
//   - all_test_payloads().len() goes 242 -> 245 (+3 variants)
//   - action_type_lock.rs TOTAL_VARIANTS bumps 303 -> 306
//   - SQL allowlist gains 'route_blocked', 'warn_limit_reached', 'cost_estimate'
// See workflow/reasoning/C-5-decisions.md for the atomic-bump rationale.
//
// C-36 (SCHEMA_VERSION 104, +1 variant): MucocoMutationApplied. The C-36
// MuCoCo mutation operator catalog (11 paper-grounded Python-source
// operators from arXiv:2604.19086, run inside the Python sidecar) emits
// one SSB row per dispatch — applied or skipped. Atomic landing:
//   - all_test_payloads().len() goes 245 -> 246 (+1 variant)
//   - action_type_lock.rs TOTAL_VARIANTS bumps 306 -> 307
//   - SQL allowlist gains 'mucoco_mutation_applied'
// Producer: orchestrator::metamorphic_testing::dispatch_mucoco_mutation
// call sites + the metamorphic_runner auditor wiring (downstream of C-29).
// Consumer: testing-trust dashboard (research-internal; no new component).
//
// D-3 closure (SCHEMA_VERSION 105, +3 variants): PpmUnavailable +
// VerifierDisagreement + CrossContextDispatched. Atomic landing closes
// the D-3 plan's wiring contract:
//   - PpmUnavailable carried atomically from D-2 (DECISION-1) — sidecar
//     down / health-failure semantics distinct from PpmGatingFailed
//     (which means "scored, below threshold"). Producer: PpmClient health
//     path + agent_pool::run_consensus_pass adapter-error route.
//   - VerifierDisagreement — emitted when one consensus run produced
//     >= 1 Pass AND >= 1 Reject signals. Tripwire-stack (D-4) input.
//   - CrossContextDispatched — emitted on every cross_context::run call
//     regardless of agreement outcome (claim_id, context_a_hash,
//     context_b_hash, jaccard, agreement label).
// Atomic landing:
//   - all_test_payloads().len() goes 246 -> 249 (+3 variants)
//   - action_type_lock.rs TOTAL_VARIANTS bumps 307 -> 310
//   - SQL allowlist gains 'ppm_unavailable', 'verifier_disagreement',
//     'cross_context_dispatched'
// See workflow/reasoning/D-3-decisions.md (DECISION-1 / DECISION-4) for
// the carried-from-D-2 + first-principles cross-context rationale.
// C-37 (SCHEMA_VERSION 106): +1 from 105.
//   - Adds SsbActionType::PerRoleDecodingApplied
//   - Adds PerRoleDecodingAppliedPayload (role, decoding_mode, constrained,
//     max_retries_on_invalid)
//   - SQL allowlist gains 'per_role_decoding_applied'
// See workflow/reasoning/C-37-decisions.md for the GAP-B rationale (the
// C-26 policy table shipped without dispatch wiring; C-37 closes that loop
// via the agent_pool::spawn emit site and the SidecarProvider helper
// `apply_role_default_decoding`). Paper grounding: arXiv:2604.06066 Section
// 4.3 (Structure Snowballing).
// C-38 (SCHEMA_VERSION 107, +3 variants): PagerankRun + DkgRebuildStarted
//   + DkgRebuildComplete. Atomic 3-variant landing.
//   - PagerankRun: emitted by knowledge::pipeline Stage 2 after every
//     `compute_repo_map` invocation (workspace_root_hash, node_count,
//     top_node_hash, elapsed_ms). Closes the silent-skip observability
//     gap flagged in C-38 cross-check.
//   - DkgRebuildStarted / DkgRebuildComplete: emitted by the CLI's
//     `dkg_lifecycle::ensure_dkg_or_rebuild` helper which replaces the
//     never-fires "if dkg.db exists" gate at live_cli.rs:1722-1732. The
//     pair brackets a rebuild path; success=false rows are still emitted
//     so failure is observable. See workflow/sessions/s-25120/aider-elai-
//     pagerank-diff.md for the spec-accuracy companion artifact.
//
// D-4 GAP-2 (SCHEMA_VERSION 108, +1 variant): TripwireFiredConcurrent.
//   Distinguishes the "two-or-more detectors fired in the same observe()
//   tick" event from N independent TripwireFired rows so the D-26
//   concurrent-fire diagnostics consumer (the sole downstream — D-19 and
//   D-27 do NOT consume tripwires per grep evidence) does not have to
//   reconstruct the join from raw rows. Producer: future agent_pool /
//   monitor wiring layer when two firings co-occur; payload carries
//   (candidate_id, tripwire_a, tripwire_b, fired_within_ms, fired_at_ms).
//   Atomic landing:
//   - all_test_payloads().len() goes 249 -> 250 (+1 variant — D-4 GAP-2
//     ships solo; the D-4 GAP-1 `source` field is an additive payload
//     extension that does NOT bump variant count).
//   - action_type_lock.rs TOTAL_VARIANTS bumps 314 -> 315
//   - SQL allowlist gains 'tripwire_fired_concurrent'
//   - TripwireFiredPayload gains `source: String` (#[serde(default)]) —
//     same wire-shape extension pattern as B-28 TaskSplitPayload.
//   See workflow/reasoning/D-4-decisions.md for the SCHEMA-bump
//   rationale (note: plan's 49->50 was stale; actual prior was 107 from
//   C-38, hence the 107->108 jump here).
//
// C-40 (SCHEMA_VERSION 109, +4 variants): GitHub PR adapter producer
//   surface. Atomic 4-variant landing wires the new
//   crate::github_pr::GhCliAdapter into SSB so every PR-touching call
//   (open / status-fetch / close / merge) leaves an audit-grade row.
//   Variants are blake3-prefix-hashed for sensitive fields (pr_url_hash,
//   ci_status_hash, review_state_hash, merge_sha_hash) so dashboards can
//   group by PR identity without surfacing the raw URL/SHA. See
//   workflow/reasoning/C-40-decisions.md.
//   - GithubPrOpened          { pr_number, pr_url_hash, ts_ms }
//   - GithubPrStatusFetched   { pr_number, ci_status_hash, review_state_hash, ts_ms }
//   - GithubPrClosed          { pr_number, ts_ms }
//   - GithubPrMerged          { pr_number, merge_sha_hash, ts_ms }
//
// D-5 closure (SCHEMA_VERSION 110, +1 variant): per-pull Thompson-sample
//   beacon. The pre-existing BaiArmUpdated row (SCHEMA 72) captures the
//   posterior state AFTER an update; ThompsonSampled captures the
//   pre-decision SAMPLE — the value drawn from Beta(alpha, beta) that
//   the Thompson policy compared against the other arms. Producer is
//   the same `bai::emit_bai_round` call site (the row is written
//   immediately before the BaiArmUpdated row, so the wire surface
//   carries the sample, the arm-update, and the promote/reject decision
//   in one ordered triple per round). See workflow/reasoning/D-5-decisions.md.
//   - ThompsonSampled { arm_id, sampled_value, alpha, beta }
//
// C-41 (SCHEMA_VERSION 111, +2 variants): skills-driven workflow
//   formalization. The skill registry in `crate::skill_registry` scores
//   incoming task descriptions against discovered SKILL.md manifests
//   (paper grounding arXiv:2604.21910, S0 -> S3 full-match intent
//   accuracy 44.0% -> 83.3% on Claude Opus 4.6, Section 5.1 Table 2).
//   Every lookup emits SkillTriggered with the top-1 candidate; every
//   subsequent dispatch (when the harness actually calls the resolved
//   skill via `crate::tools::execute_skill`) emits SkillExecuted. The
//   skill name is carried as a blake3-prefix u64 hash via
//   `skill_registry::skill_name_hash` so dashboards group by skill
//   identity without surfacing the raw string (same hashing pattern as
//   the C-40 GitHub PR row family). Atomic 2-variant landing.
//   - SkillTriggered { skill_name_hash, score, top_k_rank, ts_ms }
//   - SkillExecuted  { skill_name_hash, exit_ok, duration_ms, ts_ms }
//
// D-6 closure (SCHEMA_VERSION 112, +2 variants ATOMIC): held-out
//   promotion-gate decision + refresh-event beacons. HeldOutPromotionGate
//   captures the regression-gate outcome computed by
//   `held_out::gate_promotion_via_held_out` (the D-5 BAI plug-in path).
//   HeldOutRefresh captures retire+add bookkeeping when the auto-refresh
//   policy fires. Both variants land in the same SCHEMA bump so the
//   wire layout transitions atomically. See workflow/reasoning/D-6-
//   decisions.md DECISION-1.
//   - HeldOutPromotionGate { task_id, allow, delta, regression_threshold,
//                            passes, fails, reason }
//   - HeldOutRefresh       { source_label, added_count, retired_count,
//                            ts_ms }
//
// C-42 (SCHEMA_VERSION 113, +3 variants ATOMIC): Semantic Gateway L2
//   zero-trust RBAC beacons. Producer is
//   `plugins::mcp_bridge::DefaultMcpBridge::dispatch_tool_call_with_authz`
//   which composes the L2 AuthzPolicy with the existing C-35 ToolGate.
//   Paper grounding: arXiv:2604.25555 (MEDIUM reliability, single-author
//   preprint). Naming + 9-tuple choice is ELAI design — paper does not
//   define event-bus contract (see workflow/papers/semantic-gateway-
//   fuzzer.md "SSB / event vocabulary"). Layer attribution
//   (`deny_reason_label`) mirrors paper Section 5's "reject_reason
//   layer" wire field.
//   - GatewayPolicyAllowed { role_hash, tool_hash, deny_reason_label,
//                            ts_ms }
//     Emitted on every composed-Allow verdict so dashboards can render
//     allow-rate (vs deny-rate) without polling the policy file.
//   - GatewayPolicyDenied  { role_hash, tool_hash, deny_reason_label,
//                            deny_reason_hash, ts_ms }
//     Emitted on every composed-Deny verdict. `deny_reason_label` is
//     one of "authz" / "tool_gate" so the dashboard can branch on the
//     rejecting layer without re-running the policy.
//   - GatewayHitlPending   { role_hash, tool_hash, challenge_hash,
//                            ts_ms }
//     Beacon variant for L3 Crypto-HITL pending state. L3 is DEFERRED-
//     TO-FOLLOWUP (not implemented in session 2); the variant exists so
//     the downstream L3 plan can land emitters without a second SCHEMA
//     bump. `challenge_hash` is the SHA-256-of-(action, params, CoT)
//     evidence package per paper Section 5 — in session 2 the emit-path
//     never fires this variant because no producer is wired.
// D-7 closure (SCHEMA_VERSION 114): 3 cross-domain-canary observation
//   beacons that pair with the previously-shipped CanaryRunCompleted +
//   CanaryFamilyEvaluated rows (which capture the run/family roll-up).
//   These three variants surface the cross-domain regression observation
//   itself + the per-family pass + insufficient-data signal so the BAI
//   promotion gate + the dashboard can branch on the typed event without
//   re-parsing the CanaryRunCompleted decision_kind string.
//   ELAI design hypothesis — NanoFlux (arXiv:2509.23252) provides
//   empirical motivation that multi-domain improvement is achievable but
//   the paper itself defines NO canary mechanism, NO 5% degradation
//   threshold, and NO cross-domain regression safeguard. The 5% fail-band
//   default + the 0.02 marginal-band default + the 10-sample minimum are
//   ELAI policy choices (see workflow/reasoning/D-7-decisions.md
//   DECISION-2 / DECISION-3). The variant set is additive — the two
//   shipped variants (CanaryRunCompleted, CanaryFamilyEvaluated) remain
//   live, the three new ones capture orthogonal semantic events:
//   - CrossDomainRegression  { source_domain, regressed_domain,
//                              delta_pct, threshold_pct }
//     Emitted by `canary::emit_canary_run` when at least one
//     non-target family regressed beyond the fail-band relative to the
//     candidate's target family. `delta_pct` is positive (the magnitude
//     of the regression in percent); `threshold_pct` is the fail-band
//     in percent so dashboards can render both without re-reading the
//     toml.
//   - CanaryFamilyPass       { family, score_pct, sample_count }
//     Emitted per-family on a clean pass (transfer_score above the
//     marginal band, sample_count met). Allows the dashboard to render
//     a per-family pass timeline without scanning the raw
//     CanaryFamilyEvaluated rows.
//   - CanaryInsufficientData { family, sample_count, min_required }
//     Emitted when the per-family sample count fell below the
//     `min_per_family` floor. The family is reported but not counted
//     toward the overall decision.
// D-8 closure (SCHEMA_VERSION 115, +1 variant ATOMIC): cryptographic
//   archive verify SSB row. Producer is the `elai archive verify` CLI
//   driver (crates/cli/src/archive_verify.rs). One row per invocation,
//   regardless of pass / fail, so dashboards + CI can render a verify-
//   history timeline without re-walking the JSONL. Payload carries the
//   audit_id (chain header), the entry totals, and the typed per-entry
//   error list (Vec<ArchiveVerifyError>) — NOT Vec<String>. Typed errors
//   are required because the dashboard MutationArchivePanel needs to
//   discriminate HashMismatch / BrokenParent / DuplicateHash /
//   MalformedJson without parsing freeform strings.
//   - ArchiveVerifyResult { audit_id, total_entries, verified,
//                           error_count, errors: Vec<ArchiveVerifyError> }
//     Emitted by `archive_verify::run` (CLI driver). `verified=true` iff
//     `errors.is_empty()`. ArchiveVerifyError lives in
//     `crate::archive_chain` so the SSB payload and the CLI walker share
//     one canonical type — see workflow/reasoning/D-8-decisions.md
//     DECISION-3 for the typed-vs-stringly justification.
//   ELAI design — arxiv:2507.05619 (T1 cryptographic-checksum
//   recommendation) + DGM §5 (archive practice) are supporting
//   documentation for the integrity-rationale; neither paper specifies
//   the SSB wire layout. See workflow/reasoning/D-8-decisions.md
//   DECISION-5 (PLAN_PAPER_CONFLATION guard).
//   - action_type_lock.rs TOTAL_VARIANTS bumps 330 -> 331
//   - SQL allowlist gains 'archive_verify_result'
// DESTRUCTIVE: existing v114 session DBs are DROPped on first read (no
// migration logic). Matches B-23 / B-24 / B-27 / B-28 / D-6 / C-42 / D-7 precedent.
// C-43 closure (SCHEMA_VERSION 116, +2 variants ATOMIC): Phase 4H
//   scope-freeze ratification + violation beacons. Producer is the
//   `elai freeze ratify` / `elai freeze verify` CLI driver
//   (crates/cli/src/freeze_cmd.rs) plus the dashboard's
//   `scope_freeze_status` Tauri command. The 2 variants land alongside
//   session 3's archive_verify_result (which is already declared in
//   ssb.rs at TOTAL_VARIANTS 331 but pending action_type_lock.rs
//   lockstep). C-43 closes the lockstep gap: action_type_lock.rs
//   TOTAL_VARIANTS bumps 330 -> 333 to cover archive_verify_result +
//   scope_freeze_ratified + scope_freeze_violated.
//   - ScopeFreezeRatified  { manifest_hash, ratifier_id_hash,
//                            item_count, ts_ms }
//     Emitted once per `elai freeze ratify` invocation. Carries the
//     blake3-derived 64-bit manifest hash so the dashboard
//     FreezeStatusBar can detect drift between ratifications without
//     re-reading scope-freeze.toml.
//   - ScopeFreezeViolated  { item_path_hash, frozen_since_ms,
//                            modified_at_ms, ts_ms }
//     Emitted once per offending item observed by `elai freeze verify`
//     OR the dashboard's scope_freeze_status command. CI gates on the
//     `phase_4h_freeze_violated` orchestrator feature; enabling the
//     feature trips the `compile_error!` in scope_freeze.rs.
//   ELAI design — NO arXiv grounding. The source plan cited
//   arXiv:2604.21725 (AEL) but two independent WebFetches confirmed
//   that paper is Thompson-Sampling agent learning, NOT scope-freeze.
//   The citation has been removed. See
//   workflow/sessions/s-25120/mistakes.md for the bad-cite analysis.
// DESTRUCTIVE: existing v115 session DBs are DROPped on first read (no
// migration logic). Matches D-8 / D-7 / C-42 precedent.
// C-44 (SCHEMA_VERSION 117): atomic bump for 1 new SSB action type tied
// to the Cursor Composer 2 runtime backend extension:
//   - BackendCostAccrued — per-invocation cost row emitted by the
//     RuntimeOrchestrator after a Composer 2 (or any other future
//     paid-backend) `complete` call. Carries blake3/Fx-hashed backend
//     + role identifiers (no plaintext leak), token counts, est cost
//     in micros (1e-6 USD), and a producer-side ts_ms. ELAI design
//     hypothesis — Composer 2 paper (arXiv:2603.24477) grounds model
//     identity only, NOT cost-accrual policy. See
//     workflow/papers/composer2-interface.md for the shallow-cite-fit
//     analysis and workflow/sessions/s-25120/mistakes.md for the
//     "shallow-cite-fit" failure-mode entry.
// DESTRUCTIVE: existing v116 session DBs are DROPped on first read (no
// migration logic). Matches D-8 / D-7 / C-42 / C-43 precedent.
// C-45 (SCHEMA_VERSION 120): atomic +3 bump for 3 new SSB action types
// tied to the CANTANTE contrastive credit-attribution module
// (`crate::credit_attribution`). All three carry hashed identifiers (no
// plaintext leak) so the wire surface stays small + dashboard-safe:
//   - AgentRoleDropped — emitted by the harness BEFORE rerouting around
//     a dropped role (audit-trail invariant per the plan: emit-before-
//     reroute). Carries blake3-prefix `role_name_hash` + producer
//     `ts_ms`.
//   - CreditAttributionRun — emitted once per harness invocation at
//     setup. Carries `run_id_hash` (caller-supplied stable id),
//     `dropped_role_name_hash`, `k_rollouts`, `g_partition`, `ts_ms`.
//   - CreditAttributionResult — emitted once per harness invocation at
//     aggregation. Carries `run_id_hash`, blake3-prefix
//     `per_role_scores_hash` (collision-resistant summary of the
//     per-role credit map), `total_roles` covered, `ts_ms`.
//   Paper grounding: arXiv:2605.13295 (Zehle 2026, CANTANTE Section 3.2
//   — K rollouts NOT K^N, scalar c in [-1,+1], g=3 partition). See
//   workflow/papers/cantante-credit-attribution.md.
// DESTRUCTIVE: existing v117..v119 session DBs are DROPped on first read
// (no migration logic). Matches D-8 / C-42 / C-43 / C-44 precedent.
// C-46 (SCHEMA_VERSION 121): atomic +3 bump for 3 new SSB action types
// tied to the Uno-Orchestra parsimonious-routing / selective-delegation
// module (`crate::routing`). All three carry blake3/Fx-hashed task and
// reason identifiers so the wire surface stays small + dashboard-safe:
//   - SelectiveDelegationDecision — emitted once per SelectiveDelegator
//     dispatch call at the spawn boundary. Carries `task_id_hash`,
//     `difficulty` (in [0,1]), `alpha` blend (in [0,1]), the
//     `dispatched_count` of (model, primitive) pairs the policy chose,
//     and producer `ts_ms`.
//   - SelectiveDelegationEscalated — emitted when an audit-bubble or
//     verifier-failure triggers a re-spawn with a broader role set.
//     Carries `task_id_hash`, blake3-prefix `reason_hash`, the
//     `escalated_workers` count (workers added on top of the original
//     dispatch), and producer `ts_ms`.
//   - SelectiveDelegationCostSaved — emitted alongside the Decision
//     beacon when the policy dropped one or more workers from the
//     baseline chain. Carries `task_id_hash`, the `skipped_count` of
//     workers omitted, the estimated `est_savings_micros` (1e-6 USD;
//     non-negative by construction), and producer `ts_ms`.
//   Paper grounding: arXiv:2605.05007 (Cui et al 2026, Uno-Orchestra
//   Sections 5 + 6.1 — selective delegation as the lazy-mode branch,
//   S_max <= 0.10 cost-hacking bound, alpha in [0,1] convex blend,
//   direct-answer-cost-zero). See workflow/papers/uno-orchestra-interface.md.
// DESTRUCTIVE: existing v120 session DBs are DROPped on first read
// (no migration logic). Matches C-45 precedent.
// C-49 (SCHEMA_VERSION 123): atomic +3 bump for 3 new SSB action types
// tied to the iterative-distillation experience module
// (`crate::experience`). All three carry hashed identifiers (no
// plaintext leak) so the wire surface stays small + dashboard-safe:
//   - PromptPolicyDistillationRun — emitted once per
//     `distill_fragments` invocation on the post-audit hook path.
//     Carries `skill_name_hash` (blake3-prefix u64),
//     `trace_count` (number of traces consulted),
//     `fragments_emitted` (size of the ranked output), and producer
//     `ts_ms`.
//   - PromptFragmentSelected — emitted once per fragment included in
//     the distilled output. Carries `skill_name_hash`,
//     blake3-prefix `fragment_hash`, observed `success_rate`,
//     observed `freq`, and producer `ts_ms`.
//   - PromptFragmentDiscarded — emitted once per fragment rejected
//     by the distillation thresholds. Carries `skill_name_hash`,
//     `fragment_hash`, blake3-prefix `reason_hash` (snake_case
//     rejection reason), and producer `ts_ms`.
//   Paper grounding caveats: ELAI heuristic adaptation of
//   arXiv:2605.14443 (Sayana et al 2026) — NOT the RL-trained
//   prompter described in Algorithm 1 (RUNPOD-PENDING). Per-skill
//   binning enforced (paper Section 5.5 cross-model transfer poor).
//   Buffer is task-dependent (Web of Lies in paper Table 1 used no
//   buffer); we expose buffer-vs-no-buffer as the `min_freq=1`
//   pass-through boundary. See
//   workflow/papers/iterative-distillation-interface.md.
// DESTRUCTIVE: existing v121 / v122 session DBs are DROPped on first
// read (no migration logic). Matches C-45 / C-46 / C-47 precedent.
// D-14 (SCHEMA_VERSION 124): atomic +1 bump for the sycophancy-probe
// row pair tied to the new `crate::sycophancy` substrate (Sharma et al
// arXiv:2310.13548, Anthropic, ICLR 2024). Both rows carry the
// `claim_id` correlation key from the D-10 claim-router so the
// dashboard can align the probe pair with the originating claim:
//   - SycophancyProbeDispatched — emitted once per
//     `dispatch_sycophancy_probe_hook` invocation on the post-task
//     hook path. Carries `claim_id`, snake-case `probe_kind` label
//     (one of `feedback` | `are_you_sure` | `answer` | `mimicry`
//     per Sharma §3.1-§3.4), and the producer-side `user_message`
//     anchor.
//   - SycophancyFlag — emitted once per probe verdict. Carries
//     `claim_id`, `flip_detected` boolean, `delta` (string-similarity
//     distance, range `[0.0, 1.0]`), and the snake-case verdict
//     label (one of `convergent` | `divergent_sycophancy_flag` |
//     `inconclusive`).
//   Paper grounding caveats: ELAI structural-defense adaptation of
//   arXiv:2310.13548. Preference models do NOT reliably fix
//   sycophancy (Sharma §4 — 75% PM residual with Claude 2 PM, 25%
//   oracle floor); the probe FLAGS only, the auditor BLOCKs. Challenge
//   text "I don't think that's right. Are you sure?" is wire-locked
//   per Sharma §3.2; biasing hedge "but I'm really not sure" per
//   §3.3. See workflow/papers/sharma-sycophancy-2310.13548-interface.md.
// DESTRUCTIVE: existing v123 session DBs are DROPped on first read
// (no migration logic). Matches C-45 / C-46 / C-47 / C-49 precedent.
// C-50 (SCHEMA_VERSION 125): atomic +1 bump for the Contract-Coding
// STRUCTURAL contract test scaffolding row pair tied to the new
// `crate::contract_coding` substrate (Lin, Zhao, Shi 2026
// arXiv:2604.13100, kernel K = <N, Sigma, Delta> §4.2). Both rows
// carry `fn_name_hash` so the dashboard can align by function:
//   - ContractTestScaffolded — emitted once per
//     `scaffold_contract_test_from_signature` invocation. Carries
//     `fn_name_hash` (blake3 prefix of canonical function name),
//     `contract_kind_hash` (blake3 prefix of namespace section
//     name), and producer `ts_ms`.
//   - ContractViolationDetected — emitted once per heuristic
//     `audit_against_contract` verdict that surfaces a kernel
//     violation. Carries `fn_name_hash`, `violation_kind_hash`
//     (blake3 prefix of `ViolationKind::as_str`), `severity` (u8),
//     and producer `ts_ms`.
//   Paper grounding caveats: the headline "47%" is the Roguelike
//   subset of Greenfield-5 (Table 1, Section 5.2) and TIES Trae;
//   the differentiator the paper argues is qualitative structural
//   integrity (failures confined to Local Logic Errors). Do NOT
//   cite "47% Greenfield-5 improvement" without the subset +
//   structural-integrity qualifier. Full Generator -> Discriminator
//   RL pipeline (Algorithm 1) is RUNPOD-PENDING. See
//   workflow/papers/contract-coding-interface.md and
//   workflow/sessions/s-25120/mistakes.md (5th cite incident: "real
//   but misframed" pattern).
// DESTRUCTIVE: existing v124 session DBs are DROPped on first read
// (no migration logic). Matches C-45 / C-46 / C-47 / C-49 / D-14
// precedent.
// C-52 (SCHEMA_VERSION 126): atomic +1 bump for the Dafny Formal
// Verification Spike (`crate::dafny_verify`) host-side scaffolding.
// Single SsbActionType variant `DafnyVerifyCompleted` carries the
// tier hash (Contextless / Signature / SelfHealing per paper
// RQ1/RQ2/RQ3), the binary `dafny_pass` + `udebug_pass` outcomes,
// the `heal_iter` count (0 = first-shot pass, up to `MAX_HEAL_ITER`
// = 10 for full self-healing exhaustion), and producer `ts_ms`. The
// row is the durable timeline marker the dashboard reads to align
// a verification attempt to its target module. Hashed identifiers
// (no plaintext) so the wire format is privacy-preserving and
// stable across sessions.
//   Paper grounding caveats (arXiv:2604.22601):
//   - The paper's top performer is Gemma 4-31B (RQ3a 90.91%
//     verify@5). ELAI MUST NOT integrate Gemma 4 per memory rule
//     feedback_no_gemma4. ELAI substitutes from the paper's
//     evaluation set: Codestral-22B-v0.1 (63.64% RQ2), GPT-OSS-120B
//     (63.64% RQ2 + 81.82% RQ3b), Qwen 3 Coder 30B (45.45% RQ2).
//   - The current ELAI sidecar default (Qwen 2.5 Coder 7B) is NOT
//     in the paper's evaluation set; expect verify@5 below 45% on
//     the signature tier and document the gap.
//   - The `udebug_pass` field guards against vacuous verification
//     (paper §3): raw `dafny verify` exit-zero is INSUFFICIENT; a
//     spec MUST ALSO pass a uDebug-style functional test suite
//     before claiming a verified status.
//   - Local dev hosts lack the `dafny` binary; live execution is
//     RUNPOD-PENDING. The host-side scaffolding ships fully; the
//     emission site only fires when a remote runner with Dafny is
//     available.
//   See workflow/papers/nl2vc-dafny-interface.md and
//   workflow/reasoning/C-52-decisions.md.
// DESTRUCTIVE: existing v125 session DBs are DROPped on first read
// (no migration logic). Matches C-50 / C-49 / D-14 precedent.
//
// D-16 closure (SCHEMA_VERSION 127): single new variant
// `CalibrationCorpusUpdated` emitted once per
// `TauUpdater::update_tau` durable persistence step. Carries
// (corpus_size, new_tau, alpha, source) so the dashboard's
// ConformalCalibrationPanel can render the audit-trail history of
// every nightly threshold refresh without re-decoding tau.txt. The
// `conformal_tau_computed` row family ships separately when the
// orchestrator computes tau in-process; the new variant fires only
// from the persistent updater path. See
// workflow/reasoning/D-16-decisions.md DECISION-1 + DECISION-8.
// Paper grounding: arXiv:2509.01455 UniCR Eq.5 CRC quantile
// formula; rho (selective risk budget) is DISTINCT from alpha
// (miscoverage rate) per D-16-decisions.md DECISION-4.
//
// D-18 closure (SCHEMA_VERSION 129): atomic +4 bump for the deferred
// 4-variant pack the D-18 plan calls for. The original D-18 pass
// (SCHEMA 84) shipped the closed-enum typed surface
// (TriggerSpec / TypedObservation / SandboxTier / ConfoundFlags) +
// SandboxExecutor trait + verify_observable_effect producer + one
// SSB variant (`observable_effect_verified`). This closure ships the
// remaining 4 variants atomically:
//   * observable_effect_dispatched       -- emitted once per
//     `observable_effect::dispatch_observable_effect` invocation
//     reaching the layer-8 hook. Carries (effect_id, trigger,
//     context_kind, ts_ms). Producer: task_runner
//     `dispatch_observable_effect_hook` (gated on
//     `ELAI_OBSERVABLE_EFFECT_ENABLE`).
//   * goodhart_risk_flag                 -- emitted once per
//     `observable_effect::check_goodhart_risk` flag yielded for a
//     given EffectSet. Per Eureka §3.3 component-variation guard:
//     when one component of an effect is constant across runs
//     (variation < `goodhart_min_component_variation`) the effect
//     is not measuring what it claims. Carries (effect_id, risk_kind,
//     evidence, ts_ms).
//   * sandbox_observation                -- emitted once per
//     Coscientist-style SandboxObservation round-trip. Carries
//     (effect_id, tier, outcome, evidence_count, ts_ms).
//   * production_observation_proposal    -- emitted once per
//     Coscientist-style ProductionObservation proposal. Carries
//     (effect_id, instrument_file, requires_human_review, ts_ms).
//     Per Coscientist §4 invariant: ProductionObservation without an
//     instrument_file MUST gate on human review.
// Paper grounding caveats:
//   - Plan's arxiv:2412.16367 cite is REJECTED (WebSearch confirmed
//     unrelated battery-thermal-runaway paper). Substituted anchors:
//     Voyager arxiv:2305.16291 §3 (skill-with-measurable-effect +
//     state-delta invariant INV-5);
//     Eureka arxiv:2310.12931 §3.3 (component-variation Goodhart
//     check);
//     Coscientist arxiv:2304.05332 §4 (sandbox vs production
//     observation taxonomy + instrument_file invariant).
//   - See workflow/reasoning/D-18-decisions.md DECISION-6 +
//     workflow/papers/voyager-2305.16291-interface.md +
//     workflow/papers/eureka-2310.12931-interface.md +
//     workflow/papers/coscientist-2304.05332-interface.md.
// all_test_payloads().len() goes (current) -> (current+4).
// DESTRUCTIVE: existing v128 session DBs are DROPped on first read.
//
// E-13 (SCHEMA_VERSION 130): atomic +3 bump for the TDD verifier
// agent's per-cycle beacon family. The plan originally proposed 7
// new variants; the scope-lean re-check trimmed it to the three
// essential signals — `TddCycleStarted` (one row per
// Red->Green->Refactor cycle entry), `TddTestFailed` (one row per
// Red-phase fail observed by the agent driver), `TddTestPassed`
// (one row per Green-phase flip with optional mutation-score
// evidence). The auditor consumes these via the existing D-3
// `ConsensusRule::VerifierKindHeterogeneous` aggregator; no new
// aggregation surface ships in E-13.
// Citation discipline: the E-13 plan cites 11 arXiv IDs; only
// LLMorpheus (arXiv:2404.09952) and ThinkPRM (arXiv:2504.16828)
// remain inline-cited in the payload docstrings. The other 9 are
// referenced by author/title in workflow/reasoning/E-13-decisions.md
// only. See workflow/sessions/s-25120/mistakes.md E-13 cluster
// cite-deferral entry.
// all_test_payloads().len() goes (current) -> (current+3).
// DESTRUCTIVE: existing v129 session DBs are DROPped on first read.
//
// E-14 (SCHEMA_VERSION 131): atomic +4 bump for the browser-e2e agent's
// per-journey beacon family. The plan originally proposed 9 new variants;
// the scope-lean re-check trimmed it to the four essential signals —
// `BrowserE2eRunStarted` (one row per journey run entry),
// `BrowserE2eAssertionFailed` / `BrowserE2eAssertionPassed` (per
// assertion outcome), and `BrowserE2eSnapshotCaptured` (per Playwright
// snapshot artifact). The hybrid-diff / VLM / DeployPreviewWorker
// variants from the original plan are RUNPOD-PENDING. See
// workflow/reasoning/E-14-decisions.md.
// all_test_payloads().len() goes 300 -> 304.
// DESTRUCTIVE: existing v130 session DBs are DROPped on first read.
//
// E-18 (SCHEMA_VERSION 132): atomic +4 bump for the tool-use reliability
// gate beacon family. Reuse-heavy posture — E-18 composes on top of D-11
// RegistryCheck + E-35 SearchReplaceMatcher / SidecarXGrammarHook + C-47
// ToolCalibrationRegistry + C-42 AuthzPolicy + C-35 ToolGate rather than
// re-implementing those surfaces. The plan's WIRING_CONTRACT enumerated
// 8 new variants; the E-35 deferred-SSB list separately enumerated 10.
// Scope rebase trimmed to the 4 essentials distinct from BOTH lists:
//   * `ToolReliabilityGatePassed` (one row per Pass verdict from
//     `tool_reliability::ToolReliabilityGate::check`)
//   * `ToolReliabilityGateRejected` (one row per Reject verdict —
//     reject_reason_hash carries the snake_case reason label)
//   * `ToolCallSpilled` (one row per body that exceeds the inline cap
//     and lands at `.elai_cc/.elai_cc/spill/<ts>-<hash>.json`)
//   * `ToolCallRetryEscalated` (one row per RetryDecision::Escalate
//     verdict from `tool_reliability::should_escalate`)
// See workflow/reasoning/E-18-decisions.md.
// all_test_payloads().len() goes 304 -> 308.
// DESTRUCTIVE: existing v131 session DBs are DROPped on first read.
//
// E-37 (SCHEMA_VERSION 133): atomic +9 bump for the prompt-injection
// defense beacon family (OWASP LLM Top 10 LLM01 mitigation). Layers
// additively ABOVE E-24 mcp_schema_pin / sanitize_result / llm_guard and
// E-30 destructive_verb_allowlist / trust_dialog. The 9 variants:
//   * `InjectionAttemptDetected` — heuristic OR LLM classifier surfaces
//     a Suspect verdict on an ingest span. Logged but not blocked.
//   * `InjectionAttemptBlocked` — classifier returns Malicious OR
//     capability_gate rejects argv carrying untrusted provenance.
//   * `SpotlightWrapped` — one row per `wrap_spotlight` envelope
//     emitted around an untrusted ingest span.
//   * `SpotlightBreached` — input contained marker bytes that would
//     break the envelope (attacker tried to forge a wrapper).
//   * `ToolDescriptionPinMismatch` — per-tool blake3 description hash
//     differed from the pinned value (`DescPinStatus::Drift`).
//   * `CapabilityGateRejected` — destructive tool argv carried at
//     least one `TrustLevel::Untrusted` arg.
//   * `ProvenanceTagged` — a value was tagged with a non-trivial
//     Provenance (e.g. ToolOutput, Web, SandboxOutput) — observability
//     for the provenance flow without producing one row per dispatch.
//   * `PromptArmorVerdict` — every classify_injection call emits one
//     row (Clean / Suspect / Malicious) for verdict-rate analysis.
//   * `InjectionFixtureReplayed` — adversarial fixture replay outcome
//     (AgentDojo / EchoLeak / CometJacking). RUNPOD-PENDING for live
//     model runs; the SSB shape ships now so fixture harness can land
//     additively later.
// Paper grounding: arXiv:2403.14720 (Spotlight), arXiv:2507.15219
// (PromptArmor), arXiv:2503.18813 (CaMeL), arXiv:2406.13352 (AgentDojo),
// arXiv:2502.06703 (cheap monitors). All five were inline-cited in the
// module docs; per-paper grounding files MAY land follow-on in
// workflow/papers/. The 4 future-year arxiv IDs (2604.*) cited in the
// plan are DEFERRED — see workflow/sessions/s-25120/mistakes.md.
// See workflow/reasoning/E-37-decisions.md.
// all_test_payloads().len() goes 308 -> 317.
// DESTRUCTIVE: existing v132 session DBs are DROPped on first read.
// E-40 (SCHEMA_VERSION 134): atomic +3 bump for the orchestrator-rooted
// symbol-graph tooling beacons. Carries SymbolGraphQueriedPayload (one row
// per find_defs / find_refs / find_callers / find_callees call from the
// scip_cmd CLI handler), SymbolGraphCacheLoadedPayload (one row per
// elai_graph.json mount), LspOracleConsultedPayload (one row per
// oracle_lookup invocation; on-demand-only discipline — >5 per session is
// a bug). Paper grounding deferred per workflow/sessions/s-25120/mistakes.md
// cluster #6 (Stack Graphs arXiv:2211.01224 + GraphCoder arXiv:2406.07003
// + CodexGraph arXiv:2408.03910 + RANGER arXiv:2509.25257 + Glean Angle
// source-grounding). See workflow/reasoning/E-40-decisions.md.
// all_test_payloads().len() goes 317 -> 320.
// E-41 (SCHEMA_VERSION 135): GitHub PR review surface — 5 atomic variants
// emitted by crate::github_pr::{review, comment_watcher, package_verify,
// semgrep_gate, auto_merge_guard, git_assist} for review composition,
// new-comment polling, slopsquat dep verification, semgrep gate results,
// and auto-merge blocks. Sensitive fields hashed via blake3 (sibling
// C-40 precedent). NO new AgentRole — uses existing PerRoleRuntime
// routing (session-3 cascade-avoidance per E-41 brief).
// all_test_payloads().len() goes 320 -> 325.
// DESTRUCTIVE: existing v134 session DBs are DROPped on first read.
//
// E-44 (SCHEMA_VERSION 136): atomic +5 bump for the durability layer.
// New variants: CheckpointWritten, CheckpointResumed, CheckpointMigrated,
// IdempotentDispatchSkipped, DurabilityLockBlocked. Scope-cut from the
// original E-44 plan's 10-variant proposal to 5 essential variants per
// E-44-decisions.md DECISION-1 (avoid largest-ever cascade post-E-37
// x9 + E-41 x5). all_test_payloads().len() goes 325 -> 330.
// 3 paper cites (arXiv:2603.20625 / 2603.29231 / 2603.22744) DEFERRED
// to mistakes.md cluster #8 per memory feedback_paper_citation_verify.
//
// E-45 (SCHEMA_VERSION 137): atomic +3 bump for the cross-language drift
// detector module. New variants: CrossLangDriftDetected,
// CrossLangPairScanned, SchemaVersionMismatch. Producers:
// orchestrator::cross_lang_drift::detector (host-side drift detection
// over the Rust<->Python sidecar + Rust<->TS dashboard pairs registered
// in cross-lang-pairs.toml). Consumers: drift_cmd CLI subverb (WIRE-DARK
// anchor) + cross_lang_diff.py --json reporting. Paper grounding N/A:
// design grounded in D-10 DECISION-8 (hand-authored TS mirror over
// ts-rs codegen) — Plan Tasks 1+2 (ts-rs codegen) intentionally DEFERRED.
// all_test_payloads().len() goes 325 -> 328 (3 new entries).
//
// E-49 (SCHEMA_VERSION 138): atomic +5 bump for the coding-agent metric
// suite (scope-lean ship). New variants:
//   - CodingAgentMetricsCaptured  (one row per MetricsCollector::capture_metrics)
//   - SweBenchProRunStarted       (one row at the start of a Pro live run)
//   - SweBenchProRunCompleted     (one row at end of a Pro live run)
//   - RefineBenchScoreEmitted     (one row per RefineBench scoring pass)
//   - MetricsAggregationRun       (one row per aggregator invocation)
// Producers: orchestrator::coding_metrics::{collector, swebench_pro,
// refinebench, aggregator}. Consumers: metrics_cmd CLI subverb
// (WIRE-DARK anchor) + dashboard MetricsDrawer (deferred to a follow-on
// patch). Live SWE-Bench Pro + RefineBench runs against real models are
// RUNPOD-PENDING — variant emission stays valid because the report
// shapes are stable; the live transports return RunpodPending. Design
// source: SWE-bench Pro + RefineBench (citations DEFERRED to
// mistakes.md cluster #9). 10th lockstep site (action_type_lock.rs
// TOTAL_VARIANTS) bumped 394 -> 399 in lockstep.
// all_test_payloads().len() goes 328 -> 333 (5 new entries).
//
// E-52 Tier-B (SCHEMA_VERSION 139): atomic bump 138 -> 139 for 4 new SSB
// action types tied to the LSP-as-verifier diagnostic-probe surface and
// the SCIP refresh pipeline:
//   - DiagnosticProbeDispatched (one row per language-grouped probe dispatch)
//   - DiagnosticProbeCompleted  (one row per probe completion with outcome)
//   - DiagnosticProbeBlock      (one row when error-severity diagnostics block)
//   - ScipRefreshCompleted      (one row per .claude/tools/elai.map SCIP merge)
// Design source: LSP base protocol v3.17 (verified spec
// microsoft/language-server-protocol/3.17). Live rust-analyzer/pylsp
// handshakes are RUNPOD-PENDING (entry 15 cluster handoff); the variant
// emission shape is stable. 10th lockstep site (action_type_lock.rs
// TOTAL_VARIANTS) bumped 399 -> 403 in lockstep with this commit.
// all_test_payloads().len() goes 333 -> 337 (4 new entries).
//
// E-53 (SCHEMA_VERSION 140): atomic +1 bump for the formal-verify
// pipeline boundary beacon. Composes C-50 contract_coding +
// C-52 dafny_verify. Lean-scope ship per
// workflow/sessions/s-25120/completion-drive/exploration/E-53-integration.md
// — the plan originally proposed 13 SsbActionType variants; the
// rebased scope consolidates to ONE FormalVerifierCompleted carrying
// verifier_kind + outcome + csq_class discriminators per the C-52
// DafnyVerifyCompleted precedent. 10th lockstep site (action_type_
// lock.rs TOTAL_VARIANTS) bumped 403 -> 404 in lockstep with this
// commit. all_test_payloads().len() goes 337 -> 338 (1 new entry).
// VerifierKind enum in verification_result.rs extended atomically per
// the L3-lock escape clause "New kinds land via plan-level schema
// bumps" (Z3SmtCheck variant added). Gemma-4 BLOCKED per memory
// `feedback_no_gemma4` — pipeline substitutes Codestral / GPT-OSS /
// Qwen3 documented in workflow/reasoning/E-53-decisions.md.
//
// E-35 (SCHEMA_VERSION 141): atomic 10-variant bump for the
// structured-output substrate (XGrammar canonical + INSTRUCTOR +
// per-role grammar cache + CodeAct + SEARCH/REPLACE + cloud
// strict-schema + lm-format-enforcer / legacy json_object refusal).
// Lean-scope ship per the session-2 autonomous-mode brief:
// AgentRole::Runner variant addition DEFERRED per
// E41_AGENTROLE_FROZEN_AT_17 gotcha (230 match-arm cascade — CodeAct
// composes through existing role + RuntimeBackend composition);
// Python sidecar tool_parser.py + XGrammarLogitsProcessor are
// RUNPOD-PENDING (sidecar boot required); SearchReplaceMatcher
// dispatcher tier-0 wire DEFERRED to patch-N (the substrate ships in
// crates/tools/src/search_replace.rs and the SSB beacons are in
// scope; the dispatcher.rs touch is a single-file follow-on); DCCD
// (Task 9) DEFERRED behind env flag per plan L65. 10th lockstep site
// (action_type_lock.rs TOTAL_VARIANTS) bumped 404 -> 414 in lockstep
// with this commit. all_test_payloads().len() goes 338 -> 348 (10
// new entries). Composes on top of the already-shipped Rust
// substrate: StructuredMode / ToolParserVariant / XGrammarRequest /
// XGrammarResponse / CodeActAction / SearchReplaceTier1 in
// crates/ipc-types/src/lib.rs; SidecarXGrammarHook trait +
// xgrammar_cache_key in crates/providers/src/runtime/
// sidecar_xgrammar_hook.rs; SearchReplaceMatcher +
// DelimiterCollisionGuard in crates/tools/src/search_replace.rs;
// GrammarCompilePanel.svelte + commands_e35::get_grammar_compilations
// in crates/dashboard-v2. Inline cite ONLY codeact-2402.01030
// (ingested at workflow/papers/codeact-2402.01030-interface.md);
// XGrammar / JSONSchemaBench / CRANE / Diff-XYZ / "Think Inside the
// JSON" / DCCD / "When Correct Isn't Usable" referenced by name
// only per cluster s-25120 cite-discipline (mistakes.md cite-incident
// cluster #13). See workflow/reasoning/E-35-decisions.md.
// D-23 (SCHEMA_VERSION 142): atomic +1 bump on top of E-35's 141 head.
// Adds one new SsbActionType variant `SycophancyBenchRunCompleted`
// emitted by `crate::benchmarks::sycophancy_runner::SycophancyBenchmark::run`
// at end-of-run. Lockstep cascade: SQL CHECK list (+1 string),
// SsbActionType variant (+1 arm), as_str / from_str (+1 arm each),
// SsbPayload variant (+1 arm), action_type() match (+1 arm), payload
// struct (+1), sample dispatch in `from_action_type_for_smoke` (+1 arm),
// `all_test_payloads` Vec (+1 entry), action_type_lock.rs (TOTAL_VARIANTS
// 414 -> 415, ALL_VARIANTS +1, exhaustive_match_guard +1) +
// action_type_lock.golden (+1 row at alphabetical position
// `sycophancy_bench_run_completed`). 10th lockstep site (the SQL CHECK
// list) sits in CREATE_SSB_TABLE below. See workflow/reasoning/
// D-23-decisions.md DECISION-1.
// D-24 (SCHEMA_VERSION 143): atomic +1 bump on top of D-23's 142 head.
// Adds one new SsbActionType variant `CanaryBenchRunCompleted` emitted
// by `crate::benchmarks::canary_runner::CanaryBenchmark::run` at
// end-of-run. Same canonical 9-tuple cascade + 10th lockstep at
// action_type_lock.rs (TOTAL_VARIANTS 415 -> 416, ALL_VARIANTS +1,
// exhaustive_match_guard +1) + action_type_lock.golden (+1 row at
// alphabetical position `canary_bench_run_completed`, ordinal 0043).
// Plan-doc named the variants `BenchmarkStarted` + `BenchmarkResult`;
// REJECTED per D-19/D-23 single-variant precedent — D-24 ships exactly
// one new variant. See workflow/reasoning/D-24-decisions.md DECISION-1.
// D-25 (SCHEMA_VERSION 144): atomic +1 bump on top of D-24's 143 head.
// Adds one new SsbActionType variant `DeterminismTamperSuiteCompleted`
// emitted by `crate::benchmarks::tamper_runner::TamperBenchmark::run`
// at end-of-run. Same canonical 9-tuple cascade + 10th lockstep at
// action_type_lock.rs (TOTAL_VARIANTS 416 -> 417, ALL_VARIANTS +1,
// exhaustive_match_guard +1) + action_type_lock.golden (+1 row at
// alphabetical position `determinism_tamper_suite_completed`).
// Plan-doc named the variants `BenchmarkStarted` + `BenchmarkResult`;
// REJECTED per D-19/D-23/D-24 single-variant precedent — D-25 ships
// exactly one new variant. The 10-scenario tamper-suite drives D-8
// `verify_archive` IN-PROCESS (NOT shell subprocess — Windows-friendly
// + 5s budget achievable). JSON scenario manifests (NOT TOML — avoids
// new dep, per D-24 DECISION-4 precedent). See workflow/reasoning/
// D-25-decisions.md DECISION-1.
// D-28 (SCHEMA_VERSION 145): atomic +1 bump on top of D-25's 144 head.
// Adds one new SsbActionType variant `AdversarialFarmRunCompleted`
// emitted by `crate::farm::adv_rm::AdvRmFarmer::run` at end-of-run.
// Same canonical 9-tuple cascade + 10th lockstep at
// action_type_lock.rs (TOTAL_VARIANTS 417 -> 418, ALL_VARIANTS +1,
// exhaustive_match_guard +1) + action_type_lock.golden (+1 row at
// alphabetical position `adversarial_farm_run_completed`, ordinal
// 0000 — `adversarial*` sorts BEFORE `adversary_generated` because
// 'i' < 'y' at the 8th character; all subsequent ordinals shift +1).
// Plan-doc named the variants `AdversarialCorpusGenerated` +
// `PpmFinetuneCompleted`; REJECTED per D-19/D-23/D-24/D-25 single-
// variant precedent — D-28 ships exactly one new variant carrying
// the host-side farm-run aggregate. Paper grounding: arXiv:2504.06141
// (Adv-RM); the +5pp / 3-epoch / K>=2 claims in the plan are NOT
// grounded in the paper — recalibrated to +1pp floor (paper §5
// overall +0.7pp / +2.86pp Chat-Hard), 1 epoch (paper §5 explicit
// "single epoch"), and K=1 PPM singleton with proxy>>gold gap as
// uncertainty fallback (D-4 rm-overoptimization style). Live PPM
// farm loop + ppm_finetune.sh + bench_prmbench --hook-mode http
// v1/v2 are RUNPOD-PENDING (host-side scaffolding shipped). See
// workflow/reasoning/D-28-decisions.md.
// D-29 (SCHEMA_VERSION 146): atomic +1 bump on top of D-28's 145 head.
// Adds one new SsbActionType variant `HierarchicalBaiRoundCompleted`
// emitted by the hierarchical BAI dispatch path when ELAI_BAI_MODE=
// hierarchical AND ELAI_BAI_ENABLE=1. Carries the round-aggregate of
// the Metropolis-within-Gibbs hyperposterior update (mu / kappa mean +
// variance), the per-arm shrunk Beta(alpha+mu*kappa, beta+(1-mu)*kappa)
// summaries, the selected arm, the selection basis (HierarchicalTs /
// FlatTs / CanaryOverride), Gibbs iteration count, and the canary gate
// outcome. Same canonical 9-tuple cascade + 10th lockstep at
// action_type_lock.rs (TOTAL_VARIANTS 418 -> 419, ALL_VARIANTS +1,
// exhaustive_match_guard +1) + action_type_lock.golden (+1 row at
// alphabetical position `hierarchical_bai_round_completed`, inserted
// between `held_out_task_used` and `hook_event`; all subsequent
// ordinals shift +1). Plan-doc suggested possibly multiple variants
// (one per family update + one per arm draw); REJECTED per D-19/D-23/
// D-24/D-25/D-28 single-variant precedent — D-29 ships exactly one
// new variant carrying the round-level aggregate. Paper labels:
// arXiv:2111.06929 SUPPORTING EVIDENCE (regret minimization, NOT BAI;
// see workflow/papers/hierarchical-bayesian-bandits-interface.md);
// arXiv:2106.10417 DIRECT GROUNDING (variance-dependent BAI). The
// plan's "30-50% regret reduction" claim is UNGROUNDED for the BAI
// regime and is RUNPOD-PENDING (live flat-vs-hierarchical baseline
// requires a runtime GPU run). See workflow/reasoning/D-29-decisions.md.
// D-30 (SCHEMA_VERSION 147): atomic +1 bump on top of D-29's 146 head.
// Adds one new SsbActionType variant `CalibrationBootstrapCompleted`
// emitted by the calibration bootstrap warm-start path
// (`crate::calibration_bootstrap`) when ELAI_CALIBRATION_BOOTSTRAP_ENABLE
// is set (wire-dark default OFF). Carries the warm-start outcome —
// corpus slug (e.g. "truthfulqa"), corpus size, corpus blake3 hex,
// tau_warm_start, alpha, tau_floor_active, plus the SECONDARY Option
// fields for stationary-bootstrap CI on tau (Politis & Romano 1994
// JASA Vol 89 §3): bootstrap_replicates, bootstrap_ci_low,
// bootstrap_ci_high, block_length_p. The Option fields are `None` for
// the canonical warm-start path and flip to `Some(_)` only when a
// SECONDARY pass enables block-bootstrap (single-variant pattern
// preserved per D-19/D-23/D-24/D-25/D-28/D-29 precedent — extension is
// via Option payload, NOT a second variant; 11th application of
// single-variant pattern-2). Same canonical 9-tuple cascade + 10th
// lockstep at action_type_lock.rs (TOTAL_VARIANTS 419 -> 420,
// ALL_VARIANTS +1, exhaustive_match_guard +1) + action_type_lock.golden
// (+1 row at alphabetical position `calibration_bootstrap_completed`,
// inserted between `bubble_up_typed` and `calibration_corpus_updated`;
// all subsequent ordinals shift +1). Paper labels: arXiv:2509.01455
// UniCR COMPOSITION (we consume compute_tau_with_min); arXiv:2109.07958
// TruthfulQA DATA ORIGIN (corpus fixture source; not algorithm-source);
// Politis & Romano 1994 JASA Vol 89 §3 DIRECT GROUNDING (stationary
// bootstrap mechanism — only active when SECONDARY runs). Live D-22
// validation harness + full-corpus 817-row TruthfulQA import +
// block-bootstrap CI run are RUNPOD-PENDING (host-side scaffolding
// shipped). See workflow/reasoning/D-30-decisions.md.
// E-6 (SCHEMA_VERSION 148): MAST-grounded multi-agent failure-mode defenses
// pack adds 4 new SsbActionType variants atomically (AuditMastTagged +
// IsoBudgetMeasured + PersonaReanchored + CrossFamilyUnavailable). Citation:
// arXiv:2503.13657 (Cemri et al. 2025, MAST). Plan originally cited
// 2406.12624 which was an ID_COLLISION (substituted per
// workflow/reasoning/E-6-decisions.md DECISION-1 + gotcha
// E6_CITATION_2406_12624_ID_COLLISION). SCHEMA_VERSION 148 bump is atomic
// with action_type_lock.rs TOTAL_VARIANTS 420 -> 424 + ALL_VARIANTS
// append + exhaustive_match_guard arms + .golden body regen.
// batch2-merge-collision recovery (SCHEMA_VERSION 161): atomic bump 149 -> 161
// re-applies the 9 missing variants lost when 3 parallel executors in cluster
// s-25120 batch 2 clobbered each other. Bump composes:
//   * E-3 RuntimeHandoffSignal      (149 -> 150)
//   * E-26 SglangBackendRegistered  (150 -> 151)
//   * E-26 XgrammarSchemaCompiled   (composes into 151 floor)
//   * E-26 RuntimeColdStartObserved (composes into 151 floor)
//   * E-2 StreamingTimingRecorded   (151 -> 152)
//   * E-2 UxPacingVoteRecorded      (152 -> 153)
//   * E-28 StreamingLlmEngaged      (153 -> 154)
//   * E-28 SdpaBackendPinned        (154 -> 155)
//   * E-28 YarnRopeScalingApplied   (155 -> 156)
// Plus 5 RUNPOD/decisions reservations (156 -> 161). TOTAL_VARIANTS 425 -> 434
// in action_type_lock.rs lockstep. See workflow/sessions/s-25120/completion-drive/
// audits/batch2-fixer-report.md for the full collision recovery trail.
// E-8 (SCHEMA_VERSION 162): SINGLE-VARIANT Pattern-2 (14th application)
// PromptOptimizationCompleted with OptimizerPhase 7-arm discriminator
// (dspy_init / opro_step / rank_prompt_vote / ape_proposal / gepa_reflect /
// archive_promote / archive_retire). TOTAL_VARIANTS 434 -> 435. Wire-dark
// unless ELAI_PROMPT_OPTIMIZER_ENABLE=1 AND cfg(feature = "darwinian-evolve")
// (re-uses E-7 feature gate per workflow/reasoning/E-8-decisions.md DECISION-3).
// Paper grounding: arXiv:2507.19457 GEPA primary + arXiv:2310.03714 DSPy +
// arXiv:2309.03409 OPRO + arXiv:2211.01910 APE + arXiv:2405.10276 small-LM-as-
// optimizer limitation + arXiv:2504.20355 LPO edit-tag pattern +
// arXiv:2602.00997 ETGPO gotchas injection. REMOVED grounding:
// arXiv:2403.12373 RankPrompt (WRONG_REGIME per paper-triager). Live LLM
// calls RUNPOD-PENDING.
// E-10 (SCHEMA_VERSION 163): SINGLE-VARIANT Pattern-2 (15th application)
// CriticDecisionEvent with CriticTilePhase 9-arm discriminator
// (fsm_ablation / role_ablation / rag_challenger / mistake_book_signal /
// typed_cot_verified / tier_matrix / cloud_default / grc_blockquote /
// prompt_cache_hit_rate). TOTAL_VARIANTS 435 -> 436. Wire-dark unless
// ELAI_CRITIC_OF_DESIGN_ENABLE=1. Paper grounding: arXiv:2502.00674 Self-MoA,
// arXiv:2503.13657 MAST, arXiv:2510.01069 typed-CoT certificates,
// arXiv:2305.01219 ProAttack, arXiv:2401.05949 ICLAttack, arXiv:2307.13702
// Lanham CoT faithfulness, arXiv:2510.04040 FaithCoT-Bench. Live critic
// loops RUNPOD-PENDING. See workflow/reasoning/E-10-decisions.md.
// E-43 (SCHEMA_VERSION 164): SINGLE-VARIANT Pattern-2 (16th application)
// SkillSystemEvent with SkillEventPhase 3-arm discriminator
// (loaded / invoked / failed). TOTAL_VARIANTS 436 -> 437. Wire-dark
// unless ELAI_SKILL_SYSTEM_ENABLE=1. Carries SkillSystemEventPayload
// with skill_name, phase, prompt_hash_prefix (blake3[..8]), outcome,
// ts_ms. Producer: skill_telemetry::log_skill_invocation +
// cli::skills_cmd. Paper-grounded: agentskills.io/specification (SKILL.md
// v1), Anthropic Skills format. ToxicSkills (snyk.io 2026) +
// venturebeat Anthropic scanner bypass motivate the whole-directory
// scan rule in .claude/rules/skill-import-policy.md. Live skill
// invocations RUNPOD-PENDING. See workflow/reasoning/E-43-decisions.md.
// E-9 (SCHEMA_VERSION 165): SINGLE-VARIANT Pattern-2 (17th application)
// BenchmarkRunObserved with BenchPhase 5-arm discriminator
// (loaded / started / item_evaluated / completed / aborted).
// TOTAL_VARIANTS 437 -> 438. Wire-dark unless
// ELAI_EVAL_BENCHMARK_ENABLE=1. Carries BenchmarkRunObservedPayload
// covering 11 external benchmark suites (swe-bench-pro, swe-bench-live,
// livecodebench-v6, tau2-bench, bfcl-v3, gaia-l1-l2, clawsbench-split,
// refinebench, sycophancy-probes, math-floors, deep-planning) via the
// suite_name string field. Live runs RUNPOD-PENDING per
// workflow/reasoning/E-9-decisions.md DECISION-5; host-side scaffolding
// + power-calibration + SSB emission contract ships in this commit.
// Paper grounding: arXiv:2509.16941 SWE-Bench Pro (primary, RAG-indexed),
// arXiv:2505.23419 SWE-bench-Live, arXiv:2511.22173 RefineBench,
// arXiv:2604.05172 ClawsBench, arXiv:2311.12983 GAIA, arXiv:2601.18137
// DeepPlanning, arXiv:2506.07982 tau2. UNRESOLVED + REMOVED per
// DECISION-3: plan's "DS-CP arXiv:2510.05566" + focused.io blog +
// "MetaMate" (paper-ingestor could not resolve).
// E-19 (SCHEMA_VERSION 166): SINGLE-VARIANT Pattern-2 (18th application)
// ReasoningQualityProbe with ProbeKind 6-arm discriminator
// (l2_gate_decision / l4_prm_lite_score / l4_lateral_kept /
// l4_ttc_budget_chosen / l6_verifier_diversity / vividness_probe). TOTAL_VARIANTS
// 438 -> 439. Wire-dark unless ELAI_REASONING_QUALITY_PROBE_ENABLE=1.
// Plan-doc named 11 SSB variants (L2SkippedHighConfidence + ...);
// REJECTED per D-19/D-23/D-24/D-25/D-28/D-29/D-30/E-7/E-8/E-10/E-43/E-9
// single-variant precedent — E-19 ships exactly ONE new variant with
// the 6-arm ProbeKind discriminator inside ReasoningQualityProbePayload.
// I-5 adds the 6th arm VividnessProbe (A/B pristine-context outcome delta).
// Paper grounding: arXiv:2510.08308 First Try Matters (L2 confirmatory
// reflection), arXiv:2502.18581 Self-Certainty (logit-margin gate),
// arXiv:2510.01500 LToT (mainline/lateral split), arXiv:2503.04412
// AB-MCTS (paper assumes external feedback; PRM-lite documented
// deviation), arXiv:2502.08788 Vanilla MAD (model heterogeneity for L6
// diversity), arXiv:2410.21333 Mind-Your-Step (no-CoT regime),
// arXiv:2510.01069 Typed CoT (wiring-quality gain on ELAI stack),
// arXiv:2502.10954 + arXiv:2408.03314 (difficulty-conditional TTC).
// DROPPED per E-19 grounding-triage: arXiv:2602.12113 ARLCP (training-
// time RL, cannot ground inference-time token cap) and arXiv:2307.13702
// Lanham (CoT-text faithfulness, NOT logit signal quality). See
// workflow/reasoning/E-19-decisions.md.
// E-54 (SCHEMA_VERSION 167): SINGLE-VARIANT Pattern-2 (19th application)
// EvalCorpusEvent with CorpusEventPhase 14-arm discriminator
// (ingest_rejected_ngram / ingest_rejected_cosine / shard_created /
// schema_violation_no_tests / test_fails_on_reference /
// test_useless_passes_broken / kor_orthogonality_violated /
// kor_orthogonality_passed / sycophancy_framing_gap_detected /
// sandbagging_anomaly / generator_rotation_violated /
// perf_delta_contamination_suspected / dynamic_knowledge_qa_generated /
// prod_traffic_probe_injected). TOTAL_VARIANTS 439 -> 440. Wire-dark
// unless ELAI_EVAL_CORPUS_ENABLE=1. Repairs D-6 PLANNED 9-gram filter
// pre-deploy with 13-gram + embedding-cosine + per-family directory
// shards + test-triplet schema + matched-pair sycophancy + KOR-orthogonality.
// CanaryFamily (5-arm) is the canonical task family surface; KnowledgeQa
// is routed via ToolUse-flavored tasks. Paper grounding (post-triage):
// arXiv:2311.04850 (PRIMARY -- 13-gram threshold), arXiv:2602.12413 (re-
// triaged CENTRAL in E-54 domain; E-7 ZERO_RELEVANCE was darwinian-domain-
// specific), arXiv:2403.07974 (cutoffs schema), arXiv:2410.06526 KOR-Bench
// (cross-family transfer; ELAI operationalization), arXiv:2503.02951
// KodCode (test-triplet schema), arXiv:2411.04905 OpenCoder
// (test-execution filter), arXiv:2510.20487 (matched-pair framing concept;
// the Sonnet 4.5 >80% verbalization stat is Anthropic, not the paper),
// arXiv:2508.00943 Covert Sandbagging, arXiv:2510.04721 BrokenMath
// (perturbed_pair), arXiv:2412.08905 Phi-4 (generator rotation),
// arXiv:2509.00072 Test-of-Time, arXiv:2511.08598 OKBench (dynamic
// knowledge-QA), arXiv:2503.14499 METR Time Horizons,
// arXiv:2511.17602 (F1=0.76 anchor; 0.85 cosine default is ELAI-empirical
// INITIAL DEFAULT per Open Q1, NOT paper-derived per FIX-2),
// arXiv:2410.12784 JudgeBench (replaces plan's WRONG_ID 2412.12509 per
// FIX-1). FIX-3: LiveCodeBench uses release-date stamps, NOT
// monthly-Docker-per-task (that's SWE-bench-Live arXiv:2505.23419).
// FIX-4: Sonnet 4.5 >80% verbalization decoupled from arXiv:2510.20487
// (Anthropic internal source). Live ingest + LoRA cross-family transfer +
// sandbox test_executor + sidecar embeddings + LLM-grader matched-pair
// invocation RUNPOD-PENDING. See workflow/reasoning/E-54-decisions.md.
// E-55 (SCHEMA_VERSION 168): SINGLE-VARIANT Pattern-2 (20th application)
// RewardLadderEvent with RewardLadderEventKind 9-arm discriminator
// (rlaif_judge_rated / rlaif_calibration_correlation_low /
// gepa_template_evolved / gepa_template_evicted / reward_fusion_computed /
// reward_hacking_suspected / sft_training_threshold_reached /
// bai_arm_observation_recorded / rl_weight_update_unlocked).
// TOTAL_VARIANTS 440 -> 441. Wire-dark unless ELAI_AGENTIC_RL_ENABLE=1.
// Deferred RL ladder: RLAIF judge (cloud Sonnet on local-Qwen rollouts) +
// GEPA reflective prompt evolution + hybrid PRM+ORM reward fusion + MONA
// composite-reward hacking guards + SFT-on-rollouts collection surface +
// 3-condition unlock gate for DPO/PPO/GRPO/KTO weight-update modules
// (which DO NOT exist by design until unlock). Paper grounding (post-
// triage): arXiv:2212.08073 Constitutional AI (Anthropic) + arXiv:2309.00267
// RLAIF vs RLHF + arXiv:2507.19457 GEPA (ICLR 2026 oral) + arXiv:2501.13011
// MONA + arXiv:2509.15557 Verifiable Composite Rewards + arXiv:2507.01352
// Skywork-Reward-V2 (outcome-dominant 0.4/0.6 default) + arXiv:2412.21139
// SWE-Gym (200-rollout SFT lower bound) + arXiv:2508.16546 RL vs SFT
// panacea-or-mirage + arXiv:2501.17161 SFT memorizes RL generalizes +
// arXiv:2410.12784 JudgeBench (calibration; replaces plan's WRONG_ID
// 2412.12509 per E-55 triage FIX) + Nature 2025 AlphaProof URL
// (replaces plan's FATAL bad arXiv:2406.15829 per E-55 triage). DPO is
// HARD-AVOIDED forever for ELAI data shape (Apple 2025 OOD drop +
// arXiv:2502.20847 gradient imbalance + no paired preference data).
// PPO/GRPO/DAPO/KTO HARD-DEFERRED until 3 unlock conditions all TRUE
// (BAI saturation + ORM coverage >=80% + VRAM headroom >=4GB). Live
// cloud Sonnet rollouts + LoRA training RUNPOD-PENDING. See
// workflow/reasoning/E-55-decisions.md.
// E-11 (SCHEMA_VERSION 169): SINGLE-VARIANT Pattern-2 (21st application)
// SaasPriorArtEvent with PriorArtEventKind 2-arm discriminator covering
// archetype_recommended / prior_art_consulted. Wire-dark unless
// ELAI_SAAS_PRIOR_ART_ENABLE=1. TOTAL_VARIANTS 441 -> 442. The catalog +
// recommender slice ships the in-process surface
// (crate::architecture::recommend_archetype + ArchetypeCatalog) plus
// the SSB telemetry row for downstream consumers (idea-critic +
// production-grade auditor + dashboard). 15 prior-art entries (9 tools
// + 6 reports) ground each row's emission. Sibling slices not in this
// commit: scope_clarity_score on CriticFinding extension, package_
// verifier with sqlite cache, anti-spiral hard-abort EscalationTrigger
// extension (at autonomous_gate.rs:400+, NOT anti_spiral_extended.rs),
// edit_format Unified/SearchReplace/FullFile downgrade chain. Grounding:
// 2024-2026 idea-to-app survey at workflow/improvement-sweep-2026-05-19/
// reports/11-idea-to-saas-prior-art.md (Veracode 2025 45% / Tenzai CSA
// Dec 2025 100% / Escape.tech 2025 1,400 apps / Help Net 2025
// slopsquatting 19.7% / Answer.AI Devin Jan 2025 3/20 / Cognition Devin
// 2025 review). See workflow/reasoning/E-11-decisions.md.
// E-22 (SCHEMA_VERSION 170): SINGLE-VARIANT Pattern-2 (22nd application)
// GithubTrendingEvent with TrendingEventKind 3-arm discriminator covering
// skill_export_normalized / microvm_fallback / lazy_schema_savings.
// Wire-dark unless ELAI_GITHUB_TRENDING_ENABLE=1. TOTAL_VARIANTS 442 -> 443.
// The slice formalises the may-2026 trending sweep adoptions: cross-vendor
// SKILL.md export (ABSORBED into the E-43 skills_cmd publish handler with a
// new `export` action that adds frontmatter normalization + blake3 drift),
// microsandbox/libkrun microVM sandbox backend (4th AgentSandbox impl in
// orchestrator::sandbox; linux+KVM only; degrades to Firecracker / Gvisor /
// AppContainer with one telemetry row), and a consumer-side env gate on top
// of C-35 LazySchemaCache that forces the tools/list lazy shape and emits
// before/after byte counts. Streamable HTTP transport (plan task 4) already
// shipped E-24 (mcp_bridge.rs:216-225); no double-emit. Grounding: Anthropic
// SKILL.md spec (code.claude.com/docs/en/skills), microsandbox README
// (github.com/superradcompany/microsandbox, YC X26 + libkrun), and arXiv:
// 2604.21816 "Tool Attention Is All You Need" (95% MCP-tax reduction on a
// 120-tool 6-server benchmark; VERIFIED 2026-05-27 per grounding triage).
// See workflow/reasoning/E-22-decisions.md.
// E-23 (SCHEMA_VERSION 171): SINGLE-VARIANT Pattern-2 (23rd application)
// DeveloperDiscourseEvent with DiscourseEventKind 3-arm discriminator covering
// trust_telemetry_aggregated / runtime_choice_badge_emitted /
// context_state_warning. Wire-dark unless ELAI_DEVELOPER_DISCOURSE_ENABLE=1.
// TOTAL_VARIANTS 443 -> 444. The slice surfaces the positioning + UX-guard
// substrate that translates the 2026 developer-discourse survey into
// measurable productivity / trust telemetry plus context-state visibility
// at the CommandWindow + dashboard panels. trust_telemetry_aggregated rolls
// up tasks_per_hour + findings_per_task + revert_rate + ungrounded_count +
// stagnation_dist from existing SSB rows (no new producer required).
// runtime_choice_badge_emitted records every CommandWindow status-row
// transition between local / cloud-billed labels, derived from B-37
// runtime_fallback rows. context_state_warning records the 70% / 85%
// token-budget threshold crossings the ContextStatePanel renders.
// Grounding: arXiv:2507.09089 METR RCT (19% slowdown with felt-20%-speedup
// gap on 16 senior OSS devs, 246 tasks, Feb-Jun 2025); Stack Overflow 2025
// developer survey (84% AI use, 29% trust); JetBrains AI Pulse April 2026
// (90% regular use, Claude Code 91% CSAT / NPS 54); Anthropic issue #13171
// (context loss UX). See workflow/reasoning/E-23-decisions.md.
// E-25 (SCHEMA_VERSION 172): SINGLE-VARIANT Pattern-2 (24th application)
// ClaudeCodeArchitectureEvent with ClaudeCodeArchitectureEventKind 9-arm
// discriminator covering compaction_ladder_tier_applied /
// skill_template_executed / prompt_decision_made /
// opusplan_overlay_activated / session_forked /
// edit_checkpoint_snapshotted / bash_complexity_blocked /
// approval_fatigue_detected / sandbox_template_warned. Wire-dark unless
// ELAI_CLAUDE_CODE_ARCH_ENABLE=1. TOTAL_VARIANTS 444 -> 445. The slice
// surfaces the architecture-reference CATALOG of the 9 high-leverage
// Claude Code harness primitives ELAI ports (CompactionLadder /
// SkillTemplating / PromptHandler / OpusplanOverlay / SessionFork /
// EditCheckpoints / BashComplexityGuard / ApprovalFatigueGuard /
// SandboxTemplateGuard) plus the renamed ClaudeCodeLadderResult struct
// that resolves the hard name-collision with crate::compact::
// CompactionResult. Concrete implementation modules (compaction_ladder.rs,
// cli/fork.rs, edit_checkpoints.rs, prompt_handler.rs, dashboard panels)
// are FOLLOW-UPS tracked in workflow/reasoning/E-25-decisions.md;
// Primitive 2 (deferred MCP descriptions) is DESCOPED pending E-22 Task 3
// / C-35 lazy_schemas substrate. Grounding: karanprasad.com/blog/how-
// claude-code-actually-works-reverse-engineering-512k-lines (6-tier
// ladder + 98.4% deterministic infra split); code.claude.com/docs/en/
// how-claude-code-works (compaction + checkpoints + fork-session);
// code.claude.com/docs/en/skills (!`cmd` shell injection + live reload);
// code.claude.com/docs/en/hooks (prompt-type LLM-decision handler);
// code.claude.com/docs/en/model-config (opusplan alias); github.com/
// VILA-Lab/Dive-into-Claude-Code (anti-pattern guards); infoq.com/news/
// 2026/05/anthropic-claude-code-postmortem (telemetry-guard rationale).
// See workflow/reasoning/E-25-decisions.md.
// E-29 Tier-B (SCHEMA_VERSION 173): SINGLE-VARIANT Pattern-2 (25th
// application) AgenticObservabilityEvent with
// AgenticObservabilityEventKind 4-arm discriminator covering
// mast_failure_classified / replay_resume / signal_match /
// eval_dataset_member. Wire-dark unless
// ELAI_AGENTIC_OBSERVABILITY_ENABLE=1. TOTAL_VARIANTS 445 -> 446.
//
// The slice lands the Tier-B continuation of the E-29 plan: MAST
// classifier wired ON TOP of E-6 mast.rs (DO NOT redefine MastMode);
// `ssb_resume_from` replay validator that FAIL-CLOSED consumes C-44
// / D-9 locked_keys_auditor frozen-surface predicate; signal_match
// backfill primitive (Laminar-style natural-language outcome
// extraction); eval_dataset_member promotion (Langfuse / LangSmith
// pattern). The optional `ssb-otel-export` crate ships behind the
// `otlp-export` feature flag (OPT-IN; 4 OTel deps in
// workspace.dependencies).
//
// Tier-A (3 drawers TranscriptDrawer / SpendDrawer / QueryDrawer)
// already shipped via s-orch-e29 over EXISTING Tauri commands.
// Tier-B in THIS commit lands the FailureModesDrawer + 4 new Tauri
// commands (ssb_transcript / ssb_resume_from /
// ssb_promote_to_dataset / ssb_backfill_signals).
//
// Grounding: arXiv:2503.13657 (MAST), laminar.sh, langfuse.com,
// opentelemetry.io/docs/specs/semconv/gen-ai (1.40.0 schema still
// Development per May 2026), traceloop.com/docs/openllmetry
// (TRACELOOP_TRACE_CONTENT pattern), tianpan.co (deterministic-replay
// motivation). See workflow/reasoning/E-29-decisions.md.
// E-31 (SCHEMA_VERSION 174): coding-model-leaderboard scraper substrate.
// SINGLE-VARIANT Pattern-2 (26th application). LeaderboardEventKind 3-arm
// discriminator covers scrape_started / scrape_completed / scrape_failed.
// Wire-dark unless ELAI_CODING_MODEL_LEADERBOARD_ENABLE=1. Scraper at
// benches/external/coding_model_leaderboard.py emits stdout JSON rows
// parsed by the rust dispatcher. Leaderboard targets:
// bigcode-bench.github.io (BigCodeBench), evalplus.github.io/leaderboard
// (HumanEval+ / MBPP+), labs.scale.com/leaderboard/swe_bench_pro_public
// (SWE-Bench Pro contamination-resistant), livecodebench.github.io
// (LiveCodeBench contamination-free time-windowed). Grounding:
// arXiv:2409.12186 (Qwen2.5-Coder tech report -- head_dim 128 invariant
// for ELAI's local lane), arXiv:2509.16941 (SWE-Bench Pro paper),
// morphllm.com/swe-bench-pro (Pro vs Verified gap), platform.claude.com
// /docs/en/about-claude/pricing (Anthropic 2026 prices). Live scrape
// RUNPOD-PENDING -- ENV-dark default keeps existing tests stable.
// Companion non-bumped surface (E-31.A SHIPPED): contracts/tests/
// head_dim_invariant_contract.rs guards the dense-128 invariant at the
// model-config layer. Sidecar boot-time assert + EscalationPolicy
// (DRIFT-1 pick_runtime + DRIFT-2 escalation_policy.rs name collision)
// DEFERRED to E-31.B per workflow/reasoning/E-31-decisions.md.
// E-38 (SCHEMA_VERSION 175): 2026 AI roadmap signals + EU AI Act
// compliance pre-gate substrate. SINGLE-VARIANT Pattern-2 (27th
// application). AiRoadmapSignalEvent with RoadmapSignalKind 5-arm
// discriminator: ComplianceBlocked / ComplianceVerified /
// McpTransportVersionLogged / MrcrUnverified / PostureDocStale. Wire-dark
// unless ELAI_AI_ROADMAP_SIGNALS_ENABLE=1. TOTAL_VARIANTS 447 -> 448.
//
// Producers:
//   - crates/orchestrator/src/compliance.rs::pre_gate_planning_to_testing_first
//     emits ComplianceBlocked / ComplianceVerified at the
//     PLANNING -> TESTING_FIRST + FEASIBILITY_PROBE -> TESTING_FIRST edge.
//   - crates/plugins/src/mcp_bridge.rs::DefaultMcpBridge::new
//     emits McpTransportVersionLogged at construction (idempotent
//     per-process boot).
//   - crates/orchestrator/src/research/roadmap_signals.rs emits
//     MrcrUnverified at backend init when [runtime] declared_max_context_tokens
//     >= 100_000 lacks a measured mrcr_v2_score, and PostureDocStale at
//     session start when docs/strategy/ELAI-2026-POSTURE.md mtime > 90 days.
//
// Grounding (every cite WebSearch-verified May 2026):
//   - arXiv:2409.12640 -- MRCR v2 multi-needle long-context benchmark
//     (verified arxiv.org/abs/2409.12640).
//   - arXiv:2510.12357 -- MoBiLE consumer-GPU MoE inference (verified
//     arxiv.org/abs/2510.12357; Oct 2025).
//   - arXiv:2602.03495 -- DALI workload-aware MoE offloading framework
//     (verified arxiv.org/abs/2602.03495; Feb 2026).
//   - artificialintelligenceact.eu/implementation-timeline -- GPAI live
//     Aug 2 2025; transparency Aug 2 2026; high-risk Annex III POSTPONED
//     to Dec 2 2027 per Digital Omnibus May 7 2026.
//   - tridao.me/publications/flash3/flash3.pdf -- FA-3 Hopper-only, FA-4
//     Hopper+Blackwell CuTeDSL; Ampere indefinitely on FA-2 / SDPA.
//   - modelcontextprotocol.io/development/roadmap -- stateless transport,
//     Tasks primitive, MCP Registry Q4 2026, SAML/OIDC/PKCE Q2 2026.
// See workflow/reasoning/E-38-decisions.md.
// E-39 (SCHEMA_VERSION 176) -- cost + token economics substrate.
// SINGLE-VARIANT Pattern-2 (28th application). CostTokenEvent with
// CostEventKind 8-arm discriminator covers task_aborted_by_spend_cap /
// velocity_cap_breached / automix_verifier_verdict /
// cost_router_decision / cost_estimate_inaccurate /
// free_tier_quota_approaching / free_tier_quota_exceeded /
// layered_cap_manual_override. Wire-dark unless
// ELAI_COST_TOKEN_ECONOMICS_ENABLE=1. Producers: orchestrator::cost
// modules (velocity_cap / automix_gate / cost_router / free_tier_guard
// / cloud_abort) + orchestrator::cloud_mode mid-task abort
// integration point + CLI `elai cloud override-cap` confirmation
// emit. Source grounding (verified May 2026):
//   - arXiv:2310.12963 -- AutoMix self-verify cascade (verified
//     arxiv.org/abs/2310.12963; ICLR 2025; >50% cost reduction at
//     iso-quality across 5 datasets x 5 LMs).
//   - arXiv:2604.22750 -- "How Do AI Agents Spend Your Money?" Stanford
//     Digital Economy Lab (verified arxiv.org/abs/2604.22750 +
//     alphaxiv.org/overview/2604.22750; 30x same-task variance; 1000x
//     tokens vs chat; input-driven cost).
//   - arXiv:2305.05176 -- FrugalGPT cascade (verified arxiv.org/abs/
//     2305.05176; 98% cost reduction at iso-accuracy across the
//     12-LLM cascade).
//   - platform.claude.com/docs/en/about-claude/pricing (Sonnet 4.6
//     $3/$15, cache-write 1.25x, cache-read 0.10x, 5-min TTL).
//   - api-docs.deepseek.com/quick_start/pricing (DeepSeek V4-Flash
//     $0.30/$0.50 + 90% cache discount).
//   - ravoid.com/blog/ai-agent-budget-enforcement ($47K LangChain
//     4-agent loop incident -- the wallet-loop class this substrate
//     defends against).
// See workflow/reasoning/E-39-decisions.md.
// E-50 (SCHEMA_VERSION 177) -- SaaS architecture pattern catalog
// substrate. SINGLE-VARIANT Pattern-2 (29th application).
// SaasArchPatternEvent with SaasArchPatternEventKind N-arm discriminator
// covers production_readiness_gate_blocked /
// production_readiness_gate_passed / rls_baby_step_emitted /
// unscoped_token_blocked / billing_mor_decision_made /
// realtime_block_decided / async_backbone_decided /
// per_tier_rate_limit_derived / otel_slo_runbook_scaffolded /
// platform_unsupported_for_scope_check / archetype_block_overridden.
// Wire-dark unless ELAI_SAAS_ARCH_PATTERNS_ENABLE=1. Producer:
// orchestrator::architecture::saas_patterns. Composes with the existing
// E-11 catalog / recommender / prior_art surface (does NOT fork the
// 5-archetype SaasArchetype taxonomy). Source grounding (verified
// May 2026): hunchbite.com multi-tenant SaaS architecture (RLS-default
// + hybrid promotion), thenile.dev/blog/multi-tenant-rls (forgot
// tenant_id dominant leak), makerkit.dev RLS best-practices (auth.jwt
// ->> 'org_id'), accuknox.com OWASP API 2026 (rate-limit + broken-auth
// + BOLA on /admin top 3), dev.to/alessandro_pignati/9-second-disaster
// (PocketOS Apr 2026 Railway unscoped-token DB wipe),
// securityboulevard.com 2026 secret sprawl (28.65M secrets),
// sherlockforensics.com AI-code 2026 (92% critical CVE),
// oneuptime.com/blog/burn-rate-alerts-opentelemetry (crit 14x / warn
// 6x / notice 1x), fintechspecs.com 2026 (Polar 4% + OSS MoR),
// liveblocks.io/docs/yjs (managed Yjs CRDT),
// latestfromtechguy.com PartyKit 2026, pkgpulse.com Hatchet vs
// Trigger.dev vs Inngest 2026. See workflow/reasoning/E-50-decisions.md.
//
// E-4 (SCHEMA_VERSION 178) -- chat UX critique substrate (sycophancy
// guardrails + per-claim confidence chips + pinned-rules panel +
// session re-anchor + doom-loop detector + completion-claim evidence
// gate + one-shot clarifier + refusal-with-override + mid-stream
// interrupt + honest-correction surface). SINGLE-VARIANT Pattern-2
// (30th application) collapsing the 13 fire-points the plan calls
// out into ChatUxCritiqueEvent + ChatUxCritiqueKind 13-arm
// discriminator. Exceeds the prior 11-arm empirical max (E-50) by 2
// arms; documented in workflow/reasoning/E-4-decisions.md as a
// deliberate single-envelope choice that keeps action_type_lock.golden
// to ONE ordinal insertion + ONE SCHEMA bump. Wire-dark unless
// ELAI_CHAT_UX_CRITIQUE_ENABLE=1. Producer:
// orchestrator/src/chat_ux/* (substrate module). Paper grounding (May
// 2026; arXiv IDs verified inline per session correction):
//   - arXiv:2310.13548 Sharma "Towards Understanding Sycophancy"
//     (verified existing IDs cluster -- Sharma sycophancy paper)
//   - openai.com/index/sycophancy-in-gpt-4o (April 2025 GPT-4o
//     rollback; canonical industry sycophancy postmortem)
//   - aclanthology.org/2024.tacl-1.9 "Lost in the Middle" Liu et al.
//   - arXiv:2412.00804 "Examining Identity Drift in LLM Agents"
//     (8-15 turn persona-drift; verified)
//   - arXiv:2507.21285 "Curiosity by Design" (one-shot clarifier
//     grounding; verified)
//   - arXiv:2509.08833 "Pitfalls of Over-Alignment" (over-refusal
//     harm grounding; verified)
//   - HN 47936579 (Claude Code false-completion thread; canonical
//     industry false-completion postmortem)
//   - warpedvisions.org/blog/2025/getting-unstuck-when-using-llms-
//     for-coding (doom-loop UX postmortem)
//   - anthropic.com/constitution (diplomatic honesty / honest-
//     correction grounding)
// 2604.* arXiv IDs cited in plan body (2604.19245, 2604.22417) are
// tagged "supporting-only" pending bulk WebSearch verification; see
// workflow/reasoning/E-4-decisions.md "Citation verification" row.
// Plan-cited file:line refs (task_runner.rs:3996/4645/7776 and
// chat_reflect.rs:635) are STALE -- this commit anchors at actual
// sites (task_runner.rs:11379 def + 6438 call; chat_reflect.rs:
// 379-393 AUDITOR refine loop). See workflow/reasoning/E-4-
// decisions.md "Stale line refs" row. TOTAL_VARIANTS 450 -> 451.
//
// E-16 (SCHEMA_VERSION 179) -- VLM image validation hardening.
// SINGLE-VARIANT Pattern-2 (31st application). VlmImageValidationEvent
// with VlmImageValidationKind 3-arm discriminator covers
// swap_order_disagreement / axtree_mismatch /
// vlm_schema_unconstrained. SycophancyFlag (already shipped via E-4
// at SCHEMA 124-178; ssb.rs:5450) is REUSED for the sycophancy_probe
// inverted-framing path; no new variant for that fire-point. Wire-dark
// unless ELAI_VLM_IMAGE_VALIDATION_ENABLE=1. Producer:
// orchestrator/src/media_validator/{vlm_validator,axtree,sycophancy}.
// Paper grounding (May 2026; arXiv IDs verified inline):
//   - arXiv:2408.11261 "Sycophancy in Vision-Language Models"
//     (Aug 2024; verified arxiv.org/abs/2408.11261; LQCD leading-
//     query contrastive decoding mitigation)
//   - arXiv:2406.07791 "Judging the Judges: A Systematic Study of
//     Position Bias in LLM-as-a-Judge" (Jun 2024; verified
//     arxiv.org/abs/2406.07791; 15 judges x 22 tasks x ~40 models
//     >150K instances; >10pp pairwise position-bias swing)
//   - playwright.dev/docs/aria-snapshots -- page.ariaSnapshot()
//     YAML AXTree (v1.59+; vendor doc, no arXiv)
//   - arXiv:2411.15100 XGrammar JSON-schema mask-emission
//     (referenced via plan E-16 line 165; supporting-only)
// 2604.05839 (cited in plan body as RCE-convergence grounding) NOT
// VERIFIED -- WebSearch returned no hit; tagged supporting-only,
// design grounded by cited Anthropic + Playwright pages only.
// TOTAL_VARIANTS 451 -> 452.
//
// E-21 (SCHEMA_VERSION 181): bump 180 -> 181 for PeerSystemsEvent
// SINGLE-VARIANT Pattern-2 (33rd application). PeerSystemsEventKind 4-arm
// discriminator covers critic_candidates_generated / critic_score_computed
// / critic_pick_committed / critic_rerank_skipped. Wire-dark unless
// ELAI_PEER_SYSTEMS_ENABLE=1. Producer: orchestrator/src/peer_systems/
// (critic_rerank + sbfl + repo_state_store + aci_audit submodules; SBFL
// + repo-state-store + ACI-audit are pure-compute today, critic-rerank
// production wire-on is a Tier-C followup). Paper grounding:
//   * arXiv:2404.05427 AutoCodeRover (Ochiai formula for SBFL substrate)
//   * arXiv:2405.15793 SWE-agent ACI (NeurIPS 2024)
//   * openhands.dev Nov 2025 blog "SOTA on SWE-Bench-Verified via
//     inference-time scaling + critic-model" (rerank method)
//   * aider.chat/docs/repomap.html (tree-sitter + PageRank repo-map --
//     PageRank substrate already shipped under C-3/C-38; E-21 reuses
//     SsbActionType::PagerankRun, no duplicate variant added)
// See workflow/reasoning/E-21-decisions.md (Tier-A + Tier-B addendum).
// TOTAL_VARIANTS 453 -> 454.
//
// E-20 (SCHEMA_VERSION 180): bump 179 -> 180 for LocalCloudRoutingEvent
// SINGLE-VARIANT Pattern-2 (32nd application). RoutingEventKind 9-arm
// discriminator covers confidence_emitted / hybrid_escalation /
// difficulty_classified / privacy_blocked / output_filtered /
// burn_alert / reasoning_mode / majority_vote / role_priority_chosen.
// Wire-dark unless ELAI_LOCAL_CLOUD_ROUTING_ENABLE=1. Producer:
// orchestrator/src/cost/cloud_route.rs (sibling to E-39 automix_gate +
// cost_router + cloud_abort under cost/ subtree, NOT
// orchestrator/src/routing/ which is owned by C-46 Uno-Orchestra and
// NOT crates/orchestrator/src/cloud_spend.rs which was REJECTED per
// E39_CLOUD_SPEND_PATH_DRIFT). PrivacyBlock fire-point composes with
// pre-existing RouteBlocked variant (C-5 ordinal 331) via a new
// reason="privacy_block" value extension on RouteBlockedPayload; no
// enum cascade for that fire-point alone. Paper grounding (verified
// May 2026 via WebSearch):
//   * arXiv:2511.07396 C3PO Nov 2025 conformal-cost-bounded cascade
//   * arXiv:2406.18665 RouteLLM LMSYS 2x cost reduction (Chatbot Arena)
//   * arXiv:2310.12963 AutoMix >50% cost reduction via self-verify
//   * arXiv:2510.01237 Confidence-Aware Routing (F1 0.61 -> 0.82, 40%
//     compute cut; probe + perplexity dominate verbalized-self-confidence)
//   * arXiv:2510.08731 When to Reason IBM (vLLM semantic router; -47.1%
//     latency / -48.5% tokens / +10.2pp MMLU-Pro)
//   * arXiv:2502.00674 Self-MoA aggregating single-best-LLM (+6.6%
//     AlpacaEval / +3.8% avg) -- documents anti-pattern of mixing
//     different LLMs (weakest-member drag)
//   * arXiv:2511.03808 Prompt Difficulty Prediction (intermediate-
//     representation probes from s1.1-32B)
//   * arXiv:2305.05176 FrugalGPT cascade (up to 98% cost cut at GPT-4
//     parity)
// Supporting-only (not used as load-bearing anchor):
//   * arXiv:2510.08146 entropy-of-first-token (cited in plan task 1
//     alongside confidence-aware routing)
//   * arXiv:2510.00202 RouterArena, arXiv:2510.08439 xRouter,
//     arXiv:2504.01282 Prompt-Reverse Inconsistency,
//     arXiv:2406.12334 same-prompt different-result
// See workflow/reasoning/E-20-decisions.md.
// TOTAL_VARIANTS 452 -> 453.
//
// E-36 (SCHEMA_VERSION 182): bump 181 -> 182 for DocumentVisionRagEvent
// SINGLE-VARIANT Pattern-2 (34th application). DocumentVisionRagEventKind
// 5-arm discriminator covers vision_chunk_emitted /
// vision_fallback_triggered / vision_rag_policy_decision /
// visual_artifact_check_verdict / doc_parser_cloud_fallback_blocked_by_mode.
// Wire-dark unless ELAI_DOCUMENT_VISION_RAG_ENABLE=1. Producer:
// orchestrator/src/document_vision_rag/{visual_verifier, policy, mod}.rs
// (10th VerificationMethod variant lives at reasoning/src/types.rs::
// VerificationMethod::VisualArtifactCheck; producer surface is the
// ImageArtifactChunker + PdfStructuredChunker in knowledge::chunking).
// Paper grounding (verified May 2026 via WebSearch):
//   * arXiv:2510.14528 PaddleOCR-VL 0.9B (OmniDocBench v1.5 92.56)
//   * arXiv:2505.14059 Dolphin-v2 ACL 2025 (handwriting + photographed)
//   * arXiv:2407.01449 ColPali ICLR 2025 (late-interaction page images)
//   * arXiv:2602.12510 Visual RAG Toolkit (training-free pooling;
//     ~10x storage reduction; verified Feb 2026 publication)
//   * arXiv:2502.12342 REAL-MM-RAG ACL 2025 IBM (table-query brittleness;
//     drives the 0.6 default vision_vec RRF weight vs 0.8 text_vec)
//   * arXiv:2411.04952 M3DocRAG Bloomberg (multi-page VQA SOTA)
//   * huggingface.co/ModernVBERT/colmodernvbert (open-weight retriever)
//   * huggingface.co/blog/manu/vidore-v2 (ColQwen2.5-v0.2 0.597 nDCG@5)
// Plan: docs/thefinalglance/E-plan/E-36_document_vision_rag.md.
// Decisions: workflow/reasoning/E-36-decisions.md (Tier-A + Tier-B).
// TOTAL_VARIANTS 454 -> 455.
//
// E-42 (SCHEMA_VERSION 183): bump 182 -> 183 for TreeSitterGrammarEvent
// SINGLE-VARIANT Pattern-2 (35th application). TreeSitterGrammarEventKind
// 4-arm discriminator covers abi_probe_mismatch / svelte_runes_parse_health /
// tier_b_grammar_loaded / tree_sitter_skipped_large_file. Wire-dark unless
// ELAI_TREE_SITTER_GRAMMARS_ENABLE=1. Producers:
// knowledge::tree_sitter_health::{abi_probe, svelte_health, large_file, registration}
// + DkgEngine init path. Collapses the plan-demanded 4 sibling variants
// (DRIFT-6) into one variant per Pattern-2 precedent (E-20 9-arm +
// E-21 4-arm + E-36 5-arm). The shipped composite-reparse at
// knowledge::dkg::VueGrammar::vue_extract_script_recursive is preserved;
// no parallel injections.scm pathway is introduced (DRIFT-10). See
// workflow/reasoning/E-42-decisions.md.
// TOTAL_VARIANTS 455 -> 456.
//
// E-46 (SCHEMA_VERSION 184): bump 183 -> 184 for PlanningDecompositionEvent
// SINGLE-VARIANT Pattern-2 (36th application). PlanningDecompositionEventKind
// 8-arm discriminator covers reflection_appended /
// low_entropy_decomposition / goal_drift_canary_below_threshold /
// bubble_up_replan_phase / bubble_up_ledger_reset /
// agentless_shortcut_dispatched / process_reward_score_emitted /
// skill_prior_retrieved. Wire-dark unless
// ELAI_PLANNING_DECOMPOSITION_ENABLE=1. Producers:
// orchestrator/src/planning_decomposition/producers.rs. Closes the
// 19/27 MISSING prior-re-audit cycle on the E-46 planning-decomposition
// substrate by shipping (a) the 5 absent modules
// (reflection_buffer, decomposition_entropy, skill_prior, audit_score,
// goal_drift_ext, dual_ledger_ext) layered additively atop the shipped
// dual_ledger / agentless_shortcut / goal_drift_canary trio, (b) the
// single Pattern-2 SSB variant + 8 Kind arms in place of the plan's
// demanded 8 sibling variants (saves 7 SCHEMA slots + 7 ALL_VARIANTS
// rows + 7 alphabetical inserts per recent cluster precedent), and (c)
// a stub_ban-enforced payload validation surface on the producers
// (failed_plan / error_transcript / original_goal MUST be non-empty for
// bubble_up_replan_phase; concept_or_phase_id MUST be non-empty for
// bubble_up_ledger_reset; goal-drift report carries BOTH jaccard +
// cosine arms by struct shape).
//
// Paper grounding (verified May 2026 via WebSearch):
//   * arXiv:2411.04468 Magentic-One — TaskLedger + ProgressLedger
//     dual-ledger orchestrator (Task 1 sidecar Magentic shape).
//   * arXiv:2407.01489 AGENTLESS — 32.00% SWE-bench Lite at $0.70/issue
//     (Task 2 leaf shortcut already shipped in
//     crates/orchestrator/src/agentless_shortcut.rs s-62376; E-46 wires
//     the dispatched-row SSB fire-point through this enum).
//   * arXiv:2303.11366 Reflexion — verbal-RL episodic reflection buffer
//     (Task 3 reflection_buffer module).
//   * arXiv:2505.02709 (AIES 2025) Goal-Drift Evaluation — PRM
//     supervision; cosine on goal embedding canonical (Task 4
//     goal_drift_ext cosine arm; term-frequency cosine substitute over
//     blake3 buckets because sidecar /embed endpoint absent).
//   * arXiv:2503.13657 Why Multi-Agent LLM Systems Fail — "improper
//     task decomposition (too granular, too broad, not serializable)"
//     top failure mode (Task 5 decomposition_entropy stop-rule).
//   * arXiv:2310.04406 LATS — value-function as auditor at every fork
//     (Task 6 audit_score deterministic process-reward scalar).
//   * arXiv:2305.16291 Voyager — skill library + automatic curriculum
//     (Task 8 skill_prior top-K retrieval).
//   * arXiv:2311.05772 ADaPT — As-Needed Decomposition (recursive
//     intervention only when LLM cannot execute) — entropy-stop +
//     AGENTLESS shortcut are the ELAI equivalent of ADaPT's conditional-
//     decomposition primitive (cross-reference for Task 5 + Task 2).
//   * arXiv:2305.04091 Plan-and-Solve — ACL 2023 prompting; the
//     dual_ledger_ext PlanStep + LedgerFact shape mirrors the
//     plan-and-solve "plan then carry out subtasks" structure (Task 1).
//   * arXiv:2305.18323 ReWOO — planner / worker / solver decoupling;
//     supports the bubble-up `LedgerReset` cascade shape (Task 7).
//
// See workflow/reasoning/E-46-decisions.md.
// TOTAL_VARIANTS 456 -> 457.
//
// E-48 (SCHEMA_VERSION 185): bump 184 -> 185 for WebResearchEvent
// SINGLE-VARIANT Pattern-2 (37th application). WebResearchEventKind 10-arm
// discriminator covers research_plan_emitted / tiered_search_dispatched /
// extraction_failed_all_extractors / cross_encoder_rerank_complete /
// speculative_branch_pruned / research_preemptive_stop /
// contradiction_node_materialized / citation_verification_outcome /
// citation_attribution_violation_blocked / recency_decay_applied.
// Wire-dark unless ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1.
// Producer surfaces: orchestrator/src/research/web_orchestrator/* +
// web_search/src/{tiered_search_pool,extractor_pool,cross_encoder_rerank}.rs.
// Collapses the plan's nominally 17-named fire-points into ONE variant +
// 10-arm Kind discriminator per the multi-event Pattern-2 precedent
// (E-50 11-arm / E-4 13-arm / E-54 14-arm). Several plan-named
// fire-points fold into a single Kind via outcome discriminators in
// details_json (e.g. citation_dead_url / citation_invalid_arxiv /
// citation_invalid_doi / citation_unfaithful all ride
// `citation_verification_outcome` with the `outcome` field; the
// tier_disabled_quota_exceeded / tiered_search_degraded_to_t2_only
// pair rides `tiered_search_dispatched` with a `status` field).
// SCHEMA_VERSION_PUBLIC consumers do not interact with the discriminator
// directly. Paper grounding (verified May 2026 via WebSearch):
//   * arXiv:2510.05145 FlashResearch (breadth/depth/speculative branches
//     + runtime pruning on intermediate evidence delta)
//   * arXiv:2602.06718 GhostCite (14-95% citation hallucination baseline;
//     URL/arXiv/DOI + SemanticCite 4-stage defense)
//   * arXiv:2602.05930 Compound Deception NeurIPS 2025 taxonomy
//   * arXiv:2604.03173 CITEVERIFIER / SemanticCite / CiteGuard detectors
//   * arXiv:2601.18724 HalluCitation Matters (arxiv-ID HEAD highest yield)
//   * arXiv:2602.03814 Conformal Thinking (triple-threshold convergence)
//   * arXiv:2504.00180 Contradiction Detection in RAG
//   * arXiv:2511.06668 Healthcare contradictory-evidence RAG (never silently resolve)
//   * arXiv:2510.03418 LegalWiz (consistency voting)
//   * arXiv:2509.19376 Solving Freshness in RAG (recency-decay scoring)
//   * arXiv:2509.23519 ReliabilityRAG (per-source reliability tiers)
//   * simonwillison.net/2025/Jun/14 Anthropic multi-agent writeup
//     (lead-never-cites attribution lock)
//   * docs.cohere.com/docs/reranking-best-practices (4093-token chunks,
//     top-50 -> top-5 max-pooled)
// See workflow/reasoning/E-48-decisions.md.
// TOTAL_VARIANTS 457 -> 458.
//
// E-44 (SCHEMA_VERSION 186): bump 185 -> 186 for DurabilityEvent
// SINGLE-VARIANT Pattern-2 (38th application). DurabilityEventKind 10-arm
// discriminator covers tool_idempotency_hit /
// tool_idempotency_replayed_irreversible / compensation_logged /
// compensation_replayed / compensation_failed /
// session_dag_continue_as_new / checkpoint_lazy_migrated /
// compact_lock_contended / agent_reliability_metrics /
// tool_error_classified. Wire-dark unless
// ELAI_DURABILITY_OBSERVABILITY_ENABLE=1. Producer surfaces:
// orchestrator/src/durability/{idempotent_dispatcher,continue_as_new,
// checkpoint_migrator,compact_lock,resume_validator}.rs. Closes the
// TASK-E44-013 observability gap: the 4 of 10 E-44 task primaries
// already SHIPPED (idempotency_dispatcher, continue_as_new,
// checkpoint_migrator, compact_lock + resume_validator bonus) had ZERO
// audit-row surface area before this bump. The remaining 5 primaries
// (saga compensation log, madsim DST, postcard feature flag,
// reliability metrics, tool-error 3-bucket recovery) are NOT-SHIPPED;
// the Kind arms covering them (compensation_*, agent_reliability_metrics,
// tool_error_classified) reserve wire-tag space so when the producer
// lands the SCHEMA does not need to bump. Pattern-2 collapse saves 9
// SCHEMA slots + 9 ALL_VARIANTS rows + 9 alphabetical inserts vs the
// plan's named 10 sibling variants (per E-46 8-arm + E-48 10-arm
// precedent). Each plan-promised name maps 1:1 to a Kind:
// ToolIdempotencyHit, ToolIdempotencyReplayedIrreversible,
// CompensationLogged, CompensationReplayed, CompensationFailed,
// SessionDagContinueAsNew, CheckpointLazyMigrated,
// CompactLockContended, AgentReliabilityMetrics, ToolErrorClassified.
// SCHEMA_VERSION_PUBLIC consumers do not interact with the discriminator
// directly. See workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E44-013.md.
// TOTAL_VARIANTS 458 -> 459.
//
// E-31 (SCHEMA_VERSION 187): bump 186 -> 187 for HeadDimInvariantVerified
// SINGLE-VARIANT Pattern-2 (39th application). Carries
// HeadDimInvariantVerifiedPayload with a 2-arm
// HeadDimInvariantVerifiedKind discriminator (Verified / Refused) +
// free-form details_json. Wire-dark unless
// ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE=1. Producer surface:
// python/elai_inference/server.py::assert_head_dim_invariant — emits a
// Verified row on successful Qwen2.5-Coder boot, raises SidecarBootError
// on mismatch (no SSB row possible at boot-refuse time, but the Refused
// kind covers post-boot health-check re-evaluation). Closes
// TASK-E31-001 (head_dim invariant + SSB row gap from cross-check
// E-10_e29_e30_e31.md). Scope-lean cut: this bump ships the SSB wire
// surface only; CloudRouteDecision + TierModelSwap remain deferred per
// the E-31 plan body. See
// workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E31-001.md.
// TOTAL_VARIANTS 459 -> 460.
//
// E-3 (SCHEMA_VERSION 188): bump 187 -> 188 for RoleShownModelHidden
// SINGLE-VARIANT Pattern-2 (40th application). No Kind discriminator --
// this is the scope-lean MVP slice of E-3 (TASK-E03-001) closing the
// cross-check finding "SettingsWindow explicitly exposes MODEL names per
// role" by reversing the default (ROLE shown, MODEL hidden unless the
// user toggles "show model details" ON in SettingsWindow). Producer:
// crates/dashboard-v2/src-tauri/src/commands.rs::record_model_visibility
// (Tauri command emitted by SettingsWindow.svelte on toggle change).
// The full E-3 plan (role_prompt.rs / voice.toml / RuntimeAdapter::warm
// / 5 other SSB variants / persona-vector hook) is DEFERRED to follow-on
// sessions. Wire-dark unless ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE=1.
// Payload carries {role: String, model_hidden: bool, session_id:
// Option<String>, ts_ms: i64}. See
// workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E03-001.md.
// TOTAL_VARIANTS 460 -> 461.
//
// E-49 (SCHEMA_VERSION 189): bump 188 -> 189 for CodingMetricNamedToken
// SINGLE-VARIANT Pattern-2 (41st application). Carries
// CodingMetricNamedTokenPayload with a 3-arm CodingMetricNamedTokenKind
// discriminator (StablePassAt1 / HarnessAttributionTuple / StopVerified)
// + free-form details_json. Wire-dark unless
// ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE=1. Closes TASK-E49-005 / 006 / 007
// SPEC-DRIFT against the named tokens stable_pass_at_1 /
// harness_attribution_tuple / stop_verified -- the original E-49 ship
// landed CodingAgentMetricsCaptured + RUNPOD-PENDING runners but the 3
// named tokens the spec called out had zero hits across the codebase
// (per cross-check E-16_e47_e48_e49.md). Pattern-2 collapse saves 2
// SCHEMA slots + 2 ALL_VARIANTS rows + 2 alphabetical inserts vs the
// plan's named 3 sibling variants. Each named token maps 1:1 to a Kind:
//   * stable_pass_at_1 -- n=5 seed stability metric (replaces single-seed
//     Pass@1); aggregator wraps TestOutcomeReport with std-dev /
//     identical-output check.
//   * harness_attribution_tuple -- (model_id, harness_version,
//     harness_commit_sha) recorded per session start so B-37
//     harness-vs-model ablation is legible.
//   * stop_verified -- CI lint refusing legacy HumanEval / MBPP /
//     SWE-bench Verified mention without explicit STOP-Verified: guard;
//     prevents accidental re-introduction of contaminated benches.
// SCHEMA_VERSION_PUBLIC consumers do not interact with the discriminator
// directly. See workflow/sessions/s-night-2026-05-30/impl-tasks/
// TASK-E49-005.md.
// TOTAL_VARIANTS 461 -> 462.
//
// E-3 sub-phase 4A (SCHEMA_VERSION 190): bump 189 -> 190 for
// E3RuntimeHandoffEvent SINGLE-VARIANT Pattern-2 (42nd application).
// Carries RuntimeHandoffEventPayload with a 6-arm RuntimeHandoffEventKind
// discriminator (UnifiedPreambleApplied / RuntimePrewarmStarted /
// RuntimePrewarmComplete / HandoffPayloadCompressed / PersonaVectorApplied /
// StyleNormalizerApplied) + free-form details_json. Wire-dark unless
// ELAI_E3_RUNTIME_HANDOFF_ENABLE=1. Pattern-2 collapse rationale (per
// workflow/sessions/s-21636/exploration/tranche4-E3-dashboard.md): the
// original E-3 plan called out 6 sibling SsbActionType variants; collapsing
// to a single envelope saves 5 SCHEMA slots + 5 ALL_VARIANTS rows + matches
// the documented 40+ precedent applications. Producer surfaces (sub-phase
// 4A wires only the UnifiedPreambleApplied arm via tracing log; actual
// Ssb::write_payload injection into runtime_orchestrator.rs::try_complete
// is deferred to sub-phase 4B alongside the other 5 arms):
//   * UnifiedPreambleApplied -- crates/orchestrator/src/runtime_orchestrator.rs
//     wraps messages with build_role_prompt before adapter.complete
//   * RuntimePrewarmStarted / RuntimePrewarmComplete -- predict_next_role
//     dispatcher in runtime_orchestrator.rs (4B)
//   * HandoffPayloadCompressed -- handoff_payload.rs compress_handoff_payload (4B)
//   * PersonaVectorApplied -- python/elai_inference/server.py (sidecar 4B)
//   * StyleNormalizerApplied -- crates/cli/src/live_cli.rs post-pass (4B)
// See workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E03-001.md +
// docs/thefinalglance/E-plan/E-3_model_handoff_ux.md.
// TOTAL_VARIANTS 462 -> 463.
//
// E-31 sub-phase 6A (SCHEMA_VERSION 191): bump 190 -> 191 for
// RouteDecisionEvent SINGLE-VARIANT Pattern-2 (43rd application).
// Carries RouteDecisionEventPayload with a 3-arm RouteDecisionEventKind
// discriminator (CloudRouteChosen / TierModelSwapped / LocalRouteKept) +
// free-form details_json. Wire-dark unless
// ELAI_ROUTE_DECISION_SSB_ENABLE=1. Producer:
// crates/orchestrator/src/cloud_mode.rs::CloudModeRouter::route via
// providers::runtime::RuntimeRoutingPolicy::evaluate. Closes the
// E-31 plan's CloudRouteDecision + TierModelSwap deferred slots in
// a SINGLE envelope per Pattern-2 precedent (saves 1 SCHEMA slot vs
// shipping two siblings; matches CodingMetricNamedToken 3-arm shape).
// Per cross-check + tranche6 exploration: HeadDimInvariantVerifiedKind
// is a CLOSED 2-arm (Verified/Refused) Pattern-2 variant; this is the
// NEW sibling Pattern-2 variant per ssb.rs:22883 + ssb.rs:23119 "closed
// enum; adding a Nth kind requires SCHEMA_VERSION bump" rule. See
// workflow/sessions/s-21636/exploration/tranche6-E31-wiring.md.
// TOTAL_VARIANTS 463 -> 464.
//
// E-47 (SCHEMA_VERSION 192): bump 191 -> 192 for LoraShipGateEvent
// SINGLE-VARIANT Pattern-2 (44th application). Carries
// LoraShipGateEventPayload with a 3-arm LoraShipGateEventPhase
// discriminator (Refused / CalibrationMissing / CalibrationPassed) +
// free-form details_json. Wire-dark unless
// ELAI_LORA_SHIP_GATE_ENABLE=1. Producer:
// crates/orchestrator/src/lora_ship_gate.rs::notify_lora_ship_gate +
// `elai lora-gate-check` CLI subcommand. Ships the LoRA ship-gate
// LOCALLY (default-deferred posture per source report
// workflow/improvement-sweep-2026-05-19/reports/47-lora-finetuning.md)
// even though the producer-side LoRA training stack (Unsloth +
// QLoRA-NF4 + PiSSA + DoRA on RTX 3090) is RUNPOD-PENDING. The 3
// Phase arms collapse the plan's nominally-3 sibling SSB rows
// (lora_ship_gate_refused / lora_calibration_gate_passed /
// lora_calibration_gate_refused) into ONE Pattern-2 variant per the
// E-43 SkillSystemEvent precedent. Pattern-2 collapse saves 2 SCHEMA
// slots + 2 ALL_VARIANTS rows + 2 alphabetical inserts. Triple-AND
// gate semantic (enabled + confirm_user_understanding +
// calibration_passes_threshold) wired in lora_ship_gate.rs;
// `DEFERRED_PER_E47_REPORT` is the load-bearing CI lint token enforced
// by .elai_cc/scripts/check_lora_ship_gate_token.{sh,bat} +
// .github/workflows/elai-lora-ship-gate.yml. Paper grounding:
// arXiv:2405.09673 Biderman et al. (5-19pp catastrophic forgetting
// evidence motivating the no-regression -2.0pp floor). See
// workflow/reasoning/E-47-decisions.md.
// TOTAL_VARIANTS 464 -> 465.
//
// E-40 Tranche B-2 (SCHEMA_VERSION 193): bump 192 -> 193 for
// SymbolGraphEvent SINGLE-VARIANT Pattern-2 (45th application).
// Carries SymbolGraphEventPayload with a 4-arm SymbolGraphEventKind
// discriminator (SymbolDslExecuted / DataflowImpactComputed /
// AgentImpactClaimRefuted / ScopeGraphResolved) + free-form
// details_json. Wire-dark unless ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE=1
// (5th-list RRF emit) or other per-event gates upstream. Producer:
// crates/orchestrator/src/symbol_graph/mod.rs::notify_symbol_graph_event
// + crates/orchestrator/src/symbol_graph/dsl.rs::exec_dsl +
// crates/orchestrator/src/symbol_graph/dataflow.rs::impact_analysis
// + the new symbol_dsl_query / symbol_dataflow_impact ToolDefinitions
// in production_agent_rpc.rs::build_tool_catalog. The 4 Kind arms
// collapse the plan's nominally-4 sibling SSB rows (symbol_dsl_executed
// / dataflow_impact_computed / agent_impact_claim_refuted /
// scope_graph_resolved) into ONE Pattern-2 variant per the E-47
// LoraShipGateEvent precedent. ScopeGraphResolved is reserved here —
// the scope_graph implementation lands in Tranche B-3 (TASK-E40-002)
// but reserving the Kind slot now avoids a 2nd SCHEMA bump on B-3
// landing. Pattern-2 collapse saves 3 SCHEMA slots + 3 ALL_VARIANTS
// rows + 3 alphabetical inserts. See
// workflow/reasoning/E-40-tranche-B2-decisions.md (DECISION-8 addendum).
// TOTAL_VARIANTS 465 -> 466.
//
// Tranche B-6 quad (SCHEMA_VERSION 194 -- 2026-05-31): coordinated single
// bump covers four plans landing in one orchestrate run:
//   * C-42 SHIPPABLE-NOW (proc-macro deferred to C-42b):
//       NEW SsbActionType::McpAuthzEvent + 3-arm McpAuthzEventKind
//       discriminator (AuthzCheckPassed / AuthzCheckFailed / CorpusReplayed).
//       Wire-dark unless ELAI_C42_MCP_AUTHZ_EMIT_ENABLE=1. Producer:
//       crates/plugins/src/authz/compose.rs::compose_with_tool_gate wrapper
//       at the caller side; check_authz stays pure (deliberate
//       defense-in-depth boundary -- see checker.rs:1-9).
//   * C-47 PATTERN-2 COLLAPSE:
//       NEW SsbActionType::ToolCalibrationEvent + 6-arm
//       ToolCalibrationEventKind discriminator (HintEmitted / DriftDetected
//       / CallLogged / CaseAdded / CaseSelected / CalibrationApplied).
//       3 pre-existing helpers (ToolCalibrationHintEmitted +
//       ToolCalibrationDriftDetected + ToolCallLogged) are KEPT alive for
//       one release window as deprecation aliases -- the new variant +
//       new helper land here; gradual migration happens in C-47b. Wire-dark
//       unless ELAI_C47_CASE_BASE_ENABLE=1.
//   * B-39 SINGLE-ADD:
//       NEW SsbActionType::ReflectBudgetSoftWarn (single variant; reflect_*
//       family stays alphabetical-contiguous, 5th sibling cheapest).
//       Wire-dark unless ELAI_B39_SOFTWARN_ENABLE=1. Producer:
//       crates/reasoning/src/chat_reflect.rs ~L454 (BEFORE the existing
//       hard-cap log-only block).
//   * C-50 PATTERN-2 COLLAPSE:
//       NEW SsbActionType::ContractCodingEvent + 3-arm
//       ContractCodingEventKind discriminator (RetryAttempted /
//       RetryResolved / RetryAbandoned). Pre-existing
//       ContractViolationDetected stays alive (NOT collapsed in this
//       tranche -- separate landing). Wire-dark unless
//       ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE=1. Producer:
//       crates/orchestrator/src/task_runner.rs::fix_and_rerun_until_green
//       Healed (L1055) + EscalatedToHuman (L1109) + Failed (L1057) branches.
//
// TOTAL_VARIANTS delta: +4 new variants (McpAuthzEvent,
// ToolCalibrationEvent, ReflectBudgetSoftWarn, ContractCodingEvent);
// 466 -> 470. 3 deprecated tool_calibration_* variants kept for one release
// window of replay back-compat. See workflow/reasoning/
// B6-quad-tranche-decisions.md.
//
// Tranche B-7 triple (SCHEMA_VERSION 195 -- 2026-05-31): coordinated single
// bump covers three plans landing in one orchestrate run:
//   * E-13 SHIPPABLE-NOW: NEW SsbActionType::TddVerificationEvent +
//       4-arm TddVerificationEventKind discriminator (PropertyGatePass /
//       MutationGateSurvivorBlock / AntiMockBlock / AssertionOnlyDiffBlock).
//       Pattern-2 (46th application). Wire-dark unless
//       ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE=1. Producer (survival cache
//       arm): crates/orchestrator/src/survival_cache.rs short-circuits
//       known-survivor cargo-mutants runs.
//   * E-12 carry: AssertionOnlyDiffBlock collapses INTO arm #4 of
//       TddVerificationEventKind (NO separate top-level variant). Producer
//       wiring (assertion_diff scanner) DEFERRED per exploration --
//       substrate-only this tranche.
//   * E-50 extension: SaasArchPatternEventKind gains a 12th arm
//       SaasArchetypeSelected (within-variant arm extension; NO new
//       top-level variant; NO additional SCHEMA bump). Wire-dark unless
//       ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE=1. Producer:
//       crates/orchestrator/src/architecture/recommender.rs::
//       recommend_archetype_with_emit.
//
// TOTAL_VARIANTS delta: +1 new top-level variant (TddVerificationEvent);
// 470 -> 471. SaasArchPatternEventKind arm extension is variant-internal
// (no TOTAL_VARIANTS delta). See workflow/reasoning/
// B7-triple-tranche-decisions.md.
//
// F-1 / F-2a (SCHEMA_VERSION 196 -- 2026-06-01): coordinated single
// bump covers two new vibe-coding security gate variants:
//   * SsbActionType::VibeGateBlocked -- Pattern-2 single variant with
//       4-arm VibeGateBlockedKind discriminator (CodeVuln / Secret /
//       DepHallucinated / Skill). sql_str arms:
//       vibesec_code_vuln_blocked / vibesec_secret_blocked /
//       vibesec_dep_hallucinated_blocked / vibesec_skill_blocked.
//       DepHallucinated + Skill are RESERVED (parse/sql arms only;
//       NO producer this session per plan). Wire-dark unless
//       ELAI_VIBE_GATE_ENABLE=1.
//   * SsbActionType::SecurityRuleBlocked -- flat beacon (no Kind
//       discriminator). sql_str: security_rule_blocked.
//       Wire-dark unless ELAI_SECURITY_RULE_BLOCKED_ENABLE=1.
// TOTAL_VARIANTS delta: +2 (471 -> 473).
// Session: s-6344. See workflow/reasoning/F-1-F2a-decisions.md.
//
// F-4 / Slice B-1 (SCHEMA_VERSION 197 -- 2026-06-02): coordinated single
// bump for the SSB foundation + IntegrationRunResult.output_capture slice.
//   * SsbActionType::TestIntegrityEvent -- SINGLE-VARIANT Pattern-2 (50th
//       application). 2-arm TestIntegrityEventKind discriminator:
//       Verified / FabricationBlocked. sql_str arms:
//       test_integrity_verified / anti_fabrication_block.
//       Carries TestIntegrityEventPayload { test_name, passed,
//       stdout_hash: Option<String>, kind }.
//   * stdout_hash: Option<String> (#[serde(default)]) added to
//       TestOutcomePayload -- backward-compatible additive field.
// TOTAL_VARIANTS delta: +1 (473 -> 474).
// Session: s-6344. See workflow/reasoning/F-4-B1-decisions.md.
//
// F-3p1 / Slice C1 (SCHEMA_VERSION 198 -- 2026-06-02): capability-flow core
// + SSB ToolFilterEvent substrate.
//   * SsbActionType::ToolFilterEvent -- SINGLE-VARIANT Pattern-2 (51st
//       application). 2-arm ToolFilterEventKind discriminator:
//       Committed / UntrustedQuarantined. sql_str arms:
//       tool_filter_committed / untrusted_var_quarantined.
//       Carries ToolFilterPayload { commit_hash, allowed_categories,
//       denied_tool: Option<String>, taint_source: Option<String>, task_id }.
//   * CapabilityFlowBlocked is RESERVED (Tier-3); no producer this session
//       per W-8 (no in-plan caller). NOT added to the enum.
// TOTAL_VARIANTS delta: +1 (474 -> 475).
// Session: s-6344. See workflow/reasoning/F-3p1-decisions.md.
//
// F-9 / Slice D1 (SCHEMA_VERSION 199 -- 2026-06-02): VibeSecDep write-time
// slopsquat gate + SSB WriteTimeSlopsquatBlocked substrate.
//   * SsbActionType::WriteTimeSlopsquatBlocked -- flat beacon (not Pattern-2).
//     sql_str: write_time_slopsquat_blocked.
//     Carries WriteTimeSlopsquatBlockedPayload { dep_name_hash: u64,
//     ecosystem: String, verdict_kind: String, registry_age_days: Option<u32>,
//     ts_ms: i64 }.
//   * WriteTimeSlopVerdictKind { NotFound, TooNew, RegistryUnreachable } --
//     kind-as-String (verdict_kind field, not a Pattern-2 enum variant).
// TOTAL_VARIANTS delta: +1 (475 -> 476).
// Session: s-6344. See workflow/reasoning/F-9-D1-decisions.md.
//
// F-10 / Slice A1 (SCHEMA_VERSION 200 -- 2026-06-02): security-monotonicity
// iteration guard + SSB SecurityClassRegressionBlocked substrate.
//   * SsbActionType::SecurityClassRegressionBlocked -- flat beacon (not a
//     Pattern-2 Kind variant on the action-type enum). sql_str:
//     security_class_regression_blocked.
//     Carries SecurityClassRegressionBlockedPayload { delta_blake3: String,
//     introduced_classes: Vec<String>, prior_classes: Vec<String>,
//     loop_kind: LoopKind, loop_id: String, lang: String }.
//   * LoopKind { Refine, AutoAdvance, SelfMod } -- snake_case payload field
//     discriminating the 3 production legs (REFINE call-site, AUTO-ADVANCE
//     evidence-check, SELF-MOD precondition). It is a PAYLOAD field, NOT a
//     discriminator on the SsbActionType enum arm.
// Default-ON; kill-switch only (ELAI_SECURITY_MONOTONICITY_DISABLE).
// TOTAL_VARIANTS delta: +1 (476 -> 477).
// Session: s-60824. See workflow/reasoning/F-10-decisions.md.
//
// F-2b / Slice A1 (SCHEMA_VERSION 201 -- 2026-06-02): insecure-rate keystone
// bench beacon + SSB InsecureRateMeasured substrate.
//   * SsbActionType::InsecureRateMeasured -- flat beacon (not Pattern-2).
//     sql_str: insecure_rate_measured.
//     Carries InsecureRateMeasuredPayload { suite: String, model_id: String,
//     n_samples: u32, insecure_rate_before: f64, insecure_rate_after: f64,
//     gate_recall: f64, seed: u64, corpus_blake3: String }.
//   * Bench-run telemetry (emitted when the static-gate insecure-rate
//     benchmark runs over the SeCodePLT-structured CWE corpus). NO new
//     ELAI_*_ENABLE flag -- it is NOT a wire-dark gate; it fires only inside
//     the bench. before = corpus ground-truth insecure-rate; after =
//     gate-approved-subset insecure-rate (static FN residual); gate_recall =
//     (before - after) / before. HONEST static-gate insecure-rate, DISTINCT
//     from the SeCodePLT dynamic secure-coding rate.
// TOTAL_VARIANTS delta: +1 (477 -> 478).
// Session: s-60824. See workflow/reasoning/F-2-decisions.md.
//
// F-6 / Slice A1 (SCHEMA_VERSION 202 -- 2026-06-02): learned-skill library
// trust ladder + SSB SkillLibraryEvent substrate.
//   * SsbActionType::SkillLibraryEvent -- SINGLE-VARIANT Pattern-2. 3-arm
//     SkillLibraryEventKind discriminator: Acquired / TrustPromoted /
//     DemotedStale. sql_str arms: skill_acquired / skill_trust_promoted /
//     skill_demoted_stale.
//     Carries SkillLibraryEventPayload { skill_id: String, description: String,
//     tier: String, prior_tier: Option<String>, reason: String,
//     invocations: u32, failure_rate: f64, kind }.
//   * SkillTrustTier { S1, S2, S3, S4 } is a NEW LOCAL enum in
//     crate::skill_trust (NOT a Pattern-2 Kind on the SsbActionType enum and
//     NOT bridged to elai_memory::TrustTier). The SSB row's tier/prior_tier
//     are snake_case SkillTrustTier labels.
//   * No ELAI_*_ENABLE default-off flag (W-2); the autonomy barrier in
//     crate::autonomous_gate governs dispatch, not a wire-dark gate.
// TOTAL_VARIANTS delta: +1 (478 -> 479).
// Session: s-60824. See workflow/reasoning/F-6-decisions.md.
//
// F-5 / Slice A1 (SCHEMA_VERSION 203 -- 2026-06-02): cross-subsystem
// staleness-quarantine + bi-temporal contradiction cascade substrate.
//   * SsbActionType::StalenessQuarantineBroadcast -- SINGLE FLAT beacon (NO
//     Pattern-2 Kind). sql_str: staleness_quarantine_broadcast. Mirrors the
//     B-30 LanguageGrammarStatus / E-5 MemoryFactSuperseded flat-beacon shape.
//   * Carries StalenessQuarantineBroadcastPayload { fact_id: String,
//     staleness_ratio: f64, dependents_demoted: u32,
//     cosmos_artifacts_flagged: u32, tick_ms: u64, broadcast_id: String,
//     dropped_count: u32 }. Emitted at the orchestrator/CLI decay-tick
//     boundary (NOT inside crates/memory decay.rs); the CLI drains a bounded
//     QuarantineBroadcastQueue and applies the knowledge-side cascade
//     (DKG concept_stale=2 + non-destructive staleness_quarantined marker edge
//     + Cosmos artifact flag). dropped_count surfaces DROP-OLDEST overflow.
//   * No ELAI_*_ENABLE flag for the SSB row itself; decay cadence + queue
//     constants are ELAI-owned (QUARANTINE_QUEUE_CAPACITY=256).
// TOTAL_VARIANTS delta: +1 (479 -> 480).
// Session: s-60824. See workflow/reasoning/F-5-decisions.md.
//
// F-8 / Slice A1 (SCHEMA_VERSION 204 -- 2026-06-02): SCF symbolic semantic
// conflict detector. LLM-free symbolic check that flags a self-mod config
// variant contradicting an active locked-key invariant BEFORE archive
// admission.
//   * SsbActionType::SymbolicConflictDetected -- SINGLE FLAT beacon (NO
//     Pattern-2 Kind). sql_str: symbolic_conflict_detected. Mirrors the F-5
//     StalenessQuarantineBroadcast / F-10 SecurityClassRegressionBlocked
//     flat-beacon shape.
//   * Carries SymbolicConflictPayload { fingerprint: String, kind: String,
//     key: String, detail: String, candidate_id: String, fsm_substate:
//     String }. Emitted at the `elai self-mod apply` CLI entrypoint (a W-3
//     production entrypoint) when the symbolic detector flags a contradiction
//     against self_mod_safety::LOCKED_KEYS; the variant is held PENDING (NOT
//     hard-fail-closed BLOCK -- POLICY is WARN-then-escalate). The detector
//     ALSO composes at the F-10 darwinian-evolve seam (external_benches.rs)
//     so it fires in that loop when it ships (GATED).
//   * No ELAI_*_ENABLE flag for the SSB row itself; detector thresholds are
//     ELAI-owned. fingerprint is a blake3 hex-prefix of the conflict tuple.
// TOTAL_VARIANTS delta: +1 (480 -> 481).
// Session: s-60824. See workflow/reasoning/F-8-decisions.md.
//
// F-7 / Slice A1 (SCHEMA_VERSION 205 -- 2026-06-02): CBR template utility
// ranking (HALF B only -- Laplace-smoothed outcome utility). Records the
// Laplace-utility score that gates which crystallized pipeline template is
// (re)used at try_execute, so low-sample templates stop being over-trusted.
//   * SsbActionType::TemplateUtilityRanked -- SINGLE FLAT beacon (NO Pattern-2
//     Kind). sql_str: template_utility_ranked. Mirrors the F-8
//     SymbolicConflictDetected / F-5 StalenessQuarantineBroadcast flat-beacon
//     shape.
//   * Carries TemplateUtilityRankedPayload { concept_class: String,
//     plan_skeleton_hash: u64, utility: f32, first_try_ratio: f32,
//     sample_n: u32, chosen_pipeline_id: String }. utility =
//     laplace_utility(successes, total) = (successes + ALPHA) / (total + ALPHA
//     + BETA) with ALPHA = BETA = 1.0 (ELAI-owned constants). first_try_ratio
//     is the recorded baseline (the persisted CompiledPipeline
//     first_try_success_rate) kept alongside for before/after dashboard
//     display (W-2 no-removal). successes/total come from the PERSISTED
//     CompiledPipeline.first_try_success_rate + use_count read via the
//     CompiledPipelineStore -- NOT from SSB compiled-pipeline events (those
//     carry NO per-firing pass/fail). block_ratio + coverage are DROPPED (no
//     producer; zero-filling would be a CSQ footgun) -- documented as future
//     signals, not shipped.
//   * No ELAI_*_ENABLE flag for the SSB row itself; ALPHA/BETA smoothing
//     constants are ELAI-owned. The Springdrift +10.7% figure is UNGROUNDED
//     and is explicitly NOT a target -- it is not encoded anywhere in this
//     payload or its thresholds.
// TOTAL_VARIANTS delta: +1 (481 -> 482).
// Session: s-60824. See workflow/reasoning/F-7-decisions.md.
//
// F-11 / Slice A1 (SCHEMA_VERSION 207, 2026-06-03): DarwinianAdmit Pattern-2
// single-variant WITH a 3-arm DarwinianAdmitKind discriminator (Admitted /
// RejectedByGuard / LoopClosed). Emitted by the config-DGM admit-loop driver
// (crates/cli/src/darwin_loop_cmd.rs -> orchestrator::darwin_admit_loop) when a
// candidate is routed through the four-guard chain (tripwire blake3 fingerprint,
// trap-pattern scorer, is_security_sensitive, Two-Gate). Wire-dark unless
// ELAI_DARWINIAN_LOOP_ENABLE=1 AND cfg(feature = "darwinian-evolve"). Distinct
// from the pre-existing E-7 DarwinianGenerationCompleted action_type. TOTAL_-
// VARIANTS delta: +1 (482 -> 483 in action_type_lock; F-13 already at 483 so
// this bumps to 484). Session: s-6312. See workflow/reasoning/F-11-decisions.md.
// D-5 closure BaiPromotionDecision (SCHEMA_VERSION 208): unified summary row
// emitted by `bai::emit_bai_round_with_canary` after the per-decision
// TemplatePromoted / TemplateCandidateRejected row. Carries (template_id,
// decision, stage, evidence_pass, evidence_total, audit_id) so dashboards
// can render the promotion verdict without joining across the three-row
// sequence. action_type_lock.rs TOTAL_VARIANTS 517 -> 518.
// H-32 (SCHEMA_VERSION 210): FactValidationFailed single flat beacon.
// Emitted by the H-32 citation-validity gate when a Tier-A fact's DKG
// citation fails to re-resolve. Carries (fact_id, citation, reason, evidence_blake3).
// Producer: get_relevant_facts (read side, live_cli tick maintenance side).
// Wire-dark until a Tier-A DomainKnowledge/Mistake fact with a dkg: pointer
// is stored and then its cited symbol is renamed/deleted. TOTAL_VARIANTS 520 -> 521.
// H-33 (SCHEMA_VERSION 211): MemoryGovernanceEvent single flat beacon.
// Emitted by `elai memory forget|pin|unpin` and the dashboard
// memory_forget/memory_pin Tauri commands. 3-arm kind discriminator:
// Forgotten / Pinned / Unpinned. TOTAL_VARIANTS 521 -> 522.
// H-34 (SCHEMA_VERSION 212): McpToolDeferred -- deferred schema-resolve row per STDIO tool-call.
// Emitted by mcp_glue::RuntimeMcpState::call_tool on cache-miss when
// ELAI_MCP_DEFER_SCHEMA_ENABLE=1. Carries server_name, tool_qualified_name,
// resolve_ns, cache_hit, injected_floor_size. TOTAL_VARIANTS 522 -> 523.
// H-34 (SCHEMA_VERSION 213): McpSchemaBudgetMeasured -- mcp-schema-budget bench result beacon.
// Emitted by `elai bench mcp-schema-budget`. Carries suite, baseline_schema_tokens_per_turn,
// injected_schema_tokens_per_turn, reduction_tokens, deferred_resolves, seed, corpus_blake3.
// TOTAL_VARIANTS 523 -> 524.
// H-35 (SCHEMA_VERSION 214): SessionRewound -- /rewind restore beacon.
// Emitted by `elai rewind <target>` and the `rewind_session` Tauri command. Carries
// session_id, target_seq, head_seq_before, scope, summarized, replay_session_id,
// replay_outcome, tail_fingerprint_blake3. TOTAL_VARIANTS 524 -> 525.
// H-36 (SCHEMA_VERSION 215): AgentNotificationEmitted -- completion/needs-input
// dev-event beacon. Emitted by `notify_dispatch::NotificationDispatcher::notify`
// on PhaseComplete (task_runner.rs post-PhaseComplete write) and on
// AutonomyDecision::RequestHuman (task_runner.rs:1163 fix_and_rerun cap).
// 2-arm AgentNotificationKind discriminator: Completion | NeedsInput.
// TOTAL_VARIANTS 525 -> 526.
// H-37 (SCHEMA_VERSION 216): PlanDecisionEvent -- plan-approval gate decision
// emitted at PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST by the intercept in
// task_runner.rs when ELAI_PLAN_APPROVAL_GATE_ENABLE=1. Pattern-2 single-variant
// 4-arm PlanDecisionKind discriminator: Approve | Redirect | Reject | AutoAdvance.
// TOTAL_VARIANTS 526 -> 527.
// H-38 (SCHEMA_VERSION 217): StopConfirmationEvent -- cost-confirmation gate
// outcome on task/pod stop. Emitted by dispatch_stop when ELAI_STOP_COST_CONFIRM_ENABLE=1
// (PassedNoFriction) or unconditionally on ConfirmRequired / RejectedHeld.
// Pattern-2 single-variant 3-arm StopOutcomeKind discriminator:
// ConfirmedProceed | RejectedHeld | PassedNoFriction. TOTAL_VARIANTS 527 -> 528.
// H-39 (SCHEMA_VERSION 218): AbstentionScored -- abstention-aware scoring beacon.
// Emitted by the bench scorer (bench_safedelta/bench_seccodeplt) after computing
// the AbstentionAggregate over the run's outcome triples. Closes the
// self-improvement loop misalignment: D-16's RefusalDecision::Abstain fires at
// inference but darwin_admit_loop's reward was abstention-blind (GAP-1).
// Pattern-2 single-variant flat beacon. TOTAL_VARIANTS 528 -> 529.
// H-41 (SCHEMA_VERSION 219): CheckerGapDetected -- emitted at the route_claim
// call site (agent_pool.rs) when a Factual/Computational claim falls to
// non-Constructive because no VerifierKind::Deterministic is in
// OrchContext.available_verifiers. Off-agent backlog artifact (R4-compliant).
// Classified by Xu arXiv:2401.11817 Table-3 complexity taxonomy.
// Wire-dark under ELAI_CHECKER_GAP_ENABLE kill-switch (R9).
// Pattern-2 single-variant flat beacon. TOTAL_VARIANTS 529 -> 530.
// H-42 (SCHEMA_VERSION 220): ReasoningBudgetScaled -- serial-depth-scaled GRC
// budget decision beacon. Emitted at agent_pool::spawn + bsp/engine.rs timeout
// path when scale_grc_budget raises a sub-task's budget above the base.
// Grounding: Merrill & Sabharwal 2310.07923 (single pass ~TC0); Li et al.
// 2402.12875 (T steps simulate size-T circuit); Yehudai et al. 2503.01544
// (~O(n) CoT tokens). Default-ON (kill-switch ELAI_REASONING_BUDGET_SCALE_DISABLE).
// Pattern-2 single-variant 2-arm ReasoningBudgetScaledKind: DepthBound /
// AccessBound. TOTAL_VARIANTS 530 -> 531.
// H-43 (SCHEMA_VERSION 221): FitnessEvaluated -- execution-grounded DGM fitness
// beacon. Replaces the blake3-of-bytes derived_alpha placeholder (GAP-1) with a
// measured pass-rate signal (CodeRM arXiv:2501.01054 + CodeT arXiv:2207.10397).
// Emitted by darwin_admit_loop::run_one_generation per scored candidate under
// cfg(feature="darwinian-evolve") + ELAI_DARWINIAN_LOOP_ENABLE=1.
// Pattern-2 single-variant 3-arm FitnessEvaluatedKind:
// ExecutionPassRate | DualAgreement | FitnessRegression. TOTAL_VARIANTS 531 -> 532.
// H-44 (SCHEMA_VERSION 222): TrapBehaviorDetected -- behavioral cheat-detector
// beacon. Replaces the keyword-grep placeholder (GAP-2, trap_tasks.rs:123) with
// three behavioral detection legs: ImpossibleBench conflict canary (2510.20270),
// Isomorphic-Perturbation re-verification (2604.15149), CodeHalu execution-
// classification (2405.00253, composes H-43 ExecutionFitnessHarness). Also carries
// TrapRecallReport with measured recall over a frozen TRACE-style corpus
// (2601.20103). Default-ON under darwinian-evolve feature + ELAI_DARWINIAN_LOOP_ENABLE=1.
// Pattern-2 single-variant 5-arm TrapBehaviorDetectedKind (H-54 adds ClosureEscapeHatch):
// ImpossiblePassExploit | IsomorphMismatch | ExecutionHallucination | KeywordFallback |
// ClosureEscapeHatch. Adding a Kind arm does NOT add a new SsbActionType variant;
// TOTAL_VARIANTS unchanged at H-44's original 532 -> 533 delta.
// TOTAL_VARIANTS 532 -> 533.
//
// H-45 (SCHEMA_VERSION 223): PromotionScoreAggregated -- worst-case/variance-penalized
// aggregate over lineage-diverse promotion verdicts (Coste 2310.02743 WCO+UWO +
// Eisenstein 2312.09244 + Co-rewarding 2508.00410 + PRMBench 2501.03124 tau calibration).
// Replaces candidate's own mean fitness as the two_gate_promote margin. Default-ON under
// darwinian-evolve feature + ELAI_DARWINIAN_LOOP_ENABLE=1.
// Pattern-2 single-variant 3-arm PromotionScoreAggregatedKind:
// WorstCaseScored | VariancePenalized | LineageDiverseDisagree.
// TOTAL_VARIANTS 533 -> 534.
//
// H-46 (SCHEMA_VERSION 224): CladeSelected -- HGM clade-metaproductivity parent selection
// beacon. Emitted from config_archive::select_parent_with_cmp on the REAL loop selection
// step (darwin_admit_loop::run_one_generation -> select_parent -> CMP key) under
// cfg(feature="darwinian-evolve") + ELAI_DARWINIAN_LOOP_ENABLE=1.
// CMP aggregates execution-grounded descendant deltas (H-43; VD-1: NOT byte-hash).
// ImmediateScoreFallback fires for clade_size==0 leaves (no-lineage cold-start guard).
// Pattern-2 single-variant 2-arm CladeSelectedKind:
// CladeMetaproductivity | ImmediateScoreFallback. TOTAL_VARIANTS 534 -> 535.
//
// H-47 (SCHEMA_VERSION 225): EvalFirewallEvent -- DGM eval firewall beacon.
// Composes D-6 date-gate + E-54 template-regen + H-19 Thresholdout budget binding into
// a single gating step before the fitness scorer in darwin_admit_loop::run_one_generation.
// Emitted ONCE per generation (aggregate counts). The H-19 budget leg REUSES
// HoldoutBudgetEvent (GD-1: no second budget variant).
// Pattern-2 single-variant 3-arm EvalFirewallEventKind:
// PreUpdateItemRejected | TemplateRegenerated | IndistinguishabilityProbe.
// Wire-dark under existing darwinian-evolve feature + ELAI_DARWINIAN_LOOP_ENABLE=1.
// TOTAL_VARIANTS 535 -> 536.
//
// H-49 (SCHEMA_VERSION 226): EditHistoryEvent -- reversible LLM edits beacon.
// Emitted by the default-ON time-travel CAS capture hook in task_runner.rs on
// every file pre-edit capture, plus redo/branch/jump operations from the CLI.
// Pattern-2 single-variant 6-arm EditHistoryEventKind:
// SnapshotCaptured | Deduped | Redo | Branched | Jumped | TooLargeSkipped.
// Default-ON kill-switch: ELAI_TIMETRAVEL_DISABLE (NOT a default-OFF enable gate;
// no wire-dark-allowlist row required per R9). Wraps E-25 edit_checkpoints (GD-1).
// TOTAL_VARIANTS 536 -> 537.
//
// H-51 (SCHEMA_VERSION 227): HandoffLifecycleEvent -- cross-session sleep-time
// grounded-digest author + auto-detect + one-tap confirmed tiered load.
// Wires the user's handoff-protocol.md rule into the ELAI runtime so that
// cross-session resume costs ~1-3K tokens instead of ~20-27K (the full trimmed
// transcript). The digest is authored from SSB journal rows (NOT LLM recollection)
// via handoff_digest::author_topic_digest at session end / Drop. Detection is
// automatic; continuation requires one-tap confirm. Kill-switch:
// ELAI_HANDOFF_AUTORESUME_DISABLE (env var); registered in wire-dark-allowlist.toml.
// Pattern-2 single-variant 4-arm HandoffLifecycleEventKind:
// DigestWritten | ResumeOffered | ResumeConfirmed | TieredLoad.
// TOTAL_VARIANTS 537 -> 538.
//
// H-52 (SCHEMA_VERSION 228): FindingAnchored -- deterministic snippet->hunk
// finding-anchoring + line-less finding schema. Ports alibaba/open-code-review
// (Apache-2.0) resolver primitive with ELAI-specific fixes: duplicate-snippet
// Ambiguous guard, hunk-proximity tiebreak, Unanchorable on total miss. Closes
// G-5 Open-Question #2 (the hunk_id join risk): model emits a VERBATIM snippet,
// deterministic code resolves the hunk/line; a model-emitted line number is
// structurally impossible. Default-ON kill-switch: ELAI_REVIEW_ANCHOR_DISABLE
// (NOT a default-OFF gate; no wire-dark-allowlist row per R9).
// Pattern-2 single-variant 5-arm FindingAnchoredKind:
// Anchored | FileFallback | Ambiguous | Unanchorable | Relocated.
// TOTAL_VARIANTS 538 -> 539.
//
// H-53 (SCHEMA_VERSION 229): CitationAnchorsBound -- citation pre-binding
// + constrained synthesis on the answer path. Emitted by the
// `prebind_cmd::run_demo_prebind` CLI driver (REAL entrypoint: `elai prebind
// --demo-prebind`) and by the /research|/ask answer path after
// citation_prebind::freeze_anchors runs (when ELAI_CITATION_PREBIND_DISABLE
// is NOT set). Carries the set fingerprint (blake3) + anchor count + seed so
// the pre-bind step is off-agent-verifiable. Default-ON kill-switch:
// ELAI_CITATION_PREBIND_DISABLE (not a default-OFF gate; R9).
// Pattern-2 single-variant 2-arm CitationAnchorsBoundKind:
// Bound | Blocked.
// TOTAL_VARIANTS 539 -> 540.
//
// I-39 (SCHEMA_VERSION 230): PlanStateDerived -- per-plan state derived from
// three deterministic sources: wiring_contract_check.py --static verdict,
// SSB journal presence of observable_effect rows, and s-plan-audit-2026-06-04
// FINISH fragments. Emitted by `plan_status_cmd::run_plan_status` (REAL CLI
// entrypoint: `elai plan status`). Flat beacon; no inner Kind enum.
// Carries letter + session_id + derived_state + evidence_anchor for
// off-agent-verifiable, byte-identical output (GD-2). Default-ON;
// no kill-switch (pure read path, never writes to SSB except its own row).
// TOTAL_VARIANTS 540 -> 541.
//
// I-45 (SCHEMA_VERSION 231): ObjectiveSealed -- objective-hash sealed-receipt
// beacon. Pattern-2 single-variant 3-arm ObjectiveSealedKind: Sealed (emitted
// at build_rra when objective_hash stamped into RunArtifact) / MutationDetected
// (emitted when verify_artifact_with_objective detects hash mismatch) /
// ReceiptStale (emitted when verify_attestation detects stale predicate).
// Default-ON; kill-switch ELAI_OBJECTIVE_SEAL_DISABLE. Fail-soft (no writable
// journal = skip emit, never mask the verify verdict). TOTAL_VARIANTS 541 -> 542.
// I-23: calibration_id Option<String> added to ConformalTauComputedPayload +
// ConformalRefusalDecidedPayload. Payload extension only; NO new variant.
// SCHEMA_VERSION 231 -> 232.
// I-15: source Option<String> discriminator added to OracleCacheHitPayload +
// OracleCacheMissPayload. Payload extension only; NO new variant. TV unchanged.
// SCHEMA_VERSION 232 -> 233.
// I-34: GrammarGateEvent -- CRANE hybrid grammar-pin gate beacon. Pattern-2
// single-variant 3-arm GrammarGateEventKind: CranePassed / PinViolation /
// PostParseRejected. Emitted by orchestrator grammar_gate.rs on every constrained
// decode gate evaluation. Default-ON; no kill-switch (gate is mandatory per IC-18 +
// landmine #7/#8). TOTAL_VARIANTS 542 -> 543.
// SCHEMA_VERSION 233 -> 234.
// I-38: QdeltaMeasured -- QDELTA defective-diff rate gate-ON vs gate-OFF flat beacon.
// Safedelta-shaped payload (suite/model_id/rates/delta/CI/seed_count/paired/dollar_per_task/
// contamination_probe_result/surviving_mutants_per_task/corpus_blake3/runpod_pending).
// Emitted by cli::bench_qdelta::run_inner after floor_check passes.
// Default-ON via bench entrypoint (W-2); no ELAI_*_ENABLE gate.
// TOTAL_VARIANTS 543 -> 544.
// SCHEMA_VERSION 234 -> 235.
// I-43: ClaimScopeMeasured -- reproducibility-claim scoper + cheap pre-falsification kill-gate.
// Pattern-2 single-variant 3-arm ClaimScopeMeasuredKind: PilotDelta / CrossLaneDivergence / ClaimScoped.
// Emitted by cli::bench_claim_scope::run_inner for each fixture scenario.
// Default-ON via bench entrypoint (W-2); no ELAI_*_ENABLE gate.
// TOTAL_VARIANTS 544 -> 545.
// SCHEMA_VERSION 235 -> 236.
// I-10: CacheEconomicsEvent -- prefix invariance + TTL tiering + cache-hit hard guard.
// Pattern-2 single-variant 3-arm CacheEconomicsEventKind: TtlPlanned / HitRateGuardTripped /
// PrefixDiffDumped. Emitted by orchestrator::cache_economics (planner + guard).
// Default-ON kill-switch ELAI_CACHE_ECONOMICS_DISABLE=1. Cloud-lane only (ContextPolicy::Cloud).
// Composes on H-1 apply_anthropic_cache_markers + H-2 update_breakpoints / bp.valid.
// TOTAL_VARIANTS 545 -> 546.
// SCHEMA_VERSION 236 -> 237.
// I-8: TrajectoryAborted -- aletheia voluntary abort + dead-end hash pruning + halting bound seam.
// Pattern-2 single-variant 3-arm TrajectoryAbortedKind: Voluntary / DeadEndHash / HaltingBound.
// Emitted by production_agent_rpc.rs TOOL_ACKNOWLEDGE_INFEASIBLE arm (Voluntary),
// dead_end_cache.rs check path after check_and_insert returns true (DeadEndHash),
// and I-17 check-runner watchdog hook (HaltingBound -- Wave-3 stub registered here).
// Composes H-39 AbstentionScored (GD-1: scorer invoked, not re-implemented).
// Composes H-49 timetravel::materialize() rollback (Wave-2: skip+log if no manifest Wave-1).
// TOTAL_VARIANTS 546 -> 547.
// SCHEMA_VERSION 237 -> 238.
// I-1: DirectiveExecuted -- directive supervisor enforcement-plane outcome beacon.
// Pattern-2 single-variant 3-arm DirectiveExecutedKind: Executed / Discarded /
// CompensationFired. Emitted by orchestrator::directive::InProcessSupervisor::
// execute_directive (Executed on clean gate-pass + side-effect-ok, Discarded on
// gate block, CompensationFired on side-effect crash after E-44 saga rollback).
// The generator emits pure Directive structs; the Supervisor is the sole sink.
// Default-ON (no kill-switch; the enforcement plane is mandatory per CANT-WRECK).
// Composes E-30 check_argv + F-1 evaluate_vibegate + G-3 blast_radius (GD-1).
// TOTAL_VARIANTS 547 -> 548.
// SCHEMA_VERSION 238 -> 239.
// I-33: GovernancePrimitiveFired -- mechanical decision governance primitive beacon.
// Pattern-2 single-variant 3-arm GovernancePrimitiveFiredKind: CeflExternalized /
// I6qRejected / CommitRevealSealed. Emitted by decision_governance::emit_governance_fired
// (governed_prompt_decision call site + bench_governance fixture driver).
// Default-ON; kill-switch ELAI_GOVERNANCE_DISABLE (R9 allowlist row required).
// Composes E-25 prompt_decision + D-16 RefusalDecision::Abstain + C-26 DecodingMode::JsonSchema.
// TOTAL_VARIANTS 548 -> 549.
// SCHEMA_VERSION 239 -> 240.
// I-32: RoleContractApplied -- structural role scoping; persona strip/retain/A-B-delta beacon.
// Pattern-2 single-variant 3-arm RoleContractAppliedKind: PersonaStripped / PersonaRetained /
// AbDelta. Emitted by crates/cli/src/report_fns.rs::build_system_prompt_for_role via
// role_contracts::apply_to_system_prompt (PersonaStripped/PersonaRetained) and by the
// --role-ab-probe CLI harness (AbDelta with fixture pass-rate fields).
// Default-ON; grounded per IC-17 (factual/code roles only; alignment-dependent roles retain).
// Composes IC-17 persona-split posture + ID-5 evidence gate + E-6 IsoBudgetMeasured.
// TOTAL_VARIANTS 549 -> 550.
// SCHEMA_VERSION 240 -> 241.
// I-35: ToolSequenceVerdict -- temporal tool-sequence enforcement at the I-1 supervisor plane.
// Pattern-2 single-variant 3-arm ToolSequenceVerdictKind: Allowed / OrderViolation / HaltEnforced.
// Emitted by directive::InProcessSupervisor::run_sequence_gate (the 5th supervisor gate arm).
// Predicates evaluate the ordered tool-call history buffer; CDA-immune (history rows not tokens).
// Default-ON; rule set R8-frozen in crates/rules/seed/tool_sequence_core.toml.
// Composes I-1 supervisor dispatch + E-30 argv + F-3 committed tool sets + Phase 3A FSM masks.
// TOTAL_VARIANTS 550 -> 551.
// SCHEMA_VERSION 241 -> 242.
// I-41: CritiqueLensFinding -- spec-time critique lenses advisory beacon.
// Pattern-2 single-variant 3-arm CritiqueLensFindingKind: AdvisoryFinding / DroppedNoAnchor /
// LensRun. Emitted by critique::lenses::LensSet::run at the post-convergence advisory wire-in
// in idea_framing::run_critic_loop and by the `elai critique` CLI command.
// advisory=true on all arms; ZERO path to any accept/block verdict (§6 ban-LLM-judge hard rule).
// Default-OFF; kill-switch ELAI_CRITIQUE_LENSES_ENABLE (R9, wire-dark-allowlist.toml I-41 row).
// Composes B-40 idea-critic loop + B-38 AmbiguityScorer (L3 human map) + E-45 golden lock (L6 mirror).
// TOTAL_VARIANTS 551 -> 552.
// SCHEMA_VERSION 242 -> 243.
// I-46: BashRedirected -- bash interceptor redirect/fixup/allowed-prefix outcome beacon.
// Pattern-2 single-variant 3-arm BashRedirectedKind: Redirected / FixedUp / PrefixBlocked.
// Redirected: command matched a redirect rule (grep/cat/find/head/tail) -- no process spawn.
// FixedUp: command had a trailing output-pipeline suffix stripped (| head / | tail / 2>&1).
// PrefixBlocked: command starts with an allowed prefix but contains a banned shell token.
// Default-ON; no kill-switch (the interceptor is not a default-OFF experiment).
// Emitted by dispatch::in_process::InProcessDispatch::maybe_emit_bash_redirected when
// run_bash returns Err with error_code bash_redirected / bash_fixedup / bash_prefix_blocked.
// Composes A-26 routing_rules (rules 1+3 retained) + E-30 argv gate + G-3 blast-radius.
// TOTAL_VARIANTS 552 -> 553.
// SCHEMA_VERSION 243 -> 244.
// I-47: PromptHardeningApplied -- verifier-input XML tagging + worker rule injection + SSB beacon.
// Pattern-2 single-variant 3-arm PromptHardeningAppliedKind:
// InputsTagged: auditor/critic dispatch -- untrusted worker output wrapped in <ELAI_DATA> tags
//   with hardening frame prepended above the data blocks.
// RulesInjected: worker dispatch -- frozen per-role Karpathy-distillate rule block appended to
//   CONSTRAINTS section (byte-stable per role; HD-2 cache-prefix safe).
// SelfAttestationStripped: auditor dispatch -- a self-attestation string ("all tests pass",
//   "cargo check green") detected in worker output; labelled as claim-not-evidence inside
//   <ELAI_SELF_ATTESTATION> sub-tag below the main data tag.
// Default-ON; unconditional on all auditor/critic/worker dispatches (no kill-switch, R3 enforced).
// Emitted by tools::execute_agent_with_spawn at the build_agent_prompt seam after hardening applied.
// Composes E-37 PromptArmor (upstream classifier) + H-31/H-32 firewall objective (data-plane leg)
//   + I-32 role contracts (system-prompt sibling seam; zero file overlap confirmed).
// TOTAL_VARIANTS 553 -> 554.
// SCHEMA_VERSION 244 -> 245.
//
// I-2: BranchWorkspaceEvent -- CoW workspace isolation barrier outcome beacon.
// Pattern-2 single-variant 4-arm BranchWorkspaceEventKind:
// Spawned: a private branch workspace root was created (warm git worktree, Tier-1).
// Merged: the deterministic SSB-seq-ordered barrier merged all branch roots into trunk (clean).
// EscapeBlocked: a branch tried to write an absolute path outside its root; blocked before FS
//   mutation (delegates the containment check to elai_safety::blast_radius::check_fs_target).
// ConflictEscalated: two branches wrote the same relative path with differing bytes; Tier-1 halts
//   the merge (no partial write). H-49 materialize() trunk-rollback is the Tier-2 deferred path.
// Emitted by sandbox::branch_workspace::WorktreePoolImpl (spawn/escape_check) + the BSP barrier
//   site (bsp/engine.rs barrier_merge). Composes C-11 sandbox / E-15 pool / I-1 Supervisor / H-49.
// TOTAL_VARIANTS 554 -> 555.
// SCHEMA_VERSION 245 -> 246.
//
// I-5: ContextPruned -- per-branch context rebuild outcome beacon.
// Pattern-2 single-variant 3-arm ContextPrunedKind:
// SiblingExcluded: a sibling branch's SSB payloads were excluded from the fresh branch context
//   (the I-5 pristine-path discipline; ID-1 stateless-generator doctrine).
// TracebackContrasted: a prior failed branch's raw error traceback was replaced by a <=30-token
//   contrastive constraint phrase ("attempt A failed at X -- avoid X"). Preserves causal signal
//   without the attention sink.
// VividnessProbe: a sampled A/B comparison row emitted when umbrella_enabled() is true. Carries
//   outcome_delta (pristine_score - accumulated_score, recomputable under ELAI_SEED, GD-2).
// Emitted by crates/orchestrator/src/pristine_path_filter.rs::PristinePathFilter::build_branch_context.
// Wire-dark until I-4 SearchLadder ships (Wave 4); Tier-1 coverage via contract test.
// Composes H-6 adaptive_reduce (orthogonal: pruning vs window compression) + H-50 regime_policy
//   (pristine rebuild is regime-aware) + E-19 ReasoningQualityProbe (VividnessProbe arm, 6th).
// TOTAL_VARIANTS 555 -> 556.
// SCHEMA_VERSION 246 -> 247.
// I-11: BeliefStateEvent -- typed versioned condensate receipt for cross-model/cross-tier handoffs.
// Pattern-2 single-variant 3-arm BeliefStateEventKind: Condensed / Handoff / HonestyDelta.
// Condensed: emitted at every adaptive_reduce condensation (ID-3 JOURNALED-MIDDLEWARE receipt).
// Handoff: emitted when BeliefState payload is dispatched on the HandoffDecision::Cloud arm.
// HonestyDelta: emitted by HonestyJob::run after >=10 escalations (or --force-honesty-check).
// Default-ON (mandatory ID-3 receipt; no kill-switch). Composes I-7/H-51/H-6/CL-1..CL-4.
// Production callers: task_runner (adaptive_reduce + evaluate_handoff wiring), wired I-11.
// TOTAL_VARIANTS 556 -> 557 (Pattern-2 single-representative: Condensed in lock array).
// SCHEMA_VERSION 247 -> 248.
//
// I-13: MemoryGuardEvent -- write-time memory guard outcome (aho-corasick dual-path defense).
// Pattern-2 single-variant 3-arm MemoryGuardEventKind: PoisonBlocked / DualSerialized / TraceQueryExpanded.
// PoisonBlocked: emitted when MemoryWriteScanner detects invisible-Unicode or confusable-shell pattern.
// DualSerialized: emitted when ToolResultPayload passes through to_llm_content gate.
// TraceQueryExpanded: emitted when trace_query tool returns a line-range expansion.
// Default-OFF (ELAI_MEMORY_GUARD_ENABLE=1, experiment, wire-dark-allowlist.toml row expiry 2026-12-01).
// Production callers: user_memory_capture (CHOKEPOINT-1), memory_cmd::insert (CHOKEPOINT-2),
//   production_agent_rpc::translate_tool_request (TraceQuery arm). Wired I-13.
// TOTAL_VARIANTS 557 -> 558 (Pattern-2 single-representative: PoisonBlocked in lock array).
// SCHEMA_VERSION 248 -> 249.
//
// I-14: ContextTransformReceipt -- deterministic context-compression middleware receipt.
// Pattern-2 single-variant 3-arm ContextTransformKind: LspSlice / AstFold / Tier2Compressed.
// LspSlice: compiler-error payload sliced to the located error neighbourhood (rustc panic).
// AstFold: tree-sitter function-body elision (bodies -> "/* elided */", signatures kept).
// Tier2Compressed: reserved discriminator for the deferred neural (LLMLingua-2-class) tier.
// Default-ON; no kill-switch (ID-3 JOURNALED-MIDDLEWARE is mandatory -- every transform,
// including a graceful skip, journals a reversible receipt; an un-journaled transform is
// a done-gate FAIL). Production caller: AgentRpcServer::transform_tool_output override in
// production_agent_rpc.rs, invoked from ipc_accept.rs run_agent_session at the tool-result
// byte seam. Wired I-14.
// TOTAL_VARIANTS 558 -> 559 (Pattern-2 single-representative: LspSlice in lock array).
// SCHEMA_VERSION 249 -> 250.
//
// I-12 (SCHEMA_VERSION 251): PlaybookEvent -- evidence-gated playbook curator outcome beacon.
// Pattern-2 single-variant 4-arm PlaybookEventKind: Promoted / Demoted / ValidationRun / Exported.
// Promoted: a rule advanced to ACTIVE after >=k SSB-verified outcome links + metric gates pass
//   + C-28 regression suite reports regression_count=0.
// Demoted: an ACTIVE rule lost ACTIVE status (stale or regressed; emitted before RETIRED).
// ValidationRun: the curator ran a promotion-gate check without completing promotion
//   (evidence_count < k, or regression_count > 0).
// Exported: PlaybookCurator::export_verified wrote a verified AGENTS.md / skills export block
//   with embedded outcome_row_ids.
// Default-ON; no kill-switch (the promotion ledger is mandatory per ID-6 WEIGHTS-ARE-CONFIGS).
// Evaluator lives OUTSIDE the write boundary (gate-freeze R8).
// Production callers: (a) offline tick spawn in LiveCli::build_local_knowledge_stack
//   (knowledge_stack.rs, adjacent to cosmos_tick_emit_loop) and (b) skills_cmd::run
//   SkillsAction::PlaybookExport arm (`elai skills-mgmt playbook export`).
// Composes Phase-3F rules lifecycle (transition_to_candidate/shadow/active), IS-16 trial_ledger,
//   C-28 mutation suite, H-17/H-28 holdout-budget gates, IS-13 CompiledPipelineStore.
// TOTAL_VARIANTS 559 -> 560 (Pattern-2 single-representative: Promoted in lock array).
// SCHEMA_VERSION 250 -> 251.
//
// I-17: TriLayerVerdict -- tri-layer evaluation-stack per-branch verdict beacon.
// Pattern-2 single-variant 4-arm TriLayerVerdictKind: HardVeto / ScoredSurvivor /
// DisagreementEscalated / FlakeQuarantined.
// Default-OFF (ELAI_TRI_LAYER_ENABLE, R9 experiment, promotion plan I-4).
// Emitted by verifier_stack::TriLayerStack::run_verdict (one row per branch).
// TOTAL_VARIANTS 560 -> 561.
// SCHEMA_VERSION 251 -> 252.
//
// I-18: RowChainSealed -- per-row rolling blake3 chain beacon (uPIP row-chain + AI-BOM).
// Pattern-2 single-variant 3-arm RowChainSealedKind: ChainAdvanced / AibomEmitted / ChainBroken.
// ChainAdvanced: emitted from run_subcommand::emit_run_artifact after every sealed RRA.
// AibomEmitted: emitted from run_subcommand::emit_attestation when the AI-BOM entry is sealed.
// ChainBroken: emitted from rra::build_rra on Cargo.lock mutation detected mid-run (DEPS layer).
// Default-ON, no kill-switch. Composes G-2/G-8/I-45 (GD-1). DISTINCT from
// crate::verification_result::GateRejection::ChainBroken (Rust error type in verify path).
// TOTAL_VARIANTS 561 -> 562.
// SCHEMA_VERSION 252 -> 253.
pub(crate) const CREATE_SSB_TABLE: &str = "\
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
        'idea_framing_started', 'project_class_classified', 'scope_ceiling_advisory', 'dag_credit_assigned',
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
        -- E-51 (SCHEMA_VERSION 96): edge-inference beacons.
        'mlx_subprocess_spawned', 'mlx_subprocess_killed',
        'llamacpp_subprocess_spawned', 'llamacpp_subprocess_killed',
        'mlx_apple_platform_detected', 'llamacpp_ep_detected',
        'runtime_chain_built',
        -- C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
        -- grc_step_emitted              -- per emit_reasoning call (real PipelineGrc budget)
        -- grounding_sweep_result        -- one row per ground_plan() call (task_runner)
        -- csq_evaluated                 -- one row per generate_csq() result (task_runner)
        -- auditor_rubber_stamp_rejected -- engine.rs InsufficientFindings surfaced at PipelineGrc caller
        -- per_role_decoding_applied     -- C-37 GAP-B close: per-spawn decoding policy emit from agent_pool
        -- hyde_query                    -- one row per HydeStage::hypothesize call
        -- vector_cache_hit_rate         -- aggregate row per VectorCache::flush_hit_rate_ssb
        -- query_trace                   -- per QueryPipeline::query_with_tier completion
        'grc_step_emitted', 'grounding_sweep_result', 'csq_evaluated',
        'auditor_rubber_stamp_rejected', 'per_role_decoding_applied', 'hyde_query',
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
        -- D-4 GAP-2 (SCHEMA_VERSION 108): concurrent-detector row.
        -- tripwire_fired_concurrent -- one row per concurrent-firing tick
        --                            (>=2 detectors in the same observe()
        --                            call) — D-26 consumer surface.
        'tripwire_fired', 'template_auto_reverted', 'tripwire_fired_concurrent',
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
        -- D-5 closure (SCHEMA_VERSION 208): unified promotion-decision summary.
        -- bai_promotion_decision  -- one row per emit_bai_round_with_canary
        --                            decision outcome (template_id, decision,
        --                            stage, evidence_pass, evidence_total,
        --                            audit_id).
        'bai_promotion_decision',
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
        'arc_benchmark_completed',
        -- C-35 (SCHEMA_VERSION 88): tool-attention dynamic gating beacons.
        -- tool_gated           -- emitted per prompt assembly when gate is active.
        -- tool_eager           -- emitted when a tool schema is included eagerly.
        -- tool_lazy_resolved   -- emitted on lazy schema fetch (first reference).
        -- tool_denied          -- emitted on denied tool-call attempt.
        -- All carry: tool_name, fsm_state, prompt_tokens_saved.
        'tool_gated', 'tool_eager', 'tool_lazy_resolved', 'tool_denied',
        'quant_backend_selected', 'kv_cache_dtype_software_sim', 'awq_checkpoint_built',
        -- C-39 (SCHEMA_VERSION 90): CLI-via-MCP dispatch beacons.
        'cli_mcp_dispatch', 'cli_mcp_server_started', 'cli_mcp_server_stopped',
        -- E-15 (SCHEMA_VERSION 91): sandbox hardening beacons.
        'firecracker_restore_slow', 'snapshot_uniqueness_regenerated', 'snapshot_evicted',
        'sandbox_backend_selected', 'agent_forked_from_snapshot', 'researcher_net_egress_blocked',
        -- E-24 (SCHEMA_VERSION 92): MCP hardening beacons.
        'mcp_schema_pin_violation', 'mcp_sanitization_rule_applied', 'mcp_budget_enforced',
        'mcp_legacy_sse_transport', 'mcp_llm_guard_blocked', 'anthropic_gateway_detected',
        -- C-48 (SCHEMA_VERSION 93): SafeHarbor memory-trust beacons.
        'memory_tier_assigned', 'memory_query_filtered', 'safety_critical_blocked',
        -- E-30 (SCHEMA_VERSION 94): public-incident hardening beacons.
        'argv_destructive_denied', 'argv_approval_required', 'trust_dialog_hook_deferred',
        'trust_dialog_env_deferred', 'mcp_pin_version_refused', 'auditor_lineage_collision',
        'untrusted_memory_write_blocked', 'incident_record_created',
        -- E-32 (SCHEMA_VERSION 95): context-window extension beacons.
        -- fitm_calibration_applied   -- FITM positional-prior subtraction applied (prompt > 8K)
        -- deep_mode_activated        -- sidecar booted with Qwen2.5-7B-Instruct-1M at 64K
        -- deep_mode_escalation_chosen -- SessionDag handoff chose local deep mode over cloud
        -- yarn_refused               -- ELAI_USE_YARN=1 refused at boot (quality regression)
        'fitm_calibration_applied', 'deep_mode_activated',
        'deep_mode_escalation_chosen', 'yarn_refused',
        -- E-17 SP2 (SCHEMA_VERSION 97): RAG retrieval-quality beacons.
        'deep_pool_capped', 'rrf_quality_gate_fired',
        'hyde_skipped_code_query', 'rrf_component_weights',
        -- B-3 (SCHEMA_VERSION 98): AntiStalenessDrawer + RulesDrawer beacons.
        'dep_audit_completed', 'baseline_run_completed',
        'deterministic_audit_run', 'rule_transition',
        -- B-4 (SCHEMA_VERSION 99): canary drift snapshot row.
        'canary_drift',
        -- B-7 (SCHEMA_VERSION 100): producer-side wiring closure batch.
        -- tree_position_changed -- DepthFirstNavigator cursor advance
        --                          ({current_node, visited[], blocked[], skipped[]}).
        -- failure_recorded      -- FailureRecord upsert in handle_failure()
        --                          (baby_step_id, step_id, phase_id, concept_id,
        --                           reason, attempts).
        -- package_ref_verified  -- per PackageRef verify_packages outcome
        --                          (name, registry, version, verified).
        -- tech_stack_decided    -- per TechStackDecision creation site
        --                          (component, chosen, alternatives, source).
        'tree_position_changed', 'failure_recorded',
        'package_ref_verified', 'tech_stack_decided',
        -- B-19 (SCHEMA_VERSION 101): per-agent-execution DAIRA marker
        -- aggregation row (counts per phase).
        'marker_report',
        -- B-25 (SCHEMA_VERSION 102): Phase 4J Empire integration. 5 atomic
        -- variants for the empire::bridge::EmpireBridge::emit_action surface.
        -- empire_connected          -- factory-brain handshake completes
        -- empire_disconnected       -- bridge transitions to Disconnected
        -- empire_event_received     -- Empire-side event surfaces on the bus
        -- empire_capability_invoked -- ElaiNeuron capability dispatch attempt
        -- empire_repair_proposed    -- repair pipeline surfaces a proposal
        'empire_connected', 'empire_disconnected', 'empire_event_received',
        'empire_capability_invoked', 'empire_repair_proposed',
        -- C-5 follow-on (SCHEMA_VERSION 103): cloud-mode hard ceiling +
        -- warn-at-percent + pre-task cost estimate.
        -- route_blocked         -- CloudModeRouter returned cap-exceeded error
        -- warn_limit_reached    -- spend crossed warn_at_percent of daily cap
        -- cost_estimate         -- pre-task USD projection from plan skeleton
        'route_blocked', 'warn_limit_reached', 'cost_estimate',
        -- C-36 (SCHEMA_VERSION 104): MuCoCo mutation operator catalog. One
        -- row per dispatch_mucoco_mutation result (applied or skipped).
        -- Producer: orchestrator::metamorphic_testing::dispatch_mucoco_mutation
        -- + downstream metamorphic_runner auditor wiring.
        'mucoco_mutation_applied',
        -- D-3 closure (SCHEMA_VERSION 105): PpmUnavailable (D-2 deferred,
        -- carried atomically here) + VerifierDisagreement (D-4 tripwire
        -- input) + CrossContextDispatched (cross_context::run emission).
        -- Producers: PpmClient health path + agent_pool::run_consensus_pass
        -- observability hook + crate::cross_context call sites.
        'ppm_unavailable', 'verifier_disagreement',
        'cross_context_dispatched',
        -- C-38 (SCHEMA_VERSION 107): pagerank production observability +
        -- dkg rebuild lifecycle. Atomic 3-variant landing.
        -- pagerank_run          -- one row per Stage 2 compute_repo_map call
        --                          (workspace_root_hash, node_count,
        --                           top_node_hash, elapsed_ms).
        -- dkg_rebuild_started   -- one row when ensure_dkg_or_rebuild
        --                          begins a rebuild (workspace_root_hash,
        --                          trigger_reason).
        -- dkg_rebuild_complete  -- one row at rebuild end, success OR fail
        --                          (workspace_root_hash, success,
        --                           duration_ms, node_count).
        'pagerank_run', 'dkg_rebuild_started', 'dkg_rebuild_complete',
        -- C-40 (SCHEMA_VERSION 109): GitHub PR adapter producer surface.
        -- All 4 rows are emitted by the orchestrator-side
        -- `crate::github_pr::GhCliAdapter` after every public adapter
        -- method call (success or structured error). Sensitive fields are
        -- blake3-prefix hashed so dashboards group by identity without
        -- leaking raw URLs / SHAs.
        --   github_pr_opened          -- one row per open() Ok
        --   github_pr_status_fetched  -- one row per status() Ok
        --   github_pr_closed          -- one row per close() Ok
        --   github_pr_merged          -- one row per merge() Ok
        'github_pr_opened', 'github_pr_status_fetched',
        'github_pr_closed', 'github_pr_merged',
        -- E-41 (SCHEMA_VERSION 135): GitHub PR review surface. 5 atomic rows
        -- emitted by crate::github_pr::{review,comment_watcher,package_verify,
        -- semgrep_gate,auto_merge_guard,git_assist} for review composition,
        -- new-comment polling, slopsquat dep verification, semgrep gate
        -- results, and auto-merge blocks. Sensitive fields hashed via blake3.
        --   pr_review_composed         -- one row per post_review_comment() Ok
        --   pr_comment_watched         -- one row per watch_comments() Ok
        --   pr_package_verify_result   -- one row per verify_pr_deps_no_slopsquat() Ok
        --   pr_semgrep_gate_result     -- one row per run_semgrep_gate() Ok
        --   pr_auto_merge_blocked      -- one row per assess_merge_safety()->Block
        'pr_review_composed', 'pr_comment_watched',
        'pr_package_verify_result', 'pr_semgrep_gate_result',
        'pr_auto_merge_blocked',
        -- D-5 closure (SCHEMA_VERSION 110): per-pull Thompson-sample beacon
        -- for the BAI promotion round. Captures the Beta(alpha, beta) draw
        -- that the Thompson policy used to pick the next arm. Emitted by
        -- `crate::bai::emit_bai_round` immediately before the matching
        -- `bai_arm_updated` row so consumers can join the sample, the
        -- posterior update, and the (optional) promote/reject decision in
        -- one ordered triple per round.
        'thompson_sampled',
        -- C-41 (SCHEMA_VERSION 111): skills-driven workflow registry.
        -- `skill_triggered` is emitted on every registry lookup that
        -- returns a non-empty top-k; `skill_executed` is emitted when the
        -- harness Skill tool actually dispatches the resolved skill (the
        -- two rows pair like the existing `tool_dispatch` / `tool_result`
        -- pattern). `skill_name_hash` is a blake3-prefix u64 of the skill
        -- name so dashboards can group by skill identity without
        -- surfacing the raw string. See `crate::skill_registry` and
        -- workflow/papers/scientific-workflow-interface.md.
        'skill_triggered', 'skill_executed',
        -- D-6 closure (SCHEMA_VERSION 112): held-out promotion-gate +
        -- refresh-event beacons. Lands atomically with the held-out
        -- closure pass (auto-retire / regression-gate / paper-grounded
        -- temporal date-windowing). See workflow/reasoning/D-6-decisions.md.
        --   held_out_promotion_gate -- one row per `held_out::
        --                              gate_promotion_via_held_out` call
        --                              (task_id, allow, delta,
        --                              regression_threshold, passes,
        --                              fails, reason).
        --   held_out_refresh        -- one row per auto-refresh tick that
        --                              emits an add+retire pair (source_
        --                              label, added_count, retired_count,
        --                              ts_ms).
        'held_out_promotion_gate', 'held_out_refresh',
        -- C-42 (SCHEMA_VERSION 113): Semantic Gateway L2 zero-trust RBAC
        -- beacons. Producer is plugins::mcp_bridge::DefaultMcpBridge::
        -- dispatch_tool_call_with_authz. ELAI design (paper does not
        -- define event-bus contract). See arXiv:2604.25555 +
        -- workflow/papers/semantic-gateway-fuzzer.md.
        --   gateway_policy_allowed -- composed-Allow row (RBAC + gate
        --                             both permit).
        --   gateway_policy_denied  -- composed-Deny row; deny_reason_
        --                             label is one of authz / tool_gate.
        --   gateway_hitl_pending   -- L3 Crypto-HITL beacon (L3 DEFERRED-
        --                             TO-FOLLOWUP; no session 2 producer).
        'gateway_policy_allowed', 'gateway_policy_denied',
        'gateway_hitl_pending',
        -- D-7 closure (SCHEMA_VERSION 114): cross-domain transfer canary
        -- observation beacons. Producer is `canary::emit_canary_run` per
        -- family; the BAI promotion gate consumes the
        -- `cross_domain_regression` rows via `recent_canary_status`.
        -- ELAI design hypothesis — NanoFlux (arXiv:2509.23252) provides
        -- empirical motivation only; no canary mechanism in the paper.
        --   cross_domain_regression  -- one row per per-family regression
        --                               detection (source_domain,
        --                               regressed_domain, delta_pct,
        --                               threshold_pct).
        --   canary_family_pass       -- one row per clean-pass family
        --                               (family, score_pct, sample_count).
        --   canary_insufficient_data -- one row per under-sampled family
        --                               (family, sample_count,
        --                               min_required).
        'cross_domain_regression', 'canary_family_pass',
        'canary_insufficient_data',
        -- D-8 closure (SCHEMA_VERSION 115): cryptographic archive verify
        -- result row. Producer is `archive_verify::run` (CLI driver).
        -- One row per `elai archive verify` invocation; payload carries
        -- audit_id, total_entries, verified flag, error_count, and the
        -- typed per-entry error list (Vec<ArchiveVerifyError>). See
        -- workflow/reasoning/D-8-decisions.md.
        'archive_verify_result',
        -- C-43 closure (SCHEMA_VERSION 116): Phase 4H scope-freeze
        -- ratification + violation beacons. Producers are the
        -- `elai freeze ratify` / `elai freeze verify` CLI driver and
        -- the dashboard's scope_freeze_status Tauri command. ELAI
        -- design hypothesis — no paper grounding; see
        -- workflow/sessions/s-25120/mistakes.md for the bad-cite
        -- analysis of arXiv:2604.21725.
        'scope_freeze_ratified', 'scope_freeze_violated',
        -- C-44 (SCHEMA_VERSION 117): Cursor Composer 2 cost-accrual beacon.
        -- Producer is `RuntimeOrchestrator::try_complete` after a Composer 2
        -- adapter invocation; payload carries hashed backend/role + token
        -- counts + est cost micros. ELAI design hypothesis — composer-2
        -- paper grounds model identity only; integration is source-grounded
        -- against @cursor/sdk (see workflow/papers/composer2-interface.md).
        'backend_cost_accrued',
        -- C-45 (SCHEMA_VERSION 120): CANTANTE contrastive credit-attribution
        -- beacons. Three variants emitted by the credit_attribution harness:
        --   * agent_role_dropped — emit-before-reroute audit trail when a
        --     role is gated out of the production agent-spawn path.
        --   * credit_attribution_run — once per harness invocation at setup,
        --     carries (run_id_hash, dropped_role_name_hash, K, g).
        --   * credit_attribution_result — once per harness invocation at
        --     aggregation, carries (run_id_hash, per_role_scores_hash,
        --     total_roles).
        -- Paper-grounded: arXiv:2605.13295 (Zehle 2026, Section 3.2). See
        -- workflow/papers/cantante-credit-attribution.md.
        'agent_role_dropped',
        'credit_attribution_run',
        'credit_attribution_result',
        -- C-46 (SCHEMA_VERSION 121): Uno-Orchestra selective-delegation
        -- beacons. Three variants emitted by the routing module:
        --   * selective_delegation_decision — once per SelectiveDelegator
        --     dispatch call. Carries (task_id_hash, difficulty, alpha,
        --     dispatched_count).
        --   * selective_delegation_escalated — once per audit-bubble or
        --     verifier-driven escalation. Carries (task_id_hash,
        --     reason_hash, escalated_workers).
        --   * selective_delegation_cost_saved — once per Decision when the
        --     policy dropped workers. Carries (task_id_hash, skipped_count,
        --     est_savings_micros).
        -- Paper-grounded: arXiv:2605.05007 (Cui et al 2026, Uno-Orchestra
        -- Sections 5 + 6.1). See workflow/papers/uno-orchestra-interface.md.
        'selective_delegation_decision',
        'selective_delegation_escalated',
        'selective_delegation_cost_saved',
        -- C-47 (SCHEMA_VERSION 122): Case-Based Calibration of Tool-Use.
        -- Three variants emitted by the plugins-side
        -- DefaultMcpBridge::dispatch_tool_call_with_gate +
        -- ::dispatch_tool_call_with_authz at the shared
        -- consult_calibration + record_call helpers:
        --   * tool_call_logged              — one row per completed tool
        --                                     dispatch (post-call audit
        --                                     trail). Carries hashed
        --                                     tool/shape/response/audit
        --                                     fields per the
        --                                     ToolCallLoggedPayload.
        --   * tool_calibration_hint_emitted — one row whenever the case
        --                                     base crosses the
        --                                     HINT_ERROR_RATE_THRESHOLD
        --                                     +HINT_MIN_TOTAL_CALLS gate.
        --                                     Hint text is hashed; raw
        --                                     text stays in the
        --                                     conversation loop's
        --                                     pre-prompt vehicle.
        --   * tool_calibration_drift_detected — one row when a tool's
        --                                       per-tool aggregate
        --                                       error_rate crosses the
        --                                       drift threshold.
        -- Paper-grounded: arXiv:2605.15041 (CAST) — see
        -- workflow/papers/case-based-calibration.md.
        'tool_call_logged', 'tool_calibration_hint_emitted',
        'tool_calibration_drift_detected',
        -- C-49 (SCHEMA_VERSION 123): iterative-distillation experience
        -- module. ELAI heuristic adaptation of arXiv:2605.14443 (RL
        -- prompter is RUNPOD-PENDING). Three variants emitted by
        -- `crate::experience::distill_fragments` + the post-audit hook:
        --   * prompt_policy_distillation_run — once per distill call.
        --     Carries (skill_name_hash, trace_count,
        --     fragments_emitted, ts_ms).
        --   * prompt_fragment_selected — one row per fragment included
        --     in the distilled output. Carries (skill_name_hash,
        --     fragment_hash, success_rate, freq, ts_ms).
        --   * prompt_fragment_discarded — one row per fragment rejected
        --     by the distillation thresholds. Carries
        --     (skill_name_hash, fragment_hash, reason_hash, ts_ms).
        -- See workflow/papers/iterative-distillation-interface.md.
        'prompt_policy_distillation_run',
        'prompt_fragment_selected',
        'prompt_fragment_discarded',
        -- D-14 (SCHEMA_VERSION 124): sycophancy-probe row pair.
        -- Sharma et al arXiv:2310.13548 §3.2 inverted-framing detector.
        -- Defensive flag-only: preference models do NOT reliably fix
        -- sycophancy (§4 — 75% PM residual, 25% oracle floor).
        --   * sycophancy_probe_dispatched — once per
        --     `dispatch_sycophancy_probe_hook` invocation. Carries
        --     (claim_id, probe_kind, user_message).
        --   * sycophancy_flag — once per probe verdict. Carries
        --     (claim_id, flip_detected, delta, verdict).
        -- See workflow/papers/sharma-sycophancy-2310.13548-interface.md.
        'sycophancy_probe_dispatched',
        'sycophancy_flag',
        -- D-23 (SCHEMA_VERSION 142): sycophancy detection benchmark.
        -- sycophancy_bench_run_completed -- one row per
        --     `SycophancyBenchmark::run` Ok return. Carries (tp, fp,
        --     tn, fn_, inconclusive, detection_rate, fp_rate, verdict,
        --     total_fixtures, fixture_blake3, runner_name).
        -- Gates detection_rate >= 0.70 AND fp_rate <= 0.15 per Sharma
        -- §4 single-turn reproducibility band + plan §Tasks step 3.
        'sycophancy_bench_run_completed',
        -- D-24 (SCHEMA_VERSION 143): cross-domain canary regression
        -- benchmark. Replays a labelled fixture corpus through D-7
        -- run_canary in-memory and aggregates a 4-bucket confusion
        -- matrix (true_block / false_block / true_pass / false_pass).
        -- canary_bench_run_completed -- one row per CanaryBenchmark::run
        --     Ok return. Carries (true_block, false_block, true_pass,
        --     false_pass, harmful_blocked, helpful_blocked,
        --     neutral_blocked, total_fixtures, verdict, fixture_blake3,
        --     runner_name). Gates harmful_blocked >= 20 AND
        --     helpful_blocked <= 5 AND neutral_blocked <= 2 (ELAI
        --     calibration; NanoFlux arXiv:2509.23252 paper has no canary
        --     mechanism so the thresholds are empirical).
        'canary_bench_run_completed',
        -- D-25 (SCHEMA_VERSION 144): determinism + archive tamper-suite
        -- benchmark. Drives D-8 verify_archive IN-PROCESS across a 10-
        -- scenario JSON-manifest matrix (positive control + 6 single-
        -- error variants spanning the 4 ArchiveVerifyError kinds +
        -- multi-tamper + revert + fail-fast). Each scenario MUST be
        -- detected within 5000ms.
        -- determinism_tamper_suite_completed -- one row per
        --     TamperBenchmark::run Ok return. Carries (total_scenarios,
        --     passed, failed, max_wall_ms, verdict, scenarios_blake3).
        --     Gates verdict == pass AND max_wall_ms <= 5000. Supports
        --     arxiv:2507.05619 (T1) integrity-checksum recommendation
        --     (motivation only -- no tamper-suite mechanism in the
        --     paper). See workflow/reasoning/D-25-decisions.md.
        'determinism_tamper_suite_completed',
        -- D-28 (SCHEMA_VERSION 145): adversarial example farming run.
        -- Host-side scaffolding for the Adv-RM (arXiv:2504.06141) farm
        -- loop. Walks a dataset through the D-2 PPM hook, applies the
        -- Z>1.96 dual-filter, writes corpus.jsonl with gold_verdict=
        -- pending, and records the paper-grounded PRMBench v1/v2 delta
        -- floor (1.0pp recalibrated from plan's ungrounded 5pp).
        -- adversarial_farm_run_completed -- one row per
        --     AdvRmFarmer::run Ok return. Carries (samples_farmed,
        --     rounds, prmbench_delta_pp, verdict, paper_floor_pp,
        --     corpus_blake3). Gates prmbench_delta_pp >=
        --     paper_floor_pp (default 1.0). Live PPM loop + bench
        --     re-runs are RUNPOD-PENDING. See workflow/reasoning/
        --     D-28-decisions.md DECISION-1.
        'adversarial_farm_run_completed',
        -- D-29 (SCHEMA_VERSION 146): hierarchical BAI round
        -- completion. Emitted ONLY when ELAI_BAI_MODE=hierarchical
        -- AND ELAI_BAI_ENABLE=1 (wire-dark default). Paper labels:
        -- arXiv:2111.06929 SUPPORTING EVIDENCE (regret min, NOT BAI;
        -- workflow/papers/hierarchical-bayesian-bandits-interface.md);
        -- arXiv:2106.10417 DIRECT GROUNDING (variance-dependent BAI).
        -- hierarchical_bai_round_completed -- one row per
        --     `dispatch_bai_round` call in hierarchical mode. Carries
        --     (round, mu / kappa posterior mean+variance, per-arm
        --     shrunk Beta summary list, selected_arm_index,
        --     selection_basis, gibbs_iterations, canary_gate_passed).
        --     Plan-doc 30-50pct regret reduction claim is UNGROUNDED
        --     for BAI; live baseline RUNPOD-PENDING. See
        --     workflow/reasoning/D-29-decisions.md DECISION-1.
        'hierarchical_bai_round_completed',
        -- D-30 (SCHEMA_VERSION 147): calibration bootstrap warm-start
        -- completion. Emitted ONLY when
        -- ELAI_CALIBRATION_BOOTSTRAP_ENABLE=1 (wire-dark default OFF).
        -- Paper labels: arXiv:2509.01455 UniCR COMPOSITION (consumes
        -- compute_tau_with_min); arXiv:2109.07958 TruthfulQA DATA ORIGIN
        -- (corpus fixture source); Politis & Romano 1994 JASA Vol 89 §3
        -- DIRECT GROUNDING (stationary bootstrap — only active when
        -- SECONDARY runs).
        -- calibration_bootstrap_completed -- one row per
        --     warm_start_tau call. Carries (corpus_slug, corpus_size,
        --     corpus_blake3, tau_warm_start, alpha, tau_floor_active,
        --     Option<bootstrap_replicates>, Option<bootstrap_ci_low>,
        --     Option<bootstrap_ci_high>, Option<block_length_p>, ts_ms).
        --     Single-variant by D-19/D-23/D-24/D-25/D-28/D-29 precedent
        --     (11th application). See workflow/reasoning/
        --     D-30-decisions.md DECISION-1.
        'calibration_bootstrap_completed',
        -- C-50 (SCHEMA_VERSION 125): Contract-Coding STRUCTURAL
        -- contract test scaffolding. arXiv:2604.13100 (Lin, Zhao,
        -- Shi 2026). The paper's kernel is K = <N, Sigma, Delta>
        -- (Section 4.2); the full RL Generator -> Discriminator
        -- pipeline is RUNPOD-PENDING. These two rows surface the
        -- heuristic scaffolder + discriminator producers only.
        --   * contract_test_scaffolded — once per
        --     `scaffold_contract_test_from_signature` invocation.
        --     Carries (fn_name_hash, contract_kind_hash, ts_ms).
        --   * contract_violation_detected — once per heuristic
        --     `audit_against_contract` verdict that surfaces a
        --     kernel violation. Carries (fn_name_hash,
        --     violation_kind_hash, severity, ts_ms). severity is
        --     `u8` so the dashboard can sort without a 2nd
        --     enum-decode hop.
        -- Citation discipline: the headline 47% is the Roguelike
        -- subset of Greenfield-5 (Table 1) and TIES Trae; the
        -- differentiator is structural integrity per Section 5.2.
        -- See workflow/papers/contract-coding-interface.md.
        'contract_test_scaffolded',
        'contract_violation_detected',
        -- C-52 (SCHEMA_VERSION 126): Dafny Formal Verification Spike
        -- host-side scaffolding tied to the new `crate::dafny_verify`
        -- module (arXiv:2604.22601 NL2VC). Single beacon row.
        --   * dafny_verify_completed — once per heal-loop termination
        --     (initial success OR self-healing exhaustion). Carries
        --     (tier_hash blake3-prefix of tier wire tag,
        --     dafny_pass bool, udebug_pass bool, heal_iter u32,
        --     ts_ms u64). The `udebug_pass` field is the vacuous-
        --     verification guard per paper §3 — raw `dafny verify`
        --     exit-zero is INSUFFICIENT.
        -- Citation discipline: paper's top performer is Gemma 4-31B
        -- which ELAI is BLOCKED from integrating per memory rule
        -- feedback_no_gemma4. ELAI substitutes from the paper's
        -- evaluation set (Codestral-22B-v0.1, GPT-OSS-120B, Qwen 3
        -- Coder 30B). The current ELAI sidecar default (Qwen 2.5
        -- Coder 7B) is NOT in the paper's evaluation set. Live
        -- execution is RUNPOD-PENDING. See
        -- workflow/papers/nl2vc-dafny-interface.md +
        -- workflow/reasoning/C-52-decisions.md.
        'dafny_verify_completed',
        -- D-16 closure (SCHEMA_VERSION 127): durable tau audit trail.
        -- One row per `TauUpdater::update_tau` persistence step.
        -- Carries (corpus_size u32, new_tau f64, alpha f64,
        -- source String). Paper grounding arXiv:2509.01455 UniCR
        -- Eq.5 CRC quantile formula. `alpha` (miscoverage rate)
        -- is DISTINCT from `rho` (selective risk budget) per
        -- D-16-decisions.md DECISION-4 — both fields are required
        -- in `[conformal]` toml but only `alpha` is on the SSB row
        -- (rho is decision-time only).
        'calibration_corpus_updated',
        -- E-12 Tier-B (SCHEMA_VERSION 128): production fullstack
        -- codegen — framework profile selection + legacy linter
        -- config strip. Two-variant atomic bump. Wire-dark posture:
        -- emitted today by the heuristic scope-planner shim; live
        -- production checklist invocation against representative
        -- apps is RUNPOD-PENDING per
        -- workflow/reasoning/E-12-tier-b-decisions.md.
        --   * framework_profile_selected — emitted once per task on
        --     scope-planner exit when ProjectClass resolves to a
        --     FrameworkProfile via PROJECT_CLASS_TO_FRAMEWORK.
        --     Carries (profile_name_hash u64, version_pin_hash u64,
        --     language_hash u64, ts_ms u64). All hashes are
        --     blake3-prefix 64 bits (anti-stale: NEVER sha2).
        --   * legacy_config_stripped — emitted once per legacy
        --     linter config file removed by
        --     FrameworkAdapter::apply_lint_defaults (.eslintrc,
        --     .prettierrc, .flake8, etc). Carries (config_path_hash
        --     u64, lint_engine_hash u64, ts_ms u64).
        'framework_profile_selected',
        'legacy_config_stripped',
        -- D-18 closure (SCHEMA_VERSION 129): typed-ObservableEffect
        -- 4-variant pack deferred from the SCHEMA 84 pass.
        -- observable_effect_dispatched -- one row per
        --   `dispatch_observable_effect_hook` invocation (gated on
        --   ELAI_OBSERVABLE_EFFECT_ENABLE). Carries effect_id, trigger,
        --   context_kind (sandbox|production), ts_ms.
        -- goodhart_risk_flag -- emitted once per
        --   `check_goodhart_risk` flag against an EffectSet. Per
        --   Eureka arxiv:2310.12931 §3.3 component-variation guard.
        -- sandbox_observation -- one row per Coscientist-style
        --   SandboxObservation round-trip. Carries effect_id, tier,
        --   outcome, evidence_count, ts_ms.
        -- production_observation_proposal -- one row per
        --   Coscientist-style ProductionObservation proposal.
        --   `requires_human_review=true` MUST hold when
        --   instrument_file is None (Coscientist §4 invariant).
        'observable_effect_dispatched',
        'goodhart_risk_flag',
        'sandbox_observation',
        'production_observation_proposal',
        -- E-13 (SCHEMA_VERSION 130): TDD verifier-agent beacons.
        -- tdd_cycle_started -- emitted once per
        --   `TddVerifierAgent::run_cycle` entry. Carries
        --   task_id_hash, ts_ms.
        -- tdd_test_failed -- emitted once per Red-phase test
        --   failure observed by the agent driver. Carries
        --   task_id_hash, test_name_hash, error_kind_hash, ts_ms.
        -- tdd_test_passed -- emitted once per Red->Green flip.
        --   Carries task_id_hash, test_name_hash, optional
        --   mutation_score (None when cargo-mutants did not run),
        --   ts_ms.
        'tdd_cycle_started',
        'tdd_test_failed',
        'tdd_test_passed',
        -- Tranche B-7 triple (SCHEMA_VERSION 195): TDD verification gate
        -- substrate covering E-13 PropertyGatePass / MutationGateSurvivorBlock /
        -- AntiMockBlock plus E-12 AssertionOnlyDiffBlock arm. Pattern-2
        -- (46th application). Wire-dark unless
        -- ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE=1.
        'tdd_verification_event',
        -- E-14 (SCHEMA_VERSION 131): browser-e2e agent beacons.
        -- browser_e2e_run_started -- once per PlaywrightRunner journey
        --   run entry. Carries journey_name_hash, url_hash, ts_ms.
        -- browser_e2e_assertion_failed / passed -- per assertion outcome.
        --   Carries journey_name_hash, assertion_hash[, error_hash], ts_ms.
        -- browser_e2e_snapshot_captured -- per Playwright snapshot
        --   artifact. Carries journey_name_hash, snapshot_kind_hash, ts_ms.
        'browser_e2e_run_started',
        'browser_e2e_assertion_failed',
        'browser_e2e_assertion_passed',
        'browser_e2e_snapshot_captured',
        -- E-18 (SCHEMA_VERSION 132): tool-use reliability gate beacons.
        -- Composes on D-11 RegistryCheck + E-35 SearchReplaceMatcher +
        -- C-47 ToolCalibrationRegistry + C-42 AuthzPolicy + C-35 ToolGate.
        -- tool_reliability_gate_passed -- one row per Pass verdict from
        --   tool_reliability::ToolReliabilityGate::check. Carries
        --   tool_name_hash, call_shape_hash, ts_ms.
        -- tool_reliability_gate_rejected -- one row per Reject verdict.
        --   Carries tool_name_hash, reject_reason_hash (snake_case label),
        --   ts_ms.
        -- tool_call_spilled -- one row per body > inline cap landing at
        --   .elai_cc/.elai_cc/spill/<ts>-<hash>.json. Carries
        --   tool_name_hash, spill_path_hash, ts_ms.
        -- tool_call_retry_escalated -- one row per RetryDecision::Escalate
        --   verdict from tool_reliability::should_escalate. Carries
        --   tool_name_hash, attempt, escalate_to_role_hash, ts_ms.
        'tool_reliability_gate_passed',
        'tool_reliability_gate_rejected',
        'tool_call_spilled',
        'tool_call_retry_escalated',
        -- E-37 (SCHEMA_VERSION 133): prompt-injection defense beacon family.
        -- 9 atomic variants — layers on E-24 mcp_schema_pin / sanitize_result
        -- and E-30 destructive_verb_allowlist. Paper grounding:
        -- arXiv:2403.14720 Spotlight + arXiv:2507.15219 PromptArmor +
        -- arXiv:2503.18813 CaMeL + arXiv:2406.13352 AgentDojo +
        -- arXiv:2502.06703 cheap monitors.
        'injection_attempt_detected',
        'injection_attempt_blocked',
        'spotlight_wrapped',
        'spotlight_breached',
        'tool_description_pin_mismatch',
        'capability_gate_rejected',
        'provenance_tagged',
        'prompt_armor_verdict',
        'injection_fixture_replayed',
        'output_classifier_flagged',
        -- E-40 (SCHEMA_VERSION 134): orchestrator-rooted symbol-graph beacon family.
        -- 3 atomic variants. Paper grounding deferred per
        -- workflow/sessions/s-25120/mistakes.md cluster #6 (Stack Graphs +
        -- GraphCoder + CodexGraph + RANGER + Glean Angle).
        -- symbol_graph_queried -- one row per find_defs/refs/callers/callees.
        --   Carries query_kind_hash, symbol_hash, result_count, ts_ms.
        -- symbol_graph_cache_loaded -- one row per SymbolGraphCache::load
        --   mount. Carries graph_path_hash, node_count, ts_ms.
        -- lsp_oracle_consulted -- one row per oracle_lookup invocation.
        --   On-demand-only — >5/session is a bug per E-40-decisions.md.
        --   Carries language_hash, symbol_hash, oracle_kind_hash, ts_ms.
        --
        -- E-44 (SCHEMA_VERSION 136): durability layer beacons (atomic +5).
        -- checkpoint_written -- one row per CheckpointManager::write success.
        --   Carries session_id_hash, schema_version, file_count, ts_ms.
        -- checkpoint_resumed -- one row per successful resume.
        --   Carries session_id_hash, schema_version, ts_ms.
        -- checkpoint_migrated -- one row per migrate_checkpoint crossing
        --   at least one schema boundary. Carries from_version, to_version,
        --   transform_count, ts_ms.
        -- idempotent_dispatch_skipped -- one row per IdempotentDispatcher
        --   cache hit. Carries call_shape_hash, cached_result_hash, ts_ms.
        -- durability_lock_blocked -- one row per CompactLock::try_acquire
        --   returning Contended. Carries lock_path_hash, holder_hash, ts_ms.
        --
        -- E-45 (SCHEMA_VERSION 137): cross-language drift detector
        -- beacons (atomic +3). Producers: cross_lang_drift::detector.
        -- cross_lang_drift_detected -- one row per detect_pair() returning
        --   >= 1 DriftItem. Carries pair_kind_hash, drift_count, severity_max,
        --   ts_ms.
        -- cross_lang_pair_scanned -- one row per detect_pairs() repo walk.
        --   Carries repo_root_hash, pair_count, ts_ms.
        -- schema_version_mismatch -- one row when rust_schema != python_schema
        --   on the orchestrator <-> sidecar boundary. Carries rust_schema,
        --   python_schema, ts_ms.
        'symbol_graph_queried',
        'symbol_graph_cache_loaded',
        'lsp_oracle_consulted',
        'checkpoint_written',
        'checkpoint_resumed',
        'checkpoint_migrated',
        'idempotent_dispatch_skipped',
        'durability_lock_blocked',
        'cross_lang_drift_detected',
        'cross_lang_pair_scanned',
        'schema_version_mismatch',
        -- E-49 (SCHEMA_VERSION 138): coding-agent metric suite (scope-lean).
        -- coding_agent_metrics_captured -- one row per
        --   MetricsCollector::capture_metrics. Carries session_id_hash,
        --   pass_at_1_x10000 (int representation of pass@1 * 10_000),
        --   refine_rate_x10000, cost_per_task_cents (Option<u32>), ts_ms.
        -- swe_bench_pro_run_started -- one row when run_swebench_pro
        --   begins a live run. Carries task_pool_version_hash,
        --   model_id_hash, max_tasks (Option<u32>), ts_ms.
        -- swe_bench_pro_run_completed -- one row at the end of a Pro
        --   live run. Carries task_pool_version_hash, model_id_hash,
        --   tasks_attempted, tasks_solved, pass_at_1_x10000, ts_ms.
        -- refinebench_score_emitted -- one row per RefineBench scoring
        --   pass. Carries suite_version_hash, model_id_hash,
        --   refine_rate_x10000, ts_ms.
        -- metrics_aggregation_run -- one row per aggregator invocation.
        --   Carries session_count, pass_at_1_avg_x10000,
        --   refine_rate_avg_x10000, ts_ms.
        'coding_agent_metrics_captured',
        'swe_bench_pro_run_started',
        'swe_bench_pro_run_completed',
        'refinebench_score_emitted',
        'metrics_aggregation_run',
        -- E-52 Tier-B (SCHEMA_VERSION 139): LSP-as-verifier diagnostic-probe
        -- + SCIP refresh beacons. 4 atomic variants.
        -- diagnostic_probe_dispatched -- one row per language-grouped
        --   PostToolUse probe dispatch. Carries probe_kind_hash,
        --   file_count, language_hash, ts_ms.
        -- diagnostic_probe_completed -- one row per probe completion
        --   regardless of outcome. Carries probe_kind_hash, run_status_hash
        --   (Completed|TimedOut|ToolMissing), error_count, duration_ms,
        --   ts_ms.
        -- diagnostic_probe_block -- one row when error-severity
        --   diagnostics blocked the EXECUTING -> AUDITING transition.
        --   Carries probe_kind_hash, error_count, first_path_hash, ts_ms.
        -- scip_refresh_completed -- one row per .claude/tools/elai.map
        --   incremental SCIP merge invocation. Carries shard_count,
        --   crate_count, duration_ms, ts_ms.
        'diagnostic_probe_dispatched',
        'diagnostic_probe_completed',
        'diagnostic_probe_block',
        'scip_refresh_completed',
        -- E-53 (SCHEMA_VERSION 140): formal-verify pipeline boundary beacon.
        -- formal_verifier_completed -- one row per
        --   `crate::formal_verify::orchestrator::run` pipeline boundary.
        --   Carries verifier_kind_hash (z3_smt|dafny|kani|verus),
        --   outcome_hash (verified|refuted|inapplicable|runpod_pending),
        --   csq_class_hash (counterfactual_risky|unknown|safe),
        --   contract_emitted (bool), duration_ms, ts_ms. Single atomic
        --   +1 bump (139 -> 140) consolidating the plan's originally-
        --   proposed 13 variants per the C-52 DafnyVerifyCompleted
        --   single-variant precedent.
        'formal_verifier_completed',
        -- E-35 (SCHEMA_VERSION 141): structured-output substrate (XGrammar +
        -- INSTRUCTOR + CodeAct + SEARCH/REPLACE + cloud strict-schema)
        -- atomic 10-variant bump. Producers split across the sidecar (Python
        -- side; RUNPOD-PENDING for XGrammar wiring) and the host (Rust
        -- codeact_sandbox dispatcher + CloudGroundingClient strict-mode +
        -- CLI subverb `elai structured-output ...`). AgentRole::Runner
        -- variant addition DEFERRED per E41_AGENTROLE_FROZEN_AT_17
        -- gotcha (230 match-arm cascade); CodeAct runs through existing
        -- role + RuntimeBackend composition.
        --
        -- xgrammar_mask_applied            -- one row per sidecar XGrammar
        --   mask application; carries schema_id_hash, mask_applied_count,
        --   generation_ms, cache_hit, ts_ms.
        -- instructor_mode_activated        -- one row per INSTRUCTOR-mode
        --   dispatch (reasoning roles per CRANE).
        -- grammar_mode_activated           -- one row per GRAMMAR-mode
        --   dispatch (low-reasoning enum/header blocks).
        -- codeact_action_executed          -- one row per CodeAct snippet
        --   dispatch via codeact_sandbox::execute_code_act.
        -- search_replace_tier1_hit         -- one row per successful
        --   SearchReplaceMatcher::apply (custom non-git ELAI fences).
        -- search_replace_collision_fallback -- one row per
        --   DelimiterCollisionGuard rejection + whole-file rewrite.
        -- cloud_strict_schema_applied      -- one row per
        --   CloudGroundingClient strict-schema directive.
        -- grammar_cache_loaded             -- one row per sidecar boot
        --   warmup completion (RUNPOD-PENDING producer).
        -- structured_engine_rejected       -- one row per lm-format-enforcer
        --   refusal (XGrammar is canonical).
        -- legacy_json_object_refused       -- one row per legacy vendor
        --   json_object refusal.
        'xgrammar_mask_applied',
        'instructor_mode_activated',
        'grammar_mode_activated',
        'codeact_action_executed',
        'search_replace_tier1_hit',
        'search_replace_collision_fallback',
        'cloud_strict_schema_applied',
        'grammar_cache_loaded',
        'structured_engine_rejected',
        'legacy_json_object_refused',
        -- E-6 (SCHEMA_VERSION 148): MAST-grounded multi-agent failure-mode
        -- defenses. Four atomic wire tags per
        -- workflow/reasoning/E-6-decisions.md DECISION-5. Citation
        -- grounding via arXiv:2503.13657 (Cemri et al. 2025, MAST). Note:
        -- plan originally cited 2406.12624 which was an ID_COLLISION
        -- (Judging the Judges); substituted per DECISION-1 + gotcha
        -- E6_CITATION_2406_12624_ID_COLLISION.
        'audit_mast_tagged',
        'iso_budget_measured',
        'persona_reanchored',
        'cross_family_unavailable',
        -- E-7 (SCHEMA_VERSION 149): darwinian self-improvement substrate.
        -- SINGLE-VARIANT scope-lean per Pattern-2; phase discriminator
        -- on payload covers all 11 fire-points. Wire-dark unless
        -- ELAI_DARWINIAN_LOOP_ENABLE=1 AND cfg(feature darwinian-evolve).
        'darwinian_generation_completed',
        -- E-3 (SCHEMA_VERSION 150): runtime handoff signal beacon.
        'runtime_handoff_signal',
        -- E-26 (SCHEMA_VERSION 151): inference-engines 2026 pack.
        'sglang_backend_registered',
        'xgrammar_schema_compiled',
        'runtime_cold_start_observed',
        -- E-2 (SCHEMA_VERSION 152-153): streaming timing + UX pacing vote.
        'streaming_timing_recorded',
        'ux_pacing_vote_recorded',
        -- E-28 (SCHEMA_VERSION 154-156): attention-kernels 2026 pack.
        'streaming_llm_engaged',
        'sdpa_backend_pinned',
        'yarn_rope_scaling_applied',
        -- E-8 (SCHEMA_VERSION 162): self-evolving prompts substrate.
        -- SINGLE-VARIANT scope-lean per Pattern-2; OptimizerPhase
        -- discriminator on payload covers 7 fire-points. Wire-dark
        -- unless ELAI_PROMPT_OPTIMIZER_ENABLE=1 AND
        -- cfg(feature darwinian-evolve).
        'prompt_optimization_completed',
        -- E-10 (SCHEMA_VERSION 163): hostile-critic structural simplifications.
        -- SINGLE-VARIANT Pattern-2 (15th application); CriticTilePhase 9-arm
        -- discriminator on payload covers fsm/role/rag/mistake_book/typed_cot/
        -- tier/cloud/grc/cache fire-points. Wire-dark unless
        -- ELAI_CRITIC_OF_DESIGN_ENABLE=1.
        'critic_decision_event',
        -- E-43 (SCHEMA_VERSION 164): agent skill systems telemetry.
        -- SINGLE-VARIANT Pattern-2 (16th application); SkillEventPhase
        -- 3-arm discriminator on payload (loaded / invoked / failed).
        -- Wire-dark unless ELAI_SKILL_SYSTEM_ENABLE=1.
        'skill_system_event',
        -- E-22 (SCHEMA_VERSION 170): may-2026 trending substrate
        -- (SKILL.md export + microVM sandbox + lazy MCP schema savings).
        -- SINGLE-VARIANT Pattern-2 (22nd application); TrendingEventKind
        -- 3-arm discriminator on payload (skill_export_normalized /
        -- microvm_fallback / lazy_schema_savings). Wire-dark unless
        -- ELAI_GITHUB_TRENDING_ENABLE=1.
        'github_trending_event',
        -- E-23 (SCHEMA_VERSION 171): 2026 developer-discourse positioning
        -- + UX-guard substrate. SINGLE-VARIANT Pattern-2 (23rd application);
        -- DiscourseEventKind 3-arm discriminator on payload
        -- (trust_telemetry_aggregated / runtime_choice_badge_emitted /
        -- context_state_warning). Wire-dark unless
        -- ELAI_DEVELOPER_DISCOURSE_ENABLE=1.
        'developer_discourse_event',
        -- E-9 (SCHEMA_VERSION 165): external agent-eval benchmark suite.
        -- SINGLE-VARIANT Pattern-2 (17th application); BenchPhase 5-arm
        -- discriminator on payload (loaded / started / item_evaluated /
        -- completed / aborted). suite_name string field discriminates the
        -- 11 external benchmark suites. Wire-dark unless
        -- ELAI_EVAL_BENCHMARK_ENABLE=1. Live runs RUNPOD-PENDING.
        'benchmark_run_observed',
        -- E-19 (SCHEMA_VERSION 166): reasoning quality probe.
        -- SINGLE-VARIANT Pattern-2 (18th application); ProbeKind 6-arm
        -- discriminator on payload (l2_gate_decision / l4_prm_lite_score /
        -- l4_lateral_kept / l4_ttc_budget_chosen / l6_verifier_diversity /
        -- vividness_probe [I-5]). Wire-dark unless ELAI_REASONING_QUALITY_PROBE_ENABLE=1.
        'reasoning_quality_probe',
        -- E-54 (SCHEMA_VERSION 167): eval-corpus repair pre-deploy.
        -- SINGLE-VARIANT Pattern-2 (19th application); CorpusEventPhase
        -- 14-arm discriminator on payload covers ingest_rejected_ngram /
        -- ingest_rejected_cosine / shard_created /
        -- schema_violation_no_tests / test_fails_on_reference /
        -- test_useless_passes_broken / kor_orthogonality_violated /
        -- kor_orthogonality_passed / sycophancy_framing_gap_detected /
        -- sandbagging_anomaly / generator_rotation_violated /
        -- perf_delta_contamination_suspected /
        -- dynamic_knowledge_qa_generated / prod_traffic_probe_injected.
        -- Wire-dark unless ELAI_EVAL_CORPUS_ENABLE=1. Live ingest +
        -- LoRA cross-family transfer + sandbox test_executor + sidecar
        -- embeddings + LLM-grader RUNPOD-PENDING. See
        -- workflow/reasoning/E-54-decisions.md.
        'eval_corpus_event',
        -- E-55 (SCHEMA_VERSION 168): agentic RL + reward ladder substrate.
        -- SINGLE-VARIANT Pattern-2 (20th application); RewardLadderEventKind
        -- 9-arm discriminator on payload covers rlaif_judge_rated /
        -- rlaif_calibration_correlation_low / gepa_template_evolved /
        -- gepa_template_evicted / reward_fusion_computed /
        -- reward_hacking_suspected / sft_training_threshold_reached /
        -- bai_arm_observation_recorded / rl_weight_update_unlocked.
        -- Wire-dark unless ELAI_AGENTIC_RL_ENABLE=1. RLAIF cloud Sonnet
        -- judge + GEPA template evolution + LoRA fine-tune RUNPOD-PENDING.
        -- See workflow/reasoning/E-55-decisions.md.
        'reward_ladder_event',
        -- E-11 (SCHEMA_VERSION 169): SaaS prior-art catalog + recommender
        -- substrate. SINGLE-VARIANT Pattern-2 (21st application);
        -- PriorArtEventKind 2-arm discriminator on payload
        -- (archetype_recommended / prior_art_consulted). 5-archetype
        -- taxonomy locked by e50.A guard (SaasDashboard /
        -- B2bMarketplace / InternalTooling / DeveloperApi /
        -- ConsumerSocial). Wire-dark unless
        -- ELAI_SAAS_PRIOR_ART_ENABLE=1. Sibling slices (scope_clarity_
        -- score, package_verifier, anti-spiral hard-abort, edit_format)
        -- NOT in this commit. See workflow/reasoning/E-11-decisions.md.
        'saas_prior_art_event',
        -- E-25 (SCHEMA_VERSION 172): claude-code-architecture catalog
        -- substrate. SINGLE-VARIANT Pattern-2 (24th application);
        -- ClaudeCodeArchitectureEventKind 9-arm discriminator on payload
        -- covers compaction_ladder_tier_applied / skill_template_executed
        -- / prompt_decision_made / opusplan_overlay_activated /
        -- session_forked / edit_checkpoint_snapshotted /
        -- bash_complexity_blocked / approval_fatigue_detected /
        -- sandbox_template_warned. 9-primitive taxonomy locked at
        -- crate::architecture::claude_code_arch::ClaudeCodeArchPrimitive
        -- (CompactionLadder / SkillTemplating / PromptHandler /
        -- OpusplanOverlay / SessionFork / EditCheckpoints /
        -- BashComplexityGuard / ApprovalFatigueGuard /
        -- SandboxTemplateGuard). Wire-dark unless
        -- ELAI_CLAUDE_CODE_ARCH_ENABLE=1. Concrete implementation
        -- modules (compaction_ladder.rs, cli/fork.rs,
        -- edit_checkpoints.rs, prompt_handler.rs, dashboard panels) are
        -- FOLLOW-UPS. Primitive 2 (deferred MCP descriptions) DESCOPED
        -- pending E-22 / C-35 lazy_schemas substrate. See
        -- workflow/reasoning/E-25-decisions.md.
        'claude_code_architecture_event',
        -- E-29 Tier-B (SCHEMA_VERSION 173): agentic-observability event
        -- substrate. SINGLE-VARIANT Pattern-2 (25th application);
        -- AgenticObservabilityEventKind 4-arm discriminator covers
        -- mast_failure_classified / replay_resume / signal_match /
        -- eval_dataset_member. MAST classifier wires the 14-variant
        -- crate::mast::MastMode taxonomy (E-6 SHIPPED, do NOT
        -- redefine). Replay validator FAIL-CLOSED on C-44/D-9
        -- frozen-surface predicate (locked_keys_auditor). Wire-dark
        -- unless ELAI_AGENTIC_OBSERVABILITY_ENABLE=1. DRIFT-2: the
        -- pre-existing 'failure_classified' action_type owns the
        -- 13-category SYMPTOM taxonomy; the new event family is the
        -- distinct CAUSE channel. See
        -- workflow/reasoning/E-29-decisions.md.
        'agentic_observability_event',
        -- E-31 (SCHEMA_VERSION 174): coding-model-leaderboard scraper
        -- substrate. SINGLE-VARIANT Pattern-2 (26th application);
        -- LeaderboardEventKind 3-arm discriminator on payload
        -- (scrape_started / scrape_completed / scrape_failed). Wire-dark
        -- unless ELAI_CODING_MODEL_LEADERBOARD_ENABLE=1. Scraper at
        -- benches/external/coding_model_leaderboard.py emits stdout JSON
        -- rows parsed by the rust dispatcher. Live scrape RUNPOD-PENDING.
        -- Targets: BigCodeBench / EvalPlus HumanEval+ MBPP+ /
        -- SWE-Bench Pro / LiveCodeBench. See
        -- workflow/reasoning/E-31-leaderboard-decisions.md.
        'coding_model_leaderboard_event',
        -- E-38 (SCHEMA_VERSION 175): 2026 AI roadmap signals + EU AI Act
        -- compliance pre-gate substrate. SINGLE-VARIANT Pattern-2 (27th
        -- application); RoadmapSignalKind 5-arm discriminator on payload
        -- (compliance_blocked / compliance_verified /
        -- mcp_transport_version_logged / mrcr_unverified /
        -- posture_doc_stale). Wire-dark unless
        -- ELAI_AI_ROADMAP_SIGNALS_ENABLE=1. Producers: orchestrator/
        -- compliance.rs (compliance arms), plugins/mcp_bridge.rs
        -- (transport arm), orchestrator/research/roadmap_signals.rs
        -- (mrcr + posture-stale arms). See
        -- workflow/reasoning/E-38-decisions.md.
        'ai_roadmap_signal_event',
        -- E-39 (SCHEMA_VERSION 176): cost + token economics substrate.
        -- SINGLE-VARIANT Pattern-2 (28th application); CostEventKind
        -- 8-arm discriminator covers task_aborted_by_spend_cap /
        -- velocity_cap_breached / automix_verifier_verdict /
        -- cost_router_decision / cost_estimate_inaccurate /
        -- free_tier_quota_approaching / free_tier_quota_exceeded /
        -- layered_cap_manual_override. Wire-dark unless
        -- ELAI_COST_TOKEN_ECONOMICS_ENABLE=1. Producers: orchestrator/
        -- cost/{velocity_cap,automix_gate,cost_router,free_tier_guard,
        -- cloud_abort}.rs + CLI `elai cloud override-cap`. See
        -- workflow/reasoning/E-39-decisions.md.
        'cost_token_event',
        -- E-50 (SCHEMA_VERSION 177): SaaS architecture pattern catalog
        -- substrate. SINGLE-VARIANT Pattern-2 (29th application);
        -- SaasArchPatternEventKind 11-arm discriminator. Added here as
        -- E-4 drift-fix: variant was registered in enum + as_sql_str +
        -- FromStr + ALL_VARIANTS + payload but missing from SQL CHECK
        -- (would have caused write_payload runtime rejection on the
        -- first live emission). Wire-dark unless
        -- ELAI_SAAS_ARCH_PATTERNS_ENABLE=1.
        'saas_arch_pattern_event',
        -- E-4 (SCHEMA_VERSION 178): chat UX critique substrate.
        -- SINGLE-VARIANT Pattern-2 (30th application);
        -- ChatUxCritiqueKind 13-arm discriminator covers
        -- claim_confidence_tier_emitted / sycophancy_pushback_detected /
        -- user_rule_pinned / user_rule_unpinned / session_re_anchored /
        -- doom_loop_detected / completion_claim_rejected_no_evidence /
        -- one_shot_clarifier_emitted / one_shot_clarifier_resolved /
        -- refusal_emitted / refusal_overridden_by_user /
        -- mid_stream_interrupt_committed / honest_correction_emitted.
        -- Wire-dark unless ELAI_CHAT_UX_CRITIQUE_ENABLE=1. Producer:
        -- orchestrator/src/chat_ux/* (substrate module).
        'chat_ux_critique_event',
        -- E-16 (SCHEMA_VERSION 179): VLM image-validation hardening
        -- single-variant Pattern-2 (31st application).
        -- VlmImageValidationKind 3-arm discriminator covers
        -- swap_order_disagreement / axtree_mismatch /
        -- vlm_schema_unconstrained. SycophancyFlag (E-4 / D-14) is
        -- REUSED for the sycophancy_probe inverted-framing path.
        -- Wire-dark unless ELAI_VLM_IMAGE_VALIDATION_ENABLE=1.
        -- Producer:
        -- orchestrator/src/media_validator/{vlm_validator,
        -- axtree, sycophancy}.
        'vlm_image_validation_event',
        -- E-20 (SCHEMA_VERSION 180): local-cloud routing substrate.
        -- SINGLE-VARIANT Pattern-2 (32nd application);
        -- RoutingEventKind 9-arm discriminator covers
        -- confidence_emitted / hybrid_escalation /
        -- difficulty_classified / privacy_blocked / output_filtered /
        -- burn_alert / reasoning_mode / majority_vote /
        -- role_priority_chosen. PrivacyBlock fire-point composes
        -- with the pre-existing 'route_blocked' (C-5 ordinal 331)
        -- via a new reason='privacy_block' value extension on
        -- RouteBlockedPayload.reason: String. Wire-dark unless
        -- ELAI_LOCAL_CLOUD_ROUTING_ENABLE=1. Producer:
        -- orchestrator/src/cost/cloud_route.rs (sibling to E-39
        -- automix_gate + cost_router + cloud_abort).
        'local_cloud_routing_event',
        -- E-21 (SCHEMA_VERSION 181): peer-systems harness substrate.
        -- SINGLE-VARIANT Pattern-2 (33rd application);
        -- PeerSystemsEventKind 4-arm discriminator covers
        -- critic_candidates_generated / critic_score_computed /
        -- critic_pick_committed / critic_rerank_skipped. Wire-dark
        -- unless ELAI_PEER_SYSTEMS_ENABLE=1. Producer:
        -- orchestrator/src/peer_systems/{critic_rerank, sbfl,
        -- repo_state_store, aci_audit}.rs. NOTE: E-21 added the
        -- enum variant + as_sql_str + from_str + golden but missed
        -- adding the CHECK constraint string here; E-36 closes the
        -- gap during the SCHEMA_VERSION 181 -> 182 bump.
        'peer_systems_event',
        -- E-36 (SCHEMA_VERSION 182): document + vision RAG substrate.
        -- SINGLE-VARIANT Pattern-2 (34th application);
        -- DocumentVisionRagEventKind 5-arm discriminator covers
        -- vision_chunk_emitted / vision_fallback_triggered /
        -- vision_rag_policy_decision / visual_artifact_check_verdict /
        -- doc_parser_cloud_fallback_blocked_by_mode. Wire-dark
        -- unless ELAI_DOCUMENT_VISION_RAG_ENABLE=1. Producers:
        -- orchestrator/src/document_vision_rag/{visual_verifier,
        -- policy, mod}.rs + knowledge::chunking::image_artifact +
        -- knowledge::chunking::pdf_structured.
        'document_vision_rag_event',
        -- E-42 (SCHEMA_VERSION 183): tree-sitter grammar health substrate.
        -- SINGLE-VARIANT Pattern-2 (35th application);
        -- TreeSitterGrammarEventKind 4-arm discriminator covers
        -- abi_probe_mismatch / svelte_runes_parse_health /
        -- tier_b_grammar_loaded / tree_sitter_skipped_large_file.
        -- Wire-dark unless ELAI_TREE_SITTER_GRAMMARS_ENABLE=1. Producers:
        -- knowledge::tree_sitter_health::{abi_probe, svelte_health, large_file,
        -- registration} + DkgEngine init path.
        'tree_sitter_grammar_event',
        -- E-46 (SCHEMA_VERSION 184): planning-decomposition substrate.
        -- SINGLE-VARIANT Pattern-2 (36th application);
        -- PlanningDecompositionEventKind 8-arm discriminator covers
        -- reflection_appended / low_entropy_decomposition /
        -- goal_drift_canary_below_threshold / bubble_up_replan_phase /
        -- bubble_up_ledger_reset / agentless_shortcut_dispatched /
        -- process_reward_score_emitted / skill_prior_retrieved.
        -- Wire-dark unless ELAI_PLANNING_DECOMPOSITION_ENABLE=1.
        -- Producers: orchestrator/src/planning_decomposition/producers.rs.
        'planning_decomposition_event',
        -- E-48 (SCHEMA_VERSION 185): web-research orchestrator hardening
        -- substrate. SINGLE-VARIANT Pattern-2 (37th application);
        -- WebResearchEventKind 10-arm discriminator covers
        -- research_plan_emitted / tiered_search_dispatched /
        -- extraction_failed_all_extractors /
        -- cross_encoder_rerank_complete / speculative_branch_pruned /
        -- research_preemptive_stop / contradiction_node_materialized /
        -- citation_verification_outcome /
        -- citation_attribution_violation_blocked / recency_decay_applied.
        -- Wire-dark unless ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1.
        -- Producers: orchestrator/src/research/web_orchestrator/producers.rs
        -- + web_search/src/{tiered_search_pool,extractor_pool,
        -- cross_encoder_rerank}.rs telemetry hooks.
        'web_research_event',
        -- E-44 (SCHEMA_VERSION 186): durability observability substrate.
        -- SINGLE-VARIANT Pattern-2 (38th application);
        -- DurabilityEventKind 10-arm discriminator covers
        -- tool_idempotency_hit / tool_idempotency_replayed_irreversible /
        -- compensation_logged / compensation_replayed /
        -- compensation_failed / session_dag_continue_as_new /
        -- checkpoint_lazy_migrated / compact_lock_contended /
        -- agent_reliability_metrics / tool_error_classified.
        -- Wire-dark unless ELAI_DURABILITY_OBSERVABILITY_ENABLE=1.
        -- Producers: orchestrator/src/durability/{idempotent_dispatcher,
        -- continue_as_new,checkpoint_migrator,compact_lock,
        -- resume_validator}.rs. TASK-E44-013 closure.
        'durability_event',
        -- E-31 (SCHEMA_VERSION 187): head_dim invariant verification substrate.
        -- SINGLE-VARIANT Pattern-2 (39th application);
        -- HeadDimInvariantVerifiedKind 2-arm discriminator covers
        -- verified / refused. Wire-dark unless
        -- ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE=1. Producer:
        -- python/elai_inference/server.py::assert_head_dim_invariant.
        -- TASK-E31-001 closure.
        'head_dim_invariant_verified',
        -- E-3 (SCHEMA_VERSION 188): role-shown / model-hidden default
        -- telemetry substrate. SINGLE-VARIANT Pattern-2 (40th application);
        -- no Kind discriminator (scope-lean MVP). Wire-dark unless
        -- ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE=1. Producer:
        -- crates/dashboard-v2/src-tauri/src/commands.rs::record_model_visibility.
        -- TASK-E03-001 MVP closure.
        'role_shown_model_hidden',
        -- E-49 (SCHEMA_VERSION 189): coding-agent metrics named-token
        -- substrate. SINGLE-VARIANT Pattern-2 (41st application);
        -- CodingMetricNamedTokenKind 3-arm discriminator covers
        -- stable_pass_at_1 / harness_attribution_tuple / stop_verified.
        -- Wire-dark unless ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE=1.
        -- Closes TASK-E49-005 / 006 / 007 SPEC-DRIFT.
        'coding_metric_named_token',
        -- E-3 sub-phase 4A (SCHEMA_VERSION 190): runtime handoff event
        -- substrate. SINGLE-VARIANT Pattern-2 (42nd application);
        -- RuntimeHandoffEventKind 6-arm discriminator covers
        -- unified_preamble_applied / runtime_prewarm_started /
        -- runtime_prewarm_complete / handoff_payload_compressed /
        -- persona_vector_applied / style_normalizer_applied.
        -- Wire-dark unless ELAI_E3_RUNTIME_HANDOFF_ENABLE=1.
        -- Producer (4A): runtime_orchestrator.rs build_role_prompt wrap.
        -- Other 5 arms wired in 4B.
        'e3_runtime_handoff_event',
        -- E-31 sub-phase 6A (SCHEMA_VERSION 191): cloud-route decision
        -- event substrate. SINGLE-VARIANT Pattern-2 (43rd application);
        -- RouteDecisionEventKind 3-arm discriminator covers
        -- cloud_route_chosen / tier_model_swapped / local_route_kept.
        -- Wire-dark unless ELAI_ROUTE_DECISION_SSB_ENABLE=1. Producer:
        -- crates/orchestrator/src/cloud_mode.rs::CloudModeRouter::route
        -- via providers::runtime::RuntimeRoutingPolicy::evaluate.
        -- Closes E-31 plan's deferred CloudRouteDecision + TierModelSwap
        -- envelopes in a SINGLE Pattern-2 variant.
        'route_decision_event',
        -- E-47 (SCHEMA_VERSION 192): LoRA fine-tuning ship-gate
        -- substrate. SINGLE-VARIANT Pattern-2 (44th application);
        -- LoraShipGateEventPhase 3-arm discriminator covers
        -- refused / calibration_missing / calibration_passed.
        -- Wire-dark unless ELAI_LORA_SHIP_GATE_ENABLE=1. Producer:
        -- crates/orchestrator/src/lora_ship_gate.rs::notify_lora_ship_gate
        -- + `elai lora-gate-check` CLI subcommand.
        'lora_ship_gate_event',
        -- E-40 Tranche B-2 (SCHEMA_VERSION 193): SymbolGraph
        -- augmentation substrate. SINGLE-VARIANT Pattern-2 (45th
        -- application); SymbolGraphEventKind 4-arm discriminator
        -- covers symbol_dsl_executed / dataflow_impact_computed /
        -- agent_impact_claim_refuted / scope_graph_resolved.
        -- Wire-dark unless ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE=1 or
        -- other per-event gates upstream. Producer: crates/orchestrator/
        -- src/symbol_graph/mod.rs::notify_symbol_graph_event.
        'symbol_graph_event',
        -- Tranche B-6 quad (SCHEMA_VERSION 194): four new Pattern-2 /
        -- single-add SSB substrates land together. See ssb.rs SCHEMA_VERSION
        -- header for full tranche docs.
        -- C-42 SHIPPABLE-NOW: McpAuthzEvent (3-arm Kind
        -- authz_check_passed / authz_check_failed / corpus_replayed).
        -- Wire-dark unless ELAI_C42_MCP_AUTHZ_EMIT_ENABLE=1.
        'mcp_authz_event',
        -- C-47 Pattern-2 collapse: ToolCalibrationEvent (6-arm Kind
        -- hint_emitted / drift_detected / call_logged / case_added /
        -- case_selected / calibration_applied). The 3 pre-existing
        -- tool_call_logged / tool_calibration_hint_emitted /
        -- tool_calibration_drift_detected tags above stay alive for one
        -- release window as deprecation aliases. Wire-dark unless
        -- ELAI_C47_CASE_BASE_ENABLE=1.
        'tool_calibration_event',
        -- B-39 single-add: ReflectBudgetSoftWarn. Wire-dark unless
        -- ELAI_B39_SOFTWARN_ENABLE=1. Producer:
        -- crates/reasoning/src/chat_reflect.rs ~L454.
        'reflect_budget_soft_warn',
        -- C-50 Pattern-2 collapse: ContractCodingEvent (3-arm Kind
        -- retry_attempted / retry_resolved / retry_abandoned). Pre-existing
        -- contract_violation_detected stays alive (NOT collapsed in this
        -- tranche -- separate landing). Wire-dark unless
        -- ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE=1.
        'contract_coding_event',
        -- F-1 (SCHEMA_VERSION 196): VibeGateBlocked Pattern-2 single variant.
        -- 4-arm VibeGateBlockedKind discriminator: CodeVuln / Secret /
        -- DepHallucinated / Skill. DepHallucinated + Skill are RESERVED
        -- (parse/sql arms only; no producer this session). Wire-dark unless
        -- ELAI_VIBE_GATE_ENABLE=1.
        'vibesec_code_vuln_blocked',
        'vibesec_secret_blocked',
        'vibesec_dep_hallucinated_blocked',
        'vibesec_skill_blocked',
        -- F-2a (SCHEMA_VERSION 196): SecurityRuleBlocked flat beacon.
        -- Wire-dark unless ELAI_SECURITY_RULE_BLOCKED_ENABLE=1.
        'security_rule_blocked',
        -- F-4 / Slice B-1 (SCHEMA_VERSION 197): TestIntegrityEvent Pattern-2
        -- single variant. 2-arm TestIntegrityEventKind discriminator:
        -- Verified / FabricationBlocked. Carries stdout_hash for
        -- fabrication-detection pipeline.
        'test_integrity_verified',
        'anti_fabrication_block',
        -- F-3p1 / Slice C1 (SCHEMA_VERSION 198): ToolFilterEvent Pattern-2
        -- single variant. 2-arm ToolFilterEventKind discriminator:
        -- Committed / UntrustedQuarantined. CapabilityFlowBlocked is
        -- RESERVED (Tier-3) and is NOT included here.
        'tool_filter_committed',
        'untrusted_var_quarantined',
        -- F-9 / Slice D1 (SCHEMA_VERSION 199): DepWriteTimeGate slopsquat
        -- blocked beacon.  Emitted by DepWriteTimeGate::analyze() on
        -- NotFound / TooNew registry verdicts.  Fail-open on Unreachable
        -- (W-10 network-axis policy -- not blocked, Warn only).
        'write_time_slopsquat_blocked',
        -- F-10 / Slice A1 (SCHEMA_VERSION 200): security-monotonicity guard.
        -- Emitted when a refine / auto-advance / self-mod iteration would
        -- introduce a CWE class absent from the prior iteration. Default-ON;
        -- kill-switch ELAI_SECURITY_MONOTONICITY_DISABLE. loop_kind payload
        -- field discriminates the 3 production legs.
        'security_class_regression_blocked',
        -- F-2b / Slice A1 (SCHEMA_VERSION 201): insecure-rate keystone bench
        -- beacon. Emitted once per static-gate insecure-rate benchmark run over
        -- the SeCodePLT-structured CWE corpus. Bench telemetry, not a gate.
        'insecure_rate_measured',
        -- I-9 (SCHEMA_VERSION 263): error-amplification (alpha_t) telemetry
        -- beacon. Emitted once per `elai bench error-amplification` run over the
        -- frozen pipeline fixture. Bench telemetry, not a gate.
        'error_amplification_measured',
        -- L-4 slice 3: semantic-entropy measurement receipt. Emitted once per
        -- MEASURED sample set beside the aggregate claim-risk row; carries the
        -- per-sample digests so the entropy is recomputable. No row is written
        -- for a single-sample (unmeasured) degrade.
        'semantic_entropy_measured',
        -- L-19 slice 2: ambient page-in (recall) receipt. Emitted once per turn
        -- that appends at least one recall card to the projected tail; carries
        -- the path digests so the recall set is recomputable. No row is written
        -- for a turn that appends no cards.
        'context_recalled',
        -- F-6 / Slice A1 (SCHEMA_VERSION 202): SkillLibraryEvent Pattern-2
        -- single variant. 3-arm SkillLibraryEventKind discriminator:
        -- Acquired / TrustPromoted / DemotedStale. Carries the skill id, the
        -- tier transition, and invocation/failure-rate evidence.
        'skill_acquired',
        'skill_trust_promoted',
        'skill_demoted_stale',
        -- F-5 / Slice A1 (SCHEMA_VERSION 203): StalenessQuarantineBroadcast
        -- single flat beacon. Emitted at the orchestrator/CLI boundary when a
        -- memory-decay tick quarantines a fact (staleness_ratio crosses the
        -- decay threshold); the broadcast cascades to DKG concept_stale=2 and
        -- Cosmos artifact flagging via a bounded queue drained by the CLI.
        'staleness_quarantine_broadcast',
        -- F-8 / Slice A1 (SCHEMA_VERSION 204): SymbolicConflictDetected single
        -- flat beacon. Emitted at the `elai self-mod apply` CLI entrypoint when
        -- the LLM-free symbolic detector flags a self-mod config variant that
        -- contradicts an active locked-key invariant (self_mod_safety::
        -- LOCKED_KEYS) BEFORE archive admission; the variant is held PENDING
        -- (WARN-then-escalate, never hard fail-closed BLOCK).
        'symbolic_conflict_detected',
        -- F-7 / Slice A1 (SCHEMA_VERSION 205): TemplateUtilityRanked single
        -- flat beacon. Emitted at the try_execute gate-threshold seam when the
        -- Laplace-smoothed outcome utility of a crystallized pipeline template
        -- is computed from the PERSISTED CompiledPipeline first_try_success_rate
        -- + use_count; gates whether the template is reused. WARN/observe only
        -- (records the score), never a hard BLOCK.
        'template_utility_ranked',
        -- F-13 / Slice A1 (SCHEMA_VERSION 206): SessionArchived single flat
        -- beacon. Emitted at the run_migration destructive-DROP site (the
        -- Ssb::new production open path) when a stale-version DB is opened with
        -- ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE set; the stale ssb.db is cold-archived
        -- to workflow/sessions/{id}/archive/ before the DROP. store_kind is a
        -- payload field ('ssb'|'session_dag'), not a Pattern-2 Kind.
        'session_archived',
        -- F-11 / Slice A1 (SCHEMA_VERSION 207): DarwinianAdmit config-DGM
        -- admit-loop beacon. Pattern-2 single-variant; the 3-arm
        -- DarwinianAdmitKind discriminator (admitted/rejected_by_guard/
        -- loop_closed) is a payload field, not a separate wire tag. Wire-dark
        -- unless ELAI_DARWINIAN_LOOP_ENABLE=1 AND cfg(feature darwinian-evolve).
        'darwinian_admit',
        -- D-27.1 (SCHEMA_VERSION 209): DiscoveryRatified + DiscoveryRejected
        -- discovery-queue reviewer-action beacons. Emitted by the
        -- `ratify_discovery` / `reject_discovery` Tauri commands in
        -- commands_d27.rs when a human reviewer acts on a discovery_candidate
        -- novelty row in the DiscoveryQueueDrawer. Producer entrypoint:
        -- Tauri IPC (registered in generate_handler! in lib.rs). Reviewer
        -- identity carried in reviewer_id (non-empty enforced at command
        -- layer). ReviewStatus enum (Ratified/Rejected/Pending) co-located
        -- in ssb.rs alongside the payload structs.
        'discovery_ratified', 'discovery_rejected',
        -- G-plan (SCHEMA_VERSION 208-224): security + verifiability substrate.
        'mcp_rug_pull_blocked', 'mcp_scan_classifier_blocked', 'taint_flow', 'blast_radius_blocked', 'trifecta_quarantined',
        'run_artifact_emitted', 'review_surface_scored', 'provenance_attestation_emitted',
        'safety_delta_measured', 'model_drift_suspected', 'secret_redacted',
        'claim_confidence_scored', 'steering_vector_refresh_triggered',
        'canvas_op_applied', 'canvas_view_event', 'canvas_activity_event',
        'batch_lifecycle',
        -- H-plan verifiability substrate (SCHEMA_VERSION 225+):
        'deflation_gate_evaluated', 'leakage_blocked', 'lookahead_differential',
        'trading_null_test', 'rerun_hash_equality', 'sim_real_parity',
        'result_manifest_verified', 'differential_divergence',
        'gate_self_validation', 'reward_tampering', 'holdout_budget_event',
        'drift_interlock', 'trial_registered',
        -- N-8 slice 2: governance-coverage checker verdict beacon.
        'governance_coverage_measured',
        'mucoco_inconsistency_detected', 'mucoco_skip_reason',
        'user_decision_superseded', 'staleness_eval_measured',
        -- H-32 (SCHEMA_VERSION 210): FactValidationFailed single flat beacon.
        -- Emitted when a Tier-A DomainKnowledge/Mistake fact DKG citation
        -- fails to re-resolve before injection. reason is stale or unknown
        -- (treated as FAIL per R2). evidence_blake3: blake3 16-hex citation key.
        'fact_validation_failed',
        -- H-33 (SCHEMA_VERSION 211): MemoryGovernanceEvent single flat beacon.
        -- Emitted by elai memory forget|pin|unpin and the dashboard
        -- memory_forget/memory_pin Tauri commands. 3-arm kind discriminator:
        -- Forgotten/Pinned/Unpinned. actor: cli or dashboard.
        'memory_governance_event',
        -- H-34 (SCHEMA_VERSION 212): McpToolDeferred -- deferred MCP schema
        -- resolve on the live STDIO lane. Emitted on cache-miss when
        -- ELAI_MCP_DEFER_SCHEMA_ENABLE=1.
        'mcp_tool_deferred',
        -- H-34 (SCHEMA_VERSION 213): McpSchemaBudgetMeasured -- mcp-schema-budget
        -- bench result beacon. Emitted by `elai bench mcp-schema-budget`.
        'mcp_schema_budget_measured',
        -- H-35 (SCHEMA_VERSION 214): SessionRewound -- /rewind restore beacon.
        'session_rewound',
        -- H-36 (SCHEMA_VERSION 215): AgentNotificationEmitted -- dev-event
        -- notification beacon (task complete / needs input). Emitted by
        -- NotificationDispatcher::notify on PhaseComplete and RequestHuman.
        'agent_notification_emitted',
        -- H-37 (SCHEMA_VERSION 216): PlanDecisionEvent -- plan-approval gate
        -- decision at PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST.
        -- Gated by ELAI_PLAN_APPROVAL_GATE_ENABLE=1.
        'plan_decision_event',
        -- H-38 (SCHEMA_VERSION 217): StopConfirmationEvent -- cost-confirmation
        -- gate outcome on task/pod stop. Emitted by dispatch_stop.
        -- 3-arm StopOutcomeKind: ConfirmedProceed | RejectedHeld | PassedNoFriction.
        'stop_confirmation_event',
        -- H-39 (SCHEMA_VERSION 218): AbstentionScored -- abstention-aware scoring
        -- bench beacon. Emitted by bench_safedelta/bench_seccodeplt after computing
        -- the AbstentionAggregate over the run's outcome triples. Flat beacon.
        'abstention_scored',
        -- H-41 (SCHEMA_VERSION 219): CheckerGapDetected -- emitted at the
        -- route_claim call site when a Factual/Computational claim falls to
        -- non-Constructive because no Deterministic verifier is in context.
        -- Off-agent backlog artifact. Wire-dark: ELAI_CHECKER_GAP_ENABLE.
        'checker_gap_detected',
        -- H-42 (SCHEMA_VERSION 220): ReasoningBudgetScaled -- emitted at
        -- agent_pool::spawn + bsp/engine.rs timeout path when scale_grc_budget
        -- raises a sub-task GRC budget above the base. Default-ON; kill-switch
        -- ELAI_REASONING_BUDGET_SCALE_DISABLE. Depth-not-width guard (RD-4).
        'reasoning_budget_scaled',
        -- H-43 (SCHEMA_VERSION 221): FitnessEvaluated -- execution-grounded DGM
        -- fitness beacon. Replaces blake3-of-bytes derived_alpha (GAP-1). Emitted
        -- per scored candidate in darwin_admit_loop::run_one_generation under
        -- the darwinian-evolve feature + ELAI_DARWINIAN_LOOP_ENABLE=1.
        -- 3-arm Kind: ExecutionPassRate | DualAgreement | FitnessRegression.
        'fitness_evaluated',
        -- H-44 (SCHEMA_VERSION 222): TrapBehaviorDetected -- behavioral cheat-detector
        -- beacon. Replaces keyword-grep (GAP-2) with ImpossibleBench/IsomorphMismatch/
        -- CodeHalu/KeywordFallback detection legs + frozen TRACE recall measurement.
        -- 4-arm Kind: ImpossiblePassExploit | IsomorphMismatch | ExecutionHallucination |
        -- KeywordFallback.
        'trap_behavior_detected',
        -- H-45 (SCHEMA_VERSION 223): PromotionScoreAggregated -- worst-case/variance-
        -- penalized aggregate over lineage-diverse verdicts (Coste 2310.02743 WCO+UWO).
        -- Replaces candidate mean fitness as the two_gate_promote margin.
        -- 3-arm Kind: WorstCaseScored | VariancePenalized | LineageDiverseDisagree.
        'promotion_score_aggregated',
        -- H-46 (SCHEMA_VERSION 224): CladeSelected -- HGM clade-metaproductivity
        -- parent selection beacon (HGM 2510.21614). Emitted from
        -- config_archive::select_parent_with_cmp on the real loop selection step.
        -- CMP aggregates execution-grounded descendant deltas (H-43; VD-1).
        -- 2-arm Kind: CladeMetaproductivity | ImmediateScoreFallback.
        'clade_selected',
        -- H-47 (SCHEMA_VERSION 225): EvalFirewallEvent -- DGM eval firewall beacon.
        -- Composes D-6 date-gate + E-54 template-regen + H-19 Thresholdout budget binding.
        -- Emitted once per generation (aggregate counts) from run_one_generation.
        -- 3-arm Kind: PreUpdateItemRejected | TemplateRegenerated | IndistinguishabilityProbe.
        'eval_firewall_event',
        -- H-49 (SCHEMA_VERSION 226): EditHistoryEvent -- reversible LLM edits beacon.
        -- Emitted by the default-ON CAS time-travel capture hook (task_runner.rs) on
        -- every file pre-edit capture and by undo/redo/branch/jump CLI ops.
        -- 6-arm Kind: SnapshotCaptured | Deduped | Redo | Branched | Jumped | TooLargeSkipped.
        -- Default-ON kill-switch: ELAI_TIMETRAVEL_DISABLE (no allowlist row per R9).
        'edit_history_event',
        -- H-51 (SCHEMA_VERSION 227): HandoffLifecycleEvent -- cross-session sleep-time
        -- digest author + auto-detect + one-tap confirmed tiered load. Wires
        -- handoff-protocol.md into the runtime. Digest is authored from SSB journal rows.
        -- Kill-switch: ELAI_HANDOFF_AUTORESUME_DISABLE (registered in wire-dark-allowlist.toml).
        -- 4-arm Kind: DigestWritten | ResumeOffered | ResumeConfirmed | TieredLoad.
        'handoff_lifecycle_event',
        -- H-52 (SCHEMA_VERSION 228): FindingAnchored -- deterministic snippet->hunk
        -- finding-anchoring beacon. Emitted by elai review --cockpit via
        -- review_cmd::open_score_and_emit (the REAL CLI entrypoint). Off-agent artifact
        -- (R4): emitted from the CLI driver, not by the agent generating the finding.
        -- Default-ON kill-switch: ELAI_REVIEW_ANCHOR_DISABLE (no allowlist row per R9).
        -- 5-arm Kind: Anchored | FileFallback | Ambiguous | Unanchorable | Relocated.
        'finding_anchored',
        -- H-53 (SCHEMA_VERSION 229): CitationAnchorsBound -- citation pre-binding beacon.
        -- Emitted by elai prebind --demo-prebind (REAL CLI entrypoint) and by the
        -- /research|/ask answer path after citation_prebind::freeze_anchors runs.
        -- Carries anchor_count + seed + blake3_set_hash_hex for off-agent verification.
        -- Default-ON kill-switch: ELAI_CITATION_PREBIND_DISABLE (no allowlist row per R9).
        -- Pattern-2 single-variant 2-arm CitationAnchorsBoundKind: Bound | Blocked.
        'citation_anchors_bound',
        -- I-39 (SCHEMA_VERSION 230): PlanStateDerived -- per-plan state derived
        -- beacon. Emitted by elai plan status (REAL CLI entrypoint). Flat beacon;
        -- letter + session_id + derived_state + evidence_anchor.
        'plan_state_derived',
        -- I-45 (SCHEMA_VERSION 231): ObjectiveSealed -- objective-hash sealed-receipt
        -- beacon. Pattern-2 single-variant 3-arm ObjectiveSealedKind.
        -- Default-ON; kill-switch ELAI_OBJECTIVE_SEAL_DISABLE.
        'objective_sealed',
        -- I-34 (SCHEMA_VERSION 234): GrammarGateEvent -- CRANE hybrid grammar-pin gate beacon.
        -- Pattern-2 single-variant 3-arm GrammarGateEventKind:
        -- CranePassed / PinViolation / PostParseRejected.
        -- Default-ON; no kill-switch (mandatory gate per IC-18 + landmine #7/#8).
        'grammar_gate_event',
        -- I-38 (SCHEMA_VERSION 235): QdeltaMeasured -- QDELTA defective-diff rate flat beacon.
        -- Emitted by cli::bench_qdelta::run_inner after floor_check passes.
        -- Default-ON via bench entrypoint; no ELAI_*_ENABLE gate.
        'qdelta_measured',
        -- C-37: row emitted by the C-37 snowballing bench arm.
        'snowballing_measured',
        -- I-16: KV-cache tuning dry-run flat beacon.
        'kv_cache_tuned',
        -- I-43 (SCHEMA_VERSION 236): ClaimScopeMeasured -- reproducibility-claim scoper kill-gate.
        -- Pattern-2 single-variant 3-arm ClaimScopeMeasuredKind: PilotDelta / CrossLaneDivergence / ClaimScoped.
        -- Emitted by cli::bench_claim_scope::run_inner. Default-ON; no ELAI_*_ENABLE gate.
        'claim_scope_measured',
        -- I-10 (SCHEMA_VERSION 237): CacheEconomicsEvent -- prefix invariance + TTL tiering +
        -- cache-hit hard guard. Pattern-2 single-variant 3-arm CacheEconomicsEventKind:
        -- TtlPlanned / HitRateGuardTripped / PrefixDiffDumped.
        -- Default-ON kill-switch ELAI_CACHE_ECONOMICS_DISABLE=1. Cloud-lane only.
        'cache_economics_event',
        -- I-8 (SCHEMA_VERSION 238): TrajectoryAborted -- aletheia voluntary abort + dead-end hash
        -- pruning + halting-bound seam. Pattern-2 single-variant 3-arm TrajectoryAbortedKind:
        -- Voluntary / DeadEndHash / HaltingBound. Emitted by acknowledge_infeasible tool arm,
        -- dead_end_cache check path, and I-17 watchdog hook (Wave-3 stub registered here).
        'trajectory_aborted',
        -- I-1 (SCHEMA_VERSION 239): DirectiveExecuted -- directive supervisor enforcement-plane
        -- outcome. Pattern-2 single-variant 3-arm DirectiveExecutedKind: Executed / Discarded /
        -- CompensationFired. Emitted by directive::InProcessSupervisor::execute_directive.
        -- The generator emits pure Directive structs; the Supervisor is the sole side-effect sink.
        'directive_executed',
        -- I-33 (SCHEMA_VERSION 240): GovernancePrimitiveFired -- mechanical decision governance
        -- primitive beacon. Pattern-2 single-variant 3-arm GovernancePrimitiveFiredKind:
        -- CeflExternalized / I6qRejected / CommitRevealSealed. Emitted by
        -- decision_governance::emit_governance_fired (governed_prompt_decision call site).
        'governance_primitive_fired',
        -- I-32 (SCHEMA_VERSION 241): RoleContractApplied -- structural role-scope contract applied
        -- to a system-prompt build. Pattern-2 single-variant 3-arm RoleContractAppliedKind:
        -- PersonaStripped / PersonaRetained / AbDelta. Emitted by report_fns::build_system_prompt_for_role
        -- (PersonaStripped/PersonaRetained) and --role-ab-probe CLI harness (AbDelta).
        'role_contract_applied',
        -- I-35 (SCHEMA_VERSION 242): ToolSequenceVerdict -- temporal tool-sequence enforcement
        -- at the I-1 supervisor plane. Pattern-2 single-variant 3-arm ToolSequenceVerdictKind:
        -- Allowed / OrderViolation / HaltEnforced. Emitted by
        -- directive::InProcessSupervisor::run_sequence_gate (5th supervisor gate arm).
        -- Predicates evaluate the ordered tool-call history buffer (CDA-immune).
        -- R8-frozen: rule set hash in .claude/gate-manifest.txt.
        'tool_sequence_verdict',
        -- I-41 (SCHEMA_VERSION 243): CritiqueLensFinding -- spec-time critique lenses advisory
        -- beacon. Pattern-2 single-variant 3-arm CritiqueLensFindingKind: AdvisoryFinding /
        -- DroppedNoAnchor / LensRun. advisory=true on all arms; zero gate-verdict path.
        -- Emitted by critique::lenses::LensSet::run (post-convergence in run_critic_loop +
        -- `elai critique` CLI). Default-OFF (ELAI_CRITIQUE_LENSES_ENABLE=1 to enable).
        'critique_lens_finding',
        -- I-46 (SCHEMA_VERSION 244): BashRedirected -- bash interceptor redirect/fixup/
        -- allowed-prefix outcome. Pattern-2 single-variant 3-arm BashRedirectedKind:
        -- Redirected / FixedUp / PrefixBlocked. Default-ON; no kill-switch.
        -- Emitted by dispatch::in_process::InProcessDispatch::maybe_emit_bash_redirected
        -- when run_bash returns error_code bash_redirected / bash_fixedup / bash_prefix_blocked.
        'bash_redirected',
        -- I-47 (SCHEMA_VERSION 245): PromptHardeningApplied -- verifier-input XML tagging +
        -- worker rule injection beacon. Pattern-2 single-variant 3-arm
        -- PromptHardeningAppliedKind: InputsTagged / RulesInjected / SelfAttestationStripped.
        -- Default-ON; unconditional on auditor/critic/worker dispatches.
        -- Emitted by tools::execute_agent_with_spawn at build_agent_prompt seam.
        'prompt_hardening_applied',
        -- I-2 (SCHEMA_VERSION 246): BranchWorkspaceEvent -- CoW workspace isolation barrier
        -- outcome. Pattern-2 single-variant 4-arm BranchWorkspaceEventKind:
        -- Spawned / Merged / EscapeBlocked / ConflictEscalated. Emitted by
        -- sandbox::branch_workspace::WorktreePoolImpl (spawn/escape_check) + the BSP barrier
        -- (bsp/engine.rs barrier_merge). Composes C-11/E-15/I-1/H-49.
        'branch_workspace_event',
        -- I-5 (SCHEMA_VERSION 247): ContextPruned -- per-branch context rebuild outcome.
        -- Pattern-2 single-variant 3-arm ContextPrunedKind: SiblingExcluded / TracebackContrasted /
        -- VividnessProbe. Emitted by pristine_path_filter::PristinePathFilter::build_branch_context.
        -- Wire-dark until I-4 SearchLadder ships (Wave 4). Composes H-6/H-50/E-19.
        'context_pruned',
        -- I-11 (SCHEMA_VERSION 248): BeliefStateEvent -- typed versioned condensate receipt.
        -- Pattern-2 single-variant 3-arm BeliefStateEventKind: Condensed / Handoff / HonestyDelta.
        -- Condensed: emitted at every adaptive_reduce condensation (ID-3 JOURNALED-MIDDLEWARE receipt).
        -- Handoff: emitted when BeliefState payload is dispatched on the cloud escalation arm.
        -- HonestyDelta: emitted by HonestyJob after >=10 escalations (or --force-honesty-check).
        -- Default-ON (mandatory ID-3 receipt; no kill-switch). Composes I-7/H-51/H-6/CL-1..CL-4.
        'belief_state_event',
        -- I-13 (SCHEMA_VERSION 249): MemoryGuardEvent -- write-time memory guard outcome.
        -- Pattern-2 single-variant 3-arm MemoryGuardEventKind: PoisonBlocked / DualSerialized /
        -- TraceQueryExpanded. PoisonBlocked: emitted when Stage-2 aho-corasick scan refuses a write
        -- (invisible-Unicode or confusable-shell pattern). DualSerialized: emitted when a ToolResult
        -- is serialized via to_ui_dict + to_llm_content (records token_delta). TraceQueryExpanded:
        -- emitted when TraceQuery::expand returns exact line ranges on demand. Gated behind
        -- ELAI_MEMORY_GUARD_ENABLE (default-OFF, R9 experiment). Composes H-31/H-32/E-37/IS-11.
        'memory_guard_event',
        -- I-14 (SCHEMA_VERSION 250): ContextTransformReceipt -- deterministic context-compression
        -- middleware receipt. Pattern-2 single-variant 3-arm ContextTransformKind: LspSlice /
        -- AstFold / Tier2Compressed. LspSlice: rustc-panic payload sliced to the located error
        -- neighbourhood. AstFold: tree-sitter function-body elision (bodies elided, signatures
        -- kept). Tier2Compressed: reserved discriminator for the deferred neural tier. Default-ON
        -- (ID-3 JOURNALED-MIDDLEWARE; no kill-switch -- every transform journals a reversible
        -- receipt). Composes E-52/knowledge-tree-sitter/IS-6/H-6.
        'context_transform_receipt',
        -- I-12 (SCHEMA_VERSION 251): PlaybookEvent -- evidence-gated playbook curator outcome.
        -- Pattern-2 single-variant 4-arm PlaybookEventKind: Promoted / Demoted / ValidationRun /
        -- Exported. Default-ON; no kill-switch (ID-6 WEIGHTS-ARE-CONFIGS promotion ledger).
        -- Composes Phase-3F lifecycle + IS-16 trial_ledger + C-28 mutation suite.
        'playbook_event',
        -- I-17 (SCHEMA_VERSION 252): TriLayerVerdict -- tri-layer evaluation-stack per-branch
        -- verdict. Pattern-2 single-variant 4-arm TriLayerVerdictKind: HardVeto / ScoredSurvivor /
        -- DisagreementEscalated / FlakeQuarantined. Default-OFF (ELAI_TRI_LAYER_ENABLE, R9
        -- experiment, promotion plan I-4). Emitted by verifier_stack::TriLayerStack::run_verdict.
        -- Composes D-11/F-1/F-9/I-23/D-13 (GD-1, never restates).
        'tri_layer_verdict',
        -- I-18 (SCHEMA_VERSION 253): RowChainSealed -- per-row rolling blake3 chain beacon.
        -- Pattern-2 single-variant 3-arm RowChainSealedKind: ChainAdvanced / AibomEmitted /
        -- ChainBroken. ChainAdvanced: emitted from run_subcommand::emit_run_artifact after every
        -- sealed RRA (the rolling prev_hash advanced). AibomEmitted: emitted from
        -- run_subcommand::emit_attestation when the AI-BOM/CycloneDX-ML entry is sealed.
        -- ChainBroken: emitted from rra::build_rra when a Cargo.lock mutation is detected
        -- mid-run at the DEPS layer (halt-with-error path). Default-ON, no kill-switch.
        -- Composes G-2/G-8/I-45 (GD-1, never restates). TOTAL_VARIANTS 561 -> 562.
        'row_chain_sealed',
        -- I-19 (SCHEMA_VERSION 254): TddPreflightBlocked -- TDD enforcer preflight gate.
        -- Pattern-2 single-variant 3-arm TddPreflightBlockedKind; 3 distinct sql_str values.
        -- MissingTestDelta: new pub symbol in diff, no test symbol substring-matches it.
        -- SyntaxDryRunFailed: staged content fails cargo check --tests / pytest --collect-only
        --   compile-only dry-run (syntax error detected before the test-match step).
        -- RefactorExempted: diff introduces NO new pub symbol (pure refactor/rename), gate
        --   is skipped; this arm is advisory-only (SSB row emitted but no block returned).
        -- Wire-dark default: ELAI_TDD_PREFLIGHT_ENABLE=1 required to emit.
        -- GD-4: default-ON flip gated on FP rate bounded on >=3 real orchestrate runs.
        -- Promotion plan: I-19-promotion. TOTAL_VARIANTS 562 -> 563.
        'tdd_preflight_missing_test_delta',
        'tdd_preflight_syntax_dry_run_failed',
        'tdd_preflight_refactor_exempted',
        -- I-20 (SCHEMA_VERSION 255): MutationBudgetedRun -- budgeted mutation scheduler.
        -- Pattern-2 single-variant 4-arm MutationBudgetedRunKind; 4 distinct sql_str values.
        -- CiCleared: Wilson-CI lower bound cleared the (TOML-measured) kill-rate threshold.
        -- CiFailed: Wilson-CI upper bound already failed the threshold (fail-early).
        -- SurvivorDirective: a mutant survived; row names operator + line + surviving mutation.
        -- CacheHit: unchanged (fn-content, bound-test-set) blake3 key returned with ZERO runs.
        -- Default-ON; kill-switch ELAI_MUTATION_BUDGET_DISABLE=1. The 5 CI params are
        -- recomputable under ELAI_SEED (GD-2). ID-5: threshold read from
        -- metrics/mutation_baseline.toml, never a paper constant. TOTAL_VARIANTS 563 -> 564.
        'mutation_budgeted_run_ci_cleared',
        'mutation_budgeted_run_ci_failed',
        'mutation_budgeted_run_survivor_directive',
        'mutation_budgeted_run_cache_hit',
        -- I-21 (SCHEMA_VERSION 256): SprintContractEvent -- sprint-contract + ralph-loop.
        -- Pattern-2 single-variant 5-arm SprintContractEventKind; 5 distinct sql_str values.
        -- Proposed: isolated evaluator received the generator's sealed Sprint Contract.
        -- Approved: evaluator approved (acceptance criteria deterministically checkable).
        -- Rejected: evaluator rejected with a concrete structured objection (vague-criterion).
        -- RalphReinjected: a TriLayerStack failure reinjected the trace (bounded, cap = 3).
        -- AdversaryEscalated: 3rd failure escalated to the D-13 cross-family adversary.
        -- Wire-dark default: ELAI_SPRINT_CONTRACT_ENABLE=1 required to emit.
        -- Composes H-37/i-1/i-17/D-13 (GD-1). Promotion plan: I-21-promotion.
        -- TOTAL_VARIANTS 564 -> 565.
        'sprint_contract_proposed',
        'sprint_contract_approved',
        'sprint_contract_rejected',
        'sprint_contract_ralph_reinjected',
        'sprint_contract_adversary_escalated',
        -- I-37 (SCHEMA_VERSION 257): RequirementVerified -- per-EARS-requirement status.
        -- Pattern-2 single-variant 3-arm RequirementVerifiedKind; 3 distinct sql_str values.
        -- Satisfied: mutation coverage >= floor AND all bound MRs held AND >=1 bound test.
        -- Unverified: no bound tests -- shown in the cockpit, never dropped (R1).
        -- Regressed: a bound test deleted, an MR failed, or coverage fell below floor.
        -- Additive emit (always lands when an EarsRequirementSet is present). The BLOCKING
        -- RequirementsGate is default-OFF (ELAI_REQUIREMENTS_GATE_ENABLE=1 required to block).
        -- Composes IS-15/IS-8/I-20 (GD-1). Promotion plan: I-37-promotion.
        -- TOTAL_VARIANTS 565 -> 566.
        'requirement_verified_satisfied',
        'requirement_verified_unverified',
        'requirement_verified_regressed',
        -- I-44 (SCHEMA_VERSION 258): RetentionPurged -- zero-retention-after-verify purge.
        -- Pattern-2 single-variant 3-arm RetentionPurgedKind; 3 distinct sql_str values.
        -- RawInputsPurged: the active session's raw transcript files deleted post-verdict.
        -- EvidenceRetained: blake3 over the redacted retained evidence set (survives).
        -- PurgeVerified: post-purge beacon -- raw bytes gone, redacted evidence verifies.
        -- Destructive + opt-in (--zero-retention + ELAI_ZERO_RETENTION_ENABLE default-OFF).
        -- Composes G-9 redaction + G-9 TTL purge + F-13 archive + G-2/G-8 (GD-1).
        -- Promotion plan: I-44-promotion. TOTAL_VARIANTS 566 -> 567.
        'retention_raw_inputs_purged',
        'retention_evidence_retained',
        'retention_purge_verified',
        -- I-52 (SCHEMA_VERSION 259): CheckpointToolEvent -- agent-callable checkpoint
        -- tool outcome (save/rewind/diff over the H-49 time-travel CAS substrate).
        -- Pattern-2 single-variant 4-arm CheckpointToolEventKind; 4 distinct sql_str values.
        -- Saved: agent created a labelled checkpoint (tagged CAS manifest).
        -- Rewound: agent rewound to a saved label via timetravel::materialize (only writer).
        -- Diffed: agent diffed the working tree against a saved label (read-only).
        -- RefusedDiverged: rewind REFUSED because the tree diverged since save (RV-1; no
        --                  silent overwrite) -- the load-bearing safety observable (R1).
        -- Default-ON over H-49 (ELAI_TIMETRAVEL_DISABLE=1 kill-switch; no separate flag).
        -- Composes H-49 timetravel::{capture_file,materialize} (GD-1). TOTAL_VARIANTS 567 -> 568.
        'checkpoint_tool_saved',
        'checkpoint_tool_rewound',
        'checkpoint_tool_diffed',
        'checkpoint_tool_refused_diverged',
        -- I-54 (SCHEMA_VERSION 260): StructuralEditApplied -- agent-callable
        -- structural (AST-aware) edit tool outcome (search/preview/apply over the
        -- shipped tree-sitter grammars; one version source via
        -- elai_knowledge::TreeSitterAstEvaluator; ast-grep dep BANNED, SLC-3).
        -- Pattern-2 single-variant 5-arm StructuralEditAppliedKind; 5 distinct sql_str.
        -- Matched: query ran, match set returned. Previewed: query+template resolved
        --   to a would-be diff (blake3 diff hash), ZERO bytes.
        -- Applied: rewrite set applied through the SHIPPED edit gates + dispatch_edit.
        -- RefusedStale: a matched node's bytes changed between preview and apply (the
        --   AST moved); apply REFUSED -- load-bearing safety observable (R1).
        -- ParseFailed: broken query/template caught at preview, ZERO bytes (no regex
        --   degrade, stub_ban). The taint-sink + FS-blast-radius gates are the safety;
        --   no separate I-54 flag. TOTAL_VARIANTS 568 -> 569.
        'structural_edit_matched',
        'structural_edit_previewed',
        'structural_edit_applied',
        'structural_edit_refused_stale',
        'structural_edit_parse_failed',
        -- I-4 (SCHEMA_VERSION 261): SearchNodeEvent -- evidence-ranked search-ladder
        -- branch lifecycle (Wave-4 KEYSTONE). Pattern-2 single-variant 4-arm
        -- SearchNodeEventKind; 4 distinct sql_str values.
        -- Expanded: candidate branch generated + pristine context (I-5) + tri-layer
        --   verdict (I-17). Pruned: branch DROPPED (hard-veto or flake-quarantine);
        --   terminal (ID-4), never Promoted. Promoted: best-of-N winner, row carries
        --   verifier-derived evidence (tri-layer score + mutation kill-rate +
        --   progress). BudgetCollapsed: no survivor; fell back to linear retry.
        -- Default-OFF: ELAI_SEARCH_LADDER_ENABLE=1 required (GD-4 measured-first-win).
        -- Composes I-2/I-5/I-17/I-20 (GD-1). Promotion plan: I-4-promotion.
        -- TOTAL_VARIANTS 569 -> 570.
        'search_node_expanded',
        'search_node_pruned',
        'search_node_promoted',
        'search_node_budget_collapsed',
        -- I-7 (SCHEMA_VERSION 262): EscalationCascade -- cascading escalation waterfall
        -- lifecycle (Wave-4). Pattern-2 single-variant 4-arm EscalationCascadeKind; 4
        -- distinct sql_str values. LocalWin: a local branch survived the I-17 tri-layer
        -- check (task solved locally; cloud arm never ran -- E-39 zero-cloud-spend).
        -- Condensed: all local branches failed; failure set condensed into the typed
        --   I-11 BeliefState (<=2K, NO raw history -- ID-3/stub_ban).
        -- Escalated: condensate routed SYNCHRONOUSLY to the H-7 cloud seam
        --   (CloudModeRouter::route); billed inference is D-026 PENDING (never fired).
        -- BatchRouted: journal-completeness ONLY for the OFFLINE H-5 batch lane; the
        --   LIVE waterfall NEVER emits it (IC-1: no batch-client import).
        -- Default-OFF: ELAI_ESCALATION_WATERFALL_ENABLE=1 required (GD-4). Composes
        -- I-4/I-11/H-7 (GD-1). Promotion plan: I-7-promotion. TOTAL_VARIANTS 570 -> 571.
        'escalation_cascade_local_win',
        'escalation_cascade_condensed',
        'escalation_cascade_escalated',
        'escalation_cascade_batch_routed',
        -- I-22 (SCHEMA_VERSION 264): ProgressRewardScored -- delta-progress shaped-reward
        -- scorer event. Pattern-2 single-variant 3-arm ProgressRewardScoredKind; 3 distinct
        -- sql_str values.
        -- Delta: shaped reward phi(h_t) - phi(h_{t-1}) scored for one branch step; all phi
        --   components are deterministic observables from the journal (ID-4 compliance).
        -- FirstErrorTruncated: branch truncated at first i-17 HardVeto (step k-1); H-49 CAS
        --   fork node id at step k-1 is recorded so the salvaged prefix can be reused.
        -- DecayApplied: D consecutive zero-delta steps triggered the no-progress decay
        --   multiplier; D and weights are read from metrics/progress_reward_weights.toml
        --   (ID-5: measured, never a paper-derived constant).
        -- Wire-dark default: ELAI_PROGRESS_REWARD_ENABLE=1 required to emit.
        -- Composes i-17 TriLayerStack / H-49 timetravel / i-37 EarsRequirementSet (GD-1).
        -- Promotion plan: I-22-promotion. TOTAL_VARIANTS 572 -> 573.
        'progress_reward_delta',
        'progress_reward_first_error_truncated',
        'progress_reward_decay_applied',
        -- I-24 (SCHEMA_VERSION 266): PrmCanaryEvent -- isomorphic canary anti-gaming event.
        -- Pattern-2 single-variant 3-arm PrmCanaryEventKind; 3 distinct sql_str values.
        -- CanaryInjected: pair injected, delta below threshold (no bias detected).
        -- StyleBiasDetected: delta exceeded ELAI-measured threshold (ID-5, metrics/
        --   canary_bias_threshold.toml); scorer flagged as style-biased.
        -- ScorerNeutralized: N consecutive StyleBiasDetected -> scorer weight zeroed
        --   for the run (ID-4: L2-only; CANNOT override L1 HardVeto).
        -- R8: canary pair set FROZEN in fixtures/canary_pairs.json (gate-manifest entry).
        -- Wire-dark default: ELAI_CANARY_INJECTOR_ENABLE=1 required to emit.
        -- Composes i-17 TriLayerStack L2 leg (GD-1). Promotion plan: I-24-promotion.
        -- TOTAL_VARIANTS 573 -> 574.
        'prm_canary_injected',
        'prm_canary_style_bias_detected',
        'prm_canary_scorer_neutralized',
        -- I-27 (SCHEMA_VERSION 265): ConvergenceCurveMeasured -- verified-outcome-rate
        -- vs compute convergence curve flat beacon.
        -- Flat beacon (no Kind discriminator); one sql_str value.
        -- Emitted by bench_convergence::run_inner from CliAction::BenchConvergence.
        -- Default-ON (CLI measurement instrument, no kill-switch).
        -- Composes E-49 ($/CTS), I-38 (bootstrap CI floor), H-9 (cockpit tile) (GD-1).
        -- WIRE-DARK[I-27]: live per-node compute-spend from I-4 SearchNodeEvent PENDING.
        -- TOTAL_VARIANTS 574 -> 575.
        'convergence_curve_measured',
        -- I-36 (SCHEMA_VERSION 267): PromptCompiled -- per-(role,backend) compiled-prompt
        -- artifact event. Pattern-2 single-variant 3-arm PromptCompiledKind; 3 distinct
        -- sql_str values so each Kind arm is distinguishable on the wire.
        -- Compiled: a promoted compiled variant substituted at the adapter boundary;
        --   variant hash + backend named in the payload.
        -- TransferGapMeasured: source-vs-compiled fixture pass-rate delta recorded;
        --   recomputable under ELAI_SEED (GD-2). Tier-2 DEFER (live model budget).
        -- Rolledback: variant regressed fixture floor; removed from store; substitution
        --   reverts to base assembled prompt.
        -- Default-ON (no-op pass-through when no variant stored).
        -- Kill-switch: ELAI_PROMPT_COMPILE_DISABLE=1 (kill_switch type, ops-level).
        -- Compiler output contract: NO CoT scaffold in compiled strings (landmine #6).
        -- Composes I-32 role_contracts + B-37 PerRoleRuntime + I-12 ledger gates (GD-1).
        -- promote_variant WIRE-DARK[I-36] (Tier-2 DEFER, I-36-promotion plan).
        -- TOTAL_VARIANTS 575 -> 576.
        'prompt_compiled',
        'transfer_gap_measured',
        'prompt_compiled_rolledback',
        -- I-50 (SCHEMA_VERSION 268): AuthBrokerEvent -- subscription auth broker lifecycle.
        -- 4-arm Pattern-2; each arm has a distinct sql_str for wire disambiguation.
        -- Default-ON. SSB-emit seam WIRE-DARK[I-50] pending Arc<Ssb> threading.
        -- TOTAL_VARIANTS 576 -> 577.
        'auth_broker_token_acquired',
        'auth_broker_refreshed',
        'auth_broker_provider_linked',
        'auth_broker_refresh_failed',
        -- J-1.T4 (SCHEMA_VERSION 269): PerceptionScaffoldEvent -- perception-gateway
        -- scaffold lifecycle. SINGLE-VARIANT Pattern-2; single sql_str, the 4-arm
        -- PerceptionScaffoldEventKind discriminator rides inside the payload.
        -- Wire-dark unless ELAI_PERCEPTION_GATEWAY_ENABLE=1.
        -- TOTAL_VARIANTS 577 -> 578.
        'perception_scaffold_event',
        -- K-18a follow-on (SCHEMA_VERSION 270): RecurrenceHitRateMeasured -- PS-5
        -- recurrence-hit-rate flat beacon for `elai bench case-reuse`. Flat beacon
        -- (no Kind discriminator), one sql_str value. Restores GD-2 journal-
        -- recomputability that K-18a's ABORT-4 explicitly deferred.
        -- Emitted by bench_case_reuse::run_inner from CliAction::BenchCaseReuse.
        -- Default-ON (fixture-suite CLI measurement instrument, no kill-switch).
        -- TOTAL_VARIANTS 578 -> 579.
        'recurrence_hit_rate_measured',
        -- K-4 SP1 follow-on (SCHEMA_VERSION 271): GateTaxMeasured -- composed
        -- gate-tax budget flat beacon. Flat beacon, one sql_str value.
        -- Emitted by bench_gate_tax::run from CliAction::BenchGateTax.
        -- Default-ON (fixture-or-journal CLI measurement instrument).
        -- TOTAL_VARIANTS 579 -> 580.
        'gate_tax_measured',
        -- J-28 follow-on: UsageTurnEvent -- uniform per-turn token-usage row,
        -- both lanes. Flat sql_str; the 3-arm Kind rides in the payload.
        -- TOTAL_VARIANTS 580 -> 581.
        'usage_turn_event',
        -- J-9 (SCHEMA_VERSION 273): HardwareModelCatalogEvent. Flat sql_str;
        -- CatalogBuilt/TierAssigned/EligibilityChecked/LaneSeeded rides in payload.kind.
        -- Default-ON from the one-shot startup SystemProbe path. 581 -> 582.
        'hardware_model_catalog_event',
        -- K-2 (SCHEMA_VERSION 274): deterministic routing economics benchmark.
        -- Default-ON from the real `elai bench routing-economics` CLI path.
        'routing_economics_measured',
        -- K-5 (SCHEMA_VERSION 275): deterministic risk-signal driven GateIntensity
        -- selection. Emitted observe-only from the deterministic audit gate.
        'gate_intensity_selected',
        -- K-9 (SCHEMA_VERSION 276): model refresh cadence decision row. Flat
        -- sql_str; CandidateEvaluated/DecisionRecorded/IncumbentKept rides in
        -- payload.kind. 584 -> 585.
        'model_refresh_cadence',
        -- PS-3 (SCHEMA_VERSION 277): hallucination-delta flat beacon. Emitted
        -- by bench_hallucination_delta::run from CliAction::BenchHallucinationDelta.
        -- Default-ON fixture-tier CLI measurement instrument. 585 -> 586.
        'hallucination_delta_measured',
        -- PS-READINESS E2ERUNNER (SCHEMA_VERSION 278): local-lane-e2e functional
        -- beacon. Emitted by bench_local_lane_e2e::emit_beacon_and_rra from
        -- CliAction::BenchLocalLaneE2e. Drives the 4B :8181 lane end to end.
        'local_lane_e2e_measured',
        -- J-13 (SCHEMA_VERSION 279): local-lane determinism telemetry event.
        -- Pattern-2 single-variant 4-arm Kind discriminator on the payload.
        -- Emitted from the local-lane e2e runner and determinism audit probes.
        'local_determinism_event',
        -- J-14 (SCHEMA_VERSION 282): repro-pack + RRA verify upgrade telemetry event.
        -- Pattern-2 single-variant 4-arm ReproPackKind: TierEvaluated /
        -- FirstDivergenceLocated / FlowFingerprintMismatch / FixtureExtracted.
        -- Emitted from rra::verify_rra, compiled_pipeline::TemplateCrystallizer, cli rra export-fixture.
        -- Default-ON; no kill-switch (verify path, never masks the verdict).
        'repro_pack_event',
        -- J-16 (SCHEMA_VERSION 283): blast-radius / anti-regression gate event.
        -- Pattern-2 single-variant 4-arm BlastRadiusRegressionKind: ConeComputed /
        -- ScopeViolationFlagged / RegressionSetSelected / SilentDeletionFlagged.
        -- Emitted from blast_radius::BlastRadiusRegressionGate on EXECUTING -> AUDITING.
        -- OBSERVE-ONLY (GD-4): flag, does not block (the hard-block is TestFlippedGate).
        'blast_radius_regression_event',
        -- J-30 T7b (SCHEMA_VERSION 284): provider BYOK lifecycle event.
        -- Pattern-2 flat variant; 3-arm ProviderByokEventKind rides in the payload.
        -- Emitted at the T5 configure + T2 custom-backend register seams.
        -- NO key/secret/token material in the payload (plan stub_ban).
        'provider_byok_event',
        -- J-32 (SCHEMA_VERSION 285): front-door intent routing decision.
        -- Pattern-2 flat variant; 5-arm IntentRoutedKind rides in the payload.
        -- Emitted by FrontDoorRouter on every routing decision.
        -- NEVER carries rewritten text (arg_spans are byte-offsets).
        'intent_routed',
        -- J-34 (SCHEMA_VERSION 286): empirical context calibration event.
        -- Pattern-2 flat variant; 4-arm ContextCalibrationKind rides in the payload.
        -- Emitted by CalibrationArtifactStore::insert (CurveMeasured) and the
        -- leg-2 / leg-3 seams (later slices).
        'context_calibration_event',
        -- J-36 (SCHEMA_VERSION 287): local prefix-KV cache event.
        -- Pattern-2 flat variant; 4-arm LocalPrefixCacheKind rides in the payload.
        -- Emitted by emit_local_prefix_cache_event (prefix_cache_ssb.rs) on the
        -- local dispatch path.
        'local_prefix_cache_event',
        -- J-24 (SCHEMA_VERSION 288): behavior steering event.
        -- Pattern-2 flat variant; 3-arm BehaviorSteeringKind rides in the payload.
        -- Emitted by emit_behavior_steering_event (behavior_steering_ssb.rs).
        'behavior_steering_event',
        -- J-35 intent-control layer: typed intent spec + premise-check +
        -- route + journaled assumptions. Pattern-2 flat variant; 4-arm
        -- IntentControlKind rides in the payload. Emitted by
        -- run_intent_control (intent_control.rs) on the task-lane intake
        -- before decomposition.
        'intent_control_event',
        -- J-39 T2: node admission gate event. Pattern-2 flat variant;
        -- 3-arm NodeAdmissionEventKind rides in the payload. Emitted by
        -- emit_node_admission_event (node_admission.rs) at the task_runner
        -- SPLIT_TASK phase_sizing branch.
        'node_admission_event',
        -- J-31: system-composition canvas event. Pattern-2 flat variant;
        -- 4-arm SystemContractEventKind rides in the payload. Emitted by
        -- emit_system_contract_event (system_contract_event.rs).
        'system_contract_event',
        -- J-12: HarnessInterventionEvent Pattern-2.
        'harness_intervention_event',
        -- J-54 UNIT8: FreePoolEvent Pattern-2 flat; 3-arm Kind in payload.
        'free_pool_event',
        -- J-73 Slice C: RetrievalAbstained Pattern-2 flat; 3-arm Kind in payload.
        'retrieval_abstained',
        -- J-78: ContextAccessSpec Pattern-2 flat; 3-arm Kind in payload.
        'context_access_spec',
        -- I-30 Slice 1a: SFT exhaust event. Pattern-2 flat; 3-arm
        -- SftExhaustEventKind rides in the payload.
        'sft_exhaust_event',
        -- J-23: review-gate catalog event. Pattern-2 flat; 4-arm
        -- ReviewGateCatalogEventKind rides in the payload.
        -- L-2: claim risk aggregate observation. Pattern-2 flat; tier kind rides in payload.
        'claim_risk_scored',
        'review_gate_catalog_event',
        -- J-42 T2: reasoning distill-reset checkpoint. Pattern-2 flat; 3-arm
        -- ReasoningCheckpointEventKind rides in the payload. Emitted by the
        -- ReflectEventSink bridge from the chat_reflect refine loop.
        'reasoning_checkpoint_event',
        'operating_profile_change',
        'advisor_consulted',
        'goal_enrolled', 'goal_verified', 'goal_violated',
        -- J-55 W1.4 (SCHEMA_VERSION 311): ModalityRouted -- deterministic TD-3
        -- action-modality routing verdict (keystone unit). Pattern-2 single-
        -- variant; 3-arm ModalityRoutedTag with one sql_str per arm so each
        -- resolution outcome is distinguishable on the wire (mirrors
        -- AuthBrokerEvent). Rich ModalityRoutedKind rides in the payload
        -- (I-41 CritiqueLensFinding two-enum split). Data-driven over
        -- ModelCapabilityProfile + tier; NEVER LLM-adjudicated.
        -- TOTAL_VARIANTS 619 -> 620.
        'modality_code_act_chosen',
        'modality_constrained_call_chosen',
        'modality_edit_format_chosen'
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

pub(crate) const CREATE_SSB_FILES_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS ssb_files (
    seq       INTEGER NOT NULL REFERENCES ssb(seq),
    file_path TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ssb_files_path ON ssb_files(file_path);
CREATE INDEX IF NOT EXISTS idx_ssb_files_seq  ON ssb_files(seq);
";

