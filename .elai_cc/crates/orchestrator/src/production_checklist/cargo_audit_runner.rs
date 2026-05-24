//! E-12 Tier-B — cargo audit runner.
//!
//! Shells out to `cargo audit --json` from the target directory.
//!
//! TOOLCHAIN-PENDING-CARGO_AUDIT: cargo install cargo-audit ||
//!   apt-get install cargo-audit (https://github.com/rustsec/rustsec).

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, DEFAULT_TIMEOUT};

const BINARY: &str = "cargo";
const INSTALL_HINT: &str = "install cargo-audit via cargo install cargo-audit";

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }
    let mut cmd = Command::new(BINARY);
    cmd.arg("audit")
        .arg("--json")
        .current_dir(target)
        .kill_on_drop(true)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let output = match timeout(DEFAULT_TIMEOUT, cmd.output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) if is_path_not_found(&e) => {
            return RunResult::ToolchainMissing {
                tool: "cargo-audit".to_owned(),
                install_hint: INSTALL_HINT.to_owned(),
            };
        }
        Ok(Err(e)) => {
            return RunResult::Error {
                details: format!("cargo audit spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!("cargo audit timed out after {}s", DEFAULT_TIMEOUT.as_secs()),
                stderr_excerpt: String::new(),
            };
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    let root = match parsed {
        Ok(v) => v,
        Err(_) => {
            // cargo audit prints "no Cargo.lock found" to stderr +
            // exits non-zero before writing JSON when the target
            // is not a Rust project. Treat as toolchain not
            // applicable, NOT as runner error.
            if stderr.contains("Cargo.lock") || stderr.contains("Cargo.toml") {
                return RunResult::Ok {
                    findings: Vec::new(),
                };
            }
            return RunResult::Error {
                details: "cargo audit output unparseable".into(),
                stderr_excerpt: excerpt(&stderr),
            };
        }
    };
    let mut findings: Vec<ChecklistFinding> = Vec::new();
    if let Some(vulns) = root
        .get("vulnerabilities")
        .and_then(|v| v.get("list"))
        .and_then(|v| v.as_array())
    {
        for v in vulns {
            let advisory_id = v
                .get("advisory")
                .and_then(|a| a.get("id"))
                .and_then(|x| x.as_str())
                .unwrap_or("RUSTSEC")
                .to_owned();
            let package = v
                .get("package")
                .and_then(|p| p.get("name"))
                .and_then(|x| x.as_str())
                .unwrap_or("<unknown>")
                .to_owned();
            findings.push(ChecklistFinding {
                severity: "block".into(),
                category: "cargo_audit".into(),
                message: format!("{}: {}", package, advisory_id),
                file: Some("Cargo.lock".into()),
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
        let bogus = Path::new("/elai_e12_bogus_cargoaudit_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
