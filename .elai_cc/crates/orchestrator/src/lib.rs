pub mod agent_pool;
pub mod agent_role;
pub mod agent_role_dispatcher;
pub mod agent_template;
// D-13: adversary role + cross-family heterogeneous-verifier check.
// Producer side of the adversarial-verification pass; consumed by
// `agent_pool::run_adversarial_pass`. Grounding: arxiv:2403.07974 (T1).
pub mod adversary;
// Phase B4 — Artifacts Pipeline (library skeleton). Filesystem-only write
// helper for context artifacts + JSON manifests. SSB `ArtifactPublished`
// row emission is INTENTIONALLY deferred to a follow-up patch (see
// `artifacts.rs` doc-comment) to avoid a concurrent edit to
// `orchestrator/src/ssb.rs`, which has in-flight B-28 work. Contract:
// `crates/contracts/tests/artifacts_pipeline_contract.rs`.
pub mod artifacts;
// E-14: browser end-to-end agents (Playwright snapshot, axe-core gate,
// journey assertions). Scope-lean cut — VLM tier RUNPOD-PENDING.
pub mod browser_e2e;
// B-42 Phase 2: cosmos briefing compiler (compile_cosmos_briefing).
pub mod briefing;
// B-41: per-level auditor mode dispatch. Defines AuditorMode (6 variants),
// TierNodeId (typed sum over ConceptId/PhaseId/StepId/BabyStepId), Verdict
// (Allow/Block), Finding, LevelAuditVerdict, and the dispatch / respawn /
// cascade helpers consumed by pipeline_tree_navigator. See
// workflow/reasoning/B-41-decisions.md.
pub mod auditor_mode;
pub mod deterministic_audit_gate;
pub mod deterministic_auditor;
pub mod grc;
pub mod anti_spiral;
// C-10 Phase 4G.9: task-runner-step-level anti-spiral primitives. Coexists
// with anti_spiral.rs (GRC-step-level Spinning detector). 11 types +
// 4-pattern detector + 5-persona switch + GoalDriftCanary + Watchdog +
// AdaptiveFutilityTracker. See workflow/reasoning/C-10-decisions.md.
pub mod anti_spiral_extended;
pub mod autonomous_gate;
pub mod sandbox;
// Phase B1 — Tier C Sandbox policy layer. Sibling of `sandbox` (the
// per-process AppContainer/user-namespace isolation runtime); the policy
// module is consulted by the in-process ToolDispatcher BEFORE any
// bash/file/network operation runs. See `sandbox_policy.rs` for the
// allow/deny decision matrix.
pub mod sandbox_policy;
pub mod bsp;
mod bash;
pub mod checkpoint;
pub mod coherence_check;
// E-35: CodeAct sandbox scaffolding (Runner-role-FREE dispatch wrapper on
// top of the existing crates/sandbox substrate). Composes at the boundary
// — full CodeAct role-cascade adoption DEFERRED to E-35.D patch-N per
// gotchas.md E41_AGENTROLE_FROZEN_AT_17 (230 match-arm cascade avoidance).
// See `codeact_sandbox.rs` module docs.
pub mod codeact_sandbox;
// E-44: durability layer additions (idempotent dispatcher + saga/lock/
// migrator/continue-as-new wrappers on top of the existing SessionDag +
// AgentCheckpoint surface). All additions are additive — no existing
// public APIs are modified. See `durability/mod.rs`.
pub mod durability;
pub mod dst;
pub mod cloud_mode;
pub mod context_budget;
pub mod dispatch;
pub mod dispatcher;
pub mod privacy_redactor;
pub mod branch_lock;
mod compact;
mod conversation;
pub mod evidence;
// B-28 E-1 added the focus_tunnel namespace (FIM prompt assembler + DIFF
// parser/replayer + ModeSelector). E-3 extends it with the mode selector
// and per-mode execution wrappers — see focus_tunnel/mod.rs.
pub mod focus_tunnel;
pub mod fsm;
pub mod gate;
// C-40 — GitHub PR adapter (gh CLI shell-out). Producer of the four
// SCHEMA_VERSION 109 SsbActionType::GithubPr* rows. Cross-language
// pair: dashboard-v2/src-tauri/src/commands_c40.rs +
// dashboard-v2/src/lib/windows/GithubWindow.svelte.
pub mod github_pr;
// B-40 — Idea-Framer + Domain Research Presets + Critic Loop. Inserts a
// FRAMING substate before Phase 4D Recursive Decomposition. Public surface
// re-exported below (FramedIdea, ProjectClass, CriticCategory, etc.).
pub mod idea_framing;
pub mod hooks;
pub mod hook_dispatcher;
pub mod shortcoder_hook;
pub mod provider_signals;
pub mod indexing_service;
// C-29: Testing-Trust Tier T-2 (IntentPair) — per-BabyStep intent
// formalization. See workflow/reasoning/C-29-decisions.md.
pub mod intent_pair;
// B-31 Phase 3 (Cycle B): cross-process IndexLock primitive (lock.rs) and
// startup auto-resume + graceful_shutdown (startup.rs). Both are free
// functions/types with no dependency on `ssb.rs` (SSB variant emission is
// deferred to PATCH-2 — see workflow/reasoning/B-31-decisions.md CYCLE B
// DECISIONS / OUT-OF-SCOPE).
pub mod lock;
pub mod startup;
// C-17 (DRIFT-F closure): production consumer of LockScope::Watch.
pub mod watcher_service;
pub mod ipc;
pub mod ipc_gateway;
pub mod ipc_types;
pub mod tool_filter;
mod lane_events;
mod mcp;
mod mcp_client;
pub mod mcp_lifecycle_hardened;
mod mcp_stdio;
pub mod mcp_tool_bridge;
pub mod permission_enforcer;
mod permissions;
pub mod plugin_lifecycle;
mod preload;
mod prompt;
pub mod recovery;
pub mod recovery_recipes;
pub mod grounding_client;
pub mod research_budget;
pub mod research_pipeline;
pub mod searx_search_backend;
pub mod research_tier;
pub mod research_tree;
pub mod pipeline_builder;
pub mod phase_sizing_splitter;
pub mod pipeline_tree_navigator;
pub mod per_level_audit_dispatch;
// C-25: compiled pipeline cache, template crystallizer, compiled executor,
// on-disk store. See `workflow/reasoning/C-25-decisions.md`.
pub mod compiled_pipeline;
// C-25: background-task manager with soft/hard timeout auto-detach +
// 2 new SSB beacons. See `workflow/reasoning/C-25-decisions.md`.
pub mod background_task;
#[cfg(test)]
mod pipeline_builder_bench;
#[cfg(test)]
mod pipeline_builder_tests;
pub mod web_research_orchestrator;
mod session;
#[cfg(test)]
mod session_control;
pub mod plan_skeleton;
pub mod session_dag;
pub mod workload_profiling;
// C-32 — Code-Completeness Layer 2: skeleton-first + MiniCritic +
// AcceptAndComplete + deterministic bridge templates.
pub mod skeleton_generator;
pub mod mini_critic;
pub mod structural_retry;
pub mod bridge_template;
// C-32 close-out (audit-C32 W1): per-blank dispatch wiring that drives
// BridgeTemplate -> StructuralRetrySequencer -> MiniCritic -> flush.
pub mod c32_dispatch;
pub use c32_dispatch::{
    classify_bridge_kind, dispatch_skeleton_with_retry, DispatchOutcome, PerBlankCandidates,
};
pub use skeleton_generator::{
    emit_skeleton_flushed_ssb, emit_skeleton_generated_ssb, BlankId, FillResult,
    FilledBuffer, SkeletonError, SkeletonGenerator, SkeletonGeneratorConfig, SkeletonSpec,
};
pub use mini_critic::{
    MiniCritic, MiniCriticLanguage, MiniCriticResult, MiniCriticVerdict, MIN_BODY_LINES,
};
pub use structural_retry::{
    emit_stage_attempt_ssb, RetryContext, StageCandidates, StaticStageCandidates,
    StructuralRetryConfig, StructuralRetryOutcome, StructuralRetryPlan, StructuralRetrySequencer,
    StructuralRetryStage, ACCEPT_AND_COMPLETE_THRESHOLD,
};
pub use bridge_template::{
    emit_bridge_file_generated_ssb, BridgeError, BridgeFileSpec, BridgeFileTemplate, BridgeKind,
};
pub use plan_skeleton::{
    PlanSkeleton, PlanSkeletonError, PhaseEntry, PhaseContext, PhaseDetail,
    ExtractedContracts, AdjacentContracts, ContractRef, StalenessWarning,
};
// C-25 public surface: contracts crate + CLI handler need access to the
// store and core types. TemplateId / PipelineOutcomeId collide with
// `self_mod_types` re-exports — callers reach those via
// `compiled_pipeline::{TemplateId, PipelineOutcomeId}` explicitly.
pub use compiled_pipeline::{
    CompiledExecutor, CompiledExecutorError, CompiledPipeline, CompiledPipelineStore,
    CompiledPipelineStoreError, CompiledStep, ExecutionOutcome,
    TemplateCrystallizer, TemplateVariable, VariableType,
};
pub use background_task::{
    BackgroundTaskError, BackgroundTaskHandle, BackgroundTaskManager,
};
pub use session_dag::{
    SessionDag, SessionDagError, Snapshot, TrimmedMessage, TrimConfig,
    BranchRef, DagMessageRole, MergeResult, Stub,
};
// Phase B2 — Symbol Graph public surface. Re-exported at crate root so
// downstream consumers (contracts crate, dashboard impact preview,
// SurgicalTelescope follow-up) can `use elai_orchestrator::{SymbolGraph,
// ...}` without reaching into the module path.
pub use symbol_graph::{SymbolGraph, SymbolGraphError, SymbolRecord, SymbolTag};
// Phase B4 — Artifacts Pipeline public surface. Re-exported at crate root
// so downstream consumers (contract tests, follow-up SSB emitter) can
// `use elai_orchestrator::{ArtifactStore, ...}` without reaching into
// `artifacts::*`. The SSB `ArtifactPublished` payload is NOT here — its
// emission is deferred to a follow-up patch.
pub use artifacts::{ArtifactManifest, ArtifactStore, ArtifactsError, ContextArtifact};
pub mod irreversible_classifier;
pub mod micro_reflection_emitter;
pub mod mistake_book;
pub mod sidecar_reflection_adapter;
pub mod output_postprocessor;
pub mod profile_selector;
pub mod rule_loader;
pub mod config_archive;
pub mod evolution_gate;
pub mod msi;
pub mod osc;
pub mod regression;
pub mod self_mod_bridge;
pub mod self_mod_layer1a;
pub mod self_mod_proposal;
pub mod self_mod_orchestrator;
pub mod self_mod_safety;
pub mod self_mod_types;
// C-43: Phase 4H scope-freeze enforcer + manifest. Loads
// `<workspace>/scope-freeze.toml`, walks `frozen_items` against on-disk
// mtimes, and produces the 2 SSB beacons (ScopeFreezeRatified +
// ScopeFreezeViolated). ELAI design hypothesis — see
// `workflow/sessions/s-25120/mistakes.md` for the bad-cite analysis of
// the source plan's arXiv:2604.21725 citation.
pub mod scope_freeze;
// C-41: skills-driven workflow registry. SkillManifest schema +
// deterministic top-k ranking + SSB observability for the prompt-engineer
// harness-side agent to consume. Locally authored on top of the cli's
// SkillSummary / SkillRoot prior art; paper motivation comes from
// arXiv:2604.21910 (S0 -> S3 full-match accuracy 44.0% -> 83.3% on Claude
// Opus 4.6, Section 5.1 Table 2). See workflow/papers/scientific-workflow-interface.md.
pub mod skill_registry;
pub mod ssb;
// C-45: CANTANTE contrastive credit attribution. Paper-grounded against
// arXiv:2605.13295 (Zehle 2026, Section 3.2 — K rollouts NOT K^N, scalar
// c in [-1,+1], g=3 partition default). Wires per-role credit scores
// back into C-41 SkillRegistry via SkillManifest.learned_credit_score.
// See workflow/papers/cantante-credit-attribution.md for the interface.
pub mod credit_attribution;
// C-46: Uno-Orchestra parsimonious agent routing (selective delegation).
// Paper-grounded against arXiv:2605.05007 (Cui et al 2026, Sections 5 +
// 6.1 — (model, primitive) action space, alpha in [0, 1] convex blend,
// S_max <= 0.10 cost-hack bound, direct-answer lazy-mode cost zero).
// Composes on top of C-45 credit weights when available (cold-start
// safe — falls back to difficulty-only policy). See
// workflow/papers/uno-orchestra-interface.md for the interface.
pub mod routing;
// C-49: heuristic adaptation of arXiv:2605.14443 iterative-distillation
// (Sayana, Todi, Jash 2026). Per-skill SuccessTrace bin + counting-based
// fragment distiller + ExperienceCorpus LRU at MAX_TRACES_PER_SKILL=100.
// The full RL-trained prompter (GRPO + KL anchor) is RUNPOD-PENDING; this
// module ships the heuristic counting projection only. See
// workflow/papers/iterative-distillation-interface.md for the grounding
// caveats (per-skill not cross-model; buffer task-dependent; binary
// audit signal vs paper's scalar reward).
pub mod experience;
// C-50: STRUCTURAL contract test scaffolding. Paper-grounded against
// arXiv:2604.13100 (Lin, Zhao, Shi 2026 — Contract-Coding). The
// paper's kernel is K = <N, Sigma, Delta> (Section 4.2); ELAI ships
// the structural contract surface + heuristic discriminator +
// test-file scaffolder. The full Generator -> Discriminator RL
// training pipeline (paper Algorithm 1) is RUNPOD-PENDING. Citation
// discipline: the headline "47%" is the Roguelike subset of
// Greenfield-5 (Table 1) and TIES Trae; the differentiator is
// structural integrity (failures confined to Local Logic Errors)
// per Section 5.2. See workflow/papers/contract-coding-interface.md
// and workflow/sessions/s-25120/mistakes.md (5th cite incident:
// "real but misframed" pattern).
pub mod contract_coding;
// C-52: Dafny Formal Verification Spike host-side scaffolding. Paper-grounded
// against arXiv:2604.22601 (NL2VC). Three-tier prompting (contextless ->
// signature -> self_healing), uDebug vacuous-verification guard, capped
// self-healing loop, tokio::process::Command shellout to `dafny verify`.
// Live verification is RUNPOD-PENDING (local dev host lacks the dafny
// binary; the integration must NOT use Gemma 4 per memory rule
// feedback_no_gemma4 — substitutes from paper eval set include
// Codestral-22B-v0.1, GPT-OSS-120B, Qwen 3 Coder 30B). See
// workflow/papers/nl2vc-dafny-interface.md +
// workflow/reasoning/C-52-decisions.md.
pub mod dafny_verify;
// E-12 Tier-B: framework profile + adapter + registry. Mirrors the
// canonical `crate::task_runner::ScaffoldApplier` trait shape (Tier-A
// guard 1 enforces — no parallel FrameworkAdapterV2 names). Loads
// manifests from workflow/archetypes/ (canonical scope-clarity home,
// E-11.A). See workflow/reasoning/E-12-tier-b-decisions.md.
pub mod framework_profile;
// E-12 Tier-B: 15 production-checklist runner modules — wire-dark
// posture per memory:wire-dark-posture 5th application. Each runner
// ships full subprocess wires + ToolchainMissing branch + structured
// thiserror; live invocation of the 15 external CLIs (gitleaks,
// trufflehog, semgrep, npm-audit, pip-audit, cargo-audit, lighthouse,
// biome, ruff, clippy, sqlite3, cargo-machete, otel-check,
// sentry-check, lsof) is RUNPOD-PENDING.
pub mod production_checklist;
// D-1: ReasoningTemplate typed schema + Blake3 content-hash + L3-frozen
// closed-enum mutation surface. Bounded-capacity replacement for the
// string-literal reasoning patterns (DGM §H unbounded-mutation gap).
// See workflow/reasoning/D-1-decisions.md.
pub mod reasoning_template;
// D-2: PPM (Process Preference Model) sidecar hook + client. Heterogeneous
// verifier substrate per rStar-Math (arxiv:2501.04519, T1) §3.2 — separate
// improving model artefact (Qwen2.5-Math-PRM-7B) on port 8101, distinct
// from the policy SLM on port 8100. See workflow/reasoning/D-2-decisions.md.
pub mod sidecar_ppm_hook;
pub mod ppm_client;
// D-2: production HTTP impl of SidecarPpmHook. Speaks /v1/ppm/score on the
// second sidecar (port 8101). Defers to PpmClient for cache + gating; this
// module owns only the wire boundary. See workflow/reasoning/D-2-decisions.md
// "PpmClient does NOT impl RuntimeAdapter" deviation.
pub mod http_sidecar_ppm_hook;
// D-1: append-only ArchiveChain backing reasoning-template lifecycle.
// JSON Lines on disk, Blake3 self_hash + parent_hash chain. Consumed by the
// FSM gate + Tauri MutationArchivePanel.
pub mod archive_chain;
// D-19: PRMBench adversarial-gaming benchmark runner. Lives under
// `benchmarks::prmbench_runner`; siblings (D-20..D-25) land in the same
// `benchmarks` module. See workflow/reasoning/D-19-decisions.md.
pub mod benchmarks;
// D-28: orchestrator-side farm runners (adversarial example farming).
// Sibling of `benchmarks` — distinguishes data-producing runners
// (`farm/*`) from data-evaluating runners (`benchmarks/*`).
pub mod farm;
// D-3: VerificationResult aggregator + closed-enum ConsensusRule. Heterogeneous
// verifier substrate composed into a single AggregateVerdict consumed by the
// pre-promotion chain gate. See workflow/reasoning/D-3-decisions.md.
pub mod verification_result;
// D-3 (closure): Cross-context verifier — same claim, two independent context
// windows; blake3 content-hash + shingle-jaccard agreement. Produces
// CrossContextAgreement + CrossContextRun consumed by run_consensus_pass /
// task_runner's CLAIM-emit hook. See workflow/reasoning/D-3-decisions.md
// (DECISION-3) for the citation-substitution rationale.
pub mod cross_context;
// E-45: host-side cross-language drift detector. Diffs Rust orchestrator
// boundary structs against the Python sidecar + Tauri TS mirror per the
// pair registry loaded from `.elai_cc/cross-lang-pairs.toml`. Producer
// of the 3 E-45 SSB action types (CrossLangDriftDetected /
// CrossLangPairScanned / SchemaVersionMismatch). Per D-10 DECISION-8,
// the TS mirror is hand-authored — this module lints the mirror via
// presence checks, it does NOT codegen. See
// workflow/reasoning/E-45-decisions.md.
pub mod cross_lang_drift;
// E-49: coding-agent metric suite (scope-lean ship). Capture point
// (CodingMetric) + SWE-Bench Pro / RefineBench live-run boundaries
// (RUNPOD-PENDING) + per-session aggregator. Producer of 5 new SSB
// action types at SCHEMA_VERSION 138. WIRE-DARK avoidance via
// `crates/cli/src/metrics_cmd.rs`. Design source: SWE-bench Pro +
// RefineBench (citations DEFERRED to mistakes.md cluster #9). See
// workflow/reasoning/E-49-decisions.md.
pub mod coding_metrics;
// E-53: Counterfactual Safety Question (CSQ) classifier runtime
// Rust impl. CLAUDE.md prose CSQ rule remains authoritative for LLM
// behavior; this module is the runtime gate consumed by
// `crate::formal_verify::orchestrator::run` to decide whether a path
// dispatches a formal verifier. See workflow/reasoning/E-53-decisions.md.
pub mod csq;
// E-53: formal-verify pipeline orchestrator (lean ship). Composes
// C-50 contract_coding + C-52 dafny_verify into an end-to-end
// verifier dispatch routed by the csq classifier. Producer of 1 new
// SSB action type (FormalVerifierCompleted) at SCHEMA_VERSION 140.
// Gemma-4 is BLOCKED per memory rule feedback_no_gemma4; all 4
// toolchains (Z3 / Dafny / Kani / Verus) are RUNPOD-PENDING on local
// dev hosts — runtime detection at boundary returns
// `VerifierOutcome::RunpodPending`. See
// workflow/reasoning/E-53-decisions.md.
pub mod formal_verify;
// D-4: Tripwire stack — sliding-window KL + Δρ + IQR + perplexity + DDM
// detectors with auto-revert wiring into archive_chain on threshold cross.
// Consumed by agent_pool post-consensus. See module doc for paper grounding.
pub mod tripwire;
// D-5: Beta-posterior Thompson sampling for ReasoningTemplate promotion.
// Consumes D-1 ArchiveChain + StagedEvalResult; produces BaiArmUpdated +
// TemplatePromoted / TemplateCandidateRejected SSB rows via
// `bai::emit_bai_round`. arXiv:2104.05593 (fixed-confidence BAI) +
// Audibert COLT 2010 stopping rule.
pub mod bai;
// D-29: Hierarchical Bayesian BAI for non-IID rewards. Layered ABOVE D-5
// flat Beta/Thompson; partitions arms into FAMILIES with a shared
// Beta-hyperprior `theta_i ~ Beta(mu*kappa, (1-mu)*kappa)`. Hyperprior
// updated via a hand-rolled Metropolis-within-Gibbs sampler (no new
// deps; see pattern-7 in workflow/reasoning/D-29-decisions.md). Gated
// wire-dark by ELAI_BAI_MODE=hierarchical (default "flat"). Paper
// labels: arXiv:2111.06929 SUPPORTING EVIDENCE (regret minimization,
// not BAI); arXiv:2106.10417 DIRECT GROUNDING (variance-dependent BAI).
pub mod bai_hierarchical;
// D-30: Calibration corpus bootstrap warm-start. Loads a published
// calibration corpus subset (TruthfulQA arXiv:2109.07958 fixture by
// default), projects to D-16 CalibrationTuple rows, runs UniCR
// compute_tau with a lowered floor (20), and atomically persists
// tau_bootstrap.txt. Wire-dark by default — production callers consult
// ELAI_CALIBRATION_BOOTSTRAP_ENABLE (see runtime_config). Block-bootstrap
// CI on tau (Politis-Romano 1994 JASA Vol 89, §3) is shipped as a pure
// helper (`bootstrap_resample`) and held off the warm-start path until
// the SSB payload's Option fields are flipped by a SECONDARY pass.
// See workflow/reasoning/D-30-decisions.md.
pub mod calibration_bootstrap;
// D-6: Time-windowed held-out task store + 9-gram contamination filter.
// LiveCodeBench-style (arxiv:2403.07974, T1) contamination defense for
// the BAI promotion-gate eval pool. Sibling of `bai.rs`; consumed by
// `agent_pool::sample_held_out_task` which forwards into BAI as the
// eval source. Produces HeldOutTaskAdded + HeldOutTaskUsed SSB rows.
pub mod held_out;
// D-7: Cross-domain transfer canary — 5 orthogonal task families
// (CodeFix / MathProof / ProseReason / PlanningDecomposition / ToolUse).
// NanoFlux-style (arxiv:2509.23252, T1) defense: every L2 mutation
// candidate is evaluated across ALL 5 families before BAI promotion.
// Sibling of `held_out.rs` (held-out tasks feed the canary). Produces
// CanaryRunCompleted + CanaryFamilyEvaluated SSB rows via
// `canary::emit_canary_run`. Consumed by `agent_pool::recent_canary_status`
// (read surface for BAI gate + dashboard).
pub mod canary;
// C-27: Testing-trust Tier T(-1) DeterminismConfig + Tier T-0 WiringContract.
// Surface re-exports below for the CLI driver and contract crate. See
// workflow/reasoning/C-27-decisions.md.
pub mod determinism;
pub mod wiring_contract;
// D-9: L3 frozen-surface access-control auditor. Companion to the
// wiring_contract module — registry-backed file/line lock walker.
// See docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md.
pub mod locked_keys_auditor;
// E-46: Hierarchical decomposition tuning — three additive modules.
//   * dual_ledger: Magentic-One TaskLedger + ProgressLedger atomic
//     advancement primitive (arxiv:2411.04468 / arxiv:2412.21154).
//   * agentless_shortcut: AGENTLESS leaf-shortcut eligibility predicate
//     (arxiv:2407.01489 — 32.00% SWE-bench Lite at $0.70/issue).
//   * goal_drift_canary: intra-decomposition Jaccard-distance drift
//     canary (arxiv:2505.02709 — goal-drift PRM supervision).
// Producer wiring lives in `agent_pool::process_task_advancement` (free
// functions); Tauri read surface in
// `dashboard-v2/src-tauri/src/commands_e46.rs`.
pub mod dual_ledger;
pub mod agentless_shortcut;
pub mod goal_drift_canary;
// D-10: mode-routing predicate (orchestrator picks the grounding
// mode for every novel claim — the policy model has no input). The
// three modules are pure data + pure logic; D-9 LockedKeysRegistry
// freezes `select_grounding_mode` + `ClaimType` so the agent cannot
// self-modify the routing table (Cohen Boxed Myopic / Everitt
// current-RF, arxiv:1908.04734). See
// docs/thefinalglance/D-plan/D-10_mode_routing_predicate.md.
pub mod claim;
pub mod grounding_mode;
pub mod orch_context;
// D-11 — Constructive-checker registry (Mode A operational substrate).
// Lives next to `claim` + `grounding_mode` so the routing predicate
// dispatches into the same module surface. See
// docs/thefinalglance/D-plan/D-11_constructive_checker_registry.md.
// Grounding: arxiv:2310.04406 (constructive proof checking), arxiv:2211.12588
// (PoT), arxiv:2402.01030 (CodeAct).
pub mod checker_registry;
// D-12 — OperationalizationCheck baby-step predicate. Co-locates the
// typed BabyStep + MeasurableProcedure + ExpectedResult value objects
// with the `is_operationalized` predicate the auditor gates on, and
// the `dispatch_check_for_baby_step` helper that routes the
// procedure through the D-11 checker registry. See
// docs/thefinalglance/D-plan/D-12_operationalization_check_predicate.md.
// Grounding: arxiv:2305.16291 Voyager (T1) skill-with-measurable-effect;
// arxiv:2407.01489 AGENTLESS (T1) single-step verification.
pub mod baby_step;
// D-16 — UniCR conformal-refusal calibration (formal Mode E grounding).
// Pure tau-computation + decide() + SSB-emitting producer surface. The
// L3-locked TauUpdater + persisted tau.txt + CLI subcommands are deferred
// to a follow-up patch per the orchestrator-scoped MVP. See
// docs/thefinalglance/D-plan/D-16_unicr_conformal_refusal.md.
// Grounding: arXiv:2503.01345 (UniCR — unified conformal refusal),
// Vovk et al. *Algorithmic Learning in a Random World* (2005) §2.2.
pub mod conformal_calibration;
// D-17: Honest-refusal SLA monitoring. Sliding-window monitor over the
// D-16 RefusalDecision stream + classify against operator-configured
// healthy band + two SSB beacons (refusal_rate_snapshot +
// refusal_sla_breach). See workflow/reasoning/D-17-decisions.md.
// Grounding: arXiv:2503.01345 (UniCR honest refusal);
// docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §4.6.
pub mod refusal_sla;
// D-15: Discovery-vs-search labeling — pure `NoveltyClass` predicate over
// caller-supplied `NoveltyEvidence`, plus the producer surface
// (`emit_novelty_classified`) and read-side projection
// (`recent_novelty_rows`, `novelty_distribution`) the dashboard
// NoveltyPanel reads. No new external dep. Grounding: arXiv:2406.05761
// (LLM novelty detection three-bucket classification);
// docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §1 Mode C
// + §4.8. See workflow/reasoning/D-15-decisions.md.
pub mod novelty;
// D-18 — Typed ObservableEffect predicate (Mode D upgrade extension of
// C-22 FeasibilityProber). Ships closed-enum TriggerSpec /
// TypedObservation / SandboxTier + ConfoundFlags struct + SandboxExecutor
// trait (native async-fn-in-trait via Pin<Box<dyn Future>>, no
// async-trait crate per CLAUDE.md anti-stale-training) +
// verify_observable_effect producer emitting one
// `observable_effect_verified` SSB row per dispatch. Plan:
// docs/thefinalglance/D-plan/D-18_observable_effect_typed.md (full
// EffectSet + orthogonality + sandbox-vs-production labeling deferred
// to a follow-up session; this pass ships the minimum-viable typed
// predicate end-to-end including dashboard wiring).
pub mod observable_effect;
// C-28: Testing-trust Tier T-1 mutation testing — MutationRunner,
// MutationResult, EquivalentMutantCache, MutationConfig. Lives next to
// `wiring_contract` so the deterministic_auditor can consult both. See
// workflow/reasoning/C-28-decisions.md.
pub mod mutation_testing;
// C-29: Testing-Trust Tier T-3 — basic metamorphic relation library
// (Identity / Inverse / PermutationInvariance). Hooked into the
// deterministic_auditor as Step 7 (post-Mutation). See
// workflow/reasoning/C-29-decisions.md.
pub mod metamorphic_testing;
// E-13 — TDD verification agents module (Red/Green/Refactor cycle +
// anti-mock heuristic + cargo-mutants subprocess wrapper +
// D-3-aggregator adapter). Composes with `verification_result`'s
// `ConsensusRule::VerifierKindHeterogeneous` so we don't ship a
// parallel aggregator. See workflow/reasoning/E-13-decisions.md.
pub mod tdd_verifier;
// E-18 — Tool-use reliability gate. Reuse-heavy composition over D-11
// RegistryCheck + E-35 SearchReplaceMatcher / SidecarXGrammarHook +
// C-47 ToolCalibrationRegistry + C-42 AuthzPolicy + C-35 ToolGate.
// Three-way verdict (Pass / Reject / Spill) consulted AFTER the
// calibration step so the case base sees the same call shape regardless
// of gate outcome. See workflow/reasoning/E-18-decisions.md.
pub mod tool_reliability;
pub mod staged_eval;
pub mod ssb_viewer;
// B-37C Phase 4M.5 — per-AgentRole runtime backend mapping +
// `RuntimeOrchestrator` with lazy adapter cache + fallback chain. The
// production wiring into `task_runner.rs` is deferred to B-37D; the
// library surface ships now so dashboards / CLI commands have a stable
// target to program against.
pub mod per_role_runtime;
pub mod runtime_config;
pub mod runtime_event_sink_ssb;
pub mod runtime_orchestrator;
// E-32: deep-mode escalation hook for the SessionDag handoff threshold.
// Decides between local deep mode (Qwen2.5-7B-Instruct-1M, DCA, 64K) and
// cloud escalation when context budget approaches 85% of effective_max_context.
// Source: E-32-decisions.md DECISION 3 + 4; arxiv:2407.16833 Self-Route.
pub mod escalation_policy;
// C-1: orchestrator-side adapter that bridges
// `elai_knowledge::knowledge_ssb_sink::KnowledgeSsbSink` into
// `Ssb::write_payload`. Lives here (not in `knowledge`) so the
// orchestrator -> knowledge dependency direction stays acyclic.
pub mod knowledge_ssb_adapter;
// C-35: orchestrator-side adapter that bridges the plugins-crate
// `ToolGateSsbEmitter` trait into `Ssb::write_payload`. Lives here (not
// in `plugins`) so the plugins -> orchestrator dependency direction
// stays acyclic. Symmetric to `knowledge_ssb_adapter`.
#[cfg(feature = "mcp")]
pub mod tool_gate_ssb_adapter;
// C-47: orchestrator-side adapter that bridges the plugins-crate
// `CaseBaseSsbEmitter` trait (case-based tool-use calibration telemetry)
// into `Ssb::write_payload`. Same dependency-inversion pattern as
// `tool_gate_ssb_adapter`. Paper-grounded against arXiv:2605.15041
// (CAST). See workflow/papers/case-based-calibration.md.
#[cfg(feature = "mcp")]
pub mod tool_calibration_ssb_adapter;
// E-18: orchestrator-side adapter that bridges the plugins-crate
// `ReliabilityGateSsbEmitter` trait (tool-use reliability gate
// telemetry) into `Ssb::write_payload`. Same dependency-inversion
// pattern as `tool_calibration_ssb_adapter`. Composes on top of D-11
// RegistryCheck + E-35 SearchReplaceMatcher / SidecarXGrammarHook
// + C-47 ToolCalibrationRegistry + C-42 AuthzPolicy + C-35 ToolGate
// rather than re-implementing those surfaces. See
// workflow/reasoning/E-18-decisions.md.
#[cfg(feature = "mcp")]
pub mod tool_reliability_ssb_adapter;
// B-24 Phase 4I (Group C1): media validator skeleton. See
// `media_validator/mod.rs` for the per-group rollout map.
pub mod media_validator;
pub mod system_probe;
// Phase B2 — Symbol Graph Impact runtime (library skeleton). Lazy reader
// for `.claude/tools/elai.map`; consumed by the SurgicalTelescope blast-radius
// upgrade in a follow-up phase. See `symbol_graph/legacy_b2.rs` for the
// original typed surface and `crates/contracts/tests/symbol_graph_contract.rs`
// for the contract.
//
// E-40 (SCHEMA_VERSION 134): the module is now a directory housing both
// the original B2 surface (re-exported via legacy_b2) and the new
// JSON-rooted SymbolGraphCache + query helpers + personalized PageRank +
// SCIP-import wrapper + LSP oracle adapter. Sibling surface to
// elai_knowledge::symbol_graph (SQLite-rooted SymbolGraphCache) and
// elai_knowledge::pagerank (DKG-snapshot rooted whole-repo map); all
// three coexist via distinct crate paths. See
// workflow/reasoning/E-40-decisions.md.
pub mod symbol_graph;
pub mod telescope_caller;
pub mod feasibility_probe;
/// C-30 — Surgical Mode pipeline: DKG blast-radius decomposition + LOC gates.
pub mod surgical_decomposer;
/// C-30 — Oracle answer-level cache (TTL by SourceTier).
pub mod oracle_cache;
pub mod task_runner;
// Phase A1b — production task_runner wiring helpers (build/apply chokepoint
// bundle). Used by `elai run` CLI and the dashboard-v2 Tauri setup() callback.
pub mod runtime_setup;
/// C-13 — WIP commit squash helper.
pub mod wip_squash;
// Phase B2 — IPC accept loop: bind/accept/route cycle for orchestrator-side
// agent session management. Exposes AgentRpcServer trait + InferOutcome +
// ToolOutcome + bind_session_listener + accept_one + run_agent_session.
pub mod ipc_accept;
// Phase B2c — ProductionAgentRpcServer: impls AgentRpcServer wrapping
// AnthropicClient (sidecar or cloud via ELAI_GROUNDING_BACKEND) and
// InProcessToolDispatcher.
pub mod production_agent_rpc;
// D-14 — sycophancy probe substrate (Sharma et al. arXiv:2310.13548).
// Defensive detector: flags suspected sycophancy via inverted-framing
// challenge per §3.2; does NOT auto-correct (PM does not reliably fix
// sycophancy per §4 — 75% PM residual, 25% oracle floor).
pub mod sycophancy;
pub mod supervision;
pub mod task_packet;
pub mod task_registry;
pub mod team_cron_registry;
pub mod worker_boot;
// Phase 4J / B-25 — Empire integration (host-side scaffolding only). Gated
// behind cfg(feature = "empire") so non-empire builds pay zero compile-time
// or runtime cost (workspace-level `cargo build --no-default-features`
// excludes the submodule entirely). Every external-IO seam inside is
// labelled `EMPIRE-PENDING: <what-to-wire>` so a single follow-up patch can
// locate and flip every wire-point when Empire deploys. See module doc for
// the layout + decision-doc workflow/reasoning/phase-4J-decisions.md.
#[cfg(feature = "empire")]
pub mod empire;

