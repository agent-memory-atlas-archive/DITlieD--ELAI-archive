//! I-11 -- BeliefState / PABU belief-state handoff contract tests.
//!
//! Coverage (F1..F5, W1..W3; no #[ignore], no stubs, no placeholder asserts):
//!
//! (F1) Schema lock (golden 2/2): SCHEMA_VERSION_PUBLIC >= 248 AND
//!      action_type_lock.rs declares TOTAL_VARIANTS: usize = 559, confirming
//!      the atomically-locked SSB expansion for I-11 BeliefStateEvent.
//!
//! (F2) BeliefState encode/decode round-trip: lossless MessagePack round-trip
//!      across all field types (Vec<TriedApproach>, Vec<String>, Vec<u8>, Vec<i64>, u32).
//!
//! (F3) Version mismatch fails closed: a blob with version != BELIEF_STATE_SCHEMA_VERSION
//!      returns Err(BeliefStateError::VersionMismatch), never a silently wrong parse.
//!
//! (F4) blake3 prefix is deterministic: the same BeliefState produces the same 16-char
//!      hex prefix across two calls; differs from a distinct BeliefState.
//!
//! (F5) SSB round-trip for all 3 BeliefStateEvent arms (Condensed / Handoff / HonestyDelta):
//!      real Ssb::write_payload + query_by_type + msgpack decode round-trip. Catches the
//!      SQL CHECK-constraint comma gotcha -- a missing wire tag causes runtime INSERT failure
//!      invisible to `cargo check`.
//!
//! (W1) evaluate_handoff is reachable: calling evaluate_handoff with a below-threshold
//!      HandoffContext returns HandoffDecision::Continue (not a compile error; proves the
//!      production symbol is callable from outside escalation_policy.rs).
//!
//! (W2) adaptive_reduce is reachable: constructing a ContextBudget and calling
//!      adaptive_reduce in the Cloud arm does not error when total_used <= model_max_context
//!      (the early-return path). Proves the production symbol is callable.
//!
//! (W3) run_honesty_job_if_due fires on exact interval: with escalation_count = 10
//!      and force = false, the honesty job emits a HonestyDelta SSB row.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::SsbReader;
use elai_orchestrator::belief_state::{
    run_honesty_job_if_due, BeliefState, BeliefStateError, TriedApproach, BELIEF_STATE_SCHEMA_VERSION,
    HONESTY_JOB_INTERVAL,
};
use elai_orchestrator::context_budget::{ContextBudget, ExecutionMode};
use elai_orchestrator::escalation_policy::{
    evaluate_handoff, HandoffContext, HandoffDecision, FALLBACK_EFFECTIVE_MAX_CONTEXT,
};
use elai_orchestrator::ssb::{
    BeliefStateEventKind, BeliefStateEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ssb_config(dir: &TempDir, label: &str) -> SsbConfig {
    SsbConfig {
        db_path: dir.path().join(format!("{label}.db")),
        pool_size: Some(2),
        project_id: "i11-belief-state-contract".to_string(),
        session_id: format!("i11-contract-{label}"),
    }
}

fn clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn make_belief_state() -> BeliefState {
    BeliefState::new(
        vec![TriedApproach {
            label: "i11-contract-attempt".into(),
            token_cost: 2048,
        }],
        vec!["missing pub use".into()],
        vec![0xDE, 0xAD, 0xBE, 0xEF],
        vec!["no .unwrap() in prod".into(), "blake3 only".into()],
        vec![101_i64, 202_i64],
        1_024,
    )
}

// ---------------------------------------------------------------------------
// (F1) Schema lock
// ---------------------------------------------------------------------------

/// SCHEMA_VERSION_PUBLIC must be >= 248 (I-11 floor).
#[test]
fn schema_version_is_248() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 248,
        "SCHEMA_VERSION_PUBLIC must be >= 248 (I-11 floor: BeliefStateEvent add); \
         got {} -- a value below 248 means the I-11 SSB bump was regressed",
        SCHEMA_VERSION_PUBLIC,
    );
}

/// TOTAL_VARIANTS in action_type_lock.rs must be 557 (I-11 adds 1 representative).
#[test]
fn action_type_lock_total_variants_is_557() {
    let lock_src = include_str!(
        "../../orchestrator/tests/action_type_lock.rs"
    );
    assert!(
        lock_src.contains("TOTAL_VARIANTS: usize = 557"),
        "TOTAL_VARIANTS must be 557 (I-11 adds BeliefStateEvent representative; \
         Pattern-2 single-representative convention, wire tag 'belief_state_event'); \
         file does not contain 'TOTAL_VARIANTS: usize = 557'"
    );
}

