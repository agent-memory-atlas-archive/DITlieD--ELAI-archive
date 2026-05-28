//! E-26 contract: 3-variant inference-engines 2026 SSB pack.
//!
//! Verifies (atomic SCHEMA bump 148 -> 151, +3 SsbActionType variants):
//! 1. `SCHEMA_VERSION_PUBLIC >= 151` (E-26 floor; the 3 variants
//!    landed at this bump).
//! 2. `SsbActionType::SglangBackendRegistered` round-trips through
//!    `as_sql_str` / `ssb_action_type_from_str_public` + payload
//!    encode/decode (msgpack).
//! 3. Same for `SsbActionType::XgrammarSchemaCompiled`.
//! 4. Same for `SsbActionType::RuntimeColdStartObserved`.
//! 5. The wire tags are the canonical snake_case names expected by
//!    the orchestrator's `action_type_lock.golden` body.
//! 6. The 3 new variants are DISTINCT from E-35's 10 structured-output
//!    variants (no enum-discriminant collision).

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, RuntimeColdStartObservedPayload,
    SglangBackendRegisteredPayload, SsbActionType, SsbPayload, XgrammarSchemaCompiledPayload,
    SCHEMA_VERSION_PUBLIC,
};

/// Floor for E-26: head SCHEMA_VERSION at completion-drive close.
/// Atomic +3 bump from E-3's 148 head (the 3 new variants
/// SglangBackendRegistered + XgrammarSchemaCompiled +
/// RuntimeColdStartObserved; see E-26-decisions.md).
const E26_SCHEMA_FLOOR: i64 = 151;

#[test]
fn schema_version_at_or_above_e26_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= E26_SCHEMA_FLOOR,
        "SCHEMA_VERSION_PUBLIC={SCHEMA_VERSION_PUBLIC} but E-26 floor is {E26_SCHEMA_FLOOR}"
    );
}

#[test]
fn sglang_backend_registered_sql_tag_round_trips() {
    let variant = SsbActionType::SglangBackendRegistered;
    assert_eq!(variant.as_sql_str(), "sglang_backend_registered");
    let parsed = ssb_action_type_from_str_public("sglang_backend_registered")
        .expect("parse sglang_backend_registered");
    assert_eq!(parsed, variant);
}

#[test]
fn xgrammar_schema_compiled_sql_tag_round_trips() {
    let variant = SsbActionType::XgrammarSchemaCompiled;
    assert_eq!(variant.as_sql_str(), "xgrammar_schema_compiled");
    let parsed = ssb_action_type_from_str_public("xgrammar_schema_compiled")
        .expect("parse xgrammar_schema_compiled");
    assert_eq!(parsed, variant);
}

#[test]
fn runtime_cold_start_observed_sql_tag_round_trips() {
    let variant = SsbActionType::RuntimeColdStartObserved;
    assert_eq!(variant.as_sql_str(), "runtime_cold_start_observed");
    let parsed = ssb_action_type_from_str_public("runtime_cold_start_observed")
        .expect("parse runtime_cold_start_observed");
    assert_eq!(parsed, variant);
}

#[test]
fn sglang_backend_registered_payload_msgpack_round_trip() {
    let payload = SglangBackendRegisteredPayload {
        base_url_blake3: "a".repeat(64),
        structured_output_backend: "xgrammar".into(),
        ts_ms: 1_700_000_148_001,
    };
    let wrapped = SsbPayload::SglangBackendRegistered(payload.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack decode");
    match decoded {
        SsbPayload::SglangBackendRegistered(p) => {
            assert_eq!(p, payload);
        }
        other => panic!("expected SglangBackendRegistered, got {other:?}"),
    }
}

#[test]
fn xgrammar_schema_compiled_payload_msgpack_round_trip() {
    let payload = XgrammarSchemaCompiledPayload {
        schema_id_hash: 0xE026_0002_0000_0000,
        schema_hash: "b".repeat(64),
        compile_ms: 47,
        ts_ms: 1_700_000_148_002,
    };
    let wrapped = SsbPayload::XgrammarSchemaCompiled(payload.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack decode");
    match decoded {
        SsbPayload::XgrammarSchemaCompiled(p) => {
            assert_eq!(p, payload);
        }
        other => panic!("expected XgrammarSchemaCompiled, got {other:?}"),
    }
}

#[test]
fn runtime_cold_start_observed_payload_msgpack_round_trip() {
    let payload = RuntimeColdStartObservedPayload {
        backend_name_hash: 0xE026_0003_0000_0000,
        elapsed_ms: 8_120,
        ts_ms: 1_700_000_148_003,
    };
    let wrapped = SsbPayload::RuntimeColdStartObserved(payload);
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack decode");
    match decoded {
        SsbPayload::RuntimeColdStartObserved(p) => {
            assert_eq!(p.backend_name_hash, payload.backend_name_hash);
            assert_eq!(p.elapsed_ms, payload.elapsed_ms);
            assert_eq!(p.ts_ms, payload.ts_ms);
        }
        other => panic!("expected RuntimeColdStartObserved, got {other:?}"),
    }
}

#[test]
fn payload_action_type_projection_matches_variant() {
    let sg = SsbPayload::SglangBackendRegistered(SglangBackendRegisteredPayload {
        base_url_blake3: "0".repeat(64),
        structured_output_backend: "xgrammar".into(),
        ts_ms: 1,
    });
    assert_eq!(sg.action_type(), SsbActionType::SglangBackendRegistered);

    let xc = SsbPayload::XgrammarSchemaCompiled(XgrammarSchemaCompiledPayload {
        schema_id_hash: 0,
        schema_hash: "0".repeat(64),
        compile_ms: 0,
        ts_ms: 0,
    });
    assert_eq!(xc.action_type(), SsbActionType::XgrammarSchemaCompiled);

    let cs = SsbPayload::RuntimeColdStartObserved(RuntimeColdStartObservedPayload {
        backend_name_hash: 0,
        elapsed_ms: 0,
        ts_ms: 0,
    });
    assert_eq!(cs.action_type(), SsbActionType::RuntimeColdStartObserved);
}

#[test]
fn e26_variants_distinct_from_e35_variants() {
    // Sanity: the 3 new variants are NOT aliases of the 10 E-35 variants.
    // This is a discriminant-distinctness check — Rust enum equality.
    let e26 = [
        SsbActionType::SglangBackendRegistered,
        SsbActionType::XgrammarSchemaCompiled,
        SsbActionType::RuntimeColdStartObserved,
    ];
    let e35 = [
        SsbActionType::XgrammarMaskApplied,
        SsbActionType::InstructorModeActivated,
        SsbActionType::GrammarModeActivated,
        SsbActionType::CodeactActionExecuted,
        SsbActionType::SearchReplaceTier1Hit,
        SsbActionType::SearchReplaceCollisionFallback,
        SsbActionType::CloudStrictSchemaApplied,
        SsbActionType::GrammarCacheLoaded,
        SsbActionType::StructuredEngineRejected,
        SsbActionType::LegacyJsonObjectRefused,
    ];
    for new_v in e26 {
        for old_v in e35 {
            assert_ne!(
                new_v, old_v,
                "E-26 variant {new_v:?} collides with E-35 variant {old_v:?}"
            );
            assert_ne!(
                new_v.as_sql_str(),
                old_v.as_sql_str(),
                "E-26 wire tag collides with E-35"
            );
        }
    }
}
