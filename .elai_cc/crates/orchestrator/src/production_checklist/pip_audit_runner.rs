//! E-12 Tier-B — pip-audit runner.
//!
//! Shells out to `pip-audit --format json -r <target>/requirements.txt` (or against the venv when no requirements.txt is present).
//!
//! TOOLCHAIN-PENDING-PIP_AUDIT: pip install pip-audit ||
//!   pipx install pip-audit (https://github.com/pypa/pip-audit).

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, DEFAULT_TIMEOUT};

const BINARY: &str = "pip-audit";
const INSTALL_HINT: &str = "install pip-audit via pip install pip-audit || pipx install pip-audit";

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }
    let mut cmd = Command::new(BINARY);
    cmd.arg("--format")
        .arg("json")
        .current_dir(target)
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
                details: format!("pip-audit spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!("pip-audit timed out after {}s", DEFAULT_TIMEOUT.as_secs()),
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
            return RunResult::Error {
                details: "pip-audit output unparseable".into(),
                stderr_excerpt: excerpt(&stderr),
            };
        }
    };
    let mut findings: Vec<ChecklistFinding> = Vec::new();
    if let Some(deps) = root.get("dependencies").and_then(|v| v.as_array()) {
        for dep in deps {
            if let Some(vulns) = dep.get("vulns").and_then(|v| v.as_array()) {
                let name = dep
                    .get("name")
                    .and_then(|x| x.as_str())
                    .unwrap_or("<unknown>")
                    .to_owned();
                for v in vulns {
                    let id = v
                        .get("id")
                        .and_then(|x| x.as_str())
                        .unwrap_or("CVE")
                        .to_owned();
                    findings.push(ChecklistFinding {
                        severity: "block".into(),
                        category: "pip_audit".into(),
                        message: format!("{}: {}", name, id),
                        file: Some("requirements.txt".into()),
                        line: None,
                    });
                }
            }
        }
    }
    RunResult::Ok { findings }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_target_returns_error() {
        let bogus = Path::new("/elai_e12_bogus_pipaudit_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
