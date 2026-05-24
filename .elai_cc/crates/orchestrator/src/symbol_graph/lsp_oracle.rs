//! E-40 — LSP-as-precision-oracle subprocess adapter.
//!
//! Wires the new `RuntimeBackend::Lsp` variant (added in
//! `providers::runtime::types`) onto a subprocess invocation of the
//! per-language LSP server (rust-analyzer / pylsp / tsserver / gopls).
//! Routed only from `Unresolved`-confidence call sites — `>5 invocations
//! per session` is treated as a bug per
//! `workflow/reasoning/E-40-decisions.md` on-demand-only discipline (the
//! token-overhead anti-pattern from Preprints 202510.0924).
//!
//! `ToolchainMissing` is the documented RUNPOD-PENDING wire-dark: real
//! integration tests against running rust-analyzer / pylsp / tsserver
//! require an installed toolchain on the host, which the CI lane does
//! NOT provide. The function shape ships now so the dashboard +
//! contract test can pin the surface, and the CLI dispatch can wire it,
//! without blocking on toolchain provisioning. See
//! `workflow/reasoning/E-40-decisions.md` "RUNPOD-PENDING" section.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::Command;
use tracing::warn;

/// Languages this oracle knows how to invoke. The mapping from language
/// label to canonical binary lives in [`OracleKind::canonical_binary`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleKind {
    Rust,
    Python,
    TypeScript,
    Go,
}

impl OracleKind {
    /// Parse a language slug. Returns `None` for unknown values; callers
    /// surface that as `LspOracleError::UnsupportedLanguage`.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug.to_ascii_lowercase().as_str() {
            "rust" | "rs" => Some(Self::Rust),
            "python" | "py" => Some(Self::Python),
            "typescript" | "ts" | "tsx" => Some(Self::TypeScript),
            "go" => Some(Self::Go),
            _ => None,
        }
    }

    /// Canonical binary name (looked up via `which`-style lookup at runtime).
    #[must_use]
    pub const fn canonical_binary(self) -> &'static str {
        match self {
            Self::Rust => "rust-analyzer",
            Self::Python => "pylsp",
            Self::TypeScript => "typescript-language-server",
            Self::Go => "gopls",
        }
    }

    /// Snake-case label for SSB beacon hashing.
    #[must_use]
    pub const fn snake_label(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::TypeScript => "typescript",
            Self::Go => "go",
        }
    }
}

/// Result of an oracle lookup. Returns enough information that the
/// caller (e.g. compute_symbol_context::Unresolved escalation) can
/// promote a `Confidence::Unresolved` row to `Resolved` + cite the
/// oracle path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleAnswer {
    pub kind: OracleKind,
    pub symbol: String,
    /// Toolchain binary path the oracle dispatched to. Empty when the
    /// `RUNPOD-PENDING` short-circuit fired.
    pub binary: String,
    /// Whether the underlying toolchain was actually invoked. False ==
    /// RUNPOD-PENDING wire-dark path (toolchain absent on host).
    pub invoked: bool,
}

/// Error surface for `oracle_lookup`.
#[derive(Debug, Error)]
pub enum LspOracleError {
    /// Language slug not in the supported set.
    #[error("lsp_oracle: language '{slug}' not supported (rust|python|typescript|go)")]
    UnsupportedLanguage { slug: String },
    /// Caller passed empty symbol name.
    #[error("lsp_oracle: symbol name must be non-empty")]
    EmptySymbol,
    /// Toolchain binary not on PATH. This is RUNPOD-PENDING — the test
    /// lane cannot provide rust-analyzer / pylsp / etc., so this error
    /// SHOULD be returned as an `OracleAnswer { invoked: false, .. }`
    /// path in production. Direct callers MAY treat this as a soft
    /// signal rather than a hard error per the on-demand-only contract.
    #[error("lsp_oracle: toolchain '{binary}' not found on PATH (RUNPOD-PENDING)")]
    ToolchainMissing { binary: String },
    /// Subprocess spawn or wait failed for some other reason. Wrapped
    /// so callers can rethrow without losing the original I/O cause.
    #[error("lsp_oracle: subprocess error for {binary}: {source}")]
    Subprocess {
        binary: String,
        #[source]
        source: std::io::Error,
    },
}

/// Max wall-clock budget for a single LSP probe. The oracle is on-demand
/// only; > 5s on a single probe is a misconfiguration.
const ORACLE_TIMEOUT: Duration = Duration::from_secs(5);

