//! B-31.5 — round-trip test for the 5 new SSB action variants.
//!
//! Constructs a payload for each new variant, serializes it via the shared
//! `encode_ssb_payload` helper (mirrors production's `Ssb::write_payload`
//! encode path: `rmp_serde::to_vec_named(&payload: SsbPayload)`), deserializes
//! back, and asserts equality plus correct action_type discriminant string.

use elai_orchestrator::{
    encode_ssb_payload, IndexCancelPayload, IndexCheckpointPayload,
    IndexResumePayload, LockAcquiredPayload, LockContestedPayload, SsbActionType,
    SsbPayload,
};

fn round_trip(payload: SsbPayload) -> SsbPayload {
    let bytes = encode_ssb_payload(&payload).expect("encode");
    rmp_serde::from_slice::<SsbPayload>(&bytes).expect("decode")
}

#[test]
fn index_checkpoint_round_trip_and_action_type() {
    let payload = SsbPayload::IndexCheckpoint(IndexCheckpointPayload {
        project_id: "abc123def4567890".into(),
        batches_processed: 2,
        batches_total: Some(4),
        files_processed: 100,
        last_checkpoint_at: 1_700_000_000,
    });
    let decoded = round_trip(payload.clone());
    assert_eq!(decoded, payload);
    assert_eq!(payload.action_type(), SsbActionType::IndexCheckpoint);
    assert_eq!(SsbActionType::IndexCheckpoint.as_sql_str(), "index_checkpoint");
}

#[test]
fn index_resume_round_trip_and_action_type() {
    let payload = SsbPayload::IndexResume(IndexResumePayload {
        project_id: "abc123def4567890".into(),
        resumed_from_batch: 1,
        files_skipped: 20,
    });
    let decoded = round_trip(payload.clone());
    assert_eq!(decoded, payload);
    assert_eq!(payload.action_type(), SsbActionType::IndexResume);
    assert_eq!(SsbActionType::IndexResume.as_sql_str(), "index_resume");
}

#[test]
fn index_cancel_round_trip_and_action_type() {
    let payload = SsbPayload::IndexCancel(IndexCancelPayload {
        project_id: "abc123def4567890".into(),
        batches_completed: 1,
        files_indexed: 50,
    });
    let decoded = round_trip(payload.clone());
    assert_eq!(decoded, payload);
    assert_eq!(payload.action_type(), SsbActionType::IndexCancel);
    assert_eq!(SsbActionType::IndexCancel.as_sql_str(), "index_cancel");
}

#[test]
fn lock_acquired_round_trip_and_action_type() {
    let payload = SsbPayload::LockAcquired(LockAcquiredPayload {
        project_id: "abc123def4567890".into(),
        scope: "index".into(),
        holder_pid: 12345,
    });
    let decoded = round_trip(payload.clone());
    assert_eq!(decoded, payload);
    assert_eq!(payload.action_type(), SsbActionType::LockAcquired);
    assert_eq!(SsbActionType::LockAcquired.as_sql_str(), "lock_acquired");
}

#[test]
fn lock_contested_round_trip_and_action_type() {
    let payload = SsbPayload::LockContested(LockContestedPayload {
        project_id: "abc123def4567890".into(),
        scope: "watch".into(),
        holder_pid: 99999,
    });
    let decoded = round_trip(payload.clone());
    assert_eq!(decoded, payload);
    assert_eq!(payload.action_type(), SsbActionType::LockContested);
    assert_eq!(SsbActionType::LockContested.as_sql_str(), "lock_contested");
}

#[test]
fn ssb_action_type_strings_are_stable() {
    // First-line documentation of the lockstep string forms — fail loud if
    // any drift between as_sql_str + the CHECK constraint pair.
    let pairs = &[
        (SsbActionType::IndexCheckpoint, "index_checkpoint"),
        (SsbActionType::IndexResume, "index_resume"),
        (SsbActionType::IndexCancel, "index_cancel"),
        (SsbActionType::LockAcquired, "lock_acquired"),
        (SsbActionType::LockContested, "lock_contested"),
    ];
    for (t, s) in pairs {
        assert_eq!(t.as_sql_str(), *s);
    }
}
