#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    clippy::unneeded_struct_pattern,
    clippy::unnecessary_wraps,
    clippy::unused_self
)]
mod classifier;
mod commands;
mod dashboard;
mod dep_audit;
// F-9 / Slice D4: `elai dep-check <path> [--offline]` — write-time
// anti-slopsquatting dep-check CLI surface. DEFAULT uses the real live
// SubprocessRegistryChecker; --offline switches to StubRegistryChecker.
mod dep_check_cmd;
mod doctor;
mod handoff;
mod production_checklist_cmd;
mod indexing;
mod index_cmd;
mod init;
mod input;
mod install;
mod render;
mod runtime_config;
mod skills_cmd;
mod repl;
mod remote_sidecar;
mod sidecar;
mod sidecar_hyde;

mod args;
mod auth;
mod c17_lifecycle;
mod help;
mod live_cli;
// T12a: SSB adapter structs (SsbRerankerNotifier / NoopRerankerNotifier /
// SsbDegradationNotifier) extracted from live_cli.rs. Re-exported via
// `pub(crate) use crate::live_cli::*;` below through live_cli.rs glob.
pub(crate) mod ssb_notifiers;
// T12a: HookAbortMonitor + clock_to_epoch_ms extracted from live_cli.rs.
// Re-exported via `pub(crate) use crate::live_cli::*;` below through
// live_cli.rs glob.
pub(crate) mod hook_abort;
// T12b: MCP glue layer (RuntimeMcpState + request structs + tool-definition
// builders + permission-mode helpers) extracted from live_cli.rs. Re-exported
// via `pub(crate) use crate::live_cli::*;` below through live_cli.rs glob, so
// `crate::live_cli::RuntimeMcpState` resolves for runtime_builder.rs and
// cli_consts.rs.
pub(crate) mod mcp_glue;
// T12c: free-standing report/format/utility functions extracted from live_cli.rs.
// Re-exported via `pub(crate) use crate::live_cli::*;` below through live_cli.rs glob.
pub(crate) mod report_fns;
// T12d: `LiveCli::build_local_knowledge_stack` extracted from live_cli.rs as a
// separate `impl LiveCli` block. No re-export needed — the associated fn stays
// on `LiveCli`, so the `Self::`/`LiveCli::` call paths resolve via the type.
pub(crate) mod knowledge_stack;
mod measure;
mod pipeline_subcommand;
mod template_ranking_cmd;
mod symbol_graph;
mod progress;
mod resume;
mod run_subcommand;
mod runtime_builder;
// E-49 SWE-Bench Pro v5 driver — non-interactive task submission with
// FSM + per-role runtime overrides. Subprocess-wrap surface for the
// Python v5 driver. Composes on `start_task_runner` + B-37 toml-edit.
mod task_cmd;
mod wip_squash_subcommand;
// C-27: T(-1)+T-0 wiring contract audit dispatcher.
mod wiring_audit;
// WIRING-FIX-ELAI Phase 2 (SP3): Layer-3 entrypoint-coverage CLI driver.
mod wiring_coverage_cmd;
mod archive_verify;
// D-1: `elai template migrate` — bootstraps the reasoning-template archive.
mod template_migrate;
// C-43: `elai freeze status|verify|ratify` — Phase 4H scope-freeze CLI
// driver. Wraps elai_orchestrator::scope_freeze. ELAI design hypothesis
// — no paper grounding (see workflow/sessions/s-25120/mistakes.md for
// the bad-cite analysis of arXiv:2604.21725).
mod freeze_cmd;
// G-2: `elai rra show <path>` + `elai replay --verify <path>` — the
// Reproducible-Run Artifact viewer + offline verifier. Wraps
// elai_orchestrator::rra. See workflow/reasoning/G-2-decisions.md.
mod rra_cmd;
// I-30 Slice 1b: `elai dataset export-sft` — SFT exhaust from sealed RRA
// bundles. Wraps elai_orchestrator::rra + ssb::SftExhaustEvent.
mod dataset_cmd;
mod sft_exhaust;
mod rra_reexecute_cmd;
// J-14 S6b: `emit_j14_verify_repro_pack_events` split from rra_cmd.rs (MR-1).
mod rra_verify_emit;
// I-40: `elai rra verify <path>` — rra-spec-v1 spec-conformance checker.
// DISTINCT from `elai replay --verify` (content-hash re-derivation; rra_cmd.rs).
// Checks C1-C6 structural requirements from docs/spec/rra-spec-v1.md.
// Exits nonzero on any failure so CI can gate on it. Called from rra_cmd::run_rra.
mod rra_verify_cmd;
// I-40: `elai wrap [--output <path>] [--fixture] -- <agent-cmd...>` — trust-layer
// wrapper proxy. Spins up taint + write-gate trust substrate around an external
// agent command and emits a .rra bundle (G-2 build_rra + G-9 redaction; reuses
// TaintFlow + RunArtifactEmitted SSB types -- NO new action_type, GD-1). FORK-1:
// new top-level CliAction::Wrap with its own Arc<Ssb>. Kill-switch:
// ELAI_WRAP_RRA_DISABLE=1 (default-ON emit).
mod wrap_cmd;
mod wrap_rra_disable;
// K-6: `elai export compliance` — redacted RRA + attestation + audit trail
// + retention statement bundle. Composition only (GD-1).
mod compliance_export_cmd;
// H-35: `elai rewind <target>` — session-state restore to a prior SSB seq.
// Composes compact_session + recover_stubs + ssb_resume_from into a user-facing
// restore affordance; emits one session_rewound SSB beacon per successful rewind.
mod rewind_cmd;
// H-49: `elai undo / redo / history / jump` — CAS-backed reversible LLM edits
// over the undo-tree HEAD. Default-ON; kill-switch ELAI_TIMETRAVEL_DISABLE.
// Composes E-25 edit_checkpoints (GD-1) + SessionDag HEAD + H-35 SessionRewound.
mod timetravel_cmd;
mod review_cmd;
// H-53: `elai prebind [--demo-prebind] [--allow-freeform-citations]` — citation
// pre-binding + constrained synthesis on the /research|/ask answer path.
// W-1 seeded-fixture trigger emits CitationAnchorsBound SSB row (off-agent
// artifact, R4). Default-ON kill-switch: ELAI_CITATION_PREBIND_DISABLE.
mod prebind_cmd;
mod verify_attestation_cmd;
// I-39: `elai plan status [--letter <L>] [--json]` — per-plan state derived from
// FINISH fragments + wiring_contract_check.py + SSB journal. Emits PlanStateDerived
// SSB row (fail-soft if no ssb.db). Default-ON; no kill-switch (pure read path).
mod plan_status_cmd;
// I-41: `elai critique <spec-path> [--lenses L1,...] [--risk simple|heavy] [--mock]`
// Spec-time critique lenses (L1-L8). Advisory-only; zero path to any gate/verdict.
// Kill-switch: ELAI_CRITIQUE_LENSES_ENABLE=1. Emits CritiqueLensFinding SSB rows.
// See workflow/reasoning/phase-I41-decisions.md.
mod critique_cmd;
// C-45: `elai credit attribute|table|export` — CANTANTE contrastive
// credit-attribution CLI driver. Wraps elai_orchestrator::credit_attribution.
// Paper-grounded against arXiv:2605.13295 (Zehle 2026, Section 3.2 — K
// rollouts NOT K^N, scalar c in [-1,+1], g=3 partition default). See
// workflow/papers/cantante-credit-attribution.md.
mod credit_cmd;
// E-39: `elai cloud override-cap <USD> --confirm <typed-string>` /
// `elai cloud status`. Layered-cap manual-override CLI driver.
// Composes with E-38 EU AI Act audit-trail liveness gate. See
// workflow/reasoning/E-39-decisions.md.
mod cloud_cmd;
// C-46: `elai routing decisions|pilot|status|export` — Uno-Orchestra
// parsimonious agent routing (selective delegation) CLI driver. Wraps
// elai_orchestrator::routing. Paper-grounded against arXiv:2605.05007
// (Cui et al 2026, Sections 5 + 6.1 — (model, primitive) action space,
// alpha in [0, 1] convex blend, S_max <= 0.10 cost-hack bound). See
// workflow/papers/uno-orchestra-interface.md.
mod routing_cmd;
// C-47: `elai calibration status|reset-hints|export` — case-based
// tool-use calibration CLI driver. Wraps
// `elai_plugins::ToolCalibrationRegistry`. Paper-grounded against
// arXiv:2605.15041 (CAST — Case-Based Calibration of Adaptive Reasoning
// and Execution for LLM Tool Use). See
// workflow/papers/case-based-calibration.md.
mod calibration_cmd;
// E-14: `elai browser run|list-journeys|status` — browser-e2e agent
// CLI driver. Wraps elai_orchestrator::browser_e2e (Playwright snapshot
// + axe-core gate + journey assertions). Wire-dark for absent
// npx/playwright; VLM tier RUNPOD-PENDING.
mod browser_e2e_cmd;
mod reliability_cmd;
// E-44: `elai durability status|validate-checkpoint|migrate|
//                  continue-as-new-preview|acquire-lock` — durability
// layer operator surface anchoring the orchestrator's `durability::*`
// module as a production callable (WIRE-DARK avoidance per E-41
// precedent). See workflow/reasoning/E-44-decisions.md.
mod durability_cmd;
// E-45: `elai drift detect|list-pairs|baseline` — cross-language drift
// detector operator surface. Anchors cross_lang_drift::DriftDetector +
// PairRegistry as production callables (WIRE-DARK avoidance per E-41
// precedent). See workflow/reasoning/E-45-decisions.md.
mod drift_cmd;
// E-49: `elai metrics capture|swe-bench-pro|refinebench|aggregate|status` —
// coding-agent metric suite operator surface (scope-lean ship). Anchors
// coding_metrics::MetricsCollector + aggregate_session_metrics + the
// SWE-Bench Pro / RefineBench live-run boundaries (RUNPOD-PENDING) as
// production callables (WIRE-DARK avoidance per E-41 precedent). See
// workflow/reasoning/E-49-decisions.md.
mod metrics_cmd;
// E-35: structured-output substrate operator surface (XGrammar /
// INSTRUCTOR / CodeAct / SEARCH/REPLACE / cloud strict-schema).
// WIRE-DARK avoidance per E40_WIRE_DARK_AVOIDANCE_VIA_CLI for the
// 10 new SsbActionType variants this plan introduces. AgentRole::Runner
// DEFERRED per E41_AGENTROLE_FROZEN_AT_17. See
// workflow/reasoning/E-35-decisions.md.
mod structured_output_cmd;
// E-37: `elai safety status|list-pins|verify-pins|fixture-replay` —
// prompt-injection defense operator surface (OWASP LLM Top 10 LLM01).
// Wraps elai_orchestrator::Ssb + elai_safety pin / classifier helpers.
// Read-only; live model fixture replay RUNPOD-PENDING. See
// workflow/reasoning/E-37-decisions.md.
mod safety_cmd;
// F-8 / Slice A4: `elai self-mod apply --proposal-fixture PATH` — SCF
// symbolic-conflict detector W-3 production entrypoint + W-5 live trigger.
// Drives a real ConfigVariant through the REAL
// screen_config_variant_for_conflicts compose unit; emits
// symbolic_conflict_detected + holds pending (WARN-then-escalate). See
// workflow/reasoning/F-8-decisions.md.
mod self_mod_cmd;
// E-40: `elai scip <query|defs|refs|callers|callees|pagerank|status|import>` —
// orchestrator-rooted symbol-graph operator surface. Production caller for
// elai_orchestrator::symbol_graph (defends against E37_WIRE_DARK_GAP_PATTERN).
// See workflow/reasoning/E-40-decisions.md.
mod scip_cmd;
// D-16: `elai conformal-calibration status|update` — durable UniCR
// tau threshold CLI driver. Wraps elai_orchestrator::conformal_calibration.
// Paper-grounded against arXiv:2509.01455 (UniCR Eq.5). Namespace
// distinct from C-47 `elai calibration` per D-16-decisions.md
// DECISION-5.
mod conformal_calibration_cli;
mod experience_cmd;
// C-50: `elai contract scaffold|audit|list` — Contract-Coding
// STRUCTURAL contract test scaffolding CLI driver. Wraps
// elai_orchestrator::contract_coding. Paper-grounded against
// arXiv:2604.13100 (Lin, Zhao, Shi 2026 — kernel K = <N, Sigma,
// Delta> §4.2). Full Generator -> Discriminator RL pipeline
// (Algorithm 1) is RUNPOD-PENDING. See
// workflow/papers/contract-coding-interface.md.
mod contract_cmd;
// C-52: `elai dafny <verify|status|list-tiers>` — Dafny Formal
// Verification Spike CLI driver. Paper-grounded against
// arXiv:2604.22601 (NL2VC). Live `dafny verify` execution is
// RUNPOD-PENDING (local dev host lacks the dafny binary; model
// substitution mandatory — Gemma 4 BLOCKED per memory
// feedback_no_gemma4). See workflow/papers/nl2vc-dafny-interface.md
// + workflow/reasoning/C-52-decisions.md.
mod dafny_cmd;
// E-13: `elai tdd <run|status|mutation-score>` — TDD verifier
// agent CLI surface. Drives orchestrator::tdd_verifier (Red/Green/
// Refactor cycle + anti-mock heuristic + cargo-mutants subprocess
// wrapper). Composes with D-3's
// ConsensusRule::VerifierKindHeterogeneous for aggregation.
mod tdd_cmd;
// E-53: `elai formal-verify <run|status>` — formal-verify pipeline
// CLI surface. Drives orchestrator::formal_verify (which composes
// C-50 contract_coding + C-52 dafny_verify under a CSQ-classifier
// gate). Live Z3 / Kani / Verus dispatch is RUNPOD-PENDING; Dafny
// dispatch reuses C-52 invoke_dafny_cli and surfaces toolchain
// absence as RunpodPending. Gemma-4 BLOCKED per memory
// feedback_no_gemma4 (substrate inheritance from C-52). See
// workflow/reasoning/E-53-decisions.md.
mod formal_verify_cmd;
// E-8: `elai optimize <template|status>` — self-evolving prompts
// driver. Spawns the Python `elai_optimizer` service. Live LLM
// calls RUNPOD-PENDING. Refuses local 7B as reflection LM per
// arXiv:2405.10276 (Zhang et al.). See
// workflow/reasoning/E-8-decisions.md.
mod optimize_cmd;
// C-40: `elai pr <open|list|status|close>` — GitHub PR adapter CLI.
// Shells out to gh via the orchestrator's github_pr::GhCliAdapter.
mod github_pr_cmd;
// H-33 (SCHEMA_VERSION 211): `elai memory <sub>` canonical dispatcher.
// Routes list/inspect/insert/delete/pin/unpin/provenance to the FactStore
// and emits MemoryGovernanceEvent SSB beacons for forget/pin/unpin.
// Activates the stub that was left by s-31708.
mod memory_cmd;
mod pr_review_cmd;
// D-9: `elai audit-locked-keys ...` — LockedKeysAuditor CLI driver.
mod audit_locked_keys;
// H-17 (SCHEMA_VERSION 223): `elai trial-search` — fixture strategy search.
// W-3 Tier-1 production entrypoint: registers >=3 distinct configs through
// TrialLedger::register_trial + emits TrialRegistered SSB rows.
mod trial_search_cmd;
// H-18 (SCHEMA_VERSION 224): `elai deflation-check` — W-3 Tier-1 production
// entrypoint for the Deflated Sharpe / PBO / MinBTL promotion gate (T-2).
// Registers fixture strategy, calls evaluate_deflation, emits
// DeflationGateEvaluated SSB row (off-agent promotion artifact).
mod deflation_check_cmd;
// H-20 (SCHEMA_VERSION 225): `elai leakage-check` — W-1 production entrypoint
// for the leakage static-analysis COMPOSITE gate. Runs evaluate_leakage over a
// target module against the live DKG (fail-closed if absent) and emits a
// LeakageBlocked SSB row (off-agent done-gate artifact).
mod leakage_check_cmd;
// H-21 (SCHEMA_VERSION 226): `elai lookahead-check` — W-1 production entrypoint
// for the runtime look-ahead / causality DIFFERENTIAL gate. Diffs a full-data
// feature column-set against a future-truncated one; any column that changes
// under truncation proves a look-ahead leak. Emits a LookaheadDifferential SSB
// row (off-agent done-gate artifact). The un-gameable runtime backstop to H-20.
mod lookahead_check_cmd;
// H-22 — `elai null-test` is the W-1 production entrypoint for the trading
// metamorphic NULL-TEST gate. Runs the block-permutation null over a returns
// series (p=(C+1)/(K+1) vs alpha/N) and emits a TradingNullTest SSB row
// (off-agent done-gate artifact). A genuine edge collapses to null (Allow); an
// overfit edge survives the null (Block). The f64-time-series complement to H-21.
mod null_test_cmd;
// H-26 — `elai shadow-check` is the W-1 production entrypoint for the shadow-exec
// + financial-invariant metamorphic-relation gate. Diffs two fixture-supplied result
// sets within a LOCKED float band AND checks textbook-correct financial invariants
// (accounting IDENTITIES EXACT in integer cents; option-pricing / numeric MRs within
// the band), emitting a DifferentialDivergence SSB row (off-agent done-gate artifact).
// A consistent run -> InvariantHeld (Allow, exit 0); a violated identity / beyond-band
// divergence -> Block (exit 1). The value-correctness complement to H-25.
mod shadow_check_cmd;
// H-27 — `elai gate-selfcheck` is the W-1/W-3 production entrypoint for the verify-the-
// verifiers META gate. validate_gates drives a FROZEN registry of the 8 Track-3 gates,
// replays each gate's OWN demo-violated fixture as a planted mutant through its REAL
// async evaluate_* fn, and asserts a BLOCK-class verdict (kill the mutant). All gates
// killing their mutants -> AllGatesValid (PASS, exit 0); a surviving mutant / over-block
// / un-runnable gate / empty registry -> a fail-closed Block (exit 1). Emits a
// GateSelfValidation SSB row (off-agent done-gate artifact).
mod gate_selfcheck_cmd;
// H-28 — `elai reward-check` is the W-1/W-3 production entrypoint for the current-RF
// reward-tampering gate (Everitt arXiv:1908.04734). check_reward_tampering consumes a
// fixture triple and detects reward tampering via two sound Tier-1 legs: a frozen-evaluator
// blake3 MISMATCH -> EvaluatorModified HARD-BLOCK (leg 3a, reuses the ArchiveVerifyResult
// tripwire semantics; also the fail-closed arm for a missing baseline), and an implausible
// reward delta with the hash INTACT -> ImplausibleRewardDelta FLAG -> human Layer-3 (leg
// 3b). TD-3: leg 3a is checked FIRST and short-circuits before 3b. Tier-1 = this
// always-compiled CLI (the live darwin admit hook is feature-gated => Tier-2/deferred).
// Emits a RewardTampering SSB row (off-agent done-gate artifact).
mod reward_tamper_cmd;
// H-19 — `elai holdout-check` is the W-1/W-3 production entrypoint for the reusable-holdout
// query-budget gate (the Thresholdout DP gate, Dwork et al. arXiv:1506.02629). query_holdout
// runs the EXACT Thresholdout query over a planted fixture holdout: below-threshold queries
// answer FREE from the training set, above-threshold queries charge 1 budget unit and release
// a Laplace-noised holdout statistic; once the budget is spent the OOS query is REFUSED
// (budget_exhausted, fail-closed). The budget counter is append-only (NO agent reset). Tier-1
// = this always-compiled CLI (the live OOS-eval choke-point is Tier-2/deferred). Emits
// HoldoutBudgetEvent SSB rows (off-agent done-gate artifact).
mod holdout_check_cmd;
mod drift_check_cmd;
// H-31: `elai remember "<text>"` + the `/remember` REPL command -- the user-decision capture
// producer. Activates the wire-dark elai_memory bi-temporal substrate (dispatch_one) into the
// live path; emits UserDecisionSuperseded SSB rows behind ELAI_USER_MEMORY_ENABLE (R9).
mod user_memory_capture;
// H-32: DKG-backed CitationValidator implementation. Concrete impl of the
// elai_memory::CitationValidator trait using the on-disk SymbolGraphCache;
// lives in cli to avoid a memory→knowledge crate edge.
mod dkg_citation_validator;
// D-19: `elai bench prmbench` — PRMBench adversarial-gaming benchmark
// driver. Owns its own tokio runtime (sync entry pattern).
mod bench_prmbench;
// D-21: `elai bench mode-routing` — mode-routing coverage benchmark
// driver. Replays a hand-labeled fixture through D-10's
// `select_grounding_mode` predicate and gates pass on macro F1 + per-mode
// F1. Sibling of bench_prmbench (sync entry pattern).
mod bench_mode_routing;
// D-22: `elai bench conformal` — conformal calibration validation runner.
// Replays a held-out fixture through D-16's `decide` predicate and gates
// pass iff every distribution bucket's empirical refusal rate is inside
// the UniCR (arXiv:2503.01345) band. Sibling of bench_mode_routing.
mod bench_conformal;
// D-20: `elai bench arc` — ARC-AGI-2 OOD probe harness. Gates ELAI's
// mode-routing + novelty-classifier honesty stack against ARC-style
// out-of-distribution claims. Sibling of bench_mode_routing.
mod bench_arc;
// E-17 SP6: `elai bench rag` — retrieval-quality benchmark driver.
// Drives the local QueryPipeline at an explicit QueryTier and emits the
// final-stage retrieval results as JSON. Used by .elai_cc/eval/rag/run.py
// as its observable interface (no clean knowledge-query subcommand
// existed before SP6; /research is web research, not retrieval).
mod bench_rag;
// D-23: `elai bench sycophancy` — sycophancy detection benchmark driver.
// Quantifies the D-14 SycophancyProbe detection_rate + fp_rate against
// the labelled fixture corpus. Sibling of bench_prmbench (sync entry).
mod bench_sycophancy;
// D-24: `elai bench canary-regression` — cross-domain canary regression
// benchmark driver. Replays a labelled fixture corpus (20 Harmful + 50
// Helpful + 30 Neutral) through D-7's run_canary in-memory and emits a
// single CanaryBenchRunCompleted SSB row + a JSON report.
mod bench_canary;
// D-25: `elai bench tamper` — determinism + archive tamper-suite
// benchmark driver. Drives a 10-scenario JSON-manifest matrix
// IN-PROCESS through D-8's verify_archive and emits a single
// DeterminismTamperSuiteCompleted SSB row + a JSON report.
mod bench_tamper;
// F-2b: `elai bench seccodeplt [--json]` — SeCodePLT-structured static-gate
// insecure-rate benchmark. Drives the REAL evaluate_vibegate over the embedded
// CWE corpus OFFLINE, writes a timestamped seccodeplt-<ts>.json report under
// benchmarks/seccodeplt/, and emits one InsecureRateMeasured SSB beacon.
mod bench_seccodeplt;
// I-9 (SCHEMA_VERSION 263): `elai bench error-amplification`. Error-amplification
// (alpha_t) telemetry harness over a frozen pipeline fixture; writes a timestamped
// JSON report under benchmarks/error_amplification/ and emits one
// ErrorAmplificationMeasured SSB beacon.
mod bench_error_amplification;
// H-30 (SCHEMA_VERSION 239): `elai bench staleness [--json] [--broken-retrieval]`
// — the staleness / intent-drift INSTRUMENT. Drives the SHIPPED bi-temporal
// memory substrate (FactStore + audit_conflicts + get_relevant_facts) over a
// labeled supersession fixture in an ISOLATED in-memory store, writes a
// timestamped staleness-<ts>.json report under benchmarks/staleness/, and emits
// one StalenessEvalMeasured SSB beacon. The --broken-retrieval arm is the R2
// negative control. MEASURES, does not gate.
mod bench_staleness;
// G-6 (SCHEMA_VERSION 217): `elai bench safedelta` — gate-ON-vs-gate-OFF
// SAFEDELTA ablation. DERIVES delta from SHIPPED SecCodePltRunner (GD-1).
// Writes benchmarks/safedelta/<ts>.json + a signed .rra manifest (G-2 reuse).
// Emits one SafetyDeltaMeasured SSB beacon. Tier-2 = RUNPOD-PENDING.
mod bench_safedelta;
// PS-3 (SCHEMA_VERSION 277): `elai bench hallucination-delta` — gate-ON-vs-
// gate-OFF hallucination-delta ablation. FAITHFUL MIRROR of G-6 safedelta.
// Writes benchmarks/hallucination_delta/<ts>.json + a signed .rra manifest
// (G-2 reuse). Emits one HallucinationDeltaMeasured SSB beacon.
// Tier-1 = fixture corpus; Tier-2 live CGG = RUNPOD-PENDING.
mod bench_hallucination_delta;
// PS-READINESS-E2ERUNNER Slice B (SCHEMA_VERSION 278): `elai bench
// local-lane-e2e [--endpoint <url>] [--model <id>] [--json]` — the ONLY live
// consumer of the 4B sidecar at http://127.0.0.1:8181. Drives a REAL model,
// emits one LocalLaneE2eMeasured SSB beacon + a signed .rra manifest. A
// missing sidecar is a hard FAIL (R1), never a synthesized pass.
mod bench_local_lane_e2e;
// H-34 (SCHEMA_VERSION 213): `elai bench mcp-schema-budget [--json]` — MCP-schema
// token-budget MEASURE-FIRST benchmark. Drives the embedded multi-server MCP tool
// catalog (20 tools: local-rag / pod-lifecycle / notion) OFFLINE, writes a
// timestamped mcp-schema-budget-<ts>.json report under benchmarks/mcp_schema_budget/,
// and emits one McpSchemaBudgetMeasured SSB beacon. Default-ON; no ELAI_*_ENABLE gate.
mod bench_mcp_schema_budget;
// I-34 (SCHEMA_VERSION 234): `elai bench crane [--crane-fixture PATH] [--seed <u64>]`
// — CRANE hybrid grammar-pin gate deterministic fixture harness. Drives the
// three-layer gate (eligibility + blake3 pin verify + post-parse re-validation)
// WITHOUT live inference (W-10 PENDING). Emits 3 GrammarGateEvent SSB rows.
mod bench_crane;
// I-27 (SCHEMA_VERSION 265): `elai bench convergence --suite <PATH>` —
// verified-outcome-rate vs compute convergence curve fixture harness. Statistical
// floor mechanically enforced (seed_count>=5, paired=true). Emits 1
// ConvergenceCurveMeasured SSB flat beacon per clean run. Live per-node
// compute-spend leg pending I-4 (WIRE-DARK[I-27]).
mod bench_convergence;
// J-34 leg-1: `elai bench context-fill --suite <PATH> [--seeds <u32>] [--json]`
// Verified-outcome-rate vs context-window fill fraction curve fixture harness.
// Statistical floor mechanically enforced (seed_count>=5, paired=true). Builds
// a FillDegradationCurve, constructs a CalibrationArtifact, and emits ONE
// ContextCalibrationEvent(CurveMeasured) SSB row via
// CalibrationArtifactStore::insert_with_ssb (slice-2 seam). Live per-model
// inference leg deferred (WIRE-DARK[J-34.leg1-live]).
mod bench_context_fill;
mod bench_longcot;
mod bench_positional_probe;
// K-18a (PS-5 observe arm): `elai bench case-reuse [--suite fixture] [--seed N]
// [--json]` — recurrence-hit-rate over the shipped BM25 retrieval stack. Drives
// case_reuse_runner over the embedded fixture corpus, writes a blake3-pinned
// JSON report under benchmarks/case_reuse/. Emits 1 RecurrenceHitRateMeasured
// SSB flat beacon + a signed .rra manifest per clean run (SCHEMA_VERSION 270,
// K-18a follow-on; the beacon was walled off from the runner change at K-18a
// per ABORT-4, then landed in the coordinated schema-bump follow-on).
mod bench_case_reuse;
// K-4 (PS-2 owner, KEYSTONE of K-T3): `elai bench gate-tax [--seed N]
// [--json] [--enforce-ceiling]` — composed gate-tax budget. Drives
// gate_tax_runner over the real session journal (best-effort) or a labeled
// fixture composition on a dry journal, writes a JSON report under
// benchmarks/gate_tax/, emits 1 GateTaxMeasured SSB flat beacon + a signed
// .rra manifest per clean run (SCHEMA_VERSION 271). `--enforce-ceiling`
// (MOVE 3/6, SP2) is the actual "release gate" exit-code contract.
// K-2: `elai bench routing-economics` hybrid-vs-force-cloud cost per
// verified task benchmark. Emits RoutingEconomicsMeasured + signed RRA.
mod bench_routing_economics;
// K-9: `elai bench model-refresh` cadence decisions. Emits ModelRefreshCadence
// rows (one per decision) + signed RRA.
mod bench_model_refresh;
mod bench_gate_tax;
// I-38 (SCHEMA_VERSION 235): `elai bench qdelta --config <PATH>` — QDELTA
// defective-diff rate fixture harness. Statistical floor mechanically enforced
// (seed_count>=5, paired=true). Emits 1 QdeltaMeasured SSB flat beacon per
// clean run. Live-model leg deferred (D-026).
mod bench_qdelta;
mod bench_kv_tune;
// C-37: `elai bench snowballing` — Structure Snowballing harness CLI driver.
mod bench_snowballing;
// I-43 (SCHEMA_VERSION 236): `elai bench claim-scope [--suite <SUITE>] [--seed <u64>]`
// Reproducibility-claim scoper + cheap pre-falsification kill-gate. Runs pilot
// (reduced-N gate-ON vs gate-OFF + reviewer-reaching trap detector) + divergence
// scoper (local vs cloud action-stream divergence on security paths). Fixture
// leg only; live-model leg deferred (D-026). Emits 3 ClaimScopeMeasured SSB beacons.
mod bench_claim_scope;
// I-33 (SCHEMA_VERSION 240): `elai bench governance-probe [--seed <u64>]`
// Mechanical decision governance fixture harness (CEFL / I6Q / commit-reveal).
// Three GovernancePrimitiveFired arms without live inference. Wiring-enforcement
// R1 entry point for I-33.
mod bench_governance;
// I-35: `elai bench sequence-probe` -- temporal tool-sequence enforcement bench
// harness. Three ToolSequenceVerdict arms (Allowed / OrderViolation / HaltEnforced)
// without live inference. Wiring-enforcement R1 entry point for I-35.
mod bench_sequence_probe;
mod bench_steering_vs_prompt;
// J-72 slice D: `elai bench retrieval-adapt` — Tier-1 MRR/Recall harness for
// baseline vs adapted slot delta over a gold set. Deterministic. No LLM judge.
mod bench_retrieval_adapt;
// J-75: RRF component ablation report harness.
mod bench_rrf_ablate;
// J-82: deterministic tool-output token emission benchmark.
mod bench_tool_output;
// J-77 I11: `elai bench perception-faithfulness` — 6-metric fixture harness
// (fuse_detections + filter_ghost_regions). Thin fixture-only bench, no SSB.
mod bench_perception_faithfulness;
mod bench_ssb_poll;
// I-32: `elai role-ab-probe` -- role-contract A/B evidence harness. Fixture-driven;
// no live LLM. Emits RoleContractApplied(AbDelta) SSB rows. Handler below.
mod role_ab_probe_cmd;
// G-10 (SCHEMA_VERSION 218): `elai model-drift baseline|check|show` — exogenous
// model-version behavioral-regression gate CLI driver. DISTINCT from `elai drift`
// (E-45 cross-language SOURCE drift). Composes D-7 canary.rs (GD-1).
// Default-ON; kill-switch ELAI_MODEL_DRIFT_DISABLE only (W-2). See
// workflow/reasoning/G-10-decisions.md.
mod model_drift_cmd;
// H-41: `elai checker-gaps` — ranked checker-gap backlog read surface.
// Reads CheckerGapDetected SSB rows; aggregates by claim_type + taxon.
mod checker_gaps_cmd;
// J-28 T8: `elai usage [--json]` — cloud/local token-usage rollup read
// surface. Reads UsageTurnEvent SSB rows; folds via
// elai_orchestrator::usage_ledger::rollup_usage_turns.
mod usage_cmd;
// D-28: `elai farm-adv-rm <run|verdict>` — adversarial example farming
// runner CLI surface. See workflow/reasoning/D-28-decisions.md.
mod farm_adv_rm;
// E-10: hostile-critic structural simplifications -- 3 new CLI dispatchers.
mod bench_ablation_cmd;
mod tier_eval_cmd;
mod critic_report;
// E-6: MAST failure-mode report CLI surface (`elai mast report --session`).
// Reads `audit_mast_tagged` SSB rows read-only; deterministic 14-mode table.
mod mast_cmd;
// B-15 tranche J (orchestrate s-22600): per-arm wrapper layer that funnels
// the eleven bench / eval / critic CliAction dispatch arms through one
// module. main.rs `fn run` calls bench_cmd::run_<x>(...); the underlying
// bench_*.rs / tier_eval_cmd / critic_report `run` fns are left untouched
// so smoke tests (tests/bench_prmbench_smoke.rs etc) still compile.
mod bench_cmd;
// E-47: LoRA ship-gate dispatcher. Reads [finetune.lora] from
// elai_cc.toml + evaluates triple-AND gate + emits SSB row when
// ELAI_LORA_SHIP_GATE_ENABLE=1.
mod lora_gate_check_cmd;
// E-25 Tranche C (C.4): edit_checkpoints CLI surface. `elai
// checkpoint <list|rewind>`. Wire-dark default OFF for SSB emit;
// inspection paths work regardless of the gate.
mod checkpoint_cmd;
// E-7: darwinian-evolve substrate exerciser. See E-7-decisions.md DECISION-13.
// mod is always compiled; the dispatch fn uses #[cfg] arms internally so
// feature-off builds never pull in elai_orchestrator::darwinian_bai.
mod darwin_loop_cmd;
// E-9: external agent-eval benchmark suite (11 suites; SWE-Bench Pro
// primary; RUNPOD-PENDING live runs). See workflow/reasoning/E-9-decisions.md.
mod eval_cmd;
// J-26 S5 (slice S4): `elai eval-corpus-ingest --pack <dir> [--root <corpus-root>]`
// — load a TaskPack's data/public/task.toml (+ optional task.md overlay) and
// call elai_eval_corpus::ingest::ingest_task. Exit 0 Accepted / 1 Rejected/Err.
mod eval_corpus_ingest_cmd;
// E-55: agentic RL + reward ladder recipe runner. Substrate ships fully
// under elai_orchestrator::rl::*; live cloud Sonnet rollouts + LoRA
// training RUNPOD-PENDING. See workflow/reasoning/E-55-decisions.md.
mod rl_train_cmd;
mod runtime_client;
mod runtime_cmd;
// J-30 T5: `elai providers configure [...]` -- BYOK key entry + live
// health_check + key_store::store on success. Mirrors runtime_cmd shape
// (tokio current-thread runtime, 5s health probe timeout, fail-closed
// store on Unavailable). Handler at crates/cli/src/providers_cmd.rs.
mod providers_cmd;
mod providers_byok_ssb;
// D-2: `elai ppm health` — three-step PPM sidecar health probe
// (TCP -> /health -> canary /v1/ppm/score). Sync entrypoint owning a
// tokio current-thread runtime (mirrors archive_verify::run).
mod ppm_health;
mod mucoco_cmd;
mod mcp_client;
mod mcp_server;
// C-42 (session 2): `elai mcp authz [status|list-rejections|fuzz-report]`
// — Semantic Gateway L2 RBAC dashboard surface. Reads the workspace-root
// `authz-policy.toml` and prints status / recent denies / a bounded fuzz
// summary. Sibling of `mcp_client`. Paper grounding: arXiv:2604.25555.
mod mcp_authz_cmd;
// E-24 Task 6: `elai mcp pin [list|show <server>|update <server>]` —
// operator surface over the schema-hash pin store. Reads the on-disk
// `.elai/mcp/pins/*.toml` records (written by the bridge re-pin gate on
// connect) and gates re-approval via `update_pin`. Sibling of
// `mcp_authz_cmd`. Routed from `commands::render_mcp_report_for`.
mod mcp_pin_cmd;
mod tool_executor;
mod tool_formatting;
// C-38: DKG lifecycle helper. Replaces the never-fires "if dkg.db
// exists" gate at live_cli.rs:1722-1732 with an ensure-or-rebuild
// helper that emits paired SSB beacons (DkgRebuildStarted /
// DkgRebuildComplete) around the rebuild path.
mod dkg_lifecycle;
// F-5 / Slice A4: `elai dkg staleness-tick` (W-5 trigger) — single-process
// staleness-quarantine cascade driver. Composes dkg_lifecycle + memory decay +
// the knowledge DKG/cosmos consumers.
mod dkg_staleness_cmd;
// B-15 Tranche O.1 (orchestrate s-22600): five top-level helpers
// (shortcoder_compress_text, cmd_incident_record, build_pr_review_action,
// check_anthropic_gateway, CharEstimatorProvider) carved out of main.rs
// to bound main.rs growth. `pub(crate) use crate::cli_helpers::*;` below
// preserves runtime_client.rs:72 `use crate::*;` glob resolution for
// shortcoder_compress_text (L348+L352 call sites).
pub(crate) mod cli_helpers;
// B-15 Tranche O / T11 COHORT 3: constants + type-alias block extracted from
// main.rs. Consumers via `use crate::*;` continue to resolve through the
// glob re-export alongside cli_helpers below.
pub(crate) mod cli_consts;
// B-15 Tranche O.2 (orchestrate s-22600): bench parser block (8 parsers +
// validate_bool_arg helper, 722 LOC) carved out of args.rs L3358-L4081.
// CliAction enum UNCHANGED; only parser fns relocate. args.rs L2018 calls
// `args_bench::parse_bench_subcommand_args(...)` directly.
pub(crate) mod args_bench;
// Tranche O.3 (T10): five additional parser-family modules carved out of args.rs.
pub(crate) mod args_symbol;
pub(crate) mod args_pr;
pub(crate) mod args_memory;
pub(crate) mod args_critic;
pub(crate) mod args_skills;
// J-72 slice C: `elai retrieval gold-build` parser + handler.
pub(crate) mod args_retrieval;
pub(crate) mod retrieval_cmd;

