//! E-12 Tier-B — gitleaks secrets-scan runner.
//!
//! Shells out to `gitleaks detect --no-git --source <target> --report-format json`
//! and folds matches into a list of [`ChecklistFinding`]s. The
//! gitleaks binary is NOT installed by default; the runner returns
//! [`RunResult::ToolchainMissing`] on PATH miss instead of a panic.
//!
//! TOOLCHAIN-PENDING-GITLEAKS: brew install gitleaks ||
//!   scoop install gitleaks || cargo install --git
//!   https://github.com/gitleaks/gitleaks (the runpod host installer
//!   resolves to the appropriate package manager).

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, DEFAULT_TIMEOUT};

/// Resolved binary name. Always re-resolved against PATH at
/// invocation; no long-lived cache.
const BINARY: &str = "gitleaks";

/// Canonical install hint surfaced via `ToolchainMissing`.
const INSTALL_HINT: &str = "install gitleaks via brew / scoop / cargo install (https://github.com/gitleaks/gitleaks)";

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }

    let mut cmd = Command::new(BINARY);
    cmd.arg("detect")
        .arg("--no-git")
        .arg("--source")
        .arg(target)
        .arg("--report-format")
        .arg("json")
        .kill_on_drop(true);
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

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
                details: format!("gitleaks spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!(
                    "gitleaks timed out after {}s",
                    DEFAULT_TIMEOUT.as_secs()
                ),
                stderr_excerpt: String::new(),
            };
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    // gitleaks exits with code 0 when no leaks found, non-zero when
    // leaks found OR on error. Distinguish via stdout JSON.
    if !output.status.success() && stdout.trim().is_empty() {
        return RunResult::Error {
            details: format!(
                "gitleaks exited with code {} and empty stdout",
                output.status.code().unwrap_or(-1)
            ),
            stderr_excerpt: excerpt(&stderr),
        };
    }

    let findings = parse_findings(&stdout);
    RunResult::Ok { findings }
}

fn parse_findings(stdout: &str) -> Vec<ChecklistFinding> {
    // gitleaks JSON output is a top-level array of finding objects:
    //   [{"Description": "...", "File": "...", "StartLine": N, ...}]
    // When stdout is empty, gitleaks reports zero findings.
    let trimmed = stdout.trim();
    if trimmed.is_empty() || trimmed == "[]" || trimmed == "null" {
        return Vec::new();
    }
    let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(trimmed);
    let arr = match parsed {
        Ok(v) => v,
        // Malformed JSON: surface a single info-level finding so the
        // auditor sees the failure rather than silently dropping.
        Err(_) => {
            return vec![ChecklistFinding {
                severity: "warn".into(),
                category: "secrets_scan/parse_error".into(),
                message: "gitleaks output unparseable".into(),
                file: None,
                line: None,
            }];
        }
    };
    arr.into_iter()
        .map(|v| {
            let desc = v
                .get("Description")
                .and_then(|x| x.as_str())
                .unwrap_or("gitleaks finding")
                .to_owned();
            let file = v
                .get("File")
                .and_then(|x| x.as_str())
                .map(|s| s.to_owned());
            let line = v
                .get("StartLine")
                .and_then(|x| x.as_u64())
                .and_then(|n| u32::try_from(n).ok());
            let mut msg = desc;
            if msg.len() > 512 {
                msg.truncate(512);
            }
            ChecklistFinding {
                severity: "block".into(),
                category: "secrets_scan".into(),
                message: msg,
                file,
                line,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_target_returns_error() {
        let bogus = Path::new("/elai_e12_bogus_target_xyzzy_gitleaks");
        let res = run(bogus).await;
        match res {
            RunResult::Error { details, .. } => {
                assert!(details.contains("does not exist"));
            }
            other => panic!("expected Error, got {:?}", other),
        }
    }

    #[test]
    fn parse_findings_handles_empty_array() {
        assert!(parse_findings("[]").is_empty());
    }

    #[test]
    fn parse_findings_handles_empty_stdout() {
        assert!(parse_findings("").is_empty());
    }

    #[test]
    fn parse_findings_extracts_one_record() {
        let json = r#"[{"Description":"AWS Access Key","File":"src/app.ts","StartLine":42}]"#;
        let out = parse_findings(json);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].category, "secrets_scan");
        assert_eq!(out[0].severity, "block");
        assert_eq!(out[0].file.as_deref(), Some("src/app.ts"));
        assert_eq!(out[0].line, Some(42));
    }

    #[test]
    fn parse_findings_warns_on_malformed_json() {
        let out = parse_findings("not json {");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].severity, "warn");
        assert_eq!(out[0].category, "secrets_scan/parse_error");
    }
}
