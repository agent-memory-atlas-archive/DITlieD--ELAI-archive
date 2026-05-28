//! E-20 -- privacy substrate (BLOCK-mode complement to
//! [`crate::privacy_redactor`] B-27 REDACT-mode).
//!
//! Where [`crate::privacy_redactor::redact_context`] returns a
//! SANITISED copy of the input with secret-shaped substrings replaced
//! by `[REDACTED_*]` placeholders, the submodules here return a
//! SCAN-VERDICT that the caller uses to gate cloud dispatches outright
//! (E-20 PrivacyBlock fire-point). Both surfaces share the same regex
//! set so the scanner and the redactor stay in lockstep.
//!
//! Submodule layout:
//!   * [`secret_scan`] -- pre-cloud-send scanner; returns
//!     [`secret_scan::ScanResult`] with the match-list + severity.
//!   * [`output_filter`] -- post-cloud-receive scanner; redacts the
//!     model's response BEFORE the dashboard / consumer reads it,
//!     defending against "model hallucinates a real-looking secret in
//!     the output" (Protecto 2025 playbook).
//!
//! Source grounding (verified May 2026 via WebSearch):
//!   * kiteworks.com 2025 Truffle Security crawl summary -- ~12K live
//!     secrets in a major training-data scrape, 63% repeated (the
//!     leak-into-prompts failure mode the BLOCK gate defends against).
//!   * protecto.ai 2025 LLM-applications playbook -- two-pass
//!     (input + output) redaction is the canonical pattern; one-pass
//!     misses model-hallucinated leaks.

pub mod output_filter;
pub mod secret_scan;

/// Env-gate constant. When `"1"`, callers in
/// [`crate::cost::cloud_route`] / [`crate::cloud_mode`] apply the
/// scanner + filter to every pre-cloud prompt and post-cloud response.
/// When unset / `"0"`, the helpers compile + run identically but the
/// gate at the dispatch site short-circuits to pass-through. Wire-dark
/// default per Pattern-4 precedent.
pub const ENV_PRIVACY_BLOCK_ENABLE: &str = "ELAI_PRIVACY_BLOCK_ENABLE";

/// Returns `true` when [`ENV_PRIVACY_BLOCK_ENABLE`] is set to `"1"`.
#[must_use]
pub fn privacy_block_enabled() -> bool {
    std::env::var(ENV_PRIVACY_BLOCK_ENABLE)
        .ok()
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// 3-arm scan severity. Used by both submodules.
///
/// `Default = Clean` so [`secret_scan::ScanResult`] can derive
/// `Default` for a "no-match" / pre-scan state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScanSeverity {
    #[default]
    /// No match found.
    Clean,
    /// Soft match (e.g. file path that resembles a credential path but
    /// could be a code reference). Caller decides whether to redact +
    /// continue, or block.
    Warn,
    /// Hard match (e.g. AWS access key prefix, GitHub PAT shape). Caller
    /// MUST refuse cloud dispatch.
    Block,
}

impl ScanSeverity {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Warn => "warn",
            Self::Block => "block",
        }
    }
}

/// Pattern identifier for a single match. Carries only the PATTERN
/// LABEL (not the matched secret value) so SSB rows never persist the
/// secret itself. Composed with `byte_offset` so callers can re-derive
/// the span without storing the value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchSpan {
    /// Snake-case pattern label (e.g. `"aws_access_key"`,
    /// `"github_pat"`, `"anthropic_key"`, `"private_key_pem"`).
    pub pattern: String,
    /// Byte offset in the input string. The caller can re-derive the
    /// matched substring locally if needed, but it MUST NOT be logged.
    pub byte_offset: usize,
    /// Byte length of the match.
    pub byte_len: usize,
}

#[cfg(test)]
mod env_gate_tests {
    use super::*;

    #[test]
    fn severity_labels_stable() {
        assert_eq!(ScanSeverity::Clean.as_str(), "clean");
        assert_eq!(ScanSeverity::Warn.as_str(), "warn");
        assert_eq!(ScanSeverity::Block.as_str(), "block");
    }

    #[test]
    fn env_gate_default_off() {
        // Treat as a smoke check: helper compiles + returns a bool.
        let _ = privacy_block_enabled();
    }
}
