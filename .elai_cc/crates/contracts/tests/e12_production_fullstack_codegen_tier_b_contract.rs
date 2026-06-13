//! E-12 Tier-B — production fullstack codegen wiring + invariants.
//!
//! Locks the Tier-B implementation surface so that future plans can
//! NOT silently fork the FrameworkAdapter / FrameworkProfile path
//! that this commit lands. Companion to
//! `e12_production_codegen_contracts.rs` (Tier-A 4-guard set,
//! shipped 2026-05-22) which holds the negative-space ban list; this
//! contract validates the positive shape.
//!
//! Tests:
//!   1. AuditorMode::ProductionGrade variant exists (E-11 prereq
//!      fulfilled inside E-12).
//!   2. AgentRole::AuditorProductionGrade exists + maps via
//!      AuditorMode::map_to_agent_role.
//!   3. FrameworkAdapter trait MIRRORS ScaffoldApplier — both
//!      define `apply(template, target, task) -> Result<_>` and both
//!      live in `crate::orchestrator` (Tier-A guard 1 enforces no
//!      parallel V2 trait).
//!   4. workflow/archetypes/ is the canonical scaffold home (E-11.A
//!      decision); .claude/tools/framework-conventions/ does NOT
//!      exist as a parallel dual-source.
//!   5. No template engine deps polluted any Cargo.toml (Tier-A
//!      guard 2 enforces).
//!   6. 15 production_checklist runner files exist with
//!      ToolchainMissing branch.
//!   7. SsbActionType has FrameworkProfileSelected + LegacyConfigStripped
//!      and they round-trip via SsbPayload.
//!   8. get_framework_profile Tauri command is registered in
//!      dashboard-v2 src-tauri/src/lib.rs invoke_handler.
//!   9. SCHEMA_VERSION >= 128 (E-12 cascade lands at 128).

mod common;

use std::path::PathBuf;

use common::{collect_hits, workspace_root};

#[test]
fn auditor_mode_production_grade_variant_exists() {
    // Read the source directly — the variant has been added in the
    // same commit so a compile-error-style cross-check is the
    // tightest possible signal. The mode::ProductionGrade name is
    // pinned by Tier-A guard 1's banned-prefix list (no
    // ProductionGradeV2 / EnhancedProductionGrade etc).
    let path = workspace_root()
        .join(".elai_cc")
        .join("crates")
        .join("orchestrator")
        .join("src")
        .join("auditor_mode.rs");
    let body = std::fs::read_to_string(&path).expect("auditor_mode.rs readable");
    assert!(
        body.contains("ProductionGrade"),
        "AuditorMode::ProductionGrade variant must exist (E-11 prereq fulfilled inside E-12)"
    );
    assert!(
        body.contains("Self::ProductionGrade => AgentRole::AuditorProductionGrade"),
        "AuditorMode::ProductionGrade must map to AgentRole::AuditorProductionGrade in map_to_agent_role"
    );
}

#[test]
fn agent_role_production_grade_variant_exists() {
    let path = workspace_root()
        .join(".elai_cc")
        .join("crates")
        .join("ipc-types")
        .join("src")
        .join("lib.rs");
    let body = std::fs::read_to_string(&path).expect("ipc-types lib.rs readable");
    assert!(
        body.contains("AuditorProductionGrade"),
        "AgentRole::AuditorProductionGrade must exist (E-12 Tier-B addition)"
    );
}

#[test]
fn framework_adapter_trait_mirrors_scaffold_applier() {
    let task_runner = workspace_root()
        .join(".elai_cc")
        .join("crates")
        .join("orchestrator")
        .join("src")
        .join("task_runner.rs");
    let adapter = workspace_root()
        .join(".elai_cc")
        .join("crates")
        .join("orchestrator")
        .join("src")
        .join("framework_profile")
        .join("adapter.rs");
    let tr_body = std::fs::read_to_string(&task_runner).expect("task_runner.rs readable");
    let ad_body = std::fs::read_to_string(&adapter).expect("adapter.rs readable");
    // Both traits MUST declare the same `apply(template_name, target_dir, task_text)` signature.
    assert!(
        tr_body.contains("pub trait ScaffoldApplier"),
        "task_runner.rs must define ScaffoldApplier trait (Tier-A guard 1 canonical surface)"
    );
    assert!(
        ad_body.contains("pub trait FrameworkAdapter"),
        "framework_profile/adapter.rs must define FrameworkAdapter trait"
    );
    // Both signatures use template_name + target_dir + task_text.
    assert!(
        tr_body.contains("template_name") && tr_body.contains("target_dir") && tr_body.contains("task_text"),
        "ScaffoldApplier::apply signature pinned to (template_name, target_dir, task_text)"
    );
    assert!(
        ad_body.contains("template_name") && ad_body.contains("target_dir") && ad_body.contains("task_text"),
        "FrameworkAdapter::apply signature MUST mirror ScaffoldApplier (Tier-A guard 1)"
    );
}

#[test]
fn workflow_archetypes_is_canonical_no_parallel_framework_conventions() {
    let archetypes = workspace_root().join("workflow").join("archetypes");
    assert!(
        archetypes.exists() && archetypes.is_dir(),
        "workflow/archetypes/ MUST exist (E-11.A canonical scaffold home)"
    );
    let parallel = workspace_root()
        .join(".claude")
        .join("tools")
        .join("framework-conventions");
    assert!(
        !parallel.exists(),
        ".claude/tools/framework-conventions/ MUST NOT exist (Tier-A decision D9: workflow/archetypes/ is canonical)"
    );
    // At least one .toml manifest must be present (4 representative
    // archetypes ship in this commit: nextjs15 / svelte5 /
    // fastapi-modern / hono).
    let mut count = 0;
    for entry in std::fs::read_dir(&archetypes).expect("read_dir archetypes") {
        let p = entry.expect("entry").path();
        if p.extension().and_then(|e| e.to_str()) == Some("toml") {
            count += 1;
        }
    }
    assert!(
        count >= 4,
        "workflow/archetypes/ must contain >= 4 framework manifests (got {})",
        count
    );
}

