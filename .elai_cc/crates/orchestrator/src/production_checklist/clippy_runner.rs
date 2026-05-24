//! E-12 Tier-B — clippy --deny warnings runner.
//!
//! Shells out to `cargo clippy --message-format=json --
//!   --deny warnings` from the target directory. The `--deny warnings`
//! flag promotes every clippy warning to an error so the production
//! gate fires.
//!
//! TOOLCHAIN-PENDING-CLIPPY: ships with rustup (rustup component add
//!   clippy). https://doc.rust-lang.org/clippy/installation.html.

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, HEAVY_TIMEOUT};

const BINARY: &str = "cargo";
const INSTALL_HINT: &str = "install clippy via rustup component add clippy";

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }
    let mut cmd = Command::new(BINARY);
    cmd.arg("clippy")
        .arg("--message-format=json")
        .arg("--quiet")
        .arg("--")
        .arg("--deny")
        .arg("warnings")
        .current_dir(target)
        .kill_on_drop(true)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let output = match timeout(HEAVY_TIMEOUT, cmd.output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) if is_path_not_found(&e) => {
            return RunResult::ToolchainMissing {
                tool: "clippy".to_owned(),
                install_hint: INSTALL_HINT.to_owned(),
            };
        }
        Ok(Err(e)) => {
            return RunResult::Error {
                details: format!("clippy spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!("clippy timed out after {}s", HEAVY_TIMEOUT.as_secs()),
                stderr_excerpt: String::new(),
            };
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    // Parse NDJSON output from rustc/clippy. Each line is a
    // compiler-message object with a level field ("warning",
    // "error", etc).
    let mut findings: Vec<ChecklistFinding> = Vec::new();
    for line in stdout.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(l);
        let Ok(v) = parsed else {
            continue;
        };
        let Some(msg) = v.get("message") else {
            continue;
        };
        let level = msg
            .get("level")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_owned();
        if !matches!(level.as_str(), "warning" | "error") {
            continue;
        }
        let text = msg
            .get("message")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_owned();
        let span = msg.get("spans").and_then(|x| x.as_array());
        let (file, line_no) = match span {
            Some(arr) if !arr.is_empty() => {
                let s = &arr[0];
                let file = s
                    .get("file_name")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_owned());
                let line = s
                    .get("line_start")
                    .and_then(|x| x.as_u64())
                    .and_then(|n| u32::try_from(n).ok());
                (file, line)
            }
            _ => (None, None),
        };
        findings.push(ChecklistFinding {
            severity: if level == "error" { "block".into() } else { "warn".into() },
            category: "clippy".into(),
            message: text,
            file,
            line: line_no,
        });
    }
    // If the build succeeded but stderr had a Cargo.toml error,
    // surface that — the wire-dark posture insists on visibility.
    if findings.is_empty() && !output.status.success() {
        return RunResult::Error {
            details: format!(
                "clippy exited with code {}",
                output.status.code().unwrap_or(-1)
            ),
            stderr_excerpt: excerpt(&stderr),
        };
    }
    RunResult::Ok { findings }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_target_returns_error() {
        let bogus = Path::new("/elai_e12_bogus_clippy_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