// Re-export modules from external crates under their old names so that
// internal `crate::config::*`, `crate::sandbox::*`, etc. paths still resolve.
pub use elai_core::config;
pub mod json {
    pub use elai_core::json::{JsonError, JsonValue};
}
pub mod usage {
    // C-14 — TokenUsage / UsageTracker stay in elai_core (raw counter primitives).
    // Pricing/cost helpers moved to elai_providers::pricing (provider-layer concern).
    pub use elai_core::{TokenUsage, UsageTracker};
    pub use elai_providers::pricing::{
        estimate_cost_usd, estimate_cost_usd_with_pricing, summary_lines, summary_lines_for_model,
    };
    pub use elai_providers::{
        format_usd, pricing_for_model, ModelPricing, UsageCostEstimate,
    };
}
pub mod bootstrap {
    pub use elai_core::{BootstrapPhase, BootstrapPlan};
}
pub mod oauth {
    pub use elai_providers::oauth::*;
}
pub mod telemetry_types {
    pub use elai_providers::telemetry_types::*;
}
pub mod sse {
    pub use elai_providers::{parse_frame, SseParser};
}
pub mod green_contract {
    pub use elai_rules::green_contract::*;
}
pub mod policy_engine {
    pub use elai_rules::policy_engine::*;
}
#[cfg(test)]
pub mod trust_resolver {
    pub use elai_rules::trust_resolver::*;
}

