//! E-25 Tranche C (C.1.a) -- BashComplexityGuard.
//!
//! Refuses Bash tool invocations whose command body exceeds any of three
//! complexity thresholds. Wire-dark default OFF; gated by the
//! [`ELAI_E25_BASH_COMPLEXITY_GUARD_ENABLE_VAR`] env var.
//!
//! Trigger site: `HookPoint::PreToolUse` on Bash tool calls (see
//! `crates/orchestrator/src/hook_dispatcher.rs:1603`).
//!
//! Thresholds are NAMED CONSTANTS, not magic numbers (CLAUDE.md HARD
//! RULE Anti-Stale-Training). Token count is whitespace-split; pipe
//! count and metachar count are simple byte scans. Tuned at substrate
//! ship to flag the canonical Claude Code per-subcommand safety-bypass
//! pattern (VILA-Lab arxiv reverse-engineering of >50-subcommand
//! pipelines escaping policy review).
//!
//! Emission: reuses the SHIPPED
//! [`crate::architecture::claude_code_arch::ClaudeCodeArchitectureEventKind::BashComplexityBlocked`]
//! arm via the Pattern-2 single-variant `SsbActionType::ClaudeCodeArchitectureEvent`
//! cascade -- ZERO new SsbActionType, ZERO schema bump (per session-4
//! DECISION-2 + DECISION-5 9-arm lock).
//!
//! Pattern follows `crates/orchestrator/src/cost/free_tier_guard.rs:103`
//! `FreeTierGuard` precedent for per-session struct + check + emit.

use crate::architecture::claude_code_arch::{
    ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
    ClaudeCodeArchitectureEventPayload,
};

/// Environment-variable name controlling the wire-dark gate on the
/// BashComplexityGuard. Set to a truthy value (`"1"` / `"true"` /
/// `"yes"` / `"on"`) to enable. Default OFF.
pub const ELAI_E25_BASH_COMPLEXITY_GUARD_ENABLE_VAR: &str =
    "ELAI_E25_BASH_COMPLEXITY_GUARD_ENABLE";

/// Whitespace-token count threshold. Commands whose token-count exceeds
/// this value are refused. Tuned to flag long heredocs / for-loops the
/// Claude Code harness regularly sees abused (VILA-Lab pre-trust
/// execution-window analysis).
pub const BASH_TOKEN_COUNT_THRESHOLD: usize = 80;

/// Pipe (`|`) count threshold. Commands with more pipes than this in
/// the body are refused.
pub const BASH_PIPE_COUNT_THRESHOLD: usize = 4;

/// Shell metachar count threshold. Counts occurrences of any of the
/// canonical complexity-introducing metacharacters `> ; ` $ ( )`. The
/// backtick is excluded from the scan because skill-templating bodies
/// legitimately use it; the guard targets one-shot Bash invocations,
/// not skill templates.
pub const BASH_METACHAR_COUNT_THRESHOLD: usize = 6;

/// Verdict returned by [`BashComplexityGuard::check`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BashComplexityVerdict {
    /// Command body is within all three thresholds; safe to dispatch.
    Allow,
    /// Command body exceeds at least one threshold; dispatch refused.
    /// Carries a human-readable reason citing which threshold tripped.
    Refused { reason: String },
}

impl BashComplexityVerdict {
    #[must_use]
    pub fn is_refused(&self) -> bool {
        matches!(self, Self::Refused { .. })
    }
}

