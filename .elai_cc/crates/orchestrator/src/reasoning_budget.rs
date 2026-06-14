//! H-42 — serial-depth-scaled reasoning budget + depth-not-width guard.
//!
//! # Purpose
//! ELAI's GRC budget is per-role and STATIC (grc.rs::AgentGrcConfig).  The formal
//! CoT-expressivity literature proves that the number of serial reasoning steps a
//! problem needs scales with its SERIAL DEPTH, not with its context window size:
//!
//!   - A single forward pass is bounded to ~TC0 (Merrill & Sabharwal 2310.07923).
//!   - T serial CoT steps simulate a size-T circuit (Li et al. 2402.12875).
//!   - Compositional/serial tasks need ~O(n) CoT tokens for input size n with tight
//!     lower bounds (Yehudai et al. 2503.01544; Amiri & Hahn 2502.02393).
//!   - A bigger context window buys information ACCESS, not compute DEPTH (still one
//!     ~TC0 pass per token).
//!
//! H-42 scales the per-role GRC + E-19 reflection budget by an estimated serial-depth
//! signal (decomposition-tree level), bounded by a LOCKED ceiling, and adds the
//! DEPTH-NOT-WIDTH guard: a budget-exhaustion on a high-depth sub-task is classified
//! `DepthBound` (needs more steps) vs `AccessBound` (needs more context) so a
//! context-window lift is NEVER mis-applied as the fix for a depth-class failure.
//!
//! # Design
//! - `estimate_serial_depth` — cheapest available signal: decomposition-tree level.
//!   BabyStep=3 (deepest), Step=2, Phase=1, Concept=0. Returns a `u32` depth d.
//! - `scale_grc_budget` — multiplies per-role call+char budget by factor `1 + 0.25*d`,
//!   clamped to `LOCKED_BUDGET_CEILING` × base (loop-inflation proof: the ceiling
//!   constant is `mutable=0` — nothing at runtime can raise it).
//! - `classify_failure` — given (depth, budget_exhausted, retrieval_miss) returns
//!   `FailureClass { depth_bound, access_bound }` (struct with two booleans, not a
//!   strict enum, so a task that is BOTH deep AND retrieval-starved surfaces both;
//!   RD-4: a DepthBound row's remedy is steps/decomposition, NEVER context).
//! - `ReasoningBudgetScaledEmitter` trait + `VecReasoningBudgetEmitter` — off-agent
//!   SSB signal emitted at the dispatch site (R4/R7).
//!
//! # Kill-switch
//! `ELAI_REASONING_BUDGET_SCALE_DISABLE` — when set to "1" or "true", scaling is
//! skipped and `scale_grc_budget` returns the base config unchanged. Default-ON;
//! no wire-dark-allowlist row required for a kill-switch (R9, type=kill_switch).
//!
//! # HARD RULES (CLAUDE.md)
//! - No `.unwrap()` / `.expect()` in production code paths.
//! - blake3 for any hashing (no sha2 / blake2).
//! - `ClockProvider` / inject `now_ms: i64` at call sites — never `SystemTime`.
//! - GD-1: consume `AgentGrcConfig` from `elai_reasoning`; do NOT re-implement.
//! - GD-2: `scale_grc_budget` is deterministic from (base, depth) under ELAI_SEED.
//! - RD-4: depth-not-width guard is enforced at the call site; never silently apply
//!         a context lift for a DepthBound failure.

use std::sync::Arc;

use elai_reasoning::{AgentGrcConfig, GrcBudget};

