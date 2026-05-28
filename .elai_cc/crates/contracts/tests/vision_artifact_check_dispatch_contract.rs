//! E-36 Tier-B — VerificationMethod::VisualArtifactCheck dispatch contract.
//!
//! Locks the 10th VerificationMethod variant + its requires_*() helper +
//! the dual default-array updates. If a future refactor "8 variants is
//! enough" and removes VisualArtifactCheck, this contract fails loudly.
//!
//! HARD RULE per plan task 5: VisualArtifactCheck routes on a claim
//! whose claim_category is "visual_spatial" (or any other visual tag);
//! the dispatcher itself uses discriminant-only matching, so the
//! contract here pins the variant shape + the helper predicate.

use elai_reasoning::types::{AgentGrcConfig, GrcDefaults, VerificationMethod};

#[test]
fn visual_artifact_check_variant_exists_and_routes_through_requires_vision_index() {
    let vac = VerificationMethod::VisualArtifactCheck {
        artifact_name: "auth_diagram".into(),
        query: "the auth flow has 3 redirects".into(),
        claim_category: "visual_spatial".into(),
    };
    assert!(vac.requires_vision_index(), "VisualArtifactCheck must route through requires_vision_index");
    // Negation: must not collide with sibling helpers.
    assert!(!vac.requires_network());
    assert!(!vac.requires_tools());
    assert!(!vac.requires_artifact_store());
    assert!(!vac.requires_cosmos_store());
    assert!(!vac.requires_empire_bridge());
}

#[test]
fn other_methods_do_not_claim_vision_index() {
    // Regression: extending VerificationMethod must not have widened any
    // pre-existing variant onto the vision-index path.
    for m in [
        VerificationMethod::FileCheck,
        VerificationMethod::SourceCheck,
        VerificationMethod::TestCheck,
        VerificationMethod::WebCheck,
        VerificationMethod::ApiCheck,
        VerificationMethod::ApiRegistryCheck,
        VerificationMethod::ArtifactCheck {
            artifact_name: String::new(),
            query: String::new(),
        },
        VerificationMethod::CosmosCheck {
            artifact_name: String::new(),
            query: String::new(),
            claim_category: String::new(),
        },
        VerificationMethod::EmpireEventCheck {
            source: String::new(),
            event_type: String::new(),
        },
    ] {
        assert!(
            !m.requires_vision_index(),
            "{m:?} must not claim requires_vision_index"
        );
    }
}

#[test]
fn agent_grc_default_allowed_methods_include_visual_artifact_check() {
    // Both default arrays at reasoning/src/types.rs (AgentGrcConfig::default
    // + GrcDefaults::default) must contain the new wildcard entry; missing
    // either site = dead-variant dispatch path per the E-36-decisions doc.
    let agent_default = AgentGrcConfig::default();
    let found_in_agent = agent_default
        .allowed_methods
        .iter()
        .any(|m| matches!(m, VerificationMethod::VisualArtifactCheck { .. }));
    assert!(
        found_in_agent,
        "AgentGrcConfig::default must include VisualArtifactCheck wildcard"
    );

    let grc_default = GrcDefaults::default();
    let found_in_grc = grc_default
        .allowed_methods
        .iter()
        .any(|m| matches!(m, VerificationMethod::VisualArtifactCheck { .. }));
    assert!(
        found_in_grc,
        "GrcDefaults::default must include VisualArtifactCheck wildcard"
    );
}

#[test]
fn visual_artifact_check_serde_round_trip() {
    let m = VerificationMethod::VisualArtifactCheck {
        artifact_name: "arch.png".into(),
        query: "X sits between Y and Z".into(),
        claim_category: "visual_spatial".into(),
    };
    let json = serde_json::to_string(&m).expect("serialize ok");
    let back: VerificationMethod = serde_json::from_str(&json).expect("deserialize ok");
    assert_eq!(back, m);
}

#[test]
fn visual_artifact_check_reuses_claim_category_not_claim_class() {
    // GAP-7 reconciliation: plan said claim_class, code uses claim_category
    // (same field name as CosmosCheck for taxonomy consistency). This test
    // pins the field name so a future "rename to claim_class" PR fails loudly.
    let m = VerificationMethod::VisualArtifactCheck {
        artifact_name: "x".into(),
        query: "q".into(),
        claim_category: "visual_spatial".into(),
    };
    let json = serde_json::to_string(&m).expect("serialize");
    assert!(json.contains("\"claim_category\""), "must serialize as claim_category");
    assert!(!json.contains("\"claim_class\""), "must NOT serialize as claim_class");
}
