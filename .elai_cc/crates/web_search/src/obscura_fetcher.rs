// D4: ObscuraFetcher — headless-browser fetch via obscura CLI subprocess.
//
// Spawns `obscura fetch <URL> --dump html --timeout <secs> --stealth` as a
// subprocess. Stdout is the rendered HTML. Stderr contains status lines (not
// treated as errors unless exit code is non-zero).
//
// Enabled only under cfg(feature = "obscura"). The default build uses WebFetcher
// (reqwest). ELAI_FETCHER=obscura selects this path at runtime.
//
// A5.1: Streaming stdout cap — uses tokio::process::Command so stdout becomes an
// AsyncRead. Bytes are read in a loop into a Vec<u8> with a OBSCURA_MAX_OUTPUT_BYTES
// hard cap. On cap-hit the child process is killed and we return the buffered content
// with a truncation warning. This removes the theoretical OOM from full buffering.
//
// A5.2: check_obscura_binary() / doctor() — init-time binary resolution and version
// check, usable from CLI `elai doctor` (see crates/cli/src/doctor.rs).

use std::path::PathBuf;
use std::time::Instant;

use tokio::io::AsyncReadExt;
use tracing::{debug, warn};

use crate::fetcher::FetchedPage;
use crate::WebSearchError;

// Named constants — no magic numbers.

/// Default timeout passed to obscura --timeout (seconds, not ms).
/// obscura CLI v0.1.2 uses seconds for --timeout.
pub const OBSCURA_TIMEOUT_SECS: u64 = 30;

/// Maximum stdout bytes we stream from the subprocess (5 MB).
/// On cap-hit the child is killed and buffered content is returned with a
/// truncation warning. Partial HTML is still useful for extraction.
pub const OBSCURA_MAX_OUTPUT_BYTES: usize = 5 * 1024 * 1024;

/// Environment variable that overrides the obscura binary path at runtime.
pub const OBSCURA_PATH_ENV: &str = "OBSCURA_PATH";

/// Read chunk size for the streaming loop (64 KiB).
const OBSCURA_READ_CHUNK: usize = 64 * 1024;

/// Compile-time default path to the obscura binary on Windows.
/// The binary ships inside the project at .elai_cc/tools/obscura/obscura.exe.
#[cfg(target_os = "windows")]
const OBSCURA_DEFAULT_PATH: &str =
    r"D:\.ELAI_workflow\.elai_cc\tools\obscura\obscura.exe";

/// Compile-time default on non-Windows: assume it is on PATH.
#[cfg(not(target_os = "windows"))]
const OBSCURA_DEFAULT_PATH: &str = "obscura";

// ---------------------------------------------------------------------------
// ObscuraDoctorReport — returned by doctor()
// ---------------------------------------------------------------------------

/// Result of the obscura binary health check. All fields are optional so
/// callers can render partial information without unwrapping failures.
#[derive(Debug, Clone)]
pub struct ObscuraDoctorReport {
    /// Resolved path to the obscura binary, if found.
    pub binary_path: Option<PathBuf>,
    /// Output of `obscura --version`, if the binary could be run.
    pub version: Option<String>,
    /// Human-readable error description if any step failed.
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// ObscuraFetcher
// ---------------------------------------------------------------------------

/// Fetch pages via the obscura headless-browser CLI subprocess.
///
/// Does NOT perform robots.txt checking or per-domain rate limiting — those are
/// the WebFetcher's responsibilities and are listed as "Prod (out of scope)" in
/// the D4 spec. This struct matches the minimum interface expected by
/// DefaultWebResearchRunner.
pub struct ObscuraFetcher {
    binary_path: PathBuf,
    timeout_secs: u64,
}

impl ObscuraFetcher {
    /// Create a new ObscuraFetcher.
    ///
    /// Binary path resolution order:
    /// 1. `OBSCURA_PATH` environment variable.
    /// 2. Compile-time default (`D:\…\tools\obscura\obscura.exe` on Windows,
    ///    `"obscura"` on other platforms).
    pub fn new() -> Self {
        let binary_path = std::env::var(OBSCURA_PATH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(OBSCURA_DEFAULT_PATH));
        Self {
            binary_path,
            timeout_secs: OBSCURA_TIMEOUT_SECS,
        }
    }

