//! C-43 — `elai freeze <status|verify|ratify>` handler.
//!
//! Drives the orchestrator's [`elai_orchestrator::scope_freeze`] module
//! (manifest load + verify + ratify) and renders text / json output.
//!
//! Grounding posture: ELAI design hypothesis. The source plan cited
//! arXiv:2604.21725 (AEL) as ground truth but two independent
//! WebFetches confirmed that paper is Thompson-Sampling agent learning,
//! NOT scope-freeze. The citation has been removed; see
//! workflow/sessions/s-25120/mistakes.md.
//!
//! Exit code conventions:
//!   - `status`: 0 on successful load, 1 if the manifest is missing or
//!     malformed.
//!   - `verify`: 0 on clean verify, 1 on any violation (or load error).
//!   - `ratify`: 0 on success, 1 on write failure.

use std::path::PathBuf;

use elai_orchestrator::scope_freeze::{
    ratify_freeze, verify_freeze, FreezeReport, FreezeViolation, ScopeFreezeManifest,
};

use crate::args::{CliOutputFormat, FreezeAction};

/// Workspace-relative paths covering the Phase 4H frozen surface. Kept
/// in lockstep with `.elai_cc/scope-freeze.toml` `frozen_items` so a
/// fresh `elai freeze ratify` reproduces the canonical manifest.
const DEFAULT_FROZEN_ITEMS: &[&str] = &[
    "crates/orchestrator/src/self_mod_orchestrator.rs",
    "crates/orchestrator/src/self_mod_safety.rs",
    "crates/orchestrator/src/self_mod_layer1a.rs",
    "crates/orchestrator/src/self_mod_types.rs",
    "crates/orchestrator/src/self_mod_bridge.rs",
    "crates/orchestrator/src/self_mod_proposal.rs",
    "crates/orchestrator/src/msi.rs",
    "crates/orchestrator/src/evolution_gate.rs",
    "crates/orchestrator/src/config_archive.rs",
];

/// Resolve the workspace root from the process CWD. Surfaces a typed
/// error string to the caller rather than panicking on a missing CWD.
fn resolve_workspace_root() -> Result<PathBuf, String> {
    std::env::current_dir()
        .map_err(|e| format!("cannot determine workspace root from cwd: {e}"))
}

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(action: FreezeAction, output_format: CliOutputFormat) -> Result<(), String> {
    let workspace_root = resolve_workspace_root()?;
    match action {
        FreezeAction::Status => run_status(&workspace_root, output_format),
        FreezeAction::Verify => run_verify(&workspace_root, output_format),
        FreezeAction::Ratify { ratifier } => {
            run_ratify(&workspace_root, &ratifier, output_format)
        }
    }
}

fn run_status(workspace_root: &std::path::Path, output_format: CliOutputFormat) -> Result<(), String> {
    let manifest = ScopeFreezeManifest::load(workspace_root)
        .map_err(|e| format!("scope-freeze status failed: {e}"))?;
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&manifest)
                .map_err(|e| format!("status json encode failed: {e}"))?;
            println!("{json}");
        }
        _ => {
            println!("scope-freeze manifest:");
            println!("  ratification_ts_ms: {}", manifest.ratification_ts_ms);
            println!("  ratifier_id_hash:  {}", manifest.ratifier_id_hash);
            println!("  manifest_hash:     {}", manifest.manifest_hash());
            println!("  frozen_items ({}):", manifest.frozen_items.len());
            for item in &manifest.frozen_items {
                println!("    - {item}");
            }
        }
    }
    Ok(())
}

fn run_verify(workspace_root: &std::path::Path, output_format: CliOutputFormat) -> Result<(), String> {
    let report = verify_freeze(workspace_root)
        .map_err(|e| format!("scope-freeze verify failed: {e}"))?;
    render_report(&report, output_format);
    if report.is_clean() {
        Ok(())
    } else {
        Err(format!(
            "scope-freeze verify FAILED: {} violation(s)",
            report.violations.len()
        ))
    }
}

fn run_ratify(
    workspace_root: &std::path::Path,
    ratifier: &str,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let frozen_items: Vec<String> = DEFAULT_FROZEN_ITEMS.iter().map(|s| (*s).to_string()).collect();
    let (manifest, manifest_hash) = ratify_freeze(workspace_root, ratifier, frozen_items)
        .map_err(|e| format!("scope-freeze ratify failed: {e}"))?;
    match output_format {
        CliOutputFormat::Json => {
            let body = serde_json::json!({
                "manifest": manifest,
                "manifest_hash": manifest_hash,
            });
            println!("{}", body);
        }
        _ => {
            println!("scope-freeze ratified:");
            println!("  ratification_ts_ms: {}", manifest.ratification_ts_ms);
            println!("  ratifier_id_hash:  {}", manifest.ratifier_id_hash);
            println!("  manifest_hash:     {manifest_hash}");
            println!("  item_count:        {}", manifest.frozen_items.len());
        }
    }
    Ok(())
}

fn render_report(report: &FreezeReport, output_format: CliOutputFormat) {
    match output_format {
        CliOutputFormat::Json => {
            let body = serde_json::json!({
                "violations": report.violations,
                "ratified_at": report.ratified_at,
                "verified_at": report.verified_at,
                "clean": report.is_clean(),
            });
            println!("{body}");
        }
        _ => {
            if report.is_clean() {
                println!(
                    "scope-freeze verify: CLEAN (ratified_at={}, verified_at={})",
                    report.ratified_at, report.verified_at
                );
            } else {
                println!(
                    "scope-freeze verify: {} violation(s)",
                    report.violations.len()
                );
                for v in &report.violations {
                    render_violation(v);
                }
            }
        }
    }
}

fn render_violation(v: &FreezeViolation) {
    println!(
        "  - {} (mtime={}, frozen_since={})",
        v.item_path, v.modified_at_ms, v.frozen_since_ms
    );
}
