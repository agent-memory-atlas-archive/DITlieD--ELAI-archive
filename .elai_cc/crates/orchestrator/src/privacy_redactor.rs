//! B-27: pre-cloud-send privacy redactor.
//!
//! Wraps a `BabyStepExecutor` so cloud-bound text is scrubbed of
//! pre-defined PII patterns BEFORE the request leaves the local process
//! (D-3 redaction pipeline). The local sidecar path bypasses this — only
//! `cloud_mode.rs` installs the wrapping. Spec source: cloud-mode-design
//! txt items 11-13.
//!
//! Profiles:
//! - [`RedactionProfile::Minimal`]: pass-through. Use only when the
//!   downstream model is on infrastructure under your control.
//! - [`RedactionProfile::Standard`]: strips email, IPv4, common token
//!   shapes. Default for cloud requests where the user has not opted in
//!   to a stricter profile.
//! - [`RedactionProfile::Strict`]: standard + Windows/POSIX absolute
//!   paths and the user's home dir on Windows. Use when sending code
//!   context that may include local file references.
//!
//! Determinism:
//! - All patterns are precompiled via [`once_cell`-style `OnceLock`] so
//!   redaction is allocation-free per call after the first.
//! - Replacement strings are stable named placeholders (e.g.
//!   `[REDACTED_EMAIL]`) — downstream consumers can pattern-match them
//!   if they need to count redactions.

use std::sync::OnceLock;

use regex::Regex;

/// Configuration for a single redaction call. Held by `cloud_mode.rs`
/// inside its `CloudConfig` so the profile is read from `elai_cc.toml`
/// `[cloud.privacy]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivacyConfig {
    pub profile: RedactionProfile,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            profile: RedactionProfile::Standard,
        }
    }
}

/// Selectable redaction strength. Read from `elai_cc.toml` `[cloud.privacy].profile`.
/// Adding a profile means adding the matching arm in [`redact_context`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactionProfile {
    Minimal,
    Standard,
    Strict,
}

impl RedactionProfile {
    /// Parse a profile name as it would appear in `elai_cc.toml`.
    /// Unknown values fall back to `Standard` so a typo never produces
    /// the silently dangerous `Minimal` profile.
    #[must_use]
    pub fn from_config_str(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "minimal" => Self::Minimal,
            "strict" => Self::Strict,
            // Default for typos / unknown / "standard".
            _ => Self::Standard,
        }
    }
}

const REDACTED_EMAIL: &str = "[REDACTED_EMAIL]";
const REDACTED_IPV4: &str = "[REDACTED_IPV4]";
const REDACTED_TOKEN: &str = "[REDACTED_TOKEN]";
const REDACTED_PATH: &str = "[REDACTED_PATH]";

fn email_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}")
            .expect("static email regex must compile")
    })
}

fn ipv4_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b")
            .expect("static ipv4 regex must compile")
    })
}

fn token_re() -> &'static Regex {
    // Matches Anthropic / OpenAI / GitHub-style token prefixes that a
    // careless paste could ship to the cloud. Conservative — false
    // positives are preferable to leaks.
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b(?:sk-ant-[A-Za-z0-9_\-]{16,}|sk-[A-Za-z0-9]{20,}|ghp_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,})\b")
            .expect("static token regex must compile")
    })
}

fn windows_path_re() -> &'static Regex {
    // Matches typical Windows absolute paths (drive letter + colon + slashes).
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"[A-Za-z]:[\\/](?:[A-Za-z0-9_.\-]+[\\/])*[A-Za-z0-9_.\-]+")
            .expect("static windows path regex must compile")
    })
}

fn posix_path_re() -> &'static Regex {
    // Matches typical POSIX absolute paths under /home, /Users, /var, /etc.
    // Restricted to common roots to limit false positives on URL paths.
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"/(?:home|Users|var|etc|opt|srv)/[A-Za-z0-9_.\-/]+")
            .expect("static posix path regex must compile")
    })
}