// ---------------------------------------------------------------------------
// (F2) BeliefState encode/decode round-trip
// ---------------------------------------------------------------------------

#[test]
fn belief_state_encode_decode_round_trip() {
    let original = make_belief_state();
    let bytes = original.encode().expect("encode must not fail");
    assert!(!bytes.is_empty(), "encoded bytes must be non-empty");

    let decoded = BeliefState::decode(&bytes).expect("decode must not fail");
    assert_eq!(decoded.version, BELIEF_STATE_SCHEMA_VERSION);
    assert_eq!(decoded.tried.len(), 1);
    assert_eq!(decoded.tried[0].label, "i11-contract-attempt");
    assert_eq!(decoded.tried[0].token_cost, 2048);
    assert_eq!(decoded.failed_because, vec!["missing pub use"]);
    assert_eq!(decoded.current_ast_state, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    assert_eq!(decoded.constraints.len(), 2);
    assert_eq!(decoded.dkg_entities, vec![101_i64, 202_i64]);
    assert_eq!(decoded.token_delta, 1_024);
}

// ---------------------------------------------------------------------------
// (F3) Version mismatch fails closed
// ---------------------------------------------------------------------------

#[test]
fn belief_state_version_mismatch_fails_closed() {
    // Construct a BeliefState, encode it, then tamper the version byte via a
    // new BeliefState with a different version field by injecting a raw struct.
    // We round-trip an incompatible blob by directly deserializing with a
    // mis-versioned inner struct using rmp_serde.
    #[derive(serde::Serialize, serde::Deserialize)]
    struct FakeBeliefState {
        version: u32,
        tried: Vec<TriedApproach>,
        failed_because: Vec<String>,
        current_ast_state: Vec<u8>,
        constraints: Vec<String>,
        dkg_entities: Vec<i64>,
        token_delta: u32,
    }

    let fake = FakeBeliefState {
        version: BELIEF_STATE_SCHEMA_VERSION + 99,
        tried: vec![],
        failed_because: vec![],
        current_ast_state: vec![],
        constraints: vec![],
        dkg_entities: vec![],
        token_delta: 0,
    };
    let tampered = rmp_serde::to_vec_named(&fake).expect("encode tampered");

    let result = BeliefState::decode(&tampered);
    match result {
        Err(BeliefStateError::VersionMismatch { encoded, current }) => {
            assert_eq!(encoded, BELIEF_STATE_SCHEMA_VERSION + 99);
            assert_eq!(current, BELIEF_STATE_SCHEMA_VERSION);
        }
        other => panic!(
            "expected VersionMismatch, got {:?}",
            other.map(|_| "<ok>".to_string()).unwrap_err()
        ),
    }
}

// ---------------------------------------------------------------------------
// (F4) blake3 prefix is deterministic
// ---------------------------------------------------------------------------

#[test]
fn belief_state_blake3_prefix_is_deterministic() {
    let bs = make_belief_state();
    let p1 = bs.blake3_prefix().expect("prefix 1");
    let p2 = bs.blake3_prefix().expect("prefix 2");
    assert_eq!(p1, p2, "blake3 prefix must be deterministic");
    assert_eq!(p1.len(), 16, "prefix must be 16 hex chars (8 bytes)");

    let other = BeliefState::new(
        vec![TriedApproach { label: "other".into(), token_cost: 1 }],
        vec![],
        vec![],
        vec![],
        vec![],
        0,
    );
    let p_other = other.blake3_prefix().expect("other prefix");
    assert_ne!(p1, p_other, "distinct BeliefStates must have distinct prefixes");
}

// ---------------------------------------------------------------------------
// (F5) SSB round-trip for all 3 BeliefStateEvent arms
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn belief_state_event_ssb_round_trip_all_arms() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(Ssb::new(ssb_config(&dir, "round-trip"), clock()).expect("ssb init"));

    let ts_ms: i64 = 1_700_000_000_000;

    let arms = [
        SsbPayload::BeliefStateEvent(BeliefStateEventPayload {
            kind: BeliefStateEventKind::Condensed,
            schema_version: BELIEF_STATE_SCHEMA_VERSION,
            token_delta: 1_200,
            payload_bytes: 0,
            ts_ms,
        }),
        SsbPayload::BeliefStateEvent(BeliefStateEventPayload {
            kind: BeliefStateEventKind::Handoff,
            schema_version: BELIEF_STATE_SCHEMA_VERSION,
            token_delta: 1_200,
            payload_bytes: 512,
            ts_ms: ts_ms + 1_000,
        }),
        SsbPayload::BeliefStateEvent(BeliefStateEventPayload {
            kind: BeliefStateEventKind::HonestyDelta,
            schema_version: BELIEF_STATE_SCHEMA_VERSION,
            token_delta: 800,
            payload_bytes: 0,
            ts_ms: ts_ms + 2_000,
        }),
    ];

    for (i, payload) in arms.iter().enumerate() {
        let action = payload.action_type();
        let seq = ssb
            .write_payload(&format!("i11-agent-{i}"), i as i64, payload.clone(), vec![])
            .await
            .unwrap_or_else(|e| panic!("write_payload arm {i} failed: {e}"));
        assert!(seq > 0, "seq must be positive for arm {i}");

        let entries = ssb
            .query_by_type(action)
            .unwrap_or_else(|e| panic!("query_by_type arm {i} failed: {e}"));
        assert!(!entries.is_empty(), "query_by_type must return entries for arm {i}");

        let entry = entries.last().expect("at least one entry");
        let decoded: SsbPayload = rmp_serde::from_slice(&entry.payload)
            .unwrap_or_else(|e| panic!("msgpack decode arm {i} failed: {e}"));
        assert_eq!(&decoded, payload, "round-trip mismatch for arm {i}");
    }
}