// Usage types re-exported at crate root (many files use `crate::TokenUsage` etc.).
// C-14 — pricing/cost helpers now sourced from elai_providers; counters from elai_core.
pub use elai_core::{TokenUsage, UsageTracker};
pub use elai_providers::{
    format_usd, pricing_for_model, ModelPricing, UsageCostEstimate,
};

pub use bash::{execute_bash, BashCommandInput, BashCommandOutput};
pub use fsm::{
    EvidenceArtifact, Fsm, FsmError, FsmState, FsmSubstate, PersistedFsmState, ReplanLogEntry,
    ReplanRequest, ReplanSeverity, TransitionRecord,
};
// B-40 — Idea-Framer public surface.
pub use idea_framing::{
    classify_project_class, classify_project_class_with_pattern,
    dispatch_multi_root_research, frame_idea, load_research_preset,
    load_research_presets_from_project_root, parse_research_presets_from_toml,
    run_critic_loop, ALL_CRITIC_CATEGORIES, ALL_PROJECT_CLASSES,
    IDEA_FRAMER_GRC_BUDGET_CALLS, IDEA_FRAMER_GRC_BUDGET_CHARS, MAX_CRITIC_ROUNDS,
    NUM_CRITIC_CATEGORIES,
};
pub use idea_framing::{
    ClassPreset, CriticCategory, CriticFinding, CriticRound, FramedIdea, IdeaFramingError,
    KeyDecision, ProjectClass, ResearchPresets, ResearchStream, ResearchSummary, Severity,
    SourceTier, StreamSummary, Tradeoff, TradeoffCard,
};
pub use evidence::{
    all_gates, check_transition, daira_adaptive_run, gates_for_transition, AuditFinding,
    DairaScope, DairaScopeResult, DkgChangelogGate, EvidenceError, EvidenceGate,
    FindingSeverity, GroundingSweepGate, IntentPairsGate, KnowledgeReadiness,
    KnowledgeReadinessGate, ReplanFastTrackGate, TestFailReasonsGate, TestFlippedGate,
    TestOutcome, TransitionContext, ZeroBlocksGate,
};
pub use gate::{
    default_risk_level, run_interactive_gate, EvidenceSummary, GateDecision, GateResult,
    InteractiveGate, InteractiveGateOutcome, RiskLevel,
};
#[cfg(test)]
pub use gate::MockInteractiveGate;
// Batch 3B (Phase 4G) — AutonomousGate surface.
pub use autonomous_gate::{
    autonomous_gate_disabled_via_env, AutonomousGate, AutonomousGateConfig, AutonomyDecision,
    EscalationTrigger, Evidence as AutonomousEvidence, GateContext as AutonomousGateContext,
    GateMode, PipelineLevel, TreeAutoAdvancePolicy, ELAI_DISABLE_AUTONOMOUS_GATE_ENV,
};
// C-9 (Autonomous Mode Core Infrastructure) — additional surface.
pub use autonomous_gate::{
    AutoAdvanceEvaluator, EscalationPolicy, EvidenceCheck, EvidenceVerdict, FsmTransition,
    MergeStrategy, SplitTaskPlan, SubTask,
};
// C-10 (Phase 4G.9 Anti-Spiral Primitives) — task-runner-step-level
// detector + persona switcher + canary + watchdog.
pub use anti_spiral_extended::{
    strategy_for, suggest_persona_for_pattern, AdaptiveFutilityTracker, AttemptResult,
    DriftVerdict, ExecutionHistory, GenerationWatchdog, GoalDriftCanary, PersonaStrategy,
    StagnationDetection, StagnationPattern, ThinkingPersona, WatchdogTimeoutKind,
    DIMINISHING_MIN_IMPROVEMENT_PCT, DIMINISHING_MIN_WINDOW, FUTILITY_MIN_ATTEMPTS,
    FUTILITY_THRESHOLD, HISTORY_CAPACITY, IDLE_TIMEOUT_MS, JACCARD_GREEN, JACCARD_YELLOW,
    NO_PROGRESS_TIMEOUT_MS, OSC_WINDOW, PERSONA_STRATEGIES, SAFETY_TIMEOUT_MS, SPIN_THRESHOLD,
};
// B-41 — per-level audit evidence helpers (consumed by
// autonomous_gate_with_per_level_contract.rs + downstream gate callers).
pub use task_runner::{
    derive_babystep_zero_blocks, derive_concept_zero_blocks, derive_phase_zero_blocks,
    derive_plan_zero_blocks, derive_step_zero_blocks,
};
pub use tool_filter::{RuleCategory, ToolCategory};
pub use dispatcher::{
    BackendConfig, BackendMode, DispatchContext, DispatchError, PhaseSlot,
    build_dispatch_context, CONSTRAINTS_BLOCK_HEADER, LOCAL_MODEL_MAX_CONTEXT,
    LOCAL_BUDGET_FRACTION,
};
pub use elai_core::{BootstrapPhase, BootstrapPlan};
// Batch 2B: re-export ToolDispatcher trait + value types so downstream
// callers (dashboard, contracts) program against `elai_orchestrator::{...}`
// without a direct `elai_core::tool_dispatcher::` path. The concrete
// `InProcessToolDispatcher` impl lives in `elai-tools` and is imported
// there directly (elai-tools depends on elai-orchestrator for facade
// re-exports, so the dep direction blocks a tools re-export here).
pub use elai_core::tool_dispatcher::{
    ClassifiedFailure as ToolClassifiedFailure, CommitOutcome, EditMatchTier, EditOutcome,
    EditRequest, FailureInput, FileView, GitBranchPolicy, GitCommitRequest, IncrementalOutcome,
    IncrementalRequest, ToolDispatchError, ToolDispatcher, ToolFailureCategory, ValidateRequest,
    ValidationTier, ValidationTierResult,
};
pub use branch_lock::{detect_branch_lock_collisions, BranchLockCollision, BranchLockIntent};
pub use compact::{
    compact_session, estimate_session_tokens, format_compact_summary,
    get_compact_continuation_message, should_compact, CompactionConfig, CompactionResult,
};
pub use elai_core::config::{
    ConfigEntry, ConfigError, ConfigLoader, ConfigSource, McpConfigCollection,
    McpManagedProxyServerConfig, McpOAuthConfig, McpRemoteServerConfig, McpSdkServerConfig,
    McpServerConfig, McpStdioServerConfig, McpTransport, McpWebSocketServerConfig, OAuthConfig,
    ResolvedPermissionMode, RuntimeConfig, RuntimeFeatureConfig, RuntimeHookConfig,
    RuntimePermissionRuleConfig, RuntimePluginConfig, ScopedMcpServerConfig,
    ELAI_SETTINGS_SCHEMA_NAME,
};
pub use conversation::{
    auto_compaction_threshold_from_env, ApiClient, ApiRequest, AssistantEvent, AutoCompactionEvent,
    ConversationRuntime, LabeledText, PromptCacheEvent, RuntimeError, StaticToolExecutor,
    TextSource, ToolError, ToolExecutor, TurnSummary,
};
pub use hooks::{
    HookAbortSignal, HookEvent, HookProgressEvent, HookProgressReporter, HookRunResult, HookRunner,
    StopHookDecision, StopHookGate, STOP_HOOK_DOUBLE_PRESS_WINDOW_MS,
};
pub use hook_dispatcher::{
    emit_hook_events_to_ssb, emit_hook_latency_violation_to_ssb,
    emit_marker_report_to_ssb, emit_shortcoder_compression_to_ssb,
    emit_tiered_analysis_summary_to_ssb, hook_tier_budget_ms, report_hook_latency,
    sanitized_block_message, HookDispatcher, HookError as HookDispatcherError, HookFinding,
    HookInvocation, HookPoint, HookTier, HookVerdictKind, InProcessHookDispatcher,
    IpcHookAsyncAdapter, PreCompactHandoffSaver, ELAI_HOOKS_DISABLED_ENV,
    HOOK_EVENT_CATEGORY, HOOK_T1_BUDGET_MS, HOOK_T2_BUDGET_MS, HOOK_T3_BUDGET_MS,
    PRE_COMPACT_HANDOFF_FRACTION, SANITIZED_BLOCK_MESSAGE,
};
pub use shortcoder_hook::{
    ShortCoderHook, ShortCoderHookError, ShortCoderOutcome, ELAI_DISABLE_SHORTCODER_ENV,
    SHORTCODER_MAX_RULES_EMITTED,
};
pub use provider_signals::{
    provider_signals_disabled_via_env, EtcState, GgSignals, NoveltySignal,
    NullProviderSignalReader, ProviderSignalReader, SearchTrigger, SteeringVectorStatus,
    VramDetectionSource, VramTierConfig as ProviderVramTierConfig,
    ELAI_DISABLE_PROVIDER_SIGNALS_ENV,
};
#[cfg(feature = "sidecar")]
pub use provider_signals::SidecarProviderSignalReader;
pub use lane_events::{
    dedupe_superseded_commit_events, LaneCommitProvenance, LaneEvent, LaneEventBlocker,
    LaneEventName, LaneEventStatus, LaneFailureClass,
};
pub use mcp::{
    mcp_server_signature, mcp_tool_name, mcp_tool_prefix, normalize_name_for_mcp,
    scoped_mcp_config_hash, unwrap_ccr_proxy_url,
};
pub use mcp_client::{
    McpClientAuth, McpClientBootstrap, McpClientTransport, McpManagedProxyTransport,
    McpRemoteTransport, McpSdkTransport, McpStdioTransport,
};
pub use mcp_lifecycle_hardened::{
    McpDegradedReport, McpErrorSurface, McpFailedServer, McpLifecyclePhase, McpLifecycleState,
    McpLifecycleValidator, McpPhaseResult,
};
pub use mcp_stdio::{
    spawn_mcp_stdio_process, JsonRpcError, JsonRpcId, JsonRpcRequest, JsonRpcResponse,
    ManagedMcpTool, McpDiscoveryFailure, McpInitializeClientInfo, McpInitializeParams,
    McpInitializeResult, McpInitializeServerInfo, McpListResourcesParams, McpListResourcesResult,
    McpListToolsParams, McpListToolsResult, McpReadResourceParams, McpReadResourceResult,
    McpResource, McpResourceContents, McpServerManager, McpServerManagerError, McpStdioProcess,
    McpTool, McpToolCallContent, McpToolCallParams, McpToolCallResult, McpToolDiscoveryReport,
    UnsupportedMcpServer,
};
pub use elai_providers::oauth::{
    clear_oauth_credentials, code_challenge_s256, credentials_path, generate_pkce_pair,
    generate_state, load_oauth_credentials, loopback_redirect_uri, parse_oauth_callback_query,
    parse_oauth_callback_request_target, save_oauth_credentials, OAuthAuthorizationRequest,
    OAuthCallbackParams, OAuthRefreshRequest, OAuthTokenExchangeRequest, OAuthTokenSet,
    PkceChallengeMethod, PkceCodePair,
};
pub use permissions::{
    PermissionContext, PermissionMode, PermissionOutcome, PermissionOverride, PermissionPolicy,
    PermissionPromptDecision, PermissionPrompter, PermissionRequest,
};
pub use plugin_lifecycle::{
    DegradedMode, DiscoveryResult, PluginHealthcheck, PluginLifecycle, PluginLifecycleEvent,
    PluginState, ResourceInfo, ServerHealth, ServerStatus, ToolInfo,
};
pub use elai_rules::policy_engine::{
    evaluate, DiffScope, GreenLevel, LaneBlocker, LaneContext, PolicyAction, PolicyCondition,
    PolicyEngine, PolicyRule, ReconcileReason, ReviewStatus,
};
pub use prompt::{
    load_system_prompt, prepend_bullets, ContextFile, ProjectContext, PromptBuildError,
    SystemPromptBuilder, FRONTIER_MODEL_NAME, SYSTEM_PROMPT_DYNAMIC_BOUNDARY,
};
pub use research_pipeline::{
    run_research_pipeline, phase_a_formulate_queries, phase_b_search, phase_c_classify,
    phase_d_extract_facts, phase_e_cross_verify, phase_f_synthesize, phase_g_post_verify,
    ClassifiedResults, CollectedResults, FormulatedQueries, PostVerificationResult,
    SearchBackend, SearchResult, StubSearchBackend,
};
pub use grounding_client::{
    CloudGroundingClient, GroundingClient, GroundingError, ResearchLevel,
    SidecarGroundingClient,
    // C-16: production cosmos research dispatcher + SSB emitter adapter.
    HttpCosmosResearchDispatcher, SsbEmitterAdapter,
    CLOUD_DEFAULT_MODEL, CLOUD_GROUNDING_MAX_TOKENS,
    COSMOS_SSB_AGENT_ID, COSMOS_SSB_SUPERSTEP,
};
pub use searx_search_backend::{SearxSearchBackend, DEFAULT_SIDECAR_SEARCH_URL};
pub use pipeline_builder::{
    PipelineBuilder, PipelineBuilderError, PipelineId, NodeIndex, StateId, DataRef, RuleId,
    ExecutablePipeline, DependencyGraph, DagNode, DagRepairLoop, PipelineFsmState,
    AgentPhaseConfig, RuleBinding, ResourceEstimate, ResourceReq, TestHarnessSkeleton,
    TestCase, FixtureStub, RepairAction, RepairOutcome, ValidationError, ValidationErrorKind,
    StructuralValidator, FsmStateGenerator, TestHarnessGenerator, ResourceEstimator,
    NoveltyDetector, AgentConfigurator, RuleInstaller, ToolLookup, PredicateCompiler,
    PredicateRef, T_DAG, CONTRACT_OVERHEAD, SPLIT_RATIO, MAX_SPLIT_DEPTH,
};
pub use recovery_recipes::{
    attempt_recovery, recipe_for, EscalationPolicy as RecoveryEscalationPolicy, FailureScenario,
    RecoveryContext, RecoveryEvent, RecoveryRecipe, RecoveryResult, RecoveryStep,
};
pub use elai_core::sandbox::{
    build_linux_sandbox_command, detect_container_environment, detect_container_environment_from,
    resolve_sandbox_status, resolve_sandbox_status_for_request, ContainerEnvironment,
    FilesystemIsolationMode, LinuxSandboxCommand, SandboxConfig, SandboxDetectionInputs,
    SandboxRequest, SandboxStatus,
};
pub use sandbox::{
    AgentEnvironment, AgentProcess, AgentSandbox, AppContainerConfig, JobObjectLimits,
    PlatformSandboxHandle, SandboxError, SandboxHandle, SelfModificationLocks, create_sandbox,
};
// Phase B1 — Tier C Sandbox policy surface. Re-exported at crate root
// so the in-process ToolDispatcher (lives in `elai-tools`) and contract
// tests can program against `elai_orchestrator::SandboxAdapter` without
// the `::sandbox_policy::` path. Distinct from `sandbox::AgentSandbox`
// (per-process isolation runtime) — see `sandbox_policy.rs` doc-comment
// for the two-layer split.
pub use sandbox_policy::{
    PassthroughSandbox, SandboxAdapter, SandboxDenied, SandboxOp, SandboxPolicy,
    SandboxProfile,
};
#[cfg(target_os = "windows")]
pub use sandbox::{WindowsAppContainerSandbox, WindowsSandboxHandle};
#[cfg(target_os = "linux")]
pub use sandbox::{LinuxSandbox, LinuxSandboxHandle};
pub use ipc_gateway::{
    AgentConnection, FrameRouteResult, GroundingMethod, IpcGateway, IpcGatewayError,
    allowed_grounding_methods, create_session_socket_dir,
};
pub use session::{
    ContentBlock, ConversationMessage, MessageRole, Session, SessionCompaction, SessionError,
    SessionFork,
};
pub use task_packet::{validate_packet, TaskPacket, TaskPacketValidationError, ValidatedPacket};
#[cfg(test)]
pub use elai_rules::trust_resolver::{TrustConfig, TrustDecision, TrustEvent, TrustPolicy, TrustResolver};
pub use elai_providers::telemetry_types::{
    AnalyticsEvent, AnthropicRequestProfile, ClientIdentity, JsonlTelemetrySink,
    MemoryTelemetrySink, SessionTraceRecord, SessionTracer, TelemetryEvent, TelemetrySink,
    DEFAULT_ANTHROPIC_VERSION,
};
pub use anti_spiral::{AntiSpiralDetector, SpiralResult, ESCALATION_DEPTH, ESCALATION_MESSAGE, HARD_STOP_DEPTH, RING_SIZE, SPIRAL_THRESHOLD};
pub use deterministic_audit_gate::{
    ChangedFilesSupplier, DefaultDeterministicAuditGate, DeterministicAuditGate,
    ProjectProfileSupplier, GATE_AGENT_ID, GATE_FINDING_CATEGORY, GATE_SUPERSTEP,
};
pub use deterministic_auditor::{
    CheckKind, DeterministicAuditResult, DeterministicAuditor, DeterministicAuditorConfig,
    DeterministicFinding, DeterministicSeverity,
};
pub use agent_pool::{
    default_grc_config, role_default_allowed_paths, AgentHandle, AgentMessage, AgentPool,
    AgentPoolConfig, AgentPoolError, AgentRole, AgentStatus, InProcessAgentPool, ModelPool,
    VRAM_MIN_BYTES,
    // D-2: PPM verifier-consensus call site.
    run_ppm_verifier_check,
    // D-10: mode-routing producer surface (`route_claim` wraps the
    // pure predicate, emits the SSB row, returns an RAII guard).
    recent_claim_routings, route_claim, ClaimRoutedEmitter, ClaimRoutedRow, EmissionGuard,
    VecClaimRoutedEmitter,
    // D-11: constructive-checker dispatch producer surface
    // (`dispatch_constructive_checkers` runs the registry + emits
    // one `checker_result_emitted` SSB row per result).
    dispatch_constructive_checkers, recent_checker_results, CheckerResultEmitter,
    CheckerResultRow, VecCheckerResultEmitter,
    // D-12: OperationalizationCheck baby-step predicate gate +
    // `baby_step_dispatched` SSB row emitter + dashboard read surface.
    emit_baby_step_dispatched, recent_baby_step_dispatches, run_operational_check,
    BabyStepDispatchedEmitter, BabyStepDispatchedRow, OperationalError,
    VecBabyStepDispatchedEmitter,
};
pub use agent_template::{
    build_agent_prompt, build_agent_system_preamble, AgentOutputFormat, AgentPromptContext,
    AgentTemplateConfig, AgentTypeConfig,
};
pub use grc::{
    pipeline_grc_disabled_via_env, PipelineGrc, PipelineGrcError,
    ELAI_DISABLE_PIPELINE_GRC_ENV,
};
pub use output_postprocessor::{process as postprocess_output, OutputConfig};
pub use worker_boot::{
    Worker, WorkerEvent, WorkerEventKind, WorkerEventPayload, WorkerFailure, WorkerFailureKind,
    WorkerPromptTarget, WorkerReadySnapshot, WorkerRegistry, WorkerStatus, WorkerTrustResolution,
};
pub use ssb::{
    default_ssb_path, default_ssb_path_for_base, DairaTracePayload, DiffSummary, DkgReadyPayload,
    FailureClassifiedPayload, FeasibilityReportPayload, GitCommitPayload, HookEventPayload,
    MarkerReportPayload,
    IndexingProgressPayload, CredentialField, InterventionCredentials,
    InterventionKeyword, InterventionOption, InterventionRequestPayload,
    InterventionResponsePayload, InterventionRetry, InterventionType, MistakeEntryPayload,
    NovelStepDetectedPayload, PhaseCompletePayload, PreviewAvailablePayload, ProviderSignalsPayload,
    SearchTriggerFiredPayload, Ssb, SsbActionType, SsbConfig, SsbEntry, SsbError, SsbFileEntry,
    FewShotExample, FindingPayload, SsbPayload, SsbReader, SsbWriteRequest, StubRunPayload,
    SystemProbePayload, TaskSubmittedPayload, ToolDispatchPayload, ValidationTierPayload,
    VramTierDetectedPayload, WebResearchStepPayload, SSB_DB_FILE_NAME, SSB_SESSIONS_SUBDIR,
    // Batch 3B (Phase 4G).
    EscalationFiredPayload, GateDecisionPayload, ModeChangedPayload, TaskSplitPayload,
    // Batch 3C (Phase 4H) — Layer 1A.
    ConfigArchivedPayload, MsiUpdatePayload, ProposalAppliedPayload, ProposalGeneratedPayload,
    ProposalRejectedPayload,
    // Batch 4C — scattered A-step wiring.
    ConfigWarningPayload, FsmTransitionPayload, MicroReflectionPayload,
    ProfileSelectedPayload, TelescopeCollapsePayload,
    // Batch-fu — cross-cutting follow-up wiring.
    ConstraintsInjectedPayload, InterventionResolvedPayload,
    // Batch-ful-a — DkgQuerierAdapter hot-swap.
    DkgQuerierSwappedPayload,
    // Batch-ful-c — A-8 freshness steering vector.
    SteeringComputedPayload, SteeringComputeFailedPayload,
    // Batch-ful-d — external MCP server lifecycle.
    McpServerConnectedPayload, McpServerDisconnectedPayload, McpToolCalledPayload,
    // Tier 1 CHAT window — dashboard-originated message + card rows.
    ChatMessagePayload, ChatResponsePayload,
    // Tier 4 Settings reorg (§8) — execution-mode selection row.
    ExecutionModeChangedPayload,
    // Track A.2 — synthesis path selection row.
    ResearchPathChosenPayload,
    // Track A.6 — grounding cache hit/miss event row.
    GroundingCacheEventPayload,
    // B-10 (DECISION-C) — per-dispatch ContextBudget snapshot payload re-exported
    // so dashboard commands.rs can read real per-slot allocations from SSB.
    ContextBudgetComputedPayload,
    // B-6 — orchestration producer wiring payloads.
    AgentLifecyclePayload, AntiSpiralHitPayload, BspSuperstepPayload,
    EvidenceGateEvaluatedPayload, SupervisionEventPayload,
    // B-27 — cloud-mode spend ledger + escalation marker.
    CloudSpendPayload, EscalateToCloudPayload,
    // B-23 Phase 4H Group A — self-mod beacons (Layer 1A growth + Layer 3 +
    // OSC + REG + EVO).
    ConfigOscillationPayload, ConvergenceStagnantPayload, DataTableGrewPayload,
    EvolutionWithheldPayload, L3ProposalPayload, RegressionDetectedPayload,
    // B-39 chat reflect pipeline — per-turn REFLECT stage beacons.
    ReflectAuditVerdictPayload, ReflectCandidatePayload, ReflectRefinedPayload,
    ReflectTriagedPayload,
    // B-24 Phase 4I (Group C1) — media-validator beacons (re-exported so
    // dashboard-v2 can deserialize SSB rows by payload type without pulling
    // in the orchestrator's internal `ssb` module path).
    MediaValidationStartedPayload, MediaTier1CompletePayload,
    MediaTier2CompletePayload, RenderCompletePayload,
    VisualFeedbackIterationPayload, MediaHumanReviewRequiredPayload,
    // B-28 focus-tunnel beacons (re-exported so dashboard-v2 commands can
    // decode SSB rows without pulling in `orchestrator::ssb` directly).
    GenerationModeSelectedPayload, StreamingEarlyTerminatedPayload,
    PipelinePreAssembledPayload, DependencyResolvedPayload, MicroStepCreatedPayload,
    // B-30 (SCHEMA_VERSION 36): language grammar registry status beacon.
    LanguageGrammarStatusPayload,
    // B-29 Phase 5 (SCHEMA_VERSION 37): symbol-graph tool call beacon.
    SymbolQueryPayload,
    // B-32 (SCHEMA_VERSION 38): minified-file chunking beacon.
    MinifiedFileChunkedPayload,
    // B-31.5 (SCHEMA_VERSION 39): 5 deferred B-31 SSB beacons. Re-exported
    // so dashboard-v2 commands + tests outside the orchestrator crate can
    // decode SSB rows without reaching into `orchestrator::ssb::*` directly.
    IndexCheckpointPayload, IndexResumePayload, IndexCancelPayload,
    LockAcquiredPayload, LockContestedPayload,
    // B-33 follow-up (SCHEMA_VERSION 40): symbol-graph build event row.
    // Re-exported so dashboard-v2 GraphWindow listener + contracts crate
    // tests can reach the payload struct without `::ssb::*` paths.
    SymbolGraphUpdatedPayload,
    // B-40 (SCHEMA_VERSION 41): FRAMING substate beacons. Re-exported so
    // dashboard-v2 IdeaFramingWindow + MonitorWindow critic-rounds badge +
    // contract tests can decode SSB rows without `::ssb::*` paths.
    IdeaFramingStartedPayload, ProjectClassClassifiedPayload,
    CriticRoundCompletePayload, IdeaFramedPayload, TradeoffCardEmittedPayload,
    // B-37C (SCHEMA_VERSION 46): multi-runtime backend beacons. Re-exported
    // so dashboard-v2 RuntimePanel + contract tests can decode rows
    // without `::ssb::*` paths.
    RuntimeFallbackPayload, RuntimeHealthChangedPayload,
    CliSubprocessSpawnedPayload, CliSubprocessKilledPayload,
    // C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
    // Re-exported so dashboard-v2 KnowledgeDrawer / ReasoningDrawer Tauri
    // commands + contract tests can decode rows without `::ssb::*` paths.
    GrcStepEmittedPayload, GroundingSweepResultPayload, CsqEvaluatedPayload,
    AuditorRubberStampRejectedPayload, HydeQueryPayload,
    VectorCacheHitRatePayload, QueryTracePayload,
    // C-37 (SCHEMA_VERSION 106): per-role decoding policy applied at
    // agent_pool::spawn. Re-exported so contract tests + dashboard
    // observers can decode the row without `::ssb::*` paths.
    PerRoleDecodingAppliedPayload,
    // C-7 (SCHEMA_VERSION 49): reviewer verdict beacon. Re-exported so the
    // dashboard-v2 accept_change / decline_change / retry_audit Tauri
    // commands + the c7_change_decision_contract test can construct the
    // payload without reaching into `orchestrator::ssb::*` directly.
    ChangeDecisionPayload,
    // C-8 (SCHEMA_VERSION 50): search-mode cross-process bridge. Re-exported
    // so the dashboard-v2 set_search_mode Tauri command + the c8 stub
    // eradication contract test can construct the payload without
    // reaching into `orchestrator::ssb::*` directly.
    SearchModeChangedPayload,
    // B-42 (SCHEMA_VERSION 44): cosmos tier knowledge worker beacons.
    // Re-exported here for the live_cli C-8 tick scheduler + contract tests
    // so callers can construct payloads without `::ssb::*` paths.
    CosmosRefreshStartedPayload, CosmosArtifactUpdatedPayload,
    CosmosArtifactDecayedPayload, CosmosGroundingUsedPayload,
    // C-2 / C-10 (SCHEMA_VERSION 48 + 52): orchestration + stagnation beacons.
    // Re-exported so the dashboard-v2 get_stagnation_events Tauri command
    // + the c10_anti_spiral_contract test can construct payloads without
    // reaching into `orchestrator::ssb::*` directly.
    StagnationSpinningDetectedPayload, StagnationOscillationDetectedPayload,
    LateralThinkingActivatedPayload, GoalDriftWarningPayload, WatchdogTimeoutPayload,
    LateralThinkingSucceededPayload, LateralThinkingExhaustedPayload, GoalDriftBlockPayload,
    // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons. Re-exported
    // so the dashboard-v2 branch_fork / branch_merge / branch_close Tauri
    // commands + contract tests can construct the payloads without
    // reaching into `orchestrator::ssb::*` directly.
    BranchForkedPayload, BranchMergedPayload, BranchClosedPayload,
    // SSB schema version + string-to-enum helper (used by contract tests).
    SCHEMA_VERSION_PUBLIC, ssb_action_type_from_str_public,
    // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons.
    // Re-exported so the contract crate + cli/wiring_audit.rs can construct
    // payloads without reaching into `orchestrator::ssb::*` directly.
    WiringEvidencePayload, WiringViolationPayload, DeterminismDriftPayload,
    WiringViolationKind,
    // D-2 (SCHEMA_VERSION 66): PPM verifier beacons. Re-exported so the
    // ppm_client emitter + dashboard PpmScoresPanel commands can construct
    // payloads without reaching into `orchestrator::ssb::*` directly.
    PpmScoredPayload, PpmGatingFailedPayload,
    // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon. Re-exported
    // so the agent_pool producer + contract crate + dashboard-v2
    // ModeRoutingPanel commands can construct the payload without
    // reaching into `orchestrator::ssb::*` directly.
    ClaimRoutedPayload,
    // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
    // Re-exported so the agent_pool dispatch producer + dashboard-v2
    // CheckerRegistryPanel commands can construct the payload directly.
    CheckerResultEmittedPayload,
    // E-32 (SCHEMA_VERSION 95): context-window extension beacons.
    // Re-exported so the contract crate + sidecar wiring can construct
    // payloads without reaching into `orchestrator::ssb::*` directly.
    FitmCalibrationAppliedPayload, DeepModeActivatedPayload,
    DeepModeEscalationChosenPayload, YarnRefusedPayload,
    // E-17 SP2 (SCHEMA_VERSION 97): RAG retrieval-quality beacons.
    // Re-exported so the contract crate + dashboard KnowledgeDrawer
    // commands can construct payloads without `::ssb::*` paths.
    DeepPoolCappedPayload, RrfQualityGateFiredPayload,
    HydeSkippedCodeQueryPayload, RrfComponentWeightsPayload,
    // E-1 (SCHEMA_VERSION 67-68): A-MEM / MemGPT / sleep-time compute
    // payloads. Re-exported so the cli LiveCli + dashboard-v2 MemoryNotesPanel
    // commands + contract tests can construct rows without `::ssb::*` paths.
    // SleepComputeStarted + SleepComputeQueryCached were rebaselined to
    // MemoryEvolutionRun(phase=start | cached_query) per
    // workflow/reasoning/E-1-decisions.md DECISION-1 to honor the
    // SCHEMA_VERSION=103 lock at executor brief time.
    MemoryEvolutionRunPayload, MemoryNoteInsertedPayload,
    MemoryNoteEvolvedPayload, MemoryConflictDetectedPayload,
    CoreBlockReplacedPayload,
};
// D-2: PPM client + sidecar hook public surface.
pub use ppm_client::{
    apply_ppm_gate, emit_ppm_score, PpmClient, PpmClientError, PpmScore,
    PpmSsbEmitter, VecPpmSsbEmitter,
};
pub use sidecar_ppm_hook::{
    MockSidecarPpmHook, PpmHookError, PpmMode, PpmScoreRequest, PpmScoreResponse,
    SidecarPpmHook,
};
// D-2: production HTTP impl of SidecarPpmHook.
pub use http_sidecar_ppm_hook::{
    build_http_ppm_client, build_http_ppm_client_with_threshold, HealthReport,
    HttpSidecarPpmHook, PPM_BACKOFF_CEILING_MS, PPM_BACKOFF_INITIAL_MS,
    PPM_DEFAULT_BASE_URL, PPM_DEFAULT_MAX_RETRIES, PPM_DEFAULT_TIMEOUT_SECS,
    PPM_HEALTH_PATH, PPM_SCORE_PATH,
};
// C-27 — Testing-Trust T(-1) DeterminismConfig public surface.
pub use determinism::{
    ActionTypeSequence, DeterminismConfig, DeterminismError, DriftComparator, DriftSink,
    GpuSku, HardwareClass, ELAI_HARDWARE_CLASS_ENV, ELAI_SEED_ENV, ELAI_SYSTEM_FINGERPRINT_ENV,
};
// C-27 — Testing-Trust T-0 WiringContract public surface.
pub use wiring_contract::{
    collect_contract_markdown_files, parse_wiring_contract_from_markdown, AuditorSink,
    DkgAuditOutcome, DkgReachabilityAuditor, ObservableEffect, ObservableEffectRunner,
    ReachabilityGraph, StubBanAuditor, StubBanEntry, WiringAuditCmd, WiringAuditError,
    WiringAuditReport, WiringAuditScope, WiringContract, WiringContractError,
    WiringEntrypointSet, WiringEntrypointSetLoader,
};
// D-9 — L3 frozen-surface access-control public surface.
pub use locked_keys_auditor::{
    status_snapshot as locked_keys_status_snapshot, AuditResult as LockedKeysAuditResult,
    LockedKey, LockedKeysAuditError, LockedKeysAuditor, LockedKeysRegistry, LockedKeysSink,
    LockedKeysStatusEntry, LockedKeysStatusSnapshot, Violation as LockedKeysViolation,
    LOCKED_KEY_FULL_FILE,
};
// D-10 — mode-routing predicate public surface. The `select_grounding_mode`
// symbol is L3-locked via the locked_keys_registry; the agent cannot edit it.
pub use claim::{Claim, ClaimType, EvidenceKind, EvidenceRef};
pub use grounding_mode::{select_grounding_mode, GroundingMode};
pub use orch_context::{OrchContext, OrchPhase, OrchUserRole};
// D-11 — Constructive-checker registry public surface. The
// Mode A constructive grading path dispatches claims through
// `CheckerRegistry::run_all_applicable` and emits one SSB row per
// `(checker_id, kind, result)` triple. See
// docs/thefinalglance/D-plan/D-11_constructive_checker_registry.md.
pub use checker_registry::{
    Checker, CheckerId, CheckerKind, CheckerRegistry, CheckerResult, CompileRustChecker,
    JsonSchemaDeferredChecker, LoadError as CheckerLoadError, NumericRangeChecker,
    RegexMatchChecker, ScriptChecker, SympyChecker, TomlCheckerEntry, TomlCheckerManifest,
    Z3DeferredChecker,
};
// D-12 — OperationalizationCheck baby-step predicate public surface.
// `is_operationalized` is the predicate the auditor gates on;
// `dispatch_check_for_baby_step` routes the procedure through the
// D-11 registry; `parse_baby_step` reads the babystep-author markdown
// surface. See docs/thefinalglance/D-plan/D-12_operationalization_check_predicate.md.
pub use baby_step::{
    dispatch_check_for_baby_step, is_operationalized, parse_baby_step, BabyStep, BabyStepError,
    DispatchOutcome as BabyStepDispatchOutcome, ExpectedKind, ExpectedResult, MeasurableProcedure,
};
// C-28 — Testing-Trust T-1 mutation testing public surface.
pub use mutation_testing::{
    EquivalentMutantCache, MutantOutcome, MutantRecord, MutationConfig, MutationError,
    MutationResult, MutationRunner, PythonMutationRunner, RustMutationRunner, CACHE_TTL_SECS,
};
// C-29 — Testing-Trust T-2 (IntentPair) + T-3 (Metamorphic) public surface.
pub use intent_pair::{
    evaluate_autonomous_intent_confirm, ConfirmedBy, IntentConfig, IntentConfirmation,
    IntentError, IntentPair, IntentPairSet, ASSERTION_MAX_CHARS, INTENT_MAX_CHARS, REF_MAX_CHARS,
};
pub use metamorphic_testing::{
    parse_relation, MetamorphicConfig, MetamorphicError, MetamorphicRelation,
    MetamorphicTestRunner, MrResult, RelationLibrary,
};
// C-36 — MuCoCo mutation operator catalog. Operators run inside the Python
// sidecar (python/elai_inference/mucoco/); the Rust surface exposes the
// 11-variant enum, the dispatcher, and a registry helper. SSB payload type
// lives at ssb::MucocoMutationAppliedPayload.
pub use metamorphic_testing::{
    dispatch_mucoco_mutation, list_mucoco_operators, MucocoError, MucocoMutationResult,
    MucocoOperator,
};
// B-37C — multi-runtime orchestration public surfaces.
pub use per_role_runtime::{
    auditor_executor_lineage_collision, LineageGroup, PerRoleRuntime,
};
pub use runtime_config::{
    default_fallback_chain, PpmRuntimeConfig, RuntimeConfig as RuntimeBackendConfig,
    RuntimeConfigPolicyView, ELAI_PPM_ENABLE_ENV, ELAI_PPM_ENDPOINT_ENV,
};
pub use runtime_orchestrator::{
    CompleteOutcome, RuntimeOrchestrator, RuntimeOrchestratorError, RuntimeOrchestratorEvent,
};
pub use runtime_event_sink_ssb::{install_runtime_event_sink_ssb, SsbRuntimeEventSink};
// C-1: orchestrator adapter for the knowledge-side typed SSB sink.
pub use knowledge_ssb_adapter::OrchestratorKnowledgeSsbSink;
// C-35: orchestrator adapter for the plugins-side ToolGateSsbEmitter trait.
#[cfg(feature = "mcp")]
pub use tool_gate_ssb_adapter::OrchestratorToolGateSsbEmitter;
// C-47: orchestrator adapter for the plugins-side CaseBaseSsbEmitter trait.
#[cfg(feature = "mcp")]
pub use tool_calibration_ssb_adapter::OrchestratorCaseBaseSsbEmitter;

