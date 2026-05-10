//! B-39 — ChatReflect pipeline contract tests (TDD RED state).
//!
//! All 12 tests reference types that do not yet exist. They are expected to
//! fail to compile until the EXECUTING step of B-39 creates:
//!
//!   * `elai_reasoning::chat_reflect` — ChatReflectPipeline, ChatReflectConfig,
//!     ChatInferenceClient (trait), ReflectOutput, ReflectTokenSummary, TurnClass,
//!     ReflectError, N_CANDIDATES, REFINE_CAP.
//!   * `elai_orchestrator::ssb` — 4 new SsbActionType variants and payload
//!     structs: ReflectTriaged, ReflectCandidate, ReflectAuditVerdict, ReflectRefined.
//!   * `elai_orchestrator::ssb_viewer::SsbViewer::get_reflect_state`.
//!
//! Schema version target: 29 (current source is 28; B-39 migration bumps it).
//!
//! Constants rationale:
//!   * REFINE_CAP = 3  — RefineBench guided-mode data: t=2→t=3 yields +12-20pp;
//!     cap=3 captures the bulk of the guided-mode gain without VRAM overspend.
//!   * N_CANDIDATES = 4 — [pending-ingest] Liu et al. (arXiv pending ingestion);
//!     candidate count at N=4 maximises diversity before compute cost inflects.
//!
//! Grounding: code read from
//!   crates/orchestrator/src/ssb.rs       (SsbActionType, SsbConfig, Ssb::new, write_payload,
//!                                          query_by_type, SsbEntry, as_sql_str,
//!                                          ssb_action_type_from_str_public)
//!   crates/orchestrator/src/ssb_viewer.rs (SsbViewer::open, public fn signatures)
//!   crates/orchestrator/src/lib.rs        (pub use ssb::*, pub use ssb_viewer::*)
//!   crates/reasoning/src/lib.rs           (currently-exported surface; chat_reflect absent)
//!   crates/contracts/tests/a23_micro_reflection_contract.rs  (header + mock style)
//!   crates/contracts/tests/fetcher_used_ssb_contract.rs      (Ssb construction pattern)

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use elai_core::clock::OsClock;
use elai_orchestrator::{
    Ssb, SsbConfig, SsbPayload, SsbActionType, SsbReader,
};
use elai_orchestrator::ssb::ssb_action_type_from_str_public;
use elai_orchestrator::ssb_viewer::SsbViewer;

// B-39 types that do NOT exist yet — this file is deliberately in RED state.
// When EXECUTING lands, these imports will resolve.
use elai_reasoning::chat_reflect::{
    ChatFuture, ChatInferenceClient, ChatReflectConfig, ChatReflectPipeline, ReflectError,
    ReflectEventSink, ReflectOutput, TurnClass, N_CANDIDATES, REFINE_CAP,
};
use elai_orchestrator::ssb::{
    ReflectAuditVerdictPayload, ReflectCandidatePayload, ReflectRefinedPayload,
    ReflectTriagedPayload, SsbActionType::ReflectAuditVerdict, SsbActionType::ReflectCandidate,
    SsbActionType::ReflectRefined, SsbActionType::ReflectTriaged,
};

// ---------------------------------------------------------------------------
// Mock inference client — all test-file-local, no separate test-utils crate.
// ---------------------------------------------------------------------------

/// Canned-response mock for `ChatInferenceClient`.
///
/// Responses are handed out in FIFO order. Once exhausted, every subsequent
/// call returns `ReflectError::Unavailable("mock exhausted")`.
struct MockChatInferenceClient {
    responses: std::sync::Mutex<Vec<Result<String, ReflectError>>>,
    call_count: AtomicU32,
}

impl MockChatInferenceClient {
    fn new(responses: Vec<Result<String, ReflectError>>) -> Self {
        Self {
            responses: std::sync::Mutex::new(responses),
            call_count: AtomicU32::new(0),
        }
    }