#[cfg(test)]
mod cli_tests;
#[cfg(test)]
mod sandbox_report_tests;

use std::env;

pub(crate) use crate::args::*;
pub(crate) use crate::auth::*;
pub(crate) use crate::live_cli::*;
pub(crate) use crate::progress::*;
pub(crate) use crate::resume::*;
pub(crate) use crate::runtime_builder::*;
pub(crate) use crate::runtime_client::*;
pub(crate) use crate::tool_executor::*;
pub(crate) use crate::tool_formatting::*;
pub(crate) use crate::help::*;
pub(crate) use crate::commands::{SlashCommand, resume_supported_slash_commands};
// B-15 Tranche O.1: load-bearing re-export. runtime_client.rs:72 does
// `use crate::*;` and calls shortcoder_compress_text at L348+L352; the
// glob re-export below keeps that call site working byte-for-byte after
// the helper moved into crate::cli_helpers. Removing this line breaks
// runtime_client.rs compile (loud failure, not silent).
pub(crate) use crate::cli_helpers::*;
// T11 COHORT 3: re-export all constants + type aliases so every consumer
// that reaches them via `use crate::*;` (args.rs, live_cli.rs, repl.rs, etc.)
// continues to resolve without any import-site changes.
pub(crate) use crate::cli_consts::*;

