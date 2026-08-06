//! B-40 — Idea-Framer unit tests.
//!
//! Tests:
//!   * ProjectClass classifier accuracy on the 50-prompt fixture (>= 90%).
//!   * CriticCategory exhaustive-match compile-time guard (NUM_CRITIC_CATEGORIES==10).
//!   * Preset loader returns >=4 streams per class for the 4 shipped presets.
//!   * Multi-root ResearchTree construction + per-stream root lookup.
//!   * Severity / Tradeoff / TradeoffCard / CriticRound roundtrip via toml.

use std::path::PathBuf;

use elai_orchestrator::idea_framing::{
    classify_project_class, classify_project_class_with_pattern, load_research_preset,
    ClassPreset, CriticCategory, CriticFinding, CriticRound, FramedIdea, KeyDecision,
    ProjectClass, ResearchPresets, ResearchStream, ResearchSummary, Severity, SourceTier,
    Tradeoff, TradeoffCard, ALL_CRITIC_CATEGORIES, ALL_PROJECT_CLASSES, MAX_CRITIC_ROUNDS,
    NUM_CRITIC_CATEGORIES, frame_idea,
};

// ---------------------------------------------------------------------------
// Compile-time invariants
// ---------------------------------------------------------------------------

#[test]
fn critic_category_count_is_exactly_10() {
    assert_eq!(NUM_CRITIC_CATEGORIES, 10);
    assert_eq!(ALL_CRITIC_CATEGORIES.len(), 10);
    // Exhaustive coverage: each variant must appear in ALL_CRITIC_CATEGORIES.
    // Adding a new CriticCategory variant without bumping the slice will
    // fail this test because the slice covers exactly 10 enum values.
}

#[test]
fn project_class_count_is_exactly_9() {
    assert_eq!(ALL_PROJECT_CLASSES.len(), 9);
}

#[test]
fn max_critic_rounds_is_3() {
    assert_eq!(MAX_CRITIC_ROUNDS, 3);
}

// ---------------------------------------------------------------------------
// ProjectClass classifier — 50-fixture accuracy gate
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct Fixture {
    prompt: String,
    expected_class: String,
}

#[derive(serde::Deserialize)]
struct FixturesFile {
    fixtures: Vec<Fixture>,
}

fn load_fixtures() -> Vec<Fixture> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from(manifest_dir);
    // CARGO_MANIFEST_DIR for orchestrator is .elai_cc/crates/orchestrator;
    // fixtures live at .elai_cc/tests/fixtures/...
    path.pop(); // -> .elai_cc/crates
    path.pop(); // -> .elai_cc
    path.push("tests");
    path.push("fixtures");
    path.push("project_class_50.toml");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixtures at {}: {}", path.display(), e));
    let parsed: FixturesFile = toml::from_str(&raw).expect("parse fixtures TOML");
    parsed.fixtures
}

#[test]
fn classifier_fixture_loaded_and_has_50_entries() {
    let fx = load_fixtures();
    assert_eq!(
        fx.len(),
        50,
        "fixture must have exactly 50 labelled prompts per B-40 ACCEPTANCE line 1"
    );
}

#[test]
fn classifier_accuracy_at_least_90_percent_on_50_fixtures() {
    let fx = load_fixtures();
    let total = fx.len();
    let mut correct = 0;
    let mut failures: Vec<String> = Vec::new();
    for f in &fx {
        let expected = ProjectClass::from_snake_case(&f.expected_class)
            .unwrap_or_else(|| panic!("invalid expected_class '{}'", f.expected_class));
        let actual = classify_project_class(&f.prompt);
        if actual == expected {
            correct += 1;
        } else {
            failures.push(format!(
                "MISS: prompt={:?} expected={} actual={}",
                f.prompt,
                expected.as_snake_case(),
                actual.as_snake_case()
            ));
        }
    }
    let accuracy = correct as f64 / total as f64;
    assert!(
        accuracy >= 0.9,
        "classifier accuracy {:.2}% < 90% (correct={}, total={}); failures:\n{}",
        accuracy * 100.0,
        correct,
        total,
        failures.join("\n")
    );
}

#[test]
fn classifier_returns_other_on_unanchored_input() {
    let (class, matched) = classify_project_class_with_pattern("blah blah blah");
    assert_eq!(class, ProjectClass::Other);
    assert!(matched.is_none());
}

#[test]
fn classifier_returns_other_on_empty_input() {
    let class = classify_project_class("");
    assert_eq!(class, ProjectClass::Other);
}

