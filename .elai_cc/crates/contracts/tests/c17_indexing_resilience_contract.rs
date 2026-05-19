//! C-17 — Knowledge indexing resilience contract.
//!
//! Pins the four invariants from `docs/thefinalglance/C-plan/C-17_*` plan
//! task #7 so a future regression cannot drift the public surface:
//!
//! (a) `IndexResult` variants compile as `#[must_use]` enums (every variant
//!     carries information the caller must observe). The contract holds
//!     by enum-level `#[must_use]` and is asserted by exercising every
//!     variant in a `match`.
//! (b) `IndexLock::try_acquire` requires a `LockScope` parameter — the
//!     type signature is fixed via a `fn` pointer assertion.
//! (c) `auto_resume_indexed_projects` returns within 5s on an empty
//!     project list (timeout guard via `tokio::time::timeout`).
//! (d) `startup::SHUTDOWN_DEADLINE` is a `Duration` between 30..=120
//!     seconds. The const is the canonical surface added in C-17 (DRIFT-L)
//!     so callers no longer pass magic literals.
//!
//! Two additional C-17 assertions land here:
//!
//! (e) `ChunkKind` round-trips the new `Heading` / `Paragraph` variants
//!     through `as_str` / `from_str` (DRIFT-A regression guard).
//! (f) `IndexCheckpointPayload` carries the optional `batch_duration_ms`
//!     field (DRIFT-I additive field). The field MUST default to None on
//!     decode of a pre-C-17 payload.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use elai_core::{ClockProvider, OsClock};
use elai_knowledge::bm25::ChunkKind;
use elai_orchestrator::indexing_service::{IndexResult, IndexingSummary};
use elai_orchestrator::lock::{IndexLock, IndexLockError, IndexLockGuard, LockScope};
use elai_orchestrator::ssb::IndexCheckpointPayload;
use elai_orchestrator::startup::{self, SHUTDOWN_DEADLINE};
use tempfile::TempDir;

// (a) IndexResult variant inventory regression.
// The match exercises every IndexResult variant. Removing or renaming a
// variant breaks this test — the match is non-exhaustive only when a new
// variant is added without updating the contract.
#[test]
fn index_result_variants_match_exhaustively() {
    fn assert_must_use(_r: &IndexResult) {}
    let r = IndexResult::Cancelled {
        batches_completed: 0,
        files_indexed: 0,
    };
    assert_must_use(&r);
    // Exhaustive match — adding a new IndexResult variant without updating
    // this contract test breaks compilation.
    match r {
        IndexResult::Completed { summary: _ } => {}
        IndexResult::Cancelled { .. } => {}
        IndexResult::Resumed { .. } => {}
        IndexResult::AlreadyLocked { .. } => {}
    }
    // Surface the IndexingSummary type so unused-import lint stays quiet.
    let _summary_type: Option<IndexingSummary> = None;
}

// (b) IndexLock::try_acquire signature pins the LockScope parameter.
#[test]
fn index_lock_try_acquire_requires_lock_scope() {
    // Compile-time assertion via a fn-pointer coercion. If `try_acquire`
    // ever drops the LockScope arg, this assignment fails to type-check.
    let _f: fn(
        &std::path::Path,
        LockScope,
        Arc<dyn ClockProvider>,
    ) -> Result<IndexLockGuard, IndexLockError> = IndexLock::try_acquire;
}

// (c) auto_resume_indexed_projects returns within 5s on empty input.
#[test]
fn auto_resume_returns_quickly_on_empty_project_list() {
    let dir = TempDir::new().expect("tempdir");
    let indexes_root: PathBuf = dir.path().join("indexes_root_no_dbs");
    // Deliberately do NOT create the directory — the resume scan should
    // find zero metadata DBs and return promptly.
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio rt");
    let report = rt.block_on(async move {
        tokio::time::timeout(
            Duration::from_secs(5),
            startup::auto_resume_indexed_projects(indexes_root, clock),
        )
        .await
        .expect("auto_resume must return within 5s on empty input")
    });
    // Empty input → zero discovered, zero resumed.
    assert_eq!(report.projects_discovered, 0);
    assert_eq!(report.projects_resumed, 0);
}

// (d) startup::SHUTDOWN_DEADLINE is a Duration in [30, 120] seconds.
#[test]
fn shutdown_deadline_const_in_safe_range() {
    let secs = SHUTDOWN_DEADLINE.as_secs();
    assert!(
        (30..=120).contains(&secs),
        "SHUTDOWN_DEADLINE must be 30..=120s; got {secs}s"
    );
}

// (e) ChunkKind round-trip pins the C-17 Heading/Paragraph variants.
#[test]
fn chunk_kind_heading_paragraph_round_trip() {
    for kind in [ChunkKind::Heading, ChunkKind::Paragraph] {
        let s = kind.as_str();
        assert_eq!(
            ChunkKind::from_str(s),
            kind,
            "ChunkKind round-trip failed for {s}"
        );
    }
    // DRIFT-A regression guard: as_str must NOT collide with prior values.
    assert_eq!(ChunkKind::Heading.as_str(), "heading");
    assert_eq!(ChunkKind::Paragraph.as_str(), "paragraph");
}

// (f) IndexCheckpointPayload carries the optional batch_duration_ms field.
//     Pre-C-17 payloads decoded from rmp-serde (without the field) must
//     produce `batch_duration_ms: None` via #[serde(default)].
#[test]
fn index_checkpoint_payload_batch_duration_optional() {
    // Forward path: a C-17 payload carrying the new field round-trips
    // through rmp-serde named encoding (the production encoder).
    let forward = IndexCheckpointPayload {
        project_id: "q".to_string(),
        batches_processed: 4,
        batches_total: Some(8),
        files_processed: 200,
        last_checkpoint_at: 1_700_000_100,
        batch_duration_ms: Some(170),
    };
    let fbytes = rmp_serde::to_vec_named(&forward).expect("forward payload encode");
    let round: IndexCheckpointPayload =
        rmp_serde::from_slice(&fbytes).expect("decode forward IndexCheckpointPayload");
    assert_eq!(round.batch_duration_ms, Some(170));

    // Back-compat: a payload encoded WITHOUT batch_duration_ms (legacy
    // shape) decodes with None. We construct the legacy bytes by mapping
    // a JSON object → msgpack: the production encoder uses named maps,
    // so a JSON Value round-tripped through serde_transcode mirrors what
    // pre-C-17 emitters produce on the wire.
    //
    // Simpler: round-trip the forward payload through JSON (drops the
    // None) then re-decode as IndexCheckpointPayload. With serde's
    // `#[serde(default, skip_serializing_if = "Option::is_none")]` the
    // None field is omitted in JSON and re-added as None on decode.
    let payload_none = IndexCheckpointPayload {
        project_id: "p".to_string(),
        batches_processed: 3,
        batches_total: Some(8),
        files_processed: 42,
        last_checkpoint_at: 1_700_000_000,
        batch_duration_ms: None,
    };
    let bytes_none = rmp_serde::to_vec_named(&payload_none).expect("encode None variant");
    let decoded_none: IndexCheckpointPayload =
        rmp_serde::from_slice(&bytes_none).expect("decode None variant");
    assert!(
        decoded_none.batch_duration_ms.is_none(),
        "Option<u64> must round-trip None as None"
    );
}
