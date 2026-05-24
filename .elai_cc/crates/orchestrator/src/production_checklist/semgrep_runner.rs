//! E-12 Tier-B — semgrep OWASP static analysis runner.
//!
//! Shells out to `semgrep --config=p/owasp-top-ten --json <target>`.
//! Heavy runner (60s timeout). The OWASP rule pack is upstream-
//! maintained; the version pin lives in `elai_cc.toml`
//! `[secrets_scan]` table (NOT inline here per Tier-A guard 4).
//!
//! TOOLCHAIN-PENDING-SEMGREP: pip install semgrep ||
//!   brew install semgrep (https://semgrep.dev/docs/getting-started).

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, HEAVY_TIMEOUT};

const BINARY: &str = "semgrep";
const INSTALL_HINT: &str = "install semgrep via pip install semgrep || brew install semgrep";

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }
    let mut cmd = Command::new(BINARY);
    cmd.arg("--config=p/owasp-top-ten")
        .arg("--json")
        .arg("--quiet")
        .arg(target)
        .kill_on_drop(true)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let output = match timeout(HEAVY_TIMEOUT, cmd.output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) if is_path_not_found(&e) => {
            return RunResult::ToolchainMissing {
                tool: BINARY.to_owned(),
                install_hint: INSTALL_HINT.to_owned(),
            };
        }
        Ok(Err(e)) => {
            return RunResult::Error {
                details: format!("semgrep spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!("semgrep timed out after {}s", HEAVY_TIMEOUT.as_secs()),
                stderr_excerpt: String::new(),
            };
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    // semgrep exit 0 = no findings; 1 = findings present; 2 = error.
    let code = output.status.code().unwrap_or(-1);
    if code == 2 {
        return RunResult::Error {
            details: format!("semgrep exited with code {}", code),
            stderr_excerpt: excerpt(&stderr),
        };
    }
    // Parse JSON output: { "results": [ { "check_id": "...", "path": "...", "start": {"line": N}, "extra": {"message": "..."} } ] }
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    let mut findings: Vec<ChecklistFinding> = Vec::new();
    if let Ok(root) = parsed {
        if let Some(arr) = root.get("results").and_then(|v| v.as_array()) {
            for r in arr {
                let check_id = r
                    .get("check_id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("owasp")
                    .to_owned();
                let message = r
                    .get("extra")
                    .and_then(|e| e.get("message"))
                    .and_then(|x| x.as_str())
                    .unwrap_or(&check_id)
                    .to_owned();
                let file = r
                    .get("path")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_owned());
                let line = r
                    .get("start")
                    .and_then(|s| s.get("line"))
                    .and_then(|x| x.as_u64())
                    .and_then(|n| u32::try_from(n).ok());
                let mut msg = format!("{}: {}", check_id, message);
                if msg.len() > 512 {
                    msg.truncate(512);
                }
                findings.push(ChecklistFinding {
                    severity: "block".into(),
                    category: "owasp_static".into(),
                    message: msg,
                    file,
                    line,
                });
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
        let bogus = Path::new("/elai_e12_bogus_semgrep_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
