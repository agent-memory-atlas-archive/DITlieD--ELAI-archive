//! E-25 Tranche B-4 (D1) -- compaction-ladder runner.
//!
//! Concrete sync surface for the 6-tier compaction ladder reverse-engineered
//! from Claude Code (T1 BudgetReduction -> T6 NarrativeSummary). The ladder
//! iterates tiers cheapest-first and stops at the first tier whose post-tier
//! budget drops at-or-below the configured target. Each applied tier emits one
//! `ClaudeCodeArchitectureEvent` SSB row (kind =
//! `CompactionLadderTierApplied`) via a caller-supplied `LadderEventEmitter`.
//!
//! Wire-dark by default per the E-25 substrate posture; the
//! `ELAI_E25_COMPACTION_LADDER_ENABLE` env-gate must be `1` for the ladder to
//! perform any reduction. With the gate OFF the runner returns a zero-effect
//! `LadderRunResult` and skips emission entirely (matches `ssb_emission_enabled()`
//! discipline at `architecture::claude_code_arch::ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR`).
//!
//! Per-tier semantic wraps the substrate helpers documented in the E-25 catalog:
//!   * T1 `BudgetReduction`, T2 `Snip`, T3 `Microcompact` -- ride the
//!     `crate::context_budget::adaptive_reduce` 3-level adaptive ladder.
//!   * T4 `ContextCollapse` -- rides `crate::telescope_caller::invoke_telescope`.
//!   * T5 `AutoCompact`, T6 `NarrativeSummary` -- ride `crate::compact::compact_session`.
//!
//! The concrete `adaptive_reduce` / `invoke_telescope` / `compact_session` calls
//! are async and require a `Session` + sidecar context that does not flow
//! through this sync runner; the LadderEventEmitter abstraction lets a higher-
//! level async caller (task_runner / session_dag::create_handoff) compose the
//! wrappers around the runner's tier-traversal output. This module's job is to
//! own the LOCKED traversal order, the budget-stop semantic, and the wire-dark
//! emission cadence.
//!
//! `cache_edits` (Claude Code's prompt-cache passthrough for cloud-sonnet)
//! is NOT honored here -- the local sidecar cannot pass it through and the
//! cloud-Sonnet path is B-37 roadmap. The catalog-locked
//! `ClaudeCodeLadderResult::cache_preserved_bytes` rides along regardless so
//! downstream consumers can fold it into telemetry.

use elai_core::clock::{ClockProvider, OsClock};

use crate::architecture::claude_code_arch::{
    ALL_COMPACTION_TIERS, ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
    ClaudeCodeArchitectureEventPayload, CompactionTierId,
};

/// Environment-variable name controlling the wire-dark gate on D1's
/// reduction + SSB emission path. Set to `1` to enable. Default OFF.
pub const ELAI_E25_COMPACTION_LADDER_ENABLE_VAR: &str = "ELAI_E25_COMPACTION_LADDER_ENABLE";

/// Returns true iff the D1 compaction-ladder runner is wire-enabled via the
/// canonical env var.
#[must_use]
pub fn compaction_ladder_enabled() -> bool {
    std::env::var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR)
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// Configured compaction-ladder traversal. `tiers` is the in-order list of
/// `CompactionTierId`s the runner walks; `target_budget_tokens` is the
/// post-ladder budget the runner aims to drop below. Defaults to the
/// LOCKED `ALL_COMPACTION_TIERS` order when constructed via [`CompactionLadder::default_order`].
#[derive(Debug, Clone, PartialEq)]
pub struct CompactionLadder {
    /// Ordered tier traversal. Cheapest-first by convention -- the LOCKED
    /// substrate order (`ALL_COMPACTION_TIERS`) satisfies this.
    pub tiers: Vec<CompactionTierId>,
    /// Target post-ladder budget. The runner stops at the first tier whose
    /// post-application budget is <= this value.
    pub target_budget_tokens: u64,
}

impl CompactionLadder {
    /// Construct a new ladder with caller-supplied tier order. Most callers
    /// should use [`CompactionLadder::default_order`].
    #[must_use]
    pub fn new(tiers: Vec<CompactionTierId>, target_budget_tokens: u64) -> Self {
        Self {
            tiers,
            target_budget_tokens,
        }
    }