use crate::pipeline_tree_navigator::DecompositionLevel;
use crate::ssb::{ReasoningBudgetScaledPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Kill-switch env-var: when set to "1" or "true" (case-insensitive) the depth
/// scaling is bypassed and every sub-task gets the flat base budget.
pub const ELAI_REASONING_BUDGET_SCALE_DISABLE_ENV: &str = "ELAI_REASONING_BUDGET_SCALE_DISABLE";

/// Locked ceiling multiplier. The scaled budget is clamped so that
/// `scaled_calls <= LOCKED_BUDGET_CEILING * base.budget.max_calls`.
///
/// **This constant is intentionally NOT env-overridable at runtime** (`mutable=0`).
/// A deep task can never inflate its own budget above 3× the base.
/// Start conservative (3×); calibrate on ≥3 measured runs before raising — see
/// `workflow/reasoning/H-42-decisions.md` (GD-4, RISK-3).
pub const LOCKED_BUDGET_CEILING: u32 = 3;

// ---------------------------------------------------------------------------
// FailureClass: NOT a strict single-arm enum (RISK-2 from context-package)
// ---------------------------------------------------------------------------

/// Diagnostic for a GRC budget-exhaustion or low self-certainty event.
///
/// Two boolean flags — NOT a strict single-arm enum — because a sub-task can be
/// simultaneously depth-bound (high serial depth) AND access-bound (retrieval miss).
/// Forcing one arm would over-claim a clean dichotomy (see RISK-2, context-package).
///
/// **Depth-not-width guard (RD-4):** a `DepthBound` failure's remedy is MORE serial
/// CoT steps or finer decomposition.  It is NEVER appropriate to raise the context
/// window as a fix for a depth-class failure; the formal result (Merrill & Sabharwal
/// 2310.07923) shows context buys ACCESS (TC0 pass), not DEPTH (circuit depth class).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FailureClass {
    /// High serial depth + budget exhausted → needs MORE CoT steps / decomposition,
    /// never more context.  Corresponds to the `DepthBound` arm in the SSB Kind.
    pub depth_bound: bool,
    /// Low depth + retrieval miss (no relevant documents / context starvation) →
    /// needs richer context or better retrieval, NOT more CoT steps.
    pub access_bound: bool,
}

impl FailureClass {
    /// A failure where neither depth nor access is the dominant cause.
    pub const NEITHER: FailureClass = FailureClass {
        depth_bound: false,
        access_bound: false,
    };

    /// Returns the dominant SSB Kind label for the ReasoningBudgetScaled row.
    ///
    /// Priority: DepthBound wins over AccessBound when both are true, because the
    /// depth-not-width guard (RD-4) must visibly surface the depth-class signal.
    pub fn as_ssb_kind_str(self) -> &'static str {
        match (self.depth_bound, self.access_bound) {
            (true, _) => "depth_bound",
            (false, true) => "access_bound",
            (false, false) => "neither",
        }
    }
}

// ---------------------------------------------------------------------------
// estimate_serial_depth
// ---------------------------------------------------------------------------

/// Map a decomposition-tree level to a 0-based serial depth integer.
///
/// This is the cheapest shipped signal: the tree level already exists in
/// `pipeline_tree_navigator::DecompositionLevel` (B-7 SHIPPED).  Returns:
///   - `Concept`  → 0 (shallowest, strategic goal)
///   - `Phase`    → 1
///   - `Step`     → 2
///   - `BabyStep` → 3 (deepest, atomic execution unit)
///
/// At BabyStep depth the scale factor is `1 + 0.25 × 3 = 1.75×`.
/// The function is deterministic and seed-stable (GD-2).
pub fn estimate_serial_depth(level: DecompositionLevel) -> u32 {
    match level {
        DecompositionLevel::Concept => 0,
        DecompositionLevel::Phase => 1,
        DecompositionLevel::Step => 2,
        DecompositionLevel::BabyStep => 3,
    }
}

// ---------------------------------------------------------------------------
// scale_grc_budget
// ---------------------------------------------------------------------------

