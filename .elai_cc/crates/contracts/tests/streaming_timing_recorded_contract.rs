// E-2 contract test — SsbActionType::StreamingTimingRecorded + payload
// round-trip. Locks the wire format added in E-2 Task 5. See
// workflow/reasoning/E-2-decisions.md DECISION-8 (telemetry is computed
// dashboard-side at commands.rs::start_chat_stream from receive
// timestamps; this contract test only locks the row shape, not the
// emission site).
//
// Test plan:
//   1. as_sql_str round-trip via ssb_action_type_from_str_public.
//   2. msgpack round-trip on StreamingTimingRecordedPayload.
//   3. action_type() discriminant matches the SsbActionType variant.
//   4. SCHEMA_VERSION floor (>= 152).

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, SsbActionType, SsbPayload,
    StreamingTimingRecordedPayload, SCHEMA_VERSION_PUBLIC,
};

#[test]
fn streaming_timing_recorded_sql_round_trip() {
    let s = SsbActionType::StreamingTimingRecorded.as_sql_str();
    assert_eq!(s, "streaming_timing_recorded");
    let back = ssb_action_type_from_str_public(s).expect("from_str");
    assert_eq!(back, SsbActionType::StreamingTimingRecorded);
}

#[test]
fn streaming_timing_recorded_payload_round_trip() {
    let payload = StreamingTimingRecordedPayload {
        request_id: "cc-stream-abc123".into(),
        ttft_ms: 187,
        mean_inter_chunk_ms: 42,
        p50_inter_chunk_ms: 31,
        p99_inter_chunk_ms: 280,
        total_ms: 4_700,
        tokens_emitted: 162,
        stop_reason: "end_turn".into(),
        ts_ms: 1_748_000_000_000,
    };
    let wrapped = SsbPayload::StreamingTimingRecorded(payload.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, wrapped, "msgpack round-trip");
    assert_eq!(
        decoded.action_type(),
        SsbActionType::StreamingTimingRecorded
    );
}

#[test]
fn streaming_timing_recorded_aborted_stop_reason() {
    // A bubble cancelled mid-stream still emits a row with
    // stop_reason="aborted" so the dashboard can render the partial
    // bubble timing histogram correctly.
    let payload = StreamingTimingRecordedPayload {
        request_id: "cc-stream-aborted".into(),
        ttft_ms: 95,
        mean_inter_chunk_ms: 22,
        p50_inter_chunk_ms: 18,
        p99_inter_chunk_ms: 110,
        total_ms: 1_200,
        tokens_emitted: 47,
        stop_reason: "aborted".into(),
        ts_ms: 1_748_000_000_500,
    };
    let wrapped = SsbPayload::StreamingTimingRecorded(payload.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, wrapped);
}

#[test]
fn ssb_schema_version_at_least_e2_baseline() {
    let v = SCHEMA_VERSION_PUBLIC;
    assert!(
        v >= 152,
        "E-2 floors SCHEMA_VERSION at 152 for StreamingTimingRecorded; got {v}"
    );
}
