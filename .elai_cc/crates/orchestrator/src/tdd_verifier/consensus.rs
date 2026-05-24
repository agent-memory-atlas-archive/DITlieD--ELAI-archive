//! E-13 — TDD-cycle / D-3 consensus adapter.
//!
//! Translates a TDD cycle's evidence rows into a
//! `Vec<VerificationResult>` so the existing D-3 aggregator
//! ([`crate::verification_result::aggregate`]) can apply
//! `ConsensusRule::VerifierKindHeterogeneous { required_kinds:
//! vec![VerifierKind::Deterministic] }`.
//!
//! Why this lives here (vs. the plan's §4 stand-in):
//!   The plan §4 calls for a local `aggregate_verifiers` stand-in
//!   in `auditor_mode.rs` "if D-3 has not shipped" — but D-3 IS
//!   SHIPPED at `crates/orchestrator/src/verification_result.rs`
//!   (header line 1, `aggregate` at line 322, `ConsensusRule::
//!   VerifierKindHeterogeneous` at line 248). The re-check report's
//!   exploration verified this. Per the project's "no parallel
//!   surfaces" rule, the TDD verifier composes with D-3 instead of
//!   shipping a second aggregator.
//!
//! The adapter contributes ONE `VerifierKind::Deterministic` row
//! per signal source the TDD cycle produced (example-flip from
//! Red->Green, cargo-mutants outcome if available, anti-mock scan
//! result). D-3's `KofN` or `VerifierKindHeterogeneous` then decides
//! the aggregate verdict — no logic duplication here.

use crate::verification_result::VerificationResult;

use super::agent::CycleResult;

/// Convert a TDD cycle's evidence into a `Vec<VerificationResult>`
/// keyed by `candidate_id`.
///
/// Emits up to three rows per cycle:
///   1. `tdd_example_flip` — `passed = result.test_flipped`.
///   2. `tdd_cargo_mutants` — only when `result.mutation_outcome`
///      is `Some`; `passed = MutationOutcome::is_pass()`.
///   3. `tdd_anti_mock` — `passed = result.anti_mock_clean`.
///
/// All three are `VerifierKind::Deterministic` so they each satisfy
/// the "≥1 non-LLM verifier" rule when combined under D-3's
/// `ConsensusRule::VerifierKindHeterogeneous { required_kinds:
/// vec![VerifierKind::Deterministic] }`.
#[must_use]
pub fn tdd_results_to_verifier_results(
    candidate_id: &str,
    result: &CycleResult,
    ts_ms: i64,
) -> Vec<VerificationResult> {
    let mut out = Vec::with_capacity(3);

    out.push(VerificationResult::from_deterministic(
        candidate_id,
        "tdd_example_flip",
        result.test_flipped,
        ts_ms,
    ));

    if let Some(outcome) = result.mutation_outcome.as_ref() {
        out.push(VerificationResult::from_deterministic(
            candidate_id,
            "tdd_cargo_mutants",
            outcome.is_pass(),
            ts_ms,
        ));
    }

    out.push(VerificationResult::from_deterministic(
        candidate_id,
        "tdd_anti_mock",
        result.anti_mock_clean,
        ts_ms,
    ));

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::mutator::MutationOutcome;
    use crate::verification_result::{aggregate, ConsensusRule, VerifierKind, AggregateVerdict};

    fn flip_only_result(flipped: bool) -> CycleResult {
        CycleResult {
            iterations: 1,
            test_flipped: flipped,
            anti_mock_clean: true,
            mutation_outcome: None,
        }
    }

    #[test]
    fn flip_only_emits_two_rows_no_mutation_row() {
        let res = flip_only_result(true);
        let rows = tdd_results_to_verifier_results("cand-1", &res, 0);
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| r.kind == VerifierKind::Deterministic));
        assert!(rows.iter().all(|r| r.passed));
    }

    #[test]
    fn includes_mutation_row_when_outcome_present() {
        let res = CycleResult {
            iterations: 1,
            test_flipped: true,
            anti_mock_clean: true,
            mutation_outcome: Some(MutationOutcome::AllCaught { mutant_count: 5 }),
        };
        let rows = tdd_results_to_verifier_results("cand-2", &res, 0);
        assert_eq!(rows.len(), 3);
        let mutation_row = rows
            .iter()
            .find(|r| r.verifier_id == "tdd_cargo_mutants")
            .expect("mutation row present");
        assert!(mutation_row.passed);
    }

    #[test]
    fn d3_heterogeneous_passes_with_deterministic_rows() {
        let res = flip_only_result(true);
        let rows = tdd_results_to_verifier_results("cand-3", &res, 0);
        let rule = ConsensusRule::VerifierKindHeterogeneous {
            required_kinds: vec![VerifierKind::Deterministic],
        };
        assert_eq!(aggregate(&rule, &rows), AggregateVerdict::Pass);
    }

    #[test]
    fn d3_heterogeneous_fails_when_flip_failed() {
        let res = flip_only_result(false);
        let rows = tdd_results_to_verifier_results("cand-4", &res, 0);
        let rule = ConsensusRule::VerifierKindHeterogeneous {
            required_kinds: vec![VerifierKind::Deterministic],
        };
        match aggregate(&rule, &rows) {
            AggregateVerdict::Fail { .. } => (),
            other => panic!("expected Fail, got {other:?}"),
        }
    }
}
