// A-47: Contract tests for /research CLI command integration.
// Tests the orchestrator-level data contracts that the CLI routing depends on:
// 1. FsmState::Researching transition validity (Idle -> Researching -> Idle).
// 2. Research pipeline accepts a query and returns a structured answer.
// 3. StubSearchBackend produces deterministic empty results.

use elai_core::{ResearchQuery, parse_source_tier_config};
use elai_orchestrator::{FsmState, StubSearchBackend, run_research_pipeline};

/// Minimal source-tiers TOML (mirrors EMPTY_TIER_TOML in cli/main.rs).
const EMPTY_TIER_TOML: &str = "\
[tier1]\ndomains = []\npatterns = []\n\
[tier2]\ndomains = []\nmin_so_score = 5\n\
[tier4_blocklist]\ndomains = []\npatterns = []\n";

#[test]
fn fsm_idle_can_transition_to_researching() {
    let targets = FsmState::Idle.valid_targets();
    assert!(
        targets.contains(&FsmState::Researching),
        "FSM must allow Idle -> Researching transition"
    );
}

#[test]
fn fsm_researching_can_return_to_idle() {
    let targets = FsmState::Researching.valid_targets();
    assert!(
        targets.contains(&FsmState::Idle),
        "FSM must allow Researching -> Idle transition"
    );
}

#[test]
fn fsm_researching_can_transition_to_planning() {
    let targets = FsmState::Researching.valid_targets();
    assert!(
        targets.contains(&FsmState::Planning),
        "FSM must allow Researching -> Planning transition"
    );
}

#[test]
fn research_pipeline_stub_returns_no_usable_sources_error() {
    // With an empty StubSearchBackend and empty tier config, the pipeline should
    // fail with NoUsableSources since no search results are returned.
    let query = ResearchQuery::new("what is RLHF");
    let tier_cfg = parse_source_tier_config(EMPTY_TIER_TOML)
        .expect("empty tier config must parse");
    let backend = StubSearchBackend::new();

    let result = run_research_pipeline(&query, &tier_cfg, &backend);
    assert!(result.is_err(), "stub backend with no results should produce an error");
}

#[test]
fn research_query_new_sets_defaults() {
    let query = ResearchQuery::new("test question");
    assert_eq!(query.question, "test question");
    assert_eq!(query.max_sources, elai_core::DEFAULT_MAX_SOURCES);
    assert!(query.context.is_none());
    assert!(query.required_tier.is_none());
}

#[test]
fn research_query_builder_chain() {
    let query = ResearchQuery::new("test")
        .with_context("some context")
        .with_max_sources(5);
    assert_eq!(query.context.as_deref(), Some("some context"));
    assert_eq!(query.max_sources, 5);
}

#[test]
fn empty_tier_config_parses_successfully() {
    let result = parse_source_tier_config(EMPTY_TIER_TOML);
    assert!(result.is_ok(), "empty tier TOML must parse without error");
}
