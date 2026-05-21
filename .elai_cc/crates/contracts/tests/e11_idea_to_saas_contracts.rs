//! E-11 Tier-A — idea-to-SaaS scope clarity regression contracts.
//!
//! Four negative-space guards + walker sanity, using the shared
//! `tests/common/mod.rs` workspace walker (factored in E-12.A).

mod common;

use common::{collect_hits, format_hits, workspace_rs_files};

#[test]
fn no_parallel_critic_verdict_type() {
    let banned = [
        "struct IdeaCriticVerdict",
        "enum IdeaCriticVerdict",
        "struct CriticVerdictV2",
        "enum CriticRoundOutcome",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Parallel critic verdict types banned — extend CriticRound + CriticFinding.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_demo_grade_scaffold_emit() {
    let banned = [
        "fn emit_demo_scaffold",
        "fn build_demo_grade_app",
        "fn for_show_only_scaffold",
        "const SCAFFOLD_DEMO_GRADE",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Demo-grade scaffold emitters banned — force explicit demo class opt-in.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_parallel_package_verifier() {
    let banned = [
        "struct PackageVerifierV2",
        "struct BespokePackageVerifier",
        "trait QuickPackageCheck",
        "fn skip_package_verification",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Parallel package verifiers banned — extend safety::PackageVerifier.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_quick_audit_mode_bypassing_15_item_checklist() {
    let banned = [
        "fn quick_audit_mode",
        "fn skip_production_checklist",
        "AuditorMode::DemoOnly",
        "AuditorMode::SkipChecklist",
        "fn audit_without_checklist",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Quick-audit modes bypassing the production checklist banned.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn workspace_walker_finds_at_least_one_rs_file() {
    let files = workspace_rs_files();
    assert!(
        !files.is_empty(),
        "e11 contract walker found zero .rs files — the four anti-pattern asserts cannot be trusted",
    );
}