    fn call_count(&self) -> u32 {
        self.call_count.load(Ordering::SeqCst)
    }
}

impl ChatInferenceClient for MockChatInferenceClient {
    fn complete<'a>(&'a self, _prompt: &'a str, _max_tokens: u32) -> ChatFuture<'a, String> {
        Box::pin(async move {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            let mut lock = self.responses.lock().expect("mock lock poisoned");
            if lock.is_empty() {
                return Err(ReflectError::Inference("mock exhausted".into()));
            }
            lock.remove(0)
        })
    }
}

// ---------------------------------------------------------------------------
// Helper: open an in-memory SSB in a tempdir (matches fetcher_used_ssb_contract
// pattern).
// ---------------------------------------------------------------------------

fn new_ssb(db_path: std::path::PathBuf) -> Arc<Ssb> {
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "contracts-b39".into(),
        session_id: "test-b39".into(),
    };
    Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
}

/// Construct a happy-path mock: produces canned replies for triage (1 call),
/// reasoner (N_CANDIDATES calls), and auditor (1 call).
///
/// Triage reply: valid JSON TurnClass discriminant (`{"class":"Chat"}`).
/// Reasoner replies: N_CANDIDATES short strings.
/// Auditor reply: valid JSON best-index selector (`{"best_index":0,"rationale":"ok"}`).
fn happy_path_mock() -> MockChatInferenceClient {
    // Total calls = 1 (triage) + N_CANDIDATES (reasoner) + 1 (auditor) = 6 with N_CANDIDATES=4.
    let mut responses: Vec<Result<String, ReflectError>> = Vec::new();

    // Triage — use "problem" so the full REASONER+AUDITOR pipeline runs.
    // "Chat" would short-circuit and skip stages 2-4, so SSB rows for candidates/verdict
    // would never be written (see WARN-5 in audit-B-39.md).
    responses.push(Ok(r#"{"class":"problem"}"#.into()));

    // Reasoner candidates (N_CANDIDATES = 4)
    for i in 0..N_CANDIDATES {
        responses.push(Ok(format!("candidate reply {i}")));
    }

    // Auditor
    responses.push(Ok(
        r#"{"best_index":0,"rationale":"first candidate was best"}"#.into(),
    ));

    MockChatInferenceClient::new(responses)
}

/// Default ChatReflectConfig used across tests.
fn default_config() -> ChatReflectConfig {
    ChatReflectConfig::default()
}

// ===========================================================================
// Test 1 — SCHEMA_VERSION is 29.
// ===========================================================================

#[tokio::test]
async fn test_schema_version_is_29() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("ssb_b39.db");
    // Instantiating Ssb runs the migration, setting the schema version.
    let _ssb = new_ssb(db_path.clone());

    // Query directly via rusqlite (same technique as dashboard_4k3_4k4_4k5_contract.rs).
    let conn = rusqlite::Connection::open(&db_path).expect("open db for version check");
    let v: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("user_version pragma");
    assert_eq!(v, 29, "B-39 migration must bump schema to user_version=29");
}

// ===========================================================================
// Test 2 — SsbActionType::ReflectTriaged wire-string roundtrip.
// ===========================================================================

#[test]
fn test_action_type_reflect_triaged_roundtrip() {
    let variant = SsbActionType::ReflectTriaged;

    // Forward: enum -> SQL/wire string.
    let wire = variant.as_sql_str();
    assert_eq!(
        wire, "reflect_triaged",
        "ReflectTriaged wire form must be 'reflect_triaged'"
    );

    // Backward: wire string -> enum via the public parse helper.
    let parsed =
        ssb_action_type_from_str_public(wire).expect("reflect_triaged must parse back to enum");
    assert_eq!(
        parsed,
        SsbActionType::ReflectTriaged,
        "parse('reflect_triaged') must round-trip to ReflectTriaged"
    );
}

// ===========================================================================
// Test 3 — ReflectCandidatePayload serde roundtrip (JSON, snake_case fields).
// ===========================================================================

#[test]
fn test_reflect_candidate_payload_serde_roundtrip() {
    let payload = ReflectCandidatePayload {
        turn_id: "turn-abc-123".into(),
        candidate_index: 2,
        content: "The answer is 42".into(),
    };

    let json = serde_json::to_string(&payload)
        .expect("ReflectCandidatePayload must serialize to JSON");

    // Verify snake_case field names are present in the serialized form.
    assert!(
        json.contains("\"turn_id\""),
        "serialized JSON must contain 'turn_id' (snake_case)"
    );
    assert!(
        json.contains("\"candidate_index\""),
        "serialized JSON must contain 'candidate_index' (snake_case)"
    );
    assert!(
        json.contains("\"content\""),
        "serialized JSON must contain 'content'"
    );

    // Roundtrip: deserialize and check field equality.
    let back: ReflectCandidatePayload =
        serde_json::from_str(&json).expect("ReflectCandidatePayload must deserialize from JSON");
    assert_eq!(back.turn_id, "turn-abc-123", "turn_id must survive roundtrip");
    assert_eq!(back.candidate_index, 2, "candidate_index must survive roundtrip");
    assert_eq!(back.content, "The answer is 42", "content must survive roundtrip");
}

// ===========================================================================
// Test 4 — SsbViewer::get_reflect_state returns None/null for empty SSB.
// ===========================================================================

#[tokio::test]
async fn test_ssb_viewer_get_reflect_state_empty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("ssb_empty.db");
    let _ssb = new_ssb(db_path.clone());
    // Drop Ssb to flush WAL before opening viewer (matches existing contract pattern).
    drop(_ssb);

    let viewer = SsbViewer::open(&db_path).expect("SsbViewer::open on empty DB");
    let state = viewer
        .get_reflect_state()
        .expect("get_reflect_state must not error on empty SSB");

    // Empty SSB: reflect has never been configured, so enabled=false and all timestamps are None.
    assert!(
        !state.enabled,
        "get_reflect_state on empty SSB must return enabled=false"
    );
    assert!(
        state.last_triage.is_none(),
        "get_reflect_state on empty SSB must return last_triage=None"
    );
}

