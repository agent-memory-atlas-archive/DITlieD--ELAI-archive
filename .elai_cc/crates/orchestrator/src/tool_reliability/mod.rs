//! E-18 — Tool-use reliability gate (reuse-heavy composition).
//!
//! Plan: `docs/thefinalglance/E-plan/E-18_tool_use_reliability.md`.
//! Cross-check: `workflow/sessions/s-25120/cross-check-E/E-18_tool_use_reliability-report.md`.
//! Decisions: `workflow/reasoning/E-18-decisions.md`.
//!
//! REUSE-HEAVY POSTURE
//! ===================
//! E-18 composes on top of substrate already shipped by sibling plans
//! rather than re-implementing it. Specifically:
//!
//!   * D-11 [`crate::checker_registry::CheckerKind::RegistryCheck`] —
//!     anti-slopsquatting package registry probe + cooldown. E-18 does NOT
//!     re-author `tools::package_verify` (the plan's original ask — see
//!     scope rebase block above the module header in the orchestrate
//!     prompt). The reliability gate consults D-11 via the existing
//!     checker dispatch for package-touching tool calls.
//!   * E-35 `elai_tools::search_replace::SearchReplaceMatcher` +
//!     `DelimiterCollisionGuard` + `elai_providers::runtime::
//!     sidecar_xgrammar_hook::SidecarXGrammarHook` + the
//!     `ToolParserVariant` enum (`elai_ipc_types`). E-18 picks the
//!     parser variant via runtime config and never re-implements the
//!     parser.
//!   * C-47 [`elai_plugins::tool_calibration::ToolCalibrationRegistry`]
//!     observes per-tool aggregate signal. E-18 records `Reject` /
//!     `Spill` verdicts AFTER C-47 has been consulted (so the C-47
//!     case-base sees the same call shape regardless of whether the
//!     E-18 gate fires).
//!   * C-42 [`elai_plugins::authz::AuthzPolicy`] runs BEFORE the C-35
//!     [`crate::mcp_tool_bridge`] ToolGate. E-18 inserts AFTER the
//!     gate-allow / authz-allow result so a denied call never reaches
//!     the reliability check.
//!
//! Composition order (preserved):
//!     `AuthzPolicy -> ToolGate -> ToolCalibration consult -> ToolReliabilityGate -> dispatch -> tool exec`
//!
//! NEW 4-VARIANT SSB SCOPE
//! =======================
//! The plan's WIRING_CONTRACT enumerated 8 new variants; the E-35
//! deferred-SSB list separately enumerated 10. After scope rebase the
//! cut shipped here is the 4 variants distinct from BOTH lists
//! (overlap-trimmed):
//!
//!   * `ToolReliabilityGatePassed`
//!   * `ToolReliabilityGateRejected`
//!   * `ToolCallSpilled`
//!   * `ToolCallRetryEscalated`
//!
//! See `workflow/reasoning/E-18-decisions.md` for the consolidation
//! rationale.
//!
//! ANTI-STALE GROUNDING
//! ====================
//!   * `blake3` for any args / spill / path digest (NEVER blake2 / sha2).
//!   * `thiserror` enum, NOT `Box<dyn Error>`.
//!   * `tracing` (NEVER the `log` crate).
//!   * `tokio::sync::Mutex` in async paths (only `std::sync::Mutex` in
//!     provably synchronous test code).
//!   * No `.unwrap()` / `.expect()` outside `#[cfg(test)]` and bootstrap
//!     bootloaders.
//!   * `u32` / `u64` `try_from` over `as` casts at narrow boundaries.

pub mod gate;
pub mod retry;
pub mod spill;
pub mod types;

pub use self::gate::{ReliabilityGateInputs, ToolReliabilityGate, RELIABILITY_GATE_INLINE_CAP};
pub use self::retry::{should_escalate, RetryDecision};
pub use self::spill::{spill_call, spill_root_default, SpillRecord, SPILL_DIR_NAME};
pub use self::types::{ReliabilityRejectReason, ReliabilityVerdict, SpillReason, ToolCallSnapshot};

use thiserror::Error;

/// Unified error type for the tool-reliability surface. Per-submodule
/// errors flatten into this so callers get a single `Result<_, ToolReliabilityError>`.
#[derive(Debug, Error)]
pub enum ToolReliabilityError {
    /// Spill write failed (filesystem error or quota exhausted). Carries
    /// the underlying `io::Error` via `#[from]` so `?` propagation works
    /// at call sites.
    #[error("spill io error: {0}")]
    SpillIo(#[from] std::io::Error),

    /// Inline payload exceeded the cap AND the spill subsystem refused
    /// to take the overflow (e.g. SPILL_ENABLE=0 + payload too large).
    /// The dispatcher should surface this as a hard reject, not retry.
    #[error("payload over cap ({payload_bytes} > {cap_bytes}) and spill disabled")]
    PayloadOverCapNoSpill {
        payload_bytes: usize,
        cap_bytes: usize,
    },

    /// blake3 prefix narrowing produced an unexpected slice length. This
    /// should be unreachable because `blake3::Hash::as_bytes()` returns a
    /// 32-byte array, but we encode the invariant explicitly rather than
    /// `.unwrap()`ing per CLAUDE.md anti-unwrap rule.
    #[error("blake3 prefix narrowing failed (expected 8 bytes)")]
    HashPrefixNarrowing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_over_cap_renders_both_sizes() {
        let e = ToolReliabilityError::PayloadOverCapNoSpill {
            payload_bytes: 16_384,
            cap_bytes: 8_192,
        };
        let s = format!("{e}");
        assert!(s.contains("16384"));
        assert!(s.contains("8192"));
    }

    #[test]
    fn hash_prefix_narrowing_renders() {
        let e = ToolReliabilityError::HashPrefixNarrowing;
        let s = format!("{e}");
        assert!(s.contains("blake3"));
    }
}
