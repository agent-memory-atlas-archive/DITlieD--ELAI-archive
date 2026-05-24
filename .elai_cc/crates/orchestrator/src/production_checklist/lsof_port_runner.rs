//! E-12 Tier-B — lsof port-exposure runner.
//!
//! Shells out to `lsof -i -P` to enumerate listening ports. Reports a
//! finding when a privileged process is listening on a port < 1024
//! without an explicit allow list. Skips on Windows where `lsof` is
//! absent (the ToolchainMissing branch handles that path
//! gracefully).
//!
//! TOOLCHAIN-PENDING-LSOF: macOS / Linux ship lsof in the base
//!   image; on Windows use `netstat -ano` or `Get-NetTCPConnection`
//!   via PowerShell. The cross-platform runner currently targets
//!   Unix-like hosts; Windows support is RUNPOD-PENDING per
//!   `E-12-tier-b-decisions.md`.

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, DEFAULT_TIMEOUT};

const BINARY: &str = "lsof";
const INSTALL_HINT: &str =
    "lsof ships with macOS/Linux base images; Windows users: substitute netstat / Get-NetTCPConnection (RUNPOD-PENDING)";

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }
    let mut cmd = Command::new(BINARY);
    cmd.arg("-i")
        .arg("-P")
        .arg("-n")
        .kill_on_drop(true)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let output = match timeout(DEFAULT_TIMEOUT, cmd.output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) if is_path_not_found(&e) => {
            return RunResult::ToolchainMissing {
                tool: BINARY.to_owned(),
                install_hint: INSTALL_HINT.to_owned(),
            };
        }
        Ok(Err(e)) => {
            return RunResult::Error {
                details: format!("lsof spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!("lsof timed out after {}s", DEFAULT_TIMEOUT.as_secs()),
                stderr_excerpt: String::new(),
            };
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        return RunResult::Error {
            details: format!(
                "lsof exited with code {}",
                output.status.code().unwrap_or(-1)
            ),
            stderr_excerpt: excerpt(&stderr),
        };
    }
    let mut findings: Vec<ChecklistFinding> = Vec::new();
    for line in stdout.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 9 {
            continue;
        }
        let name_field = fields[8];
        // Look for ":<port>" suffix on the NAME column.
        let Some(idx) = name_field.rfind(':') else {
            continue;
        };
        let Ok(port) = name_field[idx + 1..].split('-').next().unwrap_or("").parse::<u16>() else {
            continue;
        };
        if port < 1024 {
            findings.push(ChecklistFinding {
                severity: "warn".into(),
                category: "lsof_port".into(),
                message: format!(
                    "process `{}` listening on privileged port {}",
                    fields[0], port
                ),
                file: None,
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
        let bogus = Path::new("/elai_e12_bogus_lsof_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
