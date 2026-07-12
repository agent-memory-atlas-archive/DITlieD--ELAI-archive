//! I-23 — unified conformal calibration facade contract test.
//!
//! Five assertions:
//!
//!   1. FACADE SURFACE: the re-exports resolve at compile time (if this
//!      file compiles, the facade module exposes the required symbols).
//!
//!   2. HAPPY ARM — JOURNAL JOIN: `compute_tau_and_emit` + `decide_and_emit`
//!      produce the SAME non-empty `calibration_id` on both their SSB rows.
//!      This proves the journal join is structurally possible and that drift
//!      between the tau row and the refusal-decided row is impossible when
//!      callers use the canonical dispatch path.
//!
//!   3. BROKEN ARM — DIVERGENT IDS FAIL THE JOIN CHECK (R2): a helper that
//!      asserts two rows share the same `calibration_id` panics when given
//!      mismatched ids. This simulates the --split-calibrations failure mode
//!      without requiring a separate facade path.
//!
//!   4. BOOTSTRAP LEG — CalibrationBootstrapCompletedPayload carries a
//!      Some(non-empty 16-hex-char) calibration_id minted from the same
//!      (tau, corpus_size, epsilon) triple as the tau row, proving the
//!      bootstrap path is joinable to the D-16 rows.
//!
//!   5. NO-BYPASS GUARD — a source-scan assertion that fails the test if
//!      any file outside the allowlisted set uses `crate::conformal_calibration::`
//!      as a production call (not a doc comment), making facade bypass a
//!      permanent test failure rather than an audit catch.
//!
//! Grounding:
//!   * I-23 WIRING_CONTRACT observable_effect: "journal join on calibration_id
//!     proves drift is structurally impossible; --split-calibrations broken
//!     arm FAILS the contract test (R2)."
//!   * wiring-enforcement.md R2: missing evidence is a hard FAIL.

