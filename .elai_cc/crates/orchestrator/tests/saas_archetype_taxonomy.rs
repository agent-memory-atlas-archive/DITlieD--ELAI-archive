//! E-11 catalog+recommender slice -- taxonomy tests for the 5-archetype
//! `SaasArchetype` enum + the `ALL_SAAS_ARCHETYPES` const slice.
//!
//! These tests are the canonical fence against E-50.A negative-space
//! guard #1 drift: the catalog enum MUST be exactly `SaasArchetype` with
//! the 5 variants from `workflow/archetypes/README.md`, snake_case
//! serde discriminants matching `ProjectClass::SaaSDashboard`'s
//! `"saas_dashboard"` discriminant.

use elai_orchestrator::architecture::{
    SaasArchetype, ALL_SAAS_ARCHETYPES,
};

#[test]
fn taxonomy_has_exactly_five_variants() {
    assert_eq!(
        ALL_SAAS_ARCHETYPES.len(),
        5,
        "5-archetype taxonomy is locked by workflow/archetypes/README.md \
         + e50.A guard. Bumping requires a README + guard update first."
    );
}

#[test]
fn taxonomy_declaration_order_is_stable() {
    // Stability assertion: the declaration order is the dashboard
    // surface render order. Reordering requires a UI test + dashboard
    // diff review.
    assert_eq!(ALL_SAAS_ARCHETYPES[0], SaasArchetype::SaasDashboard);
    assert_eq!(ALL_SAAS_ARCHETYPES[1], SaasArchetype::B2bMarketplace);
    assert_eq!(ALL_SAAS_ARCHETYPES[2], SaasArchetype::InternalTooling);
    assert_eq!(ALL_SAAS_ARCHETYPES[3], SaasArchetype::DeveloperApi);
    assert_eq!(ALL_SAAS_ARCHETYPES[4], SaasArchetype::ConsumerSocial);
}

#[test]
fn snake_case_round_trip_is_lossless() {
    for arch in ALL_SAAS_ARCHETYPES {
        let snake = arch.as_snake_case();
        let parsed = SaasArchetype::from_snake_case(snake).unwrap_or_else(|| {
            panic!("snake_case '{}' must round-trip back to its archetype", snake)
        });
        assert_eq!(parsed, *arch);
    }
}

#[test]
fn snake_case_discriminants_match_workflow_archetypes_readme() {
    // The 5 snake_case labels are LOCKED by the README + by
    // ProjectClass::SaaSDashboard's existing serde discriminant
    // ("saas_dashboard"). Lock-in any drift.
    assert_eq!(SaasArchetype::SaasDashboard.as_snake_case(), "saas_dashboard");
    assert_eq!(SaasArchetype::B2bMarketplace.as_snake_case(), "b2b_marketplace");
    assert_eq!(SaasArchetype::InternalTooling.as_snake_case(), "internal_tooling");
    assert_eq!(SaasArchetype::DeveloperApi.as_snake_case(), "developer_api");
    assert_eq!(SaasArchetype::ConsumerSocial.as_snake_case(), "consumer_social");
}

#[test]
fn display_labels_are_human_readable() {
    // Dashboard render contract: the display label is what the
    // SaasArchetypeRecommenderPanel renders (when the sibling
    // dashboard slice ships). Stable across versions.
    assert_eq!(SaasArchetype::SaasDashboard.display_label(), "SaaS Dashboard");
    assert_eq!(SaasArchetype::B2bMarketplace.display_label(), "B2B Marketplace");
    assert_eq!(SaasArchetype::InternalTooling.display_label(), "Internal Tooling");
    assert_eq!(SaasArchetype::DeveloperApi.display_label(), "Developer API");
    assert_eq!(SaasArchetype::ConsumerSocial.display_label(), "Consumer Social");
}

#[test]
fn unknown_snake_case_returns_none() {
    assert!(SaasArchetype::from_snake_case("nope").is_none());
    assert!(SaasArchetype::from_snake_case("").is_none());
    assert!(SaasArchetype::from_snake_case("SAAS_DASHBOARD").is_none());
}

#[test]
fn serde_round_trip_matches_snake_case() {
    // The serde `rename_all = "snake_case"` derive must produce the
    // same wire label as `as_snake_case`.
    for arch in ALL_SAAS_ARCHETYPES {
        let json = serde_json::to_string(arch).expect("serialize");
        let expected = format!("\"{}\"", arch.as_snake_case());
        assert_eq!(json, expected, "serde discriminant must match as_snake_case");
        let parsed: SaasArchetype = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, *arch);
    }
}