/// Probe the LSP server for `symbol` in `language`. Returns a synthetic
/// `OracleAnswer { invoked: false, .. }` when the toolchain is absent
/// (the documented RUNPOD-PENDING path) so callers can decide whether
/// to escalate or accept the gap.
///
/// Callers SHOULD emit `SsbActionType::LspOracleConsulted` around this
/// call to keep the on-demand-only discipline visible in SSB.
pub async fn oracle_lookup(
    symbol: &str,
    language: &str,
) -> Result<OracleAnswer, LspOracleError> {
    if symbol.is_empty() {
        return Err(LspOracleError::EmptySymbol);
    }
    let kind = OracleKind::from_slug(language).ok_or_else(|| {
        LspOracleError::UnsupportedLanguage {
            slug: language.to_string(),
        }
    })?;
    let binary = kind.canonical_binary();
    let on_path = match find_on_path(binary) {
        Some(p) => p,
        None => {
            // Documented RUNPOD-PENDING — surface synthetic answer.
            warn!(
                "lsp_oracle: toolchain {binary} absent on PATH; returning RUNPOD-PENDING synthetic answer for symbol={symbol} language={language}"
            );
            return Ok(OracleAnswer {
                kind,
                symbol: symbol.to_string(),
                binary: String::new(),
                invoked: false,
            });
        }
    };
    // Real LSP servers speak JSON-RPC over stdio with an initialize +
    // textDocument/definition handshake. The full handshake is RUNPOD-
    // PENDING — implementing it requires the in-tree LSP client (B-30
    // PLANNED) which is outside E-40 scope. For E-40 we shell out to
    // `--version` to confirm the binary is invocable (the smallest
    // grounded liveness probe). A follow-on patch-N completes the JSON-
    // RPC handshake when B-30 ships its LSP client surface.
    let result = tokio::time::timeout(
        ORACLE_TIMEOUT,
        Command::new(&on_path)
            .arg("--version")
            .kill_on_drop(true)
            .output(),
    )
    .await;
    let output = match result {
        Ok(Ok(o)) => o,
        Ok(Err(source)) => {
            return Err(LspOracleError::Subprocess {
                binary: binary.to_string(),
                source,
            });
        }
        Err(_elapsed) => {
            return Err(LspOracleError::Subprocess {
                binary: binary.to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "lsp_oracle --version exceeded 5s budget",
                ),
            });
        }
    };
    let invoked = output.status.success();
    Ok(OracleAnswer {
        kind,
        symbol: symbol.to_string(),
        binary: on_path.to_string_lossy().to_string(),
        invoked,
    })
}

/// Lightweight PATH lookup. Returns the resolved absolute path of
/// `binary` if found via `which`-style scanning of `$PATH`, else None.
fn find_on_path(binary: &str) -> Option<PathBuf> {
    let path_env = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_env) {
        for ext in candidate_extensions() {
            let mut candidate = dir.join(binary);
            if !ext.is_empty() {
                candidate.set_extension(ext);
            }
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(windows)]
fn candidate_extensions() -> &'static [&'static str] {
    &["exe", "cmd", ""]
}

#[cfg(not(windows))]
fn candidate_extensions() -> &'static [&'static str] {
    &[""]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_kind_from_slug_accepts_aliases() {
        assert_eq!(OracleKind::from_slug("rust"), Some(OracleKind::Rust));
        assert_eq!(OracleKind::from_slug("rs"), Some(OracleKind::Rust));
        assert_eq!(OracleKind::from_slug("python"), Some(OracleKind::Python));
        assert_eq!(OracleKind::from_slug("TS"), Some(OracleKind::TypeScript));
        assert_eq!(OracleKind::from_slug("unknown"), None);
    }

    #[tokio::test]
    async fn empty_symbol_rejected() {
        let result = oracle_lookup("", "rust").await;
        assert!(matches!(result, Err(LspOracleError::EmptySymbol)));
    }

    #[tokio::test]
    async fn unknown_language_rejected() {
        let result = oracle_lookup("foo", "cobol").await;
        assert!(matches!(
            result,
            Err(LspOracleError::UnsupportedLanguage { .. })
        ));
    }

    #[tokio::test]
    async fn missing_toolchain_returns_synthetic_answer() {
        // The test lane does not ship rust-analyzer; oracle_lookup must
        // return invoked=false rather than panicking.
        let result = oracle_lookup("std::option::Option", "rust").await;
        match result {
            Ok(answer) => {
                if !answer.invoked {
                    assert!(answer.binary.is_empty(), "synthetic answer must have empty binary");
                }
            }
            Err(LspOracleError::Subprocess { .. }) => {
                // Acceptable — toolchain found but subprocess errored. The
                // contract guarantees one of these branches.
            }
            Err(e) => panic!("unexpected error variant: {e:?}"),
        }
    }
}
