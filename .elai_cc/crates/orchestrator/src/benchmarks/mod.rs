//! D-19: orchestrator-side benchmark runners.
//!
//! Each runner consumes one of the D-2 / D-3 / D-x verifier surfaces and
//! produces a typed report + an SSB row so the dashboard surfaces benchmark
//! health without re-running the eval.
//!
//! Current submodules:
//! - `prmbench_runner` (D-19): PRMBench adversarial gaming runner against
//!   the PPM (Qwen2.5-Math-PRM-7B) verifier via `crate::ppm_client::PpmClient`.
//! - `mode_routing_runner` (D-21): mode-routing coverage benchmark replaying
//!   a hand-labeled fixture through D-10's `select_grounding_mode` and
//!   reporting per-mode F1 / macro F1 / 5x5 confusion matrix.
//!
//! Future submodules (D-20..D-25) will live alongside; each owns its own
//! fixture loader + scoring loop and emits one or more SSB rows on
//! completion.

pub mod prmbench_runner;
pub mod mode_routing_runner;
// D-22: conformal-calibration validation under distribution shift. Replays
// a hand-labeled held-out fixture through D-16's `decide` predicate and
// reports per-domain refusal rates + the UniCR (arXiv:2503.01345) band.
pub mod conformal_validation_runner;
// D-20: ARC-AGI-2 OOD probe harness — gates ELAI's mode-routing + novelty-
// classifier honesty stack against ARC-style out-of-distribution claims.
// Does NOT auto-publish DiscoveryCandidate; detects memorization-gaming
// signature (Search class + Constructive mode).
pub mod arc_agi2_runner;
// D-23: sycophancy detection benchmark on top of the D-14 SycophancyProbe
// substrate. Loads a labelled fixture corpus, dispatches each row through
// the pure-function `dispatch_sycophancy_probe` classifier, aggregates
// TP/FP/TN/FN + Inconclusive into a GatingVerdict, and emits one
// `SycophancyBenchRunCompleted` SSB row per run.
pub mod sycophancy_runner;
// D-24: cross-domain canary regression benchmark on top of the D-7
// run_canary substrate. Replays a labelled fixture corpus of synthetic
// helpful / neutral / harmful mutations through D-7's run_canary
// IN-MEMORY (no per-fixture SSB pollution) and aggregates a 4-bucket
// confusion matrix (true-block / false-block / true-pass / false-pass)
// gated by three thresholds (harmful_block_floor / helpful_block_ceiling
// / neutral_block_ceiling). Emits one `CanaryBenchRunCompleted` SSB row
// per run.
pub mod canary_runner;
// D-25: determinism + archive tamper-suite benchmark on top of the D-8
// `verify_archive` substrate. Drives a 10-scenario JSON-manifest matrix
// IN-PROCESS (no shell subprocess — Windows-friendly + 5000ms per-
// scenario budget achievable) applying typed `TamperOp` mutations
// against a sandboxed JSONL chain, then invokes verify_archive directly
// and tallies per-scenario outcomes into a `TamperBenchmarkReport`
// gated by a 5000ms wall budget. Emits one
// `DeterminismTamperSuiteCompleted` SSB row per run.
pub mod tamper_runner;
// F-2b: SeCodePLT-structured static-gate insecure-rate benchmark. Drives the
// REAL ELAI static write-gate (`vibegate::evaluate_vibegate`) over a
// SeCodePLT-structured CWE corpus (Python + TypeScript) OFFLINE (no model, no
// sandbox) and computes a before/after insecure-rate + gate recall. before =
// corpus ground-truth insecure-rate; after = the static false-negative
// residual (vulnerable candidates the gate did NOT Block). Both are COMPUTED
// from the scorer, never literals. Distinct from SeCodePLT's DYNAMIC
// secure-coding rate (the live-model 3-way metric is RunPod-gated).
pub mod seccodeplt_runner;
// G-6 (SCHEMA_VERSION 217): SAFEDELTA gate-ON-vs-gate-OFF safety-delta runner.
// DERIVES the delta from the SHIPPED SecCodePltRunner (GD-1, compose-not-
// restate). Adds a seeded bootstrap 95% CI + per-CWE delta breakdown.
// Tier-1 = static-gate ablation over the embedded corpus (offline, deterministic).
// Tier-2 = live multi-suite (SUSVIBES-200 arXiv:2512.03262, BaxBench-392
// arXiv:2502.11844, dynamic exploit-exec floor) = RUNPOD-PENDING.
pub mod safedelta_runner;
// H-30: staleness / intent-drift INSTRUMENT. Drives the SHIPPED bi-temporal
// memory substrate (FactStore + audit_conflicts + get_relevant_facts) over a
// labeled supersession fixture in an ISOLATED in-memory store and computes two
// grounded rates (STALE arXiv:2605.06527): SR (State Resolution = recognition,
// did the supersession path close the stale row) and IPA (Implicit Policy
// Adaptation = application, did retrieval serve the fresh value and never the
// stale one). The invariant SR >= IPA always holds (application cannot exceed
// recognition). The --broken-retrieval arm reads include-closed rows so a
// recognized-stale row LEAKS (stale_fact_reached_prompt > 0) -- R2: the
// instrument must be able to scream. Offline, deterministic, no model, no live
// store. MEASURES, does not gate (default-ON, no ELAI_*_ENABLE flag).
pub mod staleness_runner;
// H-34: MCP-schema token-budget MEASURE-FIRST runner. Drives an embedded
// multi-server MCP tool catalog (20 tools: local-rag / pod-lifecycle / notion)
// OFFLINE and computes a before/after schema token cost: baseline (all tools
// injected eagerly) vs. injected (floor-only set, defer-load ON-path minimum).
// Both token counts are COMPUTED from the corpus fixture, never literals.
// baseline > injected guaranteed by corpus structure. Injected <= 500 tokens
// (INJECTED_CAP_TOKENS named const). Live multi-server measurement (real
// tokenizer, RunPod) is a benchmark-class deferral tracked under H-34.
pub mod mcp_schema_budget_runner;
// I-3: SSB write-loop throughput runner. Drives N sequential write_payload
// calls against an isolated tempfile Ssb instance, records inserts/sec, and
// writes metrics/phase_I3.toml (ID-5: non-deterministic timing, not asserted).
// Runtime: multi_thread (SSB_NEW_MUST_BE_BUILT_INSIDE_BLOCK_ON_MULTI_THREAD_RUNTIME).
pub mod ssb_throughput_runner;