#[test]
fn classifier_returns_consumer_mobile_app_on_picturethis() {
    let (class, matched) = classify_project_class_with_pattern("make an app like PictureThis");
    assert_eq!(class, ProjectClass::ConsumerMobileApp);
    assert!(matched.is_some());
}

#[test]
fn project_class_snake_case_roundtrip_all_9() {
    for class in ALL_PROJECT_CLASSES {
        let s = class.as_snake_case();
        assert_eq!(ProjectClass::from_snake_case(s), Some(*class));
    }
}

// ---------------------------------------------------------------------------
// Preset loader
// ---------------------------------------------------------------------------

fn build_test_presets() -> ResearchPresets {
    use std::collections::BTreeMap;
    let mut presets: BTreeMap<String, ClassPreset> = BTreeMap::new();

    let make_stream = |name: &str, mandatory: bool| ResearchStream {
        name: name.to_owned(),
        query_template: format!("{{idea}} :: {}", name),
        min_source_tier: SourceTier::T2,
        parallelism_budget: 3,
        cosmos_grounding: None,
        mandatory,
    };

    for class in &[
        ProjectClass::ConsumerMobileApp,
        ProjectClass::WebApp,
        ProjectClass::SaaSDashboard,
        ProjectClass::GameMod,
    ] {
        let streams = vec![
            make_stream("competitor", true),
            make_stream("reviews", true),
            make_stream("guidelines", true),
            make_stream("licensing", true),
            make_stream("trends", false),
            make_stream("monetization", false),
        ];
        presets.insert(class.as_snake_case().to_owned(), ClassPreset { streams });
    }
    ResearchPresets { presets }
}

#[test]
fn preset_loader_returns_empty_for_unconfigured_class() {
    let presets = ResearchPresets::new();
    let v = load_research_preset(ProjectClass::ConsumerMobileApp, &presets);
    assert!(v.is_empty());
}

#[test]
fn preset_loader_returns_streams_for_4_classes() {
    let presets = build_test_presets();
    for class in &[
        ProjectClass::ConsumerMobileApp,
        ProjectClass::WebApp,
        ProjectClass::SaaSDashboard,
        ProjectClass::GameMod,
    ] {
        let streams = load_research_preset(*class, &presets);
        assert!(
            streams.len() >= 4,
            "preset for {} has only {} streams; expected >= 4",
            class.as_snake_case(),
            streams.len()
        );
        let mandatory_count = streams.iter().filter(|s| s.mandatory).count();
        assert!(
            mandatory_count >= 4,
            "preset for {} has only {} mandatory streams; B-40 ACCEPTANCE line 2 requires >= 4",
            class.as_snake_case(),
            mandatory_count
        );
    }
}

#[test]
fn live_elai_cc_toml_has_4_research_presets() {
    // B-40 BLOCK-1 audit fix: round-trip parse via the production-path
    // `parse_research_presets_from_toml` helper. This proves the TOML
    // shape `[[research_presets.<class>.streams]]` deserializes into
    // `ResearchPresets { presets: BTreeMap<String, ClassPreset> }`
    // — not just a string contains() check.
    use elai_orchestrator::idea_framing::parse_research_presets_from_toml;

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from(manifest_dir);
    path.pop(); // -> crates
    path.pop(); // -> .elai_cc
    path.push("elai_cc.toml");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read elai_cc.toml at {}: {}", path.display(), e));

    let presets = parse_research_presets_from_toml(&raw)
        .expect("elai_cc.toml [research_presets] block must deserialize");

    for class in &[
        ProjectClass::ConsumerMobileApp,
        ProjectClass::WebApp,
        ProjectClass::SaaSDashboard,
        ProjectClass::GameMod,
    ] {
        let streams = load_research_preset(*class, &presets);
        assert!(
            streams.len() >= 4,
            "elai_cc.toml preset for {} parsed only {} streams; expected >= 4 \
             (B-40 ACCEPTANCE line 2 + BLOCK-1 round-trip check)",
            class.as_snake_case(),
            streams.len()
        );
        let mandatory_count = streams.iter().filter(|s| s.mandatory).count();
        assert!(
            mandatory_count >= 4,
            "elai_cc.toml preset for {} has only {} mandatory streams after \
             round-trip parse; B-40 requires >= 4",
            class.as_snake_case(),
            mandatory_count
        );
    }
}

// ---------------------------------------------------------------------------
// Multi-root ResearchTree
// ---------------------------------------------------------------------------

