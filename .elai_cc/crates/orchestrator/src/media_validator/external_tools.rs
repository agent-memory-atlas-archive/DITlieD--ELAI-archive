//! B-24 Phase 4I (Group C1): external CLI wrappers — skeleton.
//!
//! Wraps each command-line tool (ImageMagick / FFprobe / Tesseract /
//! Blender) behind a uniform [`ExternalToolWrapper`] trait so the Tier-1
//! validators can substitute mocks during contract tests. `execute` is
//! deliberately stubbed at this stage (returns `ToolError::NotImplemented`)
//! — Group C3 fills in the real subprocess calls. `check_available` is
//! REAL because 4I.2's PRE_GATE relies on the ToolCapabilityFlags
//! detection working today.
//!
//! Cross-platform: Windows uses `where.exe`, everything else uses `which`
//! per CLAUDE.md context-package L301-325.

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use thiserror::Error;
use tracing::warn;

use super::types::ToolCapabilityFlags;

/// Captured output of an external CLI invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("tool {tool} timed out after {timeout_ms} ms")]
    Timeout { tool: &'static str, timeout_ms: u64 },

    #[error("tool {tool} not on PATH")]
    NotAvailable { tool: &'static str },

    /// Subprocess returned non-zero exit code AND stderr matched the
    /// canonical failure markers ("error" / "corrupt" / "could not"). Per
    /// workflow/papers/imagemagick-tesseract-interface.md L40-46: exit code
    /// alone is unreliable, must combine with stderr inspection.
    #[error("tool {tool} failed: exit={exit_code} stderr={stderr}")]
    Failed {
        tool: &'static str,
        exit_code: i32,
        stderr: String,
    },

    #[error("not implemented yet (Group C3): {0}")]
    NotImplemented(&'static str),
}

/// Uniform abstraction over a single external CLI. Implementors must be
/// `Send + Sync` so a single `Arc<dyn ExternalToolWrapper>` can be cloned
/// across the Tier-1 worker pool.
///
/// `execute` is a native async fn (NOT `async-trait`) per CLAUDE.md
/// anti-stale rule — Rust 1.75+ supports async fn in traits directly when
/// only Send + Sync future is required. We use a boxed Future return to
/// keep the trait object-safe for `Arc<dyn ExternalToolWrapper>`.
pub trait ExternalToolWrapper: Send + Sync {
    /// Static name of the wrapped CLI ("imagemagick", "ffprobe", ...).
    fn name(&self) -> &'static str;

    /// Synchronous availability probe. Run once at MediaValidator startup
    /// and cached in `ToolCapabilityFlags`.
    fn check_available(&self) -> bool;

    /// Stub execute — Group C3 replaces with real `tokio::process::Command`.
    /// Boxed Future for object-safety.
    fn execute<'a>(
        &'a self,
        args: &'a [&'a str],
        timeout: Duration,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ToolOutput, ToolError>> + Send + 'a>>;
}

// ---------------------------------------------------------------------------
// Real subprocess execution helper (Group C3)
// ---------------------------------------------------------------------------

/// Spawn `binary` with `args`, enforcing `timeout`. On timeout the child
/// is killed (best-effort) and the canonical [`ToolError::Timeout`] is
/// returned. On non-zero exit AND stderr containing one of the standard
/// failure markers, [`ToolError::Failed`] is returned. Otherwise the
/// captured output is returned regardless of exit code (callers that
/// care about exit code inspect `ToolOutput.exit_code` directly).
///
/// `tool_name` MUST be the static identifier passed to
/// [`ExternalToolWrapper::name`] so error variants and tracing logs
/// stay consistent across the wrapper hierarchy.
async fn execute_with_timeout(
    binary: &std::path::Path,
    args: &[&str],
    timeout: Duration,
    tool_name: &'static str,
) -> Result<ToolOutput, ToolError> {
    use tokio::process::Command as TokioCommand;
    let mut cmd = TokioCommand::new(binary);
    cmd.args(args);
    // Inherit env minimally; production hardening (locale, PATH stripping)
    // is the sandbox layer's job (Group C6).
    let timeout_ms = u64::try_from(timeout.as_millis()).unwrap_or(u64::MAX);
    let spawn = match tokio::time::timeout(timeout, cmd.output()).await {
        Ok(spawn_res) => spawn_res?,
        Err(_) => {
            // Tokio elapsed; the child is already aborted by drop, but
            // surface the canonical Timeout error.
            warn!(
                tool = tool_name,
                timeout_ms,
                "external tool {tool_name} exceeded {timeout_ms} ms; killed"
            );
            return Err(ToolError::Timeout {
                tool: tool_name,
                timeout_ms,
            });
        }
    };
    let exit_code = spawn.status.code().unwrap_or(-1);
    let stdout = spawn.stdout;
    let stderr_bytes = spawn.stderr;
    let stderr_lower = String::from_utf8_lossy(&stderr_bytes).to_lowercase();
    if exit_code != 0
        && (stderr_lower.contains("error")
            || stderr_lower.contains("corrupt")
            || stderr_lower.contains("could not"))
    {
        return Err(ToolError::Failed {
            tool: tool_name,
            exit_code,
            stderr: String::from_utf8_lossy(&stderr_bytes).to_string(),
        });
    }
    Ok(ToolOutput {
        stdout,
        stderr: stderr_bytes,
        exit_code,
    })
}

// ---------------------------------------------------------------------------
// Cross-platform binary discovery
// ---------------------------------------------------------------------------

/// Locate `name` on the current PATH. Returns `Some(absolute_path)` on the
/// first hit, `None` if no candidate resolves. We try `name` first then
/// each entry of `alt_names` for tools that ship multiple aliases (e.g.
/// ImageMagick exposes both `magick` and `identify`).
fn detect_binary(name: &str, alt_names: &[&str]) -> Option<PathBuf> {
    let probe = if cfg!(target_os = "windows") {
        "where.exe"
    } else {
        "which"
    };
    for candidate in std::iter::once(name).chain(alt_names.iter().copied()) {
        let Ok(output) = Command::new(probe).arg(candidate).output() else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        let path_str = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = path_str.lines().next() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed));
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Concrete wrappers (skeleton — execute() unimplemented)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ImageMagickWrapper {
    /// Absolute path resolved at construction time. `None` -> tool missing.
    pub binary_path: Option<PathBuf>,
}

impl ImageMagickWrapper {
    pub fn new() -> Self {
        Self {
            binary_path: detect_binary("magick", &["identify"]),
        }
    }
}

impl Default for ImageMagickWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalToolWrapper for ImageMagickWrapper {
    fn name(&self) -> &'static str {
        "imagemagick"
    }

    fn check_available(&self) -> bool {
        self.binary_path.is_some()
    }

    fn execute<'a>(
        &'a self,
        args: &'a [&'a str],
        timeout: Duration,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ToolOutput, ToolError>> + Send + 'a>>
    {
        Box::pin(async move {
            let Some(path) = self.binary_path.as_ref() else {
                return Err(ToolError::NotAvailable { tool: "imagemagick" });
            };
            execute_with_timeout(path, args, timeout, "imagemagick").await
        })
    }
}