// shortcoder_compress_text moved to crate::cli_helpers (Tranche O.1).
// Available here via `pub(crate) use crate::cli_helpers::*;` above,
// which also surfaces it through `use crate::*;` to runtime_client.rs.

// CharEstimatorProvider struct + LlmProvider impl moved to
// crate::cli_helpers (Tranche O.1). Re-exported via the glob above.

// All constants (DEFAULT_MODEL, max_tokens_for_model, TOKEN_CHARS_PER_TOKEN,
// BUDGET_OUTPUT_RESERVE, FENCE_LANG_MAP, DEFAULT_DATE,
// DEFAULT_OAUTH_CALLBACK_PORT, VERSION, BUILD_TARGET, GIT_SHA,
// INTERNAL_PROGRESS_HEARTBEAT_INTERVAL, SESSION_HANDOFF_THRESHOLD, PLANS_DIR,
// PRIMARY_SESSION_EXTENSION, OLLAMA_DEFAULT_URL, DEGRADED_PROMPT_PREFIX,
// OLLAMA_PROBE_TIMEOUT, KNOWLEDGE_TOP_K, EMPTY_TIER_TOML,
// LEGACY_SESSION_EXTENSION, LATEST_SESSION_REFERENCE,
// SESSION_REFERENCE_ALIASES, CLI_OPTION_SUGGESTIONS) and type aliases
// (AllowedToolSet, RuntimePluginStateBuildOutput) moved to
// crate::cli_consts (T11 COHORT 3). Re-exported via
// `pub(crate) use crate::cli_consts::*;` above.

