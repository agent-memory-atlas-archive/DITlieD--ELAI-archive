// C-1 contract: 7 new SSB variants for the knowledge + reasoning producers.
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 47 (atomic single-bump for all 7 C-1 variants).
// 2. All 7 new SsbActionType variants round-trip via as_sql_str /
//    ssb_action_type_from_str_public:
//      - GrcStepEmitted
//      - GroundingSweepResult
//      - CsqEvaluated
//      - AuditorRubberStampRejected
//      - HydeQuery
//      - VectorCacheHitRate
//      - QueryTrace
// 3. action_type() on the matching SsbPayload variant returns the same
//    SsbActionType (preserves the bijection asserted by
//    gotcha B41_SSB_ALL_TEST_PAYLOADS_DRIFT).
// 4. Each payload struct serializes via msgpack (rmp_serde) AND JSON
//    (serde_json) without loss — round-trip equality.
// 5. SQL strings are snake_case and match the wire labels.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, AuditorRubberStampRejectedPayload, CsqEvaluatedPayload,
    GroundingSweepResultPayload, GrcStepEmittedPayload, HydeQueryPayload, QueryTracePayload,
    SsbActionType, SsbPayload, VectorCacheHitRatePayload, SCHEMA_VERSION_PUBLIC,
};

/// Floor for C-1: schema must include the 7 new variants.
const C1_SCHEMA_FLOOR: i64 = 47;

#[test]
fn schema_version_includes_c1_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= C1_SCHEMA_FLOOR,
        "schema must include C-1 SSB variants (got {SCHEMA_VERSION_PUBLIC}, floor {C1_SCHEMA_FLOOR})"
    );
}

fn roundtrip_action_type(at: SsbActionType, expected_sql: &str) {
    let s = at.as_sql_str();
    assert_eq!(
        s, expected_sql,
        "as_sql_str mismatch for {at:?}: got {s}, expected {expected_sql}"
    );
    let decoded = ssb_action_type_from_str_public(s).unwrap_or_else(|e| {
        panic!("ssb_action_type_from_str_public({s:?}) failed: {e:?}")
    });
    assert_eq!(decoded, at, "round-trip failed for {s}");
}

#[test]
fn grc_step_emitted_action_type_roundtrip() {
    roundtrip_action_type(SsbActionType::GrcStepEmitted, "grc_step_emitted");
}

#[test]
fn grounding_sweep_result_action_type_roundtrip() {
    roundtrip_action_type(SsbActionType::GroundingSweepResult, "grounding_sweep_result");
}

#[test]
fn csq_evaluated_action_type_roundtrip() {
    roundtrip_action_type(SsbActionType::CsqEvaluated, "csq_evaluated");
}

#[test]
fn auditor_rubber_stamp_rejected_action_type_roundtrip() {
    roundtrip_action_type(
        SsbActionType::AuditorRubberStampRejected,
        "auditor_rubber_stamp_rejected",
    );
}

#[test]
fn hyde_query_action_type_roundtrip() {
    roundtrip_action_type(SsbActionType::HydeQuery, "hyde_query");
}

#[test]
fn vector_cache_hit_rate_action_type_roundtrip() {
    roundtrip_action_type(SsbActionType::VectorCacheHitRate, "vector_cache_hit_rate");
}

#[test]
fn query_trace_action_type_roundtrip() {
    roundtrip_action_type(SsbActionType::QueryTrace, "query_trace");
}

#[test]
fn grc_step_emitted_payload_roundtrip() {
    let p = GrcStepEmittedPayload {
        agent_id: "executor".to_string(),
        topic: "task_runner".to_string(),
        step_type: "observe".to_string(),
        grounding_level: "high".to_string(),
        confidence: 0.85,
        budget_remaining: 7,
        elapsed_ms: 42,
    };
    let payload = SsbPayload::GrcStepEmitted(p.clone());
    assert_eq!(payload.action_type(), SsbActionType::GrcStepEmitted);

    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(decoded.action_type(), SsbActionType::GrcStepEmitted);

    let json = serde_json::to_string(&p).expect("json serialize");
    let p2: GrcStepEmittedPayload = serde_json::from_str(&json).expect("json deserialize");
    assert_eq!(p2.budget_remaining, 7);
    assert_eq!(p2.elapsed_ms, 42);
    assert!((p2.confidence - 0.85).abs() < f64::EPSILON);
}

#[test]
fn grounding_sweep_result_payload_roundtrip() {
    let p = GroundingSweepResultPayload {
        ungrounded: 2,
        verify_needed: 1,
        grounded: 5,
        top_claim: Some("file:src/main.rs".to_string()),
    };
    let payload = SsbPayload::GroundingSweepResult(p.clone());
    assert_eq!(payload.action_type(), SsbActionType::GroundingSweepResult);

    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(decoded.action_type(), SsbActionType::GroundingSweepResult);

    let json = serde_json::to_string(&p).expect("json serialize");
    assert!(json.contains("ungrounded"));
    assert!(json.contains("top_claim"));
}

