//! E-13 — TDD verification agents module.
//!
//! Closes the MISSING net-new artifacts of the E-13 plan with a
//! SCOPE-LEAN ELAI-design lean (per re-check report). The plan
//! originally proposed six orchestrator modules + a `mutant-hunter`
//! agent + property-based + cargo-mutants + Fuzz4All gates; the
//! shipped surface here ships the canonical TDD cycle wrapper, an
//! anti-mock heuristic, and a cargo-mutants invocation wrapper that
//! composes with D-3's [`crate::verification_result::ConsensusRule`]
//! (instead of re-implementing aggregation as the plan §4 stand-in
//! suggested — D-3 is already SHIPPED and verified by session 3).
//!
//! The following four sub-modules are NET-NEW:
//!   - [`cycle`] — `TddCycle` enum {Red, Green, Refactor} +
//!     `RedGreenRefactorState`.
//!   - [`agent`] — `TddVerifierAgent::run_cycle` driver that emits
//!     `TddCycleStarted` / `TddTestFailed` / `TddTestPassed` SSB
//!     beacons via the orchestrator's existing producer chain.
//!   - [`mutator`] — `cargo-mutants 27.0.0` subprocess wrapper using
//!     `--file <path>` repeated argv (the plan's `--shard <files>`
//!     flag does NOT exist on cargo-mutants 27.0; see decisions
//!     doc).
//!   - [`anti_mock`] — heuristic that flags `#[mockall]` / `Mock<*>`
//!     / `MockAll::` / `mock!()` insertions in newly-emitted test
//!     code.
//!   - [`consensus`] — adapter that maps a TDD cycle result into a
//!     `Vec<VerificationResult>` for D-3's
//!     `ConsensusRule::VerifierKindHeterogeneous` aggregator.
//!
//! Citation discipline (per cluster-wide cite-incident taxonomy in
//! workflow/sessions/s-25120/mistakes.md): the E-13 plan cites 11
//! arXiv IDs as design inspiration. Only LLMorpheus (arXiv:2404.09952)
//! and ThinkPRM (arXiv:2504.16828) are inline-relevant to the TDD
//! cycle + verifier-agent shape; cite verification for the remaining
//! 9 is DEFERRED — they are named by author/title only in the
//! decisions doc, never inline-cited in code. See
//! workflow/reasoning/E-13-decisions.md DECISION-1 for the deferral
//! rationale.
//!
//! RUNPOD-PENDING: live cargo-mutants invocation on a representative
//! crate with mutation-score evaluation is RUNPOD-PENDING — the
//! local dev host may not have `cargo-mutants 27.0.0` on PATH.
//! [`mutator::run_cargo_mutants`] surfaces the
//! [`mutator::MutatorError::ToolchainMissing`] variant when the
//! binary is absent (consistent with C-52 dafny + C-28 mutation
//! testing wire-dark posture).

pub mod agent;
pub mod anti_mock;
pub mod consensus;
pub mod cycle;
pub mod mutator;

pub use agent::{CycleResult, TddTask, TddVerifierAgent, TddVerifierError};
pub use anti_mock::{anti_mock_scan, AntiMockHit};
pub use consensus::tdd_results_to_verifier_results;
pub use cycle::{RedGreenRefactorState, TddCycle};
pub use mutator::{
    cargo_mutants_argv, MutationOutcome, MutatorError, MUTANTS_DEFAULT_TIMEOUT_SECS,
};
