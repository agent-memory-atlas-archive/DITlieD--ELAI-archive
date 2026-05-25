// D-25 contract: determinism + archive tamper-suite benchmark surface.
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 144 (head; D-25 DeterminismTamperSuiteCompleted
//    variant landed in this atomic +1 bump).
// 2. SsbActionType::DeterminismTamperSuiteCompleted round-trips via
//    as_sql_str / ssb_action_type_from_str_public (binds the wire-format
//    string).
// 3. DeterminismTamperSuiteCompletedPayload round-trips via msgpack
//    (rmp_serde) AND JSON (serde_json) without loss.
// 4. action_type() on the matching SsbPayload variant returns the same
//    SsbActionType (preserves the bijection).
// 5. GatingVerdict 2-arm exhaustive serde (Pass + Fail) — closed-set
//    invariant inherited from D-19 DECISION-2 and reused by D-25.
// 6. TamperOp enum exhaustive serde (10 variants incl Box-recursive arms)
//    — closed taxonomy invariant per D-25 DECISION-6.
// 7. Outcome enum 4-arm exhaustive serde (CleanPass + ExpectedError +
//    UnexpectedError + Timeout) — closed taxonomy invariant.
// 8. scenarios_blake3 stability test (same input -> same hash field
//    through msgpack + json round-trip).
// 9. Wall-budget invariant: DEFAULT_MAX_WALL_MS_PER_SCENARIO == 5000
//    (pinned constant per plan objective; D-25 DECISION-7).

use elai_orchestrator::benchmarks::prmbench_runner::GatingVerdict;
use elai_orchestrator::benchmarks::tamper_runner::{
    ExpectedOutcome, Outcome, ScenarioManifest, TamperOp, BENCH_RUNNER_NAME,
    DEFAULT_MAX_WALL_MS_PER_SCENARIO,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, DeterminismTamperSuiteCompletedPayload, SsbActionType,
    SsbPayload, SCHEMA_VERSION_PUBLIC,
};

/// Floor for D-25: head SCHEMA_VERSION when DeterminismTamperSuiteCompleted
/// variant lands.
const D25_SCHEMA_FLOOR: i64 = 144;

#[test]
fn schema_version_includes_d25_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= D25_SCHEMA_FLOOR,
        "schema must include D-25 DeterminismTamperSuiteCompleted variant (got {SCHEMA_VERSION_PUBLIC}, floor {D25_SCHEMA_FLOOR})"
    );
}

#[test]
fn determinism_tamper_suite_completed_action_type_roundtrip() {
    let s = SsbActionType::DeterminismTamperSuiteCompleted.as_sql_str();
    assert_eq!(s, "determinism_tamper_suite_completed");
    let decoded = ssb_action_type_from_str_public(s)
        .expect("ssb_action_type_from_str_public determinism_tamper_suite_completed");
    assert_eq!(decoded, SsbActionType::DeterminismTamperSuiteCompleted);
}

#[test]
fn determinism_tamper_suite_completed_payload_msgpack_roundtrip() {
    let p = DeterminismTamperSuiteCompletedPayload {
        total_scenarios: 10,
        passed: 10,
        failed: 0,
        max_wall_ms: 250,
        verdict: "pass".into(),
        scenarios_blake3: "a".repeat(64),
    };
    let payload = SsbPayload::DeterminismTamperSuiteCompleted(p.clone());
    assert_eq!(
        payload.action_type(),
        SsbActionType::DeterminismTamperSuiteCompleted
    );

    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(
        decoded.action_type(),
        SsbActionType::DeterminismTamperSuiteCompleted
    );
    if let SsbPayload::DeterminismTamperSuiteCompleted(d) = decoded {
        assert_eq!(d, p);
    } else {
        panic!("expected DeterminismTamperSuiteCompleted payload");
    }
}

#[test]
fn determinism_tamper_suite_completed_payload_json_roundtrip() {
    let p = DeterminismTamperSuiteCompletedPayload {
        total_scenarios: 10,
        passed: 7,
        failed: 3,
        max_wall_ms: 5200,
        verdict: "fail".into(),
        scenarios_blake3: "b".repeat(64),
    };
    let json = serde_json::to_string(&p).expect("json serialize");
    assert!(json.contains("\"total_scenarios\""));
    assert!(json.contains("\"passed\""));
    assert!(json.contains("\"failed\""));
    assert!(json.contains("\"max_wall_ms\""));
    assert!(json.contains("\"verdict\""));
    assert!(json.contains("\"scenarios_blake3\""));

    let decoded: DeterminismTamperSuiteCompletedPayload =
        serde_json::from_str(&json).expect("json deserialize");
    assert_eq!(decoded, p);
}