/// Returns true iff the BashComplexityGuard is wire-enabled via the
/// canonical env var (truthy-matcher).
#[must_use]
pub fn bash_complexity_guard_enabled() -> bool {
    matches_truthy_env(ELAI_E25_BASH_COMPLEXITY_GUARD_ENABLE_VAR)
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

/// Stateless per-process guard. Mirrors the
/// [`crate::cost::free_tier_guard::FreeTierGuard`] surface shape but
/// holds no state -- each Bash invocation gets evaluated independently
/// against the named thresholds. The guard is a struct (not a free
/// function) so the gates registry can hold a stable handle and so
/// future per-session window state can be folded in without touching
/// call sites.
#[derive(Debug, Clone, Copy, Default)]
pub struct BashComplexityGuard;

impl BashComplexityGuard {
    /// Construct a new guard. Equivalent to `Self::default()`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a Bash command body against the three thresholds.
    /// Returns [`BashComplexityVerdict::Allow`] when all three are
    /// satisfied; [`BashComplexityVerdict::Refused`] with a reason
    /// otherwise.
    ///
    /// Pure function -- no side effects, no SSB emission. Callers wire
    /// emission via [`BashComplexityGuard::build_payload`] when the
    /// verdict is `Refused` and the wire-dark gate is on.
    #[must_use]
    pub fn check(&self, command: &str) -> BashComplexityVerdict {
        let token_count = count_tokens(command);
        if token_count > BASH_TOKEN_COUNT_THRESHOLD {
            return BashComplexityVerdict::Refused {
                reason: format!(
                    "bash_complexity: token_count {token_count} exceeds threshold {BASH_TOKEN_COUNT_THRESHOLD}"
                ),
            };
        }
        let pipe_count = count_byte(command, b'|');
        if pipe_count > BASH_PIPE_COUNT_THRESHOLD {
            return BashComplexityVerdict::Refused {
                reason: format!(
                    "bash_complexity: pipe_count {pipe_count} exceeds threshold {BASH_PIPE_COUNT_THRESHOLD}"
                ),
            };
        }
        let metachar_count = count_metachars(command);
        if metachar_count > BASH_METACHAR_COUNT_THRESHOLD {
            return BashComplexityVerdict::Refused {
                reason: format!(
                    "bash_complexity: metachar_count {metachar_count} exceeds threshold {BASH_METACHAR_COUNT_THRESHOLD}"
                ),
            };
        }
        BashComplexityVerdict::Allow
    }

    /// Build the SSB payload for a refusal verdict. Returns `None` for
    /// `Allow` (no row is emitted on allow per Pattern-2 lean).
    #[must_use]
    pub fn build_payload(
        &self,
        command: &str,
        verdict: &BashComplexityVerdict,
        session_id: Option<String>,
        ts_ms: i64,
    ) -> Option<ClaudeCodeArchitectureEventPayload> {
        match verdict {
            BashComplexityVerdict::Allow => None,
            BashComplexityVerdict::Refused { reason } => {
                let token_count = count_tokens(command);
                let pipe_count = count_byte(command, b'|');
                let metachar_count = count_metachars(command);
                let details_json = serde_json::json!({
                    "guard_event": "bash_complexity_refused",
                    "reason": reason,
                    "token_count": token_count,
                    "pipe_count": pipe_count,
                    "metachar_count": metachar_count,
                    "token_threshold": BASH_TOKEN_COUNT_THRESHOLD,
                    "pipe_threshold": BASH_PIPE_COUNT_THRESHOLD,
                    "metachar_threshold": BASH_METACHAR_COUNT_THRESHOLD,
                });
                Some(ClaudeCodeArchitectureEventPayload {
                    kind: ClaudeCodeArchitectureEventKind::BashComplexityBlocked,
                    primitive: ClaudeCodeArchPrimitive::BashComplexityGuard,
                    session_id,
                    details_json,
                    ts_ms,
                })
            }
        }
    }
}

fn count_tokens(s: &str) -> usize {
    s.split_whitespace().count()
}

fn count_byte(s: &str, target: u8) -> usize {
    s.as_bytes().iter().filter(|&&b| b == target).count()
}

fn count_metachars(s: &str) -> usize {
    // > ; $ ( )
    s.as_bytes()
        .iter()
        .filter(|&&b| matches!(b, b'>' | b';' | b'$' | b'(' | b')'))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_command_allowed() {
        let guard = BashComplexityGuard::new();
        let verdict = guard.check("ls -la");
        assert_eq!(verdict, BashComplexityVerdict::Allow);
    }

    #[test]
    fn token_count_over_threshold_refused() {
        let guard = BashComplexityGuard::new();
        // 90 tokens, all simple words -- only the token threshold trips.
        let cmd = (0..90).map(|i| format!("t{i}")).collect::<Vec<_>>().join(" ");
        let verdict = guard.check(&cmd);
        assert!(verdict.is_refused());
        match verdict {
            BashComplexityVerdict::Refused { reason } => {
                assert!(reason.contains("token_count"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn pipe_count_over_threshold_refused() {
        let guard = BashComplexityGuard::new();
        // 5 pipes -- exceeds BASH_PIPE_COUNT_THRESHOLD (4).
        let cmd = "a | b | c | d | e | f";
        let verdict = guard.check(cmd);
        assert!(verdict.is_refused());
        match verdict {
            BashComplexityVerdict::Refused { reason } => {
                assert!(reason.contains("pipe_count"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn metachar_count_over_threshold_refused() {
        let guard = BashComplexityGuard::new();
        // 7 metachars -- exceeds BASH_METACHAR_COUNT_THRESHOLD (6).
        let cmd = "echo $a $b $c $d $e $f $g";
        let verdict = guard.check(cmd);
        assert!(verdict.is_refused());
        match verdict {
            BashComplexityVerdict::Refused { reason } => {
                assert!(reason.contains("metachar_count"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn build_payload_returns_none_on_allow() {
        let guard = BashComplexityGuard::new();
        let verdict = BashComplexityVerdict::Allow;
        let payload = guard.build_payload("ls", &verdict, None, 0);
        assert!(payload.is_none());
    }

    #[test]
    fn build_payload_carries_kind_and_primitive_on_refused() {
        let guard = BashComplexityGuard::new();
        let cmd = "a | b | c | d | e | f";
        let verdict = guard.check(cmd);
        let payload = guard
            .build_payload(cmd, &verdict, Some("s-test".into()), 1_700_000_000_000)
            .expect("refused payload");
        assert_eq!(
            payload.kind,
            ClaudeCodeArchitectureEventKind::BashComplexityBlocked
        );
        assert_eq!(
            payload.primitive,
            ClaudeCodeArchPrimitive::BashComplexityGuard
        );
        assert_eq!(payload.session_id.as_deref(), Some("s-test"));
    }

    #[test]
    fn env_gate_defaults_off() {
        // Use an alternate var name to avoid races with the real env;
        // simulate via direct std::env scoped to this test.
        // The real var is checked via std::env::var; default-off semantics
        // mean an unset var yields `false`. We assert that against an
        // arbitrary unused name to avoid touching the production gate.
        assert!(!matches_truthy_env(
            "ELAI_E25_BASH_COMPLEXITY_GUARD_ENABLE_TEST_UNSET"
        ));
    }
}
