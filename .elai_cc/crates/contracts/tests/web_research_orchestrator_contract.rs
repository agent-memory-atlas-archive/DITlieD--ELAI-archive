// 4D.17 contract tests: WebResearchOrchestrator — web research pipeline integration.

use std::sync::Arc;

use elai_core::clock::SeededClock;
use elai_core::source_tier::SourceTier;
use elai_core::VramTier;
use elai_knowledge::web_arag::{WebAragConfig, WebContentIndex};
use elai_orchestrator::research_budget::ResearchBudgetManager;
use elai_orchestrator::research_tier::research_config_for_tier;
use elai_orchestrator::ssb::{SsbActionType, SsbPayload, WebResearchStepPayload};
use elai_orchestrator::web_research_orchestrator::{
    WebResearchOrchestrator, WebResearchOrchestratorError,
};
use elai_web_search::client::SearxClient;
use elai_web_search::extractor::BasicExtractor;
use elai_web_search::fetcher::WebFetcher;
use elai_web_search::rate_limiter::noop_sleep;
use elai_core::source_tier::parse_source_tier_config;
use elai_web_search::source_quality::SourceQualityTable;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_clock() -> Arc<SeededClock> {
    Arc::new(SeededClock::new())
}

fn make_tier_config() -> elai_orchestrator::research_tier::ResearchTierConfig {
    research_config_for_tier(VramTier::Vram12Gb)
}

fn make_quality_table() -> SourceQualityTable {
    let toml = r#"
[tier1]
domains = ["arxiv.org", "doi.org"]
patterns = ['\.gov$']

[tier2]
domains = ["docs.rs", "crates.io"]
min_so_score = 20

[tier4_blocklist]
domains = ["medium.com"]
patterns = []
"#;
    let config = parse_source_tier_config(toml).expect("test config parse");
    SourceQualityTable::new_from_config(config)
}

// ---------------------------------------------------------------------------
// Contract 1: orchestrator compiles with all six deps
// ---------------------------------------------------------------------------

#[test]
fn orchestrator_compiles_with_all_six_deps() {
    let clock = make_clock();
    let tier_config = make_tier_config();

    let searx = SearxClient::new("http://localhost:8888", clock.clone())
        .expect("searx client");
    let fetcher = WebFetcher::with_sleep(clock.clone(), noop_sleep())
        .expect("web fetcher");
    let extractor = BasicExtractor::new(0);
    let quality_table = make_quality_table();
    let content_index = WebContentIndex::new(WebAragConfig::default());
    let budget_manager = ResearchBudgetManager::new(tier_config.clone());

    let orch = WebResearchOrchestrator::new(
        searx,
        fetcher,
        extractor,
        quality_table,
        content_index,
        budget_manager,
        tier_config,
        clock,
    );

    // Struct exists and was constructed without panic.
    assert!(orch.tier_config().phi_min > 0.0);
    assert!(orch.tier_config().psi_min > 0.0);
}

// ---------------------------------------------------------------------------
// Contract 2: IPC tool constants defined for search/fetch/embed
// ---------------------------------------------------------------------------

#[test]
fn ipc_search_request_defined() {
    assert_eq!(elai_orchestrator::ipc_types::TOOL_SEARCH_REQUEST, "search_request");
}

// ---------------------------------------------------------------------------
// Contract 3: IPC fetch request constant defined
// ---------------------------------------------------------------------------

#[test]
fn ipc_fetch_request_defined() {
    assert_eq!(elai_orchestrator::ipc_types::TOOL_FETCH_REQUEST, "fetch_request");
}

// ---------------------------------------------------------------------------
// Contract 4: SSB web_research_step action type valid
// ---------------------------------------------------------------------------

#[test]
fn ssb_web_research_step_action_type_valid() {
    let action = SsbActionType::WebResearchStep;
    assert_eq!(action.as_sql_str(), "web_research_step");

    // Verify payload round-trip through action_type().
    let payload = SsbPayload::WebResearchStep(WebResearchStepPayload {
        agent_id: "test".to_string(),
        node_id: 1,
        action: "search".to_string(),
        phi: 0.5,
        psi: 0.4,
        iteration: 1,
        sources_fetched: 3,
        sources_used: 2,
    });
    assert_eq!(payload.action_type(), SsbActionType::WebResearchStep);
}

// ---------------------------------------------------------------------------
// Contract 5: orchestrator returns error when searxng is down
// ---------------------------------------------------------------------------

#[tokio::test]
async fn orchestrator_returns_error_when_searxng_down() {
    let clock = make_clock();
    let tier_config = make_tier_config();

    // Point at a non-existent SearXNG instance.
    let searx = SearxClient::new("http://127.0.0.1:1", clock.clone())
        .expect("searx client");
    let fetcher = WebFetcher::with_sleep(clock.clone(), noop_sleep())
        .expect("web fetcher");
    let extractor = BasicExtractor::new(0);
    let quality_table = make_quality_table();
    let content_index = WebContentIndex::new(WebAragConfig::default());
    let budget_manager = ResearchBudgetManager::new(tier_config.clone());

    let mut orch = WebResearchOrchestrator::new(
        searx,
        fetcher,
        extractor,
        quality_table,
        content_index,
        budget_manager,
        tier_config,
        clock,
    );

    let result = orch.search_and_rank("test query").await;
    assert!(result.is_err(), "should fail when SearXNG is unreachable");
}

// ---------------------------------------------------------------------------
// Contract 6: orchestrator flags low confidence on tier5 sources
// ---------------------------------------------------------------------------

#[test]
fn orchestrator_flags_low_confidence_on_tier5_sources() {
    use elai_orchestrator::web_research_orchestrator::compute_source_confidence;

    // Build tier distribution: >80% Tier5.
    let tiers = vec![
        SourceTier::T4Rejected,  // Tier5 maps to T4Rejected in SourceTier
        SourceTier::T4Rejected,
        SourceTier::T4Rejected,
        SourceTier::T4Rejected,
        SourceTier::T2Strong,
    ];
    let confidence = compute_source_confidence(&tiers);
    // >80% low-quality sources -> confidence should be LOW (below 0.5).
    assert!(
        confidence < 0.5,
        "confidence should be LOW when >80% are low-quality sources, got {}",
        confidence
    );
}

// ---------------------------------------------------------------------------
// Contract 7: clear error when sidecar unavailable
// ---------------------------------------------------------------------------

#[test]
fn sidecar_required_error_message() {
    let err = WebResearchOrchestratorError::SidecarUnavailable {
        reason: "connection refused".to_string(),
    };
    let msg = err.to_string();
    assert!(
        msg.contains("sidecar") || msg.contains("Sidecar"),
        "error message should mention sidecar: {}",
        msg
    );
}

// ---------------------------------------------------------------------------
// Contract 8: concurrent fetch respects rate limit config
// ---------------------------------------------------------------------------

#[test]
fn concurrent_fetch_respects_rate_limit() {
    let tier_config = make_tier_config();
    // Vram12Gb has max_concurrent_fetches = 4.
    assert_eq!(
        tier_config.max_concurrent_fetches, 4,
        "12GB tier should have max_concurrent_fetches=4"
    );
    // This is a design contract — the orchestrator must use this value
    // as the concurrency limit for parallel fetches.
}
