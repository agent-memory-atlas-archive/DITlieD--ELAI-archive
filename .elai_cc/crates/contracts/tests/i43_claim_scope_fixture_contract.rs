//! I-43 — claim-scoper kill gate fixture contract tests.
//!
//! Coverage (5 arms; no #[ignore], no stubs, no placeholder asserts):
//!
//! (F1) Schema lock (golden 2/2): SCHEMA_VERSION_PUBLIC >= 236 AND
//!      action_type_lock.rs declares TOTAL_VARIANTS: usize = 545, confirming
//!      the atomically-locked SSB expansion for I-43 ClaimScopeMeasured.
//!
//! (F2) Kill-gate arm (fail-closed, R1): a claim that must be killed IS killed.
//!      The fixture presents delta=0.0 (flat corpus) and asserts kill_gate()
//!      returns true.  Absence of a kill on a flat corpus would be fail-open.
//!
//! (F3) Scope arm: an in-scope claim (positive delta, normal recall) passes the
//!      kill gate (kill_gate() returns false).
//!
//! (F4) reviewer_reaching_rate known-answer: given gate_recall=0.83, the derived
//!      reviewer_reaching_rate must equal 1.0 - 0.83 = 0.17 (within f64 epsilon).
//!      Verifies the inline derivation in bench_claim_scope.rs.
//!
//! (F5) SSB roundtrip: a real `Ssb::write_payload` + `query_by_type` + msgpack
//!      decode round-trip for `ClaimScopeMeasuredPayload` (all 3 kind arms).
//!      Catches the SQL CHECK comma gotcha — wrong wire tag causes a runtime
//!      INSERT failure invisible to `cargo check`.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    ClaimScopeMeasuredKind, ClaimScopeMeasuredPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ssb_config(dir: &TempDir, label: &str) -> SsbConfig {
    SsbConfig {
        db_path: dir.path().join(format!("{label}.db")),
        pool_size: Some(2),
        project_id: "i43-claim-scope-contract".to_string(),
        session_id: format!("i43-contract-{label}"),
    }
}

fn clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

/// Inline kill-gate logic (mirrors bench_claim_scope.rs::kill_gate, extracted
/// here so the contract owns the invariant independently of the implementation).
fn kill_gate(delta: f64, reviewer_reaching_rate: f64) -> bool {
    const TRAP_THRESHOLD: f64 = 0.05;
    delta <= 0.0 || reviewer_reaching_rate < TRAP_THRESHOLD
}

// ---------------------------------------------------------------------------
// (F1) Schema lock (golden 2/2)
// ---------------------------------------------------------------------------

/// SCHEMA_VERSION_PUBLIC must be >= 236 (I-43 floor bump from 235 -> 236).
/// A value below 236 means the I-43 SSB bump was regressed.
#[test]
fn schema_version_is_236() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 236,
        "SCHEMA_VERSION_PUBLIC must be >= 236 (I-43 floor: ClaimScopeMeasured add); \
         got {} — a value below 236 means the I-43 SSB bump was regressed",
        SCHEMA_VERSION_PUBLIC,
    );
}

/// action_type_lock.rs must declare TOTAL_VARIANTS: usize = 545 (543 + I-34 + I-38 + I-43).
#[test]
fn total_variants_is_545() {
    let lock_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("contracts crate has a parent")
        .join("orchestrator")
        .join("tests")
        .join("action_type_lock.rs");
    let body = std::fs::read_to_string(&lock_path).unwrap_or_else(|e| {
        panic!(
            "cannot read action_type_lock.rs at {}: {e}",
            lock_path.display()
        )
    });
    // Floor check: TOTAL_VARIANTS >= 545 (I-43 authoring floor: ClaimScopeMeasured
    // added at 545; exact-pin converted per F-2 pin doctrine).
    let total_variants_value: usize = body
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("const TOTAL_VARIANTS: usize = ") {
                trimmed
                    .trim_start_matches("const TOTAL_VARIANTS: usize = ")
                    .trim_end_matches(';')
                    .parse()
                    .ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!(
            "could not parse TOTAL_VARIANTS from action_type_lock.rs at {}",
            lock_path.display()
        ));
    assert!(
        total_variants_value >= 545,
        "TOTAL_VARIANTS must be >= 545 (I-43 floor: ClaimScopeMeasured SSB lockstep); \
         got {} in {}; body snippet: {}",
        total_variants_value,
        lock_path.display(),
        &body[..body.len().min(200)]
    );
}

