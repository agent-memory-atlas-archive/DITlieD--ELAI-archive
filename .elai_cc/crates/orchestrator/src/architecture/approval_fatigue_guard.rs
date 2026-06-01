//! E-25 Tranche C (C.1.b) -- ApprovalFatigueGuard.
//!
//! Detects approval-prompt fatigue by tracking a rolling window of
//! permission decisions per session. When the auto-approval rate
//! exceeds the configured threshold over the window, the guard emits
//! one `ClaudeCodeArchitectureEvent` (kind = `ApprovalFatigueDetected`)
//! and surfaces a refusal verdict so the calling site can force
//! explicit user reconfirmation on the next prompt.
//!
//! Trigger site: each `PermissionPromptDecision` evaluation at
//! `crates/orchestrator/src/permissions.rs:312` (and the conversation-
//! level dispatch at `conversation.rs:1052`).
//!
//! Pattern mirrors `crates/orchestrator/src/cost/free_tier_guard.rs:103`
//! `FreeTierGuard` per-session shape -- one instance per orchestrator
//! session, lazy reset on session start, snapshot surface for the
//! dashboard.
//!
//! Emission: reuses the SHIPPED
//! [`crate::architecture::claude_code_arch::ClaudeCodeArchitectureEventKind::ApprovalFatigueDetected`]
//! arm. ZERO new SsbActionType. ZERO schema bump.

use std::collections::VecDeque;

use crate::architecture::claude_code_arch::{
    ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
    ClaudeCodeArchitectureEventPayload,
};

/// Environment-variable name controlling the wire-dark gate. Set to a
/// truthy value (`"1"` / `"true"` / `"yes"` / `"on"`) to enable.
/// Default OFF.
pub const ELAI_E25_APPROVAL_FATIGUE_GUARD_ENABLE_VAR: &str =
    "ELAI_E25_APPROVAL_FATIGUE_GUARD_ENABLE";

/// Rolling-window size (decision count). The guard inspects the last
/// `WINDOW_SIZE` decisions to compute the auto-approval rate.
pub const APPROVAL_WINDOW_SIZE: usize = 50;

/// Auto-approval-rate threshold expressed as percentage points.
/// Crossing this fraction over the rolling window is what triggers a
/// `ApprovalFatigueDetected` emission and a `Refuse` verdict.
pub const AUTO_APPROVE_THRESHOLD_PCT: u32 = 80;

/// 2-arm decision discriminator. Mirrors the shape of
/// `crate::permissions::PermissionPromptDecision` but kept local so
/// this guard does not pull a hard dep on the permissions module's
/// public surface (the guard is consumed from BOTH `permissions.rs`
/// and `conversation.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalKind {
    Allow,
    Deny,
}

/// Verdict returned by [`ApprovalFatigueGuard::record_and_check`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalFatigueVerdict {
    /// Window has not exceeded the threshold (or has fewer than the
    /// configured number of samples yet); proceed with the original
    /// decision.
    Allow { observed_rate_pct: u32 },
    /// Threshold tripped. The caller should force a reconfirmation
    /// prompt on the next request and emit the carried payload.
    Refused {
        reason: String,
        observed_rate_pct: u32,
    },
}

impl ApprovalFatigueVerdict {
    #[must_use]
    pub fn is_refused(&self) -> bool {
        matches!(self, Self::Refused { .. })
    }
}

