//! B-40 — Idea-Framer contract tests.
//!
//! Pins the behaviour of the FRAMING substate end-to-end:
//!   * `frame_idea` persists framed-idea.toml at the canonical session path.
//!   * `frame_idea` emits all 5 SSB action_types on a clean convergent run:
//!     idea_framing_started, project_class_classified, critic_round_complete,
//!     idea_framed, tradeoff_card_emitted.
//!   * TradeoffCard is non-empty (>= 2 tradeoffs) per WIRING_CONTRACT
//!     observable_effect.
//!   * SCHEMA_VERSION_PUBLIC reflects the B-40 bump (40 -> 41).
//!   * All 5 new SsbActionType variants are roundtrippable via
//!     `ssb_action_type_from_str_public`.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::{
    frame_idea, ssb_action_type_from_str_public, FramedIdea, ProjectClass, ResearchPresets,
    ResearchStream, SourceTier, Ssb, SsbActionType, SsbConfig, SCHEMA_VERSION_PUBLIC,
};

fn os_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

/// Build a transient SSB instance backed by a unique temp DB path.
async fn build_ssb(test_id: &str) -> (Arc<Ssb>, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("create tempdir for SSB");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: format!("proj-{}", test_id),
        session_id: format!("sess-{}", test_id),
    };
    let ssb = Ssb::new(cfg, os_clock()).expect("Ssb::new");
    (Arc::new(ssb), dir)
}

fn build_test_presets() -> ResearchPresets {
    use elai_orchestrator::ClassPreset;
    use std::collections::BTreeMap;
    let mut presets: BTreeMap<String, ClassPreset> = BTreeMap::new();
    let streams = vec![
        ResearchStream {
            name: "competitor".to_owned(),
            query_template: "{idea} :: competitor".to_owned(),
            min_source_tier: SourceTier::T2,
            parallelism_budget: 3,
            cosmos_grounding: None,
            mandatory: true,
        },
        ResearchStream {
            name: "guidelines".to_owned(),
            query_template: "{idea} :: guidelines".to_owned(),
            min_source_tier: SourceTier::T1,
            parallelism_budget: 3,
            cosmos_grounding: None,
            mandatory: true,
        },
    ];
    presets.insert(
        ProjectClass::ConsumerMobileApp.as_snake_case().to_owned(),
        ClassPreset { streams },
    );
    ResearchPresets { presets }
}

// ---------------------------------------------------------------------------
// Contract — SCHEMA_VERSION bump for B-40 lands.
// ---------------------------------------------------------------------------

#[test]
fn contract_schema_version_is_41_after_b40() {
    // s-68264 test-cleanup: SCHEMA_VERSION now 96 after C-plan + D-plan + E-plan
    // bumps. B-40 (idea-framer, 5 new action_types) ships under this envelope.
    // Source of truth: orchestrator/src/ssb.rs `const SCHEMA_VERSION: i64`.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 96,
        "schema only drifts forward; this contract was written at version 96. \
         CONTRACT: SCHEMA_VERSION expected >= 96 after s-68264 campaign \
         (C-plan + D-plan + E-plan bumps). B-40's 5 Idea-Framer variants \
         still ship under this envelope. Update ssb.rs ledger \
         comments on the next bump."
    );
}

// ---------------------------------------------------------------------------
// Contract — all 5 new SsbActionType variants roundtrip via from_str.
// ---------------------------------------------------------------------------

#[test]
fn contract_all_5_new_action_types_from_str() {
    let pairs = [
        ("idea_framing_started", SsbActionType::IdeaFramingStarted),
        ("project_class_classified", SsbActionType::ProjectClassClassified),
        ("critic_round_complete", SsbActionType::CriticRoundComplete),
        ("idea_framed", SsbActionType::IdeaFramed),
        ("tradeoff_card_emitted", SsbActionType::TradeoffCardEmitted),
    ];
    for (s, expected) in &pairs {
        let actual = ssb_action_type_from_str_public(s).expect("from_str");
        assert_eq!(
            actual, *expected,
            "CONTRACT: ssb_action_type_from_str_public({}) must yield {:?}",
            s, expected
        );
    }
}

