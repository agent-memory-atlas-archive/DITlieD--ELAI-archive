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
