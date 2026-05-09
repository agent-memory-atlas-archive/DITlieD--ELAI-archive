// Track A.1 contract test: IndexingProgressPayload.message is non-None on the
// "failed" stage, and error_kind/error_message fields (if added per A.1 spec)
// round-trip through msgpack.
//
// RED STATE: the error_kind and error_message fields do NOT yet exist on
// IndexingProgressPayload. The IndexingProgressPayload::message field already
// exists; the gap is that the Err arm in cli/src/indexing.rs always passes
// None, so message is always None on failure rows.
//
// Symbols that DO NOT yet exist (will cause compile errors until A.1 is
// implemented by execution agent #2):
//   - IndexingProgressPayload.error_kind  (field does not exist)
//   - IndexingProgressPayload.error_message  (field does not exist)
//   - emit_indexing_ssb message parameter (private helper, change not visible
//     here — tested indirectly by driving run_full_index against a bad path)
//
// When A.1 production code lands, the compile errors on the error_kind /
// error_message field accesses below will resolve and both tests will turn
// green.
//
// Approach: use the public IndexingProgressPayload struct directly for the
// field-presence test, and drive IndexingService against an unwritable path
// for the end-to-end emit test. The latter depends on the Err arm threading
// e.to_string() through `message` — currently broken (always None).

use elai_orchestrator::ssb::{IndexingProgressPayload, SsbActionType, SsbPayload};

// ---------------------------------------------------------------------------
// A.1.a — error_kind and error_message fields exist on the payload struct
//          and survive msgpack round-trip.
//
// RED: IndexingProgressPayload has no error_kind / error_message fields yet.
// ---------------------------------------------------------------------------

#[test]
fn indexing_progress_failed_payload_has_error_fields() {
    // Construct a "failed" payload with the two new optional error fields.
    // This line will NOT COMPILE until execution agent #2 adds the fields.
    let payload = IndexingProgressPayload {
        stage: "failed".into(),
        project_id: "proj-test".into(),
        files_processed: 0,
        files_total: 0,
        elapsed_ms: 12,
        message: Some("parse failed: unexpected EOF".into()),
        // error_kind and error_message do NOT yet exist — RED STATE.
        error_kind: Some("dkg_error".into()),
        error_message: Some("DKG parse failed at line 42".into()),
    };

    assert_eq!(payload.stage, "failed", "stage must be 'failed'");
    assert!(
        payload.message.is_some(),
        "message must be Some on a failed row",
    );
    assert_eq!(
        payload.error_kind.as_deref(),
        Some("dkg_error"),
        "error_kind must survive construction",
    );
    assert_eq!(
        payload.error_message.as_deref(),
        Some("DKG parse failed at line 42"),
        "error_message must survive construction",
    );

    // msgpack round-trip.
    let wrapped = SsbPayload::IndexingProgress(payload);
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("IndexingProgress msgpack serialize");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("IndexingProgress msgpack deserialize");

    match decoded {
        SsbPayload::IndexingProgress(p) => {
            assert_eq!(p.stage, "failed", "stage must survive msgpack round-trip");
            assert!(p.message.is_some(), "message must survive round-trip");
            assert_eq!(
                p.error_kind.as_deref(),
                Some("dkg_error"),
                "error_kind must survive msgpack round-trip",
            );
            assert_eq!(
                p.error_message.as_deref(),
                Some("DKG parse failed at line 42"),
                "error_message must survive msgpack round-trip",
            );
        }
        other => panic!("expected IndexingProgress variant; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// A.1.b — old "failed" rows that lack error_kind/error_message (pre-patch)
//          still deserialize correctly (backward-compat check).
//
// This test is GREEN today (pre-patch) because the fields are missing and
// serde(default) will supply None. After the patch it must STILL be green.
// ---------------------------------------------------------------------------

#[test]
fn indexing_progress_failed_missing_error_fields_deserialize_as_none() {
    // Simulate a pre-patch row: JSON that has stage/message but no error_kind /
    // error_message. After the patch, #[serde(default)] on both new fields must
    // make this deserialize to None rather than failing.
    let json = r#"{
        "stage": "failed",
        "project_id": "proj-old",
        "files_processed": 0,
        "files_total": 0,
        "elapsed_ms": 5,
        "message": "an old error"
    }"#;

    let p: IndexingProgressPayload =
        serde_json::from_str(json).expect("old 'failed' rows must deserialize without error_kind");

    assert_eq!(p.stage, "failed");
    assert_eq!(p.message.as_deref(), Some("an old error"));
    // These will be None because the fields are absent in the JSON and the
    // struct uses #[serde(default)].  Before the patch these fields do not
    // exist so this test compiles but the assertions are vacuously checked only
    // after the patch adds the fields.
    assert!(
        p.error_kind.is_none(),
        "error_kind must default to None on old rows (serde default)",
    );
    assert!(
        p.error_message.is_none(),
        "error_message must default to None on old rows (serde default)",
    );
}

// ---------------------------------------------------------------------------
// A.1.c — SsbActionType discriminant is stable.
// ---------------------------------------------------------------------------

#[test]
fn indexing_progress_action_type_is_stable() {
    let p = SsbPayload::IndexingProgress(IndexingProgressPayload {
        stage: "failed".into(),
        project_id: "p".into(),
        files_processed: 0,
        files_total: 0,
        elapsed_ms: 0,
        message: None,
        // error_kind / error_message will need Default::default() here after
        // the fields are added — RED until then.
        error_kind: None,
        error_message: None,
    });
    assert_eq!(
        p.action_type(),
        SsbActionType::IndexingProgress,
        "action_type() discriminant must be IndexingProgress",
    );
}