/// Scale a per-role GRC config by the serial depth `d`, clamped to
/// `LOCKED_BUDGET_CEILING × base`.
///
/// Scaling rule (RISK-3 starter configuration — calibrate on ≥3 runs):
///   `factor = 1.0 + 0.25 * d`   (d=0 → 1.0×, d=1 → 1.25×, d=2 → 1.5×, d=3 → 1.75×)
///   `scaled_calls = min(round(base * factor), LOCKED_BUDGET_CEILING * base)`
///   `scaled_chars = min(round(base * factor), LOCKED_BUDGET_CEILING * base)`
///
/// The ceiling (`LOCKED_BUDGET_CEILING = 3`) is a compile-time constant that
/// nothing at runtime can raise.  A loop cannot inflate its own budget (GD-3).
///
/// When the kill-switch `ELAI_REASONING_BUDGET_SCALE_DISABLE=1` is set, the base
/// config is returned unchanged (flat budget, used by the `--flat-budget` broken-arm
/// test, R2 falsifiability).
///
/// The function is pure (no IO, no side effects) and deterministic under ELAI_SEED
/// (GD-2): given the same (base, d) pair it always returns the same scaled config.
pub fn scale_grc_budget(base: AgentGrcConfig, d: u32) -> AgentGrcConfig {
    if depth_scaling_disabled_via_env() {
        return base;
    }
    if d == 0 {
        // Depth 0 (Concept level): no scaling needed; return the base unchanged.
        return base;
    }

    let ceiling_calls = base.budget.max_calls.saturating_mul(LOCKED_BUDGET_CEILING);
    let ceiling_chars = base.budget.max_chars.saturating_mul(LOCKED_BUDGET_CEILING);

    // factor = 1.0 + 0.25 * d  (scaled by 100 then divided to avoid f32 in const).
    // Use integer arithmetic: numerator = (100 + 25 * d), denominator = 100.
    let numer: u64 = 100 + 25 * (d as u64);
    let denom: u64 = 100;

    // max_calls scaled, then clamped.
    let scaled_calls = {
        let v = (base.budget.max_calls as u64)
            .saturating_mul(numer)
            .saturating_add(denom - 1) // ceiling division
            / denom;
        let v_u32 = u32::try_from(v).unwrap_or(u32::MAX);
        v_u32.min(ceiling_calls)
    };

    // max_chars scaled, then clamped.
    let scaled_chars = {
        let v = (base.budget.max_chars as u64)
            .saturating_mul(numer)
            .saturating_add(denom - 1) // ceiling division
            / denom;
        let v_u32 = u32::try_from(v).unwrap_or(u32::MAX);
        v_u32.min(ceiling_chars)
    };

    let scaled_budget = GrcBudget {
        max_calls: scaled_calls,
        max_chars: scaled_chars,
        calls_used: base.budget.calls_used,
        chars_used: base.budget.chars_used,
    };

    AgentGrcConfig {
        budget: scaled_budget,
        ..base
    }
}

// ---------------------------------------------------------------------------
// classify_failure
// ---------------------------------------------------------------------------

/// Classify a GRC budget-exhaustion or low-certainty event as DepthBound,
/// AccessBound, both, or neither.
///
/// Rules:
/// - `depth_bound = true` when `depth >= 2` AND `budget_exhausted = true`.
///   Rationale: a task at Step or BabyStep depth that runs out of reasoning
///   calls is a depth-class failure — the fix is more serial steps or finer
///   decomposition (2503.01544 ~O(n); 2310.07923 TC0 bound).
/// - `access_bound = true` when `retrieval_miss = true` AND depth is NOT the
///   dominant cause.  A shallow task (depth < 2) that fails because it could
///   not retrieve enough context is access-bound.
///
/// Both flags can be true simultaneously (RISK-2 mitigation: no forced dichotomy).
/// In that case the SSB row carries `kind = "depth_bound"` (dominant, RD-4) but
/// both signals are available for downstream diagnostics.
///
/// `retrieval_miss` is caller-supplied: `true` when the GRC engine surfaced a
/// low `compute_score` from `self_certainty.rs::compute_score` (E-19 REUSED).
pub fn classify_failure(depth: u32, budget_exhausted: bool, retrieval_miss: bool) -> FailureClass {
    let depth_bound = depth >= 2 && budget_exhausted;
    // Access-bound: if depth is not the primary cause (depth < 2) and there was
    // a retrieval miss; OR if the retrieval miss co-occurred with high depth
    // (both true — do not suppress access_bound just because depth_bound is set).
    let access_bound = retrieval_miss;

    FailureClass {
        depth_bound,
        access_bound,
    }
}

