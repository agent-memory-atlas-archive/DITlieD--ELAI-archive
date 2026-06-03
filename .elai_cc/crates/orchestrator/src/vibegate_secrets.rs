//! F-1 VibeGate — secret-pattern regex set.
//!
//! Compiled once via [`std::sync::LazyLock`] so the hot path never pays
//! regex compilation cost. Patterns are high-precision / low-FP: they target
//! well-known credential shapes rather than broad entropy heuristics so the
//! gate does not fire on incidental random strings.
//!
//! Pattern taxonomy:
//!   1. AWS access-key IDs        — `AKIA[0-9A-Z]{16}` (hardcoded format)
//!   2. PEM private-key headers   — `-----BEGIN (RSA|EC|DSA|OPENSSH) PRIVATE KEY-----`
//!   3. Generic private-key header — `-----BEGIN PRIVATE KEY-----`
//!   4. High-entropy secret assignments — `(secret|token|password|api_key)\s*=\s*"[A-Za-z0-9+/=_\-]{20,}"`
//!      (assignment to a string literal of ≥20 chars in typical credential charset)
//!
//! Each pattern index maps to a human-readable label via [`SECRET_LABELS`].
//! Call [`SECRET_PATTERNS`] directly; it is the sole public surface.

use regex::RegexSet;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Pattern strings
// ---------------------------------------------------------------------------

/// Pattern 0: AWS access-key ID. Format is `AKIA` followed by exactly 16
/// uppercase letters/digits — hardcoded by AWS; no false positives on real code.
const PAT_AWS_KEY: &str = r"AKIA[0-9A-Z]{16}";

/// Pattern 1: RSA / EC / DSA / OpenSSH PEM private-key header lines.
const PAT_PEM_TYPED: &str = r"-----BEGIN (RSA|EC|DSA|OPENSSH) PRIVATE KEY-----";

/// Pattern 2: PKCS#8 generic private-key PEM header.
const PAT_PEM_GENERIC: &str = r"-----BEGIN PRIVATE KEY-----";

/// Pattern 3: High-entropy string literal assigned to a secret-named
/// variable. Anchored to `secret`, `token`, `password`, or `api_key`
/// (case-insensitive) followed by `=` then a double-quoted string of ≥ 20
/// credential-charset characters. Matches Python, JS/TS, and Rust literal
/// assignment patterns. Single-quoted strings are excluded to avoid false
/// positives on SQL fragments.
const PAT_SECRET_ASSIGN: &str =
    r#"(?i)(secret|token|password|api_key)\s*=\s*"[A-Za-z0-9+/=_\-]{20,}""#;

// ---------------------------------------------------------------------------
// Compiled set
// ---------------------------------------------------------------------------

/// Compiled [`RegexSet`] of all secret patterns. Initialised once on first
/// access via [`LazyLock`]; `.expect` is acceptable per the CLAUDE.md
/// exception rule (literal pattern at init, panic = "won't start, you'll see
/// it immediately").
// SAFETY: LazyLock infallible regex (CLAUDE.md LazyLock exception) — all
// patterns are literal constants verified at review time.
pub static SECRET_PATTERNS: LazyLock<RegexSet> = LazyLock::new(|| {
    RegexSet::new([PAT_AWS_KEY, PAT_PEM_TYPED, PAT_PEM_GENERIC, PAT_SECRET_ASSIGN])
        .expect("vibegate_secrets: literal regex patterns must compile")
});

/// Human-readable labels for each pattern index. Indexed by the match index
/// returned by `SECRET_PATTERNS.matches(source)`.
pub static SECRET_LABELS: &[&str] = &[
    "aws_access_key_id",
    "pem_private_key_typed",
    "pem_private_key_generic",
    "secret_literal_assignment",
];

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_patterns_compile() {
        let _ = &*SECRET_PATTERNS;
    }

    #[test]
    fn matches_aws_key() {
        let source = "let key = \"AKIAIOSFODNN7EXAMPLE\";";
        assert!(SECRET_PATTERNS.is_match(source), "AWS key must match");
    }

    #[test]
    fn matches_pem_private_key() {
        let source = "-----BEGIN RSA PRIVATE KEY-----\nMIIEow...\n-----END RSA PRIVATE KEY-----";
        assert!(SECRET_PATTERNS.is_match(source), "PEM RSA key must match");
    }

    #[test]
    fn matches_pem_generic() {
        let source = "-----BEGIN PRIVATE KEY-----\nbase64data\n-----END PRIVATE KEY-----";
        assert!(SECRET_PATTERNS.is_match(source), "generic PEM key must match");
    }

    #[test]
    fn matches_secret_assignment() {
        let source = r#"SECRET = "abcdefghij1234567890XY""#;
        assert!(SECRET_PATTERNS.is_match(source), "secret assignment must match");
    }

    #[test]
    fn does_not_match_normal_code() {
        let source = "let result = some_function(x, y); // returns string";
        assert!(!SECRET_PATTERNS.is_match(source), "normal code must not match");
    }

    #[test]
    fn does_not_match_short_assignment() {
        // Fewer than 20 chars in the string value — should NOT fire.
        let source = r#"token = "short_val""#;
        assert!(!SECRET_PATTERNS.is_match(source), "short secret assignment must not match");
    }

    #[test]
    fn label_count_matches_pattern_count() {
        assert_eq!(
            SECRET_LABELS.len(),
            4,
            "SECRET_LABELS must index every pattern"
        );
    }
}