/// Returns true iff the ApprovalFatigueGuard is wire-enabled.
#[must_use]
pub fn approval_fatigue_guard_enabled() -> bool {
    matches_truthy_env(ELAI_E25_APPROVAL_FATIGUE_GUARD_ENABLE_VAR)
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

/// Per-session approval-fatigue guard. Tracks the last
/// [`APPROVAL_WINDOW_SIZE`] decisions and computes the rolling
/// Allow-rate on every observation.
#[derive(Debug, Clone)]
pub struct ApprovalFatigueGuard {
    window: VecDeque<ApprovalKind>,
    capacity: usize,
    threshold_pct: u32,
}

impl Default for ApprovalFatigueGuard {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl ApprovalFatigueGuard {
    /// Construct with the canonical [`APPROVAL_WINDOW_SIZE`] +
    /// [`AUTO_APPROVE_THRESHOLD_PCT`] constants.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self {
            window: VecDeque::with_capacity(APPROVAL_WINDOW_SIZE),
            capacity: APPROVAL_WINDOW_SIZE,
            threshold_pct: AUTO_APPROVE_THRESHOLD_PCT,
        }
    }

    /// Construct with caller-supplied window + threshold. Useful for
    /// tests + future per-session-config overrides.
    #[must_use]
    pub fn with_window(capacity: usize, threshold_pct: u32) -> Self {
        Self {
            window: VecDeque::with_capacity(capacity.max(1)),
            capacity: capacity.max(1),
            threshold_pct: threshold_pct.min(100),
        }
    }

    /// Current observed Allow-rate over the window, as percentage
    /// points. Returns `0` when the window is empty.
    #[must_use]
    pub fn observed_rate_pct(&self) -> u32 {
        if self.window.is_empty() {
            return 0;
        }
        let allows = self
            .window
            .iter()
            .filter(|&&k| k == ApprovalKind::Allow)
            .count();
        let len_u32 = u32::try_from(self.window.len()).unwrap_or(u32::MAX);
        let allows_u32 = u32::try_from(allows).unwrap_or(u32::MAX);
        if len_u32 == 0 {
            return 0;
        }
        allows_u32.saturating_mul(100) / len_u32
    }

    /// Record one decision and return the verdict for the NEW window
    /// state. When the window is shorter than the configured capacity
    /// the verdict is always `Allow` (insufficient samples).
    pub fn record_and_check(&mut self, kind: ApprovalKind) -> ApprovalFatigueVerdict {
        if self.window.len() >= self.capacity {
            self.window.pop_front();
        }
        self.window.push_back(kind);
        let rate = self.observed_rate_pct();
        if self.window.len() < self.capacity {
            return ApprovalFatigueVerdict::Allow {
                observed_rate_pct: rate,
            };
        }
        if rate >= self.threshold_pct {
            ApprovalFatigueVerdict::Refused {
                reason: format!(
                    "approval_fatigue: observed_rate_pct {rate} >= threshold_pct {} over window of {}",
                    self.threshold_pct, self.capacity
                ),
                observed_rate_pct: rate,
            }
        } else {
            ApprovalFatigueVerdict::Allow {
                observed_rate_pct: rate,
            }
        }
    }

    /// Build the SSB payload for a refusal verdict. Returns `None` for
    /// `Allow` (no row is emitted on allow per Pattern-2 lean).
    #[must_use]
    pub fn build_payload(
        &self,
        verdict: &ApprovalFatigueVerdict,
        session_id: Option<String>,
        ts_ms: i64,
    ) -> Option<ClaudeCodeArchitectureEventPayload> {
        match verdict {
            ApprovalFatigueVerdict::Allow { .. } => None,
            ApprovalFatigueVerdict::Refused {
                reason,
                observed_rate_pct,
            } => {
                let details_json = serde_json::json!({
                    "guard_event": "approval_fatigue_detected",
                    "reason": reason,
                    "observed_rate_pct": observed_rate_pct,
                    "threshold_pct": self.threshold_pct,
                    "window_size": self.capacity,
                });
                Some(ClaudeCodeArchitectureEventPayload {
                    kind: ClaudeCodeArchitectureEventKind::ApprovalFatigueDetected,
                    primitive: ClaudeCodeArchPrimitive::ApprovalFatigueGuard,
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
    fn under_window_capacity_always_allow() {
        let mut g = ApprovalFatigueGuard::with_window(10, 80);
        for _ in 0..5 {
            let v = g.record_and_check(ApprovalKind::Allow);
            assert!(matches!(v, ApprovalFatigueVerdict::Allow { .. }));
        }
    }

    #[test]
    fn under_threshold_allows() {
        let mut g = ApprovalFatigueGuard::with_window(10, 80);
        // 7 allows + 3 denies -> 70% allow rate, under 80% threshold.
        for _ in 0..7 {
            g.record_and_check(ApprovalKind::Allow);
        }
        for _ in 0..3 {
            let v = g.record_and_check(ApprovalKind::Deny);
            assert!(matches!(v, ApprovalFatigueVerdict::Allow { .. }));
        }
    }

    #[test]
    fn over_threshold_refuses() {
        let mut g = ApprovalFatigueGuard::with_window(10, 80);
        // 9 allows + 1 deny -> 90% allow rate, over 80% threshold.
        for _ in 0..9 {
            g.record_and_check(ApprovalKind::Allow);
        }
        let v = g.record_and_check(ApprovalKind::Deny);
        assert!(v.is_refused());
        match v {
            ApprovalFatigueVerdict::Refused {
                observed_rate_pct, ..
            } => {
                assert_eq!(observed_rate_pct, 90);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn rolling_window_evicts_oldest() {
        let mut g = ApprovalFatigueGuard::with_window(5, 80);
        // 5 allows -> 100% over 5. Then 4 denies should push allow-rate
        // down to 20% (1 allow + 4 denies in the window).
        for _ in 0..5 {
            g.record_and_check(ApprovalKind::Allow);
        }
        for _ in 0..4 {
            g.record_and_check(ApprovalKind::Deny);
        }
        assert_eq!(g.observed_rate_pct(), 20);
    }

    #[test]
    fn build_payload_returns_none_on_allow() {
        let g = ApprovalFatigueGuard::with_defaults();
        let v = ApprovalFatigueVerdict::Allow {
            observed_rate_pct: 50,
        };
        assert!(g.build_payload(&v, None, 0).is_none());
    }

    #[test]
    fn build_payload_carries_kind_and_primitive_on_refused() {
        let g = ApprovalFatigueGuard::with_defaults();
        let v = ApprovalFatigueVerdict::Refused {
            reason: "test".into(),
            observed_rate_pct: 95,
        };
        let payload = g
            .build_payload(&v, Some("s-test".into()), 1_700_000_000_000)
            .expect("refused payload");
        assert_eq!(
            payload.kind,
            ClaudeCodeArchitectureEventKind::ApprovalFatigueDetected
        );
        assert_eq!(
            payload.primitive,
            ClaudeCodeArchPrimitive::ApprovalFatigueGuard
        );
    }
}