// ---------------------------------------------------------------------------
// Kill-switch helper
// ---------------------------------------------------------------------------

/// Returns `true` when `ELAI_REASONING_BUDGET_SCALE_DISABLE` is set to "1" or
/// "true" (case-insensitive).  All other values (absent / empty / "0") are treated
/// as the gate ENABLED (default-ON behaviour).
pub fn depth_scaling_disabled_via_env() -> bool {
    match std::env::var(ELAI_REASONING_BUDGET_SCALE_DISABLE_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true")
        }
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// ReasoningBudgetScaledEmitter trait + VecReasoningBudgetEmitter
// ---------------------------------------------------------------------------

/// Trait for emitting a `ReasoningBudgetScaled` SSB row at the dispatch site.
///
/// Production callers wrap `Arc<Ssb>::write_payload`; tests use
/// [`VecReasoningBudgetEmitter`] which stores payloads in memory.
///
/// Native async-fn-in-trait (Anti-Stale: no `async-trait` crate).
pub trait ReasoningBudgetScaledEmitter: Send + Sync {
    /// Emit one `reasoning_budget_scaled` SSB row. Returning `Err(String)` is
    /// best-effort — the caller logs the error and continues with the scaled config.
    fn emit_scaled(
        &self,
        payload: ReasoningBudgetScaledPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>>;
}

/// Test-only emitter that accumulates [`ReasoningBudgetScaledPayload`] rows in a
/// `Vec` behind a `tokio::sync::Mutex` (async-safe per anti-stale rule).
#[derive(Default)]
pub struct VecReasoningBudgetEmitter {
    rows: tokio::sync::Mutex<Vec<ReasoningBudgetScaledPayload>>,
}

impl VecReasoningBudgetEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<ReasoningBudgetScaledPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.rows.lock().await.is_empty()
    }
}

impl ReasoningBudgetScaledEmitter for VecReasoningBudgetEmitter {
    fn emit_scaled(
        &self,
        payload: ReasoningBudgetScaledPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            self.rows.lock().await.push(payload);
            Ok(())
        })
    }
}

/// Production emitter: writes one `ReasoningBudgetScaled` SSB row via
/// [`Arc<Ssb>::write_payload`].  This is the load-bearing off-agent artifact
/// (R4/R7): the row lands in the SSB database, not just in a tracing log.
///
/// `agent_id` is fixed to `"h42_budget_scale"` and `superstep` to `0` because
/// this emission happens at pool-dispatch time, before an agent has a real ID.
/// Downstream consumers correlate via `role` + `serial_depth`.
///
/// Emit errors are non-fatal: the caller logs via `tracing::warn` and continues
/// with the scaled config.  SSB write failures do not abort the dispatch (the
/// scaled budget is control-flow; the row is diagnostic).
impl ReasoningBudgetScaledEmitter for Arc<Ssb> {
    fn emit_scaled(
        &self,
        payload: ReasoningBudgetScaledPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        let ssb = Arc::clone(self);
        Box::pin(async move {
            ssb.write_payload(
                "h42_budget_scale",
                0,
                SsbPayload::ReasoningBudgetScaled(payload),
                Vec::new(),
            )
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
        })
    }
}

// ---------------------------------------------------------------------------
// scale_and_emit: combined helper for dispatch sites
// ---------------------------------------------------------------------------