// Batch 4C — public surfaces for the new modules.
pub use profile_selector::{
    emit_profile_selected, profile_name, DefaultGenerationProfileSelector, DispatchRole,
    GenerationProfileSelector,
};
pub use telescope_caller::{invoke_telescope, TelescopeResult, TelescopeStats};
pub use micro_reflection_emitter::{
    emit_micro_reflection, parse_reflection_json, DynReflectionAdapter, DynReflectionClient,
    ReflectionClient, ReflectionClientError,
};
pub use sidecar_reflection_adapter::{
    micro_reflection_disabled_via_env, SidecarReflectionAdapter,
    ELAI_DISABLE_MICRO_REFLECTION_ENV,
};
pub use irreversible_classifier::{
    irreversible_classifier_disabled_via_env, IrreversibleActionClassifier, IrreversibleVerdict,
    SharedIrreversibleClassifier, CONFIRM_IRREVERSIBLE_ACTION_KIND,
    ELAI_DISABLE_IRREVERSIBLE_CLASSIFIER_ENV,
};
// Batch 3C (Phase 4H): Self-Modification Layer 1A surface.
pub use self_mod_layer1a::{
    acquire_rules_lock, build_pattern_regex, classify_table, estimate_fpr, is_allowed,
    is_security_sensitive, layer1a_auto_apply_via_env, layer1a_disabled_via_env,
    status_label as msi_status_label, trigger_dep_audit_background,
    trigger_steering_vector_refresh, AllowedConfigKey, AppliedProposalRecord, DataTable,
    DataTableProposal, DefaultLayer1aProcessor, FailureRecord, FindingRecord, Layer1aError,
    Layer1aProcessor, MsiStatus, PipelineMode, PipelineOutcome, ProposalOutcome, ProposalSource,
    RulesDbLockGuard, DEFAULT_ARCHIVE_CAP, DEFAULT_MAX_FPR, DEFAULT_MIN_OCCURRENCES,
    DEFAULT_MSI_LAMBDA, DEFAULT_MSI_WINDOW, ELAI_DISABLE_LAYER1A_ENV, ELAI_LAYER1A_AUTO_APPLY_ENV,
    LAYER1A_AGENT_ID, MSI_DIVERGING_THRESHOLD, MSI_IMPROVING_THRESHOLD, RULES_DB_LOCKFILE,
};
// B-23 Phase 4H Group B: DGM ConfigVariant archive (4H.3) + log-driven
// SelfModProposal builder + Class A/B routing (4H.4).
pub use config_archive::{
    exploration_bonus, parent_selection_probabilities, parent_selection_weights,
    sample_variant, sigmoid_perf_score, ConfigArchive, ConfigArchiveConfig,
    ConfigArchiveError, DEFAULT_ARCHIVE_CAP_PER_MODE, DGM_ALPHA_0, DGM_LAMBDA,
};
pub use self_mod_proposal::{
    route_proposal, ProposalClass, ProposalEngine, ProposalEngineError,
};
// B-23 Phase 4H Group C — bridge from FullPipelineOutcome to Layer 1A
// proposals + REG pre-screen stub (real REG lands in Group E).
pub use self_mod_bridge::{proposals_from_full_outcome, regression_pre_screen};
// B-23 Phase 4H Group D — staged evaluation (4H.5) + 4-layer safety (4H.6).
pub use staged_eval::{
    evaluate_stage, run_full_pipeline, BenchmarkRunner, StageOutcome, StageReport, StageVerdict,
    StagedEvalError,
};
pub use self_mod_safety::{
    apply_structural_change, check_lock_list_read, check_locked_key, is_known_parameter,
    is_locked, should_auto_revert, ParameterKey, SafetyError, DEFAULT_AUTO_REVERT_FRACTION,
    DEFAULT_AUTO_REVERT_WINDOW, KNOWN_PARAMETER_KEYS, LOCKED_KEYS,
};
// B-23 Phase 4H Group E — MSI per-mode tracker (4H.7), OSC ouroboros port
// (4H.7B), REG cross-pipeline regression detector (4H.7C), EVO evolution
// gate (4H.7D).
pub use msi::{
    msi_verdict, MsiSnapshot, MsiTracker, MsiVerdict, MSI_DIVERGING_THRESHOLD as MSI_DIVERGING,
    MSI_IMPROVING_THRESHOLD as MSI_IMPROVING, MSI_LAMBDA as MSI_LAMBDA_SPEC, MSI_WINDOW,
};
pub use osc::{compute_osc_verdict, compute_similarity};
pub use regression::detect as detect_regressions;
// E-30: named failure-taxonomy entries for the 5 plan-named symptom categories.
pub use regression::{
    e30_taxonomy_entry_for_category, e30_taxonomy_entry_for_ssb_action, E30TaxonomyEntry,
    E30_FAILURE_TAXONOMY,
};
pub use evolution_gate::compute_evolution_gate;
// B-23 Phase 4H Group F — top-level self-modification orchestration (4H.8).
pub use self_mod_orchestrator::{
    SelfModOrchestrator, SelfModOrchestratorError, SelfModUpdate,
};