#[test]
fn multi_root_research_tree_constructs_with_per_stream_root_ids() {
    use elai_core::clock::OsClock;
    use elai_orchestrator::research_tree::ResearchTree;
    use std::sync::Arc;

    let streams = vec![
        ResearchStream {
            name: "stream_a".to_owned(),
            query_template: "what is A?".to_owned(),
            min_source_tier: SourceTier::T2,
            parallelism_budget: 2,
            cosmos_grounding: None,
            mandatory: true,
        },
        ResearchStream {
            name: "stream_b".to_owned(),
            query_template: "what is B?".to_owned(),
            min_source_tier: SourceTier::T1,
            parallelism_budget: 2,
            cosmos_grounding: None,
            mandatory: true,
        },
        ResearchStream {
            name: "stream_c".to_owned(),
            query_template: "what is C?".to_owned(),
            min_source_tier: SourceTier::T2,
            parallelism_budget: 2,
            cosmos_grounding: None,
            mandatory: false,
        },
    ];

    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let tree = ResearchTree::new_multi_root(streams, clock).expect("build multi-root tree");
    let roots = tree.multi_root_ids();
    assert_eq!(roots.len(), 3, "expected 3 roots, got {}", roots.len());
    // root[0] must equal canonical `root` field (backward-compat invariant).
    assert_eq!(roots[0], tree.root);
    // All roots distinct.
    assert_ne!(roots[0], roots[1]);
    assert_ne!(roots[1], roots[2]);
    // stream_root_id lookups
    assert_eq!(tree.stream_root_id(0), Some(roots[0]));
    assert_eq!(tree.stream_root_id(2), Some(roots[2]));
    assert_eq!(tree.stream_root_id(3), None);
}

#[test]
fn multi_root_rejects_empty_streams() {
    use elai_core::clock::OsClock;
    use elai_orchestrator::research_tree::ResearchTree;
    use std::sync::Arc;
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let result = ResearchTree::new_multi_root(Vec::new(), clock);
    assert!(result.is_err());
}

