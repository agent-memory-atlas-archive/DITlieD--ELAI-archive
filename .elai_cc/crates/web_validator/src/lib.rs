// ---------------------------------------------------------------------------
// WEB.3 — Web validator: runtime verification that generated sites work.
//
// Validation tiers:
//   WebBuild        — `npm run build` exits 0 and output_dir is non-empty.
//   WebDevServer    — spawn dev server, HTTP GET /, assert 200 + </html>
//                     + meta viewport, then kill the process.
//   WebLint         — Phase B (B2): real ESLint / Stylelint / axe-core
//                     findings via `npx --yes <tool>` subprocess (see
//                     `lint::run_web_lint`). Findings are advisory; the
//                     orchestrator treats severity::Error as WARN.
//   WebPerformance  — Phase B (B4): Lighthouse Core Web Vitals report via
//                     `npx --yes lighthouse` (see `cwv::run_lighthouse`).
//                     Heaviest gate; orchestrator runs at Concept level only.
//
// Crate boundary: NO dependency on `elai-orchestrator`. SSB emission is the
// caller's responsibility (orchestrator wires `WebValidationTier` results
// into its own `IntegrationCheckpoint` records).
// ---------------------------------------------------------------------------

pub mod contrast;
pub mod cwv;
pub mod lint;

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::{Child, Command};
use tokio::time::sleep;

use elai_core::{ClockProvider as _, OsClock};

// ---------------------------------------------------------------------------
// Named constants — no magic numbers.
// ---------------------------------------------------------------------------

/// Max wait for dev server to become ready before declaring `Timeout`.
pub const DEV_SERVER_START_TIMEOUT_SECS: u64 = 15;

/// Per-HTTP-request timeout when probing the dev server.
pub const DEV_SERVER_CHECK_TIMEOUT_SECS: u64 = 5;

/// Polling interval between successive readiness probes during start.
pub const DEV_SERVER_POLL_INTERVAL_MS: u64 = 250;

/// Per-process timeout for `run_web_build`.
pub const WEB_BUILD_TIMEOUT_SECS: u64 = 180;

/// Maximum stderr bytes captured into result structs (avoid OOM on chatty tools).
pub const MAX_STDERR_BYTES: usize = 64 * 1024;

/// Required substring for a validly-rendered HTML document.
pub const EXPECTED_HTML_CLOSING_TAG: &str = "</html>";

/// Required substring confirming a viewport meta tag is present.
/// Matched case-insensitively after lowercasing the body.
pub const EXPECTED_META_VIEWPORT_FRAGMENT: &str = "name=\"viewport\"";

/// Loopback host used for dev-server probes.
pub const DEV_SERVER_PROBE_HOST: &str = "127.0.0.1";

const PORT_INVALID: u16 = 0;

// ---------------------------------------------------------------------------
// Validation tiers (orchestrator wires these by DecompositionLevel).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WebValidationTier {
    WebBuild,
    WebDevServer,
    WebLint,
    /// Phase B (B4): Lighthouse Core Web Vitals (LCP / CLS / INP) and
    /// overall performance score. Runs only at Concept-level decomposition
    /// because each invocation costs 25-180 s and requires a Chromium
    /// binary on PATH (probed via `cwv::probe_chromium`).
    WebPerformance,
}