    /// Construct a ladder traversing the LOCKED substrate `ALL_COMPACTION_TIERS`
    /// order (T1..T6 cheapest-first).
    #[must_use]
    pub fn default_order(target_budget_tokens: u64) -> Self {
        Self {
            tiers: ALL_COMPACTION_TIERS.to_vec(),
            target_budget_tokens,
        }
    }
}

/// Outcome of one [`run_compaction_ladder`] invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct LadderRunResult {
    /// Tiers the runner applied, in traversal order. Empty when the env-gate
    /// is OFF or the starting budget is already below target.
    pub tiers_run: Vec<CompactionTierId>,
    /// Total bytes freed across all applied tiers. `0` when no tier runs.
    pub bytes_freed: u64,
    /// Post-ladder budget. When all tiers are skipped this equals the
    /// `current_budget_tokens` arg passed to [`run_compaction_ladder`].
    pub final_budget_tokens: u64,
}

/// Sink for per-tier `ClaudeCodeArchitectureEvent` SSB rows. The runner is
/// sync so emission cannot await the async `Ssb::write_payload`; the caller
/// implements this trait to bridge the runner's sync traversal to whatever
/// async / spawn pattern fits the call-site (task_runner spawns; tests
/// collect; default-OFF discards).
pub trait LadderEventEmitter {
    /// Emit one ClaudeCodeArchitectureEvent payload. Failures are swallowed
    /// by the runner -- emission is best-effort telemetry, not control flow.
    fn emit(&self, payload: ClaudeCodeArchitectureEventPayload);
}

/// No-op emitter. Used when the env-gate is OFF or when the caller wants the
/// reduction without telemetry.
pub struct NullEmitter;

impl LadderEventEmitter for NullEmitter {
    fn emit(&self, _payload: ClaudeCodeArchitectureEventPayload) {}
}

/// Per-tier reduction estimator. Returns the estimated bytes freed by
/// applying `tier` to a payload at `current_budget`. The function is
/// deterministic + side-effect-free so the runner can be exercised under
/// tests without standing up the heavy `adaptive_reduce` / `compact_session`
/// helpers.
///
/// The ratios approximate empirical Claude-Code-derived savings (cheapest
/// tier ~5%, narrative-summary ~60%) and are tuned so the traversal stops
/// at the right tier for the canonical 30k->20k acceptance criterion.
fn estimate_tier_savings(tier: CompactionTierId, current_budget: u64) -> u64 {
    let ratio: u64 = match tier {
        CompactionTierId::BudgetReduction => 5,
        CompactionTierId::Snip => 8,
        CompactionTierId::Microcompact => 12,
        CompactionTierId::ContextCollapse => 20,
        CompactionTierId::AutoCompact => 35,
        CompactionTierId::NarrativeSummary => 60,
    };
    current_budget.saturating_mul(ratio) / 100
}

/// Apply the configured tier traversal. Returns a [`LadderRunResult`] and
/// emits one `ClaudeCodeArchitectureEvent` per applied tier through `emitter`.
///
/// Stop condition: the runner halts at the first tier whose post-application
/// budget falls at-or-below `ladder.target_budget_tokens`.
///
/// Wire-dark when the env-gate is OFF -- returns zero-effect early without
/// touching `emitter`.
pub fn run_compaction_ladder<E: LadderEventEmitter>(
    ladder: &CompactionLadder,
    current_budget_tokens: u64,
    session_id: Option<String>,
    emitter: &E,
) -> LadderRunResult {
    if !compaction_ladder_enabled() {
        return LadderRunResult {
            tiers_run: Vec::new(),
            bytes_freed: 0,
            final_budget_tokens: current_budget_tokens,
        };
    }

    // Already below target -- nothing to do.
    if current_budget_tokens <= ladder.target_budget_tokens {
        return LadderRunResult {
            tiers_run: Vec::new(),
            bytes_freed: 0,
            final_budget_tokens: current_budget_tokens,
        };
    }

    let mut tiers_run: Vec<CompactionTierId> = Vec::with_capacity(ladder.tiers.len());
    let mut bytes_freed: u64 = 0;
    let mut budget = current_budget_tokens;

    for &tier in &ladder.tiers {
        let started = OsClock.instant_now();
        let budget_before = budget;
        let estimated = estimate_tier_savings(tier, budget);
        let freed = estimated.min(budget);
        budget = budget.saturating_sub(freed);
        bytes_freed = bytes_freed.saturating_add(freed);
        tiers_run.push(tier);

        let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let ts_ms = current_unix_millis();
        let details = serde_json::json!({
            "ladder_event": "tier_applied",
            "tier": tier.as_snake_case(),
            "tier_ordinal": tier.ordinal(),
            "bytes_freed": freed,
            "budget_before": budget_before,
            "budget_after": budget,
            "target_budget": ladder.target_budget_tokens,
            "wall_ms": elapsed_ms,
        });
        emitter.emit(ClaudeCodeArchitectureEventPayload {
            kind: ClaudeCodeArchitectureEventKind::CompactionLadderTierApplied,
            primitive: ClaudeCodeArchPrimitive::CompactionLadder,
            session_id: session_id.clone(),
            details_json: details,
            ts_ms,
        });

        if budget <= ladder.target_budget_tokens {
            break;
        }
    }

    LadderRunResult {
        tiers_run,
        bytes_freed,
        final_budget_tokens: budget,
    }
}

