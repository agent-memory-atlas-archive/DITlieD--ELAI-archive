//! E-11 Tier-B catalog+recommender slice -- positive-space contract.
//!
//! Asserts the SHIPPED public surface of `orchestrator::architecture`:
//!
//!   * 5-archetype taxonomy locked at SaasArchetype + ALL_SAAS_ARCHETYPES
//!   * ArchetypeCatalog has one entry per archetype
//!   * Recommender returns High confidence for canonical inputs and Low
//!     confidence with >= 3 clarifying questions for ambiguous inputs
//!   * PRIOR_ART registry has >= 8 entries each archetype-tagged
//!   * SSB roundtrip: SaasPriorArtEvent + PriorArtEventKind 2-arm
//!     discriminator + payload struct present + SCHEMA_VERSION >= 169
//!
//! Sibling E-11.A negative-space guards stay in
//! `e11_idea_to_saas_contracts.rs` (4 guards). Sibling E-50.A guards stay
//! in `e50_saas_archetype_contracts.rs` (4 guards). Together they fence
//! the shipped catalog+recommender surface.

use elai_orchestrator::architecture::{
    recommend_archetype, ArchetypeCatalog, AuthScheme, MultiTenancyModel,
    PriorArtRegistry, RecommendationConfidence, SaasArchetype, ALL_SAAS_ARCHETYPES,
};
use elai_orchestrator::idea_framing::ProjectClass;
use elai_orchestrator::ssb::{
    PriorArtEventKind, SaasPriorArtEventPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};

#[test]
fn schema_version_meets_e11_floor() {
    // E-11 catalog+recommender slice ships at SCHEMA_VERSION 169.
    // Lower than 169 means the SSB cascade did not land.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 169,
        "SCHEMA_VERSION_PUBLIC ({}) must be >= 169 -- the E-11 catalog \
         + recommender SSB cascade is part of this floor.",
        SCHEMA_VERSION_PUBLIC
    );
}

#[test]
fn saas_archetype_enum_has_five_variants() {
    assert_eq!(ALL_SAAS_ARCHETYPES.len(), 5);
}

#[test]
fn saas_archetype_snake_case_labels_match_readme() {
    let labels: Vec<&'static str> = ALL_SAAS_ARCHETYPES
        .iter()
        .map(|a| a.as_snake_case())
        .collect();
    assert_eq!(
        labels,
        vec![
            "saas_dashboard",
            "b2b_marketplace",
            "internal_tooling",
            "developer_api",
            "consumer_social",
        ]
    );
}

#[test]
fn catalog_iter_returns_exactly_five_entries() {
    let count = ArchetypeCatalog::iter().count();
    assert_eq!(count, 5);
}

#[test]
fn catalog_entry_for_each_archetype_has_required_fields() {
    for arch in ALL_SAAS_ARCHETYPES {
        let entry = ArchetypeCatalog::entry_for(*arch);
        assert_eq!(entry.archetype, *arch);
        // default_framework_profile is a non-empty stem reference --
        // version literal ban (e12.A guard 4) prevents inline pins.
        assert!(
            !entry.default_framework_profile.is_empty(),
            "default_framework_profile empty for {:?}",
            arch
        );
        // The auth scheme + multi-tenancy fields are enum-only --
        // raw SQL is banned (e50.A guard 2).
        let _ = entry.default_auth_scheme;
        let _ = entry.multi_tenancy_model;
        // 4-point scope-clarity rubric is locked.
        assert_eq!(entry.scope_clarity_profile.len(), 4);
        // Production checklist subset is non-empty.
        assert!(!entry.production_checklist_subset.is_empty());
    }
}

#[test]
fn auth_scheme_wire_labels_are_stable() {
    assert_eq!(AuthScheme::OauthGoogle.as_snake_case(), "oauth_google");
    assert_eq!(AuthScheme::OauthGithub.as_snake_case(), "oauth_github");
    assert_eq!(AuthScheme::OidcWorkOs.as_snake_case(), "oidc_workos");
    assert_eq!(AuthScheme::EmailMagicLink.as_snake_case(), "email_magic_link");
    assert_eq!(AuthScheme::ApiKey.as_snake_case(), "api_key");
}

#[test]
fn multi_tenancy_wire_labels_are_stable() {
    assert_eq!(MultiTenancyModel::SingleTenant.as_snake_case(), "single_tenant");
    assert_eq!(MultiTenancyModel::SchemaPerTenant.as_snake_case(), "schema_per_tenant");
    assert_eq!(
        MultiTenancyModel::RowLevelSecurity.as_snake_case(),
        "row_level_security"
    );
    assert_eq!(
        MultiTenancyModel::SeparateDatabase.as_snake_case(),
        "separate_database"
    );
}

