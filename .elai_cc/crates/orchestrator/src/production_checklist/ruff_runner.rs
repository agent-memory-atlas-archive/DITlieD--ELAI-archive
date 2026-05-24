//! E-12 Tier-B — ruff linter runner.
//!
//! Shells out to `ruff check --output-format=json <target>`. Ruff
//! consolidates flake8 + isort + pyupgrade + pydocstyle.
//!
//! TOOLCHAIN-PENDING-RUFF: pip install ruff || pipx install ruff ||
//!   uv tool install ruff (https://docs.astral.sh/ruff/installation/).

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, DEFAULT_TIMEOUT};

const BINARY: &str = "ruff";
const INSTALL_HINT: &str = "install ruff via pip install ruff || uv tool install ruff";

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }
    let mut cmd = Command::new(BINARY);
    cmd.arg("check")
        .arg("--output-format=json")
        .arg(target)
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
                details: format!("ruff spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!("ruff timed out after {}s", DEFAULT_TIMEOUT.as_secs()),
                stderr_excerpt: String::new(),
            };
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    // ruff JSON output is an array of diagnostic objects.
    let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&stdout);
    let arr = match parsed {
        Ok(v) => v,
        Err(_) => {
            // Empty stdout + exit 0 = no findings.
            if stdout.trim().is_empty() && output.status.success() {
                return RunResult::Ok {
                    findings: Vec::new(),
                };
            }
            return RunResult::Error {
                details: "ruff output unparseable".into(),
                stderr_excerpt: excerpt(&stderr),
            };
        }
    };
    let mut findings: Vec<ChecklistFinding> = Vec::with_capacity(arr.len());
    for v in arr {
        let code = v
            .get("code")
            .and_then(|x| x.as_str())
            .unwrap_or("ruff")
            .to_owned();
        let message = v
            .get("message")
            .and_then(|x| x.as_str())
            .unwrap_or(&code)
            .to_owned();
        let filename = v
            .get("filename")
            .and_then(|x| x.as_str())
            .map(|s| s.to_owned());
        let line = v
            .get("location")
            .and_then(|l| l.get("row"))
            .and_then(|x| x.as_u64())
            .and_then(|n| u32::try_from(n).ok());
        findings.push(ChecklistFinding {
            severity: "warn".into(),
            category: "ruff".into(),
            message: format!("{}: {}", code, message),
            file: filename,
            line,
        });
    }
    RunResult::Ok { findings }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_target_returns_error() {
        let bogus = Path::new("/elai_e12_bogus_ruff_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
