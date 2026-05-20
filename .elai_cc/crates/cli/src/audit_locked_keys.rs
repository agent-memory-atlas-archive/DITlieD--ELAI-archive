//! D-9 — `elai audit-locked-keys` CLI driver.
//!
//! Thin shell over [`elai_orchestrator::LockedKeysAuditor`]. Resolves
//! the staged-diff (or explicit file list) into a `Vec<PathBuf>` and
//! delegates to the auditor. Exit code 0 on clean audit (or empty
//! registry); 1 on any violation OR registry-unreadable error (fail-
//! closed per WIRING_CONTRACT.stub_ban).
//!
//! Invocation shapes (mirrored from `archive_verify`):
//!   elai audit-locked-keys                           # implicit --staged
//!   elai audit-locked-keys --staged --json
//!   elai audit-locked-keys --files a.rs,b.rs --json
//!   elai audit-locked-keys --registry path/to/registry.toml --files a.rs
//!
//! Default registry path resolves to
//! `<cwd>/scripts/locked_keys_registry.toml` so the binary is invoked
//! from `.elai_cc/`. The pre-commit hook shells out from the workspace
//! root via `cargo run --quiet -p elai-cli -- audit-locked-keys ...`
//! so the cwd matches the convention from the other audit subcommands.

use std::path::PathBuf;
use std::process::Command;

use elai_orchestrator::{LockedKeysAuditError, LockedKeysAuditor};

/// CLI entry point — sync wrapper that builds the staged-files list
/// and delegates to the orchestrator-side auditor.
pub(crate) fn run(
    registry_path: Option<String>,
    files: Option<String>,
    use_staged: bool,
    json: bool,
    _fail_fast: bool,
) -> anyhow::Result<()> {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let registry = registry_path.map(PathBuf::from).unwrap_or_else(|| {
        project_root
            .join("scripts")
            .join("locked_keys_registry.toml")
    });

    let staged = if let Some(list) = files {
        list.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect::<Vec<_>>()
    } else if use_staged {
        collect_staged_via_git(&project_root)?
    } else {
        Vec::new()
    };

    let auditor = match LockedKeysAuditor::load(&registry, None) {
        Ok(a) => a,
        Err(LockedKeysAuditError::RegistryUnreadable { path, detail }) => {
            // Fail-closed: an unreadable registry must NOT silently
            // permit edits. Emit a structured error and exit 1.
            let msg = format!(
                "audit-locked-keys: registry unreadable at {} ({}); failing closed",
                path.display(),
                detail
            );
            if json {
                let body = serde_json::json!({
                    "ok": false,
                    "error": "registry_unreadable",
                    "registry_path": path.display().to_string(),
                    "detail": detail,
                });
                println!("{}", serde_json::to_string_pretty(&body)?);
            } else {
                eprintln!("{msg}");
            }
            std::process::exit(1);
        }
        Err(other) => return Err(anyhow::Error::from(other)),
    };

    let result = auditor.audit_diff(&staged);

    if json {
        let body = serde_json::json!({
            "ok": result.is_clean(),
            "registry_path": registry.display().to_string(),
            "n_audited": staged.len(),
            "n_violations": result.violations.len(),
            "violations": result.violations.iter().map(|v| serde_json::json!({
                "attempted_edit": v.attempted_edit.display().to_string(),
                "attempted_lines": [v.attempted_lines.0, v.attempted_lines.1],
                "locked_key_path": v.locked_key.path_as_posix(),
                "reason": v.locked_key.reason,
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else if result.is_clean() {
        println!(
            "LOCKED-KEYS AUDIT PASS (registry={} n_audited={} n_violations=0)",
            registry.display(),
            staged.len()
        );
    } else {
        for v in &result.violations {
            println!(
                "VIOLATION attempted_edit={} attempted_lines=[{},{}] locked_key={} reason={}",
                v.attempted_edit.display(),
                v.attempted_lines.0,
                v.attempted_lines.1,
                v.locked_key.path_as_posix(),
                v.locked_key.reason
            );
        }
        println!(
            "LOCKED-KEYS AUDIT FAIL ({} violations; registry={})",
            result.violations.len(),
            registry.display()
        );
    }

    if !result.is_clean() {
        std::process::exit(1);
    }
    Ok(())
}

/// Shell out to `git diff --cached --name-only` and parse the
/// newline-separated file list. Returns an empty vec when not in a
/// git repo OR when there are no staged files (both are treated as
/// "nothing to audit, the run row is still emitted" — the auditor's
/// audit-run SSB beacon documents the no-op).
fn collect_staged_via_git(cwd: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(["diff", "--cached", "--name-only"])
        .output();
    let output = match output {
        Ok(o) => o,
        Err(err) => {
            // git not installed / not a repo — return empty and let the
            // auditor emit a clean-no-op run row. The pre-commit hook
            // is unaffected because git itself must be present to fire
            // the hook in the first place.
            tracing::warn!(
                target: "elai_cli::audit_locked_keys",
                "git diff --cached failed: {err}; treating as empty staged set"
            );
            return Ok(Vec::new());
        }
    };
    if !output.status.success() {
        tracing::warn!(
            target: "elai_cli::audit_locked_keys",
            "git diff --cached exited non-zero ({}); stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        return Ok(Vec::new());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect())
}
