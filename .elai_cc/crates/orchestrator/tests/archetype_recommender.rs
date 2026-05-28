//! E-11 catalog+recommender slice -- recommender behaviour contract.
//!
//! Each of the 9 `ProjectClass` variants is exercised. The recommender
//! is fully deterministic for a given `(class, brief)` -- duplicate
//! calls return equal recommendations.

use elai_orchestrator::architecture::{
    recommend_archetype, ArchetypeCatalog, RecommendationConfidence, SaasArchetype,
};
use elai_orchestrator::idea_framing::ProjectClass;

fn recommend(class: ProjectClass, brief: &str) -> (SaasArchetype, RecommendationConfidence, usize) {
    let r = recommend_archetype(class, brief);
    (r.primary, r.confidence, r.clarifying_questions.len())
}

#[test]
fn saas_dashboard_maps_high() {
    let (primary, conf, qcount) = recommend(ProjectClass::SaaSDashboard, "");
    assert_eq!(primary, SaasArchetype::SaasDashboard);
    assert_eq!(conf, RecommendationConfidence::High);
    assert_eq!(qcount, 0, "High confidence emits zero clarifying questions");
}

#[test]
fn library_or_sdk_maps_high_to_developer_api() {
    let (primary, conf, _q) = recommend(ProjectClass::LibraryOrSDK, "");
    assert_eq!(primary, SaasArchetype::DeveloperApi);
    assert_eq!(conf, RecommendationConfidence::High);
}

#[test]
fn cli_tool_maps_high_to_developer_api() {
    let (primary, conf, _q) = recommend(ProjectClass::CLITool, "");
    assert_eq!(primary, SaasArchetype::DeveloperApi);
    assert_eq!(conf, RecommendationConfidence::High);
}

#[test]
fn research_tool_maps_medium_to_internal_tooling() {
    let (primary, conf, qcount) = recommend(ProjectClass::ResearchTool, "");
    assert_eq!(primary, SaasArchetype::InternalTooling);
    assert_eq!(conf, RecommendationConfidence::Medium);
    assert!(qcount >= 2, "Medium confidence must emit >=2 clarifying questions");
}

#[test]
fn consumer_mobile_app_maps_medium_to_consumer_social() {
    let (primary, conf, _q) = recommend(ProjectClass::ConsumerMobileApp, "");
    assert_eq!(primary, SaasArchetype::ConsumerSocial);
    assert_eq!(conf, RecommendationConfidence::Medium);
}

#[test]
fn web_app_with_empty_brief_falls_low() {
    let (primary, conf, qcount) = recommend(ProjectClass::WebApp, "");
    assert_eq!(primary, SaasArchetype::SaasDashboard);
    assert_eq!(conf, RecommendationConfidence::Low);
    assert!(qcount >= 3, "Low confidence must emit >=3 clarifying questions");
}

#[test]
fn web_app_with_billing_keyword_picks_marketplace() {
    let (primary, conf, _q) = recommend(
        ProjectClass::WebApp,
        "we need a marketplace with stripe billing and subscription tiers",
    );
    assert_eq!(primary, SaasArchetype::B2bMarketplace);
    assert_eq!(conf, RecommendationConfidence::Medium);
}

#[test]
fn web_app_with_api_keyword_picks_developer_api() {
    let (primary, conf, _q) = recommend(
        ProjectClass::WebApp,
        "build an OpenAPI surface with an SDK and rate-limited endpoint",
    );
    assert_eq!(primary, SaasArchetype::DeveloperApi);
    assert_eq!(conf, RecommendationConfidence::Medium);
}

#[test]
fn web_app_with_feed_keyword_picks_consumer_social() {
    let (primary, conf, _q) = recommend(
        ProjectClass::WebApp,
        "build a social feed with posts followers and share functionality",
    );
    assert_eq!(primary, SaasArchetype::ConsumerSocial);
    assert_eq!(conf, RecommendationConfidence::Medium);
}

#[test]
fn desktop_app_with_empty_brief_falls_low_to_internal_tooling() {
    let (primary, conf, qcount) = recommend(ProjectClass::DesktopApp, "");
    assert_eq!(primary, SaasArchetype::InternalTooling);
    assert_eq!(conf, RecommendationConfidence::Low);
    assert!(qcount >= 3);
}

#[test]
fn game_mod_with_empty_brief_falls_low_to_consumer_social() {
    let (primary, conf, _q) = recommend(ProjectClass::GameMod, "");
    assert_eq!(primary, SaasArchetype::ConsumerSocial);
    assert_eq!(conf, RecommendationConfidence::Low);
}

#[test]
fn other_class_with_empty_brief_falls_low() {
    let (primary, conf, qcount) = recommend(ProjectClass::Other, "");
    assert_eq!(primary, SaasArchetype::ConsumerSocial);
    assert_eq!(conf, RecommendationConfidence::Low);
    assert!(qcount >= 3);
}

#[test]
fn recommender_is_deterministic() {
    let r1 = recommend_archetype(ProjectClass::Other, "build an internal admin ops console");
    let r2 = recommend_archetype(ProjectClass::Other, "build an internal admin ops console");
    assert_eq!(r1.primary, r2.primary);
    assert_eq!(r1.confidence, r2.confidence);
    assert_eq!(r1.clarifying_questions, r2.clarifying_questions);
    assert_eq!(r1.prior_art_pointers.len(), r2.prior_art_pointers.len());
}

#[test]
fn clarifying_questions_drawn_from_catalog_profile() {
    // Recommender draws from ArchetypeCatalog::entry_for(primary)
    // .scope_clarity_profile. Medium = 2; Low = 3.
    let r = recommend_archetype(ProjectClass::ResearchTool, "");
    let entry = ArchetypeCatalog::entry_for(r.primary);
    for q in &r.clarifying_questions {
        assert!(
            entry.scope_clarity_profile.contains(&q.as_str()),
            "clarifying question '{}' must come from the archetype's profile",
            q
        );
    }
}

#[test]
fn high_confidence_returns_zero_clarifying_questions() {
    let r = recommend_archetype(ProjectClass::SaaSDashboard, "");
    assert_eq!(r.confidence, RecommendationConfidence::High);
    assert_eq!(r.clarifying_questions.len(), 0);
}

#[test]
fn recommendation_confidence_wire_labels_are_stable() {
    assert_eq!(RecommendationConfidence::High.as_snake_case(), "high");
    assert_eq!(RecommendationConfidence::Medium.as_snake_case(), "medium");
    assert_eq!(RecommendationConfidence::Low.as_snake_case(), "low");

    for c in [
        RecommendationConfidence::High,
        RecommendationConfidence::Medium,
        RecommendationConfidence::Low,
    ] {
        assert_eq!(
            RecommendationConfidence::from_snake_case(c.as_snake_case()),
            Some(c)
        );
    }
    assert_eq!(
        RecommendationConfidence::from_snake_case("nope"),
        None
    );
}

#[test]
fn min_clarifying_questions_per_arm_is_stable() {
    assert_eq!(RecommendationConfidence::High.min_clarifying_questions(), 0);
    assert_eq!(RecommendationConfidence::Medium.min_clarifying_questions(), 2);
    assert_eq!(RecommendationConfidence::Low.min_clarifying_questions(), 3);
}
