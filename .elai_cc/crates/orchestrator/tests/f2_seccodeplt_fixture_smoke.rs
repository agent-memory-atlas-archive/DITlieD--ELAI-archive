//! F-2b / Slice A4 -- SeCodePLT fixture + runner smoke contract.
//!
//! This is an integration-level smoke test for the SeCodePLT-structured
//! static-gate insecure-rate runner ([`elai_orchestrator::benchmarks::seccodeplt_runner`]).
//! It is DISTINCT from the runner's in-crate `#[cfg(test)] mod tests`: those
//! exercise the runner against the embedded corpus from inside the crate; this
//! file exercises the public API exactly as an external caller (the bench CLI,
//! the contracts crate) sees it, and pins the four A4 contract guarantees:
//!
//!   (a) every fixture sample carries all required fields (non-empty id, lang,
//!       cwe, task_description, vulnerable_code, patched_code);
//!   (b) the runner produces `before > after` (non-degenerate) and a
//!       `gate_recall` in `(0, 1]`, both COMPUTED, recomputed from the fixtures
//!       and asserted NOT to equal any hard-coded constant;
//!   (c) `corpus_blake3` is stable across two independent runner builds
//!       (determinism / provenance pin);
//!   (d) the corpus has >=40 samples spanning >=4 distinct CWEs, including at
//!       least one vulnerable sample the static gate MISSES (so `after > 0`).
//!
//! HONESTY: the number under test is a STATIC-gate insecure-rate, distinct from
//! the SeCodePLT dynamic secure-coding rate (sandbox-executed). See
//! `workflow/reasoning/F-2-decisions.md`.

use std::collections::BTreeSet;

use elai_orchestrator::benchmarks::seccodeplt_runner::{
    SecCodePltReport, SecCodePltRunner, CORPUS_MODEL_ID, SUITE_LABEL,
};

/// Build a runner from the embedded corpus via the public production
/// constructor. `from_embedded` reads `ELAI_SEED` for the *recorded* seed only;
/// per the runner contract the candidate set is assembled in corpus order with
/// NO RNG dependency, so the seed never affects the computed rates, the per-CWE
/// breakdown, or `corpus_blake3` -- every value this test asserts is identical
/// regardless of any ambient `ELAI_SEED`.
fn build_runner() -> SecCodePltRunner {
    SecCodePltRunner::from_embedded().expect("embedded SeCodePLT corpus must parse")
}

// ---------------------------------------------------------------------------
// (a) Every fixture sample carries all required fields.
// ---------------------------------------------------------------------------

#[test]
fn fixture_samples_have_all_required_fields() {
    let runner = build_runner();
    let samples = runner.samples();
    assert!(
        !samples.is_empty(),
        "embedded corpus must contain at least one sample"
    );

    for s in samples {
        assert!(!s.id.is_empty(), "sample id must be non-empty");
        assert!(
            !s.lang.is_empty(),
            "sample {} must declare a lang",
            s.id
        );
        assert!(
            s.lang == "python" || s.lang == "typescript",
            "sample {} lang must be python|typescript, got {:?}",
            s.id,
            s.lang
        );
        assert!(
            s.cwe.starts_with("CWE-"),
            "sample {} cwe must be a CWE-* identifier, got {:?}",
            s.id,
            s.cwe
        );
        assert!(
            !s.task_description.is_empty(),
            "sample {} must have a task_description",
            s.id
        );
        assert!(
            !s.vulnerable_code.is_empty(),
            "sample {} must have vulnerable_code",
            s.id
        );
        assert!(
            !s.patched_code.is_empty(),
            "sample {} must have patched_code",
            s.id
        );
        assert_ne!(
            s.vulnerable_code, s.patched_code,
            "sample {} vulnerable_code and patched_code must differ",
            s.id
        );
    }
}

// ---------------------------------------------------------------------------
// (b) before > after, gate_recall in (0, 1], both COMPUTED (recomputed from
//     the fixtures, never compared to a hard-coded constant).
// ---------------------------------------------------------------------------

