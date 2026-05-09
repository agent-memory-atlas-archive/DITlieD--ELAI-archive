// Track A.6 contract test: GroundingCacheEvent SSB variant + cache hit/miss emission.
//
// After A.6 production code lands:
//   - SsbActionType::GroundingCacheEvent (enum variant added in ssb.rs)
//   - SsbPayload::GroundingCacheEvent(GroundingCacheEventPayload) (variant added)
//   - GroundingCacheEventPayload (struct added to ssb.rs)
//   - research_decompose accepts Option<&Arc<Ssb>> (A.6 wired)
//   - ground_or_cached emits GroundingCacheEvent rows on hit/miss
//
// Half 1 (A.6.1): pure SSB variant roundtrip — construct GroundingCacheEventPayload,
//   verify action_type() and msgpack round-trip.
// Half 2 (A.6.2): drive research_decompose twice on the same key with a real Ssb
//   handle; assert the second call emits at least one hit row in the SSB.
//
// Fixture: hermetic CannedClient (inline, no live sidecar required).
// Canned JSON: minimal but structurally valid per grounding_client.rs parser.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use elai_core::clock::SeededClock;
use elai_orchestrator::grounding_client::{
    parse_research_report_json, GroundingClient, GroundingError, ResearchLevel,
};
use elai_orchestrator::research_tree::{
    research_decompose, ExecutionMode, ResearchCache, ResearchReport, DEFAULT_CACHE_MAX_ENTRIES,
    DEFAULT_CACHE_TTL_SECONDS,
};
use elai_orchestrator::ssb::{
    GroundingCacheEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Canned JSON: minimal shapes that satisfy parse_research_report_json +
// extract_concepts (needs "concept" or "framework" keyword in sub_results).
// ---------------------------------------------------------------------------

const CANNED_L0: &str = r#"{"question":"cache test","answer":"Ans.","alternatives":[{"name":"a","description":"d","citations":[],"confidence":0.8,"trade_offs":[]}],"contradictions":[],"sub_results":[{"question":"What concept applies?","answer":"LRU","citations":[],"confidence":0.9,"evidence_level":"MultiSource"},{"question":"What framework applies?","answer":"in-mem","citations":[],"confidence":0.7,"evidence_level":"MultiSource"}],"snapshot":{"nodes":[],"convergence":"Converged","total_iterations":1,"elapsed":{"secs":0,"nanos":0}},"quality_metrics":[],"convergence":"Converged"}"#;
const CANNED_L1: &str = r#"{"question":"phases?","answer":"d->i","alternatives":[{"name":"seq","description":"seq","citations":[],"confidence":0.9,"trade_offs":[]}],"contradictions":[],"sub_results":[{"question":"What phase?","answer":"phase-a","citations":[],"confidence":0.9,"evidence_level":"MultiSource"}],"snapshot":{"nodes":[],"convergence":"Converged","total_iterations":1,"elapsed":{"secs":0,"nanos":0}},"quality_metrics":[],"convergence":"Converged"}"#;
const CANNED_L2: &str = r#"{"question":"steps?","answer":"s1->s2","alternatives":[{"name":"tdd","description":"test-first","citations":[],"confidence":0.9,"trade_offs":[]}],"contradictions":[],"sub_results":[{"question":"What step?","answer":"step-a","citations":[],"confidence":0.9,"evidence_level":"MultiSource"}],"snapshot":{"nodes":[],"convergence":"Converged","total_iterations":1,"elapsed":{"secs":0,"nanos":0}},"quality_metrics":[],"convergence":"Converged"}"#;

struct CannedClient {
    calls: Arc<AtomicUsize>,
}
impl CannedClient {
    fn new() -> Self {
        Self { calls: Arc::new(AtomicUsize::new(0)) }
    }
}
impl GroundingClient for CannedClient {
    async fn ground(&self, _prompt: &str, level: ResearchLevel) -> Result<ResearchReport, GroundingError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        match level {
            ResearchLevel::Idea => parse_research_report_json(CANNED_L0),
            ResearchLevel::Concept => parse_research_report_json(CANNED_L1),
            ResearchLevel::Phase | ResearchLevel::Step => parse_research_report_json(CANNED_L2),
        }
    }
    async fn synthesize_text(&self, _prompt: &str) -> Result<String, GroundingError> {
        Err(GroundingError::EmptyResponse)
    }
}