// ===========================================================================
// Test 5 — ChatReflectPipeline::new with ssb=None does not panic.
// ===========================================================================

#[test]
fn test_chat_reflect_pipeline_new_with_none_ssb_does_not_panic() {
    let config = default_config();
    // None for the Ssb arc — pipeline must degrade gracefully (no persistence).
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _: ChatReflectPipeline = ChatReflectPipeline::new(config, None);
        }));
    assert!(
        result.is_ok(),
        "ChatReflectPipeline::new with ssb=None must not panic"
    );
}

// ===========================================================================
// Test 6 — run() returns skipped=true when provider errors.
// ===========================================================================

#[tokio::test]
async fn test_run_returns_skipped_on_provider_error() {
    let config = default_config();
    // Mock that always returns an error.
    let mock =
        MockChatInferenceClient::new(vec![Err(ReflectError::Inference("sidecar down".into()))]);
    let pipeline = ChatReflectPipeline::new(config, None);

    let input = "Hello, what time is it?";
    let output: ReflectOutput = pipeline
        .run("t6".to_string(), input, &mock)
        .await;

    assert!(output.skipped, "skipped must be true when provider errors");
    assert_eq!(
        output.final_reply, input,
        "final_reply must be the original input when pipeline is skipped"
    );
}

// ===========================================================================
// Test 7 — happy-path run() writes exactly 1 'reflect_triaged' SSB row.
// ===========================================================================

