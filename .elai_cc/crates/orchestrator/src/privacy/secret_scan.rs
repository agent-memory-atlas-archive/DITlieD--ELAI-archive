//! E-20 -- pre-cloud-send secret scanner.
//!
//! Sits alongside [`crate::privacy_redactor`] but returns a SCAN
//! VERDICT instead of a redacted string. The scanner's match-list +
//! severity decide whether the caller refuses cloud dispatch
//! (PrivacyBlock fire-point of E-20).
//!
//! The scanner is INTENTIONALLY conservative -- a false positive that
//! refuses cloud dispatch is recoverable; a false negative that ships
//! a real secret to a third-party LLM is not. The
//! [`scan_prompt_for_secrets`] fn applies hard-block patterns first
//! (AWS / GitHub / Anthropic / private-key shapes), then soft-warn
//! patterns (denylisted paths, env-var sigils). Callers in
//! [`crate::cloud_mode::CloudModeRouter`] interpret `Block` as "refuse
//! dispatch" and `Warn` as "log + continue".
//!
//! Source grounding (May 2026):
//!   * kiteworks.com Truffle Security crawl summary -- regex shapes
//!     for AWS access keys, GitHub PATs, Anthropic keys, OpenAI keys
//!     are the dominant leak modes in training-data scrapes.
//!   * protecto.ai LLM-applications playbook -- denylist common
//!     credential paths in addition to value shapes.

use std::sync::OnceLock;

use regex::Regex;

use super::{MatchSpan, ScanSeverity};

/// Composite result returned by [`scan_prompt_for_secrets`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScanResult {
    /// Match-list. Each span carries the pattern label + byte offsets
    /// (NOT the matched value -- the value never lands in SSB).
    pub matches: Vec<MatchSpan>,
    /// Overall severity: `Block` overrides `Warn` overrides `Clean`.
    pub severity: ScanSeverity,
}

impl ScanResult {
    /// Returns true when the caller should refuse cloud dispatch.
    #[must_use]
    pub fn is_block(&self) -> bool {
        matches!(self.severity, ScanSeverity::Block)
    }

    /// Number of distinct pattern matches (regardless of severity).
    #[must_use]
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Distinct pattern labels in deterministic order. Used by the
    /// `LocalCloudRoutingEventPayload.details_json.patterns` field.
    #[must_use]
    pub fn pattern_labels(&self) -> Vec<String> {
        let mut labels: Vec<String> = self.matches.iter().map(|m| m.pattern.clone()).collect();
        labels.sort();
        labels.dedup();
        labels
    }
}

// ---------------------------------------------------------------------------
// Block-severity patterns (hard match -> refuse cloud dispatch).
// ---------------------------------------------------------------------------

fn aws_access_key_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // AKIA = root/IAM-user keys; ASIA = STS session keys.
        Regex::new(r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b")
            .expect("static aws access key regex must compile")
    })
}

fn aws_secret_key_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // 40-char base64-ish secret. Conservative; requires the literal
        // assignment prefix to avoid pure-base64-text false positives.
        Regex::new(r#"(?i)aws_secret_access_key\s*[:=]\s*['"]?[A-Za-z0-9/+=]{40}['"]?"#)
            .expect("static aws secret regex must compile")
    })
}

fn github_pat_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b(?:ghp_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,})\b")
            .expect("static github pat regex must compile")
    })
}

fn anthropic_key_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\bsk-ant-[A-Za-z0-9_\-]{16,}\b").expect("static anthropic regex must compile")
    })
}

fn openai_key_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\bsk-[A-Za-z0-9]{20,}\b").expect("static openai regex must compile")
    })
}

fn private_key_pem_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"-----BEGIN (?:RSA |EC |OPENSSH |DSA |PGP )?PRIVATE KEY-----")
            .expect("static private key regex must compile")
    })
}

fn jwt_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // 3-segment base64url JWT (alg.payload.sig) -- header always
        // starts with eyJ (base64 of '{"').
        Regex::new(r"\beyJ[A-Za-z0-9_\-]{10,}\.eyJ[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,}\b")
            .expect("static jwt regex must compile")
    })
}

// ---------------------------------------------------------------------------
// Warn-severity patterns (soft match -> log + continue, caller decides).
// ---------------------------------------------------------------------------

fn denylist_path_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Common credential paths. Restricted to common roots so we do
        // not flag every URL.
        Regex::new(
            r"(?:\.env(?:\.[A-Za-z0-9_]+)?|~/\.ssh/|/Users/[^/\s]+/\.ssh/|~/\.aws/credentials|secrets/[A-Za-z0-9_.\-]+\.ya?ml)",
        )
        .expect("static denylist path regex must compile")
    })
}

fn env_var_sigil_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Catches "${SECRET}" + "$ENV_VAR" forms that often leak intent.
        Regex::new(r"\$(?:\{[A-Z_][A-Z0-9_]{2,}\}|[A-Z_][A-Z0-9_]{2,})")
            .expect("static env var regex must compile")
    })
}

fn collect_block(re: &Regex, pattern: &'static str, text: &str, out: &mut Vec<MatchSpan>) {
    for m in re.find_iter(text) {
        out.push(MatchSpan {
            pattern: pattern.to_owned(),
            byte_offset: m.start(),
            byte_len: m.end() - m.start(),
        });
    }
}

