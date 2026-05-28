// E-2 contract test — SsbActionType::UxPacingVoteRecorded + payload
// round-trip. Locks the wire format added in E-2 Task 10. The Bilgihan
// ResearchGate 326858669 citation is SUPPORTING-ONLY per DECISION-4 in
// workflow/reasoning/E-2-decisions.md — the 30-50% perceived-faster
// headline is NOT quoted in the payload nor in this contract.
//
// Test plan:
//   1. as_sql_str round-trip via ssb_action_type_from_str_public.
//   2. msgpack round-trip on UxPacingVoteRecordedPayload.
//   3. action_type() discriminant matches the SsbActionType variant.
//   4. SCHEMA_VERSION floor (>= 153).
//   5. Both vote values ("snappy" / "slow") round-trip.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, SsbActionType, SsbPayload,
    UxPacingVoteRecordedPayload, SCHEMA_VERSION_PUBLIC,
};

#[test]
fn ux_pacing_vote_recorded_sql_round_trip() {
    let s = SsbActionType::UxPacingVoteRecorded.as_sql_str();
    assert_eq!(s, "ux_pacing_vote_recorded");
    let back = ssb_action_type_from_str_public(s).expect("from_str");
    assert_eq!(back, SsbActionType::UxPacingVoteRecorded);
}

#[test]
fn ux_pacing_vote_recorded_payload_round_trip_snappy() {
    let payload = UxPacingVoteRecordedPayload {
        request_id: "cc-stream-vote-snappy".into(),
        vote: "snappy".into(),
        frame_pace_enabled_at_vote: true,
        cps_at_vote: 60,
        effort_level: "medium".into(),
        ts_ms: 1_748_000_000_700,
    };
    let wrapped = SsbPayload::UxPacingVoteRecorded(payload.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, wrapped, "msgpack round-trip");
    assert_eq!(
        decoded.action_type(),
        SsbActionType::UxPacingVoteRecorded
    );
}

#[test]
fn ux_pacing_vote_recorded_payload_round_trip_slow_pacer_off() {
    // "slow" vote when the FramePacer was OFF — this is the
    // counterfactual that anchors the A/B comparison.
    let payload = UxPacingVoteRecordedPayload {
        request_id: "cc-stream-vote-slow".into(),
        vote: "slow".into(),
        frame_pace_enabled_at_vote: false,
        cps_at_vote: 0,
        effort_level: "high".into(),
        ts_ms: 1_748_000_000_800,
    };
    let wrapped = SsbPayload::UxPacingVoteRecorded(payload.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, wrapped);
}

#[test]
fn ssb_schema_version_at_least_e2_vote_baseline() {
    let v = SCHEMA_VERSION_PUBLIC;
    assert!(
        v >= 153,
        "E-2 floors SCHEMA_VERSION at 153 for UxPacingVoteRecorded; got {v}"
    );
}