#[tokio::test]
async fn test_triage_writes_one_reflect_triaged_row() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("ssb_triage.db");
    let ssb = new_ssb(db_path);
    let config = default_config();
    let mock = happy_path_mock();

    let sink: Arc<dyn ReflectEventSink> = ssb.clone();
    let pipeline = ChatReflectPipeline::new(config, Some(sink));
    let _output = pipeline
        .run("t7".to_string(), "What is Rust?", &mock)
        .await;

    // SSB emit is fire-and-forget via a channel to an async writer task.
    // Yield to the executor to let the writer task drain the queue before asserting.
    for _ in 0..10 { tokio::task::yield_now().await; }

    let rows = ssb
        .query_by_type(SsbActionType::ReflectTriaged)
        .expect("query ReflectTriaged rows");
    assert_eq!(rows.len(), 1, "exactly 1 reflect_triaged row must be written");
}

// ===========================================================================
// Test 8 — happy-path run() writes exactly N_CANDIDATES 'reflect_candidate' rows
//           with candidate_index values {0, 1, 2, 3}.
// ===========================================================================

#[tokio::test]
async fn test_reasoner_writes_four_reflect_candidate_rows() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("ssb_candidates.db");
    let ssb = new_ssb(db_path);
    let config = default_config();
    let mock = happy_path_mock();

    let sink: Arc<dyn ReflectEventSink> = ssb.clone();
    let pipeline = ChatReflectPipeline::new(config, Some(sink));
    let _output = pipeline
        .run("t8".to_string(), "Explain async/await.", &mock)
        .await;

    // Yield to let the SSB fire-and-forget writer task drain its queue.
    for _ in 0..10 { tokio::task::yield_now().await; }

    let rows = ssb
        .query_by_type(SsbActionType::ReflectCandidate)
        .expect("query ReflectCandidate rows");
    assert_eq!(
        rows.len(),
        N_CANDIDATES as usize,
        "exactly N_CANDIDATES={N_CANDIDATES} reflect_candidate rows must be written"
    );

    // Collect candidate_index values and verify they are exactly {0,1,2,3}.
    let mut indices: Vec<u32> = rows
        .iter()
        .map(|entry| {
            let payload: SsbPayload =
                rmp_serde::from_slice(entry.payload.as_ref())
                    .expect("decode reflect_candidate payload");
            match payload {
                SsbPayload::ReflectCandidate(p) => p.candidate_index,
                other => panic!("expected ReflectCandidate; got {other:?}"),
            }
        })
        .collect();
    indices.sort_unstable();
    assert_eq!(
        indices,
        vec![0u32, 1, 2, 3],
        "candidate_index values must be {{0,1,2,3}}"
    );
}

// ===========================================================================
// Test 9 — happy-path run() writes exactly 1 'reflect_audit_verdict' row with
//           best_index in 0..N_CANDIDATES.
// ===========================================================================

#[tokio::test]
async fn test_auditor_writes_one_reflect_audit_verdict_row() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("ssb_verdict.db");
    let ssb = new_ssb(db_path);
    let config = default_config();
    let mock = happy_path_mock();

    let sink: Arc<dyn ReflectEventSink> = ssb.clone();
    let pipeline = ChatReflectPipeline::new(config, Some(sink));
    let _output = pipeline
        .run("t9".to_string(), "What is ownership in Rust?", &mock)
        .await;

    // Yield to let the SSB fire-and-forget writer task drain its queue.
    for _ in 0..10 { tokio::task::yield_now().await; }

    let rows = ssb
        .query_by_type(SsbActionType::ReflectAuditVerdict)
        .expect("query ReflectAuditVerdict rows");
    assert_eq!(rows.len(), 1, "exactly 1 reflect_audit_verdict row must be written");

    let decoded: SsbPayload =
        rmp_serde::from_slice(rows[0].payload.as_ref())
            .expect("decode reflect_audit_verdict payload");
    match decoded {
        SsbPayload::ReflectAuditVerdict(p) => {
            assert!(
                (p.best_index as usize) < N_CANDIDATES as usize,
                "best_index={} must be in 0..{N_CANDIDATES}",
                p.best_index
            );
        }
        other => panic!("expected ReflectAuditVerdict; got {other:?}"),
    }
}