/// B-23 Phase 4H Group F — msgpack helper for callers that don't want to
/// depend on `rmp_serde` directly. Used by dashboard-v2 Tauri commands
/// to encode SSB payloads.
pub fn encode_ssb_payload(payload: &ssb::SsbPayload) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec_named(payload)
}

/// B-24 Phase 4I Group C8 audit WARN 2 — inner-payload msgpack helper.
/// B-24 Phase 4I Group C8 — generic msgpack decode helper for callers
/// (currently dashboard-v2) that consume SSB payload bytes from the
/// broadcast subscriber but do not want a direct `rmp_serde` dependency.
/// Returns the deserialization error string so the caller can log/propagate
/// without coupling to the underlying msgpack lib version.
///
/// Decision 92 (B-24 Phase 4I Group C8 audit-fix): the dashboard
/// subscriber must instantiate `T = SsbPayload` so the externally-tagged
/// enum decode matches the production wire format produced by
/// `Ssb::write_payload` (`rmp_serde::to_vec_named(&payload: SsbPayload)`,
/// see ssb.rs L2988). Decoding into the inner payload struct directly
/// fails on real production bytes — the previous helper
/// `encode_inner_payload` was removed because it documented a
/// test-only encoding that did not exist in production.
pub fn decode_ssb_payload_bytes<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
) -> Result<T, String> {
    rmp_serde::from_slice::<T>(bytes).map_err(|e| e.to_string())
}
// B-23 Phase 4H Group A: shared self-modification types used across the
// new modules (config_archive, msi, osc, regression, evolution_gate) and
// the dashboard wiring layer.
//
// `PipelineId` and `RuleId` are intentionally NOT re-exported here — they
// collide with existing pipeline_builder + rules-crate types of the same
// name. Consumers that need the self-mod alias use the fully-qualified
// `crate::self_mod_types::PipelineId` path.
pub use self_mod_types::{
    AcRegression, AgentId, BabyStepId, CatchLayer, CatchRate, ConfigVariant,
    ConfigVariantSimilarity, DataTableValidation, DataTableVerdict, EvolutionGateResult,
    FullPipelineOutcome, GrcStepType, Language, OscVerdict, PhaseId,
    PipelineOutcomeId, ProjectScope, ProposedChange, ReasoningTraceRef, RegressionReport,
    ScoreWeights, SelfModProposal, Slot, SsbLogEntry, StagedEvaluation, StaleApiCatch,
    StructuralChange, StructuralChangeType, TemplateId, TestId, TestOutcomeReport, TriggerCount,
    UnclassifiedError, VariantId, VariantStatus, ProjectId,
    DEFAULT_SCORE_WEIGHTS, SIM_STAGNATION_THRESHOLD, SIM_STAGNATION_WINDOW,
    SIM_WEIGHT_EXACT_MATCH, SIM_WEIGHT_KEY_OVERLAP, SIM_WEIGHT_TYPE_MATCH,
    STAGE1_DEFAULT_TASKS, STAGE2_DEFAULT_TASKS, STAGE3_DEFAULT_TASKS,
    STAGE3_PROMOTION_THRESHOLD, STAGE3_RANKING_SUBSET,
};
pub use system_probe::{
    classify_vram_tier, system_probe_disabled_via_env, DefaultSystemProber, DetectionMethod,
    GpuInfo, SystemProbeResult, SystemProber, ELAI_DISABLE_SYSTEM_PROBE_ENV,
    ELAI_FREE_DISK_GB_ENV, ELAI_FREE_RAM_GB_ENV, ELAI_VRAM_FREE_GB_ENV, ELAI_VRAM_GB_ENV,
};
pub use feasibility_probe::{
    alternative_within_cap, check_constraints, decide_with_mode, enforce_stub_line_limit,
    feasibility_probe_disabled_via_env, fsm_transition_idempotent, read_stub_results,
    BabyStepEscalation, BabyStepFeasibilityReport, BabyStepStubResult, ConstraintResult,
    DefaultFeasibilityProber, FeasibilityDecision, FeasibilityEscalation,
    FeasibilityExecutionMode, FeasibilityFsmBridge, FeasibilityProber, FeasibilityReport,
    NovelStep, ResourceDelta, ResourceMargins, StubExecutor, StubResult, StubResultsReadError,
    SyntheticStubExecutor, SystemConstraintsRequired, TaskFeasibility, Tier3StubExecutor,
    ELAI_DISABLE_FEASIBILITY_PROBE_ENV, FEASIBILITY_PROBE_TIMEOUT, MAX_ALTERNATIVES_PER_STEP,
    RESOURCE_MARGIN_THRESHOLD_PCT, STUB_LINE_LIMIT, STUB_TIMEOUT_GRACEFUL, STUB_TIMEOUT_HARD,
};
pub use ssb_viewer::{
    ChatEntry, CloudSpendSummary, FeasibilityReportRow, FeasibilitySummary, FileChange,
    FsmStateSnapshot, InterventionRequest, NovelStepRow, PhaseStatus, SessionInfo,
    RuleViolationSummary, SsbActionDetail, SsbActionSummary, SsbViewer, SsbViewerError,
    StepStatus, StubRunRow, TaskSummary, ThinkStepSummary, validate_request_id,
    // Tier 2 Track B (dashboard-wiring §6) — PlanWindow / PreviewWindow DTOs.
    // `PhaseDetail` is renamed on re-export to `DashboardPhaseDetail` because
    // `plan_skeleton::PhaseDetail` (re-exported above) already occupies the
    // bare name in this module. Dashboard callers import the alias.
    BabyStepDetail, BabyStepRef, ConceptNodeRef, DecompositionTreeSnapshot,
    PhaseDetail as DashboardPhaseDetail, PhaseNodeRef, PreviewAvailableRow, StepNodeRef,
    // Tier 3 partial (session s-84064) — drawer DTOs consumed by
    // elai-dashboard::commands. Zero new SsbActionType variants; all DTOs
    // are typed views over existing payload structs.
    AgentCheckpointEntry, Bm25StatsSnapshot, ContextBudgetSnapshot,
    DairTraceEntry, DkgStatsSnapshot, FailureTaxonomyEntry, FreshnessState,
    GgSignalEntry, GrcBudgetEntry, HookVerdictEntry, KnowledgeReadinessSnapshot,
    MistakeBookEntry, PageRankEntry, QueryTierStats, SessionHandoffEntry,
    ShortCoderEffectEntry, TokenReclamationStats, ValidationTierEntry,
    VectorStatsSnapshot, VramTierSnapshot,
    // Tier 3 complete (session s-456340) — Research drawer DTOs. Typed
    // views over existing payload structs; zero new SsbActionType variants.
    ResearchBudgetSnapshot, ResearchStepEntry, SourceTierDistribution,
    // B-22 Phase 4G — auto-advance counters DTO consumed by dashboard-v2.
    AutoAdvanceStats,
    // B-39 chat reflect pipeline — reflect state snapshot DTO.
    ReflectState,
};
pub use task_runner::{
    detect_scaffold_template, emit_integration_checkpoint_for_test,
    enforce_file_read_budget_guard, evaluate_post_execution_block_found,
    fix_and_rerun_until_green, is_web_research_task, run_confirmation_gate,
    is_web_scaffold_task, new_dev_server_registry, project_dir_is_web,
    serialize_decomposition_for_grounding, start_task_runner,
    AgentPoolBabyStepExecutor, BabyStepExecutionCtx, BabyStepExecutionResult, BabyStepExecutor,
    CancelHandle, CwdProjectDirResolver, DefaultWebResearchRunner, DevServerLauncher,
    DevServerRegistry, FileReadBudgetVerdict, FixAndRerunOutcome, FixerValidator,
    FixerValidatorReport, LaunchedPreview, ProjectDirResolver,
    RuleEvaluator, ScaffoldApplier, SsbWebResearchStepEmitter, SyntheticBabyStepExecutor,
    TaskRunnerConfig, TaskRunnerError, WebResearchRunner, WebResearchSummary,
    WebValidatorLauncher, AGENT_FAILED_CATEGORY, BABY_STEP_PRE_GATE_ACTION,
    DEFAULT_BABY_STEP_TIMEOUT_MS, DEFAULT_DEV_SERVER_COMMAND, DEFAULT_DEV_SERVER_PORT,
    DEFAULT_ROOT_QUESTION_MAX_CHARS, DEFAULT_SCAFFOLD_TEMPLATE, DEFAULT_SOURCE_TIERS_TOML,
    DETERMINISTIC_AUDIT_CATEGORY, DEV_SERVER_FAILED_CATEGORY,
    ELAI_DISABLE_SHORTCODER_POSTOOLUSE_ENV, INDEXING_SUMMARY_CATEGORY,
    INTEGRATION_CHECKPOINT_FAILED_CATEGORY, INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY,
    MAX_BABY_STEPS_PER_TASK, POST_EXECUTION_RULES_AUDIT_STEP, PREVIEW_SLOT_LIVE,
    RULE_VIOLATION_CATEGORY, RULE_VIOLATION_POST_CATEGORY, RUNNER_AGENT_ID,
    SPIRAL_HARDSTOP_CATEGORY, WEB_RESEARCH_FAILED_CATEGORY, WEB_SCAFFOLD_FAILED_CATEGORY,
    WEB_SCAFFOLD_TEMPLATE_ASTRO, WEB_SCAFFOLD_TEMPLATE_REACT, WEB_SCAFFOLD_TEMPLATE_SVELTE,
    WEB_SCAFFOLD_TEMPLATE_VANILLA,
    // B-29 Phase 5: IMPACT_SUMMARY formatter + baseline tools constant.
    format_impact_summary, DISPATCH_CONTEXT_BASELINE_TOOLS_PUBLIC,
    // C-16 (DECISION-D + DECISION-G): expose the trigger_cosmos_refresh
    // helper to the dashboard-v2 Tauri command. Previously orphan (no caller
    // in the production path); now reachable from
    // commands::trigger_cosmos_refresh.
    trigger_cosmos_refresh,
};
pub use rule_loader::load_seed_rules_from_dir;
// Phase A1b — chokepoint helpers exposed at the orchestrator crate root so
// the CLI run subcommand and the dashboard-v2 Tauri setup() callback both
// reach the same RuntimeWiring + apply_task_runner_wiring entry points.
// Contract: `crates/contracts/tests/task_runner_chokepoint_contract.rs`.
pub use runtime_setup::{
    apply_task_runner_wiring, build_default_baby_step_executor,
    build_production_hook_dispatcher, build_task_runner_wiring, locate_default_agent_binary,
    RuntimeWiring,
};
// Phase B2 — IPC accept-loop public surface.
pub use ipc_accept::{
    accept_one, bind_session_listener, run_agent_session, AgentFinalState, AgentRpcServer,
    AgentSessionOutcome, IpcAcceptError, IpcHookDispatcher, InferOutcome, NoOpAgentRpcServer,
    PlatformListener, PlatformStream, ToolOutcome,
};
// Phase B2c — ProductionAgentRpcServer public surface.
// NullToolDispatcher removed (B2c fix): CLI/dashboard build the real InProcessToolDispatcher
// directly via elai_tools::dispatcher (they have elai-tools as a regular dep).
pub use production_agent_rpc::ProductionAgentRpcServer;
pub use task_runner::runtime_model_pool;
pub use supervision::{
    EscalationAction, SupervisionError, SupervisionManager, SupervisionOutcome, SupervisionPolicy,
};
// B-24 Phase 4I (Group C1) — top-level media-validator surface. Only the
// three spec-mandated entry types are re-exported here; the deeper
// surface lives under `elai_orchestrator::media_validator::*`.
pub use media_validator::{MediaValidationRequest, MediaValidationResult, Verdict};

#[cfg(test)]
pub(crate) fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