// ---------------------------------------------------------------------------
// A.6.1a — GroundingCacheEventPayload "hit" roundtrips through msgpack.
// RED: struct and variant do not exist yet.
// ---------------------------------------------------------------------------

#[test]
fn grounding_cache_event_hit_roundtrips_msgpack() {
    let payload = GroundingCacheEventPayload {
        kind: "hit".into(),
        key_fingerprint: "abcdef12".into(),
        level: "L0".into(),
        elapsed_us: 42,
    };
    let wrapped = SsbPayload::GroundingCacheEvent(payload);
    assert_eq!(wrapped.action_type(), SsbActionType::GroundingCacheEvent);

    let bytes = rmp_serde::to_vec_named(&wrapped)
        .expect("GroundingCacheEvent hit msgpack serialize");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("GroundingCacheEvent hit msgpack deserialize");

    match decoded {
        SsbPayload::GroundingCacheEvent(p) => {
            assert_eq!(p.kind, "hit");
            assert_eq!(p.key_fingerprint, "abcdef12");
            assert_eq!(p.level, "L0");
            assert_eq!(p.elapsed_us, 42);
        }
        other => panic!("expected GroundingCacheEvent; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// A.6.1b — GroundingCacheEventPayload "miss" roundtrips.
// ---------------------------------------------------------------------------

#[test]
fn grounding_cache_event_miss_roundtrips_msgpack() {
    let payload = GroundingCacheEventPayload {
        kind: "miss".into(),
        key_fingerprint: "01234567".into(),
        level: "L1".into(),
        elapsed_us: 1_250,
    };
    let bytes = rmp_serde::to_vec_named(&SsbPayload::GroundingCacheEvent(payload))
        .expect("miss serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("miss deserialize");
    match decoded {
        SsbPayload::GroundingCacheEvent(p) => {
            assert_eq!(p.kind, "miss");
            assert_eq!(p.level, "L1");
        }
        other => panic!("expected GroundingCacheEvent; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// A.6.2 — second research_decompose call makes fewer GroundingClient.ground()
//          calls than the first (cache hit path verified indirectly).
//          SSB row assertion emits WARN+skip until A.6 wires Arc<Ssb> in.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn grounding_cache_second_call_emits_hit_row() {
    // Create a real Ssb so we can assert on GroundingCacheEvent rows.
    let tmp = TempDir::new().expect("tempdir for A.6 contract test");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(SeededClock::new());
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: tmp.path().join("a6_test.db"),
                pool_size: Some(2),
                project_id: "a6-contract".into(),
                session_id: "a6-session".into(),
            },
            clock,
        )
        .expect("Ssb::new must succeed for A.6 contract test"),
    );

    let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);
    let client = CannedClient::new();
    let calls_ref = client.calls.clone();

    // First call: populates cache keys for all grounded levels.
    // Passes Some(&ssb) so ground_or_cached emits GroundingCacheEvent(miss) rows.
    research_decompose("cache test", ExecutionMode::Autonomous, &mut cache, &client, Some(&ssb), None)
        .await
        .expect("first research_decompose must succeed with canned client");
    let calls_first = calls_ref.load(Ordering::SeqCst);

    // Second call: cache.peek() serves already-computed keys — fewer ground() calls.
    // Passes Some(&ssb) so ground_or_cached emits GroundingCacheEvent(hit) rows.
    research_decompose("cache test", ExecutionMode::Autonomous, &mut cache, &client, Some(&ssb), None)
        .await
        .expect("second research_decompose must succeed (cache-hit path)");
    let calls_second = calls_ref.load(Ordering::SeqCst);

    assert!(
        calls_second <= calls_first,
        "second research_decompose must not exceed first call's grounding count \
         (cache must serve some keys). first={calls_first}, total_after_second={calls_second}",
    );

    // A.6 SSB assertion: the second call must have emitted at least one hit row.
    let rows = ssb
        .query_by_type(SsbActionType::GroundingCacheEvent)
        .expect("query_by_type(GroundingCacheEvent) must not fail");
    assert!(
        rows.iter().any(|r| {
            if let Ok(elai_orchestrator::ssb::SsbPayload::GroundingCacheEvent(ref p)) =
                rmp_serde::from_slice(&r.payload)
            {
                p.kind == "hit"
            } else {
                false
            }
        }),
        "second research_decompose call must emit at least one GroundingCacheEvent(hit) row; \
         total GroundingCacheEvent rows = {}",
        rows.len(),
    );
}
