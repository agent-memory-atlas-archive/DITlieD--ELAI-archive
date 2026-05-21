//! elai-safety: Non-LLM destructive-verb allowlist + trust-dialog ordering.
//!
//! E-30: Ships the top-priority public-agent-incident mitigations:
//! (1) argv-regex allowlist at T1 (<1ms) before PreToolUse hook fires.
//! (2) Trust-dialog-before-side-effect ordering (CVE-2025-59536 / CVE-2026-21852).

pub mod destructive_verb_allowlist;
pub mod trust_dialog;

pub use destructive_verb_allowlist::{check_argv, AllowlistVerdict, argv_blake3};
pub use trust_dialog::{ensure_session_trust, mint_trust_token, TrustToken, TrustDialogError};
