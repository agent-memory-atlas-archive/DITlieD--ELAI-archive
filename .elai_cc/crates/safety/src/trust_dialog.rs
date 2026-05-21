//! E-30: Trust-dialog-before-side-effect ordering.
//!
//! CVE-2025-59536: Claude Code hooks RCE — hooks executed before trust dialog.
//!   Source: research.checkpoint.com/2026/rce-and-api-token-exfiltration-through-claude-code-project-files-cve-2025-59536/
//! CVE-2026-21852: ANTHROPIC_BASE_URL honored before trust dialog — API token exfil.
//!
//! Fix pattern: `ensure_session_trust()` MUST be called before loading any
//! `.claude/settings.json` hooks or honoring `ANTHROPIC_BASE_URL` / `OPENAI_BASE_URL`.
//! Hook trust check runs BEFORE FSM transition checks (CLAUDE.md FSM gate rule).

use std::time::{SystemTime, UNIX_EPOCH};

/// A minted session trust token.
///
/// Produced by [`mint_trust_token`] after the user has issued their first
/// session trust action (typing a prompt counts; flag-passing does NOT).
/// Hooks and env vars are only honored after this token exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustToken {
    pub session_id: String,
    pub minted_at_ms: u64,
}

/// Error returned when trust has not yet been established.
#[derive(Debug, thiserror::Error)]
pub enum TrustDialogError {
    /// Hooks must not load before the user has approved this project.
    /// CVE-2025-59536 pattern: hooks executed before consent.
    #[error("trust dialog required before loading hooks from {path}")]
    TrustDialogRequired { path: String },

    /// An env var would redirect the API endpoint before consent was given.
    /// CVE-2026-21852 pattern: ANTHROPIC_BASE_URL honored before trust dialog.
    #[error("hook load deferred: {reason}")]
    HookLoadDeferred { reason: String },
}

/// Verify that a session trust token exists before allowing hooks or env var
/// side-effects.
///
/// # Arguments
/// - `_project_root`: the project root directory (reserved for future path checks).
/// - `settings_path`: path to `.claude/settings.json` or `.elai_cc/settings.json`.
/// - `token`: the session trust token, if one has been minted.
///
/// # Errors
/// - [`TrustDialogError::HookLoadDeferred`] when `ANTHROPIC_BASE_URL` or
///   `OPENAI_BASE_URL` is set and `token` is `None` (CVE-2026-21852 pattern).
/// - [`TrustDialogError::TrustDialogRequired`] when `token` is `None` (no
///   trust action has been taken yet — CVE-2025-59536 pattern).
pub fn ensure_session_trust(
    _project_root: &std::path::Path,
    settings_path: &std::path::Path,
    token: Option<&TrustToken>,
) -> Result<(), TrustDialogError> {
    // CVE-2026-21852: check env var redirect before token check.
    if token.is_none() {
        let has_anthropic_redirect = std::env::var("ANTHROPIC_BASE_URL").is_ok();
        let has_openai_redirect = std::env::var("OPENAI_BASE_URL").is_ok();
        if has_anthropic_redirect || has_openai_redirect {
            return Err(TrustDialogError::HookLoadDeferred {
                reason: "ANTHROPIC_BASE_URL or OPENAI_BASE_URL is set before trust was \
                         established. This pattern can exfiltrate API tokens \
                         (CVE-2026-21852). Mint a trust token first."
                    .to_string(),
            });
        }
    }

    // CVE-2025-59536: refuse hook load without explicit user trust action.
    match token {
        None => Err(TrustDialogError::TrustDialogRequired {
            path: settings_path.display().to_string(),
        }),
        Some(_) => Ok(()),
    }
}

/// Mint a session trust token.
///
/// Call this after the user has issued their first trust action in this session.
/// The returned token should be stored in session state and passed to
/// [`ensure_session_trust`] on all subsequent hook/env load attempts.
pub fn mint_trust_token(session_id: &str) -> TrustToken {
    let minted_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    TrustToken {
        session_id: session_id.to_string(),
        minted_at_ms,
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn no_token_returns_trust_dialog_required() {
        // CVE-2025-59536 fixture: hooks must not load without a token.
        let result = ensure_session_trust(
            Path::new("/tmp/test-project"),
            Path::new("/tmp/test-project/.claude/settings.json"),
            None,
        );
        assert!(result.is_err(), "must return Err without a trust token");
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
    fn with_token_returns_ok() {
        let token = mint_trust_token("test-session-cve-test");
        let result = ensure_session_trust(
            Path::new("/tmp/test-project"),
            Path::new("/tmp/test-project/.claude/settings.json"),
            Some(&token),
        );
        assert!(result.is_ok(), "must return Ok with a valid trust token");
    }

    #[test]
    fn minted_token_has_correct_session_id() {
        let token = mint_trust_token("session-abc-123");
        assert_eq!(token.session_id, "session-abc-123");
        assert!(token.minted_at_ms > 0, "minted_at_ms must be non-zero");
    }

    #[test]
    fn mint_two_tokens_different_sessions() {
        let t1 = mint_trust_token("session-A");
        let t2 = mint_trust_token("session-B");
        assert_ne!(t1.session_id, t2.session_id);
    }
}
