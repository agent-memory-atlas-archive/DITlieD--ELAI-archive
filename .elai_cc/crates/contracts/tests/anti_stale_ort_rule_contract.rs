// E-34 contract: anti-stale Rust rule for the ort crate matches reality.
//
// Verifies the CLAUDE.md "Anti-Stale-Training" Rust line carries the
// amended rc-track wording (E-34 task 1) AND that the broken
// "ort stable (never -rc)" wording is gone. Also verifies:
//
// 1. swiftide REJECTED line present (E-34 task 11).
// 2. The canonical ort pin at crates/knowledge/Cargo.toml:54 remains
//    `=2.0.0-rc.12` (acceptance: ort pin UNCHANGED by E-34).
// 3. The B-PLAN-SUMMARY.md B-16 spec line is corrected to "rc-track pin
//    maintained" (E-34 task 3).
// 4. The outdated-tech-audit.md file exists with the 2026-05 ort
//    appendix (E-34 task 2).
//
// CI-resilient: when run inside a sandbox that omits the workspace-root
// CLAUDE.md / docs/ tree, each individual check soft-skips with an
// explanatory message rather than panicking. The harness still fails
// loudly when the files ARE present but carry stale content — which is
// the regression case this contract guards.
//
// Source-grounding: workflow/reasoning/E-34-decisions.md
// (rule-amendment rationale), .elai_cc/outdated-tech-audit.md
// (2026-05 ort appendix the rule cites).

use std::fs;
use std::path::{Path, PathBuf};

/// Resolves `<repo-root>` from the contracts crate manifest. The
/// contracts crate lives at `<elai_cc>/crates/contracts`; the workflow
/// root (where CLAUDE.md lives) is one directory above the
/// `.elai_cc` workspace root.
fn workflow_root() -> PathBuf {
    // CARGO_MANIFEST_DIR -> .elai_cc/crates/contracts
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // .elai_cc/crates/contracts -> .elai_cc/crates -> .elai_cc -> workflow root
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// Resolves the `.elai_cc` workspace root.
fn elai_cc_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// Reads a file if it exists; returns None when absent (CI-resilient soft
/// skip). Surfaces panic on read error so the harness fails loudly on
/// permission / corruption issues.
fn read_if_present(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }
    Some(
        fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display())),
    )
}

#[test]
fn claude_md_carries_rc_track_amendment() {
    let claude_md_path = workflow_root().join("CLAUDE.md");
    let Some(content) = read_if_present(&claude_md_path) else {
        eprintln!(
            "soft-skip: {} absent (sandbox without workflow tree)",
            claude_md_path.display()
        );
        return;
    };

    assert!(
        content.contains("ort 2.x rc track acceptable"),
        "CLAUDE.md must carry the E-34 amended ort rule (`ort 2.x rc track acceptable`); see workflow/reasoning/E-34-decisions.md"
    );
    assert!(
        content.contains("=2.0.0-rc.12"),
        "CLAUDE.md amended rule must cite the exact rc pin `=2.0.0-rc.12`"
    );
}

#[test]
fn claude_md_no_longer_says_ort_stable_never_rc() {
    let claude_md_path = workflow_root().join("CLAUDE.md");
    let Some(content) = read_if_present(&claude_md_path) else {
        eprintln!(
            "soft-skip: {} absent (sandbox without workflow tree)",
            claude_md_path.display()
        );
        return;
    };

    assert!(
        !content.contains("ort stable (never -rc)"),
        "CLAUDE.md must NOT carry the pre-E-34 broken rule `ort stable (never -rc)`; the rule contradicts crates/knowledge/Cargo.toml:54 pin `=2.0.0-rc.12`"
    );
}

#[test]
fn claude_md_carries_swiftide_rejection() {
    let claude_md_path = workflow_root().join("CLAUDE.md");
    let Some(content) = read_if_present(&claude_md_path) else {
        eprintln!(
            "soft-skip: {} absent (sandbox without workflow tree)",
            claude_md_path.display()
        );
        return;
    };

    assert!(
        content.contains("swiftide REJECTED"),
        "CLAUDE.md must carry the E-34 swiftide rejection (`swiftide REJECTED`); 57 dl/mo per lib.rs/crates/swiftide"
    );
}

#[test]
fn knowledge_cargo_toml_still_pins_ort_rc12() {
    let cargo_toml = elai_cc_root()
        .join("crates")
        .join("knowledge")
        .join("Cargo.toml");
    let Some(content) = read_if_present(&cargo_toml) else {
        // Hard-fail: the contracts crate is INSIDE elai_cc, so
        // knowledge/Cargo.toml MUST exist when the contracts test runs.
        panic!("knowledge/Cargo.toml missing at {}", cargo_toml.display());
    };

    assert!(
        content.contains(r#"ort = "=2.0.0-rc.12""#),
        "crates/knowledge/Cargo.toml must keep the exact-version `=2.0.0-rc.12` pin (E-34 acceptance: ort pin UNCHANGED)"
    );
}

#[test]
fn b_plan_summary_b16_carries_rc_track_correction() {
    let b_plan_summary = workflow_root()
        .join("docs")
        .join("thefinalglance")
        .join("B-plan")
        .join("B-PLAN-SUMMARY.md");
    let Some(content) = read_if_present(&b_plan_summary) else {
        eprintln!(
            "soft-skip: {} absent (sandbox without workflow tree)",
            b_plan_summary.display()
        );
        return;
    };

    assert!(
        content.contains("ort 2.x rc-track pin maintained"),
        "B-PLAN-SUMMARY.md B-16 spec must carry the E-34 correction (`ort 2.x rc-track pin maintained`); see workflow/reasoning/E-34-decisions.md"
    );
    assert!(
        !content.contains(r#"ort stable (drop "=2.0.0-rc.12""#),
        "B-PLAN-SUMMARY.md must NOT carry the pre-E-34 broken B-16 spec (`ort stable (drop \"=2.0.0-rc.12\"`)"
    );
}

#[test]
fn outdated_tech_audit_carries_2026_05_ort_appendix() {
    let audit_path = elai_cc_root().join("outdated-tech-audit.md");
    let Some(content) = read_if_present(&audit_path) else {
        eprintln!(
            "soft-skip: {} absent (sandbox without workflow tree)",
            audit_path.display()
        );
        return;
    };

    assert!(
        content.contains("2026-05 ort 2.x rc track audit"),
        "outdated-tech-audit.md must carry the 2026-05 ort appendix authored by E-34"
    );
    assert!(
        content.contains("crates/knowledge/Cargo.toml:54"),
        "audit appendix must cite the canonical pin site at crates/knowledge/Cargo.toml:54"
    );
}
