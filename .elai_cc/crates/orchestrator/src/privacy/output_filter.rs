//! E-20 -- post-cloud-receive secret scan + redact.
//!
//! Even when the input scan is clean (i.e. the prompt did not leak a
//! secret) the cloud model can hallucinate-and-emit a real-looking
//! credential shape in its response. The Protecto 2025 LLM-applications
//! playbook calls out two-pass (input AND output) scanning as the
//! canonical pattern; one-pass scanning misses this leak.
//!
//! This module performs the second pass on the cloud response BEFORE
//! the dashboard / consumer reads it. Returned [`FilterResult`] carries
//! the redacted text + the redaction count + the matched pattern labels
//! so the dashboard can render "we redacted 2 patterns from this
//! response" without storing the secret values.

use std::borrow::Cow;

use super::secret_scan::{scan_prompt_for_secrets, ScanResult};
use super::ScanSeverity;

/// Replacement strings. Mirrors
/// [`crate::privacy_redactor`] B-27 placeholders so downstream
/// consumers can pattern-match either source identically.
const REDACTED_TOKEN: &str = "[REDACTED_OUTPUT_TOKEN]";

/// Output of the post-cloud filter pass.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterResult<'a> {
    /// Redacted response text. Borrowed when no redactions were
    /// applied; owned otherwise.
    pub redacted: Cow<'a, str>,
    /// Number of distinct redaction spans applied.
    pub redaction_count: usize,
    /// Pattern labels seen in the response (deterministic order). Same
    /// shape as [`ScanResult::pattern_labels`].
    pub pattern_labels: Vec<String>,
    /// Echo of the scan severity (Clean / Warn / Block) before
    /// redaction. Block-tier matches are always redacted; Warn-tier
    /// matches pass through unchanged (the caller may still surface
    /// them to the user).
    pub original_severity: ScanSeverity,
}

/// Scan + (conditionally) redact a cloud response.
///
/// `redact_warn_tier` controls whether Warn-tier matches (denylist
/// paths / env-var sigils) are redacted. Default callers pass `false`
/// because Warn-tier matches in OUTPUT are often legitimate references
/// (e.g. the model citing a real file path) and over-redaction degrades
/// readability. Pass `true` for ultra-conservative output filtering.
#[must_use]
pub fn scan_output_for_secrets<'a>(
    cloud_response: &'a str,
    redact_warn_tier: bool,
) -> FilterResult<'a> {
    let scan: ScanResult = scan_prompt_for_secrets(cloud_response);
    if scan.matches.is_empty() {
        return FilterResult {
            redacted: Cow::Borrowed(cloud_response),
            redaction_count: 0,
            pattern_labels: Vec::new(),
            original_severity: scan.severity,
        };
    }

    // Sort matches by byte_offset desc so we can splice in place
    // without recomputing offsets after each redaction. Filter to the
    // set of spans we want to redact.
    let mut spans = scan.matches.clone();
    if !redact_warn_tier {
        // Drop warn-tier patterns from the redaction set; keep them in
        // pattern_labels so the dashboard still surfaces them.
        spans.retain(|m| !is_warn_tier_pattern(&m.pattern));
    }
    spans.sort_by_key(|m| std::cmp::Reverse(m.byte_offset));

    let mut buf = cloud_response.to_owned();
    let mut redacted_count = 0_usize;
    for m in &spans {
        if m.byte_offset > buf.len() {
            // Defensive: scanner produced an out-of-range span (should
            // never happen because we scanned the same input).
            continue;
        }
        let end = m.byte_offset + m.byte_len;
        if end > buf.len() {
            continue;
        }
        // Safe replace_range across UTF-8 boundary because the regex
        // engine returns char-aligned offsets.
        buf.replace_range(m.byte_offset..end, REDACTED_TOKEN);
        redacted_count += 1;
    }
    FilterResult {
        redacted: Cow::Owned(buf),
        redaction_count: redacted_count,
        pattern_labels: scan.pattern_labels(),
        original_severity: scan.severity,
    }
}

fn is_warn_tier_pattern(label: &str) -> bool {
    matches!(label, "denylist_path" | "env_var_sigil")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_output_passes_through_unredacted() {
        let result = scan_output_for_secrets("the answer is 42", false);
        assert_eq!(result.redaction_count, 0);
        assert_eq!(result.original_severity, ScanSeverity::Clean);
        assert_eq!(result.redacted.as_ref(), "the answer is 42");
    }

    #[test]
    fn aws_key_in_output_is_redacted() {
        let result = scan_output_for_secrets("use AKIA1234567890ABCDEF for s3", false);
        assert_eq!(result.redaction_count, 1);
        assert!(!result.redacted.contains("AKIA1234567890ABCDEF"));
        assert!(result.redacted.contains(REDACTED_TOKEN));
        assert!(result.pattern_labels.contains(&"aws_access_key".to_string()));
    }

    #[test]
    fn warn_tier_default_not_redacted() {
        let result = scan_output_for_secrets("config at ~/.aws/credentials", false);
        // Warn-tier match present but not redacted by default.
        assert_eq!(result.redaction_count, 0);
        assert!(result
            .pattern_labels
            .contains(&"denylist_path".to_string()));
        assert!(result.redacted.contains("~/.aws/credentials"));
    }

    #[test]
    fn warn_tier_redacted_when_opt_in() {
        let result = scan_output_for_secrets("config at ~/.aws/credentials", true);
        assert_eq!(result.redaction_count, 1);
        assert!(!result.redacted.contains("~/.aws/credentials"));
    }

    #[test]
    fn multiple_matches_all_redacted() {
        let result = scan_output_for_secrets(
            "first AKIA1234567890ABCDEF then ghp_abcdefghijklmnopqrstuvwx0123",
            false,
        );
        assert_eq!(result.redaction_count, 2);
        let labels = &result.pattern_labels;
        assert!(labels.contains(&"aws_access_key".to_string()));
        assert!(labels.contains(&"github_pat".to_string()));
    }

    #[test]
    fn block_severity_propagates_to_filter_result() {
        let result = scan_output_for_secrets("key ghp_abcdefghijklmnopqrstuvwx0123", false);
        assert_eq!(result.original_severity, ScanSeverity::Block);
    }

    #[test]
    fn redact_idempotent_no_change_on_second_pass() {
        let first = scan_output_for_secrets("AKIA1234567890ABCDEF leaked", false);
        let second = scan_output_for_secrets(first.redacted.as_ref(), false);
        assert_eq!(second.redaction_count, 0);
        assert_eq!(second.original_severity, ScanSeverity::Clean);
    }
}
