//! C-41 — Skills-driven workflow registry contract.
//!
//! Paper context (workflow/papers/scientific-workflow-interface.md):
//! arXiv:2604.21910 "From Research Question to Scientific Workflow" reports a
//! full-match intent-classification accuracy of 44.0% -> 83.3% on Claude Opus
//! 4.6 (S0 baseline vs S3 all-skills, Section 5.1 Table 2, 150-query stratified
//! test). The paper does NOT publish a formal manifest schema, scoring
//! function, or SSB vocabulary — those are LOCALLY AUTHORED in the
//! `elai_orchestrator::skill_registry` module. This contract test pins the
//! local surface (manifest parse, discovery, deterministic ranking, SSB
//! payload round-trip, project-local resolve path) without asserting the
//! paper's effect-size number — that requires a live RUNPOD-PENDING eval
//! against Claude Opus 4.6.
//!
//! Spec: F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-41_skills_driven_workflow_formalization.md
//! Decisions: workflow/sessions/s-25120/completion-drive/contexts/C-41_skills_driven_workflow_formalization-context.md

use std::fs;
use std::path::PathBuf;

use elai_orchestrator::skill_registry::{
    eval_skill_routing, find_skills, parse_skill_frontmatter_extended,
    score_skill_for_context, skill_name_hash, top_k_skills, DiscoveryRoot, SkillContext,
    SkillEvalQuery, SkillManifest, SkillOrigin, SKILL_CONFIDENCE_THRESHOLD,
};
use elai_orchestrator::ssb::{
    decode_ssb_payload, SkillExecutedPayload, SkillTriggeredPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};

use tempfile::TempDir;

// ---------------------------------------------------------------------------
// D1. Manifest deserializes from representative frontmatter
// ---------------------------------------------------------------------------

#[test]
fn manifest_parses_extended_frontmatter_from_representative_skill() {
    let body = "---\nname: focused_edit\nversion: 1.0.0\n\
                description: Localized single-file edit skill.\n\
                triggers: edit, fix, patch, update\n\
                inputs: file_path, change_description\n\
                outputs: edit_result\n\
                scope: execution\n---\n# Body\nSkill scaffold text.";
    let fields = parse_skill_frontmatter_extended(body);
    assert_eq!(fields.name.as_deref(), Some("focused_edit"));
    assert_eq!(fields.version.as_deref(), Some("1.0.0"));
    assert_eq!(fields.scope.as_deref(), Some("execution"));
    assert_eq!(
        fields.triggers,
        vec!["edit", "fix", "patch", "update"],
        "comma-separated triggers must parse in declaration order"
    );
    assert_eq!(
        fields.inputs,
        vec!["file_path", "change_description"],
    );
    assert_eq!(fields.outputs, vec!["edit_result"]);
}

#[test]
fn manifest_parses_legacy_frontmatter_without_extended_keys() {
    let body = "---\nname: legacy_skill\ndescription: Older skill missing C-41 keys.\n---\n# Body";
    let fields = parse_skill_frontmatter_extended(body);
    assert_eq!(fields.name.as_deref(), Some("legacy_skill"));
    assert!(fields.triggers.is_empty());
    assert!(fields.version.is_none());
    assert!(fields.scope.is_none());
}

// ---------------------------------------------------------------------------
// D2. find_skills returns >= 1 manifest from a fixture dir
// ---------------------------------------------------------------------------

#[test]
fn find_skills_walks_fixture_dir_and_returns_manifests() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().join("skills");
    fs::create_dir_all(&root).expect("create root");
    let skill_dir = root.join("planner");
    fs::create_dir_all(&skill_dir).expect("create skill dir");
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: planner\nversion: 0.1.0\ntriggers: plan, design\n---\n# Planner\n",
    )
    .expect("write skill md");

    let manifests =
        find_skills(&[DiscoveryRoot::skills_dir(root)]).expect("discovery should succeed");
    assert!(
        !manifests.is_empty(),
        "find_skills must surface at least the fixture skill"
    );
    let planner = manifests
        .iter()
        .find(|m| m.name == "planner")
        .expect("planner must be discovered");
    assert_eq!(planner.version, "0.1.0");
    assert_eq!(planner.triggers, vec!["plan", "design"]);
    assert_eq!(planner.origin, SkillOrigin::SkillsDir);
}

// ---------------------------------------------------------------------------
// D3. top_k_skills(k=3) returns at most 3 candidates AND is deterministic
// ---------------------------------------------------------------------------

fn fixture_manifest(name: &str, version: &str, triggers: &[&str]) -> SkillManifest {
    SkillManifest {
        name: name.into(),
        version: version.into(),
        triggers: triggers.iter().map(|t| (*t).to_string()).collect(),
        inputs: vec![],
        outputs: vec![],
        scope: String::new(),
        origin: SkillOrigin::SkillsDir,
        file_path: PathBuf::from(name),
        learned_credit_score: None,
        distilled_fragments: Vec::new(),
    }
}

