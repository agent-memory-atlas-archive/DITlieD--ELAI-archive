//! Batch 2A — ShortCoder PostToolUse hook.
//!
//! Wraps [`elai_knowledge::ShortCoder`] in an orchestrator-level hook that
//! fires from the `InProcessHookDispatcher::dispatch_post_tool_use` chain.
//! The goal, per `docs/mvp-wiring-audit-2026-04-18.md` section 3 A-24, is
//! to compress an agent's PostToolUse response via AST-preserving
//! tree-sitter simplification BEFORE the auditor reads it — the executor
//! still sees the raw text. This is non-mutating on the input: we return a
//! fresh `String` and the original is preserved.
//!
//! Env kill switch: `ELAI_DISABLE_SHORTCODER=1` — when set, every call is
//! an identity no-op (compressed == input). This short-circuits before any
//! tree-sitter parse so a suspect file can be debugged without recompiling.
//!
//! Language detection: derived from the `file_path` extension the hook is
//! invoked with (same extension→language map used by T2). When no file
//! path is available (task-level events), the hook falls back to the
//! `"rust"` grammar, which is conservative — unknown content won't parse
//! cleanly and we fall back to identity rather than emitting garbage.

use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, warn};

use elai_knowledge::shortcoder::{
    ShortCoder, ShortCoderError as KnowledgeShortCoderError, SimplificationResult,
};

// ---------------------------------------------------------------------------
// Env + constants
// ---------------------------------------------------------------------------

/// Env var that disables ShortCoder compression entirely. When `1`/`true`/`on`,
/// `compress()` returns an identity [`ShortCoderOutcome::Skipped`] result.
pub const ELAI_DISABLE_SHORTCODER_ENV: &str = "ELAI_DISABLE_SHORTCODER";

/// Cap on the number of rule labels the dashboard row carries. Prevents
/// runaway payload sizes if a future rule expansion pushes past 10.
pub const SHORTCODER_MAX_RULES_EMITTED: usize = 10;

// ---------------------------------------------------------------------------
// Error + outcome
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ShortCoderHookError {
    #[error("shortcoder failed: {0}")]
    Underlying(String),
}

impl From<KnowledgeShortCoderError> for ShortCoderHookError {
    fn from(err: KnowledgeShortCoderError) -> Self {
        Self::Underlying(err.to_string())
    }
}

/// Outcome of a single ShortCoder invocation.
///
/// `Compressed` carries the simplification result produced by
/// `elai_knowledge::ShortCoder::simplify` verbatim. `Skipped` is returned
/// when the env kill switch is on, when the language is unsupported, or
/// when the input is empty — the caller should use the original input
/// unchanged in every Skipped case.
#[derive(Debug, Clone)]
pub enum ShortCoderOutcome {
    /// Tree-sitter simplification ran. The `result` may have zero rules
    /// applied (identity) when the input had nothing to compress.
    Compressed { result: SimplificationResult },
    /// Hook short-circuited. `reason` is a stable label for telemetry.
    Skipped { reason: &'static str },
}

impl ShortCoderOutcome {
    /// Short label suitable for the SSB row `reduction_pct` field.
    #[must_use]
    pub fn reduction_pct(&self) -> u32 {
        match self {
            Self::Compressed { result } => {
                if result.original_bytes == 0 || result.simplified_bytes >= result.original_bytes {
                    return 0;
                }
                let saved = result.original_bytes - result.simplified_bytes;
                // saved <= original_bytes, so the cast is bounded. The
                // multiplication saturates at u64::MAX which is well past the
                // largest sensible file size; we clamp to 100 after anyway.
                let pct = (saved as u64).saturating_mul(100) / result.original_bytes as u64;
                u32::try_from(pct.min(100)).unwrap_or(100)
            }
            Self::Skipped { .. } => 0,
        }
    }

    /// Approximate tokens-in count using the codebase-standard 4-chars-per-token.
    #[must_use]
    pub fn tokens_in(&self) -> u32 {
        match self {
            Self::Compressed { result } => approx_tokens(result.original_bytes),
            Self::Skipped { .. } => 0,
        }
    }

    /// Approximate tokens-out count using the codebase-standard 4-chars-per-token.
    #[must_use]
    pub fn tokens_out(&self) -> u32 {
        match self {
            Self::Compressed { result } => approx_tokens(result.simplified_bytes),
            Self::Skipped { .. } => 0,
        }
    }

