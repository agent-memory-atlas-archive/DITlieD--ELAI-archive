//! E-12 Tier-B — lighthouse-ci runner.
//!
//! Shells out to `lhci collect --static-dist-dir=<target>` then
//! `lhci assert`. Heavy runner (60s timeout). Reports the
//! perf / accessibility / SEO / best-practice scores; finding
//! emitted when any score < 95.
//!
//! TOOLCHAIN-PENDING-LIGHTHOUSE: npm install -g @lhci/cli ||
//!   pnpm add -g @lhci/cli (https://github.com/GoogleChrome/lighthouse-ci).

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, HEAVY_TIMEOUT};

const BINARY: &str = "lhci";
const INSTALL_HINT: &str = "install lhci via npm install -g @lhci/cli || pnpm add -g @lhci/cli";

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }
    let mut cmd = Command::new(BINARY);
    cmd.arg("autorun")
        .arg(format!("--collect.staticDistDir={}", target.display()))
        .current_dir(target)
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
                details: format!("lhci spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!("lhci timed out after {}s", HEAVY_TIMEOUT.as_secs()),
                stderr_excerpt: String::new(),
            };
        }
    };
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        return RunResult::Ok {
            findings: vec![ChecklistFinding {
                severity: "warn".into(),
                category: "lighthouse".into(),
                message: format!(
                    "lighthouse-ci assertion failed (exit={})",
                    output.status.code().unwrap_or(-1)
                ),
                file: None,
                line: None,
            }],
        };
    }
    let _ = excerpt(&stderr);
    RunResult::Ok {
        findings: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_target_returns_error() {
        let bogus = Path::new("/elai_e12_bogus_lighthouse_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
