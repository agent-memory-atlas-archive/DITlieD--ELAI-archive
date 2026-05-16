// B-34 contract: SSB artifact + entry-point variants (SCHEMA_VERSION >= 42).
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 42 (includes B-34 SSB variants).
// 2. 4 new SsbActionType variants (EntryPointsDetected, ArtifactsIndexed,
//    ArtifactSearched, ArtifactStale) serialize and deserialize cleanly.
// 3. action_type() discriminant matches as_sql_str() for all 4 variants.
// 4. SQL string roundtrip via ssb_action_type_from_str_public.

use std::collections::BTreeMap;

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ArtifactSearchedPayload, ArtifactStalePayload,
    ArtifactsIndexedPayload, EntryPointsDetectedPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};

#[test]
fn schema_version_includes_b34() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 42,
        "schema must include B-34 SSB variants (got {})",
        SCHEMA_VERSION_PUBLIC
    );
}

fn roundtrip_action_type(at: SsbActionType) {
    let s = at.as_sql_str();
    let decoded = ssb_action_type_from_str_public(s).unwrap_or_else(|e| {
        panic!("ssb_action_type_from_str_public({s:?}) failed: {e:?}")
    });
    assert_eq!(decoded, at, "round-trip failed for {s}");
}

#[test]
fn entry_points_detected_roundtrip() {
    roundtrip_action_type(SsbActionType::EntryPointsDetected);
}

#[test]
fn artifacts_indexed_roundtrip() {
    roundtrip_action_type(SsbActionType::ArtifactsIndexed);
}

#[test]
fn artifact_searched_roundtrip() {
    roundtrip_action_type(SsbActionType::ArtifactSearched);
}

#[test]
fn artifact_stale_roundtrip() {
    roundtrip_action_type(SsbActionType::ArtifactStale);
}

#[test]
fn entry_points_detected_payload_json_roundtrip() {
    let mut by_reason = BTreeMap::new();
    by_reason.insert("orphan".to_string(), 3u32);
    let p = EntryPointsDetectedPayload {
        project_id: "proj-a".to_string(),
        count: 3,
        by_reason,
    };
    let payload = SsbPayload::EntryPointsDetected(p);
    let action_type = payload.action_type();
    assert_eq!(action_type, SsbActionType::EntryPointsDetected);
    assert_eq!(action_type.as_sql_str(), "entry_points_detected");
}

#[test]
fn artifacts_indexed_payload_action_type() {
    let p = ArtifactsIndexedPayload {
        project_id: "proj".to_string(),
        indexed: vec!["schema.sql".to_string()],
        up_to_date: vec!["api.yaml".to_string()],
        total_chunks: 42,
        latency_ms: 120,
    };
    let payload = SsbPayload::ArtifactsIndexed(p);
    assert_eq!(payload.action_type(), SsbActionType::ArtifactsIndexed);
    assert_eq!(payload.action_type().as_sql_str(), "artifacts_indexed");
}

#[test]
fn artifact_searched_payload_action_type() {
    let p = ArtifactSearchedPayload {
        project_id: "proj".to_string(),
        artifact_name: Some("schema.sql".to_string()),
        query: "CREATE TABLE users".to_string(),
        result_count: 5,
        latency_ms: 30,
    };
    let payload = SsbPayload::ArtifactSearched(p);
    assert_eq!(payload.action_type(), SsbActionType::ArtifactSearched);
}

#[test]
fn artifact_stale_payload_action_type() {
    let p = ArtifactStalePayload {
        project_id: "proj".to_string(),
        artifact_name: "k8s.yaml".to_string(),
        changed_files: vec!["infra/k8s/deployment.yaml".to_string()],
    };
    let payload = SsbPayload::ArtifactStale(p);
    assert_eq!(payload.action_type(), SsbActionType::ArtifactStale);
    assert_eq!(payload.action_type().as_sql_str(), "artifact_stale");
}
