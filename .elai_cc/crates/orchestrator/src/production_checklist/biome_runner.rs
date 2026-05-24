//! E-12 Tier-B — biome linter runner.
//!
//! Shells out to `biome check --reporter=json <target>`. Biome
//! consolidates ESLint + Prettier + (subset of) tsc — emits a single
//! linter+formatter finding stream.
//!
//! TOOLCHAIN-PENDING-BIOME: npm install -D @biomejs/biome ||
//!   pnpm add -D @biomejs/biome (https://biomejs.dev/).

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, DEFAULT_TIMEOUT};

const BINARY: &str = "biome";
const INSTALL_HINT: &str =
    "install biome via npm install -D @biomejs/biome || pnpm add -D @biomejs/biome";

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }
    let mut cmd = Command::new(BINARY);
    cmd.arg("check")
        .arg("--reporter=json")
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
                details: format!("biome spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!("biome timed out after {}s", DEFAULT_TIMEOUT.as_secs()),
                stderr_excerpt: String::new(),
            };
        }
    };
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    // biome exit 0 = clean; non-zero = warnings or errors. Surface
    // the exit code as a single finding rather than parsing the
    // JSON output (the JSON shape varies across biome major
    // versions; the auditor consumer can re-parse if needed).
    if !output.status.success() {
        return RunResult::Ok {
            findings: vec![ChecklistFinding {
                severity: "warn".into(),
                category: "biome".into(),
                message: format!(
                    "biome check reported issues (exit={})",
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
        let bogus = Path::new("/elai_e12_bogus_biome_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
