//! E-12 Tier-B — cargo-machete (unused-dependency) runner.
//!
//! Shells out to `cargo machete --with-metadata` from the target.
//! Surfaces unused Rust dependencies — relevant for SaaS Rust
//! backends shipping with the production-grade auditor.
//!
//! TOOLCHAIN-PENDING-CARGO_MACHETE: cargo install cargo-machete
//!   (https://github.com/bnjbvr/cargo-machete).

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, DEFAULT_TIMEOUT};

const BINARY: &str = "cargo";
const INSTALL_HINT: &str = "install cargo-machete via cargo install cargo-machete";

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }
    let mut cmd = Command::new(BINARY);
    cmd.arg("machete")
        .current_dir(target)
        .kill_on_drop(true)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let output = match timeout(DEFAULT_TIMEOUT, cmd.output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) if is_path_not_found(&e) => {
            return RunResult::ToolchainMissing {
                tool: "cargo-machete".to_owned(),
                install_hint: INSTALL_HINT.to_owned(),
            };
        }
        Ok(Err(e)) => {
            return RunResult::Error {
                details: format!("cargo-machete spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!(
                    "cargo-machete timed out after {}s",
                    DEFAULT_TIMEOUT.as_secs()
                ),
                stderr_excerpt: String::new(),
            };
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    // cargo-machete exit 0 = clean; 1 = unused deps found.
    // Plain-text output; count lines starting with whitespace +
    // package name + path.
    if !output.status.success() && stdout.is_empty() {
        return RunResult::Error {
            details: format!(
                "cargo-machete exited with code {}",
                output.status.code().unwrap_or(-1)
            ),
            stderr_excerpt: excerpt(&stderr),
        };
    }
    let mut findings: Vec<ChecklistFinding> = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        // Look for `\t<package_name>` lines under "unused dependencies"
        // sections. cargo-machete prefixes each unused dep with a tab.
        if line.starts_with('\t') && !trimmed.is_empty() {
            findings.push(ChecklistFinding {
                severity: "warn".into(),
                category: "cargo_machete".into(),
                message: format!("unused dependency: {}", trimmed),
                file: Some("Cargo.toml".into()),
                line: None,
            });
        }
    }
    RunResult::Ok { findings }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_target_returns_error() {
        let bogus = Path::new("/elai_e12_bogus_machete_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