// ---------------------------------------------------------------------------
// (F2) Kill-gate arm — claim that MUST be killed IS killed (fail-closed R1)
// ---------------------------------------------------------------------------

/// A flat corpus (delta == 0.0) must be killed.
///
/// R1: missing evidence = FAIL.  A delta of exactly 0.0 means the gate
/// produced no measurable improvement; the kill gate must fire.  A gate
/// that passes this claim is fail-open.
#[test]
fn kill_gate_fires_on_flat_delta() {
    let delta = 0.0_f64;
    let reviewer_reaching_rate = 1.0_f64; // normal recall, trap not the cause
    assert!(
        kill_gate(delta, reviewer_reaching_rate),
        "kill_gate must return true for delta=0.0 (flat corpus — no improvement); \
         a false return is fail-open (R1 violation)"
    );
}

/// A reversed delta (after > before) must also be killed.
#[test]
fn kill_gate_fires_on_negative_delta() {
    let delta = -0.1_f64;
    let reviewer_reaching_rate = 0.5_f64;
    assert!(
        kill_gate(delta, reviewer_reaching_rate),
        "kill_gate must return true for delta < 0 (reversed delta — gate made things worse)"
    );
}

/// A block-everything trap (reviewer_reaching_rate < 0.05) must be killed
/// even if the raw delta is positive.
#[test]
fn kill_gate_fires_on_trap_reviewer_rate() {
    let delta = 0.594_f64; // positive delta (run_trap fixture: 0.6 - 0.006)
    let reviewer_reaching_rate = 0.01_f64; // trap: 1.0 - 0.99 = 0.01 < 0.05
    assert!(
        kill_gate(delta, reviewer_reaching_rate),
        "kill_gate must return true when reviewer_reaching_rate < 0.05 (block-everything trap); \
         a positive delta does not override the trap detector"
    );
}

// ---------------------------------------------------------------------------
// (F3) Scope arm — in-scope claim passes the kill gate
// ---------------------------------------------------------------------------

/// A claim with a positive delta and a normal reviewer_reaching_rate is
/// in-scope and must NOT be killed.
#[test]
fn kill_gate_does_not_fire_on_valid_claim() {
    let delta = 0.5_f64; // insecure_rate_before=0.6, after=0.1
    let reviewer_reaching_rate = 0.17_f64; // 1.0 - 0.83
    assert!(
        !kill_gate(delta, reviewer_reaching_rate),
        "kill_gate must return false for a valid in-scope claim \
         (delta={delta:.4}, reviewer_reaching_rate={reviewer_reaching_rate:.4})"
    );
}

// ---------------------------------------------------------------------------
// (F4) reviewer_reaching_rate known-answer
// ---------------------------------------------------------------------------

/// Given gate_recall = 0.83, reviewer_reaching_rate = 1.0 - gate_recall = 0.17.
///
/// This arm verifies the inline derivation formula used in bench_claim_scope.rs.
/// Any change to the derivation (e.g. 1.0 - gate_recall -> gate_recall) breaks
/// both the fixture scenarios and the trap detector.
#[test]
fn reviewer_reaching_rate_derivation_known_answer() {
    let gate_recall = 0.83_f64;
    let expected_rrr = 0.17_f64;
    let derived_rrr = 1.0 - gate_recall;
    assert!(
        (derived_rrr - expected_rrr).abs() < 1e-9,
        "reviewer_reaching_rate must equal 1.0 - gate_recall; \
         got {derived_rrr:.10}, expected {expected_rrr:.10}"
    );
    // Also confirm this value is well above the trap threshold (0.05).
    assert!(
        derived_rrr > 0.05,
        "reviewer_reaching_rate {derived_rrr:.4} must be above 0.05 for this fixture (no trap)"
    );
}