/// Scan `text` for secret-shaped patterns. Returns the canonical
/// [`ScanResult`] with severity = max(individual-match-severity).
///
/// The fn is allocation-light: a `Vec<MatchSpan>` is the only
/// allocation per call (no full copy of `text`).
#[must_use]
pub fn scan_prompt_for_secrets(text: &str) -> ScanResult {
    let mut matches: Vec<MatchSpan> = Vec::new();

    // Block tier.
    collect_block(aws_access_key_re(), "aws_access_key", text, &mut matches);
    collect_block(aws_secret_key_re(), "aws_secret_key", text, &mut matches);
    collect_block(github_pat_re(), "github_pat", text, &mut matches);
    collect_block(anthropic_key_re(), "anthropic_key", text, &mut matches);
    collect_block(openai_key_re(), "openai_key", text, &mut matches);
    collect_block(private_key_pem_re(), "private_key_pem", text, &mut matches);
    collect_block(jwt_re(), "jwt", text, &mut matches);
    let has_block = !matches.is_empty();
    let block_count = matches.len();

    // Warn tier.
    collect_block(denylist_path_re(), "denylist_path", text, &mut matches);
    collect_block(env_var_sigil_re(), "env_var_sigil", text, &mut matches);
    let has_warn = matches.len() > block_count;

    let severity = if has_block {
        ScanSeverity::Block
    } else if has_warn {
        ScanSeverity::Warn
    } else {
        ScanSeverity::Clean
    };
    ScanResult { matches, severity }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_text_is_clean() {
        let result = scan_prompt_for_secrets("Please refactor the calculate_burn function");
        assert_eq!(result.severity, ScanSeverity::Clean);
        assert_eq!(result.match_count(), 0);
        assert!(!result.is_block());
    }

    #[test]
    fn aws_access_key_is_block() {
        let result = scan_prompt_for_secrets("AKIA1234567890ABCDEF is my key");
        assert_eq!(result.severity, ScanSeverity::Block);
        assert!(result.is_block());
        assert!(result.pattern_labels().contains(&"aws_access_key".to_string()));
    }

    #[test]
    fn aws_sts_session_key_is_block() {
        let result = scan_prompt_for_secrets("ASIA1234567890ABCDEF rotated session");
        assert!(result.is_block());
    }

    #[test]
    fn github_pat_is_block() {
        let result = scan_prompt_for_secrets("token ghp_abcdefghijklmnopqrstuvwx0123 expired");
        assert!(result.is_block());
        assert!(result.pattern_labels().contains(&"github_pat".to_string()));
    }

    #[test]
    fn github_classic_pat_is_block() {
        let result = scan_prompt_for_secrets(
            "auth github_pat_11ABCDEFG0abcdefg_abcdefghijklmnopqr0123456789ABCDEFGH ok",
        );
        assert!(result.is_block());
    }

    #[test]
    fn anthropic_key_is_block() {
        let result = scan_prompt_for_secrets("use sk-ant-AbCdEfGhIjKlMnOpQrStUv01234 for inference");
        assert!(result.is_block());
        assert!(result.pattern_labels().contains(&"anthropic_key".to_string()));
    }

    #[test]
    fn openai_key_is_block() {
        let result = scan_prompt_for_secrets("ai sk-AbCdEfGhIjKlMnOpQrStUv01234 fallback");
        assert!(result.is_block());
    }

    #[test]
    fn private_key_pem_is_block() {
        let result =
            scan_prompt_for_secrets("here is the\n-----BEGIN RSA PRIVATE KEY-----\nbase64\n");
        assert!(result.is_block());
        assert!(result.pattern_labels().contains(&"private_key_pem".to_string()));
    }

    #[test]
    fn jwt_is_block() {
        let result = scan_prompt_for_secrets(
            "session eyJabc1234567890.eyJsubdataaaaaaa.signaturedataaaaaaaa expired",
        );
        assert!(result.is_block());
    }

    #[test]
    fn denylist_path_only_is_warn() {
        let result = scan_prompt_for_secrets("Check ~/.aws/credentials for the value");
        assert_eq!(result.severity, ScanSeverity::Warn);
        assert!(!result.is_block());
        assert!(result.pattern_labels().contains(&"denylist_path".to_string()));
    }

    #[test]
    fn env_var_sigil_only_is_warn() {
        let result = scan_prompt_for_secrets("Use $SECRET_PASSWORD when calling");
        assert_eq!(result.severity, ScanSeverity::Warn);
        assert!(result.pattern_labels().contains(&"env_var_sigil".to_string()));
    }

    #[test]
    fn env_var_sigil_curly_is_warn() {
        let result = scan_prompt_for_secrets("Inject ${SECRET_PASSWORD}");
        assert_eq!(result.severity, ScanSeverity::Warn);
    }

    #[test]
    fn block_overrides_warn() {
        let result = scan_prompt_for_secrets(
            "Read .env then use AKIA1234567890ABCDEF in the request",
        );
        // Both denylist_path (warn) AND aws_access_key (block) present.
        assert!(result.is_block());
        assert!(result.match_count() >= 2);
    }

    #[test]
    fn pattern_labels_sorted_and_dedup() {
        let result = scan_prompt_for_secrets(
            "AKIA1234567890ABCDEF and AKIA0987654321ABCDEF both seen",
        );
        let labels = result.pattern_labels();
        // 2 matches, 1 label after dedup.
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0], "aws_access_key");
    }
}
