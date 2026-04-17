// Session 4D.10 contract test: GroundingClient trait, SidecarGroundingClient,
// ResearchLevel enum, GroundingError variants, and async research_decompose
// wiring. Native async fn in traits (no async_trait crate) + static dispatch.

use elai_orchestrator::grounding_client::{
    parse_research_report_json, GroundingClient, GroundingError, ResearchLevel,
    SidecarGroundingClient, SyntheticGroundingClient,
};
use elai_orchestrator::research_tree::{
    research_decompose, ExecutionMode, ResearchCache, ResearchReport, ResearchTreeError,
    DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Mock client: errors on every call.
// ---------------------------------------------------------------------------

struct AlwaysErrClient;

impl GroundingClient for AlwaysErrClient {
    async fn ground(
        &self,
        _prompt: &str,
        _level: ResearchLevel,
    ) -> Result<ResearchReport, GroundingError> {
        Err(GroundingError::SidecarUnavailable {
            url: "http://localhost:8100".to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Mock client: counts calls, delegates to SyntheticGroundingClient.
// ---------------------------------------------------------------------------

struct CountingClient {
    calls: Arc<AtomicUsize>,
    inner: SyntheticGroundingClient,
}

impl GroundingClient for CountingClient {
    async fn ground(
        &self,
        prompt: &str,
        level: ResearchLevel,
    ) -> Result<ResearchReport, GroundingError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.ground(prompt, level).await
    }
}

// ---------------------------------------------------------------------------
// Contract: ResearchLevel has the four expected variants and stable template
// names matching the four prompt template files.
// ---------------------------------------------------------------------------

#[test]
fn research_level_has_four_variants_with_stable_template_names() {
    assert_eq!(ResearchLevel::Idea.template_name(), "l0_idea");
    assert_eq!(ResearchLevel::Concept.template_name(), "l1_concept");
    assert_eq!(ResearchLevel::Phase.template_name(), "l2_phase");
    assert_eq!(ResearchLevel::Step.template_name(), "l3_step");
}

// ---------------------------------------------------------------------------
// Contract: SidecarGroundingClient exposes explicit base_url and timeout; its
// default points at port 8100 (not 8080).
// ---------------------------------------------------------------------------

#[test]
fn sidecar_grounding_client_constructs_with_explicit_config() {
    let client =
        SidecarGroundingClient::new("http://localhost:8100".to_string(), Duration::from_secs(30));
    assert_eq!(client.base_url(), "http://localhost:8100");
    assert_eq!(client.timeout(), Duration::from_secs(30));

    let default_client = SidecarGroundingClient::default();
    assert!(
        default_client.base_url().contains("8100"),
        "default base_url must target port 8100, got {}",
        default_client.base_url()
    );
}

// ---------------------------------------------------------------------------
// Contract: research_decompose is async and propagates GroundingError through
// ResearchTreeError when the client fails.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn research_decompose_surfaces_grounding_error_as_tree_error() {
    let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);
    let client = AlwaysErrClient;
    let result =
        research_decompose("test idea", ExecutionMode::Autonomous, &mut cache, &client).await;
    match result {
        Err(ResearchTreeError::GroundingFailed { .. }) => {}
        Err(other) => panic!("expected GroundingFailed, got {other:?}"),
        Ok(_) => panic!("expected Err, got Ok"),
    }
}

// ---------------------------------------------------------------------------
// Contract: research_decompose invokes the client at least once (no silent
// synthetic fallback). SyntheticGroundingClient returns deterministic reports.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn research_decompose_invokes_client_and_succeeds_with_synthetic() {
    let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);
    let counting = CountingClient {
        calls: Arc::new(AtomicUsize::new(0)),
        inner: SyntheticGroundingClient::new(),
    };
    let calls_handle = counting.calls.clone();
    let decomp = research_decompose(
        "build a web server",
        ExecutionMode::Autonomous,
        &mut cache,
        &counting,
    )
    .await
    .expect("synthetic client must succeed");
    assert!(!decomp.concepts.is_empty(), "must produce concepts");
    assert!(
        calls_handle.load(Ordering::SeqCst) >= 1,
        "client must be invoked at least once (no synthetic fallback)"
    );
}

// ---------------------------------------------------------------------------
// Contract: GroundingError is Send + Sync + 'static and each variant's Display
// is non-empty (required for telemetry and SSB Finding rows).
// ---------------------------------------------------------------------------

#[test]
fn grounding_error_variants_are_typed_and_thread_safe() {
    fn assert_send_sync<T: Send + Sync + 'static>() {}
    assert_send_sync::<GroundingError>();

    let variants = [
        GroundingError::SidecarUnavailable { url: "http://x".to_string() },
        GroundingError::Timeout {
            url: "http://x".to_string(),
            after: Duration::from_secs(1),
        },
        GroundingError::HttpStatus {
            status: 500,
            url: "http://x".to_string(),
        },
        GroundingError::InvalidJson {
            reason: "truncated".to_string(),
        },
        GroundingError::EmptyResponse,
        GroundingError::RequestFailed {
            reason: "tls handshake".to_string(),
        },
    ];
    for v in variants {
        assert!(!format!("{v}").is_empty());
    }
}

// ---------------------------------------------------------------------------
// Contract: JSON parser rejects empty and malformed input with typed errors.
// ---------------------------------------------------------------------------

#[test]
fn grounding_json_parser_rejects_empty_and_malformed() {
    match parse_research_report_json("") {
        Err(GroundingError::EmptyResponse) => {}
        other => panic!("expected EmptyResponse, got {other:?}"),
    }
    match parse_research_report_json("   \n\t  ") {
        Err(GroundingError::EmptyResponse) => {}
        other => panic!("expected EmptyResponse, got {other:?}"),
    }
    match parse_research_report_json("{not json}") {
        Err(GroundingError::InvalidJson { .. }) => {}
        other => panic!("expected InvalidJson, got {other:?}"),
    }
    match parse_research_report_json(r#"{"unrelated": 42}"#) {
        Err(GroundingError::InvalidJson { .. }) => {}
        other => panic!("expected InvalidJson, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract: parser strips a ```json ... ``` fenced wrapper and parses the
// inner JSON (common LLM output form).
// ---------------------------------------------------------------------------

#[test]
fn grounding_json_parser_accepts_fenced_payload() {
    let synth = SyntheticGroundingClient::new();
    let report: ResearchReport = synth.build_report_for_test("topic");
    let raw = serde_json::to_string(&report).expect("serialize stub");
    let fenced = format!("```json\n{raw}\n```");
    let parsed = parse_research_report_json(&fenced).expect("must accept fenced payload");
    assert_eq!(parsed.question, "topic");
}