// ---------------------------------------------------------------------------
// Contract — frame_idea persists framed-idea.toml under session_dir.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_frame_idea_persists_artifact() {
    let (ssb, _ssb_dir) = build_ssb("artifact").await;
    let session_dir = tempfile::tempdir().expect("session dir");
    let presets = build_test_presets();

    let framed: FramedIdea = frame_idea(
        "make an app like PictureThis",
        &presets,
        session_dir.path(),
        "sess-artifact",
        Some(&ssb),
        os_clock(),
    )
    .await
    .expect("frame_idea should succeed");

    let path = session_dir.path().join("idea-framing").join("framed-idea.toml");
    assert!(
        path.exists(),
        "CONTRACT: framed-idea.toml must be persisted at {}",
        path.display()
    );
    assert_eq!(
        framed.project_class,
        ProjectClass::ConsumerMobileApp,
        "PictureThis prompt must classify as ConsumerMobileApp"
    );
    assert!(
        framed.tradeoff_card.tradeoffs.len() >= 2,
        "CONTRACT: TradeoffCard must have >= 2 tradeoffs (observable_effect)"
    );
}

// ---------------------------------------------------------------------------
// Contract — frame_idea emits all 5 SSB action_types on convergent run.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_frame_idea_emits_all_5_ssb_action_types() {
    use elai_orchestrator::SsbReader;
    let (ssb, _ssb_dir) = build_ssb("ssb-emit").await;
    let session_dir = tempfile::tempdir().expect("session dir");
    let presets = build_test_presets();

    let _ = frame_idea(
        "make an app like PictureThis",
        &presets,
        session_dir.path(),
        "sess-ssb-emit",
        Some(&ssb),
        os_clock(),
    )
    .await
    .expect("frame_idea");

    // Wait a moment for SSB writer task to drain (writer is async).
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let expected = [
        SsbActionType::IdeaFramingStarted,
        SsbActionType::ProjectClassClassified,
        SsbActionType::CriticRoundComplete,
        SsbActionType::IdeaFramed,
        SsbActionType::TradeoffCardEmitted,
    ];
    for action_type in &expected {
        let rows = ssb
            .query_by_type(*action_type)
            .expect("query_by_type");
        assert!(
            !rows.is_empty(),
            "CONTRACT: at least one SSB row of type {:?} must exist",
            action_type
        );
    }
}

// ---------------------------------------------------------------------------
// Contract — critic loop converges with zero BLOCKs within MAX_CRITIC_ROUNDS.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_critic_loop_converges_on_simple_idea() {
    let (ssb, _ssb_dir) = build_ssb("converge").await;
    let session_dir = tempfile::tempdir().expect("session dir");
    let presets = build_test_presets();

    let framed = frame_idea(
        "build a Mailchimp competitor with cohort analytics",
        &presets,
        session_dir.path(),
        "sess-converge",
        Some(&ssb),
        os_clock(),
    )
    .await
    .expect("frame_idea on saas prompt");

    assert!(
        framed.is_converged(),
        "CONTRACT: critic loop must converge (zero BLOCKs) on a simple anchored idea"
    );
    assert!(
        !framed.critic_rounds.is_empty(),
        "CONTRACT: at least one critic round must run (stub_ban: must NOT short-circuit max_rounds)"
    );
    assert!(
        framed.critic_rounds.len() <= 3,
        "CONTRACT: must respect MAX_CRITIC_ROUNDS=3 cap"
    );
}

// ---------------------------------------------------------------------------
// Contract — TradeoffCard emitted with >= 2 tradeoffs per observable_effect.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_tradeoff_card_emitted_with_min_2_tradeoffs() {
    let (ssb, _ssb_dir) = build_ssb("tradeoff").await;
    let session_dir = tempfile::tempdir().expect("session dir");
    let presets = build_test_presets();

    let framed = frame_idea(
        "Rust crate: parser combinator library",
        &presets,
        session_dir.path(),
        "sess-tradeoff",
        Some(&ssb),
        os_clock(),
    )
    .await
    .expect("frame_idea on library prompt");

    assert!(
        framed.tradeoff_card.tradeoffs.len() >= 2,
        "CONTRACT: tradeoff_card.tradeoffs.len() >= 2 per WIRING_CONTRACT observable_effect"
    );
    // Each tradeoff must have at least 2 options.
    for t in &framed.tradeoff_card.tradeoffs {
        assert!(
            t.options.len() >= 2,
            "CONTRACT: each Tradeoff must have >= 2 options"
        );
    }
}

// ---------------------------------------------------------------------------
// Contract — empty prompt rejected with EmptyPrompt error.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_empty_prompt_rejected() {
    let (ssb, _ssb_dir) = build_ssb("empty").await;
    let session_dir = tempfile::tempdir().expect("session dir");
    let presets = ResearchPresets::new();

    let result = frame_idea(
        "   ",
        &presets,
        session_dir.path(),
        "sess-empty",
        Some(&ssb),
        os_clock(),
    )
    .await;
    assert!(result.is_err(), "CONTRACT: empty prompt must reject");
}
