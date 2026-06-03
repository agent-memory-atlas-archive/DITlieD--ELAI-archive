//! F-13 SessionArchived SSB round-trip contract tests.
//!
//! Validates the session-archive-on-schema-bump beacon (SCHEMA_VERSION 206):
//!   1. `SessionArchivedPayload` survives the EXACT on-wire encoder
//!      `Ssb::write_payload` uses (`rmp_serde::to_vec_named`) -> `from_slice`,
//!      with all fields intact (session_id / stale_version / archive_path /
//!      manifest_blake3 / store_kind).
//!   2. The wrapped payload reports `action_type() == SessionArchived` and
//!      sql_str `"session_archived"`.
//!   3. `ssb_action_type_from_str_public("session_archived")` maps back to the
//!      flat-beacon variant and its sql_str round-trips stably.
//!   4. The full DB path: write through `Ssb::write_payload`, recover via
//!      `query_by_type`, decode the stored bytes the way the dashboard/CLI
//!      readers do, and assert every field survives.
//!
//! POLICY note: SessionArchived is a forensic best-effort beacon emitted by the
//! migration when `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE` is set; these tests pin the
//! wire shape only. The migration emission path is exercised by the smoke test.

use std::sync::Arc;

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
    SessionArchivedPayload,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_ssb(tmp: &TempDir, project: &str, session: &str) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: project.into(),
        session_id: session.into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(elai_core::clock::OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("ssb new"))
}

/// A representative beacon payload. `manifest_blake3` is a blake3 hex-prefix of
/// the archived bytes (blake3-only per HARD RULE). The strings are chosen to
/// expose any field-reordering / truncation bug on the wire.
fn sample_session_archived_payload() -> SessionArchivedPayload {
    let manifest_blake3 = {
        let h = blake3::hash(b"contract_session_archive|ssb-v205.sqlite|fixture-bytes");
        format!("{:.16}", h.to_hex())
    };
    SessionArchivedPayload {
        session_id: "f13-contract-session".to_string(),
        stale_version: 205,
        archive_path: "workflow/sessions/f13-contract-session/archive/ssb-v205.sqlite".to_string(),
        manifest_blake3,
        store_kind: "ssb".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Contract 1: on-wire encoder round-trip (rmp_serde::to_vec_named)
// ---------------------------------------------------------------------------

#[test]
fn contract_session_archived_wire_encoder_roundtrip() {
    let original = sample_session_archived_payload();
    let wrapped = SsbPayload::SessionArchived(original.clone());

    assert_eq!(
        wrapped.action_type(),
        SsbActionType::SessionArchived,
        "SessionArchived payload action_type must be SessionArchived"
    );
    assert_eq!(
        wrapped.action_type().as_sql_str(),
        "session_archived",
        "SessionArchived sql_str must be session_archived"
    );

    // The on-wire encoder: this is the exact call inside Ssb::write_payload.
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("encode SessionArchived on-wire");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("decode SessionArchived from wire bytes");

    match decoded {
        SsbPayload::SessionArchived(p) => {
            assert_eq!(p.session_id, original.session_id, "session_id must round-trip");
            assert_eq!(
                p.stale_version, original.stale_version,
                "stale_version must round-trip"
            );
            assert_eq!(
                p.archive_path, original.archive_path,
                "archive_path must round-trip"
            );
            assert_eq!(
                p.manifest_blake3, original.manifest_blake3,
                "manifest_blake3 must round-trip"
            );
            assert_eq!(p.store_kind, original.store_kind, "store_kind must round-trip");
        }
        other => panic!("expected SessionArchived, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 2: ssb_action_type_from_str_public recognises the wire tag
// ---------------------------------------------------------------------------

#[test]
fn contract_session_archived_str_roundtrip() {
    let at = ssb_action_type_from_str_public("session_archived")
        .expect("session_archived must parse");
    assert_eq!(
        at,
        SsbActionType::SessionArchived,
        "session_archived must map to SsbActionType::SessionArchived"
    );
    assert_eq!(
        at.as_sql_str(),
        "session_archived",
        "as_sql_str round-trip must be stable"
    );
}

// ---------------------------------------------------------------------------
// Contract 3: full DB path via Ssb::write_payload + query_by_type
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_session_archived_db_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "f13-test", "f13-archive-rt");

    let original = sample_session_archived_payload();
    ssb.write_payload(
        "ssb-migration",
        0,
        SsbPayload::SessionArchived(original.clone()),
        Vec::new(),
    )
    .await
    .expect("write SessionArchived payload");

    let rows = ssb
        .query_by_type(SsbActionType::SessionArchived)
        .expect("query SessionArchived rows");
    assert_eq!(rows.len(), 1, "exactly one SessionArchived row in DB");

    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode SessionArchived from DB");
    match decoded {
        SsbPayload::SessionArchived(p) => {
            assert_eq!(p.session_id, original.session_id);
            assert_eq!(p.stale_version, original.stale_version);
            assert_eq!(p.archive_path, original.archive_path);
            assert_eq!(p.manifest_blake3, original.manifest_blake3);
            assert_eq!(p.store_kind, original.store_kind);
        }
        other => panic!("expected SessionArchived, got {other:?}"),
    }
}