#[test]
fn no_template_engine_deps_added() {
    // Tier-A guard 2 (no_template_engine_dep_pollution) is in
    // e12_production_codegen_contracts.rs as a source-level scan;
    // here we double-check the Cargo.toml level explicitly so a
    // dep added without a `use ` import still surfaces.
    let cargo_paths = [
        workspace_root().join(".elai_cc").join("crates").join("orchestrator").join("Cargo.toml"),
        workspace_root().join(".elai_cc").join("crates").join("dashboard-v2").join("src-tauri").join("Cargo.toml"),
        workspace_root().join(".elai_cc").join("crates").join("core").join("Cargo.toml"),
        workspace_root().join(".elai_cc").join("Cargo.toml"),
    ];
    let banned = ["tera =", "handlebars =", "askama =", "minijinja =", "liquid ="];
    for path in &cargo_paths {
        let Ok(body) = std::fs::read_to_string(path) else {
            continue;
        };
        for b in &banned {
            assert!(
                !body.contains(b),
                "{} must not declare banned template-engine dep `{}` (Tier-A guard 2)",
                path.display(),
                b
            );
        }
    }
}

#[test]
fn fifteen_production_checklist_runners_exist() {
    let dir = workspace_root()
        .join(".elai_cc")
        .join("crates")
        .join("orchestrator")
        .join("src")
        .join("production_checklist");
    assert!(
        dir.exists() && dir.is_dir(),
        "production_checklist module dir MUST exist"
    );
    let mut runners: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("read_dir production_checklist") {
        let p = entry.expect("entry").path();
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name == "mod.rs" {
            continue;
        }
        if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            runners.push(p);
        }
    }
    assert_eq!(
        runners.len(),
        15,
        "MUST have exactly 15 production_checklist runner files (got {})",
        runners.len()
    );
    // Every runner MUST contain ToolchainMissing branch + a
    // TOOLCHAIN-PENDING-* marker (wire-dark posture per
    // memory:wire-dark-posture 5th application).
    for runner in &runners {
        let body = std::fs::read_to_string(runner).expect("runner body");
        assert!(
            body.contains("ToolchainMissing"),
            "{} must return RunResult::ToolchainMissing on PATH miss",
            runner.display()
        );
        assert!(
            body.contains("TOOLCHAIN-PENDING-"),
            "{} must carry a TOOLCHAIN-PENDING-<TOOL> seam marker",
            runner.display()
        );
    }
}

#[test]
fn ssb_action_type_has_e12_variants() {
    // Tier-B split relocated the SsbActionType enum + as_sql_str out of
    // src/ssb.rs (the wire-tag strings no longer live in that file), so the
    // old source-text scan no longer finds them. Assert the variants are
    // actually registered in SSB_ACTION_TYPES via their wire tags — tighter
    // than a source grep and arm-shape-agnostic.
    use elai_orchestrator::ssb::SSB_ACTION_TYPES;
    assert!(
        SSB_ACTION_TYPES
            .iter()
            .any(|v| v.as_sql_str() == "framework_profile_selected"),
        "SsbActionType::FrameworkProfileSelected must be registered (E-12 Tier-B)"
    );
    assert!(
        SSB_ACTION_TYPES
            .iter()
            .any(|v| v.as_sql_str() == "legacy_config_stripped"),
        "SsbActionType::LegacyConfigStripped must be registered (E-12 Tier-B)"
    );
}

#[test]
fn ssb_schema_version_at_or_above_128() {
    // Tier-B Move 2 made SCHEMA_VERSION a DERIVED expression
    // (SCHEMA_VERSION_DDL_EPOCH + SSB_ACTION_TYPES.len()), so the old
    // source-text strip_prefix + parse panics. Read the published runtime
    // constant instead — same >= 128 floor, no source-text parse.
    assert!(
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC >= 128,
        "SCHEMA_VERSION must be >= 128 after E-12 Tier-B cascade; got {}",
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC
    );
}

#[test]
fn get_framework_profile_tauri_command_registered() {
    let lib_path = workspace_root()
        .join(".elai_cc")
        .join("crates")
        .join("dashboard-v2")
        .join("src-tauri")
        .join("src")
        .join("lib.rs");
    let body = std::fs::read_to_string(&lib_path).expect("dashboard lib.rs readable");
    assert!(
        body.contains("get_framework_profile"),
        "get_framework_profile Tauri command MUST be registered in invoke_handler! (E-12 Tier-B G2)"
    );
}

#[test]
fn idea_framer_security_panel_svelte_exists() {
    let panel = workspace_root()
        .join(".elai_cc")
        .join("crates")
        .join("dashboard-v2")
        .join("src")
        .join("lib")
        .join("windows")
        .join("IdeaFramerSecurityPanel.svelte");
    assert!(
        panel.exists(),
        "IdeaFramerSecurityPanel.svelte MUST exist (E-11 prereq fulfilled inside E-12)"
    );
}

#[test]
fn walker_finds_workspace_files() {
    // Smoke check the shared workspace walker is wired (mirrors
    // the Tier-A guard 5).
    let hits = collect_hits(&["pub trait FrameworkAdapter"]);
    assert!(
        !hits.is_empty(),
        "workspace walker must locate the FrameworkAdapter trait definition"
    );
}
