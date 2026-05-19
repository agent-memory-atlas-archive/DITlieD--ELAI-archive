// C-18 B-35 backfill — Designer artifact briefing injection contract.
//
// `build_existing_project_context_impl` is the C-18 D3 public alias for the
// task_runner-private `build_existing_project_context`. It returns the
// EXISTING_ENTRY_POINTS + AVAILABLE_ARTIFACTS context blocks injected into
// the Designer's decompose call for "existing project" mode.
//
// Two assertions:
//   1. test_existing_project_injects_available_artifacts:
//      a project with two [[knowledge.artifacts]] entries produces a briefing
//      string containing "AVAILABLE_ARTIFACTS" + both artifact names.
//   2. test_greenfield_skips_artifact_block:
//      an empty / no-manifest project produces a briefing string that does
//      NOT contain "AVAILABLE_ARTIFACTS".
//
// Manifest loader contract anchor (crates/knowledge/src/artifacts/manifest.rs:103):
//   - precedence: `[[knowledge.artifacts]]` in elai_cc.toml, else
//     `.elai_artifacts.json` at project root, else empty.
//   - each entry requires non-empty `name`/`path`/`description`.
//   - `path` is resolved by `path::Path::canonicalize` against the
//     canonicalised `project_root` (must exist).

use std::fs;

use elai_orchestrator::task_runner::build_existing_project_context_impl;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helper: build a project tempdir with `[[knowledge.artifacts]]` entries.
// ---------------------------------------------------------------------------

fn write_two_artifact_project(root: &std::path::Path) {
    // Create the artifact target files first — manifest::resolve_spec
    // canonicalises and verifies path existence.
    let docs_dir = root.join("docs");
    fs::create_dir_all(&docs_dir).expect("create docs/");
    fs::write(docs_dir.join("api_contract.md"), "# API contract\n")
        .expect("write api_contract.md");
    fs::write(docs_dir.join("design_doc.md"), "# Design doc\n")
        .expect("write design_doc.md");

    let toml_content = "\
[[knowledge.artifacts]]
name = \"api_contract\"
path = \"docs/api_contract.md\"
description = \"API contract for the public REST surface (auth, rate limits, error shapes).\"

[[knowledge.artifacts]]
name = \"design_doc\"
path = \"docs/design_doc.md\"
description = \"High-level architecture overview, module boundaries, data flow diagram.\"
";
    fs::write(root.join("elai_cc.toml"), toml_content).expect("write elai_cc.toml");
}

// ---------------------------------------------------------------------------
// 1. Existing project — manifest contains two artifacts -> briefing names them.
// ---------------------------------------------------------------------------

#[test]
fn test_existing_project_injects_available_artifacts() {
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();
    write_two_artifact_project(project_root);

    let briefing = build_existing_project_context_impl(project_root);

    assert!(
        briefing.contains("AVAILABLE_ARTIFACTS"),
        "briefing must contain the AVAILABLE_ARTIFACTS header for a project with a manifest; \
         briefing was: {:?}",
        briefing,
    );
    assert!(
        briefing.contains("api_contract"),
        "briefing must reference the first artifact `api_contract`; briefing was: {:?}",
        briefing,
    );
    assert!(
        briefing.contains("design_doc"),
        "briefing must reference the second artifact `design_doc`; briefing was: {:?}",
        briefing,
    );
}

// ---------------------------------------------------------------------------
// 2. Greenfield — no manifest, no DKG -> briefing has no artifact block.
// ---------------------------------------------------------------------------

#[test]
fn test_greenfield_skips_artifact_block() {
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();
    // Intentionally do NOT write elai_cc.toml or .elai_artifacts.json.

    let briefing = build_existing_project_context_impl(project_root);

    assert!(
        !briefing.contains("AVAILABLE_ARTIFACTS"),
        "briefing for a greenfield project (no manifest, no DKG) must NOT contain the \
         AVAILABLE_ARTIFACTS block; briefing was: {:?}",
        briefing,
    );
}