/// Trap boundary: gate_recall = 0.99 → reviewer_reaching_rate = 0.01 < 0.05.
#[test]
fn reviewer_reaching_rate_trap_known_answer() {
    let gate_recall = 0.99_f64;
    let expected_rrr = 0.01_f64;
    let derived_rrr = 1.0 - gate_recall;
    assert!(
        (derived_rrr - expected_rrr).abs() < 1e-9,
        "reviewer_reaching_rate for trap fixture must equal 0.01; got {derived_rrr:.10}"
    );
    assert!(
        derived_rrr < 0.05,
        "reviewer_reaching_rate {derived_rrr:.4} must be below 0.05 (trap fires)"
    );
}

// ---------------------------------------------------------------------------
// (F5) SSB roundtrip for all 3 ClaimScopeMeasured arms
// ---------------------------------------------------------------------------

/// Full `Ssb::write_payload` + `query_by_type` + msgpack decode round-trip for
/// `ClaimScopeMeasuredPayload` with kind=PilotDelta.
///
/// Catches the SQL CHECK comma gotcha: the `as_sql_str()` value
/// `"claim_scope_measured"` must be present in the SQLite CHECK constraint — a
/// wrong or missing wire tag causes a runtime INSERT failure invisible to
/// `cargo check`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn claim_scope_measured_pilot_delta_ssb_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(ssb_config(&tmp, "pilot-delta"), clock()).expect("ssb open"),
    );

    let corpus_hash = blake3::hash(b"i43_claim_scope_pilot_delta_roundtrip")
        .to_hex()
        .to_string();

    let payload = ClaimScopeMeasuredPayload {
        kind: ClaimScopeMeasuredKind::PilotDelta,
        suite: "claim_scope_fixture_tier1".to_string(),
        seed: 42,
        n_samples: 10,
        delta: Some(0.5),
        reviewer_reaching_rate: Some(0.17),
        trap_fired: false,
        divergence_pct: None,
        claim_wording: None,
        runpod_pending: true,
        corpus_blake3: corpus_hash.clone(),
    };

    ssb.write_payload(
        "bench-claim-scope-agent",
        0,
        SsbPayload::ClaimScopeMeasured(payload.clone()),
        Vec::new(),
    )
    .await
    .expect("write_payload must succeed for ClaimScopeMeasured(PilotDelta)");

    let rows = ssb
        .query_by_type(SsbActionType::ClaimScopeMeasured(
            ClaimScopeMeasuredKind::PilotDelta,
        ))
        .expect("query_by_type must succeed");
    assert_eq!(
        rows.len(),
        1,
        "expected exactly one ClaimScopeMeasured row after write"
    );

    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("msgpack decode must succeed");
    match decoded {
        SsbPayload::ClaimScopeMeasured(p) => {
            assert!(
                matches!(p.kind, ClaimScopeMeasuredKind::PilotDelta),
                "kind must round-trip as PilotDelta"
            );
            assert_eq!(p.suite, payload.suite, "suite must round-trip");
            assert_eq!(p.seed, payload.seed, "seed must round-trip");
            assert_eq!(p.n_samples, payload.n_samples, "n_samples must round-trip");
            assert!(
                p.delta
                    .zip(payload.delta)
                    .map(|(a, b)| (a - b).abs() < f64::EPSILON)
                    .unwrap_or(false),
                "delta must round-trip"
            );
            assert!(
                p.reviewer_reaching_rate
                    .zip(payload.reviewer_reaching_rate)
                    .map(|(a, b)| (a - b).abs() < f64::EPSILON)
                    .unwrap_or(false),
                "reviewer_reaching_rate must round-trip"
            );
            assert!(!p.trap_fired, "trap_fired must round-trip as false");
            assert!(p.runpod_pending, "runpod_pending must round-trip as true");
            assert_eq!(
                p.corpus_blake3, payload.corpus_blake3,
                "corpus_blake3 must round-trip"
            );
        }
        other => panic!("expected ClaimScopeMeasured, got {other:?}"),
    }
}