#[test]
fn top_k_with_k_three_returns_at_most_three_and_is_deterministic() {
    let pool = vec![
        fixture_manifest("alpha", "1.0.0", &["plan", "design"]),
        fixture_manifest("beta", "1.0.0", &["plan"]),
        fixture_manifest("gamma", "2.0.0", &["plan"]),
        fixture_manifest("delta", "1.0.0", &["plan"]),
        fixture_manifest("epsilon", "1.0.0", &["plan"]),
    ];
    let ctx = SkillContext::from_text("please plan this design");

    let first = top_k_skills(&pool, &ctx, 3);
    assert!(
        first.len() <= 3,
        "top_k with k=3 must not return more than 3 candidates"
    );
    assert!(!first.is_empty(), "should return at least one candidate");

    let second = top_k_skills(&pool, &ctx, 3);
    let first_names: Vec<&str> = first.iter().map(|s| s.manifest.name.as_str()).collect();
    let second_names: Vec<&str> = second.iter().map(|s| s.manifest.name.as_str()).collect();
    assert_eq!(
        first_names, second_names,
        "deterministic ranking required; repeat invocations must agree"
    );

    // Top candidate must clear the documented threshold for an explicit
    // multi-trigger match — if not, the fall-through path is the expected
    // outcome and the test below documents that branch instead.
    assert!(
        first[0].score >= SKILL_CONFIDENCE_THRESHOLD,
        "explicit multi-trigger hit must clear SKILL_CONFIDENCE_THRESHOLD (score={})",
        first[0].score
    );
}

#[test]
fn fall_through_when_no_trigger_matches() {
    let pool = vec![fixture_manifest("planner", "1.0.0", &["plan", "design"])];
    let ctx = SkillContext::from_text("random unrelated query body");
    let ranked = top_k_skills(&pool, &ctx, 3);
    if let Some(top) = ranked.first() {
        assert!(
            top.score < SKILL_CONFIDENCE_THRESHOLD,
            "no trigger match must produce a score below the threshold to signal fall-through"
        );
    }
}

// ---------------------------------------------------------------------------
// D4. SsbActionType::SkillTriggered + SkillExecuted round-trip
// ---------------------------------------------------------------------------

#[test]
fn ssb_action_type_skill_triggered_and_executed_are_present() {
    // as_sql_str + from_str round-trip via the public payload decoder.
    let triggered = SsbActionType::SkillTriggered;
    assert_eq!(triggered.as_sql_str(), "skill_triggered");
    let executed = SsbActionType::SkillExecuted;
    assert_eq!(executed.as_sql_str(), "skill_executed");
}

#[test]
fn skill_triggered_payload_roundtrips_via_msgpack() {
    let original = SkillTriggeredPayload {
        skill_name_hash: 0xc410_dead_beef_c410,
        score: 0.875_f32,
        top_k_rank: 1_u8,
        ts_ms: 1_700_000_000_000_u64,
    };
    let payload = SsbPayload::SkillTriggered(original.clone());
    let bytes = rmp_serde::to_vec(&payload).expect("encode");
    let decoded = decode_ssb_payload(&bytes).expect("decode");
    match decoded {
        SsbPayload::SkillTriggered(got) => {
            assert_eq!(got.skill_name_hash, original.skill_name_hash);
            assert!((got.score - original.score).abs() < 1e-6);
            assert_eq!(got.top_k_rank, original.top_k_rank);
            assert_eq!(got.ts_ms, original.ts_ms);
        }
        other => panic!("unexpected variant: {other:?}"),
    }
    // action_type discriminant survives the round-trip.
    assert_eq!(
        SsbPayload::SkillTriggered(original).action_type(),
        SsbActionType::SkillTriggered
    );
}

#[test]
fn skill_executed_payload_roundtrips_via_msgpack() {
    let original = SkillExecutedPayload {
        skill_name_hash: 0xc410_dead_beef_c410,
        exit_ok: true,
        duration_ms: 42_u64,
        ts_ms: 1_700_000_000_042_u64,
    };
    let payload = SsbPayload::SkillExecuted(original.clone());
    let bytes = rmp_serde::to_vec(&payload).expect("encode");
    let decoded = decode_ssb_payload(&bytes).expect("decode");
    match decoded {
        SsbPayload::SkillExecuted(got) => {
            assert_eq!(got.skill_name_hash, original.skill_name_hash);
            assert_eq!(got.exit_ok, original.exit_ok);
            assert_eq!(got.duration_ms, original.duration_ms);
            assert_eq!(got.ts_ms, original.ts_ms);
        }
        other => panic!("unexpected variant: {other:?}"),
    }
}

#[test]
fn schema_version_advanced_to_at_least_c41_landing() {
    // C-41 bumps SCHEMA_VERSION 110 -> 111. Subsequent landings may push
    // it higher (concurrent sibling sessions are allowed to piggyback);
    // the contract only requires that the C-41 minimum has been reached.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 111,
        "C-41 requires SCHEMA_VERSION >= 111 (got {SCHEMA_VERSION_PUBLIC})"
    );
}

