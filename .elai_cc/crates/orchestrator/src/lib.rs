pub mod agent_pool;
pub mod hardware_catalog;
pub mod agent_role;
pub mod agent_role_dispatcher;
// H-31 (SCHEMA_VERSION 240): the Personalization-Trap firewall -- the compile-time role allowlist
// gating user-memory injection. ALLOW = {AgentRole::Executor, ConsensusRole::Worker} ONLY; every
// reviewer/auditor/critic/adversary role gets ZERO user-memory. Consumed by the CLI prompt-
// assembly path (report_fns::build_system_prompt) and the user_memory_capture producer.
pub mod user_memory_firewall;
// I-13: Stage-2 write-time memory guard (aho-corasick invisible-Unicode +
// confusable-shell scan). Composes the Stage-1 classify() gate in
// elai_memory::defense. Gated behind ELAI_MEMORY_GUARD_ENABLE (default-OFF).
// Production callers: user_memory_capture.rs (CHOKEPOINT 1) + memory_cmd.rs (CHOKEPOINT 2).
pub mod memory_guard;
pub mod agent_template;
// D-13: adversary role + cross-family heterogeneous-verifier check.
// Producer side of the adversarial-verification pass; consumed by
// `agent_pool::run_adversarial_pass`. Grounding: arxiv:2403.07974 (T1).
pub mod adversary;
// P0-1 MR-1 fix: CloudAdversary extracted from adversary.rs; re-exported via adversary.
pub mod cloud_adversary;
// SP-3A MR-1 fix: ModelFamily extracted from adversary.rs; re-exported via adversary.
pub mod model_family;
// E-6: MAST 14-mode failure taxonomy (Cemri et al. 2025 arXiv:2503.13657).
// Citation correction from plan's bad cite 2406.12624 (ID_COLLISION)
// documented in workflow/reasoning/E-6-decisions.md DECISION-1 +
// gotchas.md E6_CITATION_2406_12624_ID_COLLISION.
pub mod mast;
// E-29 Tier-B: agentic-observability subtree.
//   * mast_classify -- post-run MAST classifier reusing
//     crate::mast::{MastMode, finding_to_mast_mode}.
//   * replay -- ssb_resume_from() validator that consumes the C-44/D-9
//     locked_keys_auditor predicate FAIL-CLOSED.
//   * signal_match -- candidate-row sweep primitive (Laminar Signals).
//   * dataset_promote -- traces-to-eval-dataset promotion primitive.
//   * pricing_rollup -- per-phase/agent/tool cost helper REUSING C-14
//     providers::pricing::ModelPricing.
// All four SSB fire-points hang off a single Pattern-2 variant
// `SsbActionType::AgenticObservabilityEvent` with a 4-arm kind
// discriminator gated by ELAI_AGENTIC_OBSERVABILITY_ENABLE. See
// workflow/reasoning/E-29-decisions.md (Tier-B section).
pub mod observability;
// E-6: PhaseKind enum used to gate cross-family-adversary requirement
// at the AUDITING -> MEMORIZING transition. 6 variants {Code,
// Architecture, Security, Docs, Test, Other}. See E-6-decisions.md
// DECISION-3 + GAP-1 in the integration-explorer report.
pub mod phase_kind;
// E-6: iso-budget ablation harness — compares full 8-role ELAI pipeline
// against single-agent + "Wait" reflection baseline at equal thinking-token
// budget. Host-side scaffolding ONLY; live multi-agent runs RUNPOD-PENDING.
// Grounding: arXiv:2604.02460 + arXiv:2507.02778 + arXiv:2502.00674.
pub mod iso_budget_harness;
// E-10: hostile-critic structural simplifications -- 7 new modules + 1
// StructuredFailureSignal delta in mistake_book.rs. Substrate-only,
// wire-dark unless ELAI_CRITIC_OF_DESIGN_ENABLE=1. See
// workflow/reasoning/E-10-decisions.md.
pub mod critic_stats;
pub mod fsm_ablation;
pub mod role_ablation;
pub mod rag_challenger;
pub mod cot_proof;
pub mod cache_check;
// I-10: prefix invariance + TTL tiering + cache-hit hard guard.
// Provides CacheEconomicsPlanner (TTL tier selector), PrefixInvarianceGuard
// (miss-rate monitor + blake3 prefix diff), and max_tokens_clamp (schema-bound
// helper). Emits CacheEconomicsEvent SSB rows. Composes on H-1/H-2 shipped
// substrate; never re-implements apply_anthropic_cache_markers or update_breakpoints.
// Default-ON kill-switch: ELAI_CACHE_ECONOMICS_DISABLE=1.
pub mod cache_economics;
// I-8: dead-end fingerprint cache (voluntary abort / spiral prevention).
pub mod dead_end_cache;
pub mod grc_blockquote;
// E-10 P5 wire-on (offline-buildable) -- per-VRAM tier matrix + cloud-default
// switch test producers. Reuse `CriticTilePhase::TierMatrix` /
// `CriticTilePhase::CloudDefault` (no new SsbActionType, no schema bump).
// Wire-dark unless ELAI_CRITIC_OF_DESIGN_ENABLE=1. Live bench inputs are
// RUNPOD-PENDING; emit paths are now structurally complete.
pub mod tier_matrix;
pub mod cloud_default;
// E-6: persona re-anchor — re-injects role prompt every N turns AND when
// cosine drift < threshold. EmbedProvider is pluggable; cosine gate
// degrades to turn-counter-only when no provider is injected. Grounding:
// arXiv:2511.09710 + arXiv:2412.00804 + arXiv:2601.04170.
pub mod persona_anchor;
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
// E-4 chat UX critique substrate: SycophancyGuard + ClaimWithConfidence
// + ConfidenceTier + OneShotClarifier + CompletionClaimCheck +
// DoomLoopDetector (thin facade over anti_spiral). All emit via the
// SsbActionType::ChatUxCritiqueEvent Pattern-2 envelope when the
// ELAI_CHAT_UX_CRITIQUE_ENABLE env var is set to "1". See
// workflow/reasoning/E-4-decisions.md.
pub mod chat_ux;
// E-11 catalog+recommender slice: 5-archetype SaaS taxonomy + per-
// archetype catalog (auth scheme + multi-tenancy + framework profile
// stem + production-checklist subset + scope-clarity profile) +
// ProjectClass-aware recommender (RecommendationConfidence 3-arm enum,
// no f32) + 2024-2026 prior-art registry (15 entries: 9 tools + 6
// reports). Wire-dark SSB emission gated by
// `ELAI_SAAS_PRIOR_ART_ENABLE=1`. See workflow/reasoning/
// E-11-decisions.md.
pub mod architecture;
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
// E-52 Tranche B-3 (s-22300): LOCAL-ONLY diagnostic probe producer. Wires the
// already-shipped `DiagnosticProbe*` SSB substrate (variants + payloads +
// canonical encoders at `ssb.rs:6400+/19553+/26690+/30303+`) and the
// `elai_knowledge::diagnostic` envelope into a runner-side dispatcher. Wire-
// dark via `ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE=1`. multilspy / ts-morph / lsp-
// server / lsp-types DEFERRED — see workflow/sessions/s-22300/stubs.md.
pub mod diagnostic_probe;
// E-44: durability layer additions (idempotent dispatcher + saga/lock/
// migrator/continue-as-new wrappers on top of the existing SessionDag +
// AgentCheckpoint surface). All additions are additive — no existing
// public APIs are modified. See `durability/mod.rs`.
pub mod durability;
pub mod dst;
pub mod cloud_mode;
// E-39 -- cost + token economics substrate. HARD ceiling mid-task
// cooperative abort + velocity caps + AutoMix self-verify gate +
// difficulty-x-budget cost router + free-tier quota guard +
// layered caps. Pattern-2 (28th application) single CostTokenEvent
// + 8-arm CostEventKind discriminator. Wire-dark unless
// ELAI_COST_TOKEN_ECONOMICS_ENABLE=1. See
// workflow/reasoning/E-39-decisions.md.
pub mod cost;
pub mod context_budget;
// H-50: regime-aware in-session context management.
// Single effective_context_window truth + ContextPolicy-by-BackendMode.
// LOCAL: trim at 75% of real window via compaction ladder; CLOUD: 85% cache-
// stable prefix policy (unchanged from pre-H-50). Kill-switch:
// ELAI_CONTEXT_POLICY_DISABLE=1. No new SSB variant (reuses ContextReduction
// per GD-1). Wire-in points: conversation.rs:789 + live_cli.rs:1185.
pub mod context_boundary;
pub mod completion_gate;
pub use completion_gate::{maybe_downgrade_passed, non_terminal_task_ids_for_agent};
pub mod context_policy;
// I-14: deterministic context compression middleware (CACHE-INVARIANT ECONOMICS).
// Tier-1 AstFold (tree-sitter body elision) + LspSlice (compiler-error slicing),
// each journaled as a reversible ContextTransformReceipt SSB row (ID-3). Wired at
// the tool-result byte seam via AgentRpcServer::transform_tool_output (ipc_accept.rs).
pub mod context_transform;
// I-5: pristine context injector -- per-branch context rebuild excluding failed
// sibling tracebacks + compressed contrastive constraint (<=30 tokens). Emits
// ContextPruned SSB rows (SCHEMA_VERSION 247). Wire-dark until I-4 SearchLadder
// ships (Wave 4). Composes H-6/H-50/E-19. Production caller: I-4
// search/ladder.rs::SearchLadder::run_best_of_n (WIRE-DARK[I-5]).
pub mod pristine_path_filter;
// H-51: cross-session sleep-time handoff digest author + open-topic detector.
// Emits HandoffLifecycleEvent SSB rows (SCHEMA_VERSION 227). Wire-in points:
//   - LiveCli::drop (DigestWritten)
//   - repl.rs run_repl startup (ResumeOffered + one-tap ResumeConfirmed)
//   - handoff.rs continue_from_handoff (TieredLoad)
// Kill-switch: ELAI_HANDOFF_AUTORESUME_DISABLE=1 (detect only; digest
// write always runs). Registered in wire-dark-allowlist.toml (R9).
pub mod handoff_digest;
pub use handoff_digest::TopicHandle;
// G-7: per-change confidence PRODUCER (fuses log-prob + self-consistency +
// oracle-free test execution-agreement into the real claim_score feeding the
// SHIPPED D-16 conformal abstention layer).
pub mod claim_confidence;
// G-7: lethal-trifecta detection + dual-LLM quarantine (route an untrusted span
// through a structurally no-tool sub-LLM when secret + untrusted + egress co-occur).
pub mod quarantine;
pub mod dispatch;
pub mod dispatcher;
pub mod privacy;
pub mod privacy_redactor;
pub mod redaction;
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
pub mod gate_intensity;
// H-38 — cost-confirmation gate predicate for TaskStop / pod_stop.
// Reads the H-9 CloudSpend ledger at dispatch_stop time; pure + sync predicate.
pub mod stop_cost_gate;
// I-21 — sprint contracts + ralph loop. Structured Sprint Contract + isolated
// evaluator + bounded trace reinjection + D-13 adversary escalation. Composes
// H-37/i-1/i-17/D-13 (GD-1). Wire-dark unless ELAI_SPRINT_CONTRACT_ENABLE=1.
pub mod sprint_contract;
// C-40 — GitHub PR adapter (gh CLI shell-out). Producer of the four
// SCHEMA_VERSION 109 SsbActionType::GithubPr* rows. Cross-language
// pair: dashboard-v2/src-tauri/src/commands_c40.rs +
// dashboard-v2/src/lib/windows/GithubWindow.svelte.
pub mod github_pr;
// B-40 — Idea-Framer + Domain Research Presets + Critic Loop. Inserts a
// FRAMING substate before Phase 4D Recursive Decomposition. Public surface
// re-exported below (FramedIdea, ProjectClass, CriticCategory, etc.).
pub mod idea_framing;
// I-41: spec-time critique lenses (L1-L8). Advisory-only; zero path to any
// accept/block verdict. Lens findings are surfaced to humans / plan-improvement
// and emitted as CritiqueLensFinding SSB rows (advisory=true). Never imported
// by directive.rs or any gate-plane module. Module-level isolation is
// compiler-enforced: critique/ does NOT import CriticFinding or Severity.
pub mod critique;
pub mod hooks;
pub mod hook_dispatcher;
// I-1: directive supervisor enforcement plane. The generator emits pure
// Directive structs; the Supervisor is the sole side-effect executor, gate-
// checking (F-1/E-30/G-3) before execution and firing E-44 saga compensation
// on crash. Tier-1 in-process; OS-process separation is Tier-2 (gated).
pub mod directive;
// I-32: schema-first role contracts for factual/code agent roles. Replaces narrative
// persona blocks in Executor/Researcher/ContextCompiler/Designer/MemoryWriter prompts
// with typed I/O constraint lists. Alignment-dependent roles (Auditor, IdeaCritic,
// InjectionClassifier) are unchanged. Evidence-gated per ID-5 via A/B harness.
pub mod role_contracts;
// I-35: temporal tool-sequence enforcement — ordered history rules at the I-1 supervisor
// dispatch plane. TOML-declared rules (R8-frozen) evaluated over an ordered HistoryEntry
// buffer; CDA-immune (predicates evaluate tool-call history, not token content).
// Adds ToolSequenceVerdict SSB action_type (Pattern-2 3-arm: Allowed/OrderViolation/HaltEnforced).
pub mod tool_sequence;
// I-19: TDD enforcer preflight gate. Pure-sync AST predicate (tree-sitter) that checks
// every Edit/Write invocation for a matching test-symbol delta before allowing the write.
// Wire-dark default (ELAI_TDD_PREFLIGHT_ENABLE=1 required to activate). Default-ON flip
// gated on GD-4 FP rate bounded on >=3 real orchestrate runs (promotion plan: I-19-promotion).
// Emits TddPreflightBlocked SSB rows (Pattern-2 3-arm: MissingTestDelta/SyntaxDryRunFailed/
// RefactorExempted). Wired into hook_dispatcher::InProcessHookDispatcher::dispatch_pre_tool_use.
// See docs/thefinalglance/I-plan/I-19_tdd_enforcer_preflight.md.
pub mod tdd_preflight;
// I-50 (SCHEMA_VERSION 268): auth broker SSB-emit wrapper.
// Thin orchestrator-layer shim that maps elai-providers::auth_broker::ResolveOutcome →
// AuthBrokerEventKind, emits the SSB row, and exposes the production entry point
// broker_resolve_anthropic_auth_with_ssb (PRODUCTION_HOOK: runtime_cmd.rs:387).
// WIRE-DARK[I-50] at the writer=None call site until Arc<Ssb> is threaded.
pub mod auth_broker_emit;
// I-54: structural (AST-aware) search/preview/apply engine over the shipped
// tree-sitter grammars (reuses elai_knowledge::TreeSitterAstEvaluator; one
// version source, SLC-3). Wired into production_agent_rpc::translate_tool_request
// (structural_search/_preview/_apply tool arms) + build_tool_catalog.
// See docs/thefinalglance/I-plan/I-54_structural_ast_edit_tool.md.
pub mod structural_edit;
// I-33: mechanical decision governance primitives -- CEFL (Candidate
// Externalization for Fair Labelling), I6Q rationale-quality validator,
// commit-reveal integrity, and the governed_prompt_decision production call
// site. Emits GovernancePrimitiveFired SSB rows (3-arm Kind).
pub mod decision_governance;
// H-36: completion / needs-input dev-event notification dispatcher.
// Fires terminal escape + optional OS notification + writes agent_notification_emitted SSB row.
// Mount points: task_runner.rs post-PhaseComplete (Completion) and
// post-emit_autonomous_decision for RequestHuman (NeedsInput).
pub mod notify_dispatch;
// F-1 VibeGate: compound security evaluator + secret regex patterns.
// vibegate_secrets is a private impl module (regex set); vibegate is the
// public surface wired into hook_dispatcher::InProcessHookDispatcher.
mod vibegate_secrets;
pub mod vibegate;
// F-10 Slice A2: deterministic CWE-class classifier. Reuses the F-1
// vibegate scanner (Layer 1) + compiled-once injection-class regexes the
// F-1 seed corpus does not gate (Layer 2: CWE-78/95/502/22). Input layer
// of the security-monotonicity gate.
pub mod cwe_classifier;
pub use cwe_classifier::{classify_delta, delta_fingerprint, CweClass};
// F-10 Slice A3: security-monotonicity iteration guard. Compares the CWE-class
// set of one self-improvement iteration against the prior iteration's set and
// emits a SecurityClassRegressionBlocked beacon when a new class is introduced.
// Wired at all three production legs (REFINE, AUTO-ADVANCE, SELF-MOD).
pub mod security_monotonicity;
pub use security_monotonicity::{
    emit_security_class_regression_blocked, run_refine_leg, MonotonicityVerdict,
    SecurityMonotonicityGate,
};
// F-9 / Slice D1: VibeSecDep write-time slopsquat gate.  Detects dependency
// imports at PreToolUse time and probes the registry; emits
// WriteTimeSlopsquatBlocked SSB rows on NotFound / TooNew verdicts.
pub mod vibesec;
pub use vibesec::dep_writetime_gate::{
    DepWriteTimeGate, LruExistenceCache, RegistryExistenceChecker,
    StubRegistryChecker, SubprocessRegistryChecker,
};
pub mod shortcoder_hook;
pub mod provider_signals;
pub mod indexing_service;
// C-29: Testing-Trust Tier T-2 (IntentPair) — per-BabyStep intent
// formalization. See workflow/reasoning/C-29-decisions.md.
pub mod intent_pair;
// B-14 STEP 3: Dashboard chat intent classification. 7 spec categories
// (TASK / FIX / EXPLORE / CONFIG / INTERVENE / STEER / ABORT) with
// deterministic slash-command + keyword tier and an Ambiguous fallback
// for LLM disambiguation (spec FLOW 4). Production caller is the Tauri
// `classify_intent` command in dashboard-v2 commands.rs. See
// docs/thefinalglance/B-plan/B-14_live_token_streaming_chat.md.
pub mod intent_classifier;
// I-37: EARS requirement objects — typed, hash-keyed, mutation-covered EARS
// requirement records with a deterministic verified status + RequirementsGate +
// RequirementVerified SSB emit. See docs/thefinalglance/I-plan/I-37_ears_requirement_objects.md.
pub mod requirements;
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
// F-3p1: capability commitment + untrusted-taint bridge (cap-flow core).
pub mod capability_flow;
// E-37 U2: LLM-injection classifier with GPU-PAUSED-BY-OWNER fail-open.
pub mod llm_injection_classifier;
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
// G-2 (SCHEMA_VERSION 210): Reproducible-Run Artifact (RRA) — the G-plan
// trust-spine keystone. Bundles the SSB action stream + determinism pins +
// F-gate evidence manifest + blake3 root into a portable .rra; verify_rra
// re-checks it offline. CONSUMES F-1/F-4/F-9/F-10 verdict hashes (GD-1).
pub mod rra;
pub mod rra_emit;
pub mod rra_export;
pub mod rra_tiers;
// G-8 (SCHEMA_VERSION 213): signed in-toto v1 provenance attestation. Re-expresses
// the G-2 RunArtifact manifest as a custom in-toto Statement + predicate, signs it
// with a LOCAL blake3-Merkle proof (no new dep), and verifies it OFFLINE
// (trust-without-re-running, SLSA-VSA). CONSUMES the G-2 manifest (GD-1); extends
// F-4 off-machine. Article-50 honest posture (E-38, DD6).
pub mod attestation;
// I-34 (SCHEMA_VERSION 234): CRANE hybrid grammar-pin gate caller. Orchestrator-layer
// entry point for the three-layer grammar hardening: (1) crane_adaptive_eligible gate
// eligibility check, (2) blake3 pin manifest verification via providers::grammar_store,
// (3) post-parse schema re-validation via providers::re_validate_output. Emits
// GrammarGateEvent SSB rows; routes PostParseRejected to D-16 conformal-refusal seam.
pub mod grammar_gate;
// G-5 (SCHEMA_VERSION 212): reviewer-cockpit Diff-Surface-to-Review (DSR)
// scoring engine. Pure function over parsed git-diff hunks + the per-file
// deterministic signals decoded from the SSB stream. CONSUMES F-4/F-1/F-2a
// verdicts (re-implements none, GD-1); the DSR recomputes byte-identically from
// the pinned git-diff + the seq-ASC stream (GD-2). See
// workflow/reasoning/G-5-decisions.md.
pub mod review_surface;
// H-52 (SCHEMA_VERSION 228): deterministic snippet->hunk finding-anchoring.
// The resolver closes G-5 Open-Question #2 (the hunk_id join risk). Default-ON
// kill-switch: ELAI_REVIEW_ANCHOR_DISABLE (not a default-OFF enable gate; R9).
pub mod review_anchor;
pub mod grounding_client;
pub mod research_budget;
pub mod research_pipeline;
// E-48 (SCHEMA_VERSION 185): web-research orchestrator hardening
// substrate. Sits alongside the future E-22 + E-23 research add-ons
// the session-prompt anticipates. Wire-dark default; emission gated by
// ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE. See
// workflow/reasoning/E-48-decisions.md.
pub mod research;
// E-38 (SCHEMA_VERSION 175): 2026 AI roadmap signals + EU AI Act
// compliance pre-gate substrate. See workflow/reasoning/E-38-decisions.md.
pub mod compliance;
pub mod roadmap_signals;
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
// F-7 / Slice A2: pure Laplace add-one utility scoring for CBR template
// ranking (HALF B only). No SSB cascade here (A1 owns the beacon); pure
// detector over compiled_pipeline first_try_success_rate + use_count.
// See `workflow/reasoning/F-7-decisions.md`.
pub mod template_utility;
// F-6 / Slice A1: learned-skill trust ladder (S1..S4) + skill store +
// staleness demotion. LOCAL trust enum, no bridge to elai_memory::TrustTier.
// See `workflow/reasoning/F-6-decisions.md`.
pub mod skill_trust;
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
pub mod session_archive;
// I-44 — zero-retention-after-verification export mode (post-seal raw-input purge).
pub mod retention;
pub mod session_dag;
pub mod workload_profiling;
// C-32 — Code-Completeness Layer 2: skeleton-first + MiniCritic +
// AcceptAndComplete + deterministic bridge templates.
pub mod skeleton_generator;
pub mod mini_critic;
// E-21 (SCHEMA_VERSION 181): peer-systems harness substrate -- inference-
// time critic rerank + SBFL Ochiai compute + repo-as-state-store +
// ACI tool-shape audit emitter. Wire-dark unless
// `ELAI_PEER_SYSTEMS_ENABLE=1`. Distinct from `mini_critic` (role-time
// signature linter); see crate-doc on `peer_systems::critic_rerank`.
// Plan: docs/thefinalglance/E-plan/E-21_peer_systems_llm_as_worker.md
// Decisions: workflow/reasoning/E-21-decisions.md.
pub mod peer_systems;
// E-36 (SCHEMA_VERSION 182): document + vision RAG substrate.
// Visual artifact verifier (10th VerificationMethod variant
// dispatch surface) + policy gate (per-project enable + cloud-mode
// blocker). Producer for the single-variant Pattern-2
// DocumentVisionRagEvent (34th application). Wire-dark unless
// `ELAI_DOCUMENT_VISION_RAG_ENABLE=1`. Composes with
// `knowledge::chunking::image_artifact::ImageArtifactChunker` +
// `knowledge::chunking::pdf_structured::PdfStructuredChunker`.
// Plan: docs/thefinalglance/E-plan/E-36_document_vision_rag.md.
// Decisions: workflow/reasoning/E-36-decisions.md.
pub mod document_vision_rag;
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
// F-6 / Slice A1 public surface: the skill ladder + store are reached by the
// autonomy gate (candidate_skill_tier evidence), the C6b acquisition hook, the
// C6c scheduler tick, and the CLI/dashboard library surfaces.
pub use skill_trust::{
    DemotedSkill, LearnedSkill, SkillStore, SkillStoreError, SkillTrustTier,
    SKILL_EXPECTED_LIFETIME_SECS, SKILL_STALE_THRESHOLD, TIER_MAX_FAILURE_RATE,
    TIER_MIN_AGE_SECS, TIER_MIN_INVOCATIONS,
};
// H-50: re-export the public policy surface so live_cli.rs + contract tests
// reach it via `elai_orchestrator::{ContextPolicy, effective_context_window,
// trim_threshold_for_mode, regime_policy, context_policy_disabled}`.
pub use context_policy::{
    context_policy_disabled, effective_context_window, regime_policy,
    trim_threshold, trim_threshold_for_mode, ContextPolicy,
    ELAI_CONTEXT_POLICY_DISABLE_VAR, ELAI_LOCAL_MAX_CONTEXT_VAR,
    LOCAL_DEFAULT_WINDOW, CLOUD_DEFAULT_WINDOW, LEGACY_FLAT_THRESHOLD,
    LOCAL_TRIM_FRACTION, CLOUD_TRIM_FRACTION,
    PressureLevel, pressure_level, usable_context_tokens, DEFAULT_MAX_OUTPUT_CAP,
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
// H-46: HGM clade-metaproductivity parent-selection key.
pub mod clade_metaproductivity;
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
// F-8 SCF: LLM-free symbolic semantic conflict detector for the self-mod
// pre-execution path. Pure/deterministic guard SIGNAL (WARN-then-escalate,
// never fail-closed) — see workflow/reasoning/F-8-decisions.md.
pub mod symbolic_conflict;
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
pub mod skill_telemetry;
// E-47: LoRA fine-tuning ship-gate. Triple-AND check (enabled +
// confirm_user_understanding + calibration passing) gates the boot-path
// adapter-load step. Wire-dark unless ELAI_LORA_SHIP_GATE_ENABLE=1.
// Producer-side LoRA training stack (Unsloth + QLoRA-NF4 + PiSSA + DoRA)
// is RUNPOD-PENDING; the gate ships locally so the default-deferred
// posture from workflow/improvement-sweep-2026-05-19/reports/47-lora-finetuning.md
// is enforced from day one. See workflow/reasoning/E-47-decisions.md.
pub mod lora_ship_gate;
pub mod ssb;
// J-28: token ledger -- usage unification. Rollup + estimator + T7
// compaction-growth rule over `ssb::UsageTurnEventPayload` rows written at
// the cloud (grounding_client.rs) + local (runtime_orchestrator.rs) emit
// seams. Sole production reader: `crates/cli/src/usage_cmd.rs::run_usage`
// (the `elai usage` verb).
pub mod usage_ledger;
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
// E-9: external agent-eval benchmark substrate. Compiles unconditionally
// (host-side scaffolding); live eval emission gated by
// `ELAI_EVAL_BENCHMARK_ENABLE=1`. The 11 external benchmark suites
// (swe-bench-pro, swe-bench-live, livecodebench-v6, tau2-bench, bfcl-v3,
// gaia-l1-l2, clawsbench-split, refinebench, sycophancy-probes,
// math-floors, deep-planning) are registered in `external_benches::list_suites`.
// See workflow/reasoning/E-9-decisions.md.
pub mod bench_significance;
pub mod ds_cp_validator;
pub mod external_benches;
// E-7: darwinian self-improvement substrate (wire-dark). Feature-gated
// behind `darwinian-evolve`; all live evolution loops gated additionally
// behind `ELAI_DARWINIAN_LOOP_ENABLE=1`. See workflow/reasoning/E-7-decisions.md.
// Paper grounding: arXiv:2510.04399 (Statistical Limits / Two-Gate) +
// arXiv:2305.10718 (Discounted Thompson Sampling) + arXiv:2506.13131
// (AlphaEvolve MAP-Elites) + arXiv:2507.05619 (EST). REMOVED:
// arXiv:2602.12413 (wrong paper per DECISION-3). Krakovna spec-gaming
// spreadsheet seeds the trap_tasks pattern enum.
#[cfg(feature = "darwinian-evolve")]
pub mod template_archive;
#[cfg(feature = "darwinian-evolve")]
pub mod promotion_gate;
#[cfg(feature = "darwinian-evolve")]
pub mod est_probe;
#[cfg(feature = "darwinian-evolve")]
pub mod cross_domain_canary;
#[cfg(feature = "darwinian-evolve")]
pub mod trap_tasks;
#[cfg(feature = "darwinian-evolve")]
pub mod archive_crypto;
#[cfg(feature = "darwinian-evolve")]
pub mod l3_lock;
#[cfg(feature = "darwinian-evolve")]
pub mod darwinian_bai;
// F-11: config-DGM admit-loop driver core. Closes ONE config-only generation
// (propose -> evaluate -> four-guard chain -> insert_variant -> select_parent)
// and emits SsbActionType::DarwinianAdmit rows. Wire-dark behind
// darwinian-evolve + ELAI_DARWINIAN_LOOP_ENABLE=1. The CLI surface
// (crates/cli/src/darwin_loop_cmd.rs) is a thin wrapper around
// run_one_generation. See workflow/reasoning/F-11-decisions.md.
#[cfg(feature = "darwinian-evolve")]
pub mod darwin_admit_loop;
// H-43: execution-grounded DGM fitness scorer. Replaces the blake3-of-bytes
// derived_alpha placeholder (GAP-1) with a measured pass-rate signal
// (CodeRM arXiv:2501.01054 + CodeT arXiv:2207.10397).
// Always compiled (the scorer struct + fn are feature-agnostic; the SSB emit
// and darwin_admit_loop wiring are feature-gated via darwinian-evolve).
// See workflow/reasoning/H-43-decisions.md.
pub mod execution_fitness;
// H-45: worst-case / variance-penalized promotion scoring over lineage-diverse
// verdicts (Coste 2310.02743 WCO/UWO + Eisenstein 2312.09244 + Co-rewarding
// 2508.00410 + PRMBench 2501.03124 tau calibration). Replaces the candidate's
// own mean fitness as the two_gate_promote margin. Always compiled (pure
// aggregation logic; darwin_admit_loop wiring is feature-gated).
// See workflow/reasoning/H-45-decisions.md.
pub mod promotion_score;
// H-44: behavioral trap detection + measured recall. Replaces the keyword-grep
// placeholder (GAP-2, trap_tasks.rs::score_candidate_for_traps) with three
// behavioral detection legs (ImpossibleBench 2510.20270 / IsomorphMismatch
// 2604.15149 / CodeHalu 2405.00253) and a frozen TRACE-style recall measurement
// (2601.20103). Feature-gated via darwinian-evolve (same kill-switch as
// trap_tasks + darwin_admit_loop) since it composes trap_tasks.
// See workflow/reasoning/H-44-decisions.md.
#[cfg(feature = "darwinian-evolve")]
pub mod trap_behavioral;
// E-8: self-evolving prompts substrate. The LPO span guard
// + structured-auditor-feedback exporter + gotchas-injection helper
// are unconditional (no E-7 surface required); they support the
// optimizer admit path AND the default-features build's LPO contract
// tests. See workflow/reasoning/E-8-decisions.md.
pub mod baby_step_spec_lpo;
pub mod auditor_feedback_export;
pub mod gotchas_inject;
pub mod optimizer_admit;
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
// D-3 (closure): concrete VerifierAdapter implementations
// (DeterministicAdapter / IntrinsicAdapter / PpmAdapter) consumed by
// run_consensus_pass via the task_runner's post-task dispatch_verifiers
// hook. The model-free DeterministicAdapter is the production default the
// boot-site build_verifier_adapters wires under ELAI_VERIFIER_ENABLE; the
// observable smoke drives it with no model. See workflow/reasoning/
// D-3-decisions.md.
pub mod verifier_adapters;
// I-17: tri-layer evaluation stack (the veto-supreme branch scorer). Composes the
// SHIPPED substrate (D-11 checker registry + F-1 VibeGate + F-9 dep-gate at L1;
// I-23 conformal calibration at L2; D-13 cross-family adversary at L3) into a
// per-branch VerdictRow + TriLayerVerdict SSB row (Pattern-2 4-arm:
// HardVeto/ScoredSurvivor/DisagreementEscalated/FlakeQuarantined). NEVER restates
// the substrate (GD-1); ID-4 veto supremacy is structural (literal 0.0 early
// return, l2_call_count==0). Wire-dark via ELAI_TRI_LAYER_ENABLE; consumer is I-4
// (Wave-4). Production hook: task_runner::dispatch_tri_layer_verdict.
pub mod verifier_stack;
// I-4 (Wave-4 KEYSTONE): evidence-ranked search ladder. Consumes the I-17
// tri-layer stack, the I-5 pristine context filter, and the I-20 mutation
// receipts to rank K candidate branches by verifier evidence (best-of-N v1) and
// emit SearchNodeEvent rows. Default-OFF via ELAI_SEARCH_LADDER_ENABLE (GD-4
// measured-first-win). Production hook: task_runner::dispatch_search_ladder.
pub mod search;
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
// E-55: Agentic RL + reward ladder substrate -- RLAIF judge (cloud Sonnet
// on local-Qwen rollouts) + GEPA reflective prompt evolution + hybrid
// PRM+ORM reward fusion + MONA composite-reward hacking guards +
// SFT-on-rollouts collection surface + 3-condition unlock gate for
// DPO/PPO/GRPO/KTO weight-update modules (which DO NOT exist by design).
// Wire-dark unless ELAI_AGENTIC_RL_ENABLE=1. Live training runs RUNPOD-
// PENDING. See workflow/reasoning/E-55-decisions.md.
pub mod rl;
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
// I-23: unified conformal calibration facade — pure re-export of
// conformal_calibration + calibration_bootstrap; the single import path
// for all calibration consumers after the I-23 migration.
pub mod calibration;
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
// I-24 (SCHEMA_VERSION 266): isomorphic canary anti-gaming (scorer-side bias
// detection). Injects AST-preserving code-pair canaries into the i-17 L2 scoring
// path; a scorer whose delta exceeds the ELAI-measured threshold is flagged and
// (after N consecutive detections) neutralized for the run. R8: pair set FROZEN
// in fixtures/canary_pairs.json. Composes i-17 TriLayerStack L2 leg (GD-1).
// Wire-dark: ELAI_CANARY_INJECTOR_ENABLE=1 required. See
// workflow/reasoning/I-24-decisions.md.
pub mod canary_injector;
// G-10 (SCHEMA_VERSION 218): exogenous model-version behavioral-regression gate.
// Composes D-7 canary.rs (GD-1: re-implements NONE of it) + G-2 RunArtifact
// identity tuple to detect per-family regression across an identity change.
// Emits ModelDriftSuspected SSB rows on drift (SURFACE not auto-swap; GD-4).
// Default-ON; kill-switch ELAI_MODEL_DRIFT_DISABLE only (W-2). See
// workflow/reasoning/G-10-decisions.md.
pub mod model_drift;
// C-27: Testing-trust Tier T(-1) DeterminismConfig + Tier T-0 WiringContract.
// Surface re-exports below for the CLI driver and contract crate. See
// workflow/reasoning/C-27-decisions.md.
pub mod determinism;
pub mod wiring_contract;
// WIRING-FIX-ELAI Phase 2 (SP3) — Layer-3 entrypoint-coverage runner.
pub mod wiring_coverage;
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
// E-46 (SCHEMA_VERSION 184) — planning-decomposition substrate. Layered
// additively atop the shipped dual_ledger / agentless_shortcut /
// goal_drift_canary modules so the existing 4 downstream consumers (incl
// the BAI promotion-gate seam) keep compiling unchanged. Wire-dark unless
// ELAI_PLANNING_DECOMPOSITION_ENABLE=1. See
// docs/thefinalglance/E-plan/E-46_planning_decomposition.md +
// workflow/reasoning/E-46-decisions.md.
pub mod planning_decomposition;
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
// H-41 — live checker-coverage roll-up + Xu Table-3 complexity-taxon classifier.
// Instruments the shipped D-10/D-11 sound-checker-first preference (GD-1: reuse,
// not reimplement). Produces: mode_a_coverage roll-up, classify_complexity_taxon,
// checker_gap_backlog. Emitted SSB beacon: CheckerGapDetected (219/530).
// See docs/thefinalglance/H-plan/H-41_checker_coverage_gap_instrument.md.
pub mod checker_coverage;
// H-42 — serial-depth-scaled GRC reasoning budget + depth-not-width guard.
// estimate_serial_depth (from DecompositionLevel) + scale_grc_budget (monotone,
// LOCKED_BUDGET_CEILING bounded) + classify_failure (FailureClass struct with
// depth_bound + access_bound flags) + ReasoningBudgetScaledEmitter trait +
// VecReasoningBudgetEmitter (test emitter). scale_and_emit is the combined
// helper used by both dispatch sites. Default-ON; kill-switch
// ELAI_REASONING_BUDGET_SCALE_DISABLE. Emitted SSB beacon:
// ReasoningBudgetScaled (220/531).
// See docs/thefinalglance/H-plan/H-42_serial_depth_reasoning_budget.md.
pub mod reasoning_budget;
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
// H-39 — Abstention-aware promotion + eval scoring. Closes the self-improvement
// loop misalignment: D-16's RefusalDecision::Abstain fires at inference but the
// reward darwin_admit_loop climbs is abstention-blind. Builds AbstentionAwareScore
// + AbstentionAggregate, wires into promotion reward + coding_metrics + bench
// scorers, emits AbstentionScored SSB row. Grounding: arXiv:2509.04664.
pub mod abstention_score;
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
// I-20 — property-based-testing relation adapter (proptest/Hypothesis as a
// MetamorphicRelation::PropertyBased). Extracted from metamorphic_testing.rs
// to keep that file under the 1000-line maintainability boundary; registered
// behind the existing RelationLibrary dispatch (GD-1, no new dispatch layer).
pub mod metamorphic_pbt;
// I-20 (Track 15 KEYSTONE) — budgeted mutation scheduler: diff-cone selector
// + Wilson-CI adaptive sampler + early-stop + SurvivorDirective emitter +
// EquivalentMutantCache key extension + ID-5 baseline TOML. EXTENSION on
// C-28 (mutation_testing) + C-29 (metamorphic_testing); composes via their
// interfaces, never re-implements. Cone-select wired into
// deterministic_auditor::check_mutation; MutationBudgetedRun SSB rows emitted
// from deterministic_audit_gate::evaluate. See
// workflow/reasoning/I-20-decisions.md.
pub mod mutation_budget;
// E-13 — TDD verification agents module (Red/Green/Refactor cycle +
// anti-mock heuristic + cargo-mutants subprocess wrapper +
// D-3-aggregator adapter). Composes with `verification_result`'s
// `ConsensusRule::VerifierKindHeterogeneous` so we don't ship a
// parallel aggregator. See workflow/reasoning/E-13-decisions.md.
pub mod tdd_verifier;
// Tranche B-7 triple (E-13 SHIPPABLE-NOW) -- rusqlite-backed survival
// cache for cargo-mutants survivor rows. Wire-dark default; gated by
// `ELAI_E13_SURVIVAL_CACHE_ENABLE=1`. See `survival_cache::SurvivalCache`
// + workflow/tooling/mutant-hunter.md.
pub mod survival_cache;
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
// E-3 sub-phase 4A (SCHEMA_VERSION 190): per-role prompt builder +
// VoiceAnchor loaded from .elai_cc/config/voice.toml. Pure module (no
// I/O on the hot path); consumed by runtime_orchestrator::try_complete
// to wrap the system prompt with a unified voice preamble BEFORE the
// adapter dispatch so every backend sees the same anchor. Wire-dark
// unless ELAI_E3_RUNTIME_HANDOFF_ENABLE=1. See
// docs/thefinalglance/E-plan/E-3_model_handoff_ux.md.
pub mod handoff_payload;
pub mod role_predict;
pub mod role_prompt;
pub mod runtime_config;
pub mod runtime_event_sink_ssb;
pub mod runtime_orchestrator;
// E-32: deep-mode escalation hook for the SessionDag handoff threshold.
// Decides between local deep mode (Qwen2.5-7B-Instruct-1M, DCA, 64K) and
// cloud escalation when context budget approaches 85% of effective_max_context.
// Source: E-32-decisions.md DECISION 3 + 4; arxiv:2407.16833 Self-Route.
pub mod escalation_policy;
// I-7: cascading escalation waterfall — local-first fan-out (I-4 ladder) ->
// per-branch I-17 tri-layer check -> condense-on-all-fail into an I-11 BeliefState
// -> synchronous H-7 CloudModeRouter::route decision. IC-1: structurally has NO
// path to the H-5 batch client (the batch racing design is dead). Wired into the
// drive_task `maybe_run_escalation_waterfall` hook (default-OFF
// ELAI_ESCALATION_WATERFALL_ENABLE). See docs/thefinalglance/I-plan/I-7_*.md.
pub mod escalation_waterfall;
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
// C-42b (Tranche D, s-22600): orchestrator-side adapter that bridges
// the plugins-crate `authz::McpAuthzSsbEmitter` trait (composed-verdict
// SSB row for the per-call authz decision) into `Ssb::write_payload`.
// Mirrors the trait-in-plugins / impl-in-orchestrator pattern of
// `tool_calibration_ssb_adapter` (C-47) and `tool_gate_ssb_adapter`
// (C-35) so the plugins -> orchestrator dependency stays acyclic.
// See workflow/reasoning/D-trancheD-decisions.md (D7-DUAL-EMIT) +
// workflow/sessions/s-22600/exploration/trancheD-integration.md.
#[cfg(feature = "mcp")]
pub mod mcp_authz_ssb_adapter;
// G-4 (s-56068): orchestrator-side adapter that bridges the plugins-crate
// `RugPullSsbEmitter` trait (live MCP rug-pull / line-jump re-pin gate) into
// `Ssb::write_payload`. Mirrors mcp_authz_ssb_adapter; the row is default-ON
// (kill-switch ELAI_MCP_REPIN_DISABLE only). See workflow/reasoning/G-4-decisions.md.
#[cfg(feature = "mcp")]
pub mod mcp_rugpull_ssb_adapter;
// E-37 U3: MCP-scan classifier SSB emitter. Mirrors mcp_rugpull_ssb_adapter;
// writes an `mcp_scan_classifier_blocked` row when the heuristic scan classifier
// flags malicious tool descriptions at `http_list_tools`.
#[cfg(feature = "mcp")]
pub mod mcp_scan_ssb_adapter;
// G-1: data-plane taint propagation SSB emitter. Writes a taint_flow row when
// an untrusted ingest flows into a destructive sink and the capability gate
// rejects it. No mcp dependency -- unconditional. See G-1-decisions.md.
pub mod taint_flow_ssb_adapter;
// G-3: deterministic runtime blast-radius SSB emitter. Writes a
// blast_radius_blocked row when a fail-closed gate refuses a post-expansion FS
// escape (translate_tool_request) or a denied egress host (fetch_and_extract).
// Default-ON (kill-switch ELAI_BLAST_RADIUS_DISABLE only). See G-3-decisions.md.
pub mod blast_radius_ssb_adapter;
// J-16: blast-radius / anti-regression gate (auditor-owned, OBSERVE-ONLY).
// J-90 Leg A will extend this module with fuse_crosslang_cone.
pub mod blast_radius;
// G-7: lethal-trifecta dual-LLM quarantine SSB emitter. Writes a
// trifecta_quarantined row when the secret-present + untrusted-span + egress
// legs co-occur at translate_tool_request and the untrusted span is held back.
// Default-ON (kill-switch ELAI_TRIFECTA_QUARANTINE_DISABLE only). See G-7-decisions.md.
pub mod trifecta_quarantine_ssb_adapter;
// G-4: STDIO transport leg of the live MCP rug-pull / re-pin gate. Wires the
// schema-pin engine into the REAL default-run tool-discovery seam
// (mcp_stdio::McpServerManager::discover_tools_for_server). Feature-gated on
// `mcp` because it depends on `elai_plugins::mcp_schema_pin`.
#[cfg(feature = "mcp")]
pub mod mcp_rugpull_gate;
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
// E-25 Tranche B-4 (D1): concrete 6-tier compaction-ladder runner. Wraps the
// existing compact + context_budget + telescope_caller helpers behind a sync
// surface that emits one ClaudeCodeArchitectureEvent SSB row per applied
// tier. Wire-dark default OFF (ELAI_E25_COMPACTION_LADDER_ENABLE).
pub mod compaction_ladder;
// E-25 Tranche B-4 (D2): runtime expansion of !`cmd` blocks inside SKILL.md
// bodies. Allowlisted commands only (git/ls/pwd/date); 2s timeout; shell
// metacharacters rejected; emits ClaudeCodeArchitectureEvent per block. Wire-
// dark default OFF (ELAI_E25_SKILL_TEMPLATING_ENABLE).
pub mod skill_templating;
pub mod feasibility_probe;
/// C-30 — Surgical Mode pipeline: DKG blast-radius decomposition + LOC gates.
pub mod surgical_decomposer;
/// C-30 — Oracle answer-level cache (TTL by SourceTier).
pub mod oracle_cache;
pub mod task_runner;
// Phase A1b — production task_runner wiring helpers (build/apply chokepoint
// bundle). Used by `elai run` CLI and the dashboard-v2 Tauri setup() callback.
pub mod runtime_setup;
// I-42 — critique-only operating profile (RunProfile { Orchestrate,
// CritiqueOnly }). Threaded through the `elai run` CLI surface and consulted
// at the executor-install chokepoint in `runtime_setup::apply_task_runner_wiring`.
pub mod run_profile;
/// C-13 — WIP commit squash helper.
pub mod wip_squash;
// I-3 — shared atomic-write helpers (tmp+sync_all+rename) used by all
// checkpoint write sites in this crate.
pub mod util;
// Phase B2 — IPC accept loop: bind/accept/route cycle for orchestrator-side
// agent session management. Exposes AgentRpcServer trait + InferOutcome +
// ToolOutcome + bind_session_listener + accept_one + run_agent_session.
pub mod ipc_accept;
// J-1 — perception gateway: condition every inbound image before any model
// sees it. Skeleton + scaffold (T2 wired) + hard-limit router (T3 wired).
// Wire-dark by default (ELAI_PERCEPTION_GATEWAY_ENABLE=1 to enable; GD-4).
// Plan: docs/thefinalglance/J-plan/J-1_perception_gateway.md.
pub mod perception;
// Phase B2c — ProductionAgentRpcServer: impls AgentRpcServer wrapping
// AnthropicClient (sidecar or cloud via ELAI_GROUNDING_BACKEND) and
// InProcessToolDispatcher.
pub mod production_agent_rpc;
// H-13 — Canvas op dispatcher (mode-namespace gating + sanitizeAction) and
// per-mode constrained grammar infra (CI compile-ceiling + grammar_unavailable
// fallback). See workflow/reasoning/H-13-decisions.md.
pub mod canvas_dispatcher;
pub mod canvas_grammar;
// H-15 — Canvas UI action types + taint-gated Select handoff.
// CanvasUiAction enum + the Select gate reusing capability_flow::taint_text
// + G-1 TaintFlowCtx::lookup_for_argv. See workflow/reasoning/H-15-decisions.md.
pub mod canvas_ui_action;
// D-14 — sycophancy probe substrate (Sharma et al. arXiv:2310.13548).
// Defensive detector: flags suspected sycophancy via inverted-framing
// challenge per §3.2; does NOT auto-correct (PM does not reliably fix
// sycophancy per §4 — 75% PM residual, 25% oracle floor).
pub mod sycophancy;
// E-19 — reasoning quality probe wrappers (pre-L2 confidence gate, PRM-lite
// L4 scoring, LToT lateral split, L6 verifier diversity). Wire-dark unless
// ELAI_REASONING_QUALITY_PROBE_ENABLE=1. See workflow/reasoning/E-19-decisions.md.
pub mod reasoning;
pub mod supervision;
pub mod task_packet;
pub mod task_registry;
pub mod team_cron_registry;
// E-23 -- 2026 developer-discourse positioning + UX-guard trust telemetry
// plane. Wire-dark unless ELAI_DEVELOPER_DISCOURSE_ENABLE=1. Reads existing
// SSB rows only; the producer side is the dashboard-v2 Tauri command.
pub mod trust_telemetry;
pub mod worker_boot;
// H-17 (SCHEMA_VERSION 223): persisted untamperable trial ledger.
// The selection-bias keystone T-1: a single register_trial() choke-point +
// append-only strategy_trials table + read API for the H-18 DSR/PBO gate.
// Default-ON; kill-switch ELAI_TRIAL_LEDGER_DISABLE. See
// workflow/reasoning/H-17-decisions.md.
pub mod trial_ledger;
// I-12 (SCHEMA_VERSION 251): evidence-gated playbook curator. Promotes rule entries only
// when linked to >=k SSB-verified successful outcome rows; carries measured lift; demotes
// by the same ledger; exports verified content to AGENTS.md / skills formats.
// Feeds the Phase-3F PROPOSED->CANDIDATE->SHADOW->ACTIVE->RETIRED lifecycle (never replaces it).
// Production callers: (a) offline tick in LiveCli::build_local_knowledge_stack,
//   (b) skills_cmd::run SkillsAction::PlaybookExport arm. Default-ON (ID-6 ledger is mandatory).
pub mod rule_curator;
// H-18 (SCHEMA_VERSION 224): Deflated Sharpe / PBO / MinBTL promotion gate.
// The #1 net-new trust gate (T-2). Fail-closed on TrialCountMissing (TD-1)
// and absent PBO (R1/R2). Thresholds LOCKED in mutable=0 rule seed.
// Default-ON; kill-switch ELAI_DEFLATION_GATE_DISABLE. See
// workflow/reasoning/H-18-decisions.md.
pub mod deflation_gate;
// H-20 (SCHEMA_VERSION 225): leakage static-analysis COMPOSITE gate. Composes
// the rules-engine GraphReachability over the DKG (fit-site reaches BOTH
// train+test sink => SoundBlock L1.2/L1.3; no split => L1.1). Sound for only
// 3 of the 8 Kapoor classes; the residue (L1.4/L2/L3.x) routes to human
// Layer-3. FAIL-CLOSED on absent/empty DKG. Default-ON. See
// workflow/reasoning/H-20-decisions.md.
pub mod leakage_gate;
// H-21 (SCHEMA_VERSION 226): runtime look-ahead / causality DIFFERENTIAL gate.
// A PURE perturb-and-diff (full-data vs future-truncated feature column-set);
// any column that changes under truncation proves a look-ahead leak (FutureLeak
// -> Block). The un-gameable runtime backstop to the H-20 static leakage
// COMPOSITE (R5). FAIL-CLOSED on missing/empty truncated run + shape/column-set
// mismatch. Tier-2 Python feature-runner (re-runs the pipeline twice) DEFERRED
// (RunPod-pending). See workflow/reasoning/H-21-decisions.md.
pub mod lookahead_differential;
// H-22 — trading metamorphic NULL-TEST gate (SCHEMA_VERSION 227). A NEW
// f64-domain Tier-1 gate: does the edge survive a block-permutation null? A
// genuine edge sits in the tail (small p = (C+1)/(K+1), Ojala & Garriga 2010)
// and is significant after Bonferroni N-correction (N from H-17) -> ALLOW; an
// overfit edge sits inside the null band (p >= alpha/N) -> it survived the null
// -> BLOCK. FAIL-CLOSED on missing series / under-powered K / TrialCountMissing
// / NaN. Emits a TradingNullTest SSB row (off-agent artifact). Reuses MrViolated
// for per-relation MR failures; the Tier-2 Python permutation sidecar is frozen
// + deferred (cross-lang-skip). See workflow/reasoning/H-22-decisions.md.
pub mod trading_null_gate;
// H-23 (SCHEMA_VERSION 228) — same-seed re-run hash-equality METRIC_GATE (the
// reproducibility centerpiece, T-7). verify_rerun_digest blake3-hashes the
// DECLARED metric outputs of two same-seed re-runs and asserts byte-equality; a
// hidden unseeded RNG makes run #2's digest differ -> Divergent Block. PURE
// re-run-and-diff: REUSES the RRA blake3 16-hex convention + DriftComparator
// first-divergence (per-metric digests wrapped as ActionTypeSequence). Emits a
// RerunHashEquality SSB row on every decision (off-agent artifact). FAIL-CLOSED
// on missing second run / missing seed (NOT silent seed=0, hazard A) /
// empty-or-trivial declared outputs (hazard B). Same-env/same-seed scope (TD-5);
// the Tier-2 Python experiment-runner is deferred (cross-lang-skip). See
// workflow/reasoning/H-23-decisions.md.
pub mod rerun_equality;
// H-24 (SCHEMA_VERSION 229) — sim-to-real parity gate (T-8). Part C (PURE Rust):
// sim_real_parity::verify_replay_golden blake3-EXACT-equality-checks a seeded
// replay event log against a FROZEN golden event log; any divergent event proves
// a sim-real divergence -> Block (first named via the reused DriftComparator).
// Part A (mode-divergent-branch AST ban SCOPED to the order/signal execution path
// + single-entrypoint SchemaRequired manifest) is SEEDED mutable=0 in
// crates/rules/seed/repro/sim_real_parity.toml. Part B (numeric live-vs-backtest
// reconciliation) is DEFERRED Tier-2 (GD-5): structural parity is necessary-not-
// sufficient. FAIL-CLOSED on missing replay / missing-or-empty golden / event-set
// mismatch. Same-env/same-seed scope (TD-5); the Tier-2 live feed is deferred
// (cross-lang-skip). See workflow/reasoning/H-24-decisions.md.
pub mod sim_real_parity;
// H-25 (SCHEMA_VERSION 230) — result manifest re-derivation gate. verify_result
// re-parses each declared empirical NUMBER from its orchestrator-captured output
// (RESTRICTED parser grammar, NO string-eval) and asserts parsed==declared, then
// re-hashes the pinned inputs (captured_output/data/env/commit); any mismatch
// proves a fabricated number -> Block (first result named via the reused
// DriftComparator). Generalises the F-4 stdout_hash anti-fab leg from test-PASS to
// empirical-RESULT. Pure Rust; the produced-vs-declared completeness diff is the
// deferred Tier-2 (cross-lang-skip). See workflow/reasoning/H-25-decisions.md.
pub mod result_manifest;
// H-26 (SCHEMA_VERSION 232) — shadow-execution + financial-invariant metamorphic-
// relation gate. diff_shadow_impls diffs two fixture-supplied result sets within a
// LOCKED float band AND checks textbook-correct financial invariants: accounting
// IDENTITIES EXACT in integer cents, option-pricing / numeric MRs within the band.
// Pure Rust; the two-impl re-execution runner is the deferred Tier-2 (cross-lang-
// skip). See workflow/reasoning/H-26-decisions.md.
pub mod shadow_exec;
// H-27 (SCHEMA_VERSION 235) — gate self-validation, the verify-the-verifiers META
// gate. validate_gates drives a FROZEN registry of the 8 Track-3 gates (deflation,
// leakage, lookahead, trading_null, rerun, sim_real, result_manifest, shadow_exec),
// replays each gate's OWN demo-violated fixture as a planted mutant through its REAL
// async evaluate_* fn, and asserts a BLOCK-class verdict (kill the mutant). Reuses the
// mutation_score==100.0 SEMANTICS, NOT the cargo-mutants MutationRunner. Fail-closed on
// empty registry / surviving mutant / over-block / un-runnable gate. Validates the 8
// OTHER gates, NEVER itself (no infinite regress). See workflow/reasoning/H-27-decisions.md.
pub mod gate_self_validation;
// H-28 (SCHEMA_VERSION 236) — the current-RF reward-tampering gate (Everitt
// arXiv:1908.04734). check_reward_tampering consumes a fixture triple (frozen-evaluator
// hash baseline, observed hash, claimed/prior reward + band) and detects reward tampering
// via two sound Tier-1 legs: a frozen-evaluator blake3 MISMATCH => EvaluatorModified
// HARD-BLOCK (leg 3a, reuses the ArchiveVerifyResult tripwire semantics; also the
// fail-closed arm for a missing baseline), and an implausible reward delta with the hash
// INTACT => ImplausibleRewardDelta FLAG -> human Layer-3 (leg 3b). TD-3 ordering: leg 3a is
// checked FIRST and short-circuits before 3b (a hash mismatch is dispositive). Tier-1 = the
// always-compiled `elai reward-check` CLI; the live darwin_admit_loop hook is feature-gated
// (Tier-2/deferred). See workflow/reasoning/H-28-decisions.md.
pub mod reward_tampering;
// H-28-WIRE-1: integration smoke for reward-tampering gate in darwin admit loop.
#[cfg(all(test, feature = "darwinian-evolve"))]
mod reward_tamper_admit_observable_smoke;
// H-19 — reusable-holdout query budget (the Thresholdout DP gate; Dwork et al.
// arXiv:1506.02629). HoldoutBudgetLedger is an append-only budget counter (queries_spent =
// COUNT of QuerySpent rows; NO agent-facing reset/delete/update/provision method, mirrors the
// H-17 trial_ledger) over the holdout_budget table on the Ssb WAL. query_holdout runs the
// EXACT Thresholdout query (T_hat=T+Lap(2*sigma), eta~Lap(4*sigma), xi~Lap(sigma),
// sigma=tau/(96*ln(4*m/beta)), T=3*tau/4; Laplace ONLY, seeded ChaCha8 GD-2, NO new dep):
// below-threshold queries answer FREE from the training set, above-threshold queries charge
// 1 budget unit and release the Laplace-noised holdout statistic. Budget exhausted => REFUSE
// (fail-closed). provision_fresh_holdout is a FREE FUNCTION (human/frozen path, NOT an agent
// method) that asserts the new data_window is DISJOINT from the spent window (covert reset-
// attack guard). Tier-1 = the always-compiled `elai holdout-check` CLI. See
// workflow/reasoning/H-19-decisions.md.
pub mod holdout_budget;
// H-47 (SCHEMA_VERSION 225) — DGM eval firewall: date-gate + template-regen +
// holdout-budget binding. Composes D-6 held_out, E-54 contamination filter,
// and H-19 Thresholdout budget into a single pre-score gate for
// darwin_admit_loop::run_one_generation. Emits EvalFirewallEvent SSB row.
// See workflow/reasoning/H-47-decisions.md.
pub mod eval_firewall;
// H-53 (SCHEMA_VERSION 229): citation pre-binding + constrained synthesis on the answer
// path. Freezes ranked evidence into a numbered EvidenceAnchorSet (blake3-pinned,
// rank-stable under ELAI_SEED) BEFORE synthesis; E-48 CitationVerifier degenerates from
// open-world ghost-hunting to a closed-set membership + support check. Default-ON
// kill-switch: ELAI_CITATION_PREBIND_DISABLE (not a default-OFF gate; R9).
// Tier-2 (constrained-decode grammar via C-26 PER_ROLE_DECODING) DEFERRED.
// See workflow/reasoning/H-53-decisions.md.
pub mod citation_prebind;
// H-29 (SCHEMA_VERSION 238) — the LOCKED fail-closed drift state machine (T-13): the master
// deployment-trust interlock that gates resuming a live trading strategy. Four closed-form
// drift axes (PSI on inputs+outputs, blake3 schema-fingerprint mismatch, rolling-14d
// live-vs-backtest deviation, 90d validation TTL) + composed G-10/D-7/H-18 signals; a
// persistent journal-derived latch (is_halted = COUNT('halt') > COUNT('unlock')) that DEFAULTS
// to HALTED on missing/un-evaluable evidence and SURVIVES restart; the human unlock is a FREE
// FUNCTION requiring fresh re-validation evidence (NOT an agent method). Tier-1 = the
// always-compiled `elai drift-check` CLI. See workflow/reasoning/H-29-decisions.md.
pub mod drift_interlock;
// Phase 4J / B-25 — Empire integration (host-side scaffolding only). Gated
// behind cfg(feature = "empire") so non-empire builds pay zero compile-time
// or runtime cost (workspace-level `cargo build --no-default-features`
// excludes the submodule entirely). Every external-IO seam inside is
// labelled `EMPIRE-PENDING: <what-to-wire>` so a single follow-up patch can
// locate and flip every wire-point when Empire deploys. See module doc for
// the layout + decision-doc workflow/reasoning/phase-4J-decisions.md.
#[cfg(feature = "empire")]
pub mod empire;

// I-11: typed, versioned BeliefState for cross-model / cross-tier handoffs.
// Replaces raw session-history passthrough on the cloud escalation path.
// Wired into task_runner (adaptive_reduce + evaluate_handoff production callers).
pub mod belief_state;
pub use belief_state::{
    run_honesty_job_if_due, BeliefState, BeliefStateError, HonestyJob, HonestyDeltaPayload,
    TriedApproach, BELIEF_STATE_SCHEMA_VERSION, HONESTY_JOB_INTERVAL,
};

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
    AntiFabricationGate, DairaScope, DairaScopeResult, DiagnosticProbeGate, DkgChangelogGate,
    EvidenceError, EvidenceGate, FindingSeverity, GroundingSweepGate, IntegrationTestRowSummary,
    IntentPairsGate, KnowledgeReadiness, KnowledgeReadinessGate, ReplanFastTrackGate,
    TestFailReasonsGate, TestFlippedGate, TestOutcome, TransitionContext, ZeroBlocksGate,
};
pub use gate::{
    default_risk_level, run_interactive_gate, EvidenceSummary, GateDecision, GateResult,
    InteractiveGate, InteractiveGateOutcome, RiskLevel,
};
// MockInteractiveGate is exported unconditionally (not just #[cfg(test)]) so
// external contract test crates can construct mock gates without depending on
// a test-only re-export. The struct is clearly labeled "Mock" and is only used
// in tests; this does not affect production binaries (dead code elision).
pub use gate::MockInteractiveGate;
// H-38 — cost-confirmation gate predicate surface.
pub use stop_cost_gate::{
    evaluate_stop, StopCostSnapshot, StopGateVerdict, CONFIRM_COST_FLOOR_USD,
};
// Batch 3B (Phase 4G) — AutonomousGate surface.
pub use autonomous_gate::{
    autonomous_gate_disabled_via_env, AutonomousGate, AutonomousGateConfig, AutonomyDecision,
    EscalationTrigger, Evidence as AutonomousEvidence, GateContext as AutonomousGateContext,
    GateMode, PipelineLevel, TreeAutoAdvancePolicy, ELAI_DISABLE_AUTONOMOUS_GATE_ENV,
};
// I-42 — critique-only operating profile surface.
pub use run_profile::{RunProfile, RUN_PROFILE_CRITIQUE_ONLY, RUN_PROFILE_ORCHESTRATE};
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
// I-1: directive supervisor enforcement plane. Re-exported so the i1 contract
// test (and Tier-2 callers) can construct directives + supervisors and assert
// gate outcomes without reaching into the module path directly.
pub use directive::{
    Directive, DirectiveOutcome, GateOutcome, InProcessSupervisor, Supervisor, SupervisorError,
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
    spawn_mcp_stdio_process, DeferredResolveInfo, JsonRpcError, JsonRpcId, JsonRpcRequest,
    JsonRpcResponse, ManagedMcpTool, McpDiscoveryFailure, McpInitializeClientInfo,
    McpInitializeParams, McpInitializeResult, McpInitializeServerInfo, McpListResourcesParams,
    McpListResourcesResult, McpListToolsParams, McpListToolsResult, McpReadResourceParams,
    McpReadResourceResult, McpResource, McpResourceContents, McpServerManager,
    McpServerManagerError, McpStdioProcess, McpTool, McpToolCallContent, McpToolCallParams,
    McpToolCallResult, McpToolDiscoveryReport, StdioSchemaCache, UnsupportedMcpServer,
    ALWAYS_AVAILABLE, ELAI_MCP_DEFER_SCHEMA_ENABLE_ENV,
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
    prebind_research_answer, PrebindOutcome,
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
// I-2: CoW workspace isolation surface. Re-exported at crate root so the BSP
// engine, task_runner dispatch path, and the i2 contract test program against
// `elai_orchestrator::{BranchWorkspace, WorktreePoolImpl, ...}` without the
// `::sandbox::branch_workspace::` path. NOT a replacement for `AgentSandbox`
// (the per-process face) — they compose.
pub use sandbox::{
    BranchRoot, BranchWorkspace, BranchWorkspaceError, WorkspaceMergeResult, WorktreePoolImpl,
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
    // H-41: checker-gap emit surface (wrapper + env-wrapper + trait + test emitter).
    route_claim_with_gap, route_claim_with_gap_env, CheckerGapEmitter, VecCheckerGapEmitter,
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
    ELAI_DISABLE_PIPELINE_GRC_ENV, CSQ_CRITICAL_BLOCK_REASON,
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
    // H-41 (SCHEMA_VERSION 219): checker-gap backlog beacon. Re-exported
    // so the agent_pool gap-emit wrapper + contract crate + dashboard-v2
    // CheckerCoveragePanel commands can construct the payload directly.
    CheckerGapDetectedPayload,
    // H-42 (SCHEMA_VERSION 220): serial-depth-scaled GRC budget decision beacon.
    // Re-exported so the reasoning_budget module + contract crate + dashboard-v2
    // ReasoningDrawer commands can construct the payload directly.
    ReasoningBudgetScaledPayload,
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
    // E-16 (SCHEMA_VERSION 179): VLM image-validation Pattern-2.
    // Re-exported so dashboard-v2 media_validation.rs fold path +
    // contract crate tests can decode rows without `::ssb::*` paths.
    VlmImageValidationEventPayload, VlmImageValidationKind,
    ELAI_VLM_IMAGE_VALIDATION_ENABLE_ENV,
    // F-1 (SCHEMA_VERSION 196): VibeGate security gate payloads +
    // F-2a SecurityRuleBlocked flat beacon. Re-exported so dashboard-v2
    // commands + contract tests can decode rows without `::ssb::*` paths.
    // The two ELAI_*_ENABLE consts are LEGACY (the live emit path is now
    // default-ON behind the ELAI_VIBEGATE_DISABLED kill-switch per B1/W-2);
    // they remain re-exported only for the existing SSB round-trip tests.
    VibeGateBlockedKind, VibeGateBlockedPayload, SecurityRuleBlockedPayload,
    ELAI_VIBE_GATE_ENABLE_ENV, ELAI_SECURITY_RULE_BLOCKED_ENABLE_ENV,
    // F-4 Slice B-1 (SCHEMA_VERSION 197): TestIntegrityEvent Pattern-2 substrate.
    // Re-exported so dashboard-v2 get_test_integrity_events Tauri command +
    // contract tests can decode rows and construct SsbActionType arms without
    // reaching into `orchestrator::ssb::*` directly.
    TestIntegrityEventKind, TestIntegrityEventPayload,
    // F-3p1 (SCHEMA_VERSION 198): ToolFilterEvent Pattern-2 substrate.
    // Re-exported so dashboard-v2 + contract tests can decode rows and
    // construct SsbActionType arms without reaching into `::ssb::*` directly.
    // Emission is unconditional default-ON (W-2: no default-off terminal
    // state); there is no enable-env const to expose.
    ToolFilterEventKind, ToolFilterPayload,
    // F-9 Slice D1 (SCHEMA_VERSION 199): WriteTimeSlopsquatBlocked flat beacon.
    // Re-exported so dashboard-v2 + contract tests can decode rows and construct
    // SsbActionType::WriteTimeSlopsquatBlocked arms without reaching into
    // `orchestrator::ssb::*` directly.
    WriteTimeSlopVerdictKind, WriteTimeSlopsquatBlockedPayload,
    // F-7 Slice A1 (SCHEMA_VERSION 205): TemplateUtilityRanked single flat
    // beacon. Re-exported so the cli template_ranking_cmd emitter +
    // dashboard-v2 + contract tests can construct the payload without
    // reaching into `orchestrator::ssb::*` directly.
    TemplateUtilityRankedPayload,
    // G-2 (SCHEMA_VERSION 210): RunArtifactEmitted reproducible-run artifact
    // beacon. Re-exported so the cli rra_cmd + dashboard-v2 commands_g2.rs +
    // contract tests can decode rows / construct the SsbActionType arm without
    // reaching into `orchestrator::ssb::*` directly.
    RunArtifactEmittedPayload,
    // G-5 (SCHEMA_VERSION 212): ReviewSurfaceScored reviewer-cockpit DSR score
    // beacon. Re-exported so the cli review_cmd emit + dashboard-v2
    // commands_g5.rs + contract tests can decode rows / construct the
    // SsbActionType arm without reaching into `orchestrator::ssb::*` directly.
    ReviewSurfaceScoredPayload,
    // G-8 (SCHEMA_VERSION 213): ProvenanceAttestationEmitted signed in-toto
    // provenance attestation beacon. Re-exported so the cli run_subcommand emit +
    // verify_attestation_cmd + dashboard-v2 commands_g8.rs + contract tests can
    // decode rows / construct the SsbActionType arm without reaching into
    // `orchestrator::ssb::*` directly.
    ProvenanceAttestationEmittedPayload,
    // G-6 (SCHEMA_VERSION 217): SafetyDeltaMeasured gate-ON-vs-OFF SAFEDELTA
    // flat beacon. Re-exported so the cli bench_safedelta.rs emit + dashboard-v2
    // commands_g6.rs + g6 contract tests can decode rows / construct the
    // SsbActionType arm without reaching into `orchestrator::ssb::*` directly.
    SafetyDeltaMeasuredPayload,
    // G-10 (SCHEMA_VERSION 218): ModelDriftSuspected exogenous behavioral-regression
    // beacon. Re-exported so the cli model_drift_cmd emit + run_subcommand + dashboard-v2
    // commands_g10.rs + g10 contract tests can construct the payload without reaching
    // into `orchestrator::ssb::*` directly.
    ModelDriftSuspectedPayload,
    // H-17 (SCHEMA_VERSION 223): TrialRegistered persisted trial count off-agent
    // artifact. Re-exported so the h17 contract test + future H-18 promotion gate
    // can construct the payload without reaching into `orchestrator::ssb::*` directly.
    TrialRegisteredPayload,
    // H-37 (SCHEMA_VERSION 216): PlanDecisionEvent plan-approval gate decision.
    // Re-exported so the h37 contract test + commands_h37.rs dashboard command
    // can construct the payload without reaching into `orchestrator::ssb::*` directly.
    PlanDecisionKind, PlanDecisionPayload,
    // H-38 (SCHEMA_VERSION 217): StopConfirmationEvent cost-confirm gate.
    // Re-exported so the h38 contract test can construct the payload and assert
    // round-trip without reaching into `orchestrator::ssb::*` directly.
    StopConfirmationPayload, StopOutcomeKind,
    // H-39 (SCHEMA_VERSION 218): AbstentionScored bench beacon.
    // Re-exported so the h39 contract test can construct the payload directly.
    AbstentionScoredPayload,
    // I-8 (SCHEMA_VERSION 238): TrajectoryAborted voluntary abort + dead-end hash pruning
    // + halting-bound seam. Re-exported so the i8 contract test can construct / assert
    // payloads without reaching into `orchestrator::ssb::*` directly.
    TrajectoryAbortedKind, TrajectoryAbortedPayload,
    // I-1 (SCHEMA_VERSION 239): DirectiveExecuted directive-supervisor outcome. Re-exported
    // so the i1 contract test can construct / decode payloads without reaching into
    // `orchestrator::ssb::*` directly (mirrors the I-8 TrajectoryAborted re-export above).
    DirectiveExecutedKind, DirectiveExecutedPayload,
    // I-33 (SCHEMA_VERSION 240): GovernancePrimitiveFired CEFL/I6Q/commit-reveal governance.
    // Re-exported so the i33 contract test can construct / decode payloads.
    GovernancePrimitiveFiredKind, GovernancePrimitiveFiredPayload,
    // H-45 (SCHEMA_VERSION 223): PromotionScoreAggregated worst-case/variance-penalized
    // aggregate promotion score. Re-exported so h45 contract test + dashboard can decode.
    PromotionScoreAggregatedKind, PromotionScoreAggregatedPayload,
    // H-46 (SCHEMA_VERSION 224): CladeSelected HGM clade-metaproductivity parent
    // selection beacon. Re-exported so h46 contract test + ssb_viewer clade_selections
    // + dashboard get_clade_selections can construct/decode without reaching into
    // `orchestrator::ssb::*` directly.
    CladeSelectedKind, CladeSelectedPayload,
    // I-32 (SCHEMA_VERSION 241): RoleContractApplied persona-strip / retain / A/B-delta beacon.
    // Re-exported so the i32 contract test can construct / decode payloads.
    RoleContractAppliedKind, RoleContractAppliedPayload,
    // I-35 (SCHEMA_VERSION 242): ToolSequenceVerdict temporal tool-sequence enforcement beacon.
    // Re-exported so the i35 contract test can construct / decode payloads.
    ToolSequenceVerdictKind, ToolSequenceVerdictPayload,
    // I-47 (SCHEMA_VERSION 245): PromptHardeningApplied verifier-input XML tagging +
    // worker rule injection beacon. Re-exported so the i47 contract test + the
    // tools crate prompt-hardening module can construct / decode payloads.
    PromptHardeningAppliedKind, PromptHardeningAppliedPayload,
};
// I-32: role_contracts public surface. Re-exported so report_fns (in elai-cli) and
// the i32 contract test can access RoleContract / RoleContractStore / apply_to_system_prompt
// without reaching into the orchestrator internals.
pub use role_contracts::{
    agent_role_from_snake_case, apply_to_system_prompt, ApplyOutcome, RoleContract,
    RoleContractStore,
};
// I-35: tool_sequence public surface. Re-exported so the i35 contract test and the
// bench_sequence_probe_cmd.rs CLI driver can construct supervisors with sequence
// rules, inject HistoryEntry values, and assert on ToolSequenceVerdict without
// reaching into orchestrator internals.
pub use tool_sequence::{
    emit_tool_sequence_verdict, evaluate_sequence_rule, evaluate_sequence_rules,
    tool_class_from_action_type, EnforcementAction, HistoryBuffer, HistoryEntry,
    SequencePredicate, SequenceRule, SequenceRuleSet, ToolSequenceVerdict,
};
// G-5: reviewer-cockpit DSR scoring engine public surface. Re-exported so the
// cli review_cmd + dashboard-v2 commands_g5.rs + contract tests can score a
// diff + per-file signal set without reaching into `orchestrator::review_surface`
// internals.
pub use review_surface::{
    build_file_signals, hunk_id_for, score_review_surface, FileSignals, FindingKind, HunkVouch,
    ReviewFinding, ReviewSurfaceScore, VouchKind,
};
// H-52: deterministic finding-anchoring public surface.
// ReviewFinding is the UNIFIED struct (review_surface::ReviewFinding) carrying
// snippet + anchor fields (H-52 line-less schema). review_anchor::ReviewFinding
// is a type alias pointing to the same struct; AnchoredReviewFinding is a
// stable alias preserved for call-site compat — both refer to the same type.
pub use review_anchor::{
    anchor_disabled, anchor_finding, AnchorKind, AnchorResult, ExcludeReason,
    ReviewFinding as AnchoredReviewFinding,
};
// G-2: Reproducible-Run Artifact public surface. Re-exported so the cli
// rra_cmd / run_subcommand emit + dashboard-v2 commands_g2.rs + contract tests
// can build / write / verify a .rra without reaching into `orchestrator::rra`
// internals.
pub use rra::{
    build_rra, canonical_objective_bytes, compute_root, read_rra, verify_artifact,
    verify_artifact_with_objective, verify_rra, write_rra,
    GateVerdictHash, GuaranteeLevel, ReplayVerdict, RraError, RraEvent, RunArtifact,
    RRA_COMPRESSION_DEFERRED, RRA_FORMAT_VERSION, RRA_HEX_PREFIX_LEN,
};
// J-14 S5: pure ReproPackEvent payload builders (TierEvaluated +
// FirstDivergenceLocated). No Ssb handle, no async write; the CLI / orchestrator
// layer owns the ssb.write_payload call. Re-exported so rra_cmd.rs and contract
// tests can build payloads without reaching into `orchestrator::rra_emit`
// internals.
pub use rra_emit::{
    build_first_divergence_payload, build_tier_evaluated_payload, downgrades_wire,
    tier_effective_wire, tier_label_wire,
};
// G-8: signed in-toto v1 provenance attestation public surface. Re-exported so
// the cli verify_attestation_cmd + run_subcommand emit + dashboard-v2 commands_g8.rs
// + contract tests can build / sign / write / verify an attestation without
// reaching into `orchestrator::attestation` internals.
pub use attestation::{
    build_attestation, read_sidecar, sign_local_merkle, verify_attestation,
    verify_attestation_against_trailer, write_sidecar, AttestationError, AttestationVerdict,
    InTotoPredicate, InTotoStatement, InTotoSubject, VerifiedLevel, ARTICLE_50_POSTURE,
    ELAI_PREDICATE_TYPE, IN_TOTO_STATEMENT_TYPE, SIGNING_MODEL_DEFERRED_COSIGN,
    SUBJECT_DIGEST_ALG,
};
// F-3p1: capability-flow public surface. Re-exported so dashboard-v2 and
// contract tests can use CommittedToolSet + CapabilityFlowDenial + taint helpers
// without reaching into `orchestrator::capability_flow` directly.
pub use capability_flow::{
    check_against_commitment, commit_minimal_set, committed_slot, taint_from_verdict,
    taint_text, CapabilityFlowDenial, CommittedToolSet, UntrustedTaint,
};
// F-3 Slice C2: CapabilityFlowAdapter -- orchestrator-side impl of the
// elai-plugins::CommittedSetChecker trait seam. Re-exported so session-driver
// and dashboard-v2 bootstrap can wire the adapter into ToolGate without
// reaching into `capability_flow` internals.
#[cfg(feature = "mcp")]
pub use capability_flow::CapabilityFlowAdapter;
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
// WIRING-FIX-ELAI Phase 2 (SP3) — Layer-3 coverage runner public surface.
// `layer3_coverage_clean` + `WiringCoverageReport::is_clean` are the
// cross-boundary symbols the D-EV cluster (WAVE-4) consumes to populate the
// `Evidence::layer3_entrypoint_covered` field; `LAYER3_LEG` is the canonical
// SSB leg discriminator string WAVE-4 selects rows on. `LAYER4_LEG` is the
// symmetric Layer-4 leg WAVE 5 emits and WAVE-4's
// `derive_layer4_trace_present` selects on.
pub use wiring_coverage::{
    layer3_coverage_clean, parse_llvm_cov_json, parse_new_functions_from_diff, CoverageViolation,
    CoveredFunction, NewFunction, SmokeFeature, SmokeManifestLoader, WiringCoverageCmd,
    WiringCoverageError, WiringCoverageReport, LAYER3_LEG, LAYER4_LEG,
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
// H-41 — live checker-coverage + complexity-taxon public surface.
pub use checker_coverage::{
    checker_gap_backlog, classify_complexity_taxon, mode_a_coverage, CheckerGapBacklogRow,
    ComplexityTaxon, CoverageReport,
};
// H-42 — serial-depth-scaled reasoning budget public surface.
pub use reasoning_budget::{
    classify_failure, depth_scaling_disabled_via_env, estimate_serial_depth, scale_and_emit,
    scale_grc_budget, FailureClass, ReasoningBudgetScaledEmitter, VecReasoningBudgetEmitter,
    ELAI_REASONING_BUDGET_SCALE_DISABLE_ENV, LOCKED_BUDGET_CEILING,
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
    // E-46 D-8: planning-decomposition config — re-exported so cli/runtime_config.rs
    // can embed it in ElaiRuntimeConfig without a direct orchestrator::runtime_config
    // import (the orchestrator crate is not the CLI's runtime_config, avoid confusion).
    PlanningConfig, PlanningLedgerConfig, PlanningGoalDriftConfig, PlanningEntropyConfig,
    PlanningDepthCeilingConfig, PlanningReflectionBufferConfig, PlanningSkillPriorsConfig,
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
// C-42b (Tranche D): orchestrator adapter for the plugins-side
// McpAuthzSsbEmitter trait (composed-verdict per-call SSB row).
#[cfg(feature = "mcp")]
pub use mcp_authz_ssb_adapter::OrchestratorMcpAuthzSsbEmitter;
#[cfg(feature = "mcp")]
pub use mcp_rugpull_ssb_adapter::OrchestratorRugPullSsbEmitter;
#[cfg(feature = "mcp")]
pub use mcp_scan_ssb_adapter::OrchestratorMcpScanSsbEmitter;

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
    sample_variant, sample_variant_weights, sigmoid_perf_score, ConfigArchive,
    ConfigArchiveConfig, ConfigArchiveError, DEFAULT_ARCHIVE_CAP_PER_MODE,
    DGM_ALPHA_0, DGM_LAMBDA,
};
// H-46: CMP selection types re-exported for contract tests + dashboard.
pub use clade_metaproductivity::{
    clade_metaproductivity, CladeScore, CmpAggregate, SelectionKey,
    CMP_AGGREGATE, CMP_MIN_CLADE_SIZE,
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
// F-8 SCF symbolic conflict detector re-exports.
pub use symbolic_conflict::{
    conflict_fingerprint, current_process_model, detect_conflicts,
    emit_symbolic_conflict_detected, intent_from_config_variant, intent_from_proposed_change,
    intent_from_structural_change, screen_config_variant_for_conflicts,
    tripwire_quiescent_violated, ConflictKind, IntentCondition, IntentOp, ProcessModel,
    SymbolicConflict, SymbolicIntent, CONFLICT_FINGERPRINT_HEX_LEN, RESOURCE_RAIL_KEYS,
};
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
    detect_scaffold_template, emit_integration_checkpoint_for_test, run_plan_approval_gate_for_test,
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
    truncate_for_log,
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
pub use rule_loader::load_security_seed_rules_builtin;
// Phase A1b — chokepoint helpers exposed at the orchestrator crate root so
// the CLI run subcommand and the dashboard-v2 Tauri setup() callback both
// reach the same RuntimeWiring + apply_task_runner_wiring entry points.
// Contract: `crates/contracts/tests/task_runner_chokepoint_contract.rs`.
pub use runtime_setup::{
    apply_task_runner_wiring, build_default_baby_step_executor,
    build_default_baby_step_executor_with_ipc_hooks,
    build_default_baby_step_executor_with_sandbox, build_production_hook_dispatcher,
    build_production_ipc_hook_dispatcher, build_stub_tiered_analysis,
    build_task_runner_wiring, locate_default_agent_binary, RuntimeWiring,
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
// I-36 (SCHEMA_VERSION 268): per-(role,backend) compiled-prompt artifact store.
// Re-exported so report_fns (elai-cli) + dashboard commands_i36 + i36 contract tests
// can use CompiledPromptStore / CompiledPromptVariant / VariantStatus without reaching
// into orchestrator internals. promote_variant and run_transfer_gap_fixture are exported
// but WIRE-DARK[I-36] (Tier-2 DEFER; see wire-dark-allowlist.toml + stubs.md).
pub mod prompt_artifacts;
pub use prompt_artifacts::{
    CompiledPromptStore, CompiledPromptStoreError, CompiledPromptVariant,
    VariantStatus as CompiledVariantStatus,
    PromoteError, RollbackOutcome,
    promote_variant, rollback_on_regression, run_transfer_gap_fixture, TransferGapResult,
};

#[cfg(test)]
pub(crate) fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