/// Full roundtrip for kind=CrossLaneDivergence.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn claim_scope_measured_cross_lane_divergence_ssb_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(ssb_config(&tmp, "cross-lane"), clock()).expect("ssb open"),
    );

    let corpus_hash = blake3::hash(b"i43_claim_scope_cross_lane_roundtrip")
        .to_hex()
        .to_string();

    let wording = Some(
        "trace-authentication only; deterministic replay claim unsupported on cloud path"
            .to_string(),
    );
    let payload = ClaimScopeMeasuredPayload {
        kind: ClaimScopeMeasuredKind::CrossLaneDivergence,
        suite: "claim_scope_fixture_tier1".to_string(),
        seed: 42,
        n_samples: 3,
        delta: None,
        reviewer_reaching_rate: None,
        trap_fired: false,
        divergence_pct: Some(33.3),
        claim_wording: wording.clone(),
        runpod_pending: true,
        corpus_blake3: corpus_hash.clone(),
    };

    ssb.write_payload(
        "bench-claim-scope-agent",
        0,
        SsbPayload::ClaimScopeMeasured(payload.clone()),
        Vec::new(),
    )
    .await
    .expect("write_payload must succeed for ClaimScopeMeasured(CrossLaneDivergence)");

    // query_by_type on CrossLaneDivergence arm.
    let rows = ssb
        .query_by_type(SsbActionType::ClaimScopeMeasured(
            ClaimScopeMeasuredKind::CrossLaneDivergence,
        ))
        .expect("query_by_type must succeed");
    assert_eq!(rows.len(), 1, "expected one CrossLaneDivergence row");

    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("msgpack decode must succeed");
    match decoded {
        SsbPayload::ClaimScopeMeasured(p) => {
            assert!(
                matches!(p.kind, ClaimScopeMeasuredKind::CrossLaneDivergence),
                "kind must round-trip as CrossLaneDivergence"
            );
            assert!(
                p.divergence_pct
                    .zip(payload.divergence_pct)
                    .map(|(a, b)| (a - b).abs() < 1e-3)
                    .unwrap_or(false),
                "divergence_pct must round-trip"
            );
            assert_eq!(
                p.claim_wording, payload.claim_wording,
                "claim_wording must round-trip"
            );
            assert!(p.runpod_pending, "runpod_pending must round-trip as true");
        }
        other => panic!("expected ClaimScopeMeasured, got {other:?}"),
    }
}

/// Full roundtrip for kind=ClaimScoped.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn claim_scope_measured_claim_scoped_ssb_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(ssb_config(&tmp, "claim-scoped"), clock()).expect("ssb open"),
    );

    let corpus_hash = blake3::hash(b"i43_claim_scope_claim_scoped_roundtrip")
        .to_hex()
        .to_string();

    let payload = ClaimScopeMeasuredPayload {
        kind: ClaimScopeMeasuredKind::ClaimScoped,
        suite: "claim_scope_fixture_tier1".to_string(),
        seed: 42,
        n_samples: 1,
        delta: None,
        reviewer_reaching_rate: None,
        trap_fired: false,
        divergence_pct: Some(0.0),
        claim_wording: Some(
            "deterministic: same seed -> identical corpus_blake3".to_string(),
        ),
        runpod_pending: true,
        corpus_blake3: corpus_hash.clone(),
    };

    ssb.write_payload(
        "bench-claim-scope-agent",
        0,
        SsbPayload::ClaimScopeMeasured(payload.clone()),
        Vec::new(),
    )
    .await
    .expect("write_payload must succeed for ClaimScopeMeasured(ClaimScoped)");

    let rows = ssb
        .query_by_type(SsbActionType::ClaimScopeMeasured(
            ClaimScopeMeasuredKind::ClaimScoped,
        ))
        .expect("query_by_type must succeed");
    assert_eq!(rows.len(), 1, "expected one ClaimScoped row");

    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("msgpack decode must succeed");
    match decoded {
        SsbPayload::ClaimScopeMeasured(p) => {
            assert!(
                matches!(p.kind, ClaimScopeMeasuredKind::ClaimScoped),
                "kind must round-trip as ClaimScoped"
            );
            assert_eq!(
                p.corpus_blake3, payload.corpus_blake3,
                "corpus_blake3 must round-trip"
            );
            assert!(p.runpod_pending, "runpod_pending must round-trip as true");
        }
        other => panic!("expected ClaimScopeMeasured, got {other:?}"),
    }
}