#[test]
fn single_root_constructor_still_works_after_b40_additions() {
    use elai_core::clock::OsClock;
    use elai_orchestrator::research_tree::{QueryComplexity, ResearchPlan, ResearchTree};
    use std::sync::Arc;
    use std::time::Duration;

    let plan = ResearchPlan {
        root_question: "test question".to_owned(),
        sub_questions: vec!["sub1".to_owned()],
        complexity: QueryComplexity::Simple,
        time_budget: Duration::from_secs(10),
        require_contradictions: false,
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let tree = ResearchTree::new(plan, true, clock).expect("single-root build");
    // multi_root_ids must be empty for single-root trees (preserved invariant).
    assert!(tree.multi_root_ids().is_empty());
    assert_eq!(tree.stream_root_id(0), None);
}

// ---------------------------------------------------------------------------
// Severity + Category coverage
// ---------------------------------------------------------------------------

#[test]
fn all_severities_have_snake_case() {
    assert_eq!(Severity::Block.as_snake_case(), "block");
    assert_eq!(Severity::Warn.as_snake_case(), "warn");
    assert_eq!(Severity::Info.as_snake_case(), "info");
}

#[test]
fn all_critic_categories_have_distinct_snake_case() {
    let mut seen: std::collections::HashSet<&'static str> = std::collections::HashSet::new();
    for cat in ALL_CRITIC_CATEGORIES {
        let s = cat.as_snake_case();
        assert!(seen.insert(s), "duplicate snake_case for CriticCategory: {}", s);
    }
    assert_eq!(seen.len(), 10);
}

// ---------------------------------------------------------------------------
// TradeoffCard + CriticRound roundtrip
// ---------------------------------------------------------------------------

#[test]
fn tradeoff_card_toml_roundtrip() {
    let card = TradeoffCard {
        tradeoffs: vec![Tradeoff {
            name: "build_or_buy".to_owned(),
            options: vec!["build".to_owned(), "buy".to_owned()],
            default: "buy".to_owned(),
            user_facing_question: "?".to_owned(),
        }],
    };
    let s = toml::to_string(&card).expect("encode");
    let decoded: TradeoffCard = toml::from_str(&s).expect("decode");
    assert_eq!(decoded, card);
}

#[test]
fn critic_round_block_counts_correctly() {
    let round = CriticRound {
        round_index: 1,
        findings: vec![
            CriticFinding {
                category: CriticCategory::Missing,
                severity: Severity::Block,
                evidence_ref: None,
                proposed_fix: "fix1".to_owned(),
            },
            CriticFinding {
                category: CriticCategory::Vague,
                severity: Severity::Warn,
                evidence_ref: None,
                proposed_fix: "fix2".to_owned(),
            },
        ],
    };
    assert_eq!(round.block_count(), 1);
    assert_eq!(round.warn_count(), 1);
    assert_eq!(round.info_count(), 0);
}

#[test]
fn critic_round_category_counts_emits_all_10_buckets() {
    let round = CriticRound {
        round_index: 1,
        findings: vec![CriticFinding {
            category: CriticCategory::Slop,
            severity: Severity::Warn,
            evidence_ref: None,
            proposed_fix: "fix".to_owned(),
        }],
    };
    let counts = round.category_counts();
    assert_eq!(counts.len(), 10);
    let slop_count = counts
        .iter()
        .find(|(name, _)| name == "slop")
        .map(|(_, c)| *c);
    assert_eq!(slop_count, Some(1));
}

// ---------------------------------------------------------------------------
// FramedIdea convergence
// ---------------------------------------------------------------------------

#[test]
fn framed_idea_is_converged_with_zero_blocks() {
    let framed = FramedIdea {
        project_class: ProjectClass::ConsumerMobileApp,
        raw_prompt: "test".to_owned(),
        research_summary: ResearchSummary {
            streams: Vec::new(),
            degraded_streams: Vec::new(),
        },
        key_decisions: Vec::new(),
        known_unknowns: Vec::new(),
        tradeoff_card: TradeoffCard::default(),
        critic_rounds: vec![CriticRound {
            round_index: 1,
            findings: vec![CriticFinding {
                category: CriticCategory::Vague,
                severity: Severity::Warn,
                evidence_ref: None,
                proposed_fix: "ok".to_owned(),
            }],
        }],
    };
    assert!(framed.is_converged());
}

#[test]
fn framed_idea_is_not_converged_with_blocks() {
    let framed = FramedIdea {
        project_class: ProjectClass::Other,
        raw_prompt: "test".to_owned(),
        research_summary: ResearchSummary {
            streams: Vec::new(),
            degraded_streams: Vec::new(),
        },
        key_decisions: Vec::new(),
        known_unknowns: Vec::new(),
        tradeoff_card: TradeoffCard::default(),
        critic_rounds: vec![CriticRound {
            round_index: 1,
            findings: vec![CriticFinding {
                category: CriticCategory::Missing,
                severity: Severity::Block,
                evidence_ref: None,
                proposed_fix: "fix".to_owned(),
            }],
        }],
    };
    assert!(!framed.is_converged());
}

#[test]
fn key_decision_construction() {
    // Sanity check: KeyDecision compiles + serializes.
    let kd = KeyDecision {
        name: "x".to_owned(),
        rationale: "y".to_owned(),
        source_refs: vec!["z".to_owned()],
    };
    let s = toml::to_string(&kd).expect("encode");
    let _back: KeyDecision = toml::from_str(&s).expect("decode");
}


#[tokio::test(flavor = "current_thread")]
async fn emits_scope_ceiling_advisory_on_trigger() {
    use elai_core::clock::OsClock;
    use elai_orchestrator::ssb::{Ssb, SsbConfig};
    use elai_orchestrator::ssb_viewer::SsbViewer;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn run(prompt: &str) -> Vec<elai_orchestrator::ssb_viewer::SsbActionDetail> {
        let dir = TempDir::new().expect("tempdir");
        let db = dir.path().join("ssb.db");
        let ssb = Arc::new(Ssb::new(SsbConfig {
            db_path: db.clone(),
            pool_size: Some(1),
            project_id: "test-project".into(),
            session_id: "test-session".into(),
        }, Arc::new(OsClock)).expect("ssb init"));
        frame_idea(
            prompt,
            &ResearchPresets::new(),
            dir.path(),
            "test-session",
            Some(&ssb),
            Arc::new(OsClock),
        ).await.expect("frame idea");
        let viewer = SsbViewer::open(&db).expect("viewer");
        viewer.recent_rows_of_type("scope_ceiling_advisory", 32).expect("rows")
    }

    let positive = run("Build a Replit Agent 3 project with external API auth").await;
    assert_eq!(positive.len(), 1, "triggered brief must emit one advisory row");
    let payload: serde_json::Value = positive[0].payload_json.clone();
    assert_eq!(payload["class"], "replit_agent_3_external_api_auth");
    assert!(payload["triggered_by"].as_str().is_some());

    let negative = run("Build a private calculator with local arithmetic only").await;
    assert!(negative.is_empty(), "control brief must emit no advisory row");
}