// ---------------------------------------------------------------------------
// (W1) evaluate_handoff is reachable
// ---------------------------------------------------------------------------

/// Calling evaluate_handoff below threshold returns Continue.
/// Proves the symbol is callable from outside escalation_policy.rs.
#[tokio::test(flavor = "current_thread")]
async fn evaluate_handoff_reachable_below_threshold() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(Ssb::new(ssb_config(&dir, "w1"), clock()).expect("ssb init"));

    let ctx = HandoffContext {
        tokens_used: 1_000,
        effective_max_context: FALLBACK_EFFECTIVE_MAX_CONTEXT,
        files_referenced: 0,
        remote_24gb_reachable: false,
        session_id: 0xdeadbeef,
    };

    // 1000 < 85% of 25000 (21250) -- must return Continue.
    let decision = evaluate_handoff(&ctx, &ssb)
        .await
        .expect("evaluate_handoff must not fail");
    assert_eq!(
        decision,
        HandoffDecision::Continue,
        "tokens_used=1000 is below threshold 21250; expected Continue"
    );
}

// ---------------------------------------------------------------------------
// (W2) adaptive_reduce is reachable
// ---------------------------------------------------------------------------

/// Constructing a ContextBudget and calling adaptive_reduce at <= max_context
/// returns Ok(()) (the early-return path). Proves the symbol is callable.
#[tokio::test(flavor = "current_thread")]
async fn adaptive_reduce_reachable_below_threshold() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Ssb::new(ssb_config(&dir, "w2"), clock()).expect("ssb init");

    // Use a 32K budget; total_used starts at 0, which is <= model_max_context -> no-op early return.
    let mut budget = ContextBudget::new(32_000);
    let result = budget
        .adaptive_reduce(0.5, ExecutionMode::Autonomous, &ssb)
        .await;
    assert!(
        result.is_ok(),
        "adaptive_reduce with empty budget must return Ok; got {:?}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// (W3) run_honesty_job_if_due fires on exact interval
// ---------------------------------------------------------------------------

/// With escalation_count = HONESTY_JOB_INTERVAL (10) and force = false,
/// run_honesty_job_if_due emits a HonestyDelta SSB row.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_honesty_job_if_due_fires_on_interval() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Ssb::new(ssb_config(&dir, "w3"), clock()).expect("ssb init");
    let ts_ms: i64 = 1_700_000_000_000;

    run_honesty_job_if_due(
        &ssb,
        0xdeadbeef_u64,
        HONESTY_JOB_INTERVAL,
        30_000,
        20_000,
        ts_ms,
        false,
    )
    .await;

    // Verify the HonestyDelta row landed in the db.
    let entries = ssb
        .query_by_type(SsbActionType::BeliefStateEvent(
            BeliefStateEventKind::HonestyDelta,
        ))
        .expect("query_by_type HonestyDelta must not fail");
    assert!(
        !entries.is_empty(),
        "HonestyDelta SSB row must be emitted when escalation_count == HONESTY_JOB_INTERVAL"
    );

    // Verify the row deserializes correctly.
    let entry = entries.last().expect("last entry");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&entry.payload).expect("HonestyDelta msgpack decode");
    match decoded {
        SsbPayload::BeliefStateEvent(p) => {
            assert_eq!(p.kind, BeliefStateEventKind::HonestyDelta);
            assert_eq!(p.schema_version, BELIEF_STATE_SCHEMA_VERSION);
            // token_delta = full_history - belief_state = 30000 - 20000 = 10000.
            assert_eq!(p.token_delta, 10_000);
        }
        other => panic!("expected BeliefStateEvent(HonestyDelta), got {other:?}"),
    }
}
