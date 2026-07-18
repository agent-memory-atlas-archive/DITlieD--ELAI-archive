//! E-37 U1 — output sanitizer contract (OWASP LLM Top 10 LLM01 + LLM06).
//!
//! Three pure-function tests against `elai_safety::output_sanitizer`:
//!   * strip removes a markdown image whose host is not on the allowlist
//!   * strip preserves a markdown image whose host IS on the allowlist
//!   * classify flags a `<script` tag
//!
//! No sidecar, no orchestrator wiring — these fence the safety primitive
//! at the unit boundary. The orchestrator wiring
//! (`conversation::run_post_tool_use_hook` emits
//! `SsbActionType::OutputClassifierFlagged`) is fenced by the orchestrator
//! SSB action-type lock test + the existing E-37 prompt-injection-defense
//! contract G3 round-trip surface.
//!
//! Build target: `cargo test -p elai_cc_contracts --test e37_output_sanitizer_contract`.

use elai_safety::output_sanitizer::{
    classify_output, strip_markdown_images, OutputVerdict,
};

#[test]
fn strip_markdown_images_removes_non_allowlisted_host() {
    let input = "before ![exfil](https://evil.example/x.png) after";
    let sanitized = strip_markdown_images(input, &[]);
    assert_eq!(sanitized.stripped_count, 1);
    assert_eq!(sanitized.stripped_hosts, vec!["evil.example".to_string()]);
    assert!(!sanitized.cleaned.contains("evil.example"));
    assert!(!sanitized.cleaned.contains("https://evil.example/x.png"));
    assert!(sanitized.cleaned.contains("before"));
    assert!(sanitized.cleaned.contains("after"));
}

#[test]
fn strip_markdown_images_preserves_allowlisted_host() {
    let input = "see ![paper](https://arxiv.org/pdf/2507.15219) here";
    let allowlist = vec!["arxiv.org".to_string()];
    let sanitized = strip_markdown_images(input, &allowlist);
    assert_eq!(sanitized.stripped_count, 0);
    assert!(sanitized.stripped_hosts.is_empty());
    assert!(sanitized.cleaned.contains("arxiv.org"));
    assert!(sanitized
        .cleaned
        .contains("https://arxiv.org/pdf/2507.15219"));
}

#[test]
fn classify_output_flags_script_tag() {
    let verdict = classify_output("ok <script>alert(1)</script> tail");
    match verdict {
        OutputVerdict::Flagged { patterns } => {
            assert!(
                patterns.iter().any(|p| p == "<script"),
                "expected <script in flagged patterns: {:?}",
                patterns
            );
        }
        OutputVerdict::Clean => panic!("expected Flagged, got Clean"),
    }
}