    /// Override the timeout (seconds). Used in tests.
    #[allow(dead_code)]
    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Fetch a URL using obscura, returning a FetchedPage.
    ///
    /// URL validation:
    /// - Must start with `http://` or `https://`.
    /// - Must not contain whitespace.
    ///
    /// Subprocess args are passed as separate arguments (not shell-concatenated)
    /// to prevent shell injection.
    ///
    /// A5.1: stdout is streamed via tokio::process into a capped Vec<u8>. On
    /// cap-hit the child process is killed before returning.
    pub async fn fetch(&self, url: &str) -> Result<FetchedPage, WebSearchError> {
        // --- URL validation (applied BEFORE spawning, no subprocess injection risk) ---
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(WebSearchError::InvalidUrl {
                url: url.to_owned(),
                reason: "URL must start with http:// or https://".to_owned(),
            });
        }
        if url.chars().any(|c| c.is_whitespace()) {
            return Err(WebSearchError::InvalidUrl {
                url: url.to_owned(),
                reason: "URL must not contain whitespace".to_owned(),
            });
        }

        debug!(url = %url, binary = %self.binary_path.display(), timeout_secs = self.timeout_secs, "obscura fetch start");

        let start = Instant::now();
        let html = run_obscura_streaming(
            &self.binary_path,
            url,
            self.timeout_secs,
        )
        .await?;

        let fetch_time_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        debug!(url = %url, fetch_time_ms, "obscura fetch complete");

        Ok(FetchedPage {
            url: url.to_owned(), // obscura doesn't expose final redirect URL via stdout
            status: 200,         // obscura exits non-zero on HTTP error; assume 200 on success
            content_type: "text/html".to_owned(),
            html,
            fetch_time_ms,
        })
    }
}

// ---------------------------------------------------------------------------
// A5.1: streaming subprocess
// ---------------------------------------------------------------------------

/// Run the obscura subprocess and stream its stdout into a Vec<u8> capped at
/// OBSCURA_MAX_OUTPUT_BYTES.
///
/// Uses tokio::process::Command (async) so stdout is an AsyncRead — no
/// spawn_blocking needed. On cap-hit the child is killed and partial content is
/// returned with a truncation warning.
async fn run_obscura_streaming(
    binary_path: &std::path::Path,
    url: &str,
    timeout_secs: u64,
) -> Result<String, WebSearchError> {
    use tokio::process::Command;

    let mut child = Command::new(binary_path)
        .arg("fetch")
        .arg(url)
        .arg("--dump")
        .arg("html")
        .arg("--timeout")
        .arg(timeout_secs.to_string())
        .arg("--stealth")
        .arg("--quiet") // suppress status lines on stderr during non-verbose runs
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            let reason = if e.kind() == std::io::ErrorKind::NotFound {
                format!(
                    "obscura binary not found at '{}': {e}",
                    binary_path.display()
                )
            } else {
                format!("failed to spawn obscura: {e}")
            };
            WebSearchError::FetchFailed {
                url: url.to_owned(),
                reason,
            }
        })?;

    // Take stdout handle — safe to unwrap because we set Stdio::piped() above.
    let mut stdout = child
        .stdout
        .take()
        .expect("stdout was piped but handle is None");

    // Stream stdout in chunks, enforcing the 5 MB hard cap.
    let mut buf = Vec::with_capacity(OBSCURA_READ_CHUNK);
    let mut chunk = vec![0u8; OBSCURA_READ_CHUNK];
    let mut truncated = false;

    loop {
        let n = stdout
            .read(&mut chunk)
            .await
            .map_err(|e| WebSearchError::FetchFailed {
                url: url.to_owned(),
                reason: format!("failed to read obscura stdout: {e}"),
            })?;

        if n == 0 {
            // EOF — subprocess closed stdout.
            break;
        }

        let remaining = OBSCURA_MAX_OUTPUT_BYTES.saturating_sub(buf.len());
        if n > remaining {
            // This read would exceed the cap — take only what fits, then kill.
            buf.extend_from_slice(&chunk[..remaining]);
            truncated = true;
            warn!(
                url = %url,
                buffered = buf.len(),
                cap = OBSCURA_MAX_OUTPUT_BYTES,
                "obscura stdout exceeded cap; killing child and truncating"
            );
            // Kill is best-effort; ignore the error so we can return the content.
            let _ = child.kill().await;
            break;
        }

        buf.extend_from_slice(&chunk[..n]);
    }

    // If we did not truncate, wait for the child to exit and check exit code.
    if !truncated {
        let status = child
            .wait()
            .await
            .map_err(|e| WebSearchError::FetchFailed {
                url: url.to_owned(),
                reason: format!("failed to wait on obscura process: {e}"),
            })?;

        if !status.success() {
            // Read stderr for the error message (best-effort, small read).
            let code = status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "unknown".to_owned());
            return Err(WebSearchError::FetchFailed {
                url: url.to_owned(),
                reason: format!("obscura exited with code {code}"),
            });
        }
    }

    if buf.is_empty() {
        return Err(WebSearchError::FetchFailed {
            url: url.to_owned(),
            reason: "obscura returned empty output".to_owned(),
        });
    }

    // Non-UTF8 safety: lossy conversion (same as WebFetcher).
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