// ===========================================================================
// Test 10 — SYNTHESIZE stage makes NO extra inference call.
//            Happy-path breakdown: 1 (triage) + N_CANDIDATES (reasoner) +
//            1 (auditor) = 6 total; SYNTHESIZE adds 0.
// ===========================================================================

#[tokio::test]
async fn test_synthesize_makes_no_extra_inference_call() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("ssb_synth.db");
    let ssb = new_ssb(db_path);
    let config = default_config();

    // Build the happy-path mock and wrap it in Arc so we can read call_count after run().
    let mock = Arc::new(happy_path_mock());

    // SYNTHESIZE is local selection — no extra call beyond 1 triage + 4 reasoner + 1 auditor.
    // Expected total = 1 + N_CANDIDATES + 1 = 6.
    let expected_calls = 1 + N_CANDIDATES + 1;

    // ChatReflectPipeline::run() borrows the client; keep Arc to read call_count after run().
    struct ArcClient(Arc<MockChatInferenceClient>);
    impl ChatInferenceClient for ArcClient {
        fn complete<'a>(&'a self, prompt: &'a str, max_tokens: u32) -> ChatFuture<'a, String> {
            self.0.complete(prompt, max_tokens)
        }
    }

    let mock_ref = mock.clone();
    let arc_client = ArcClient(mock);
    let sink: Arc<dyn ReflectEventSink> = ssb;
    let pipeline = ChatReflectPipeline::new(config, Some(sink));

    let _output = pipeline
        .run("t10".to_string(), "Describe lifetimes.", &arc_client)
        .await;

    assert_eq!(
        mock_ref.call_count(),
        expected_calls,
        "total inference calls must be {expected_calls} \
         (1 triage + {N_CANDIDATES} reasoner + 1 auditor); SYNTHESIZE adds 0"
    );
}

// ===========================================================================
// Test 11 — token_summary.total == sum of stage_tokens.
// ===========================================================================

#[tokio::test]
async fn test_token_summary_total_equals_sum_of_stages() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("ssb_tokens.db");
    let ssb = new_ssb(db_path);
    let config = default_config();
    let mock = happy_path_mock();

    let sink: Arc<dyn ReflectEventSink> = ssb;
    let pipeline = ChatReflectPipeline::new(config, Some(sink));
    let output: ReflectOutput = pipeline
        .run("t11".to_string(), "Summarize closures in Rust.", &mock)
        .await;

    let computed_total: u32 = output.token_summary.stage_tokens.iter().sum();
    assert_eq!(
        output.token_summary.total,
        computed_total,
        "token_summary.total must equal stage_tokens.iter().sum() — \
         got total={} but sum={}",
        output.token_summary.total,
        computed_total
    );
}

// ===========================================================================
// Test 12 — REFINE_CAP is 3.
//
// RefineBench guided-mode evidence: t=2→t=3 yields +12-20pp quality gain;
// t=3→t=4 yields diminishing returns below noise floor. Cap=3 is the
// cost-optimal ceiling for the guided-mode regime.
// [pending-ingest] Liu et al. arXiv — awaiting paper-ingestor ingestion to
// formally ground the N=4 claim above; REFINE_CAP=3 is grounded in
// RefineBench guided-mode data already in workflow/papers/.
// ===========================================================================

#[test]
fn test_refine_cap_is_three() {
    // REFINE_CAP is a pub const in elai_reasoning::chat_reflect.
    // RefineBench guided-mode: t=2→t=3 captures bulk of gain; t=3→t=4 flat.
    assert_eq!(
        REFINE_CAP, 3u32,
        "REFINE_CAP must be 3 per RefineBench guided-mode data \
         (t=2→t=3 yields +12-20pp; t=3→t=4 below noise floor)"
    );
}
