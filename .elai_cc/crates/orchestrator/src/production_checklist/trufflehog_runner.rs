//! E-12 Tier-B — trufflehog secrets-scan runner (second-source).
//!
//! Companion to `gitleaks_runner` — runs trufflehog as a
//! cross-engine verifier. Findings overlap is expected; the
//! deterministic-merge logic lives in the auditor consumer, not
//! here.
//!
//! TOOLCHAIN-PENDING-TRUFFLEHOG: pip install trufflehog ||
//!   brew install trufflesecurity/trufflehog/trufflehog ||
//!   `docker run trufflesecurity/trufflehog` (the runpod host
//!   installer resolves the appropriate package manager).

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, DEFAULT_TIMEOUT};

const BINARY: &str = "trufflehog";
const INSTALL_HINT: &str =
    "install trufflehog via pip / brew / docker (https://github.com/trufflesecurity/trufflehog)";

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }
    let mut cmd = Command::new(BINARY);
    cmd.arg("filesystem")
        .arg(target)
        .arg("--no-update")
        .arg("--json")
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
                details: format!("trufflehog spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!("trufflehog timed out after {}s", DEFAULT_TIMEOUT.as_secs()),
                stderr_excerpt: String::new(),
            };
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() && stdout.trim().is_empty() {
        return RunResult::Error {
            details: format!(
                "trufflehog exited with code {}",
                output.status.code().unwrap_or(-1)
            ),
            stderr_excerpt: excerpt(&stderr),
        };
    }
    // trufflehog --json emits NDJSON (one object per line). Count
    // unique findings; do not parse the deep structure here (the
    // auditor consumer can re-parse if needed).
    let mut findings: Vec<ChecklistFinding> = Vec::new();
    for line in stdout.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(l);
        match parsed {
            Ok(v) => {
                let detector = v
                    .get("DetectorName")
                    .and_then(|x| x.as_str())
                    .unwrap_or("trufflehog finding")
                    .to_owned();
                let file = v
                    .get("SourceMetadata")
                    .and_then(|m| m.get("Data"))
                    .and_then(|d| d.get("Filesystem"))
                    .and_then(|f| f.get("file"))
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_owned());
                findings.push(ChecklistFinding {
                    severity: "block".into(),
                    category: "trufflehog_scan".into(),
                    message: detector,
                    file,
                    line: None,
                });
            }
            Err(_) => {
                // Skip unparseable lines silently (trufflehog
                // intermixes status output on some versions).
                continue;
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
        let bogus = Path::new("/elai_e12_bogus_trufflehog_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