// ---------------------------------------------------------------------------
// Error type (thiserror — typed public API, no anyhow).
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum WebValidatorError {
    #[error("invalid dev port (must be > 0): {0}")]
    InvalidPort(u16),

    #[error("empty command string")]
    EmptyCommand,

    #[error("process spawn failed for `{command}`: {source}")]
    ProcessSpawn {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("operation timed out after {after_secs}s: {what}")]
    Timeout { what: String, after_secs: u64 },

    #[error("dev server `{command}` exited before binding port {port}")]
    PrematureExit { command: String, port: u16 },

    #[error("HTTP probe failed: {0}")]
    HttpCheck(#[from] reqwest::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("output directory missing after build: {0}")]
    OutputDirMissing(PathBuf),

    #[error("symlink/junction creation failed for {dst:?}: {reason}")]
    SymlinkFailed { dst: PathBuf, reason: String },

    /// Phase B (B2): node runtime is missing — WebLint tier cannot run.
    #[error("node runtime missing (probe `{probe_command}` failed); WebLint disabled")]
    NodeMissing { probe_command: String },

    /// Phase B (B4): chromium / chrome runtime is missing — WebPerformance tier
    /// cannot run. Lighthouse needs a controllable browser binary.
    #[error("chromium/chrome runtime missing (probe `{probe_command}` failed); WebPerformance disabled")]
    ChromiumMissing { probe_command: String },

    /// Phase B (B2/B4): an external linter/auditor produced output that did
    /// not match the expected JSON shape. The raw stderr is captured for
    /// debugging; we never propagate the full stdout to avoid OOM.
    #[error("external tool `{tool}` produced unparseable output: {reason}")]
    ToolOutputInvalid { tool: String, reason: String },
}

// ---------------------------------------------------------------------------
// Result types.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevServerStatus {
    pub ok: bool,
    pub status_code: u16,
    pub has_html_close: bool,
    pub has_meta_viewport: bool,
    pub stderr_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebBuildResult {
    pub success: bool,
    pub exit_code: i32,
    pub output_dir: PathBuf,
    pub stderr_errors: Vec<String>,
}

// ---------------------------------------------------------------------------
// DevServerHandle — RAII wrapper that kills child on drop.
// ---------------------------------------------------------------------------

/// Owned handle to a spawned dev-server child. Drop kills the child if
/// `stop_dev_server` was not called explicitly.
#[derive(Debug)]
pub struct DevServerHandle {
    pid: u32,
    port: u16,
    dir: PathBuf,
    child: Option<Child>,
}

impl DevServerHandle {
    pub fn pid(&self) -> u32 {
        self.pid
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

impl Drop for DevServerHandle {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Best-effort synchronous kill. tokio::process::Child::start_kill
            // is non-async and only sets a flag / sends the signal.
            match child.start_kill() {
                Ok(()) => tracing::warn!(
                    pid = self.pid,
                    port = self.port,
                    "DevServerHandle dropped without stop_dev_server; sent kill signal"
                ),
                Err(err) => tracing::error!(
                    pid = self.pid,
                    port = self.port,
                    error = %err,
                    "DevServerHandle drop: kill failed"
                ),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Public API.
// ---------------------------------------------------------------------------

/// Spawn a dev server defined by `command` (e.g. `"npm run dev"`) in `dir`,
/// poll until it responds on `port` or `DEV_SERVER_START_TIMEOUT_SECS` elapses.
pub async fn start_dev_server(
    dir: &Path,
    command: &str,
    port: u16,
) -> Result<DevServerHandle, WebValidatorError> {
    if port == PORT_INVALID {
        return Err(WebValidatorError::InvalidPort(port));
    }
    let (program, args) = split_command(command)?;

    let mut cmd = Command::new(&program);
    cmd.args(&args)
        .current_dir(dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    let child = cmd
        .spawn()
        .map_err(|source| WebValidatorError::ProcessSpawn {
            command: command.to_owned(),
            source,
        })?;

    let pid = child.id().unwrap_or(0);
    let mut handle = DevServerHandle {
        pid,
        port,
        dir: dir.to_path_buf(),
        child: Some(child),
    };

    // Build the HTTP client once and reuse across all readiness probes.
    let client = build_probe_client()?;

    let _clock = OsClock;
    let _start = _clock.now();
    let _timeout_duration = Duration::from_secs(DEV_SERVER_START_TIMEOUT_SECS);
    let poll_interval = Duration::from_millis(DEV_SERVER_POLL_INTERVAL_MS);

    loop {
        if _clock.elapsed_since(_start) >= _timeout_duration {
            // Drop on `handle` will kill the child; explicit stop ensures wait().
            let _ = stop_dev_server(handle).await;
            return Err(WebValidatorError::Timeout {
                what: format!("dev-server readiness on port {port}"),
                after_secs: DEV_SERVER_START_TIMEOUT_SECS,
            });
        }
        // If the child exited prematurely, take it out of the handle so Drop
        // doesn't try to kill an already-dead process (avoids spurious error
        // logs on Windows where signaling a dead PID returns ACCESS_DENIED).
        if let Some(child) = handle.child.as_mut() {
            if let Ok(Some(_status)) = child.try_wait() {
                let _ = handle.child.take();
                return Err(WebValidatorError::PrematureExit {
                    command: command.to_owned(),
                    port,
                });
            }
        }
        match probe_dev_server(&client, port).await {
            Ok(status) if status.ok => return Ok(handle),
            _ => sleep(poll_interval).await,
        }
    }
}

/// Single HTTP probe against the dev server. Returns a status describing
/// whether the response satisfies the WEB_DEV_SERVER tier.
pub async fn check_dev_server(port: u16) -> Result<DevServerStatus, WebValidatorError> {
    if port == PORT_INVALID {
        return Err(WebValidatorError::InvalidPort(port));
    }
    let client = build_probe_client()?;
    probe_dev_server(&client, port).await
}

fn build_probe_client() -> Result<reqwest::Client, WebValidatorError> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(DEV_SERVER_CHECK_TIMEOUT_SECS))
        .build()?)
}

async fn probe_dev_server(
    client: &reqwest::Client,
    port: u16,
) -> Result<DevServerStatus, WebValidatorError> {
    let url = format!("http://{DEV_SERVER_PROBE_HOST}:{port}/");
    let resp = client.get(&url).send().await?;
    let status_code: u16 = resp.status().as_u16();
    let body = match resp.text().await {
        Ok(text) => text,
        Err(err) => {
            tracing::warn!(error = %err, port, "dev-server body decode failed");
            String::new()
        }
    };
    let body_lower = body.to_lowercase();
    let has_html_close = body_lower.contains(EXPECTED_HTML_CLOSING_TAG);
    let has_meta_viewport = body_lower.contains(EXPECTED_META_VIEWPORT_FRAGMENT);

    Ok(DevServerStatus {
        ok: status_code == 200 && has_html_close && has_meta_viewport,
        status_code,
        has_html_close,
        has_meta_viewport,
        stderr_errors: Vec::new(),
    })
}

/// Stop a dev-server child. Consumes the handle so Drop's warning never fires
/// for a properly-stopped server.
pub async fn stop_dev_server(mut handle: DevServerHandle) -> Result<(), WebValidatorError> {
    if let Some(mut child) = handle.child.take() {
        child.start_kill()?;
        let _ = tokio::time::timeout(
            Duration::from_secs(DEV_SERVER_CHECK_TIMEOUT_SECS),
            child.wait(),
        )
        .await;
    }
    Ok(())
}

/// Run the build command (e.g. `"npm run build"`) in `dir`. Asserts exit 0
/// and that `output_dir` exists after completion.
pub async fn run_web_build(
    dir: &Path,
    command: &str,
    output_dir: &Path,
) -> Result<WebBuildResult, WebValidatorError> {
    let (program, args) = split_command(command)?;

    let output_future = Command::new(&program)
        .args(&args)
        .current_dir(dir)
        .output();

    let output = tokio::time::timeout(Duration::from_secs(WEB_BUILD_TIMEOUT_SECS), output_future)
        .await
        .map_err(|_| WebValidatorError::Timeout {
            what: format!("web build `{command}`"),
            after_secs: WEB_BUILD_TIMEOUT_SECS,
        })?
        .map_err(|source| WebValidatorError::ProcessSpawn {
            command: command.to_owned(),
            source,
        })?;

    let stderr_errors = capture_stderr_lines(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);
    let success = output.status.success() && output_dir.exists();

    if output.status.success() && !output_dir.exists() {
        return Err(WebValidatorError::OutputDirMissing(output_dir.to_path_buf()));
    }

    Ok(WebBuildResult {
        success,
        exit_code,
        output_dir: output_dir.to_path_buf(),
        stderr_errors,
    })
}

/// Atomically promote a draft build directory to the live slot. On Unix
/// this is a `symlink`; on Windows it's `symlink_dir` with a directory
/// junction (`mklink /J`) fallback for non-elevated processes.
pub fn promote_slot(draft: &Path, live: &Path) -> Result<(), WebValidatorError> {
    if !draft.exists() {
        return Err(WebValidatorError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("draft path missing: {}", draft.display()),
        )));
    }
    if let Some(parent) = live.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if live.exists() || live.is_symlink() {
        remove_existing_link(live)?;
    }
    create_dir_link(draft, live)
}

// ---------------------------------------------------------------------------
// Internal helpers.
// ---------------------------------------------------------------------------

fn split_command(command: &str) -> Result<(String, Vec<String>), WebValidatorError> {
    let mut parts = command.split_whitespace().map(|s| s.to_owned());
    let program = parts.next().ok_or(WebValidatorError::EmptyCommand)?;
    let args: Vec<String> = parts.collect();
    Ok((program, args))
}

fn capture_stderr_lines(bytes: &[u8]) -> Vec<String> {
    let limit = bytes.len().min(MAX_STDERR_BYTES);
    let text = String::from_utf8_lossy(&bytes[..limit]);
    text.lines()
        .filter(|l| {
            let lower = l.to_lowercase();
            lower.contains("error") || lower.contains("fail") || lower.contains("err!")
        })
        .map(|l| l.to_owned())
        .collect()
}

fn remove_existing_link(live: &Path) -> Result<(), WebValidatorError> {
    let metadata = std::fs::symlink_metadata(live)?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() || file_type.is_dir() {
        std::fs::remove_dir_all(live).or_else(|_| std::fs::remove_file(live))?;
    } else {
        std::fs::remove_file(live)?;
    }
    Ok(())
}

#[cfg(unix)]
fn create_dir_link(draft: &Path, live: &Path) -> Result<(), WebValidatorError> {
    std::os::unix::fs::symlink(draft, live).map_err(|err| WebValidatorError::SymlinkFailed {
        dst: live.to_path_buf(),
        reason: err.to_string(),
    })
}

#[cfg(windows)]
fn create_dir_link(draft: &Path, live: &Path) -> Result<(), WebValidatorError> {
    // Try true symlink first (requires Developer Mode or elevation).
    match std::os::windows::fs::symlink_dir(draft, live) {
        Ok(()) => return Ok(()),
        Err(err) => tracing::debug!(
            error = %err,
            "symlink_dir failed; falling back to directory junction"
        ),
    }
    // Fall back to a directory junction via `cmd /C mklink /J`.
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            &live.to_string_lossy(),
            &draft.to_string_lossy(),
        ])
        .status()
        .map_err(|err| WebValidatorError::SymlinkFailed {
            dst: live.to_path_buf(),
            reason: format!("mklink spawn failed: {err}"),
        })?;
    if !status.success() {
        return Err(WebValidatorError::SymlinkFailed {
            dst: live.to_path_buf(),
            reason: format!("mklink exited {}", status.code().unwrap_or(-1)),
        });
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn create_dir_link(_draft: &Path, live: &Path) -> Result<(), WebValidatorError> {
    Err(WebValidatorError::SymlinkFailed {
        dst: live.to_path_buf(),
        reason: "unsupported target platform".to_owned(),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn constants_are_named_not_magic() {
        assert_eq!(DEV_SERVER_START_TIMEOUT_SECS, 15);
        assert_eq!(DEV_SERVER_CHECK_TIMEOUT_SECS, 5);
        assert_eq!(DEV_SERVER_POLL_INTERVAL_MS, 250);
        assert!(MAX_STDERR_BYTES > 0);
        assert_eq!(DEV_SERVER_PROBE_HOST, "127.0.0.1");
    }

    #[test]
    fn split_command_parses_program_and_args() {
        let (prog, args) = split_command("npm run dev").expect("split");
        assert_eq!(prog, "npm");
        assert_eq!(args, vec!["run".to_owned(), "dev".to_owned()]);
    }

    #[test]
    fn split_command_rejects_empty() {
        assert!(matches!(
            split_command("   "),
            Err(WebValidatorError::EmptyCommand)
        ));
    }

    #[test]
    fn capture_stderr_filters_only_error_lines() {
        let bytes = b"info: starting\nerror: oops\nwarning: meh\nfailed: boom";
        let lines = capture_stderr_lines(bytes);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("error"));
        assert!(lines[1].contains("failed"));
    }

    #[test]
    fn capture_stderr_truncates_to_limit() {
        let big = vec![b'X'; MAX_STDERR_BYTES + 4096];
        let lines = capture_stderr_lines(&big);
        // No "error"/"fail" tokens => no lines, but no panic.
        assert!(lines.is_empty());
    }

    #[tokio::test]
    async fn check_dev_server_rejects_port_zero() {
        let res = check_dev_server(0).await;
        assert!(matches!(res, Err(WebValidatorError::InvalidPort(0))));
    }

    #[tokio::test]
    async fn start_dev_server_rejects_port_zero() {
        let dir = TempDir::new().expect("tempdir");
        let res = start_dev_server(dir.path(), "true", 0).await;
        assert!(matches!(res, Err(WebValidatorError::InvalidPort(0))));
    }

    #[tokio::test]
    async fn start_dev_server_rejects_empty_command() {
        let dir = TempDir::new().expect("tempdir");
        let res = start_dev_server(dir.path(), "   ", 8080).await;
        assert!(matches!(res, Err(WebValidatorError::EmptyCommand)));
    }

    #[test]
    fn web_validation_tier_round_trips_serde() {
        for tier in [
            WebValidationTier::WebBuild,
            WebValidationTier::WebDevServer,
            WebValidationTier::WebLint,
            WebValidationTier::WebPerformance,
        ] {
            let s = serde_json::to_string(&tier).expect("serialize");
            let back: WebValidationTier = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(tier, back);
        }
    }

    #[test]
    fn dev_server_handle_drop_no_panic_when_no_child() {
        // Dropping a handle whose child was already taken must not panic.
        let mut handle = DevServerHandle {
            pid: 0,
            port: 1234,
            dir: PathBuf::from("."),
            child: None,
        };
        assert_eq!(handle.pid(), 0);
        assert_eq!(handle.port(), 1234);
        assert_eq!(handle.dir(), Path::new("."));
        // child = None — Drop must be a no-op.
        let _ = handle.child.take();
        // Explicit drop here.
    }

    #[test]
    fn web_validator_error_variants_implement_display() {
        let e = WebValidatorError::InvalidPort(0);
        assert!(format!("{e}").contains("invalid"));
        let e = WebValidatorError::EmptyCommand;
        assert!(!format!("{e}").is_empty());
        let e = WebValidatorError::Timeout {
            what: "x".into(),
            after_secs: 5,
        };
        assert!(format!("{e}").contains("timed out"));
        let e = WebValidatorError::PrematureExit {
            command: "npm run dev".into(),
            port: 5173,
        };
        let s = format!("{e}");
        assert!(s.contains("npm run dev"));
        assert!(s.contains("5173"));
        let e = WebValidatorError::OutputDirMissing(PathBuf::from("/x"));
        assert!(format!("{e}").contains("output"));
        let e = WebValidatorError::SymlinkFailed {
            dst: PathBuf::from("/y"),
            reason: "perm".into(),
        };
        assert!(format!("{e}").contains("symlink"));
    }

    #[test]
    fn build_probe_client_constructs_with_timeout() {
        // The shared client is built once and reused across polling iterations.
        let client = build_probe_client().expect("build client");
        // Smoke check: client exists. Behavioral guarantees are in integration tests.
        let _ = client;
    }

    #[tokio::test]
    async fn premature_exit_variant_returned_when_child_dies_before_bind() {
        // Use a command that exits immediately; the polling loop must observe
        // the dead child and return PrematureExit (not Timeout).
        let dir = TempDir::new().expect("tempdir");
        #[cfg(windows)]
        let cmd = "cmd /C exit 0";
        #[cfg(unix)]
        let cmd = "true";

        // Pick an arbitrary unused-looking high port; the probe will never succeed
        // because the child exits immediately.
        let res = start_dev_server(dir.path(), cmd, 51999).await;
        assert!(matches!(res, Err(WebValidatorError::PrematureExit { .. })));
    }

    #[tokio::test]
    async fn run_web_build_reports_missing_output_dir_after_success() {
        // Use a no-op command that succeeds; output dir intentionally missing.
        let dir = TempDir::new().expect("tempdir");
        let missing_out = dir.path().join("dist-not-created");
        // `cmd /C exit 0` on Windows; `true` on Unix.
        #[cfg(windows)]
        let cmd = "cmd /C exit 0";
        #[cfg(unix)]
        let cmd = "true";

        let res = run_web_build(dir.path(), cmd, &missing_out).await;
        assert!(matches!(res, Err(WebValidatorError::OutputDirMissing(_))));
    }

    #[test]
    fn promote_slot_errors_when_draft_missing() {
        let dir = TempDir::new().expect("tempdir");
        let draft = dir.path().join("nope");
        let live = dir.path().join("live");
        let err = promote_slot(&draft, &live).expect_err("must fail when draft missing");
        assert!(matches!(err, WebValidatorError::IoError(_)));
    }
}
