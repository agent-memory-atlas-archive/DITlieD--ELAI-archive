//! E-20 task 5 -- output-side privacy filter contract.
//!
//! Cloud responses are scanned BEFORE the dashboard / consumer reads
//! them. Even when input is clean the model can hallucinate-and-emit a
//! real-looking credential shape; the second pass catches it.
//!
//! Anchor: protecto.ai 2025 LLM-applications playbook (pre + post
//! model redaction). See workflow/reasoning/E-20-decisions.md
//! DECISION-7.

use elai_orchestrator::privacy::output_filter::scan_output_for_secrets;
use elai_orchestrator::privacy::ScanSeverity;

const REDACTED_TOKEN: &str = "[REDACTED_OUTPUT_TOKEN]";

#[test]
fn clean_output_unchanged() {
    let result = scan_output_for_secrets("The function returns 42 on success.", false);
    assert_eq!(result.redaction_count, 0);
    assert_eq!(result.original_severity, ScanSeverity::Clean);
    assert_eq!(result.redacted.as_ref(), "The function returns 42 on success.");
}

#[test]
fn fake_aws_key_in_output_is_redacted() {
    let cloud_response = "Use AKIA1234567890ABCDEF for S3 ops";
    let result = scan_output_for_secrets(cloud_response, false);
    assert_eq!(result.redaction_count, 1);
    assert!(!result.redacted.contains("AKIA1234567890ABCDEF"));
    assert!(result.redacted.contains(REDACTED_TOKEN));
}

#[test]
fn redaction_count_matches_match_count() {
    let cloud_response =
        "First AKIA1234567890ABCDEF then ghp_abcdefghijklmnopqrstuvwx0123";
    let result = scan_output_for_secrets(cloud_response, false);
    assert_eq!(result.redaction_count, 2);
}

#[test]
fn warn_tier_not_redacted_by_default() {
    let cloud_response = "Config typically lives at ~/.aws/credentials";
    let result = scan_output_for_secrets(cloud_response, false);
    assert_eq!(result.redaction_count, 0);
    assert!(result.redacted.contains("~/.aws/credentials"));
}

#[test]
fn warn_tier_redacted_under_strict_mode() {
    let cloud_response = "Config typically lives at ~/.aws/credentials";
    let result = scan_output_for_secrets(cloud_response, true);
    assert_eq!(result.redaction_count, 1);
    assert!(!result.redacted.contains("~/.aws/credentials"));
}

#[test]
fn pattern_labels_propagated_to_result() {
    let cloud_response = "leak ghp_abcdefghijklmnopqrstuvwx0123 found";
    let result = scan_output_for_secrets(cloud_response, false);
    assert!(result.pattern_labels.contains(&"github_pat".to_string()));
}

#[test]
fn redaction_is_idempotent() {
    let first = scan_output_for_secrets("AKIA1234567890ABCDEF leaked", false);
    let second = scan_output_for_secrets(first.redacted.as_ref(), false);
    assert_eq!(second.redaction_count, 0);
}