// ---------------------------------------------------------------------------
// D5. resolve_skill_path candidates include .elai_cc/skills path
// ---------------------------------------------------------------------------

#[test]
fn resolve_skill_path_includes_project_local_elai_cc_skills() {
    // Create a temporary workspace shape: <tmp>/.elai_cc/skills/<name>/SKILL.md.
    let tmp = TempDir::new().expect("tempdir");
    let skill_root = tmp.path().join(".elai_cc").join("skills");
    let skill_dir = skill_root.join("c41_fixture");
    fs::create_dir_all(&skill_dir).expect("create skill dir");
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: c41_fixture\ndescription: contract fixture\n---\n# Body\n",
    )
    .expect("write skill md");

    // Use the orchestrator-side discovery walker as the canonical
    // candidate check: if find_skills surfaces the fixture from a root
    // pointing at .elai_cc/skills, the project-local convention is
    // honoured. (We do not invoke the tools-side resolve_skill_path
    // directly here because it depends on the process CWD, which is set
    // by cargo-test and cannot be safely altered from a contract test;
    // the equivalent assertion lives in the unit tests of tools/src/lib.rs.)
    let manifests = find_skills(&[DiscoveryRoot::skills_dir(skill_root.clone())])
        .expect("project-local skill root must be readable");
    assert_eq!(manifests.len(), 1, ".elai_cc/skills root must surface the fixture");
    assert_eq!(manifests[0].name, "c41_fixture");
    assert_eq!(manifests[0].file_path, skill_dir.join("SKILL.md"));
}

// ---------------------------------------------------------------------------
// Scoring + ranking determinism + name-hash stability
// ---------------------------------------------------------------------------

#[test]
fn score_skill_for_context_is_deterministic() {
    let manifest = fixture_manifest("planner", "1.0.0", &["plan", "design"]);
    let ctx = SkillContext::from_text("please plan the work");
    let a = score_skill_for_context(&manifest, &ctx);
    let b = score_skill_for_context(&manifest, &ctx);
    assert_eq!(a, b, "score must be deterministic for identical input");
}

#[test]
fn skill_name_hash_is_stable_across_calls() {
    let a = skill_name_hash("planning_workflow");
    let b = skill_name_hash("planning_workflow");
    let c = skill_name_hash("paper_ingest");
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ---------------------------------------------------------------------------
// Eval harness shape (full 150-query eval is RUNPOD-PENDING)
// ---------------------------------------------------------------------------

#[test]
fn eval_skill_routing_runs_5_query_stub_fixture_without_panic() {
    // Minimal fixture covers 3 of the 5 paper tiers (T1 explicit, T2
    // synonym, T3 implicit). The full 150-query stratified test (30 per
    // tier x 5 tiers) lives behind RUNPOD-PENDING gate
    // `gates_eval_skills_150queries_qwen3_8b` because the paper's
    // 44% -> 83.3% baseline requires Claude Opus 4.6 inference, not a
    // local model — the deterministic check here only asserts the harness
    // surface is wired correctly.
    let registry = vec![
        fixture_manifest("planner", "1.0.0", &["plan", "design"]),
        fixture_manifest("writer", "1.0.0", &["write", "compose"]),
        fixture_manifest("researcher", "1.0.0", &["research", "investigate"]),
    ];
    let queries = vec![
        SkillEvalQuery {
            question: "plan the project structure".into(),
            expected_skill_name: "planner".into(),
            tier: 1,
        },
        SkillEvalQuery {
            question: "design the architecture for the new module".into(),
            expected_skill_name: "planner".into(),
            tier: 1,
        },
        SkillEvalQuery {
            question: "compose a release note paragraph".into(),
            expected_skill_name: "writer".into(),
            tier: 2,
        },
        SkillEvalQuery {
            question: "investigate the cause of the production incident".into(),
            expected_skill_name: "researcher".into(),
            tier: 3,
        },
        SkillEvalQuery {
            question: "write a brief summary for the team".into(),
            expected_skill_name: "writer".into(),
            tier: 2,
        },
    ];

    let result = eval_skill_routing(&queries, &registry);
    assert_eq!(result.total_queries, 5);
    // Stub must classify ALL queries correctly — the trigger words in the
    // questions deliberately match the manifest triggers. If this drops
    // below 5/5 it signals a regression in the registry's scoring path.
    assert_eq!(
        result.full_match_count, 5,
        "stub fixture must full-match all 5 queries; got {result:?}"
    );
    assert!((result.full_match_rate - 1.0).abs() < 1e-5);
    // Per-tier checks: T1 had 2 queries, T2 had 2, T3 had 1.
    assert!((result.per_tier_match[1] - 1.0).abs() < 1e-5);
    assert!((result.per_tier_match[2] - 1.0).abs() < 1e-5);
    assert!((result.per_tier_match[3] - 1.0).abs() < 1e-5);
}
