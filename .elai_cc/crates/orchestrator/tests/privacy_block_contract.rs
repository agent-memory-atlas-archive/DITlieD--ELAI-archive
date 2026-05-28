//! E-20 task 4 -- input-side PrivacyBlock contract.
//!
//! Asserts that the secret-shape scanner blocks the canonical leak
//! patterns (AWS / GitHub / Anthropic / OpenAI / private-key / JWT)
//! AND lets clean prompts through.
//!
//! Anchor: kiteworks.com Truffle Security 2025 crawl summary
//! (~12K live secrets, 63% repeated) + protecto.ai 2025 LLM-
//! applications playbook (two-pass input + output redaction). See
//! workflow/reasoning/E-20-decisions.md DECISION-6.

use elai_orchestrator::privacy::secret_scan::scan_prompt_for_secrets;
use elai_orchestrator::privacy::ScanSeverity;

#[test]
fn aws_access_key_pattern_is_blocked() {
    let result = scan_prompt_for_secrets("AKIA1234567890ABCDEF is my key");
    assert_eq!(result.severity, ScanSeverity::Block);
    assert!(result.pattern_labels().contains(&"aws_access_key".to_string()));
}

#[test]
fn aws_sts_session_key_is_blocked() {
    let result = scan_prompt_for_secrets("session ASIA1234567890ABCDEF rotated");
    assert!(result.is_block());
}

#[test]
fn github_pat_is_blocked() {
    let result = scan_prompt_for_secrets("auth ghp_abcdefghijklmnopqrstuvwx0123 expired");
    assert!(result.is_block());
}

#[test]
fn anthropic_key_is_blocked() {
    let result = scan_prompt_for_secrets("use sk-ant-AbCdEfGhIjKlMnOpQrStUv01234 for inference");
    assert!(result.is_block());
}

#[test]
fn openai_key_is_blocked() {
    let result = scan_prompt_for_secrets("call with sk-AbCdEfGhIjKlMnOpQrStUv01234");
    assert!(result.is_block());
}

#[test]
fn private_key_pem_header_is_blocked() {
    let result = scan_prompt_for_secrets("payload -----BEGIN OPENSSH PRIVATE KEY----- base64");
    assert!(result.is_block());
}

#[test]
fn jwt_pattern_is_blocked() {
    let result = scan_prompt_for_secrets(
        "session eyJabcdefghijk.eyJsubdataaaaa.signaturedataaaaaa expired",
    );
    assert!(result.is_block());
}

#[test]
fn clean_prompt_passes() {
    let result = scan_prompt_for_secrets("Please refactor the burn-rate projection function");
    assert_eq!(result.severity, ScanSeverity::Clean);
    assert!(!result.is_block());
    assert_eq!(result.match_count(), 0);
}

#[test]
fn denylist_path_alone_is_warn_not_block() {
    let result = scan_prompt_for_secrets("Check the value at ~/.aws/credentials");
    assert_eq!(result.severity, ScanSeverity::Warn);
    assert!(!result.is_block());
}

#[test]
fn env_var_sigil_alone_is_warn_not_block() {
    let result = scan_prompt_for_secrets("Inject $SECRET_PASSWORD when calling");
    assert_eq!(result.severity, ScanSeverity::Warn);
}

#[test]
fn block_overrides_warn_when_both_present() {
    let result = scan_prompt_for_secrets(
        "Read .env then use AKIA1234567890ABCDEF in the request",
    );
    assert!(result.is_block());
    assert!(result.match_count() >= 2);
}

#[test]
fn pattern_labels_dedup_across_repeats() {
    let result = scan_prompt_for_secrets(
        "AKIA1234567890ABCDEF and AKIA0987654321ABCDEF",
    );
    assert!(result.is_block());
    let labels = result.pattern_labels();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0], "aws_access_key");
}

#[test]
fn scan_result_carries_byte_offsets_not_values() {
    // Spans MUST carry byte_offset / byte_len so the caller can audit
    // WHERE the leak was without persisting the secret value.
    let result = scan_prompt_for_secrets("prefix AKIA1234567890ABCDEF suffix");
    assert!(result.is_block());
    let span = result
        .matches
        .iter()
        .find(|m| m.pattern == "aws_access_key")
        .expect("aws_access_key span");
    assert_eq!(span.byte_len, 20);
    assert!(span.byte_offset > 0);
}
