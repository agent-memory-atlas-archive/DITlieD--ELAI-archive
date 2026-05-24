//! C-40: thiserror-derived error surface for the GitHub PR adapter.
//!
//! Per CLAUDE.md anti-stale-training rule: thiserror at the public
//! library boundary, no `Box<dyn Error>`, no anyhow at the trait
//! surface. The concrete `GhCliAdapter` impl may use `anyhow` internally
//! for IO/parse plumbing (mirroring `wip_squash.rs`), but the trait
//! method signatures expose only [`GithubPrError`].

use std::process::ExitStatus;

use thiserror::Error;

/// Failure modes surfaced by the GitHub PR adapter trait. Each variant
/// maps to a distinct, actionable diagnostic the dashboard / CLI can
/// render without parsing free-form error strings.
#[derive(Debug, Error)]
pub enum GithubPrError {
    /// The `gh` binary could not be located on `PATH`. The user needs
    /// to install GitHub CLI (cli.github.com) before retrying.
    #[error(
        "gh CLI not found on PATH. Install GitHub CLI (https://cli.github.com) and retry."
    )]
    GhCliNotInstalled,

    /// `gh auth status` reported the binary is present but not
    /// authenticated. The user needs to run `gh auth login` before
    /// retrying. `stderr` is the captured `gh` stderr for diagnostics.
    #[error("gh CLI not authenticated: {stderr}")]
    GhCliNotAuthenticated {
        /// Captured stderr from `gh auth status` so callers can surface
        /// the exact failure to the user.
        stderr: String,
    },

    /// A `gh` invocation exited non-zero. `command` is the full argv
    /// joined by spaces (auth tokens are never embedded in argv — `gh`
    /// reads them from its own keychain — so this is safe to log).
    /// `status` is the OS-level exit code; `stderr` is captured stderr.
    #[error("gh command failed (exit={status}): {command} :: stderr={stderr}")]
    GhCommandFailed {
        /// Full `gh ...` argv joined by single spaces.
        command: String,
        /// Best-effort textual exit-code representation.
        status: String,
        /// Captured stderr text.
        stderr: String,
    },

    /// `gh` exited cleanly but its `--json` output failed to deserialize
    /// into the expected schema. Carries the raw stdout (capped at 4KB
    /// to avoid log bloat) and the underlying serde error.
    #[error("gh JSON parse failed: {source} :: raw={raw_preview}")]
    GhJsonParse {
        /// Raw stdout (truncated to 4096 bytes) for postmortem inspection.
        raw_preview: String,
        /// Underlying serde error.
        #[source]
        source: serde_json::Error,
    },

    /// Local IO failure while spawning or reading from the `gh`
    /// subprocess (e.g. pipe closed, broken stdin handle).
    #[error("gh subprocess IO error: {source}")]
    GhIo {
        /// Underlying std::io::Error.
        #[source]
        source: std::io::Error,
    },

    /// An SSB write failed while emitting a C-40 PR row. The adapter
    /// continues operating (the GH-side change is already committed by
    /// the time the row is written), but observability is degraded.
    /// Surfaced as Err so callers can decide whether to retry or warn.
    #[error("ssb write failed: {source}")]
    SsbWrite {
        /// Underlying SSB error.
        #[source]
        source: crate::ssb::SsbError,
    },

    /// Caller passed a malformed [`PrOpenRequest`] (empty title /
    /// branch, etc). Defensive — the CLI / dashboard layer should
    /// validate first, but we double-check at the adapter boundary so a
    /// stray empty string can't open a broken PR on GitHub.
    #[error("invalid PrOpenRequest: {reason}")]
    InvalidRequest {
        /// Human-readable explanation.
        reason: String,
    },
}

impl GithubPrError {
    /// Helper for constructing [`GithubPrError::GhCommandFailed`] from a
    /// process [`ExitStatus`] + stderr bytes.
    #[must_use]
    pub fn from_exit(command: String, status: ExitStatus, stderr: Vec<u8>) -> Self {
        let stderr = String::from_utf8_lossy(&stderr).to_string();
        let status = match status.code() {
            Some(code) => code.to_string(),
            None => "signal".to_string(),
        };
        GithubPrError::GhCommandFailed {
            command,
            status,
            stderr,
        }
    }

    /// Truncating constructor for [`GithubPrError::GhJsonParse`]: caps
    /// the raw preview at 4096 bytes so a malformed gh response can't
    /// flood logs.
    #[must_use]
    pub fn from_json(raw: &str, source: serde_json::Error) -> Self {
        const CAP: usize = 4096;
        let raw_preview = if raw.len() > CAP {
            // Cap at CAP bytes; rely on string truncation at a char
            // boundary by finding the largest substring <= CAP.
            let mut end = CAP;
            while !raw.is_char_boundary(end) && end > 0 {
                end -= 1;
            }
            format!("{}...(truncated)", &raw[..end])
        } else {
            raw.to_string()
        };
        GithubPrError::GhJsonParse {
            raw_preview,
            source,
        }
    }
}

impl From<std::io::Error> for GithubPrError {
    fn from(source: std::io::Error) -> Self {
        GithubPrError::GhIo { source }
    }
}

impl From<crate::ssb::SsbError> for GithubPrError {
    fn from(source: crate::ssb::SsbError) -> Self {
        GithubPrError::SsbWrite { source }
    }
}