#[derive(Debug, Clone)]
pub struct FFprobeWrapper {
    pub binary_path: Option<PathBuf>,
}

impl FFprobeWrapper {
    pub fn new() -> Self {
        Self {
            binary_path: detect_binary("ffprobe", &[]),
        }
    }
}

impl Default for FFprobeWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalToolWrapper for FFprobeWrapper {
    fn name(&self) -> &'static str {
        "ffprobe"
    }

    fn check_available(&self) -> bool {
        self.binary_path.is_some()
    }

    fn execute<'a>(
        &'a self,
        args: &'a [&'a str],
        timeout: Duration,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ToolOutput, ToolError>> + Send + 'a>>
    {
        Box::pin(async move {
            let Some(path) = self.binary_path.as_ref() else {
                return Err(ToolError::NotAvailable { tool: "ffprobe" });
            };
            execute_with_timeout(path, args, timeout, "ffprobe").await
        })
    }
}

#[derive(Debug, Clone)]
pub struct TesseractWrapper {
    pub binary_path: Option<PathBuf>,
}

impl TesseractWrapper {
    pub fn new() -> Self {
        Self {
            binary_path: detect_binary("tesseract", &[]),
        }
    }
}

impl Default for TesseractWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalToolWrapper for TesseractWrapper {
    fn name(&self) -> &'static str {
        "tesseract"
    }

    fn check_available(&self) -> bool {
        self.binary_path.is_some()
    }

    fn execute<'a>(
        &'a self,
        args: &'a [&'a str],
        timeout: Duration,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ToolOutput, ToolError>> + Send + 'a>>
    {
        Box::pin(async move {
            let Some(path) = self.binary_path.as_ref() else {
                return Err(ToolError::NotAvailable { tool: "tesseract" });
            };
            execute_with_timeout(path, args, timeout, "tesseract").await
        })
    }
}