fn main() {
    if let Err(error) = run() {
        let message = error.to_string();
        if message.contains("`elai--help`") {
            eprintln!("error: {message}");
        } else {
            eprintln!(
                "error: {message}\n\nRun `elai--help` for usage."
            );
        }
        std::process::exit(1);
    }
}

// check_anthropic_gateway moved to crate::cli_helpers (Tranche O.1).
// Re-exported via the glob above; used at the run() entrypoint below.

fn run() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().skip(1).collect();
    // C-35: emit startup warning when --tool-gate-off is active.
    // The flag is recognized in parse_args (args.rs) and stored as ELAI_TOOL_GATE_OFF=1.
    // Check before parse_args so the warning fires even if the action itself changes.
    if args.iter().any(|a| a == "--tool-gate-off") {
        eprintln!("[WARN] tool gating disabled; prompt costs will be 5-10x higher");
    }
    // E-24: gateway-rejection guard. Warn if ANTHROPIC_BASE_URL points at a
    // known LLM gateway (openrouter, litellm, portkey). Prompt cache breaks
    // through these gateways and latency overhead can be 50ms+.
    if let Ok(base_url) = env::var("ANTHROPIC_BASE_URL") {
        if let Some(gateway) = check_anthropic_gateway(&base_url) {
            tracing::warn!(
                gateway = %gateway,
                base_url = %base_url,
                "anthropic_gateway_detected: ANTHROPIC_BASE_URL points at a known gateway; \
                 prompt cache is broken through gateways and latency overhead applies. \
                 Use api.anthropic.com directly or localhost sidecar."
            );
        }
    }
    // E-47 SECONDARY FIX (WIRING_CONTRACT reachable_from boot-path gap): on
    // every invocation, load [finetune.lora] from .elai_cc/elai_cc.toml.
    // When enabled=true AND confirm_user_understanding=false, print the deferral
    // warning to stderr before any command dispatch. This is separate from the
    // `elai lora-gate-check` subcommand -- it fires on EVERY command so the
    // operator cannot silently run tasks while the gate is misconfigured.
    // The check is best-effort (missing file = skip silently).
    {
        use elai_orchestrator::lora_ship_gate::LORA_SHIP_GATE_DEFERRAL_TOKEN;
        let config_path = std::path::Path::new(".elai_cc/elai_cc.toml");
        if let Ok(text) = std::fs::read_to_string(config_path) {
            if let Ok(root) = toml::from_str::<toml::Value>(&text) {
                let lora = root.get("finetune").and_then(|f| f.get("lora"));
                let enabled = lora
                    .and_then(|l| l.get("enabled"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let confirm = lora
                    .and_then(|l| l.get("confirm_user_understanding"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if enabled && !confirm {
                    eprintln!(
                        "[WARN] DEFERRED_PER_E47_REPORT: [finetune.lora] enabled=true but \
                         confirm_user_understanding=false. LoRA adapter load is blocked. \
                         Read arXiv:2405.09673 (5-19pp catastrophic forgetting), then set \
                         confirm_user_understanding=true in .elai_cc/elai_cc.toml. \
                         Token: {LORA_SHIP_GATE_DEFERRAL_TOKEN}. \
                         Flip runbook: workflow/decisions/E-47-flip-runbook.md."
                    );
                }
            }
        }
    }
    match parse_args(&args).map_err(|e| anyhow::anyhow!(e))? {
        CliAction::DumpManifests { output_format } => auth::dump_manifests(output_format)?,
        CliAction::BootstrapPlan { output_format } => auth::print_bootstrap_plan(output_format)?,
        CliAction::Agents { args, output_format } => LiveCli::print_agents(args.as_deref(), output_format)?,
        CliAction::Mcp { args, output_format } => LiveCli::print_mcp(args.as_deref(), output_format)?,
        CliAction::Runtime { action, role, target, output_format: _ } => {
            // B-37D Phase 4M: text-only output for now; JSON shaping deferred to
            // when RuntimeOrchestrator::health_snapshot lands (Wave-1B).
            LiveCli::print_runtime(action.as_deref(), role.as_deref(), target.as_deref())?;
        }
        CliAction::Providers { backend, key, base_url, model, output_format: _ } => {
            // J-30 T5: text-only output; JSON shaping deferred. The handler
            // owns the tokio current-thread runtime + 5s health probe.
            LiveCli::print_providers(backend, key, base_url, model)?;
        }
        CliAction::ProvidersStatus { backend, output_format } => {
            // J-30 T6: inventory status (configured / verified_at / models /
            // stale). Text table or --json; no network, no GPU.
            LiveCli::print_providers_status(backend, output_format)?;
        }
        CliAction::Skills { args, output_format } => LiveCli::print_skills(args.as_deref(), output_format)?,
        CliAction::PrintSystemPrompt { cwd, date, output_format } => print_system_prompt(cwd, date, output_format)?,
        CliAction::Version { output_format } => print_version(output_format)?,
        CliAction::ResumeSession { session_path, commands, output_format } => resume_session(&session_path, &commands, output_format),
        CliAction::Status { model, permission_mode, output_format } => live_cli::print_status_snapshot(&model, permission_mode, output_format)?,
        CliAction::Sandbox { output_format } => live_cli::print_sandbox_status_snapshot(output_format)?,
        CliAction::Prompt { prompt, model, output_format, allowed_tools, permission_mode, backend } => {
            // C-17 (DRIFT-D/E): auto-resume any persisted indexing batches
            // and install the process-wide SIGINT/SIGTERM observer before
            // dispatching the prompt. The project root is the CWD — the
            // CLI does not accept --project-root on this arm.
            let prompt_project_root = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            c17_lifecycle::install_lifecycle(&prompt_project_root);
            LiveCli::new_prompt(model, true, allowed_tools, permission_mode, backend)?
                .run_turn_with_output(&prompt, output_format)?;
        }
        CliAction::Login { output_format } => run_login(output_format)?,
        CliAction::Logout { output_format } => run_logout(output_format)?,
        CliAction::Doctor { output_format } => doctor::run_doctor(output_format)?,
        // E-17 SP1 task 1 — `elai doctor knowledge-models [...]`.
        CliAction::DoctorKnowledgeModels {
            project_root,
            output_format,
        } => doctor::run_knowledge_models_probe(&project_root, output_format)?,
        // E-12 — `elai production-checklist [--project-root PATH]
        // [--output-format text|json|toml]`.
        CliAction::ProductionChecklist {
            project_root,
            output_format,
        } => production_checklist_cmd::run_production_checklist_cli(&project_root, output_format)?,
        // B-43 Phase 7 — `elai install [--skip-models] [--reset-config]`.
        // Re-entry from the already-installed CLI; spawns scripts/install/install.{sh,ps1}
        // with forwarded flags and reads per-step state files for the report.
        CliAction::Install {
            skip_models,
            reset_config,
            project_root,
            output_format,
        } => install::run_install(
            install::InstallArgs {
                skip_models,
                reset_config,
                project_root,
            },
            output_format,
        )?,
        // C-43 — `elai freeze status|verify|ratify`. Phase 4H scope-freeze
        // CLI surface. ELAI design hypothesis (see
        // workflow/sessions/s-25120/mistakes.md for the bad-cite
        // analysis of arXiv:2604.21725).
        CliAction::Freeze { action, output_format } => {
            freeze_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // G-2 — `elai rra show <path>`. Reproducible-Run Artifact viewer.
        CliAction::Rra { action, output_format } => {
            rra_cmd::run_rra(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // I-30 Slice 1b — `elai dataset export-sft --rra-dir <p> --out-dir <p>`.
        // SFT exhaust from sealed RRA bundles.
        CliAction::Dataset { action, output_format } => {
            dataset_cmd::run_dataset(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // J-2 — local Tier-1 read-only MCP control-plane server over stdio.
        CliAction::McpServe { output_format } => {
            mcp_server::run_mcp_serve(output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // G-2 — `elai replay --verify <path>`. Offline .rra verifier; exits
        // non-zero on Divergence / Tampered so CI can gate on it.
        CliAction::Replay { action, output_format } => {
            rra_cmd::run_replay(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // I-40 — `elai wrap [--output <path>] [--fixture] -- <agent-cmd...>`.
        // Trust-layer wrapper proxy: taint + write-gate trust substrate around an
        // external agent command. Emits a .rra bundle (G-2 build_rra + G-9
        // redaction). FORK-1: new top-level CliAction::Wrap with its own Arc<Ssb>.
        CliAction::Wrap { agent_cmd, output_path, output_format } => {
            wrap_cmd::run(agent_cmd, output_path, output_format)
                .map_err(|e| anyhow::anyhow!(e))?
        }
        // K-6 — `elai export compliance [--session <id>] [--out <path>] [--zero-retention]`.
        // Redacted RRA + attestation + audit trail + retention statement bundle.
        CliAction::Compliance {
            session_id,
            output_path,
            zero_retention,
            output_format,
        } => {
            compliance_export_cmd::run(
                session_id,
                output_path,
                elai_orchestrator::retention::ExportRetentionMode::from_cli_flag(zero_retention),
                output_format,
            )
            .map_err(|e| anyhow::anyhow!(e))?
        }
        // H-35 — `elai rewind <target>`. Session-state restore to a prior SSB seq.
        // Emits one session_rewound beacon; idempotent on target==head (zero rows).
        CliAction::Rewind { action, output_format } => {
            rewind_cmd::run_rewind(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // H-49 — `elai undo / redo / history / jump`. CAS-backed reversible LLM
        // edits over the undo-tree HEAD. Default-ON; kill-switch ELAI_TIMETRAVEL_DISABLE.
        // Composes E-25 edit_checkpoints (GD-1) + SessionDag HEAD + H-35 SessionRewound.
        CliAction::TimeTravel { action, output_format } => {
            timetravel_cmd::run_timetravel(action, output_format)
                .map_err(|e| anyhow::anyhow!(e))?
        }
        // G-5 — `elai review --cockpit`. Reviewer-cockpit DSR producer: parses
        // the working-tree git diff against a pinned base, reads the session SSB
        // vouch stream, computes the DSR, prints the bands, and emits ONE
        // review_surface_scored row through the live session Arc<Ssb>.
        CliAction::Review { action, output_format } => {
            review_cmd::run_review(action, output_format)?
        }
        // H-53 — `elai prebind [--demo-prebind] [--allow-freeform-citations]`.
        // Citation pre-binding + constrained synthesis on the /research|/ask
        // answer path. W-1 seeded-fixture trigger builds deterministic evidence,
        // calls freeze_anchors, runs closed-set check (Bound/Ghost/Stale), and
        // emits a genuine CitationAnchorsBound SSB row (off-agent artifact, R4).
        // --allow-freeform-citations fires the Blocked arm (R2 broken-arm proof).
        CliAction::Prebind { allow_freeform, output_format } => {
            prebind_cmd::run_demo_prebind(allow_freeform, output_format)?
        }
        // G-8 — `elai verify-attestation <commit>`. Offline in-toto provenance
        // attestation verifier: reads the digest-only ELAI-Provenance-Digest
        // commit trailer + the sidecar attestation.intoto.json, recomputes the
        // blake3-Merkle root, and confirms it equals both the subject digest and
        // the trailer (F-gate hashes Merkle-bound). READ-ONLY (no Ssb); exits
        // non-zero on not-verified; trust-without-re-running (SLSA-VSA).
        CliAction::VerifyAttestation { commit, output_format, chain, current_objective } => {
            verify_attestation_cmd::run(commit, output_format, chain, current_objective)
                .map_err(|e| anyhow::anyhow!(e))?
        }
        // C-45 — `elai credit attribute|table|export`. CANTANTE
        // contrastive credit-attribution surface; paper-grounded against
        // arXiv:2605.13295. Synthetic-fixture path is GPU-free; live
        // pilot is `RUNPOD-PENDING: cantante_pilot_450runs_qwen3_8b`.
        CliAction::Credit { action, output_format } => {
            credit_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-43 -- `elai skills-mgmt list|eval|publish`. Agent skill systems
        // surface (frontmatter + eval/ + publish dist). ABSORBs the
        // E-22 SKILL.md export shim. Paper-grounded against
        // agentskills.io/specification.
        CliAction::SkillsManagement { action, output_format } => {
            skills_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-39 -- `elai cloud override-cap|status`. Layered-cap
        // manual-override CLI surface. Composes with E-38 EU AI Act
        // audit-trail liveness gate. See
        // workflow/reasoning/E-39-decisions.md.
        CliAction::Cloud { action, output_format } => {
            cloud_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // C-46 — `elai routing decisions|pilot|status|export`.
        // Uno-Orchestra parsimonious agent routing surface; paper-
        // grounded against arXiv:2605.05007. Synthetic-fixture path is
        // GPU-free; live pilot is
        // `RUNPOD-PENDING: uno_orchestra_pilot_8roles_qwen3_8b`.
        CliAction::Routing { action, output_format } => {
            routing_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // C-47 — `elai calibration status|reset-hints|export`. Case-based
        // tool-use calibration; paper-grounded against arXiv:2605.15041
        // (CAST). The CLI surface is synthetic + GPU-free; live registry
        // IPC is `RUNPOD-PENDING`.
        CliAction::Calibration { action, output_format } => {
            calibration_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-14 — `elai browser run|list-journeys|status`. Browser-e2e
        // agent surface; wraps elai_orchestrator::browser_e2e. Wire-dark
        // posture for absent npx/playwright; live VLM-tier RUNPOD-PENDING.
        CliAction::Browser { action, output_format } => {
            browser_e2e_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-18 — `elai reliability status|list-spills|replay-spill`.
        // Tool-use reliability gate surface; handler at
        // crates/cli/src/reliability_cmd.rs. Composes on D-11 +
        // E-35 + C-47 + C-42 + C-35 (reuse-heavy posture). See
        // workflow/reasoning/E-18-decisions.md.
        CliAction::Durability { action, output_format } => {
            durability_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-45 — `elai drift detect|list-pairs|baseline`. Cross-language
        // drift detector operator surface; handler at
        // crates/cli/src/drift_cmd.rs. WIRE-DARK avoidance for
        // cross_lang_drift::DriftDetector + PairRegistry per E-41
        // precedent. See workflow/reasoning/E-45-decisions.md.
        CliAction::Drift { action, output_format } => {
            drift_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-49 — `elai metrics capture|swe-bench-pro|refinebench|aggregate|status`.
        // Coding-agent metric suite operator surface (scope-lean ship);
        // handler at crates/cli/src/metrics_cmd.rs. WIRE-DARK avoidance
        // for coding_metrics::MetricsCollector + aggregate_session_metrics
        // per E-41 precedent. Live SWE-Bench Pro + RefineBench transports
        // are RUNPOD-PENDING. See workflow/reasoning/E-49-decisions.md.
        CliAction::Metrics { action, output_format } => {
            metrics_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-35 — `elai structured-output status|codeact-run|strict-mode|
        // grammar emit-test|search-replace check|reject-engine`.
        // Anchors host-side structured-output substrate so the 10 new
        // E-35 SSB beacon variants are not WIRE-DARK. Python sidecar
        // wiring is RUNPOD-PENDING; AgentRole::Runner DEFERRED per
        // E41_AGENTROLE_FROZEN_AT_17. See
        // workflow/reasoning/E-35-decisions.md.
        CliAction::StructuredOutput { action, output_format } => {
            structured_output_cmd::run(action, output_format)
                .map_err(|e| anyhow::anyhow!(e))?
        }
        CliAction::Reliability { action, output_format } => {
            reliability_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-37 — `elai safety status|list-pins|verify-pins|fixture-replay`.
        // Prompt-injection defense operator surface (OWASP LLM01); handler
        // at crates/cli/src/safety_cmd.rs. Layers additively atop E-24
        // mcp_schema_pin + sanitize_result + llm_guard and E-30
        // destructive_verb_allowlist. See
        // workflow/reasoning/E-37-decisions.md.
        CliAction::Safety { action, output_format } => {
            safety_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // F-8 / Slice A4 — `elai self-mod apply --proposal-fixture PATH`.
        // SCF symbolic-conflict detector W-3 production entrypoint + W-5 live
        // trigger; handler at crates/cli/src/self_mod_cmd.rs. Drives a real
        // ConfigVariant through the REAL screen_config_variant_for_conflicts
        // compose unit (emits symbolic_conflict_detected + holds pending,
        // WARN-then-escalate). See workflow/reasoning/F-8-decisions.md.
        CliAction::SelfMod { action, output_format } => {
            self_mod_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-40 — `elai scip query|defs|refs|callers|callees|pagerank|status|import`.
        // Orchestrator-rooted symbol-graph operator surface; handler at
        // crates/cli/src/scip_cmd.rs. Production caller for the new
        // elai_orchestrator::symbol_graph module — defends against
        // E37_WIRE_DARK_GAP_PATTERN. See workflow/reasoning/E-40-decisions.md.
        CliAction::SymbolGraph { action, output_format } => {
            scip_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // D-16 — `elai conformal-calibration status|update`. Durable
        // UniCR tau persistence; handler at
        // crates/cli/src/conformal_calibration_cli.rs. Paper-grounded
        // against arXiv:2509.01455 (UniCR Eq.5). Namespace distinct
        // from C-47's `elai calibration` per D-16-decisions.md
        // DECISION-5.
        CliAction::ConformalCalibration { action, output_format } => {
            conformal_calibration_cli::run(action, output_format)
                .map_err(|e| anyhow::anyhow!(e))?
        }
        // C-49 — `elai experience distill|list|export|reset`. ELAI
        // heuristic adaptation of arXiv:2605.14443 (Sayana, Todi, Jash
        // 2026). Synthetic-fixture path is GPU-free; live trace
        // recorder is `RUNPOD-PENDING:
        // iterative_distillation_prompter_rl_qwen3_8b`.
        CliAction::Experience { action, output_format } => {
            experience_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // C-50 — `elai contract scaffold|audit|list`. Contract-Coding
        // STRUCTURAL contract test scaffolding surface; paper-
        // grounded against arXiv:2604.13100. Synthetic-fixture path
        // is GPU-free; the live Generator -> Discriminator RL
        // pipeline (Algorithm 1) is `RUNPOD-PENDING:
        // contract_coding_gen_disc_rl_qwen3_8b`. Citation
        // discipline: the "47%" is the Roguelike subset of
        // Greenfield-5 (Table 1) and TIES Trae; the differentiator
        // is structural integrity per Section 5.2.
        CliAction::Contract { action, output_format } => {
            contract_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // C-52 — `elai dafny verify|status|list-tiers`. Dafny Formal
        // Verification Spike CLI surface; paper-grounded against
        // arXiv:2604.22601 (NL2VC). Live `dafny verify` execution
        // is `RUNPOD-PENDING: live_dafny_verification_with_non_gemma_substitution`.
        // Model substitution mandatory (Gemma 4 BLOCKED per memory
        // feedback_no_gemma4).
        CliAction::Dafny { action, output_format } => {
            dafny_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-13: `elai tdd run|status|mutation-score`. TDD verifier
        // agent dispatch — composes with orchestrator::tdd_verifier
        // and D-3's ConsensusRule::VerifierKindHeterogeneous. Live
        // cargo-mutants execution is RUNPOD-PENDING when the binary
        // is absent (wire-dark posture).
        CliAction::Tdd { action, output_format } => {
            tdd_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-53 — `elai formal-verify run|status`. Formal-verify
        // pipeline dispatch — composes orchestrator::formal_verify
        // (C-50 contract_coding + C-52 dafny_verify under CSQ gate).
        // Live Z3 / Kani / Verus dispatch is RUNPOD-PENDING; Dafny
        // dispatch reuses C-52 invoke_dafny_cli. Gemma-4 BLOCKED per
        // memory feedback_no_gemma4 (substrate inheritance from C-52).
        CliAction::FormalVerify { action, output_format } => {
            formal_verify_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-8 — `elai optimize template|status`. Self-evolving prompts
        // driver. Spawns the Python `elai_optimizer` service (GEPA
        // primary per arXiv:2507.19457 + DSPy MIPROv2 / OPRO
        // supporting). Live LLM calls RUNPOD-PENDING. Refuses local 7B
        // as reflection LM per arXiv:2405.10276 (Zhang et al.) -- the
        // args parser enforces the refusal before this handler runs.
        // See workflow/reasoning/E-8-decisions.md.
        CliAction::Optimize { action, output_format } => {
            optimize_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        CliAction::Init { output_format } => live_cli::run_init(output_format)?,
        CliAction::Repl { model, allowed_tools, permission_mode, export_jsonl } => {
            // C-17 (DRIFT-D/E): auto-resume + signal observer for the
            // long-lived REPL scope. Sync entrypoint — see c17_lifecycle.rs
            // for the transient-runtime pattern.
            let repl_project_root = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            c17_lifecycle::install_lifecycle(&repl_project_root);
            repl::run_repl(model, allowed_tools, permission_mode, export_jsonl)?;
        }
        CliAction::DepAudit { project_dir, output_format } => doctor::run_dep_audit_command(&project_dir, output_format)?,
        // F-9 / Slice D4: `elai dep-check <path> [--offline]` — write-time
        // anti-slopsquatting dep-check. DEFAULT uses the real live
        // SubprocessRegistryChecker; --offline uses StubRegistryChecker. The
        // handler opens its OWN session ssb.db (via a current-thread runtime +
        // block_on) and writes WriteTimeSlopsquatBlocked rows durably (BLOCK-3
        // observable_effect).
        CliAction::DepCheck { path, offline } => dep_check_cmd::run(path, offline)?,
        CliAction::Local { port } => repl::run_local_repl(port)?,
        CliAction::Dashboard => dashboard::run_dashboard()?,
        // Phase A1a: production caller of `start_task_runner`. Pinned by
        // `crates/contracts/tests/task_runner_chokepoint_contract.rs`.
        CliAction::Run { project_root, mode, provider, via_mcp, empire, export_ssb, emit_rra, profile, zero_retention } => {
            // C-17 (DRIFT-D/E): auto-resume + process-wide signal observer
            // for the long-lived `elai run` task_runner scope. Runs BEFORE
            // run_subcommand builds its own tokio runtime so the resume
            // pass uses a transient runtime that finishes promptly.
            // C-40 session-2 minimal repair: surface the empire + export_ssb
            // fields added by args.rs (session 3 is editing main.rs
            // concurrently and owns the canonical wiring; this is the
            // smallest valid destructure so the workspace compiles).
            c17_lifecycle::install_lifecycle(&project_root);
            run_subcommand::run_task_runner_subcommand(run_subcommand::RunArgs {
                project_root,
                mode,
                provider,
                via_mcp,
                empire,
                export_ssb,
                emit_rra,
                profile,
                zero_retention,
            })?
        }
        // E-49 SWE-Bench Pro v5 driver — non-interactive task surface.
        // Subprocess-wrap: composes B-37 per-role overrides + the
        // production task_runner inline (no daemon), emits JSONL summary.
        CliAction::Task {
            problem_statement,
            repo_root,
            mode,
            output_format,
            max_wall_seconds,
            executor_runtime,
            auditor_runtime,
        } => {
            c17_lifecycle::install_lifecycle(&repo_root);
            task_cmd::run_task_subcommand(task_cmd::TaskArgs {
                problem_statement,
                repo_root,
                mode,
                output_format,
                max_wall_seconds,
                executor_runtime,
                auditor_runtime,
            })?
        }
        CliAction::RemoteSidecar => remote_sidecar::run_remote_sidecar()?,
        CliAction::HelpTopic(topic) => live_cli::print_help_topic(topic),
        CliAction::Help { output_format } => help::print_help(output_format)?,
        // F-5 / Slice A4: `elai dkg staleness-tick --workspace <path>` (W-5).
        CliAction::DkgStalenessTick { workspace } => {
            dkg_staleness_cmd::run_dkg_staleness_tick(&workspace)?
        }
        // B-29 Phase 4: symbol graph CLI subcommands.
        CliAction::SymbolGraphBuild { project_root } => {
            symbol_graph::run_symbol_graph_build(project_root)?
        }
        CliAction::SymbolGraphStats { project_root } => {
            symbol_graph::run_symbol_graph_stats(project_root)?
        }
        // E-21 Tier-A: `elai symbol_graph rank --top N` — top-N pagerank
        // symbols from the existing dkg_nodes_pagerank table (C-3 ship).
        CliAction::SymbolGraphRank { project_root, top } => {
            symbol_graph::run_symbol_graph_rank(project_root, top)?
        }
        // B-29 Phase 5b: fork-exec shell wrappers around
        // SymbolGraphCache + B-34 entry-point detection. Lets external
        // scripts (Python eval, agents) call compute_impact_radius /
        // compute_symbol_context / detect_entry_points with JSON output
        // without going through the MCP/IPC bridge.
        CliAction::SymbolImpact { project_root, target, depth, output_format } => {
            symbol_graph::run_symbol_impact(project_root, target, depth, output_format)?
        }
        CliAction::ConeTokenFloor { project_root, target, depth, output_format } => {
            symbol_graph::run_cone_token_floor(project_root, target, depth, output_format)?
        }
        CliAction::SymbolContext { project_root, name, file, output_format } => {
            symbol_graph::run_symbol_context(project_root, name, file, output_format)?
        }
        CliAction::EntryPoints { project_root, limit, output_format } => {
            symbol_graph::run_entry_points(project_root, limit, output_format)?
        }
        // C-13: `elai wip-squash [--project-root <path>]` — squashes leading
        // WIP commits when count > 5. Invoked by the Python FSM hook chain
        // at AUDITING -> MEMORIZING transitions.
        CliAction::WipSquash { project_root } => {
            wip_squash_subcommand::run_wip_squash_subcommand(project_root)?
        }
        // C-20: `elai measure [--scope <name>] [--cloud] [--output toml|json]`
        // — test-infra metrics backfill driver. Owns its own tokio runtime
        // (sync entrypoint pattern mirrored from `run_subcommand.rs`).
        CliAction::Measure { scope, cloud, output_format: _ } => {
            let project_root = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            measure::run_measure_subcommand(scope, cloud, project_root)?
        }
        // C-25: `elai pipeline [list|reset]` — inspect or reset the
        // compiled-pipeline cache at `.elai/cache/compiled_pipelines.msgpack`.
        CliAction::Pipeline { action, output_format: _ } => {
            let project_root = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            pipeline_subcommand::run_pipeline_subcommand(action.as_deref(), &project_root)?
        }
        // F-7 / Slice A4: `elai template-ranking list [--json]` — rank
        // compiled-pipeline templates by Laplace-smoothed outcome utility
        // (HALF B CBR ranking). Reads the persisted CompiledPipelineStore +
        // emits a template_utility_ranked beacon per template (default-ON).
        CliAction::TemplateRanking { action, json } => {
            let project_root = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            template_ranking_cmd::run_template_ranking_subcommand(
                action.as_deref(),
                json,
                &project_root,
            )?
        }
        // C-27: `elai wiring audit ...` — deterministic Tier T-0 wiring
        // contract audit. Owns its own tokio runtime (sync entry, mirrors
        // pipeline/measure subcommands). Exits 1 on any violation.
        CliAction::WiringAudit {
            scope,
            config_path,
            contracts_dir,
            output_format,
        } => {
            wiring_audit::run(scope, config_path, contracts_dir, output_format)?;
        }
        // WIRING-FIX-ELAI Phase 2 (SP3): `elai wiring-coverage ...` — Layer-3
        // entrypoint-coverage gate. Sync entry (mirrors wiring audit). Exits 1
        // on any violation OR un-measurable evidence (R2 fail-closed). This
        // arm is mandatory: the CliAction match is exhaustive, so a missing
        // arm is a COMPILE error (the compile-time wiring proof — producer
        // CliAction::WiringCoverage has a real consumer here, R6).
        CliAction::WiringCoverage {
            manifest_path,
            diff_base,
            output_format,
        } => {
            wiring_coverage_cmd::run(manifest_path, diff_base, output_format)?;
        }
        // D-8: `elai archive verify [--path PATH] [--json] [--fail-fast]`
        // Walks the JSONL ArchiveChain via D-1's verify primitives and
        // exits 1 on any tamper / broken-parent / IO / malformed-JSON.
        CliAction::ArchiveVerify {
            path,
            json,
            fail_fast,
        } => {
            archive_verify::run(path, json, fail_fast)?;
        }
        // D-1: `elai template migrate [--source DIR] [--path PATH] [--json]`
        // Bootstraps the append-only reasoning-template archive (JSONL) from
        // the built-in canonical seed set + optional source dir, emitting one
        // ArchiveEntryAdded SSB row per appended Frozen template (off-agent
        // observable effect). Idempotent on re-run.
        CliAction::TemplateMigrate { source, path, json } => {
            template_migrate::run(source, path, json)?;
        }
        // D-9: `elai audit-locked-keys [--registry PATH] [--staged | --files A,B]
        // [--json] [--fail-fast]` — LockedKeysAuditor driver. Exits 1 on
        // any registry overlap; 0 on clean / empty registry.
        CliAction::AuditLockedKeys {
            registry_path,
            files,
            use_staged,
            json,
            fail_fast,
        } => {
            audit_locked_keys::run(registry_path, files, use_staged, json, fail_fast)?;
        }
        // H-17 (SCHEMA_VERSION 223): `elai trial-search` — fixture strategy
        // search. Registers >=3 distinct configs through TrialLedger::register_trial,
        // emitting >=3 TrialRegistered SSB rows. W-3 Tier-1 production entrypoint
        // proof for the selection-bias keystone. Handler: trial_search_cmd::run.
        CliAction::TrialSearch { objective, window, json } => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .build()
                .map_err(|e| anyhow::anyhow!("tokio runtime: {e}"))?;
            rt.block_on(trial_search_cmd::run(objective, window, json))
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        // H-18 (SCHEMA_VERSION 224): `elai deflation-check` — W-3 Tier-1
        // production entrypoint for the Deflated Sharpe / PBO / MinBTL gate.
        // Registers a fixture strategy search, calls evaluate_deflation, and
        // emits a DeflationGateEvaluated SSB row (off-agent promotion artifact).
        CliAction::DeflationCheck { sr, t_periods, n_fixture, sharpe_std, pbo, seed, json } => {
            // Parse String->f64 at dispatch time (CliAction stores f64 as String
            // because CliAction derives Eq and f64 does not implement Eq).
            let sr_f64 = sr.as_deref().map(|s| s.parse::<f64>()).transpose()
                .map_err(|e| anyhow::anyhow!("--sr parse error: {e}"))?;
            let sharpe_std_f64 = sharpe_std.as_deref().map(|s| s.parse::<f64>()).transpose()
                .map_err(|e| anyhow::anyhow!("--sharpe-std parse error: {e}"))?;
            let pbo_f64 = pbo.as_deref().map(|s| s.parse::<f64>()).transpose()
                .map_err(|e| anyhow::anyhow!("--pbo parse error: {e}"))?;
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .build()
                .map_err(|e| anyhow::anyhow!("tokio runtime: {e}"))?;
            rt.block_on(deflation_check_cmd::run(
                sr_f64, t_periods, n_fixture, sharpe_std_f64, pbo_f64, seed, json,
            ))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        // H-20 (SCHEMA_VERSION 225): `elai leakage-check` — W-1 production
        // entrypoint for the leakage static-analysis COMPOSITE gate. Runs
        // evaluate_leakage over a target module against the live DKG
        // (fail-closed if absent/empty) and emits a LeakageBlocked SSB row.
        CliAction::LeakageCheck { module, no_split, demo_fit_both, residue_temporal, seed, json } => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .build()
                .map_err(|e| anyhow::anyhow!("tokio runtime: {e}"))?;
            rt.block_on(leakage_check_cmd::run(
                module, no_split, demo_fit_both, residue_temporal, seed, json,
            ))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        // H-21 (SCHEMA_VERSION 226): `elai lookahead-check` — W-1 production
        // entrypoint for the runtime look-ahead / causality DIFFERENTIAL gate.
        // Diffs full vs future-truncated feature column-sets and emits a
        // LookaheadDifferential SSB row (off-agent done-gate artifact).
        CliAction::LookaheadCheck { demo_future_leak, demo_causal, seed, json } => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .build()
                .map_err(|e| anyhow::anyhow!("tokio runtime: {e}"))?;
            rt.block_on(lookahead_check_cmd::run(
                demo_future_leak, demo_causal, seed, json,
            ))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        // H-22 — `elai null-test` is the W-1 production entrypoint for the
        // trading metamorphic NULL-TEST gate. Runs the block-permutation null
        // and emits a TradingNullTest SSB row (off-agent done-gate artifact).
        CliAction::NullTest { demo_overfit, demo_genuine, seed, json } => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .build()
                .map_err(|e| anyhow::anyhow!("tokio runtime: {e}"))?;
            rt.block_on(null_test_cmd::run(
                demo_overfit, demo_genuine, seed, json,
            ))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        // H-26 — `elai shadow-check` is the W-1 production entrypoint for the
        // shadow-exec + financial-invariant metamorphic-relation gate. Diffs two
        // fixture-supplied result sets within a LOCKED float band + checks the
        // textbook-correct financial invariants, emitting a DifferentialDivergence
        // SSB row. Exits non-zero on Block so CI gates.
        CliAction::ShadowCheck { demo_invariant_held, demo_invariant_violated, seed, json } => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .build()
                .map_err(|e| anyhow::anyhow!("tokio runtime: {e}"))?;
            rt.block_on(shadow_check_cmd::run(
                demo_invariant_held, demo_invariant_violated, seed, json,
            ))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        // H-27 — `elai gate-selfcheck` is the W-1/W-3 production entrypoint for the
        // verify-the-verifiers META gate. Drives the FROZEN registry of the 8 Track-3
        // gates and emits a GateSelfValidation SSB row. Exits non-zero on a fail-closed
        // Block (surviving mutant / over-block / un-runnable gate / empty registry).
        CliAction::GateSelfcheck { demo_all_valid, demo_gate_broken, seed, json } => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .build()
                .map_err(|e| anyhow::anyhow!("tokio runtime: {e}"))?;
            rt.block_on(gate_selfcheck_cmd::run(
                demo_all_valid, demo_gate_broken, seed, json,
            ))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        // H-28 — `elai reward-check` is the W-1/W-3 production entrypoint for the
        // current-RF reward-tampering gate. Drives check_reward_tampering over a planted
        // fixture triple and emits a RewardTampering SSB row. Exits non-zero on any
        // non-PASS verdict (EvaluatorModified HARD-BLOCK / ImplausibleRewardDelta FLAG).
        CliAction::RewardCheck { demo_clean, demo_tampered, demo_score_jump, seed, json } => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .build()
                .map_err(|e| anyhow::anyhow!("tokio runtime: {e}"))?;
            rt.block_on(reward_tamper_cmd::run(
                demo_clean, demo_tampered, demo_score_jump, seed, json,
            ))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        // H-19 — `elai holdout-check` is the W-1/W-3 production entrypoint for the
        // reusable-holdout query-budget gate (Thresholdout DP). Provisions a fixture holdout,
        // drives query_holdout, and emits HoldoutBudgetEvent SSB rows. Exits non-zero when the
        // final query is REFUSED (Exhausted, budget_exhausted).
        CliAction::HoldoutCheck { demo_spend, demo_exhausted, seed, json } => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .build()
                .map_err(|e| anyhow::anyhow!("tokio runtime: {e}"))?;
            rt.block_on(holdout_check_cmd::run(demo_spend, demo_exhausted, seed, json))
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        // H-29 — `elai drift-check` is the W-1/W-3 production entrypoint for the LOCKED
        // fail-closed drift state machine (T-13, the master deployment-trust interlock).
        // Plants a clean or drifted fixture deployment, drives evaluate_drift_interlock, and
        // emits DriftInterlock SSB rows. Exits non-zero when the interlock latches DriftHalted.
        CliAction::DriftCheck { demo_stable, demo_drift, seed, json } => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .build()
                .map_err(|e| anyhow::anyhow!("tokio runtime: {e}"))?;
            rt.block_on(drift_check_cmd::run(demo_stable, demo_drift, seed, json))
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        // H-31 — `elai remember "<text>"` captures a durable WORK preference into the bi-temporal
        // user-memory ledger (activating the wire-dark elai_memory substrate via dispatch_one) and
        // emits a UserDecisionSuperseded SSB row. Gated behind ELAI_USER_MEMORY_ENABLE (R9).
        CliAction::Remember { text, json } => {
            let seat_id = std::env::var("ELAI_SEAT_IDENTITY")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| user_memory_capture::DEFAULT_SEAT_ID.to_string());
            let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
                std::sync::Arc::new(elai_core::clock::OsClock);
            let outcome = user_memory_capture::capture_remember_with_clock(&text, &seat_id, &clock)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            user_memory_capture::print_capture_outcome(&outcome, json);
        }
        // I-39 — `elai plan status [--letter <L>] [--json]`. Per-plan state derived
        // from FINISH fragments + wiring_contract_check.py + SSB journal (SsbViewer).
        // Emits one PlanStateDerived SSB row (fail-soft if no ssb.db). Default-ON.
        CliAction::PlanStatus { action, output_format } => {
            plan_status_cmd::run_plan_status(action, output_format)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        // I-41 — `elai critique <spec-path>`. Advisory-only spec-time critique lenses.
        // ELAI_CRITIQUE_LENSES_ENABLE=1 activates live LLM runs. Default-OFF (R9).
        CliAction::Critique { action, output_format } => {
            critique_cmd::run_critique(action, output_format)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        // B-15 tranche J (orchestrate s-22600): the eleven bench / eval /
        // critic dispatch arms (BenchPrmbench D-19, BenchModeRouting D-21,
        // BenchConformal D-22, BenchArc D-20, BenchRag E-17 SP6,
        // BenchSycophancy D-23, BenchCanary D-24, BenchTamper D-25,
        // BenchAblation + TierEval + CriticReport E-10 trio) funnel
        // through `bench_cmd::run_*` wrappers. Doc-comments + dispatcher
        // contracts live in bench_cmd.rs alongside each wrapper.
        // EXCLUDED from this funnel: FarmAdvRm (D-28 sibling boundary)
        // and DarwinLoop (#[cfg(feature = "darwinian-evolve")] both arms);
        // both stay inline below.
        CliAction::BenchPrmbench { dataset, hook, output, gating_threshold, category } => {
            bench_cmd::run_bench_prmbench(dataset, hook, output, gating_threshold, category)?;
        }
        CliAction::BenchModeRouting { dataset, output, macro_floor, per_mode_floor } => {
            bench_cmd::run_bench_mode_routing(dataset, output, macro_floor, per_mode_floor)?;
        }
        CliAction::BenchConformal { dataset, corpus, output, epsilon } => {
            bench_cmd::run_bench_conformal(dataset, corpus, output, epsilon)?;
        }
        CliAction::BenchArc { dataset, output, novelty_config } => {
            bench_cmd::run_bench_arc(dataset, output, novelty_config)?;
        }
        CliAction::BenchRag { tier, top_k, query, json, project_root } => {
            bench_cmd::run_bench_rag(tier, top_k, query, json, project_root)?;
        }
        CliAction::IndexBuild { project_root, vectors, json } => {
            crate::index_cmd::run_index_build(project_root, vectors, json)?;
        }
        CliAction::BenchSycophancy { fixtures, output, detection_floor, fp_ceiling } => {
            bench_cmd::run_bench_sycophancy(fixtures, output, detection_floor, fp_ceiling)?;
        }
        CliAction::BenchCanary {
            fixtures,
            output,
            harmful_block_floor,
            helpful_block_ceiling,
            neutral_block_ceiling,
        } => {
            bench_cmd::run_bench_canary(
                fixtures,
                output,
                harmful_block_floor,
                helpful_block_ceiling,
                neutral_block_ceiling,
            )?;
        }
        CliAction::BenchTamper { scenarios, output, fail_fast, max_wall_ms_per_scenario } => {
            bench_cmd::run_bench_tamper(scenarios, output, fail_fast, max_wall_ms_per_scenario)?;
        }
        // H-34 (SCHEMA_VERSION 213): MCP-schema token-budget MEASURE-FIRST
        // benchmark. Drives the embedded multi-server MCP tool catalog OFFLINE,
        // writes a timestamped report, emits one McpSchemaBudgetMeasured SSB
        // beacon. Default-ON; no ELAI_*_ENABLE gate. Exits 0/1.
        CliAction::BenchMcpSchemaBudget { json } => {
            bench_cmd::run_bench_mcp_schema_budget(json)?;
        }
        // F-2b: SeCodePLT-structured static-gate insecure-rate benchmark.
        // Drives the REAL evaluate_vibegate OFFLINE, writes a timestamped
        // report, emits one InsecureRateMeasured SSB beacon. Exits 0/1.
        CliAction::BenchSeccodeplt { json } => {
            bench_cmd::run_bench_seccodeplt(json)?;
        }
        // I-9 (SCHEMA_VERSION 263): error-amplification (alpha_t) telemetry
        // harness. Injects a deterministic byte-perturbation at one step of a
        // frozen pipeline fixture, computes alpha_t + recovered fraction +
        // active-annealing temperature, writes a timestamped report, emits one
        // ErrorAmplificationMeasured SSB beacon. --no-repair runs the R2
        // negative control (alpha == 1.0). Exits 0/1.
        CliAction::BenchErrorAmplification {
            fixture,
            seed,
            json,
            no_repair,
        } => {
            bench_cmd::run_bench_error_amplification(fixture, seed, json, no_repair)?;
        }
        // H-30 (SCHEMA_VERSION 239): staleness / intent-drift INSTRUMENT. Drives
        // the SHIPPED bi-temporal memory substrate over a labeled supersession
        // fixture in an ISOLATED in-memory store, writes a timestamped report,
        // emits one StalenessEvalMeasured SSB beacon. --broken-retrieval runs the
        // R2 negative control (stale_fact_reached_prompt > 0). Exits 0/1.
        CliAction::BenchStaleness { json, broken_retrieval } => {
            bench_cmd::run_bench_staleness(json, broken_retrieval)?;
        }
        // G-6 (SCHEMA_VERSION 217): gate-ON-vs-gate-OFF SAFEDELTA ablation.
        // Derives the delta from the SHIPPED SecCodePltRunner (GD-1, no F-2
        // re-implement). Writes timestamped JSON + signed .rra manifest.
        // Emits ONE SafetyDeltaMeasured SSB beacon. Exits 0/1.
        CliAction::BenchSafedelta {
            suites,
            seed,
            json,
            endpoint,
            model,
            live,
            protocol,
        } => {
            bench_safedelta::run_live(suites, seed, json, live, protocol, endpoint, model)?;
        }
        // PS-3 (SCHEMA_VERSION 277): gate-ON-vs-gate-OFF hallucination-delta
        // ablation. FAITHFUL MIRROR of G-6 safedelta. Tier-1 = fixture corpus
        // (pure computation, no live model). Writes timestamped JSON + signed
        // .rra manifest. Emits ONE HallucinationDeltaMeasured SSB beacon.
        // Tier-2 live CGG = RUNPOD-PENDING. Exits 0/1.
        CliAction::BenchHallucinationDelta {
            suites,
            seed,
            json,
            endpoint,
            model,
        } => {
            bench_cmd::run_bench_hallucination_delta(suites, seed, json, endpoint, model)?;
        }
        // PS-READINESS-E2ERUNNER Slice B (SCHEMA_VERSION 278): the ONLY live
        // consumer of the 4B sidecar at http://127.0.0.1:8181. Drives a REAL
        // model (health-check, generate Rust fn, splice, compile, run), emits
        // ONE LocalLaneE2eMeasured SSB beacon + a signed .rra manifest. A
        // missing sidecar is a hard FAIL (R1), never a synthesized pass.
        CliAction::BenchLocalLaneE2e {
            json,
            endpoint,
            model,
        } => {
            bench_cmd::run_bench_local_lane_e2e(json, endpoint, model)?;
        }
        // K-18a (PS-5 observe arm): recurrence-hit-rate over the shipped BM25
        // retrieval stack. Drives case_reuse_runner over the embedded fixture
        // corpus; writes a blake3-pinned JSON report + emits 1
        // RecurrenceHitRateMeasured SSB beacon + signed .rra (SCHEMA_VERSION 270).
        // Exits 0/1.
        CliAction::BenchCaseReuse { suite, seed, json } => {
            bench_cmd::run_bench_case_reuse(suite, seed, json)?;
        }
        // K-2: hybrid-vs-force-cloud routing economics over deterministic
        // UsageTurnEvent fixture rows. Emits RoutingEconomicsMeasured + RRA.
        CliAction::BenchRoutingEconomics { seed, json, force_cloud_everything } => {
            bench_cmd::run_bench_routing_economics(seed, json, force_cloud_everything)?;
        }
        // K-9: model-refresh cadence over the hardware catalog. Emits one
        // ModelRefreshCadence SSB row per decision + sealed RRA.
        CliAction::BenchModelRefresh { lane, seed, json } => {
            bench_cmd::run_bench_model_refresh(lane, seed, json)?;
        }
        // K-4 (PS-2 owner, KEYSTONE of K-T3): composed gate-tax budget.
        // Drives gate_tax_runner over the real session journal or a labeled
        // fixture composition on a dry journal; writes a JSON report + emits
        // 1 GateTaxMeasured SSB beacon + signed .rra (SCHEMA_VERSION 271).
        // `--enforce-ceiling` (MOVE 3/6, SP2) applies the ceiling-breach
        // exit-code contract (CEILING-PENDING[K-4] fails closed, ABORT-5).
        // Exits 0/1.
        CliAction::BenchGateTax { seed, json, enforce_ceiling } => {
            bench_cmd::run_bench_gate_tax(seed, json, enforce_ceiling)?;
        }
        // I-34 (SCHEMA_VERSION 234): CRANE hybrid grammar-pin gate deterministic
        // fixture harness. Drives the three-layer gate WITHOUT live inference
        // (W-10 PENDING). Emits 3 GrammarGateEvent SSB rows. Exits 0/1.
        CliAction::BenchCrane { crane_fixture, seed } => {
            bench_crane::run(crane_fixture, seed)?;
        }
        // I-27 (SCHEMA_VERSION 265): convergence curve fixture harness.
        // Statistical floor mechanically enforced (seed_count>=5, paired=true).
        // Emits 1 ConvergenceCurveMeasured SSB flat beacon. Live per-node
        // compute-spend leg pending I-4 (WIRE-DARK[I-27]). Exits 0/1.
        CliAction::BenchConvergence { suite, seeds, json } => {
            bench_convergence::run(suite, seeds, json)?;
        }
        // J-34 leg-1 (SCHEMA_VERSION 286): context-fill calibration curve
        // fixture harness. Statistical floor mechanically enforced
        // (seed_count>=5, paired=true). Builds a FillDegradationCurve,
        // constructs a CalibrationArtifact, and emits 1
        // ContextCalibrationEvent(CurveMeasured) SSB row via
        // CalibrationArtifactStore::insert_with_ssb. Live per-model inference
        // leg deferred (WIRE-DARK[J-34.leg1-live]). Exits 0/1.
        CliAction::BenchContextFill { suite, seeds, json } => {
            bench_context_fill::run(suite, seeds, json)?;
        }
        // J-107: LongCoT two-arm fixture benchmark.
        CliAction::BenchLongcot { suite, seeds, json } => {
            bench_longcot::run(suite, seeds, json)?;
        }
        // J-34 leg-3: positional-value curve fixture harness. Builds a
        // PositionalValueCurve at depth fractions 0.0/0.25/0.5/0.75/1.0,
        // constructs a CalibrationArtifact, and emits ONE
        // ContextCalibrationEvent(CurveMeasured) SSB row via
        // CalibrationArtifactStore::insert_with_ssb. Live per-model inference
        // leg deferred (WIRE-DARK[J-34.leg3-live]). Exits 0/1.
        CliAction::BenchPositionalProbe { suite, seeds, json } => {
            bench_positional_probe::run(suite, seeds, json)?;
        }
        // I-38 (SCHEMA_VERSION 235): QDELTA defective-diff rate fixture harness.
        // Statistical floor mechanically enforced (seed_count>=5, paired=true).
        // Emits 1 QdeltaMeasured SSB flat beacon. Live-model leg deferred (D-026).
        CliAction::BenchQdelta { config, seed, json } => {
            bench_qdelta::run(config, seed, json)?;
        }

        CliAction::BenchKvTune { dry_run, emit_ssb, json } => {
            bench_kv_tune::run(dry_run, emit_ssb, json)?;
        }        // C-37: Structure Snowballing harness. Constrained-decoding 5-class
        // Reflector replication. Exits 0/1.
        CliAction::BenchSnowballing {
            pool,
            n_samples,
            model,
            max_trials,
            reflector_mode,
            sidecar_url,
            output,
            python_bin,
        } => {
            bench_snowballing::run(
                pool,
                n_samples,
                model,
                max_trials,
                reflector_mode,
                sidecar_url,
                output,
                python_bin,
            )?;
        }
        // I-43 (SCHEMA_VERSION 236): `elai bench claim-scope` — claim scoper kill-gate.
        // Fixture leg only; live-model leg deferred (D-026). Emits 3 ClaimScopeMeasured
        // SSB beacons (PilotDelta / CrossLaneDivergence / ClaimScoped).
        CliAction::BenchClaimScope { suite, seed, json } => {
            bench_claim_scope::run(suite, seed, json)?;
        }
        // I-33 (SCHEMA_VERSION 240): `elai bench governance-probe` — mechanical
        // decision governance fixture harness (CEFL / I6Q / commit-reveal). Three
        // GovernancePrimitiveFired arms without live inference. Exits 0/1.
        CliAction::BenchGovernanceProbe { seed } => {
            bench_governance::run(seed)?;
        }
        // I-35: `elai bench sequence-probe [--seed N]` — tool-sequence enforcement bench.
        // Replays 3 mock fixtures against the builtin SequenceRuleSet, emits
        // ToolSequenceVerdict SSB rows, exits non-zero on any mismatch.
        CliAction::BenchSequenceProbe { seed } => {
            bench_sequence_probe::run(seed)?;
        }
        // J-24 MS4: `elai bench steering-vs-prompt --task <fixture> [--seed <u64>] [--json]`
        // Deterministic fixture scoring of steered vs baseline arm, emits
        // SteeringEffectMeasured SSB row, writes TOML report.
        CliAction::BenchSteeringVsPrompt { task, seed, json } => {
            bench_steering_vs_prompt::run(task, seed, json)?;
        }
        // J-72 slice D: `elai bench retrieval-adapt` — Tier-1 MRR/Recall
        // harness for baseline vs adapted slot delta over a gold set.
        CliAction::BenchRetrievalAdapt {
            gold,
            k,
            baseline_ranks,
            adapted_ranks,
            out,
            json,
        } => {
            bench_cmd::run_bench_retrieval_adapt(
                gold,
                k,
                baseline_ranks,
                adapted_ranks,
                out,
                json,
            )?;
        }
        // J-77 I11: `elai bench perception-faithfulness` — 6-metric fixture
        // harness (fuse_detections + filter_ghost_regions). Thin fixture-only
        // bench, no SSB. --enforce exits non-zero on metric fail.
        CliAction::BenchToolOutput { corpus, seed, json, out } => {
            bench_cmd::run_bench_tool_output(corpus, seed, json, out)?;
        }
        CliAction::BenchRrfAblate { gold, k, full_ranks, drop_pagerank_ranks, drop_second_dense_ranks, nav_only_graph_ranks, out, json } => {
            bench_cmd::run_bench_rrf_ablate(gold, k, full_ranks, drop_pagerank_ranks, drop_second_dense_ranks, nav_only_graph_ranks, out, json)?;
        }
        CliAction::BenchPerceptionFaithfulness {
            fixture,
            json,
            enforce,
        } => {
            bench_perception_faithfulness::run(fixture, json, enforce)?;
        }
        // I-32: `elai role-ab-probe [--seed N]` — role-contract A/B harness.
        // Loads fixtures from crates/contracts/tests/fixtures/role_contracts/,
        // assembles system-prompt sections with/without role contracts, emits
        // RoleContractApplied(AbDelta) SSB rows, exits non-zero on regression.
        CliAction::RoleAbProbe { seed } => {
            role_ab_probe_cmd::run(seed)?;
        }
        // G-10 (SCHEMA_VERSION 218): `elai model-drift baseline|check|show`.
        // Exogenous model-version behavioral-regression gate. DISTINCT from
        // `elai drift` (E-45 cross-language SOURCE drift). Composes D-7
        // canary.rs (GD-1). Default-ON; ELAI_MODEL_DRIFT_DISABLE kill-switch only.
        CliAction::ModelDrift { action, output_format } => {
            model_drift_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?;
        }
        // H-41: `elai checker-gaps [--limit N] [--json]` — ranked backlog of
        // Factual/Computational claim-classes that fell to non-Constructive.
        CliAction::CheckerGaps { limit, json } => {
            checker_gaps_cmd::run(limit, json)?;
        }
        // J-28 T8: `elai usage [--json]` — cloud/local token-usage rollup,
        // grouped by (model, task, session, lane), cloud and local
        // SEPARATELY LABELED (stub_ban: no naive cross-lane sum).
        CliAction::Usage { json } => {
            usage_cmd::run_usage(json)?;
        }
        // D-28: `elai farm-adv-rm <run|verdict>` — adversarial example
        // farming runner. Drives AdvRmFarmer (orchestrator-side) and
        // writes corpus.jsonl with `gold_verdict: "pending"`. Single
        // SsbActionType::AdversarialFarmRunCompleted SSB row at end-of-
        // run when invoked from a task_runner; CLI does not persist SSB
        // rows directly (mirrors D-19/D-23/D-24/D-25 bench dispatcher
        // precedent). Exits 0 on Pass, 1 on Fail. EXCLUDED from
        // bench_cmd (Tranche J) per ctx package decision-5: D-28 owns
        // its own sibling boundary + inline FarmAdvRmArgs construction.
        CliAction::FarmAdvRm { sub } => {
            farm_adv_rm::dispatch(sub)?;
        }
        CliAction::BenchAblation { tile, fixtures, budget } => {
            bench_cmd::run_bench_ablation(tile, fixtures, budget)?;
        }
        // E-7: darwinian-evolve substrate exerciser. Both cfg arms now live
        // inside darwin_loop_cmd::dispatch; main.rs is cfg-agnostic.
        CliAction::DarwinLoop { generations, mutants, seed, traps, gamma } => {
            darwin_loop_cmd::dispatch(generations, mutants, seed, traps, gamma)?;
        }
        CliAction::TierEval { suites, budget } => {
            bench_cmd::run_tier_eval(suites, budget)?;
        }
        CliAction::CriticReport { output } => {
            bench_cmd::run_critic_report(output)?;
        }
        // E-6: `elai mast report [--session <id>] [--limit N] [--json]` --
        // deterministic MAST failure-mode report read from audit_mast_tagged
        // SSB rows (read-only). Producer is per_level_audit_dispatch
        // record_verdict (wire-dark unless ELAI_FAILURE_TRACKER_ENABLE=1).
        CliAction::Mast { session, limit, json } => {
            mast_cmd::run(session, limit, json)?;
        }
        // E-47: `elai lora-gate-check [--config PATH] [--json]` -- ship-gate
        // dispatcher for the LoRA fine-tuning subsystem. Wire-dark unless
        // ELAI_LORA_SHIP_GATE_ENABLE=1. Producer-side training stack is
        // RUNPOD-PENDING; gate ships locally per E-47 default-deferred
        // posture. See workflow/reasoning/E-47-decisions.md.
        CliAction::LoraGateCheck { config, json } => {
            lora_gate_check_cmd::run(config, json)?;
        }
        // E-25 Tranche C (C.4): `elai checkpoint <list|rewind>` --
        // edit_checkpoints CLI surface. Wire-dark default OFF for SSB
        // emit; inspection paths work regardless.
        CliAction::CheckpointSub { sub } => {
            checkpoint_cmd::run(sub)?;
        }
        // E-9: `elai eval <run|list> --benchmark=<NAME> [--harness-vary]
        // [--p-target=0.05] [--list] [--json]`. External agent-eval benchmark
        // dispatcher across 11 suites (swe-bench-pro / swe-bench-live /
        // livecodebench-v6 / tau2-bench / bfcl-v3 / gaia-l1-l2 /
        // clawsbench-split / refinebench / sycophancy-probes / math-floors /
        // deep-planning). Live evals RUNPOD-PENDING; host-side scaffolding
        // ships fully + emits SsbActionType::BenchmarkRunObserved when env
        // gate `ELAI_EVAL_BENCHMARK_ENABLE=1` is set. See
        // workflow/reasoning/E-9-decisions.md.
        CliAction::EvalRun {
            benchmark,
            harness_vary,
            p_target,
            list,
            json,
        } => {
            eval_cmd::run(benchmark, harness_vary, p_target, list, json)?;
        }
        // E-55: agentic RL recipe runner. Live cloud Sonnet rollouts +
        // LoRA training RUNPOD-PENDING (env-gated by
        // ELAI_AGENTIC_RL_ENABLE=1). See workflow/reasoning/E-55-decisions.md.
        CliAction::RlTrain { recipe, list, json } => {
            rl_train_cmd::run(recipe, list, json)?;
        }
        // E-30: `elai incident record ...` — store an incident postmortem entry.
        CliAction::IncidentRecord {
            title,
            incident_date,
            source_url,
            incident_id,
        } => {
            cmd_incident_record(title, incident_date, source_url, incident_id)?;
        }
        // D-2: `elai ppm health [--endpoint URL] [--timeout SECS] [--json]`.
        // Three-step PPM sidecar health probe (TCP -> /health -> canary
        // /v1/ppm/score). Owns its own tokio runtime in `ppm_health::run`
        // (sync entrypoint pattern, same as `archive_verify::run`). Exits 1
        // on any step failure via std::process::exit inside the handler.
        CliAction::PpmHealth {
            endpoint,
            timeout_secs,
            json,
        } => {
            ppm_health::run(endpoint, timeout_secs, json)?;
        }
        // H-33 (SCHEMA_VERSION 211): `elai memory <sub>` — now fully routed.
        // Activates the stub left by s-31708. Handler in memory_cmd.rs.
        CliAction::Memory {
            sub,
            project_root,
            user_id,
            scope,
            fact_type,
            fact_id,
            key,
            value_json,
            limit,
            json,
            count_only,
        } => {
            memory_cmd::run_memory_subcommand(
                sub,
                &project_root,
                &user_id,
                scope.as_deref(),
                fact_type.as_deref(),
                fact_id.as_deref(),
                key.as_deref(),
                value_json.as_deref(),
                limit,
                json,
                count_only,
            )?;
        }
        // C-40: `elai pr <open|list|status|close> [args...]` — GitHub
        // PR adapter CLI surface. Owns its own current-thread tokio
        // runtime in `run_pr_subcommand` (sync entry, mirrors
        // archive_verify::run + wip_squash_subcommand::run).
        CliAction::Pr {
            sub,
            repo,
            title,
            body,
            head,
            base,
            draft,
            pr_number,
            state_filter,
            limit,
            json,
        } => {
            github_pr_cmd::run_pr_subcommand(
                sub,
                repo,
                title,
                body,
                head,
                base,
                draft,
                pr_number,
                state_filter,
                limit,
                json,
            )?;
        }
        CliAction::PrReview {
            sub,
            pr,
            body,
            file,
            line,
            template,
            kv_pairs,
            since,
            ruleset,
            loc_added,
            loc_removed,
            files_changed,
            touches_owned,
            repo,
            branch,
            touched_files,
            default_owner,
            title,
            json,
        } => {
            let action = build_pr_review_action(
                &sub,
                pr,
                body,
                file,
                line,
                template,
                kv_pairs,
                since,
                ruleset,
                loc_added,
                loc_removed,
                files_changed,
                touches_owned,
                repo,
                branch,
                touched_files,
                default_owner,
                title,
            )?;
            pr_review_cmd::run_pr_review_subcommand(action, json)?;
        }
        CliAction::EvalCorpusIngest { pack, root } => {
            eval_corpus_ingest_cmd::run(pack, root)?;
        }
        // J-72 slice C: `elai retrieval gold-build` — Tier-1 gold-set
        // assembler. Handler in retrieval_cmd.rs.
        CliAction::RetrievalGoldBuild {
            out,
            rra_dir,
            history,
            seed_queries,
            max_hard_neg,
            json,
        } => {
            retrieval_cmd::run_retrieval_gold_build(
                out,
                rra_dir,
                history,
                seed_queries,
                max_hard_neg,
                json,
            )?;
        }
    }
    Ok(())
}

// build_pr_review_action + cmd_incident_record moved to crate::cli_helpers
// (Tranche O.1). Re-exported via the glob above; called by run() above
// at the CliAction::PrReview and CliAction::IncidentRecord match arms.