/// Scale a GRC config by depth and emit a `ReasoningBudgetScaled` SSB row.
///
/// Used at both dispatch sites (agent_pool.rs::spawn and bsp/engine.rs timeout path).
/// Returns the scaled `AgentGrcConfig`.  A failure to emit the SSB row is logged via
/// `tracing::warn` but does NOT abort the dispatch — the row is diagnostic, not
/// load-bearing for control flow.
///
/// `failure_class` is determined by the caller from prior failure context (e.g. a
/// previous budget-exhaustion + retrieval miss at this role/depth).  Pass
/// `FailureClass::NEITHER` when no prior failure is known (normal dispatch path).
pub async fn scale_and_emit<E: ReasoningBudgetScaledEmitter>(
    base: AgentGrcConfig,
    level: DecompositionLevel,
    failure_class: FailureClass,
    seed: u64,
    emitter: &E,
) -> AgentGrcConfig {
    let d = estimate_serial_depth(level);
    let scaled = scale_grc_budget(base.clone(), d);

    // Only emit the SSB row when scaling actually changed the budget.
    // At depth=0 (Concept) or when the kill-switch is active, scale_grc_budget
    // returns the base unchanged — no scaling event occurred, so no row is emitted
    // (Missing = FAIL R1 means the row is absent for non-events, not for events).
    if scaled.budget.max_calls != base.budget.max_calls {
        let payload = ReasoningBudgetScaledPayload {
            kind: failure_class.as_ssb_kind_str().to_owned(),
            role: base.role.clone(),
            base_calls: base.budget.max_calls,
            serial_depth: d,
            scaled_calls: scaled.budget.max_calls,
            ceiling: base.budget.max_calls.saturating_mul(LOCKED_BUDGET_CEILING),
            seed,
            // I-26 payload extension fields — None on GRC-dispatch rows (H-42 path);
            // populated only by the I-26 BudgetController search-node collapse path.
            depth_level: None,
            budget_pct: None,
            plateau_fired: None,
        };

        if let Err(e) = emitter.emit_scaled(payload).await {
            tracing::warn!(
                target: "orchestrator::reasoning_budget",
                role = %base.role,
                depth = d,
                error = %e,
                "H-42: failed to emit ReasoningBudgetScaled SSB row; dispatch continues"
            );
        }
    }

    scaled
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_reasoning::GrcBudget;

    fn base_config(role: &str, max_calls: u32, max_chars: u32) -> AgentGrcConfig {
        AgentGrcConfig {
            role: role.into(),
            budget: GrcBudget::new(max_calls, max_chars),
            ..AgentGrcConfig::default()
        }
    }

    // -----------------------------------------------------------------------
    // estimate_serial_depth
    // -----------------------------------------------------------------------

    #[test]
    fn concept_depth_is_zero() {
        assert_eq!(estimate_serial_depth(DecompositionLevel::Concept), 0);
    }

    #[test]
    fn phase_depth_is_one() {
        assert_eq!(estimate_serial_depth(DecompositionLevel::Phase), 1);
    }

    #[test]
    fn step_depth_is_two() {
        assert_eq!(estimate_serial_depth(DecompositionLevel::Step), 2);
    }

    #[test]
    fn baby_step_depth_is_three() {
        assert_eq!(estimate_serial_depth(DecompositionLevel::BabyStep), 3);
    }

    // -----------------------------------------------------------------------
    // scale_grc_budget
    // -----------------------------------------------------------------------

    #[test]
    fn concept_depth_zero_unchanged() {
        let base = base_config("executor", 8, 1200);
        let scaled = scale_grc_budget(base.clone(), 0);
        assert_eq!(scaled.budget.max_calls, 8);
        assert_eq!(scaled.budget.max_chars, 1200);
    }

    #[test]
    fn baby_step_depth_3_scales_by_1_75() {
        // factor = 1 + 0.25 * 3 = 1.75; ceiling = 3 * 8 = 24 (not hit)
        // 8 * 1.75 = 14 (ceiling div: 8*175/100 = 1400/100 = 14)
        let base = base_config("executor", 8, 1200);
        let scaled = scale_grc_budget(base, 3);
        assert_eq!(scaled.budget.max_calls, 14, "8 * 1.75 = 14");
        // 1200 * 175 / 100 = 2100
        assert_eq!(scaled.budget.max_chars, 2100, "1200 * 1.75 = 2100");
    }

    #[test]
    fn ceiling_is_respected() {
        // With very high d the ceiling clamps output.
        // base = 8 calls; d = 100 => factor = 26.0 => 208 calls, but ceil = 24
        let base = base_config("executor", 8, 1200);
        let scaled = scale_grc_budget(base, 100);
        assert_eq!(
            scaled.budget.max_calls,
            LOCKED_BUDGET_CEILING * 8,
            "ceiling must clamp at LOCKED_BUDGET_CEILING × base"
        );
    }

    #[test]
    fn base_is_the_floor_at_depth_zero() {
        let base = base_config("auditor", 10, 1600);
        let scaled = scale_grc_budget(base.clone(), 0);
        assert_eq!(scaled.budget.max_calls, base.budget.max_calls);
        assert_eq!(scaled.budget.max_chars, base.budget.max_chars);
    }

    #[test]
    fn deep_gets_larger_budget_than_shallow() {
        let base_shallow = base_config("executor", 8, 1200);
        let base_deep = base_config("executor", 8, 1200);
        let shallow = scale_grc_budget(base_shallow, 0);
        let deep = scale_grc_budget(base_deep, 3);
        assert!(
            deep.budget.max_calls > shallow.budget.max_calls,
            "deep sub-task must get a larger budget than shallow"
        );
    }

    #[test]
    fn role_is_preserved_after_scaling() {
        let base = base_config("context_compiler", 5, 800);
        let scaled = scale_grc_budget(base, 2);
        assert_eq!(scaled.role, "context_compiler");
    }

    // -----------------------------------------------------------------------
    // classify_failure
    // -----------------------------------------------------------------------

    #[test]
    fn high_depth_exhaustion_is_depth_bound() {
        let fc = classify_failure(3, true, false);
        assert!(fc.depth_bound, "depth=3 + exhausted must be DepthBound");
        assert!(!fc.access_bound);
    }

    #[test]
    fn low_depth_retrieval_miss_is_access_bound() {
        let fc = classify_failure(1, false, true);
        assert!(!fc.depth_bound, "depth=1 is not DepthBound");
        assert!(fc.access_bound, "retrieval miss must be AccessBound");
    }

    #[test]
    fn both_depth_and_access_can_be_set() {
        // RISK-2: no forced single-arm enum
        let fc = classify_failure(3, true, true);
        assert!(fc.depth_bound);
        assert!(fc.access_bound);
        // Dominant SSB kind is still depth_bound (RD-4)
        assert_eq!(fc.as_ssb_kind_str(), "depth_bound");
    }

    #[test]
    fn no_failure_gives_neither() {
        let fc = classify_failure(0, false, false);
        assert_eq!(fc, FailureClass::NEITHER);
        assert_eq!(fc.as_ssb_kind_str(), "neither");
    }

    // -----------------------------------------------------------------------
    // FailureClass::as_ssb_kind_str
    // -----------------------------------------------------------------------

    #[test]
    fn depth_bound_wins_over_access_bound_in_kind_str() {
        let fc = FailureClass {
            depth_bound: true,
            access_bound: true,
        };
        assert_eq!(fc.as_ssb_kind_str(), "depth_bound");
    }

    #[test]
    fn access_bound_only_gives_access_bound_kind_str() {
        let fc = FailureClass {
            depth_bound: false,
            access_bound: true,
        };
        assert_eq!(fc.as_ssb_kind_str(), "access_bound");
    }

    // -----------------------------------------------------------------------
    // VecReasoningBudgetEmitter
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn vec_emitter_accumulates_rows() {
        let emitter = VecReasoningBudgetEmitter::new();
        let payload = ReasoningBudgetScaledPayload {
            kind: "depth_bound".into(),
            role: "executor".into(),
            base_calls: 8,
            serial_depth: 3,
            scaled_calls: 14,
            ceiling: 24,
            seed: 42,
            // I-26 extension fields — None on GRC-dispatch rows (H-42 path).
            depth_level: None,
            budget_pct: None,
            plateau_fired: None,
        };
        emitter.emit_scaled(payload.clone()).await.expect("emit ok");
        let rows = emitter.rows().await;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, "depth_bound");
        assert_eq!(rows[0].serial_depth, 3);
    }
}