#[derive(Debug, Clone)]
pub struct BlenderWrapper {
    pub binary_path: Option<PathBuf>,
}

impl BlenderWrapper {
    pub fn new() -> Self {
        Self {
            binary_path: detect_binary("blender", &[]),
        }
    }
}

impl Default for BlenderWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalToolWrapper for BlenderWrapper {
    fn name(&self) -> &'static str {
        "blender"
    }

    fn check_available(&self) -> bool {
        self.binary_path.is_some()
    }

    fn execute<'a>(
        &'a self,
        args: &'a [&'a str],
        timeout: Duration,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ToolOutput, ToolError>> + Send + 'a>>
    {
        Box::pin(async move {
            let Some(path) = self.binary_path.as_ref() else {
                return Err(ToolError::NotAvailable { tool: "blender" });
            };
            execute_with_timeout(path, args, timeout, "blender").await
        })
    }
}

// ---------------------------------------------------------------------------
// Aggregate detection — used at MediaValidator startup.
// ---------------------------------------------------------------------------

/// Probe the host system once and return the union of detected tools.
/// Called from `MediaValidator::new_if_media_steps` (Group C5).
pub fn build_capability_flags() -> ToolCapabilityFlags {
    let mut flags = ToolCapabilityFlags::empty();
    if ImageMagickWrapper::new().check_available() {
        flags |= ToolCapabilityFlags::IMAGEMAGICK;
    }
    if FFprobeWrapper::new().check_available() {
        flags |= ToolCapabilityFlags::FFPROBE;
    }
    // ffmpeg is detected via the same crate as ffprobe but separately so
    // future consumers can branch on it; still uses `which`/`where.exe`.
    if detect_binary("ffmpeg", &[]).is_some() {
        flags |= ToolCapabilityFlags::FFMPEG;
    }
    if TesseractWrapper::new().check_available() {
        flags |= ToolCapabilityFlags::TESSERACT;
    }
    if BlenderWrapper::new().check_available() {
        flags |= ToolCapabilityFlags::BLENDER;
    }
    if detect_binary("npx", &[]).is_some() {
        flags |= ToolCapabilityFlags::PLAYWRIGHT;
    }
    // OLLAMA / SGLANG / COMFYUI default OFF — they are services, not CLIs;
    // Group C5 adds /v1/health probes.
    flags
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_binary_returns_none_for_garbage() {
        // We use a guaranteed-missing name; this protects against false
        // positives from environment-specific PATH entries.
        let probe = detect_binary("definitely-not-a-real-binary-9f3c1", &[]);
        assert!(probe.is_none());
    }

    #[test]
    fn build_capability_flags_does_not_panic() {
        // Smoke test: regardless of which tools are installed, the call
        // must succeed without panicking. PRE_GATE relies on this.
        let _flags = build_capability_flags();
    }

    #[test]
    fn imagemagick_wrapper_execute_dispatches_or_reports_unavailable() {
        // Group C3 GREEN-phase replacement: the wrapper now returns either a
        // real ToolOutput (when ImageMagick is installed) or
        // ToolError::NotAvailable (when it is not). Both outcomes are
        // acceptable; the previous NotImplemented sentinel is gone.
        let wrapper = ImageMagickWrapper::new();
        let fut = wrapper.execute(&["-version"], Duration::from_secs(5));
        let res = futures_executor_block_on(fut);
        match res {
            Ok(_) => {}
            Err(ToolError::NotAvailable { .. }) => {}
            // Failed/Timeout are also acceptable on hostile hosts.
            Err(ToolError::Failed { .. }) | Err(ToolError::Timeout { .. }) => {}
            other => panic!("unexpected error variant from execute: {other:?}"),
        }
    }

    /// Single-future executor backed by a tokio current-thread runtime. The
    /// previous hand-rolled `Pending => continue` loop with a Noop waker only
    /// worked when futures resolved immediately; once Group C3 introduces
    /// `tokio::process::Command`, real async I/O will yield `Pending` and the
    /// spin loop would hang. Tokio is already a workspace dependency with the
    /// `rt` feature, so no new crate is required.
    fn futures_executor_block_on<F: std::future::Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build current-thread tokio runtime for test")
            .block_on(future)
    }
}
