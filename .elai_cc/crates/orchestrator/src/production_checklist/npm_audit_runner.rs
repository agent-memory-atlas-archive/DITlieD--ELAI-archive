//! E-12 Tier-B — npm audit runner.
//!
//! Shells out to `npm audit --json` from the target directory.
//!
//! TOOLCHAIN-PENDING-NPM: install Node.js via nvm / fnm / corepack —
//!   npm ships bundled with Node. https://nodejs.org/en/download/.

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, DEFAULT_TIMEOUT};

const BINARY: &str = "npm";
const INSTALL_HINT: &str = "install Node.js (https://nodejs.org/) — npm ships bundled";

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
                tool: BINARY.to_owned(),
                install_hint: INSTALL_HINT.to_owned(),
            };
        }
        Ok(Err(e)) => {
            return RunResult::Error {
                details: format!("npm audit spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!("npm audit timed out after {}s", DEFAULT_TIMEOUT.as_secs()),
                stderr_excerpt: String::new(),
            };
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    // npm audit exit 1 = vulnerabilities found; 0 = clean.
    // Parse JSON output: { "vulnerabilities": { "<name>": { "severity": "...", ... } } }
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    let root = match parsed {
        Ok(v) => v,
        Err(_) => {
            // Some npm configs (no package.json) emit non-JSON
            // errors on stdout. Surface as parse_error finding.
            return RunResult::Error {
                details: "npm audit output unparseable".into(),
                stderr_excerpt: excerpt(&stderr),
            };
        }
    };
    let mut findings: Vec<ChecklistFinding> = Vec::new();
    if let Some(vulns) = root.get("vulnerabilities").and_then(|v| v.as_object()) {
        for (name, v) in vulns {
            let severity = v
                .get("severity")
                .and_then(|x| x.as_str())
                .unwrap_or("warn")
                .to_owned();
            let sev_label = match severity.as_str() {
                "critical" | "high" => "block",
                "moderate" => "warn",
                _ => "info",
            }
            .to_owned();
            findings.push(ChecklistFinding {
                severity: sev_label,
                category: "npm_audit".into(),
                message: format!("vulnerable package: {} (severity={})", name, severity),
                file: Some("package.json".into()),
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
        let bogus = Path::new("/elai_e12_bogus_npm_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