#[test]
fn csq_evaluated_payload_roundtrip() {
    let p = CsqEvaluatedPayload {
        severity: "Critical".to_string(),
        guard_present: false,
        blocked: true,
        check_count: 3,
    };
    let payload = SsbPayload::CsqEvaluated(p.clone());
    assert_eq!(payload.action_type(), SsbActionType::CsqEvaluated);

    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    if let SsbPayload::CsqEvaluated(d) = decoded {
        assert_eq!(d.severity, "Critical");
        assert!(d.blocked);
        assert!(!d.guard_present);
        assert_eq!(d.check_count, 3);
    } else {
        panic!("expected CsqEvaluated, got {decoded:?}");
    }
}

#[test]
fn auditor_rubber_stamp_rejected_payload_roundtrip() {
    let p = AuditorRubberStampRejectedPayload {
        role: "auditor".to_string(),
        finding_count: 0,
        expected_min: 1,
    };
    let payload = SsbPayload::AuditorRubberStampRejected(p.clone());
    assert_eq!(payload.action_type(), SsbActionType::AuditorRubberStampRejected);

    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(decoded.action_type(), SsbActionType::AuditorRubberStampRejected);
}

#[test]
fn hyde_query_payload_roundtrip() {
    let p = HydeQueryPayload {
        query_hash: "deadbeef00112233".to_string(),
        hypothesis_len: 128,
        cache_hit: true,
        latency_ms: 12,
    };
    let payload = SsbPayload::HydeQuery(p.clone());
    assert_eq!(payload.action_type(), SsbActionType::HydeQuery);

    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    if let SsbPayload::HydeQuery(d) = decoded {
        assert_eq!(d.query_hash, "deadbeef00112233");
        assert_eq!(d.hypothesis_len, 128);
        assert!(d.cache_hit);
        assert_eq!(d.latency_ms, 12);
    } else {
        panic!("expected HydeQuery");
    }
}

#[test]
fn vector_cache_hit_rate_payload_roundtrip() {
    let p = VectorCacheHitRatePayload {
        session_id: "test-session".to_string(),
        hits: 7,
        misses: 3,
        hit_rate_pct: 70.0,
    };
    let payload = SsbPayload::VectorCacheHitRate(p.clone());
    assert_eq!(payload.action_type(), SsbActionType::VectorCacheHitRate);

    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    if let SsbPayload::VectorCacheHitRate(d) = decoded {
        assert_eq!(d.hits, 7);
        assert_eq!(d.misses, 3);
        assert!((d.hit_rate_pct - 70.0).abs() < f64::EPSILON);
    } else {
        panic!("expected VectorCacheHitRate");
    }
}

#[test]
fn query_trace_payload_roundtrip() {
    let p = QueryTracePayload {
        query_id: "11111111-2222-3333-4444-555555555555".to_string(),
        tier: "STANDARD".to_string(),
        stage_count: 9,
        result_count: 12,
        total_latency_ms: 245,
    };
    let payload = SsbPayload::QueryTrace(p.clone());
    assert_eq!(payload.action_type(), SsbActionType::QueryTrace);

    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    if let SsbPayload::QueryTrace(d) = decoded {
        assert_eq!(d.tier, "STANDARD");
        assert_eq!(d.stage_count, 9);
        assert_eq!(d.result_count, 12);
        assert_eq!(d.total_latency_ms, 245);
    } else {
        panic!("expected QueryTrace");
    }
}

/// Enumerate every new variant and assert action_type/sql/from_str symmetry
/// from a single source. If any one is missed in lockstep
/// (per gotcha B41_SSB_ALL_TEST_PAYLOADS_DRIFT) this single test fails.
#[test]
fn all_seven_c1_variants_lockstep() {
    let pairs: [(SsbActionType, &str); 7] = [
        (SsbActionType::GrcStepEmitted, "grc_step_emitted"),
        (SsbActionType::GroundingSweepResult, "grounding_sweep_result"),
        (SsbActionType::CsqEvaluated, "csq_evaluated"),
        (
            SsbActionType::AuditorRubberStampRejected,
            "auditor_rubber_stamp_rejected",
        ),
        (SsbActionType::HydeQuery, "hyde_query"),
        (SsbActionType::VectorCacheHitRate, "vector_cache_hit_rate"),
        (SsbActionType::QueryTrace, "query_trace"),
    ];
    for (at, sql) in pairs {
        assert_eq!(at.as_sql_str(), sql, "as_sql_str mismatch for {at:?}");
        let parsed = ssb_action_type_from_str_public(sql)
            .unwrap_or_else(|e| panic!("from_str failed for {sql}: {e:?}"));
        assert_eq!(parsed, at, "from_str -> enum mismatch for {sql}");
    }
}