#[test]
fn runner_before_gt_after_and_recall_is_a_fraction() {
    let runner = build_runner();
    let report = runner.run();

    // Non-degenerate: the gate must catch SOME insecure candidates and MISS
    // some (residual), so before strictly exceeds after and after is positive.
    assert!(
        report.insecure_rate_before > 0.0,
        "insecure_rate_before ({}) must be > 0",
        report.insecure_rate_before
    );
    assert!(
        report.insecure_rate_before > report.insecure_rate_after,
        "insecure_rate_before ({}) must exceed insecure_rate_after ({})",
        report.insecure_rate_before,
        report.insecure_rate_after
    );
    assert!(
        report.insecure_rate_after > 0.0,
        "insecure_rate_after ({}) must be > 0 (gate must MISS some residual classes)",
        report.insecure_rate_after
    );

    // gate_recall in (0, 1].
    assert!(
        report.gate_recall > 0.0 && report.gate_recall <= 1.0,
        "gate_recall ({}) must be in (0, 1]",
        report.gate_recall
    );

    // RECOMPUTE the keystone numbers independently from the fixtures so the
    // assertion proves the report is COMPUTED, not a literal. We re-derive
    // before / after / recall directly from the parsed samples + the public
    // n_samples field, never from a hard-coded target.
    let n = f64::from(report.n_samples);
    assert!(n > 0.0, "n_samples must be > 0");

    // Each sample contributes exactly two candidates (vulnerable + patched), so
    // the candidate count is 2 * samples and the ground-truth insecure count is
    // exactly the sample count (one vulnerable per sample). before therefore
    // equals samples / (2*samples) by construction -- but we assert the report's
    // value MATCHES this recomputation rather than asserting any fixed number.
    let sample_count_u32 =
        u32::try_from(runner.samples().len()).expect("sample count fits in u32");
    let sample_count = f64::from(sample_count_u32);
    let recomputed_before = sample_count / n;
    assert!(
        (report.insecure_rate_before - recomputed_before).abs() < 1e-12,
        "insecure_rate_before ({}) must equal recomputed samples/n ({})",
        report.insecure_rate_before,
        recomputed_before
    );

    // after recomputed from the per-CWE residual breakdown.
    let residual: u32 = report.per_cwe.iter().map(|c| c.insecure_residual).sum();
    let recomputed_after = f64::from(residual) / n;
    assert!(
        (report.insecure_rate_after - recomputed_after).abs() < 1e-12,
        "insecure_rate_after ({}) must equal recomputed residual/n ({})",
        report.insecure_rate_after,
        recomputed_after
    );

    // recall recomputed from the recomputed before/after.
    let recomputed_recall = (recomputed_before - recomputed_after) / recomputed_before;
    assert!(
        (report.gate_recall - recomputed_recall).abs() < 1e-12,
        "gate_recall ({}) must equal recomputed (before-after)/before ({})",
        report.gate_recall,
        recomputed_recall
    );

    // Explicit ANTI-HARD-CODE guard: the keystone numbers must be the corpus +
    // gate COMPUTATION, not any of the F-2 NON-TARGET literals. If a future
    // edit accidentally pinned a number, these would trip.
    for forbidden in [0.591_f64, 0.923, 0.432, 0.760, 0.55, 0.15] {
        assert!(
            (report.insecure_rate_before - forbidden).abs() > 1e-9,
            "insecure_rate_before must not equal NON-TARGET literal {forbidden}"
        );
        assert!(
            (report.insecure_rate_after - forbidden).abs() > 1e-9,
            "insecure_rate_after must not equal NON-TARGET literal {forbidden}"
        );
        assert!(
            (report.gate_recall - forbidden).abs() > 1e-9,
            "gate_recall must not equal NON-TARGET literal {forbidden}"
        );
    }

    // Provenance fields are wired through.
    assert_eq!(report.suite, SUITE_LABEL, "suite label must round-trip");
    assert_eq!(
        report.model_id, CORPUS_MODEL_ID,
        "model_id (candidate-gen provenance) must round-trip"
    );
}

// ---------------------------------------------------------------------------
// (c) corpus_blake3 is stable across two independent runner builds.
// ---------------------------------------------------------------------------

#[test]
fn corpus_blake3_is_stable_across_runs() {
    let a = build_runner();
    let b = build_runner();
    assert_eq!(
        a.corpus_blake3(),
        b.corpus_blake3(),
        "two runner builds over the same embedded corpus must hash identically"
    );
    assert_eq!(
        a.corpus_blake3().len(),
        64,
        "blake3 hex digest must be 64 chars"
    );
    assert!(
        a.corpus_blake3().chars().all(|c| c.is_ascii_hexdigit()),
        "blake3 digest must be all hex"
    );

    // The digest must also survive into the report, byte-for-byte, twice.
    let ra: SecCodePltReport = a.run();
    let rb: SecCodePltReport = b.run();
    assert_eq!(
        ra.corpus_blake3, rb.corpus_blake3,
        "report corpus_blake3 must be deterministic"
    );
    assert_eq!(
        ra.corpus_blake3,
        a.corpus_blake3(),
        "report corpus_blake3 must match the runner digest"
    );
    // Full report equality is the strongest determinism guarantee.
    assert_eq!(ra, rb, "two reports over the same corpus must be identical");
}

// ---------------------------------------------------------------------------
// (d) >=40 samples spanning >=4 distinct CWEs, including >=1 the gate MISSES.
// ---------------------------------------------------------------------------

#[test]
fn corpus_scale_cwe_spread_and_has_a_gate_miss() {
    let runner = build_runner();

    assert!(
        runner.samples().len() >= 40,
        "corpus must have >=40 samples, got {}",
        runner.samples().len()
    );

    let cwes: BTreeSet<&str> = runner.samples().iter().map(|s| s.cwe.as_str()).collect();
    assert!(
        cwes.len() >= 4,
        "corpus must span >=4 distinct CWEs, got {} ({:?})",
        cwes.len(),
        cwes
    );

    // At least one vulnerable sample the gate MISSES => insecure residual > 0 in
    // the run. We read this off the COMPUTED report (the per-CWE residual sum),
    // which is the same signal the bench beacon will carry.
    let report = runner.run();
    let residual: u32 = report.per_cwe.iter().map(|c| c.insecure_residual).sum();
    assert!(
        residual > 0,
        "corpus must contain >=1 vulnerable sample the static gate does NOT block (after>0)"
    );

    // The residual must come from a CWE class distinct from the block classes,
    // i.e. there is a per-CWE entry with insecure_residual > 0.
    let has_residual_class = report.per_cwe.iter().any(|c| c.insecure_residual > 0);
    assert!(
        has_residual_class,
        "at least one per-CWE entry must report insecure_residual > 0"
    );

    // No patched (secure) candidate may be falsely blocked anywhere -- the FP
    // floor for the static gate on this corpus is zero.
    let false_blocks: u32 = report.per_cwe.iter().map(|c| c.secure_false_blocks).sum();
    assert_eq!(
        false_blocks, 0,
        "no patched (secure) candidate may be falsely blocked"
    );
}
