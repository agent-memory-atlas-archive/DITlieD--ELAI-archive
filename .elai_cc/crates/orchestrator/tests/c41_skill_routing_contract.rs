//! C-41 sub-phase 5A — application-skill routing contract test.
//!
//! Asserts that the 9 shipped SKILL.md manifests under `.elai_cc/skills/`
//! route real-world queries to the expected skill via the deterministic
//! `top_k_skills` ranker. Two cases:
//!   1. A high-confidence SaaS plan prompt routes to `heavy_plan` with
//!      score >= `SKILL_CONFIDENCE_THRESHOLD`.
//!   2. An off-topic factual prompt either falls below the threshold OR
//!      resolves to `simple_query` — the negative branch confirms the
//!      threshold gate is the correct fall-through trigger.
//!
//! The test uses a fixture copy of `.elai_cc/skills/` so the run is
//! hermetic and does not depend on whether the repo's canonical skills
//! directory has been mutated mid-session. The fixture is built from the
//! 9 canonical SKILL.md files at fixed-path resolution time.

use std::fs;
use std::path::{Path, PathBuf};

use elai_orchestrator::skill_registry::{
    find_skills, top_k_skills, DiscoveryRoot, SkillContext, SKILL_CONFIDENCE_THRESHOLD,
};

use tempfile::TempDir;

/// Resolve the workspace's canonical `.elai_cc/skills/` directory.
///
/// Test files run with CWD inside the orchestrator crate; the application
/// skills root is two directories up from `crates/orchestrator/`.
fn canonical_skills_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_skills = Path::new(manifest_dir)
        .join("..")
        .join("..")
        .join("skills");
    workspace_skills
        .canonicalize()
        .unwrap_or(workspace_skills)
}

/// Copy every `<name>/SKILL.md` from the canonical skills root into the
/// supplied tempdir, preserving the per-skill directory layout the
/// discovery walker expects.
fn copy_canonical_skills_into(tmp_root: &Path) {
    let canonical = canonical_skills_root();
    let entries = fs::read_dir(&canonical)
        .unwrap_or_else(|e| panic!("canonical skills root unreadable at {}: {e}", canonical.display()));
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        let skill_md = entry_path.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }
        let name = entry_path
            .file_name()
            .expect("skill dir has name")
            .to_string_lossy()
            .to_string();
        let dest_dir = tmp_root.join(&name);
        fs::create_dir_all(&dest_dir).expect("create dest skill dir");
        let body = fs::read_to_string(&skill_md).expect("read canonical SKILL.md");
        fs::write(dest_dir.join("SKILL.md"), body).expect("write fixture SKILL.md");
    }
}

#[test]
fn nine_canonical_skill_manifests_load() {
    let tmp = TempDir::new().expect("tempdir");
    copy_canonical_skills_into(tmp.path());
    let roots = vec![DiscoveryRoot::skills_dir(tmp.path().to_path_buf())];
    let manifests = find_skills(&roots).expect("discovery succeeds");
    assert_eq!(
        manifests.len(),
        9,
        "expected exactly 9 canonical application skills, got {}",
        manifests.len()
    );

    let names: Vec<&str> = manifests.iter().map(|m| m.name.as_str()).collect();
    for expected in [
        "simple_query",
        "focused_edit",
        "heavy_plan",
        "idea_intake",
        "oracle",
        "planning_workflow",
        "paper_ingest",
        "source_ingest",
        "benchmark_run",
    ] {
        assert!(
            names.contains(&expected),
            "expected skill '{expected}' not found in registry: {names:?}"
        );
    }
}

#[test]
fn heavy_plan_routes_for_saas_dashboard_prompt() {
    let tmp = TempDir::new().expect("tempdir");
    copy_canonical_skills_into(tmp.path());
    let roots = vec![DiscoveryRoot::skills_dir(tmp.path().to_path_buf())];
    let manifests = find_skills(&roots).expect("discovery succeeds");

    let ctx = SkillContext::from_text("Plan a SaaS dashboard for invoice tracking");
    let ranked = top_k_skills(&manifests, &ctx, 3);
    assert!(!ranked.is_empty(), "ranker must return at least one candidate");
    assert_eq!(
        ranked[0].manifest.name, "heavy_plan",
        "top-1 for SaaS plan prompt must be heavy_plan, got {} (full ranking: {:?})",
        ranked[0].manifest.name,
        ranked
            .iter()
            .map(|s| (s.manifest.name.clone(), s.score))
            .collect::<Vec<_>>()
    );
    assert!(
        ranked[0].score >= SKILL_CONFIDENCE_THRESHOLD,
        "heavy_plan top score must clear SKILL_CONFIDENCE_THRESHOLD ({}), got {}",
        SKILL_CONFIDENCE_THRESHOLD,
        ranked[0].score
    );
}

#[test]
fn off_topic_prompt_falls_through_or_resolves_simple_query() {
    let tmp = TempDir::new().expect("tempdir");
    copy_canonical_skills_into(tmp.path());
    let roots = vec![DiscoveryRoot::skills_dir(tmp.path().to_path_buf())];
    let manifests = find_skills(&roots).expect("discovery succeeds");

    // Pure-trivia query with no plan / edit / research markers — should
    // either fail to clear the confidence threshold (preferred) OR resolve
    // to simple_query (acceptable fallback).
    let ctx =
        SkillContext::from_text("What is the airspeed velocity of an unladen swallow?");
    let ranked = top_k_skills(&manifests, &ctx, 1);
    let top = ranked.first().expect("ranker must return one candidate");

    let below_threshold = top.score < SKILL_CONFIDENCE_THRESHOLD;
    let resolves_to_simple_query = top.manifest.name == "simple_query";
    assert!(
        below_threshold || resolves_to_simple_query,
        "off-topic prompt expected to fall through (score < threshold) OR resolve to simple_query; got skill={} score={}",
        top.manifest.name,
        top.score
    );
}

#[test]
fn type_application_field_present_in_canonical_manifests() {
    // The `type: application` discriminator is not parsed by
    // parse_skill_frontmatter_extended (unknown keys are silently ignored)
    // but MUST be present in every canonical SKILL.md as a forward-compat
    // marker for future namespace-aware routing. This test reads the
    // raw markdown so it catches drift even when the parser doesn't.
    let canonical = canonical_skills_root();
    let entries = fs::read_dir(&canonical)
        .unwrap_or_else(|e| panic!("canonical skills root unreadable at {}: {e}", canonical.display()));
    for entry in entries.flatten() {
        let skill_md = entry.path().join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }
        let body = fs::read_to_string(&skill_md).expect("read SKILL.md");
        assert!(
            body.contains("type: application"),
            "canonical SKILL.md at {} missing required `type: application` discriminator",
            skill_md.display()
        );
    }
}