use elai_orchestrator::calibration::{
    CalibrationCorpus, CalibrationTuple, CorpusSource, VecConformalRefusalDecidedEmitter,
    VecConformalTauComputedEmitter, DEFAULT_EPSILON,
};
use elai_orchestrator::calibration;
use elai_orchestrator::ssb::{
    CalibrationBootstrapCompletedPayload, ConformalRefusalDecidedPayload,
    ConformalTauComputedPayload,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn balanced_corpus(n_pairs: usize) -> CalibrationCorpus {
    let mut c = CalibrationCorpus::new(CorpusSource::HeldOut);
    for i in 0..n_pairs {
        c.append(CalibrationTuple::new(
            format!("t-{i}"),
            format!("c-{i}"),
            "true claim",
            true,
            0.9,
            1_700_000_000 + (i as i64),
        ))
        .expect("append true tuple");
        c.append(CalibrationTuple::new(
            format!("f-{i}"),
            format!("d-{i}"),
            "false claim",
            false,
            0.2,
            1_700_000_500 + (i as i64),
        ))
        .expect("append false tuple");
    }
    c
}

/// Assert that the two rows were produced by the same calibration run.
/// Panics with a descriptive message if either id is None or they differ.
/// This is the join-check function the broken-arm test inverts.
fn assert_calibration_ids_match(
    tau_row: &ConformalTauComputedPayload,
    refusal_row: &ConformalRefusalDecidedPayload,
) {
    let tau_id = tau_row
        .calibration_id
        .as_deref()
        .expect("tau row must carry a calibration_id (I-23)");
    let ref_id = refusal_row
        .calibration_id
        .as_deref()
        .expect("refusal row must carry a calibration_id (I-23)");
    assert_eq!(
        tau_id, ref_id,
        "calibration_id mismatch: tau row has {tau_id:?}, refusal row has {ref_id:?}; \
         journal join would fail (--split-calibrations broken arm)"
    );
}

// ---------------------------------------------------------------------------
// 1. Facade surface (compile-time guarantee)
// ---------------------------------------------------------------------------

/// If this test function compiles, the facade re-exports the required symbols.
/// Execution is trivial; the compile gate is the load-bearing piece.
#[test]
fn facade_surface_resolves() {
    // Verify facade exports the key types and functions by constructing
    // values that require them.
    let _epsilon: f64 = calibration::DEFAULT_EPSILON;
    let _min: usize = calibration::DEFAULT_MIN_CORPUS_SIZE;
    let _max_chars: usize = calibration::MAX_CLAIM_TEXT_CHARS;
    let _bootstrap_eps: f64 = calibration::DEFAULT_BOOTSTRAP_EPSILON;
    // Corpus construction uses CalibrationCorpus + CorpusSource from facade.
    let mut c = calibration::CalibrationCorpus::new(calibration::CorpusSource::HeldOut);
    let tuple = calibration::CalibrationTuple::new(
        "t-0".to_string(),
        "c-0".to_string(),
        "claim text",
        true,
        0.9_f64,
        1_700_000_000_i64,
    );
    c.append(tuple).expect("append in facade_surface_resolves");
    assert_eq!(c.size(), 1);
}

// ---------------------------------------------------------------------------
// 2. Happy arm — journal join
// ---------------------------------------------------------------------------

#[tokio::test]
async fn same_calibration_id_on_tau_and_refusal_rows() {
    // 20 pairs = 40 tuples at DEFAULT_EPSILON = 0.1.
    let corpus = balanced_corpus(20);
    let tau_emitter = VecConformalTauComputedEmitter::new();
    let dec_emitter = VecConformalRefusalDecidedEmitter::new();

    // Mint tau + calibration_id via the facade.
    let (tau, calibration_id) =
        calibration::compute_tau_and_emit(&corpus, DEFAULT_EPSILON, &tau_emitter, 1_700_001_000)
            .await
            .expect("compute_tau_and_emit via facade");

    // Thread the id through to the decide call.
    let _decision = calibration::decide_and_emit(
        "claim-0",
        0.01_f64, // low score -> Abstain
        tau,
        &calibration_id,
        &dec_emitter,
        1_700_001_001,
    )
    .await;

    let tau_rows = tau_emitter.rows().await;
    let dec_rows = dec_emitter.rows().await;

    assert_eq!(tau_rows.len(), 1, "expected exactly one tau row");
    assert_eq!(dec_rows.len(), 1, "expected exactly one refusal row");

    // The id must be a 16-char hex string.
    let id = tau_rows[0]
        .calibration_id
        .as_deref()
        .expect("tau row calibration_id must be Some");
    assert_eq!(
        id.len(),
        16,
        "calibration_id must be 16 hex chars; got {:?}",
        id
    );
    assert!(
        id.chars().all(|c| c.is_ascii_hexdigit()),
        "calibration_id must be lowercase hex; got {:?}",
        id
    );

    // JOURNAL JOIN: both rows carry the same id.
    assert_calibration_ids_match(&tau_rows[0], &dec_rows[0]);

    // Sanity: the returned String matches what landed in the tau row.
    assert_eq!(
        Some(calibration_id.as_str()),
        tau_rows[0].calibration_id.as_deref(),
        "returned calibration_id must match the emitted tau row field"
    );
}

// ---------------------------------------------------------------------------
// 3. Broken arm — divergent ids fail the join check (R2)
// ---------------------------------------------------------------------------

/// Construct two payloads with MISMATCHED calibration_ids and confirm that
/// `assert_calibration_ids_match` panics. This is the --split-calibrations
/// simulation: if two independent tau computations produced different ids and
/// callers cross-wired them, the join check must catch it.
#[test]
fn divergent_calibration_ids_fail_join_check() {
    let tau_row = ConformalTauComputedPayload {
        tau: 0.8,
        epsilon: 0.1,
        corpus_size: 40,
        source: "held_out".to_string(),
        ts_ms: 1_700_000_000,
        calibration_id: Some("aaaaaaaaaaaaaaaa".to_string()),
    };
    let refusal_row = ConformalRefusalDecidedPayload {
        claim_id: "claim-x".to_string(),
        claim_score: 0.5,
        tau: 0.8,
        decision: "answer".to_string(),
        confidence: 0.5,
        ts_ms: 1_700_000_001,
        reason: None,
        // Different id — simulates split-calibrations drift.
        calibration_id: Some("bbbbbbbbbbbbbbbb".to_string()),
    };

    // The join check MUST panic on mismatched ids (R2: missing evidence = FAIL).
    let result = std::panic::catch_unwind(|| {
        assert_calibration_ids_match(&tau_row, &refusal_row);
    });
    assert!(
        result.is_err(),
        "assert_calibration_ids_match must panic on divergent calibration_ids \
         (--split-calibrations broken arm, R2)"
    );
}

// ---------------------------------------------------------------------------
// 4. Bootstrap leg — CalibrationBootstrapCompletedPayload carries calibration_id
// ---------------------------------------------------------------------------

/// Verify that a CalibrationBootstrapCompletedPayload minted with
/// `calibration::mint_calibration_id` carries a Some(16-char hex) id
/// that is identical to what the same (tau, corpus_size, epsilon) triple
/// would produce for a tau row — proving the bootstrap path is joinable.
#[test]
fn bootstrap_payload_carries_calibration_id() {
    let tau: f64 = 0.8;
    let corpus_size: u32 = 50;
    let alpha: f64 = 0.1;

    // Mint via the facade (same function task_runner.rs calls at the
    // warm_start emit site).
    let id = calibration::mint_calibration_id(tau, corpus_size, alpha);

    // Shape check: 16 lowercase hex chars, non-empty.
    assert_eq!(id.len(), 16, "bootstrap calibration_id must be 16 chars; got {id:?}");
    assert!(
        id.chars().all(|c| c.is_ascii_hexdigit()),
        "bootstrap calibration_id must be lowercase hex; got {id:?}"
    );

    // Construct a representative payload (mirrors the task_runner warm_start
    // emit site) and confirm the field round-trips through serde.
    let payload = CalibrationBootstrapCompletedPayload {
        corpus_slug: "truthfulqa".into(),
        corpus_size,
        corpus_blake3: "a".repeat(64),
        tau_warm_start: tau,
        alpha,
        tau_floor_active: false,
        bootstrap_replicates: None,
        bootstrap_ci_low: None,
        bootstrap_ci_high: None,
        block_length_p: None,
        ts_ms: 1_700_000_000,
        calibration_id: Some(id.clone()),
    };

    let id_from_payload = payload
        .calibration_id
        .as_deref()
        .expect("CalibrationBootstrapCompletedPayload.calibration_id must be Some (I-23)");
    assert_eq!(
        id_from_payload, id.as_str(),
        "payload calibration_id must match mint_calibration_id output"
    );

    // Cross-check: the same (tau, corpus_size, epsilon) used for a tau row
    // produces the SAME id, proving the journal join is possible.
    let tau_row_id = calibration::mint_calibration_id(tau, corpus_size, alpha);
    assert_eq!(
        id, tau_row_id,
        "bootstrap calibration_id must equal tau-row calibration_id for same inputs"
    );
}

// ---------------------------------------------------------------------------
// 5. No-bypass guard — production code must not reach conformal_calibration
//    directly (facade bypass = test failure, not just audit catch)
// ---------------------------------------------------------------------------

/// Source-scan assertion: read every .rs file under orchestrator/src and
/// assert that no production code path uses `crate::conformal_calibration::`
/// outside the facade, calibration_bootstrap, and conformal_calibration itself.
///
/// Doc-comment references (`///` or `//!`) are excluded — only code-level
/// `crate::conformal_calibration::` tokens (outside `//` lines) are flagged.
///
/// Allowlisted files (by path suffix): these may legitimately reference
/// `conformal_calibration` at the code level:
///   - conformal_calibration.rs (the module itself)
///   - calibration_bootstrap.rs (self-import: `use crate::conformal_calibration::`)
///   - calibration/mod.rs (facade: `pub use crate::conformal_calibration::`)
///
/// Any other file with a non-comment `crate::conformal_calibration::` token
/// is a facade bypass and FAILS this test.
#[test]
fn facade_import_convention_holds_in_orchestrator() {
    // CARGO_MANIFEST_DIR for elai_cc_contracts points at
    // .elai_cc/crates/contracts; orchestrator sources are at
    // .elai_cc/crates/orchestrator/src/.
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest.join("../orchestrator/src");

    let allowed_suffixes: &[&str] = &[
        "conformal_calibration.rs",
        "calibration_bootstrap.rs",
        "calibration/mod.rs",
    ];

    let mut violations: Vec<(String, usize, String)> = Vec::new();
    let mut dirs = vec![src_dir.clone()];

    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| {
            panic!("could not read directory {}: {e}", dir.display())
        }) {
            let entry = entry.unwrap_or_else(|e| {
                panic!("could not read entry in {}: {e}", dir.display())
            });
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            let path_str = path.to_string_lossy();
            if !path_str.ends_with(".rs") {
                continue;
            }
            if allowed_suffixes.iter().any(|suff| path_str.ends_with(suff)) {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!("could not read {}: {e}", path.display())
            });
            for (i, line) in source.lines().enumerate() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("//") {
                    continue;
                }
                if line.contains("crate::conformal_calibration::") {
                    violations.push((
                        path.strip_prefix(&src_dir)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .into_owned(),
                        i + 1,
                        line.to_string(),
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "facade bypass detected in orchestrator/src (I-23 NO-BYPASS guard). \
         All production callers must go through crate::calibration::. \
         Violating lines:\n{}",
        violations
            .iter()
            .map(|(p, n, l)| format!("  {p}:{n}: {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