// ---------------------------------------------------------------------------
// A5.2: binary check and doctor
// ---------------------------------------------------------------------------

/// Resolve the obscura binary path and verify it exists.
///
/// Resolution order:
/// 1. `OBSCURA_PATH` environment variable (if set and non-empty).
/// 2. On Windows:
///    a. cwd-relative `tools/obscura/obscura.exe` (works when run from project root).
///    b. Absolute fallback `D:\.ELAI_workflow\.elai_cc\tools\obscura\obscura.exe`.
/// 3. On non-Windows: scan PATH for `obscura` using `which`-style lookup.
///
/// Returns `Ok(resolved_path)` if the binary is found and readable, or
/// `Err(WebSearchError::FetchFailed)` with a diagnostic message if not found.
pub fn check_obscura_binary() -> Result<PathBuf, WebSearchError> {
    // Step 1: explicit env override.
    if let Ok(env_path) = std::env::var(OBSCURA_PATH_ENV) {
        if !env_path.trim().is_empty() {
            let p = PathBuf::from(&env_path);
            if p.exists() {
                return Ok(p);
            }
            return Err(WebSearchError::FetchFailed {
                url: String::new(),
                reason: format!(
                    "OBSCURA_PATH env var points to non-existent path: {}",
                    p.display()
                ),
            });
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Step 2a: cwd-relative path (works when elai is launched from project root).
        let cwd_relative = PathBuf::from("tools").join("obscura").join("obscura.exe");
        if cwd_relative.exists() {
            return Ok(cwd_relative);
        }

        // Step 2b: absolute project path fallback.
        let absolute =
            PathBuf::from(r"D:\.ELAI_workflow\.elai_cc\tools\obscura\obscura.exe");
        if absolute.exists() {
            return Ok(absolute);
        }

        Err(WebSearchError::FetchFailed {
            url: String::new(),
            reason: format!(
                "obscura binary not found; searched: {} and {}",
                cwd_relative.display(),
                absolute.display()
            ),
        })
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Step 3: scan PATH for `obscura`.
        if let Some(found) = find_in_path("obscura") {
            return Ok(found);
        }
        Err(WebSearchError::FetchFailed {
            url: String::new(),
            reason: "obscura not found in PATH".to_owned(),
        })
    }
}

/// Scan the PATH environment variable for a binary named `name`.
/// Returns the first match as an absolute PathBuf, or None.
#[cfg(not(target_os = "windows"))]
fn find_in_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// Run the obscura binary health-check and return a structured report.
///
/// This function is idempotent and side-effect-free (read-only subprocess).
/// It is NOT auto-invoked at module init — call it explicitly from `elai doctor`
/// (see crates/cli/src/doctor.rs, gated on cfg(feature = "obscura")).
pub fn doctor() -> ObscuraDoctorReport {
    match check_obscura_binary() {
        Err(e) => ObscuraDoctorReport {
            binary_path: None,
            version: None,
            error: Some(format!("{e}")),
        },
        Ok(path) => {
            // Run `obscura --version` to confirm the binary is executable.
            let version_result = std::process::Command::new(&path)
                .arg("--version")
                .output();

            match version_result {
                Ok(output) if output.status.success() => {
                    let version_str = String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .to_owned();
                    ObscuraDoctorReport {
                        binary_path: Some(path),
                        version: Some(version_str),
                        error: None,
                    }
                }
                Ok(output) => {
                    let code = output
                        .status
                        .code()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".to_owned());
                    ObscuraDoctorReport {
                        binary_path: Some(path),
                        version: None,
                        error: Some(format!(
                            "obscura --version exited with code {code}"
                        )),
                    }
                }
                Err(e) => ObscuraDoctorReport {
                    binary_path: Some(path),
                    version: None,
                    error: Some(format!("failed to run obscura --version: {e}")),
                },
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_rejects_no_scheme() {
        let fetcher = ObscuraFetcher::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let err = rt.block_on(fetcher.fetch("example.com")).unwrap_err();
        match err {
            WebSearchError::InvalidUrl { reason, .. } => {
                assert!(reason.contains("http"));
            }
            other => panic!("expected InvalidUrl, got {other:?}"),
        }
    }

    #[test]
    fn test_url_rejects_whitespace() {
        let fetcher = ObscuraFetcher::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let err = rt
            .block_on(fetcher.fetch("https://example.com/path with spaces"))
            .unwrap_err();
        match err {
            WebSearchError::InvalidUrl { reason, .. } => {
                assert!(reason.contains("whitespace"));
            }
            other => panic!("expected InvalidUrl, got {other:?}"),
        }
    }

    #[test]
    fn test_url_rejects_ftp_scheme() {
        let fetcher = ObscuraFetcher::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let err = rt
            .block_on(fetcher.fetch("ftp://example.com/file"))
            .unwrap_err();
        assert!(matches!(err, WebSearchError::InvalidUrl { .. }));
    }

    #[test]
    fn test_binary_not_found_gives_fetch_failed() {
        let fetcher = ObscuraFetcher {
            binary_path: PathBuf::from("nonexistent-obscura-binary-xyz"),
            timeout_secs: 5,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let err = rt
            .block_on(fetcher.fetch("https://example.com"))
            .unwrap_err();
        match err {
            WebSearchError::FetchFailed { reason, .. } => {
                assert!(
                    reason.contains("not found") || reason.contains("obscura"),
                    "unexpected reason: {reason}"
                );
            }
            other => panic!("expected FetchFailed, got {other:?}"),
        }
    }

    #[test]
    fn test_constants_values() {
        assert_eq!(OBSCURA_TIMEOUT_SECS, 30);
        assert_eq!(OBSCURA_MAX_OUTPUT_BYTES, 5 * 1024 * 1024);
        assert!(!OBSCURA_PATH_ENV.is_empty());
    }

    #[test]
    fn test_default_path_non_empty() {
        assert!(!OBSCURA_DEFAULT_PATH.is_empty());
    }

    #[test]
    fn test_check_obscura_binary_env_missing_binary() {
        // Point OBSCURA_PATH at a non-existent file; expect FetchFailed.
        std::env::set_var(OBSCURA_PATH_ENV, "/nonexistent/path/to/obscura-xyz");
        let result = check_obscura_binary();
        std::env::remove_var(OBSCURA_PATH_ENV);
        assert!(matches!(result, Err(WebSearchError::FetchFailed { .. })));
    }

    #[test]
    fn test_doctor_returns_report_structure() {
        // doctor() must not panic — result content depends on env.
        let report = doctor();
        // At minimum: if there's no error, binary_path must be Some.
        if report.error.is_none() {
            assert!(report.binary_path.is_some());
        }
    }
}
