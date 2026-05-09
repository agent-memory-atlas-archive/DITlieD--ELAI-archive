// Session 4D.10 contract test: GroundingClient trait, SidecarGroundingClient,
// ResearchLevel enum, GroundingError variants, and async research_decompose
// wiring. Native async fn in traits (no async_trait crate) + static dispatch.

use elai_orchestrator::grounding_client::{
    parse_research_report_json, CannedSynthesisGroundingClient, GroundingClient, GroundingError,
    ResearchLevel, SidecarGroundingClient, SyntheticGroundingClient,
};
use elai_orchestrator::research_tree::{
    research_decompose, ExecutionMode, ResearchCache, ResearchReport, ResearchTreeError,
    DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS,
};
use elai_orchestrator::research_pipeline::{phase_f_synthesize, ClassifiedResults};
use elai_core::research_types::{ResearchQuery, VerificationLevel, VerifiedClaim};
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

    async fn synthesize_text(&self, _prompt: &str) -> Result<String, GroundingError> {
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

    async fn synthesize_text(&self, prompt: &str) -> Result<String, GroundingError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.synthesize_text(prompt).await
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
        research_decompose("test idea", ExecutionMode::Autonomous, &mut cache, &client, None, None).await;
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
        None, // Track A.6: no SSB handle in this test.
        None, // B-7: no tree emitter in this test.
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

// ---------------------------------------------------------------------------
// Contract: Phase F — happy-path LLM branch.
// CannedSynthesisGroundingClient returns cited prose; phase_f_synthesize must
// use the LLM text directly (not the deterministic fallback).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_phase_f_synthesize_happy_path_with_cited_prose() {
    // Cited prose containing `[1]` and `[2]` — must satisfy the numeric-citation guard.
    let canned = "RAG works by retrieving relevant documents first [1]. \
                  The retrieval step narrows the context window [2]."
        .to_owned();

    let grounding = CannedSynthesisGroundingClient::new(canned.clone());

    let query = ResearchQuery::new("How does RAG work?");
    let claims = vec![
        VerifiedClaim {
            claim: "RAG retrieves relevant documents first.".to_owned(),
            supporting_facts: vec![],
            verification_level: VerificationLevel::Verified,
            independent_source_count: 2,
        },
        VerifiedClaim {
            claim: "The retrieval step narrows the context window.".to_owned(),
            supporting_facts: vec![],
            verification_level: VerificationLevel::Verified,
            independent_source_count: 2,
        },
    ];
    let classified = ClassifiedResults {
        usable: Vec::new(),
        rejected: Vec::new(),
        classification_errors: Vec::new(),
    };

    let answer = phase_f_synthesize(&query, &claims, &classified, &grounding).await;

    // LLM path must have been used — answer_text must equal the canned prose, NOT
    // the deterministic "[1] ... [2] ..." concatenation produced by the fallback.
    assert_eq!(
        answer.answer_text, canned,
        "phase_f_synthesize must return the LLM-supplied cited prose unchanged"
    );

    // Sanity: the answer is attributed to the right question.
    assert_eq!(answer.question, "How does RAG work?");
}

// ---------------------------------------------------------------------------
// Contract: Phase F — negative path (existing behaviour: Err -> fallback fires).
// SyntheticGroundingClient.synthesize_text always errors; the deterministic
// concatenation must run instead.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_phase_f_synthesize_fallback_fires_on_synthesize_error() {
    let grounding = SyntheticGroundingClient::new();

    let query = ResearchQuery::new("fallback test");
    let claims = vec![VerifiedClaim {
        claim: "some fact".to_owned(),
        supporting_facts: vec![],
        verification_level: VerificationLevel::Supported,
        independent_source_count: 1,
    }];
    let classified = ClassifiedResults {
        usable: Vec::new(),
        rejected: Vec::new(),
        classification_errors: Vec::new(),
    };

    let answer = phase_f_synthesize(&query, &claims, &classified, &grounding).await;

    // Deterministic path formats claims as "[N] <claim>".
    assert!(
        answer.answer_text.contains("[1]"),
        "deterministic fallback must produce [1]-prefixed text; got: {}",
        answer.answer_text
    );
    // Must NOT be the LLM canned prose.
    assert!(
        !answer.answer_text.contains("RAG works"),
        "fallback must not contain LLM prose"
    );
}
