//! E-41: small wall-clock helper shared across the github_pr submodules.
//!
//! Centralising the `SystemTime::now() -> i64 ms` conversion lets the new
//! E-41 modules (review, comment_watcher, package_verify, semgrep_gate,
//! auto_merge_guard, codeowners_gen, git_assist) emit consistent
//! `ts_ms` fields in their SSB payloads without each module re-deriving
//! the saturating semantics.
//!
//! Sibling precedent: `GhCliAdapter::now_ms` (gh_cli.rs:87). This helper
//! has the same saturating fall-back semantics so a pre-1970 system
//! clock cannot panic the adapter.

use std::time::{SystemTime, UNIX_EPOCH};

/// Wall-clock milliseconds since the UNIX epoch, saturating to 0 when
/// the clock is before 1970 (sibling fallback shape from
/// `GhCliAdapter::now_ms`).
#[must_use]
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| {
            let ms = d.as_millis();
            i64::try_from(ms).unwrap_or(i64::MAX)
        })
        .unwrap_or(0)
}
