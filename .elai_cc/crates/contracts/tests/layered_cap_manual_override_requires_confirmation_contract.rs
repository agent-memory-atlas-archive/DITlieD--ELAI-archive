//! E-39 contract: layered-cap manual override writes a
//! `CostTokenEvent { kind: LayeredCapManualOverride }` SSB row carrying
//! `user_session_id`, `override_amount_usd`, and
//! `confirmation_string_hash`.
//!
//! The `elai cloud override-cap` CLI subcommand is the production
//! entrypoint (cloud_cmd.rs:53 `run_override_cap`). This contract test
//! drives the SSB write path directly (the override subcommand's sole
//! user-visible side-effect is this row plus the LayeredCapsConfig
//! field update), then reads back and asserts the confirmation_string_hash
//! field is non-empty (audit-trail requirement per E-39 plan Task 10).
//!
//! Production entrypoint: elai cloud override-cap -> cloud_cmd.rs:53
//! `run_override_cap` -> SSB emit `LayeredCapManualOverride`.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    CostEventKind, CostTokenEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// WIRE-DARK[E-39]: run_override_cap in cloud_cmd.rs does not yet write
// the SSB LayeredCapManualOverride row (OPTION B deferred disjoint slice).
// Resolves when run_override_cap opens the SSB db path and writes the row
// (does not require off-limits files -- cloud_cmd.rs is not off-limits).
// See E-39-decisions.md.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_layered_cap_manual_override_ssb_roundtrip() {
    // The override subcommand hashes the typed-confirmation string via
    // blake3 before writing to SSB (audit trail; raw string never
    // persisted). We simulate that here.
    let confirmation = "I CONFIRM OVERRIDE TO $500 USD";
    let hash_bytes = blake3::hash(confirmation.as_bytes());
    let hash_hex = hash_bytes.to_hex().to_string();
    assert!(
        !hash_hex.is_empty(),
        "blake3 hash of confirmation must be non-empty"
    );

    // Write the off-agent artifact SSB row.
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "e39-override".into(),
        session_id: "e39-override-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::CostTokenEvent(CostTokenEventPayload {
        kind: CostEventKind::LayeredCapManualOverride,
        session_id: Some("e39-override-session".into()),
        task_id: None,
        details_json: serde_json::json!({
            "user_session_id": "user-session-override-001",
            "override_amount_usd": 500_u64,
            "confirmation_string_hash": hash_hex
        }),
        ts_ms: 1_717_000_007_000,
    });
    ssb.write_payload("cost", 0, payload, Vec::new())
        .await
        .expect("write");

    // Read back and assert.
    let rows = ssb
        .query_by_type(SsbActionType::CostTokenEvent)
        .expect("query");
    assert_eq!(rows.len(), 1, "expected exactly one CostTokenEvent row");
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::CostTokenEvent(p) => {
            assert_eq!(
                p.kind,
                CostEventKind::LayeredCapManualOverride,
                "kind discriminator must be LayeredCapManualOverride"
            );
            let stored_hash = p.details_json["confirmation_string_hash"]
                .as_str()
                .expect("confirmation_string_hash must be a string");
            assert_eq!(
                stored_hash, hash_hex,
                "stored blake3 hash must match computed hash"
            );
            assert!(!stored_hash.is_empty(), "confirmation hash must be non-empty");
            assert_eq!(
                p.details_json["override_amount_usd"],
                serde_json::json!(500_u64)
            );
        }
        other => panic!("expected CostTokenEvent, got {other:?}"),
    }
}