#[test]
fn gating_verdict_exhaustive_serde_pass_and_fail() {
    for v in [GatingVerdict::Pass, GatingVerdict::Fail] {
        let json = serde_json::to_string(&v).expect("verdict json serialize");
        let decoded: GatingVerdict = serde_json::from_str(&json).expect("verdict json deserialize");
        assert_eq!(decoded, v);
    }
}

#[test]
fn tamper_op_exhaustive_serde_roundtrip() {
    let cases = vec![
        TamperOp::BreakParentLink { entry_index: 1 },
        TamperOp::MutateContentHash { entry_index: 2 },
        TamperOp::DuplicateEntry { entry_index: 3 },
        TamperOp::CorruptJsonl { entry_index: 4 },
        TamperOp::DeleteEntry { entry_index: 0 },
        TamperOp::AppendMalformedTrailing,
        TamperOp::IdempotencyDoubleVerify,
        TamperOp::RevertBackup {
            inner: vec![Box::new(TamperOp::MutateContentHash { entry_index: 1 })],
        },
        TamperOp::MultiTamper {
            inner: vec![
                Box::new(TamperOp::BreakParentLink { entry_index: 0 }),
                Box::new(TamperOp::CorruptJsonl { entry_index: 2 }),
            ],
        },
        TamperOp::FailFastSingleError {
            inner: Box::new(TamperOp::BreakParentLink { entry_index: 1 }),
        },
    ];
    // 10 variants — the closed taxonomy invariant.
    assert_eq!(cases.len(), 10);
    for op in &cases {
        let json = serde_json::to_string(op).expect("op json serialize");
        let decoded: TamperOp = serde_json::from_str(&json).expect("op json deserialize");
        assert_eq!(&decoded, op);
    }
}

#[test]
fn outcome_exhaustive_serde_roundtrip() {
    let cases = vec![
        Outcome::CleanPass,
        Outcome::ExpectedError {
            kind: "hash_mismatch".into(),
        },
        Outcome::UnexpectedError {
            kind: "fail_fast_arm".into(),
            msg: "msg".into(),
        },
        Outcome::Timeout,
    ];
    // 4 variants — the closed taxonomy invariant.
    assert_eq!(cases.len(), 4);
    for o in &cases {
        let json = serde_json::to_string(o).expect("outcome json serialize");
        let decoded: Outcome = serde_json::from_str(&json).expect("outcome json deserialize");
        assert_eq!(&decoded, o);
    }
}

#[test]
fn scenarios_blake3_stable_through_json_roundtrip() {
    let p = DeterminismTamperSuiteCompletedPayload {
        total_scenarios: 10,
        passed: 10,
        failed: 0,
        max_wall_ms: 100,
        verdict: "pass".into(),
        scenarios_blake3:
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into(),
    };
    let json = serde_json::to_string(&p).expect("serialize");
    let decoded: DeterminismTamperSuiteCompletedPayload =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded.scenarios_blake3, p.scenarios_blake3);
    assert_eq!(decoded.scenarios_blake3.len(), 64);
}

#[test]
fn wall_budget_default_is_locked_at_5000ms() {
    assert_eq!(
        DEFAULT_MAX_WALL_MS_PER_SCENARIO, 5000,
        "plan objective: each scenario MUST be detected within 5s (5000ms)"
    );
}

#[test]
fn bench_runner_name_is_locked() {
    assert_eq!(BENCH_RUNNER_NAME, "tamper_bench_v1");
}

#[test]
fn scenario_manifest_json_roundtrip() {
    let m = ScenarioManifest {
        id: "scenario_03".into(),
        kind: "ContentHashMismatch".into(),
        tamper_ops: vec![TamperOp::MutateContentHash { entry_index: 1 }],
        expected_outcome: ExpectedOutcome::ExpectedError {
            kind: "hash_mismatch".into(),
        },
        notes: "mutate body without recomputing self_hash".into(),
    };
    let json = serde_json::to_string(&m).expect("serialize manifest");
    let decoded: ScenarioManifest = serde_json::from_str(&json).expect("deserialize manifest");
    assert_eq!(decoded, m);
}

#[test]
fn expected_outcome_multi_error_roundtrip() {
    let e = ExpectedOutcome::MultiError {
        kinds: vec!["broken_parent".into(), "hash_mismatch".into()],
    };
    let json = serde_json::to_string(&e).expect("serialize");
    let decoded: ExpectedOutcome = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded, e);
}
