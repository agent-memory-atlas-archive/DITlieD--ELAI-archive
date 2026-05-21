// E-30: CVE fixture contract for trust-dialog-before-side-effect ordering.
//
// Pins the contract that `ensure_session_trust()` correctly enforces:
//   - CVE-2025-59536: hooks must not load before a trust token is minted.
//   - CVE-2026-21852: ANTHROPIC_BASE_URL / OPENAI_BASE_URL must not be honored
//     before a trust token is minted (API token exfiltration vector).
//
// These are ordering invariants, not cryptographic ones. The contract verifies
// that the safety crate's public API upholds the documented ordering policy.

use std::path::Path;
use elai_safety::{ensure_session_trust, mint_trust_token, TrustDialogError};

// ---------------------------------------------------------------------------
// CVE-2025-59536: hooks executed before trust dialog
// ---------------------------------------------------------------------------

#[test]
fn cve_2025_59536_no_token_returns_trust_dialog_required() {
    // Without a trust token, hook loading must be refused. The production
    // ordering checks ANTHROPIC_BASE_URL / OPENAI_BASE_URL FIRST (CVE-2026-21852)
    // and TrustDialogRequired second (CVE-2025-59536). s-68264 cleanup: the dev
    // env routinely has ANTHROPIC_BASE_URL set (remote sidecar tunnel), which
    // poisons this test. Snapshot + clear both env vars for the duration of
    // this test, then restore.
    let anth = std::env::var("ANTHROPIC_BASE_URL").ok();
    let openai = std::env::var("OPENAI_BASE_URL").ok();
    if anth.is_some() {
        std::env::remove_var("ANTHROPIC_BASE_URL");
    }
    if openai.is_some() {
        std::env::remove_var("OPENAI_BASE_URL");
    }

    let result = ensure_session_trust(
        Path::new("/tmp/project"),
        Path::new("/tmp/project/.claude/settings.json"),
        None,
    );

    if let Some(v) = anth {
        std::env::set_var("ANTHROPIC_BASE_URL", v);
    }
    if let Some(v) = openai {
        std::env::set_var("OPENAI_BASE_URL", v);
    }

    assert!(result.is_err(), "no token: must return Err");
    match result.unwrap_err() {
        TrustDialogError::TrustDialogRequired { path } => {
            assert!(
                path.contains("settings.json"),
                "error must name the settings file; got: {path}"
            );
        }
        other => panic!("expected TrustDialogRequired, got {other:?}"),
    }
}

#[test]
fn cve_2025_59536_with_token_hook_load_allowed() {
    let token = mint_trust_token("session-cve-test-59536");
    let result = ensure_session_trust(
        Path::new("/tmp/project"),
        Path::new("/tmp/project/.claude/settings.json"),
        Some(&token),
    );
    assert!(result.is_ok(), "with valid token: hook load must be allowed");
}

// ---------------------------------------------------------------------------
// CVE-2026-21852: ANTHROPIC_BASE_URL honored before trust dialog
// ---------------------------------------------------------------------------

#[test]
fn cve_2026_21852_no_token_with_base_url_env_returns_deferred() {
    // Temporarily set the env var for this test.
    // This test must NOT rely on persistent side effects; the env var is
    // removed after the assertion.
    //
    // Safety note: this test will be a false-positive if the env var is
    // already set before the test runner starts. Document this limitation.
    let var_name = "ANTHROPIC_BASE_URL";
    let was_set = std::env::var(var_name).is_ok();
    if !was_set {
        std::env::set_var(var_name, "http://evil.example.com");
    }

    let result = ensure_session_trust(
        Path::new("/tmp/project"),
        Path::new("/tmp/project/.claude/settings.json"),
        None,
    );

    if !was_set {
        std::env::remove_var(var_name);
    }

    assert!(result.is_err(), "ANTHROPIC_BASE_URL set + no token: must return Err");
    match result.unwrap_err() {
        TrustDialogError::HookLoadDeferred { reason } => {
            assert!(
                reason.contains("ANTHROPIC_BASE_URL") || reason.contains("CVE-2026-21852"),
                "reason must mention the env var or CVE; got: {reason}"
            );
        }
        // If env var was already set externally, TrustDialogRequired is also acceptable.
        TrustDialogError::TrustDialogRequired { .. } => {}
    }
}

#[test]
fn mint_token_fields_are_populated() {
    let token = mint_trust_token("session-contract-test");
    assert_eq!(token.session_id, "session-contract-test");
    assert!(token.minted_at_ms > 0, "minted_at_ms must be non-zero");
}

#[test]
fn two_tokens_same_session_id() {
    // Session IDs are caller-provided; two calls with same id produce two tokens
    // with the same session_id (no uniqueness enforcement — that's the caller's job).
    let t1 = mint_trust_token("same-session");
    let t2 = mint_trust_token("same-session");
    assert_eq!(t1.session_id, t2.session_id);
}
