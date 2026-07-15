//! K-9 model-refresh cadence contract.
//!
//! Locks the SSB ModelRefreshCadence surface (SCHEMA_VERSION 276) and the
//! runner's safe-arm: keep / re_evaluate only, no silent adopt, fixture_run
//! false this leg, and one decision per catalog lane.

use elai_orchestrator::benchmarks::model_refresh_runner::run_model_refresh;
use elai_orchestrator::hardware_catalog::CatalogLaneKind;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ModelRefreshCadenceKind, ModelRefreshCadencePayload,
    SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};

const K9_SCHEMA_FLOOR: i64 = 276;

fn sample_payload() -> ModelRefreshCadencePayload {
    ModelRefreshCadencePayload {
        kind: ModelRefreshCadenceKind::DecisionRecorded,
        lane: "coding".into(),
        candidate_model: "Qwen2.5-Coder-7B-Instruct".into(),
        incumbent_model: "Qwen2.5-Coder-7B-Instruct".into(),
        verdict: "keep".into(),
        pass_rate: None,
        vram_mb: Some(8192),
        tok_per_s: None,
        dollars_per_verified_task: None,
        fixture_run: false,
        notes: "contract fixture".into(),
    }
}

#[test]
fn schema_version_includes_k9_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= K9_SCHEMA_FLOOR,
        "schema must include K-9 ModelRefreshCadence (got {SCHEMA_VERSION_PUBLIC}, floor {K9_SCHEMA_FLOOR})"
    );
}

#[test]
fn model_refresh_cadence_action_type_roundtrip() {
    let s = SsbActionType::ModelRefreshCadence.as_sql_str();
    assert_eq!(s, "model_refresh_cadence");
    let decoded = ssb_action_type_from_str_public(s)
        .expect("ssb_action_type_from_str_public model_refresh_cadence");
    assert_eq!(decoded, SsbActionType::ModelRefreshCadence);
}

#[test]
fn model_refresh_cadence_payload_msgpack_and_json_roundtrip() {
    let p = sample_payload();
    let payload = SsbPayload::ModelRefreshCadence(p.clone());

    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(decoded, payload);
    if let SsbPayload::ModelRefreshCadence(d) = decoded {
        assert_eq!(d, p);
    } else {
        panic!("expected ModelRefreshCadence payload");
    }

    let json = serde_json::to_string(&p).expect("json serialize");
    let from_json: ModelRefreshCadencePayload =
        serde_json::from_str(&json).expect("json deserialize");
    assert_eq!(from_json, p);
}

#[test]
fn model_refresh_cadence_action_type_bijection() {
    let payload = SsbPayload::ModelRefreshCadence(sample_payload());
    assert_eq!(
        payload.action_type(),
        SsbActionType::ModelRefreshCadence
    );
}

#[test]
fn model_refresh_cadence_kind_three_arm_closed_set() {
    let cases = [
        (ModelRefreshCadenceKind::CandidateEvaluated, "candidate_evaluated"),
        (ModelRefreshCadenceKind::DecisionRecorded, "decision_recorded"),
        (ModelRefreshCadenceKind::IncumbentKept, "incumbent_kept"),
    ];
    for (variant, wire) in cases {
        let json = serde_json::to_string(&variant).expect("kind json serialize");
        assert_eq!(json, format!("\"{wire}\""));
        let decoded: ModelRefreshCadenceKind =
            serde_json::from_str(&json).expect("kind json deserialize");
        assert_eq!(decoded, variant);
    }
    fn _exhaustive(k: ModelRefreshCadenceKind) -> &'static str {
        match k {
            ModelRefreshCadenceKind::CandidateEvaluated => "candidate_evaluated",
            ModelRefreshCadenceKind::DecisionRecorded => "decision_recorded",
            ModelRefreshCadenceKind::IncumbentKept => "incumbent_kept",
        }
    }
}

#[test]
fn runner_emits_one_decision_per_lane_with_measured_schema() {
    let report = run_model_refresh(None, 0).expect("run_model_refresh");
    assert!(
        !report.decisions.is_empty(),
        "expected one decision per catalog lane"
    );
    for d in &report.decisions {
        assert!(!d.fixture_run, "this leg never runs the live fixture");
        assert_eq!(d.pass_rate, None);
        assert_eq!(d.tok_per_s, None);
        assert_eq!(d.dollars_per_verified_task, None);
    }
}

#[test]
fn runner_safe_arm_never_adopts() {
    let report = run_model_refresh(None, 0).expect("run_model_refresh");
    for d in &report.decisions {
        assert!(
            d.verdict == "keep" || d.verdict == "re_evaluate",
            "safe-arm only emits keep/re_evaluate, got {} for lane {}",
            d.verdict,
            d.lane
        );
        assert_ne!(d.verdict, "adopt");
    }
}

#[test]
fn runner_keep_retains_incumbent() {
    let report = run_model_refresh(None, 0).expect("run_model_refresh");
    for d in &report.decisions {
        if d.verdict == "keep" {
            assert!(
                !d.incumbent_model.is_empty(),
                "keep must retain a non-empty incumbent for lane {}",
                d.lane
            );
        }
    }
}

#[test]
fn runner_is_byte_deterministic() {
    let first = run_model_refresh(None, 0).expect("first");
    let second = run_model_refresh(None, 0).expect("second");
    assert_eq!(
        serde_json::to_vec(&first).expect("serialize first"),
        serde_json::to_vec(&second).expect("serialize second")
    );
}

#[test]
fn runner_single_lane_query_filters_coding() {
    let report = run_model_refresh(Some(CatalogLaneKind::Coding), 0).expect("coding-only");
    assert!(!report.decisions.is_empty());
    for d in &report.decisions {
        assert_eq!(d.lane, "coding");
    }
}
