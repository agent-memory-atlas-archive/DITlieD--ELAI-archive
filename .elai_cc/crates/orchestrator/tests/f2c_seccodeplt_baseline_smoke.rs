//! F-2c SeCodePLT baseline smoke (offline static-gate, NO model).
//!
//! Drives the SHIPPED `SecCodePltRunner` over the embedded corpus through the
//! REAL `vibegate::evaluate_vibegate` static gate (now carrying the 16-rule /
//! 20-row security ruleset incl CWE-798/321/532/95) and asserts:
//!   1. n_samples (candidates) >= 80 (>= 40 corpus samples × 2 candidates).
//!   2. before/after/gate_recall are COMPUTED (before > after, recall in (0,1)).
//!   3. the per-CWE breakdown INCLUDES CWE-798/321/532/95, each with at least
//!      one insecure candidate caught (insecure_blocked > 0) so before != after
//!      for the new classes (the rules actually fire on the new fixtures).
//!   4. the InsecureRateMeasured beacon emits exactly one row into a tempdir
//!      SSB store — the off-agent artifact — built INSIDE a tokio runtime
//!      (Ssb::new panics "no reactor running" outside rt.block_on).
//!
//! Deterministic: the static gate is model-independent and the corpus is
//! embedded, so the run is byte-stable under a fixed seed.

use std::sync::Arc;

use elai_orchestrator::benchmarks::seccodeplt_runner::SecCodePltRunner;
use elai_orchestrator::ssb::{
    InsecureRateMeasuredPayload, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
};

/// The four CWE classes F-2c added to the ruleset + corpus.
const NEW_CWES: &[&str] = &["CWE-798", "CWE-321", "CWE-532", "CWE-95"];

#[test]
fn baseline_run_is_nondegenerate_and_covers_new_cwes() {
    let runner = SecCodePltRunner::from_embedded().expect("embedded corpus must parse");
    let report = runner.run();

    // 1. corpus scale: >= 40 samples => >= 80 candidates.
    assert!(
        report.n_samples >= 80,
        "expected >= 80 candidates (>= 40 samples × 2); got {}",
        report.n_samples
    );

    // 2. computed, non-degenerate rates.
    assert!(
        report.insecure_rate_before > report.insecure_rate_after,
        "before ({}) must exceed after ({}) — the gate must catch some insecure",
        report.insecure_rate_before,
        report.insecure_rate_after
    );
    assert!(
        report.gate_recall > 0.0 && report.gate_recall < 1.0,
        "gate_recall ({}) must be a non-trivial fraction",
        report.gate_recall
    );

    // 3. per-CWE breakdown includes the 4 new classes, each with the new rule
    //    actually firing (insecure_blocked > 0 => before != after for the class).
    for cwe in NEW_CWES {
        let slice = report
            .per_cwe
            .iter()
            .find(|p| p.cwe == *cwe)
            .unwrap_or_else(|| panic!("per-CWE breakdown must include {cwe}"));
        assert!(
            slice.insecure_candidates > 0,
            "{cwe} must have at least one insecure candidate in the corpus"
        );
        assert!(
            slice.insecure_blocked > 0,
            "{cwe}: the F-2c rule must BLOCK at least one insecure candidate \
             (before != after for this class); got insecure_blocked=0"
        );
        // FP guard: the patched candidates for the new classes must not all be
        // false-blocked (at least the secure side is partially clean). A fully
        // false-blocking rule would make secure_false_blocks == (candidates -
        // insecure_candidates); assert it is strictly less.
        let secure_candidates = slice.candidates - slice.insecure_candidates;
        assert!(
            slice.secure_false_blocks < secure_candidates || secure_candidates == 0,
            "{cwe}: the rule false-blocked EVERY secure candidate ({} of {}) — FP guard",
            slice.secure_false_blocks,
            secure_candidates
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn baseline_emits_one_insecure_rate_measured_row() {
    let runner = SecCodePltRunner::from_embedded().expect("embedded corpus must parse");
    let report = runner.run();

    let dir = tempfile::TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb_seccodeplt_smoke.db"),
        pool_size: Some(2),
        project_id: "test-project".into(),
        session_id: "test-session".into(),
    };
    let clock: Arc<dyn elai_core::ClockProvider> =
        Arc::new(elai_core::clock::SeededClock::new());

    // Ssb::new spawns the single-writer task and MUST be constructed inside the
    // tokio runtime context (the #[tokio::test] provides it). Outside a reactor
    // this panics "no reactor running".
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb init"));

    let payload = InsecureRateMeasuredPayload {
        suite: report.suite.clone(),
        model_id: report.model_id.clone(),
        n_samples: report.n_samples,
        insecure_rate_before: report.insecure_rate_before,
        insecure_rate_after: report.insecure_rate_after,
        gate_recall: report.gate_recall,
        seed: report.seed,
        corpus_blake3: report.corpus_blake3.clone(),
    };
    ssb.write_payload(
        "test:bench-seccodeplt",
        0,
        SsbPayload::InsecureRateMeasured(payload),
        Vec::new(),
    )
    .await
    .expect("write InsecureRateMeasured beacon");

    let rows = ssb
        .query_by_type(SsbActionType::InsecureRateMeasured)
        .expect("query InsecureRateMeasured rows");
    assert_eq!(
        rows.len(),
        1,
        "exactly one InsecureRateMeasured beacon must land in the SSB store"
    );
}
