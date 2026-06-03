//! F-12 contract (W-9 — SSB reuse, NO new action_type, NO schema bump):
//!
//! The `semantic_contradiction` signal flows through the EXISTING
//! `MemoryConflictDetected` action_type with `conflict_kind =
//! "semantic_contradiction"` (a FREE-FORM string field — ssb.rs docstring
//! "future kinds can land without a schema bump"). Asserts:
//!   - `MemoryConflictDetected.as_sql_str() == "memory_conflict_detected"`
//!   - the existing parse arm round-trips
//!   - a payload carrying `conflict_kind="semantic_contradiction"`
//!     round-trips through rmp (msgpack) unchanged
//!   - `action_type()` resolves to `MemoryConflictDetected` (NOT a new
//!     variant)
//!   - a real `Ssb` accepts the row (CHECK constraint admits
//!     "memory_conflict_detected")
//!   - NO `SemanticContradictionDetected` variant exists in ssb.rs source
//!     (the negative grep — assert no stray action_type snuck in)

use std::path::Path;

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, MemoryConflictDetectedPayload, SsbActionType, SsbPayload,
};
use elai_orchestrator::{Ssb, SsbConfig};

const SEMANTIC_KIND: &str = "semantic_contradiction";

#[test]
fn memory_conflict_sql_str_is_stable() {
    assert_eq!(
        SsbActionType::MemoryConflictDetected.as_sql_str(),
        "memory_conflict_detected"
    );
}

#[test]
fn memory_conflict_parses_back() {
    let parsed = ssb_action_type_from_str_public("memory_conflict_detected")
        .expect("memory_conflict_detected must parse");
    assert_eq!(parsed, SsbActionType::MemoryConflictDetected);
}

#[test]
fn semantic_contradiction_kind_roundtrips_through_rmp() {
    let payload = MemoryConflictDetectedPayload {
        fact_id_a: "older-fact".to_string(),
        fact_id_b: "newer-fact".to_string(),
        conflict_kind: SEMANTIC_KIND.to_string(),
    };
    let wrapped = SsbPayload::MemoryConflictDetected(payload);
    // The reused variant — NOT a new action_type.
    assert_eq!(
        wrapped.action_type(),
        SsbActionType::MemoryConflictDetected,
        "semantic contradiction must reuse MemoryConflictDetected, not a new variant"
    );

    let bytes = rmp_serde::to_vec_named(&wrapped).expect("encode");
    let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    match back {
        SsbPayload::MemoryConflictDetected(p) => {
            assert_eq!(p.fact_id_a, "older-fact");
            assert_eq!(p.fact_id_b, "newer-fact");
            assert_eq!(
                p.conflict_kind, SEMANTIC_KIND,
                "free-form conflict_kind must round-trip unchanged"
            );
        }
        other => panic!("expected MemoryConflictDetected, got {other:?}"),
    }
}

#[test]
fn no_stray_semantic_contradiction_action_type_in_source() {
    // Negative grep over the always-compiled ssb.rs source: there must be
    // NO new SsbActionType variant for semantic contradiction. The signal
    // reuses MemoryConflictDetected via the free-form conflict_kind.
    let ssb_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("orchestrator")
        .join("src")
        .join("ssb.rs");
    let src = std::fs::read_to_string(&ssb_path)
        .unwrap_or_else(|e| panic!("read ssb.rs failed: {e}"));
    assert!(
        src.len() > 1000,
        "ssb.rs scan reached an empty/short file — dead evidence path"
    );
    assert!(
        !src.contains("SemanticContradictionDetected"),
        "no SemanticContradictionDetected action_type may exist — reuse MemoryConflictDetected"
    );
    assert!(
        !src.contains("\"semantic_contradiction_detected\""),
        "no semantic_contradiction_detected sql_str may exist — reuse memory_conflict_detected"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ssb_check_constraint_accepts_semantic_contradiction_row() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: None,
        project_id: "contract".to_string(),
        session_id: "semantic-contradiction-reuse".to_string(),
    };
    let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
        std::sync::Arc::new(elai_core::OsClock);
    let ssb = Ssb::new(cfg, clock).expect("ssb init");

    let payload = SsbPayload::MemoryConflictDetected(MemoryConflictDetectedPayload {
        fact_id_a: "older-fact".to_string(),
        fact_id_b: "newer-fact".to_string(),
        conflict_kind: SEMANTIC_KIND.to_string(),
    });
    let seq = ssb
        .write_payload("contract:f12", 0, payload, Vec::new())
        .await
        .expect("CHECK constraint must accept 'memory_conflict_detected'");
    assert!(seq >= 0);
}
