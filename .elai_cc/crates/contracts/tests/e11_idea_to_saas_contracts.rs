//! E-11 Tier-A — idea-to-SaaS scope clarity regression contracts.
//!
//! Four negative-space guards + walker sanity, continuing the campaign-wide
//! contract pattern. E-11 builds on the shipped B-40 Idea-Framer (FsmSubstate::
//! Framing, FramedIdea, CriticRound, IdeaFramingWindow, 5 SSB variants) and
//! pre-positions invariants the Tier-B work will need.
//!
//! Today's codebase complies (verified by integration-explorer-elai in
//! s-orch-e11). The contracts make the compliance load-bearing so:
//!   - No-one ships a "demo-grade" SaaS scaffold (KNOWN_DEMO_VS_DEPLOY_CHASMS
//!     pattern) by accident.
//!   - The auditor's ProductionGrade mode (Tier-B) cannot be bypassed via a
//!     parallel `quick_audit` shape that skips the 15-item checklist.
//!   - Package additions don't slopsquat through a parallel verifier.
//!   - Critic round verdicts don't fork into a competing IdeaCriticVerdict
//!     type (plan DRIFT — the canonical type is CriticRound).
//!
//! Tier-B for E-11 will land: 6 new SsbActionType variants + scope_clarity_score
//! field on CriticRound + AuditorMode::ProductionGrade + PackageVerifier in
//! crates/safety + EditFormat enum + per-level critic at Step+BabyStep +
//! anti_spiral_hard_abort + IdeaFramerSecurityPanel.svelte + workflow/archetypes/.
//! See workflow/reasoning/E-11-decisions.md.

use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest.to_path_buf())
}

fn collect_rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    walk_dir(root, &mut out);
    out
}

fn walk_dir(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if matches!(
                name,
                "target"
                    | ".git"
                    | "node_modules"
                    | ".elai"
                    | "indexes"
                    | "scip_indices"
                    | "models"
            ) {
                continue;
            }
            walk_dir(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            // Cross-skip the 4 sibling negative-space contracts so each
            // contract's banned-string literals don't trip the others.
            if matches!(
                name,
                "e11_idea_to_saas_contracts.rs"
                    | "e25_compaction_architecture_contracts.rs"
                    | "e38_anti_pattern_contracts.rs"
                    | "e48_research_contracts.rs"
            ) {
                continue;
            }
            out.push(path);
        }
    }
}

fn file_contains_any(path: &Path, needles: &[&str]) -> Option<String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return None;
    };
    for needle in needles {
        if contents.contains(needle) {
            return Some((*needle).to_string());
        }
    }
    None
}

fn collect_hits(root: &Path, needles: &[&str]) -> Vec<(PathBuf, String)> {
    let mut hits: Vec<(PathBuf, String)> = Vec::new();
    for file in collect_rust_sources(root) {
        if let Some(needle) = file_contains_any(&file, needles) {
            hits.push((file, needle));
        }
    }
    hits
}

fn format_hits(hits: &[(PathBuf, String)]) -> String {
    hits.iter()
        .map(|(p, n)| format!("{} :: {}", p.display(), n))
        .collect::<Vec<_>>()
        .join("\n  ")
}

#[test]
fn no_parallel_critic_verdict_type() {
    // Plan E-11 originally cited "IdeaCriticVerdict" — that type does NOT
    // exist. Canonical type is CriticRound (orchestrator/src/idea_framing/...)
    // with CriticFinding entries. A parallel IdeaCriticVerdict / CriticVerdict
    // type would fragment the critic-loop wire shape.
    let root = workspace_root();
    let banned = [
        "struct IdeaCriticVerdict",
        "enum IdeaCriticVerdict",
        "struct CriticVerdictV2",
        "enum CriticRoundOutcome",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "Parallel critic verdict types banned — extend CriticRound + CriticFinding.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_demo_grade_scaffold_emit() {
    // E-11's KNOWN_DEMO_VS_DEPLOY_CHASMS pattern enumerates what makes a
    // scaffold "demo-grade" (no auth, no rate-limit, no input validation,
    // no error budgets, no observability, no migration plan, etc). Any
    // helper that emits a scaffold marked "demo" or "for-show" is banned —
    // forces the user to either ship a production-grade scaffold OR
    // explicitly opt into the demo class with a visible flag.
    let root = workspace_root();
    let banned = [
        "fn emit_demo_scaffold",
        "fn build_demo_grade_app",
        "fn for_show_only_scaffold",
        "const SCAFFOLD_DEMO_GRADE",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "Demo-grade scaffold emitters banned — force explicit demo class opt-in.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_parallel_package_verifier() {
    // E-11 Tier-B lands a PackageVerifier in crates/safety/ (slopsquatting
    // gate that resolves package name -> repo URL -> last-published date
    // BEFORE allowing an import). A parallel verifier in another crate
    // means imports can route around the safety gate.
    let root = workspace_root();
    let banned = [
        "struct PackageVerifierV2",
        "struct BespokePackageVerifier",
        "trait QuickPackageCheck",
        "fn skip_package_verification",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "Parallel package verifiers banned — extend safety::PackageVerifier.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_quick_audit_mode_bypassing_15_item_checklist() {
    // E-11 Tier-B adds AuditorMode::ProductionGrade with a 15-item checklist
    // (auth / rate-limit / input-validation / migration / observability /
    // backup / etc). A parallel quick_audit / skip_production_checklist /
    // demo_audit lets ship-it-fast culture bypass the gate.
    let root = workspace_root();
    let banned = [
        "fn quick_audit_mode",
        "fn skip_production_checklist",
        "AuditorMode::DemoOnly",
        "AuditorMode::SkipChecklist",
        "fn audit_without_checklist",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "Quick-audit modes bypassing the production checklist banned.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn workspace_walker_finds_at_least_one_rs_file() {
    let root = workspace_root();
    let files = collect_rust_sources(&root);
    assert!(
        !files.is_empty(),
        "e11 contract walker found zero .rs files under {} — the four anti-pattern asserts cannot be trusted",
        root.display()
    );
}
