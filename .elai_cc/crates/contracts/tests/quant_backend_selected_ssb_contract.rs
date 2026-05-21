// E-27 contract: 3 new SSB variants for quantization-backend beacons.
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 89 (atomic bump for all 3 E-27 variants).
// 2. All 3 new SsbActionType variants round-trip via as_sql_str /
//    ssb_action_type_from_str_public.
// 3. action_type() on matching SsbPayload variant returns correct SsbActionType.
// 4. Each payload struct serializes via msgpack (rmp_serde) without loss.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, AwqCheckpointBuiltPayload, KvCacheDtypeSoftwareSimPayload,
    QuantBackendSelectedPayload, SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};

const E27_SCHEMA_FLOOR: i64 = 89;

#[test]
fn schema_version_includes_e27_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= E27_SCHEMA_FLOOR,
        "schema must include E-27 SSB variants (got {SCHEMA_VERSION_PUBLIC}, floor {E27_SCHEMA_FLOOR})"
    );
}

fn roundtrip_action_type(at: SsbActionType, expected_sql: &str) {
    let s = at.as_sql_str();
    assert_eq!(s, expected_sql, "as_sql_str mismatch");
    let decoded = ssb_action_type_from_str_public(s)
        .unwrap_or_else(|e| panic!("from_str failed: {e:?}"));
    assert_eq!(decoded, at, "round-trip failed for {s}");
}

#[test]
fn quant_backend_selected_action_type_roundtrip() {
    roundtrip_action_type(SsbActionType::QuantBackendSelected, "quant_backend_selected");
}

#[test]
fn kv_cache_dtype_software_sim_action_type_roundtrip() {
    roundtrip_action_type(SsbActionType::KvCacheDtypeSoftwareSim, "kv_cache_dtype_software_sim");
}

#[test]
fn awq_checkpoint_built_action_type_roundtrip() {
    roundtrip_action_type(SsbActionType::AwqCheckpointBuilt, "awq_checkpoint_built");
}

#[test]
fn quant_backend_selected_payload_bijection() {
    let p = SsbPayload::QuantBackendSelected(QuantBackendSelectedPayload {
        backend: "awq".to_string(),
        kv_dtype: "fp8_e5m2".to_string(),
        compute_capability: 86,
        marlin_enabled: true,
    });
    assert_eq!(p.action_type(), SsbActionType::QuantBackendSelected);
}

#[test]
fn kv_cache_dtype_software_sim_payload_bijection() {
    let p = SsbPayload::KvCacheDtypeSoftwareSim(KvCacheDtypeSoftwareSimPayload {
        kv_dtype: "fp8_e5m2".to_string(),
        throughput_cost_pct: 5.0,
        compute_capability: 86,
    });
    assert_eq!(p.action_type(), SsbActionType::KvCacheDtypeSoftwareSim);
}

#[test]
fn awq_checkpoint_built_payload_bijection() {
    let p = SsbPayload::AwqCheckpointBuilt(AwqCheckpointBuiltPayload {
        model_path: "/models/Qwen2.5-Coder-7B".to_string(),
        calibration_size: 256,
        duration_s: 7200.0,
        output_path: "/models/Qwen2.5-Coder-7B-AWQ".to_string(),
    });
    assert_eq!(p.action_type(), SsbActionType::AwqCheckpointBuilt);
}

#[test]
fn quant_backend_selected_msgpack_roundtrip() {
    let original = QuantBackendSelectedPayload {
        backend: "awq".to_string(),
        kv_dtype: "fp8_e5m2".to_string(),
        compute_capability: 86,
        marlin_enabled: true,
    };
    let bytes = rmp_serde::to_vec_named(&original).expect("serialize");
    let decoded: QuantBackendSelectedPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    assert_eq!(original, decoded);
}

#[test]
fn kv_cache_dtype_software_sim_msgpack_roundtrip() {
    let original = KvCacheDtypeSoftwareSimPayload {
        kv_dtype: "fp8_e4m3".to_string(),
        throughput_cost_pct: 5.0,
        compute_capability: 86,
    };
    let bytes = rmp_serde::to_vec_named(&original).expect("serialize");
    let decoded: KvCacheDtypeSoftwareSimPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    assert_eq!(original, decoded);
}

#[test]
fn awq_checkpoint_built_msgpack_roundtrip() {
    let original = AwqCheckpointBuiltPayload {
        model_path: "/models/Qwen2.5-Coder-7B".to_string(),
        calibration_size: 256,
        duration_s: 7200.0,
        output_path: "/models/Qwen2.5-Coder-7B-AWQ".to_string(),
    };
    let bytes = rmp_serde::to_vec_named(&original).expect("serialize");
    let decoded: AwqCheckpointBuiltPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    assert_eq!(original, decoded);
}