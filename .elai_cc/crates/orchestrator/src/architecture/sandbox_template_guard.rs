//! E-25 Tranche C (C.1.c) -- SandboxTemplateGuard.
//!
//! Detects when a skill-template `` `!cmd` `` block is about to run
//! WITHOUT a sandbox layer in place (i.e. the `microsandbox` crate is
//! unavailable). The guard emits one
//! `ClaudeCodeArchitectureEvent` (kind = `SandboxTemplateWarned`) and
//! returns a `Warn` verdict so the caller falls back to the
//! `.elai_cc/.elai/state/skill_template_jail/` process-jail.
//!
//! Trigger site: `crate::skill_templating::expand_skill_template`
//! pre-shell-execute branch (the substrate is shipped at
//! `crates/orchestrator/src/skill_templating.rs:144` but the
//! microsandbox-presence check today is always FALSE because the
//! `microsandbox` crate is not yet on disk -- E-22 Task 2 dep). The
//! guard always-warns in the current state, which matches the plan
//! intent until E-22 lands.
//!
//! Pattern: stateless per-process; matches
//! [`crate::architecture::bash_complexity_guard::BashComplexityGuard`]
//! shape.
//!
//! Emission: reuses the SHIPPED
//! [`crate::architecture::claude_code_arch::ClaudeCodeArchitectureEventKind::SandboxTemplateWarned`]
//! arm. ZERO new SsbActionType. ZERO schema bump.

use crate::architecture::claude_code_arch::{
    ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
    ClaudeCodeArchitectureEventPayload,
};

/// Environment-variable name controlling the wire-dark gate. Set to a
/// truthy value (`"1"` / `"true"` / `"yes"` / `"on"`) to enable.
/// Default OFF.
pub const ELAI_E25_SANDBOX_TEMPLATE_GUARD_ENABLE_VAR: &str =
    "ELAI_E25_SANDBOX_TEMPLATE_GUARD_ENABLE";

/// Canonical jail path used when microsandbox is unavailable. Stored
/// as a named constant so the path is a single source of truth
/// (matches plan's stub_ban list).
pub const SKILL_TEMPLATE_JAIL_PATH: &str = ".elai_cc/.elai/state/skill_template_jail";

/// Verdict returned by [`SandboxTemplateGuard::check`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxTemplateVerdict {
    /// microsandbox is available; safe to dispatch.
    Allow,
    /// microsandbox is unavailable. Caller MUST apply the jail fallback
    /// at [`SKILL_TEMPLATE_JAIL_PATH`] and emit the
    /// `SandboxTemplateWarned` row.
    Refused {
        reason: String,
        jail_path: &'static str,
    },
}

impl SandboxTemplateVerdict {
    #[must_use]
    pub fn is_refused(&self) -> bool {
        matches!(self, Self::Refused { .. })
    }
}

/// Returns true iff the SandboxTemplateGuard is wire-enabled.
#[must_use]
pub fn sandbox_template_guard_enabled() -> bool {
    matches_truthy_env(ELAI_E25_SANDBOX_TEMPLATE_GUARD_ENABLE_VAR)
}

fn matches_truthy_env(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => {
            let trimmed = v.trim();
            trimmed.eq_ignore_ascii_case("1")
                || trimmed.eq_ignore_ascii_case("true")
                || trimmed.eq_ignore_ascii_case("yes")
                || trimmed.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

/// Returns true iff the `microsandbox` crate is available in this
/// build. Today this is always FALSE -- the crate is not yet on disk
/// per E-22 Task 2. Wrapped as a function so a future feature-flag
/// build can replace the body without touching call sites.
#[must_use]
pub fn microsandbox_available() -> bool {
    // E-22 Task 2 dep -- the microsandbox crate is not yet present.
    // Once E-22 lands, swap this body for a real cfg-feature probe.
    false
}

/// Stateless per-process guard. Mirrors
/// [`crate::architecture::bash_complexity_guard::BashComplexityGuard`].
#[derive(Debug, Clone, Copy, Default)]
pub struct SandboxTemplateGuard;

impl SandboxTemplateGuard {
    /// Construct a new guard. Equivalent to `Self::default()`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Evaluate sandbox availability. Returns
    /// [`SandboxTemplateVerdict::Allow`] when microsandbox is on disk;
    /// [`SandboxTemplateVerdict::Refused`] with the jail-path fallback
    /// otherwise.
    #[must_use]
    pub fn check(&self) -> SandboxTemplateVerdict {
        if microsandbox_available() {
            SandboxTemplateVerdict::Allow
        } else {
            SandboxTemplateVerdict::Refused {
                reason:
                    "sandbox_template: microsandbox crate unavailable; applying process-jail fallback"
                        .to_owned(),
                jail_path: SKILL_TEMPLATE_JAIL_PATH,
            }
        }
    }

    /// Build the SSB payload for a refusal verdict. Returns `None` for
    /// `Allow` (no row is emitted on allow per Pattern-2 lean).
    #[must_use]
    pub fn build_payload(
        &self,
        verdict: &SandboxTemplateVerdict,
        template_label: Option<&str>,
        session_id: Option<String>,
        ts_ms: i64,
    ) -> Option<ClaudeCodeArchitectureEventPayload> {
        match verdict {
            SandboxTemplateVerdict::Allow => None,
            SandboxTemplateVerdict::Refused { reason, jail_path } => {
                let details_json = serde_json::json!({
                    "guard_event": "sandbox_template_warned",
                    "reason": reason,
                    "jail_path": jail_path,
                    "template_label": template_label,
                });
                Some(ClaudeCodeArchitectureEventPayload {
                    kind: ClaudeCodeArchitectureEventKind::SandboxTemplateWarned,
                    primitive: ClaudeCodeArchPrimitive::SandboxTemplateGuard,
                    session_id,
                    details_json,
                    ts_ms,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn microsandbox_currently_unavailable() {
        // Documents the current state per E-22 Task 2 dep. When E-22
        // lands and microsandbox_available() returns true, update this
        // test to reflect the new posture.
        assert!(!microsandbox_available());
    }

    #[test]
    fn check_refuses_when_microsandbox_absent() {
        let g = SandboxTemplateGuard::new();
        let v = g.check();
        assert!(v.is_refused());
        match v {
            SandboxTemplateVerdict::Refused { jail_path, .. } => {
                assert_eq!(jail_path, SKILL_TEMPLATE_JAIL_PATH);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn build_payload_returns_none_on_allow() {
        let g = SandboxTemplateGuard::new();
        let v = SandboxTemplateVerdict::Allow;
        assert!(g.build_payload(&v, Some("test"), None, 0).is_none());
    }

    #[test]
    fn build_payload_carries_kind_and_primitive_on_refused() {
        let g = SandboxTemplateGuard::new();
        let v = g.check();
        let payload = g
            .build_payload(&v, Some("test-template"), Some("s-test".into()), 0)
            .expect("refused payload");
        assert_eq!(
            payload.kind,
            ClaudeCodeArchitectureEventKind::SandboxTemplateWarned
        );
        assert_eq!(
            payload.primitive,
            ClaudeCodeArchPrimitive::SandboxTemplateGuard
        );
        assert_eq!(
            payload
                .details_json
                .get("jail_path")
                .and_then(|v| v.as_str()),
            Some(SKILL_TEMPLATE_JAIL_PATH)
        );
    }
}