#[test]
fn rls_requirement_matches_multi_tenancy_model() {
    assert!(MultiTenancyModel::RowLevelSecurity.requires_rls_probe());
    assert!(MultiTenancyModel::SchemaPerTenant.requires_rls_probe());
    assert!(!MultiTenancyModel::SingleTenant.requires_rls_probe());
    assert!(!MultiTenancyModel::SeparateDatabase.requires_rls_probe());
}

#[test]
fn recommender_high_confidence_for_saas_dashboard() {
    let r = recommend_archetype(ProjectClass::SaaSDashboard, "");
    assert_eq!(r.primary, SaasArchetype::SaasDashboard);
    assert_eq!(r.confidence, RecommendationConfidence::High);
    assert_eq!(r.clarifying_questions.len(), 0);
}

#[test]
fn recommender_high_confidence_for_library_or_sdk() {
    let r = recommend_archetype(ProjectClass::LibraryOrSDK, "");
    assert_eq!(r.primary, SaasArchetype::DeveloperApi);
    assert_eq!(r.confidence, RecommendationConfidence::High);
}

#[test]
fn recommender_high_confidence_for_cli_tool() {
    let r = recommend_archetype(ProjectClass::CLITool, "");
    assert_eq!(r.primary, SaasArchetype::DeveloperApi);
    assert_eq!(r.confidence, RecommendationConfidence::High);
}

#[test]
fn recommender_low_confidence_with_three_questions_for_other() {
    let r = recommend_archetype(ProjectClass::Other, "");
    assert_eq!(r.confidence, RecommendationConfidence::Low);
    assert!(r.clarifying_questions.len() >= 3);
}

#[test]
fn recommender_low_confidence_with_three_questions_for_desktop_app() {
    let r = recommend_archetype(ProjectClass::DesktopApp, "");
    assert_eq!(r.confidence, RecommendationConfidence::Low);
    assert!(r.clarifying_questions.len() >= 3);
}

#[test]
fn recommender_low_confidence_with_three_questions_for_game_mod() {
    let r = recommend_archetype(ProjectClass::GameMod, "");
    assert_eq!(r.confidence, RecommendationConfidence::Low);
    assert!(r.clarifying_questions.len() >= 3);
}

#[test]
fn recommender_keyword_bias_drives_developer_api_pick() {
    let r = recommend_archetype(
        ProjectClass::WebApp,
        "build an api endpoint surface with an sdk and openapi docs",
    );
    assert_eq!(r.primary, SaasArchetype::DeveloperApi);
}

#[test]
fn prior_art_registry_has_at_least_eight_entries() {
    assert!(PriorArtRegistry::len() >= 8);
}

#[test]
fn prior_art_registry_tags_every_archetype() {
    for arch in ALL_SAAS_ARCHETYPES {
        assert!(!PriorArtRegistry::entries_for_archetype(*arch).is_empty());
    }
}

#[test]
fn ssb_action_type_saas_prior_art_event_round_trips() {
    let at = SsbActionType::SaasPriorArtEvent;
    assert_eq!(at.as_sql_str(), "saas_prior_art_event");
}

#[test]
fn ssb_payload_saas_prior_art_event_action_type_matches() {
    let payload = SsbPayload::SaasPriorArtEvent(SaasPriorArtEventPayload {
        kind: PriorArtEventKind::ArchetypeRecommended,
        session_id: Some("s-test".into()),
        archetype: "saas_dashboard".into(),
        confidence: "high".into(),
        details_json: serde_json::json!({"clarifying_question_count": 0}),
        ts_ms: 1_700_000_169_001,
    });
    assert_eq!(payload.action_type(), SsbActionType::SaasPriorArtEvent);
}

#[test]
fn prior_art_event_kind_wire_labels_are_stable() {
    assert_eq!(PriorArtEventKind::ArchetypeRecommended.as_str(), "archetype_recommended");
    assert_eq!(PriorArtEventKind::PriorArtConsulted.as_str(), "prior_art_consulted");
}

#[test]
fn catalog_default_framework_profile_has_no_inline_version_literal() {
    // e12.A guard 4: catalog fields must NOT carry inline version
    // pins. The stem references workflow/archetypes/<stem>.toml where
    // pins live.
    for arch in ALL_SAAS_ARCHETYPES {
        let entry = ArchetypeCatalog::entry_for(*arch);
        let stem = entry.default_framework_profile;
        // A version literal would contain '@' (npm), '==' (pip), or
        // start with a digit. The catalog stem is a TOML filename
        // base, which is alphanumeric + dash only.
        assert!(
            !stem.contains('@'),
            "default_framework_profile must NOT contain inline @-version: '{}'",
            stem
        );
        assert!(
            !stem.contains("=="),
            "default_framework_profile must NOT contain inline ==-version: '{}'",
            stem
        );
    }
}