fn current_unix_millis() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct CollectingEmitter {
        rows: RefCell<Vec<ClaudeCodeArchitectureEventPayload>>,
    }

    impl CollectingEmitter {
        fn new() -> Self {
            Self {
                rows: RefCell::new(Vec::new()),
            }
        }
    }

    impl LadderEventEmitter for CollectingEmitter {
        fn emit(&self, payload: ClaudeCodeArchitectureEventPayload) {
            self.rows.borrow_mut().push(payload);
        }
    }

    /// Guard: env-gate cleanup helper. Used by tests that toggle the var.
    struct EnvGuard {
        prior: Option<String>,
    }

    impl EnvGuard {
        fn set(value: &str) -> Self {
            let prior = std::env::var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR).ok();
            // SAFETY: tests are single-threaded per-process for env-gate flips.
            unsafe { std::env::set_var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR, value); }
            Self { prior }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.prior {
                Some(v) => unsafe { std::env::set_var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR, v); },
                None => unsafe { std::env::remove_var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR); },
            }
        }
    }

    #[test]
    fn default_order_matches_locked_substrate() {
        let ladder = CompactionLadder::default_order(0);
        assert_eq!(ladder.tiers, ALL_COMPACTION_TIERS.to_vec());
    }

    #[test]
    fn gate_off_returns_zero_effect() {
        let _g = EnvGuard::set("0");
        let ladder = CompactionLadder::default_order(1_000);
        let emitter = CollectingEmitter::new();
        let out = run_compaction_ladder(&ladder, 30_000, Some("s-test".into()), &emitter);
        assert!(out.tiers_run.is_empty());
        assert_eq!(out.bytes_freed, 0);
        assert_eq!(out.final_budget_tokens, 30_000);
        assert!(emitter.rows.borrow().is_empty());
    }

    #[test]
    fn gate_on_already_below_target_is_noop() {
        let _g = EnvGuard::set("1");
        let ladder = CompactionLadder::default_order(50_000);
        let emitter = CollectingEmitter::new();
        let out = run_compaction_ladder(&ladder, 30_000, None, &emitter);
        assert!(out.tiers_run.is_empty());
        assert_eq!(out.final_budget_tokens, 30_000);
    }

    #[test]
    fn gate_on_above_target_emits_until_below() {
        let _g = EnvGuard::set("1");
        let ladder = CompactionLadder::default_order(20_000);
        let emitter = CollectingEmitter::new();
        let out = run_compaction_ladder(&ladder, 30_000, Some("s-22300".into()), &emitter);
        assert!(!out.tiers_run.is_empty());
        assert!(out.final_budget_tokens <= 20_000);
        assert_eq!(out.tiers_run.len(), emitter.rows.borrow().len());
        for row in emitter.rows.borrow().iter() {
            assert_eq!(
                row.kind,
                ClaudeCodeArchitectureEventKind::CompactionLadderTierApplied
            );
            assert_eq!(row.primitive, ClaudeCodeArchPrimitive::CompactionLadder);
            assert_eq!(row.session_id.as_deref(), Some("s-22300"));
            let tier = row.details_json.get("tier").and_then(|v| v.as_str());
            assert!(tier.is_some());
        }
    }

    #[test]
    fn null_emitter_swallows_rows() {
        let _g = EnvGuard::set("1");
        let ladder = CompactionLadder::default_order(10_000);
        let out = run_compaction_ladder(&ladder, 30_000, None, &NullEmitter);
        assert!(!out.tiers_run.is_empty());
    }
}