/// Apply the configured redaction profile to a piece of context text.
/// Idempotent and allocation-light: returns `Cow<str>` semantics by
/// always returning a fresh `String` for simplicity (typical baby-step
/// payloads are 4-32 KB).
#[must_use]
pub fn redact_context(input: &str, config: &PrivacyConfig) -> String {
    match config.profile {
        RedactionProfile::Minimal => input.to_owned(),
        RedactionProfile::Standard => apply_standard(input),
        RedactionProfile::Strict => apply_strict(input),
    }
}

fn apply_standard(input: &str) -> String {
    let after_email = email_re().replace_all(input, REDACTED_EMAIL);
    let after_ipv4 = ipv4_re().replace_all(&after_email, REDACTED_IPV4);
    let after_token = token_re().replace_all(&after_ipv4, REDACTED_TOKEN);
    after_token.into_owned()
}

fn apply_strict(input: &str) -> String {
    let standard = apply_standard(input);
    let after_windows = windows_path_re().replace_all(&standard, REDACTED_PATH);
    let after_posix = posix_path_re().replace_all(&after_windows, REDACTED_PATH);
    after_posix.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_minimal_profile_is_passthrough() {
        let cfg = PrivacyConfig {
            profile: RedactionProfile::Minimal,
        };
        let input = "Send email to user@example.com";
        let out = redact_context(input, &cfg);
        assert_eq!(out, input);
    }

    #[test]
    fn redact_standard_profile_removes_email() {
        let cfg = PrivacyConfig {
            profile: RedactionProfile::Standard,
        };
        let out = redact_context("ping user@example.com about it", &cfg);
        assert!(!out.contains("user@example.com"));
        assert!(out.contains(REDACTED_EMAIL));
    }

    #[test]
    fn redact_standard_profile_removes_ipv4() {
        let cfg = PrivacyConfig {
            profile: RedactionProfile::Standard,
        };
        let out = redact_context("Connect to 192.168.1.42 quickly", &cfg);
        assert!(!out.contains("192.168.1.42"));
        assert!(out.contains(REDACTED_IPV4));
    }

    #[test]
    fn redact_standard_profile_removes_anthropic_key() {
        let cfg = PrivacyConfig {
            profile: RedactionProfile::Standard,
        };
        let out = redact_context(
            "use sk-ant-AbCdEfGhIjKlMnOpQrStUv01234 for the call",
            &cfg,
        );
        assert!(!out.contains("sk-ant-"));
        assert!(out.contains(REDACTED_TOKEN));
    }

    #[test]
    fn redact_strict_profile_removes_windows_path() {
        let cfg = PrivacyConfig {
            profile: RedactionProfile::Strict,
        };
        let out = redact_context("Read C:\\Users\\Alice\\secrets.txt please", &cfg);
        assert!(!out.contains("Alice"));
        assert!(!out.contains("secrets.txt"));
        assert!(out.contains(REDACTED_PATH));
    }

    #[test]
    fn redact_strict_profile_removes_posix_path() {
        let cfg = PrivacyConfig {
            profile: RedactionProfile::Strict,
        };
        let out = redact_context("cat /home/alice/.ssh/id_rsa", &cfg);
        assert!(!out.contains("alice"));
        assert!(out.contains(REDACTED_PATH));
    }

    #[test]
    fn redact_standard_profile_keeps_paths() {
        // Standard does NOT redact paths; that's a Strict-only behavior.
        let cfg = PrivacyConfig {
            profile: RedactionProfile::Standard,
        };
        let out = redact_context("see C:\\Users\\Alice\\repo\\file.rs", &cfg);
        assert!(out.contains("Alice"));
    }

    #[test]
    fn profile_from_config_str_unknown_falls_back_to_standard() {
        assert_eq!(
            RedactionProfile::from_config_str("paranoid"),
            RedactionProfile::Standard
        );
    }

    #[test]
    fn profile_from_config_str_case_insensitive() {
        assert_eq!(
            RedactionProfile::from_config_str("STRICT"),
            RedactionProfile::Strict
        );
        assert_eq!(
            RedactionProfile::from_config_str("Minimal"),
            RedactionProfile::Minimal
        );
    }
}