    /// Stable rule labels that fired (capped to
    /// [`SHORTCODER_MAX_RULES_EMITTED`]). Empty for Skipped.
    #[must_use]
    pub fn rules_applied_labels(&self) -> Vec<String> {
        match self {
            Self::Compressed { result } => result
                .rules_applied
                .iter()
                .take(SHORTCODER_MAX_RULES_EMITTED)
                .map(|r| r.to_string())
                .collect(),
            Self::Skipped { .. } => Vec::new(),
        }
    }
}

fn approx_tokens(bytes: usize) -> u32 {
    // 4 chars per token — same heuristic used by output_postprocessor.rs.
    // Saturates at u32::MAX for absurdly large inputs; the max real input
    // is bounded by PAYLOAD_MAX_BYTES (1 MiB), well under u32 range.
    u32::try_from((bytes / 4).min(u32::MAX as usize)).unwrap_or(u32::MAX)
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

/// Production PostToolUse ShortCoder hook.
///
/// Thread-safe (holds only immutable grammars via `Arc<ShortCoder>`) so
/// one instance can be shared across every task_runner baby step. The
/// `compress` call is synchronous — tree-sitter is CPU-bound and typically
/// finishes in single-digit milliseconds for the agent-response sizes
/// seen at PostToolUse (usually <5k LOC).
pub struct ShortCoderHook {
    inner: Arc<ShortCoder>,
    disabled: bool,
}

impl ShortCoderHook {
    /// Build a new hook wrapping the shared `ShortCoder` engine. Reads the
    /// kill-switch env var once at construction — rebuild the hook to pick
    /// up a later change.
    #[must_use]
    pub fn new(inner: Arc<ShortCoder>) -> Self {
        let disabled = std::env::var(ELAI_DISABLE_SHORTCODER_ENV)
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
            .unwrap_or(false);
        if disabled {
            warn!(
                env = ELAI_DISABLE_SHORTCODER_ENV,
                "ShortCoder disabled via env — PostToolUse compression bypassed",
            );
        }
        Self { inner, disabled }
    }

    /// Whether the env-var kill switch is engaged.
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Compress an agent's PostToolUse response.
    ///
    /// The `file_path` is used ONLY to pick the tree-sitter grammar —
    /// ShortCoder never touches the filesystem. When `file_path` is
    /// `None` or the extension is unsupported, the hook returns
    /// `Skipped`. When the env kill switch is engaged, returns `Skipped`.
    /// When the underlying simplifier fails, returns `Skipped` instead of
    /// propagating the error (the executor still sees the raw text, the
    /// auditor falls back to it too).
    pub fn compress(
        &self,
        source: &str,
        file_path: Option<&str>,
    ) -> ShortCoderOutcome {
        if self.disabled {
            return ShortCoderOutcome::Skipped { reason: "env_disabled" };
        }
        if source.is_empty() {
            return ShortCoderOutcome::Skipped { reason: "empty_input" };
        }
        let Some(lang) = file_path.and_then(detect_language) else {
            return ShortCoderOutcome::Skipped { reason: "unsupported_language" };
        };
        match self.inner.simplify(source, lang) {
            Ok(result) => ShortCoderOutcome::Compressed { result },
            Err(e) => {
                debug!(error = %e, lang, "ShortCoder.simplify failed; falling back to identity");
                ShortCoderOutcome::Skipped { reason: "simplify_failed" }
            }
        }
    }
}

/// Map a file extension to a ShortCoder language key. Uses the same 7
/// languages the `ShortCoder::new()` ctor registers (rust / python /
/// typescript / go / java / c_sharp / ruby).
fn detect_language(file_path: &str) -> Option<&'static str> {
    let ext = file_path.rsplit('.').next()?;
    match ext {
        "rs" => Some("rust"),
        "py" | "pyi" => Some("python"),
        "ts" | "tsx" => Some("typescript"),
        "go" => Some("go"),
        "java" => Some("java"),
        "cs" => Some("c_sharp"),
        "rb" => Some("ruby"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests (unit level — contract-level coverage lives in tiered_analysis_contract.rs)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skipped_when_file_path_absent() {
        let hook = ShortCoderHook::new(Arc::new(ShortCoder::new()));
        let out = hook.compress("fn x() { let a = 1; }", None);
        assert!(matches!(out, ShortCoderOutcome::Skipped { reason } if reason == "unsupported_language"));
        assert_eq!(out.reduction_pct(), 0);
        assert_eq!(out.tokens_in(), 0);
        assert_eq!(out.tokens_out(), 0);
        assert!(out.rules_applied_labels().is_empty());
    }

    #[test]
    fn skipped_when_input_empty() {
        let hook = ShortCoderHook::new(Arc::new(ShortCoder::new()));
        let out = hook.compress("", Some("src/x.rs"));
        assert!(matches!(out, ShortCoderOutcome::Skipped { reason } if reason == "empty_input"));
    }

    #[test]
    fn compresses_rust_function_body() {
        let hook = ShortCoderHook::new(Arc::new(ShortCoder::new()));
        let src = "fn greet() {\n    println!(\"hello, world\");\n    let x = 42;\n}\n";
        let out = hook.compress(src, Some("src/main.rs"));
        match out {
            ShortCoderOutcome::Compressed { result } => {
                // Either the body collapsed (rules_applied non-empty) or
                // the input was already minimal. Either way tokens_in is
                // non-zero and the output is no larger than the input.
                assert!(result.original_bytes > 0);
                assert!(result.simplified_bytes <= result.original_bytes);
            }
            ShortCoderOutcome::Skipped { reason } => panic!("unexpected skip: {reason}"),
        }
    }

    #[test]
    fn detect_language_maps_all_seven_extensions() {
        assert_eq!(detect_language("a.rs"), Some("rust"));
        assert_eq!(detect_language("a.py"), Some("python"));
        assert_eq!(detect_language("a.ts"), Some("typescript"));
        assert_eq!(detect_language("a.go"), Some("go"));
        assert_eq!(detect_language("a.java"), Some("java"));
        assert_eq!(detect_language("a.cs"), Some("c_sharp"));
        assert_eq!(detect_language("a.rb"), Some("ruby"));
        assert!(detect_language("a.toml").is_none());
    }

    #[test]
    fn reduction_pct_zero_on_skip() {
        let out = ShortCoderOutcome::Skipped { reason: "empty_input" };
        assert_eq!(out.reduction_pct(), 0);
    }
}
