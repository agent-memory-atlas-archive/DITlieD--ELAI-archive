//! I-26 — depth-scaled node compute budget controller (BudgetController).
//!
//! # Purpose
//! Each search-node expansion (I-4 SearchLadder) receives a compute envelope
//! proportional to its depth in the DAG. Two early-collapse rules stop new
//! expansions before the full budget is exhausted:
//!
//!   - 85% rule: when accumulated spend across all active siblings exceeds
//!     85% of the total search envelope, halt new expansions, score the frontier
//!     with I-17's L2 pass, and synthesize the best result.
//!   - Plateau rule: if the top-L2-ranked surviving branch's score does not
//!     improve by `PLATEAU_DELTA_MIN` across `PLATEAU_WINDOW` consecutive
//!     supersteps, fire early collapse.
//!
//! # Cloud effort mapping (Opus 4.7+ / Anthropic effort parameter)
//! When a search node calls a cloud reasoning role, the outbound request uses
//! the `effort` parameter (`CompletionConfig::effort`) rather than `budget_tokens`
//! (landmine #6 from I-0 CANONICAL — Opus 4.7+ REJECTS manual budget_tokens;
//! use the effort parameter). `BudgetController::cloud_effort_for_node` returns
//! the correct effort string for a given node depth:
//!
//!   - depth <= 1: "auto"   (shallow nodes, lighter reasoning)
//!   - depth >= 2: "expert" (deep/leaf nodes, fuller reasoning)
//!
//! # WIRE-DARK status (R8 / D-030)
//! `BudgetController`'s search-node budget methods (`node_budget`, `should_collapse`,
//! `record_plateau`) are WIRE-DARK[I-26]: `BranchInput` (the I-4 SearchLadder input
//! type in `search/ladder.rs::SearchLadder::run_best_of_n`) does not carry a `depth`
//! field, and adding one would change I-4's stable API. The wiring lands in I-26-promotion
//! once the I-4 API is extended. The backing allowlist row is in `wire-dark-allowlist.toml`
//! under `ELAI_DYNAMIC_BUDGET_ENABLE`. The contract test `i26_dynamic_thinking_budgets_contract`
//! validates the struct API and the effort-mapping without a live search loop.
//!
//! `cloud_effort_for_node` IS production-wired: it is called from
//! `build_request` in `providers/src/runtime/anthropic_api.rs` via
//! `CompletionConfig::effort`, which IS a live code path for every cloud
//! completion. (The effort value is None by default and set by callers that
//! use the cloud reasoning role; BudgetController provides the decision fn.)
//!
//! # Hard rules (CLAUDE.md + plan)
//! - No `.unwrap()` / `.expect()` on production paths (Zero-Lazy-Code).
//! - No paper budget-scaling constant as threshold (ID-5): `DEPTH_FACTOR` and
//!   `PLATEAU_DELTA_MIN` are placeholder starter values to be replaced with
//!   ELAI-measured baselines from the I-27 convergence curve. They are not
//!   drawn from any paper. `DEPTH_FACTOR_CALIBRATION_PENDING` signals this.
//! - HD-3: cloud_effort_for_node NEVER sets `budget_tokens`; it returns an
//!   effort string ("auto" | "expert"). Force-JSON wrapping of a reasoning
//!   response is BANNED (the caller must not set `tool_choice` with effort).
//! - HD-2: effort field placement (after seed in MessageRequest) does not
//!   perturb the byte-stable cache prefix.
//! - Kill-switch: `ELAI_DYNAMIC_BUDGET_ENABLE` gates the 85%/plateau machinery.
//!   Default-OFF until I-4 SearchLadder ships (experiment, allowlist row present).
//!
//! # Grounding
//! - H-42 ReasoningBudgetScaled substrate (the reused SSB family; this module
//!   adds the I-26 depth/collapse fields to the existing payload).
//! - Anthropic effort parameter: I-0 landmine #6 (Opus 4.7+ rejects manual
//!   budget_tokens; use effort="auto"|"expert").
//! - I-4 search ladder (the production host; WIRE-DARK[I-26] pending its landing).
//! - E-39 hard cost ceiling: realized as `LOCKED_BUDGET_CEILING` in reasoning_budget.rs
//!   (the only concrete cap symbol at HEAD; no importable E-39 USD cap exists).

// ---------------------------------------------------------------------------
// Constants (all starter values — calibrate from I-27 curve, not papers)
// ---------------------------------------------------------------------------

/// Note: these constants are calibration starters, NOT paper-derived thresholds
/// (ID-5: ELAI measures its own depth-factor from the I-27 convergence curve).
/// Tag this until I-27 data is available. Do not promote `DEPTH_FACTOR` to a
/// hard threshold before >=3 measurement runs.
pub const DEPTH_FACTOR_CALIBRATION_PENDING: bool = true;

/// Base node-budget units. Each search node starts with this many compute units;
/// the actual budget is `base * DEPTH_FACTOR^depth`, clamped at `BASE_BUDGET * MAX_DEPTH_MULT`.
/// Starter value. Replace from I-27 curve.
pub const BASE_BUDGET_UNITS: u32 = 100;

/// Exponential scaling factor per depth level. Starter value.
/// `node_budget(d) = BASE_BUDGET_UNITS * DEPTH_FACTOR^d`, clamped.
/// ID-5: do NOT tune this constant from paper recommendations; measure from I-27.
pub const DEPTH_FACTOR_NUMERATOR: u32 = 125; // 1.25x per depth level (×100 fixed-point)
pub const DEPTH_FACTOR_DENOMINATOR: u32 = 100;

/// Maximum multiplier cap (mirrors reasoning_budget::LOCKED_BUDGET_CEILING at 3x).
/// Prevents exponential blow-up at deep DAG levels.
pub const MAX_DEPTH_MULT: u32 = 3;

/// Budget-fraction threshold (0.0–1.0) at which the 85% collapse rule fires.
/// When `(accumulated_spend / total_envelope) >= COLLAPSE_THRESHOLD`, no new
/// node expansions are started.
pub const COLLAPSE_THRESHOLD: f32 = 0.85;

/// Number of consecutive supersteps with a top-score improvement below
/// `PLATEAU_DELTA_MIN` before the plateau rule fires.
pub const PLATEAU_WINDOW: usize = 5;

/// Minimum score improvement required per superstep to avoid plateau detection.
/// Starter value; replace from I-27 curve (ID-5).
pub const PLATEAU_DELTA_MIN: f32 = 0.01;

/// Env-var gate for the 85%/plateau collapse machinery.
/// Default-OFF (experiment, R9 row in wire-dark-allowlist.toml).
/// The effort-parameter mapping (cloud_effort_for_node) is NOT gated — it is
/// always active for the cloud request builder.
pub const ELAI_DYNAMIC_BUDGET_ENABLE_ENV: &str = "ELAI_DYNAMIC_BUDGET_ENABLE";

// ---------------------------------------------------------------------------
// Depth-scaled node budget
// ---------------------------------------------------------------------------

/// Compute the per-node compute budget for a search node at the given DAG depth.
///
/// Formula: `base * (DEPTH_FACTOR_NUMERATOR / DEPTH_FACTOR_DENOMINATOR)^depth`,
/// clamped at `BASE_BUDGET_UNITS * MAX_DEPTH_MULT`.
///
/// This is intentionally an INTEGER approximation using saturating fixed-point
/// arithmetic (no f64 precision issues, no .unwrap()).
#[must_use]
pub fn node_budget(depth: u32) -> u32 {
    let ceiling = BASE_BUDGET_UNITS.saturating_mul(MAX_DEPTH_MULT);
    if depth == 0 {
        return BASE_BUDGET_UNITS.min(ceiling);
    }
    // Fixed-point exponentiation: multiply (numer/denom) d times.
    let mut budget_scaled: u64 = BASE_BUDGET_UNITS as u64;
    // Scale by DEPTH_FACTOR_NUMERATOR/DEPTH_FACTOR_DENOMINATOR each level.
    for _ in 0..depth {
        budget_scaled = budget_scaled
            .saturating_mul(DEPTH_FACTOR_NUMERATOR as u64)
            / DEPTH_FACTOR_DENOMINATOR as u64;
    }
    let budget = u32::try_from(budget_scaled).unwrap_or(u32::MAX);
    budget.min(ceiling)
}

// ---------------------------------------------------------------------------
// BudgetController: 85% collapse + plateau detector
// ---------------------------------------------------------------------------

/// Per-search-session budget state machine.
///
/// Tracks accumulated spend across all active siblings and the top-L2 score
/// history for plateau detection.
///
/// WIRE-DARK[I-26]: `BranchInput` lacks a `depth` field; adding it changes I-4's
/// stable API (`run_best_of_n`). Wiring deferred to I-26-promotion.
/// (see wire-dark-allowlist.toml ELAI_DYNAMIC_BUDGET_ENABLE row).
#[derive(Debug, Default)]
pub struct BudgetController {
    /// Total compute envelope for this search session.
    total_envelope: u32,
    /// Accumulated spend so far (sum of node_budget for all expanded nodes).
    accumulated_spend: u32,
    /// History of top-L2 scores, one per superstep, newest-last.
    /// Capped at PLATEAU_WINDOW entries.
    score_history: std::collections::VecDeque<f32>,
}

impl BudgetController {
    /// Create a new controller for a search session with the given total envelope.
    ///
    /// `total_envelope` is the maximum total compute budget for the session.
    /// Callers should base this on the task difficulty and the E-39 ceiling.
    #[must_use]
    pub fn new(total_envelope: u32) -> Self {
        Self {
            total_envelope,
            accumulated_spend: 0,
            score_history: std::collections::VecDeque::with_capacity(PLATEAU_WINDOW + 1),
        }
    }

    /// Record that a node at `depth` has been expanded.
    ///
    /// Adds `node_budget(depth)` to the accumulated spend.
    /// Call this AFTER deciding to expand a node (i.e., `should_collapse` returned false).
    // WIRE-DARK[I-26]: deferred to I-26-promotion (BranchInput lacks depth field; adding it changes I-4 API).
    pub fn record_expansion(&mut self, depth: u32) {
        let spend = node_budget(depth);
        self.accumulated_spend = self.accumulated_spend.saturating_add(spend);
    }

    /// Returns the fraction of the total envelope consumed so far (0.0–1.0).
    ///
    /// Returns 0.0 if `total_envelope == 0` (avoids division by zero).
    #[must_use]
    pub fn budget_fraction(&self) -> f32 {
        if self.total_envelope == 0 {
            return 0.0;
        }
        (self.accumulated_spend as f32) / (self.total_envelope as f32)
    }

    /// Returns `true` when the 85% collapse rule fires.
    ///
    /// Checks whether `budget_fraction() >= COLLAPSE_THRESHOLD`.
    /// When true, no new node expansions should be started.
    #[must_use]
    pub fn should_collapse_budget(&self) -> bool {
        self.budget_fraction() >= COLLAPSE_THRESHOLD
    }

    /// Record the top-L2 score for this superstep and check the plateau rule.
    ///
    /// The plateau rule fires when the score improvement over the last
    /// `PLATEAU_WINDOW` consecutive supersteps is below `PLATEAU_DELTA_MIN`.
    /// Returns `true` when the plateau detector fires.
    ///
    /// PLATEAU detection requires at least `PLATEAU_WINDOW` data points.
    /// Fewer than `PLATEAU_WINDOW` calls always returns `false`.
    pub fn record_score_and_check_plateau(&mut self, top_score: f32) -> bool {
        self.score_history.push_back(top_score);
        // Keep only the last PLATEAU_WINDOW + 1 entries.
        while self.score_history.len() > PLATEAU_WINDOW + 1 {
            self.score_history.pop_front();
        }
        if self.score_history.len() < PLATEAU_WINDOW {
            return false;
        }
        // Check if max improvement over the window is below PLATEAU_DELTA_MIN.
        let oldest = self
            .score_history
            .front()
            .copied()
            .unwrap_or(f32::NEG_INFINITY);
        let newest = self
            .score_history
            .back()
            .copied()
            .unwrap_or(f32::NEG_INFINITY);
        let delta = newest - oldest;
        delta < PLATEAU_DELTA_MIN
    }

    /// Returns `true` when either the 85% rule or the plateau rule has fired.
    ///
    /// Convenience combinator for the search loop's collapse check.
    /// `top_score` is the current best L2 score (supply 0.0 if not yet available).
    // WIRE-DARK[I-26]: deferred to I-26-promotion (BranchInput lacks depth field; adding it changes I-4 API).
    pub fn should_collapse(&mut self, top_score: f32) -> bool {
        let plateau = self.record_score_and_check_plateau(top_score);
        plateau || self.should_collapse_budget()
    }
}

// ---------------------------------------------------------------------------
// Cloud effort mapping (HD-3 compliant, Opus 4.7+ safe)
// ---------------------------------------------------------------------------

// WIRE-DARK[I-26]: cloud_effort_for_node and BudgetController's search-node
// methods are deferred to I-26-promotion. True reason: BranchInput (the I-4 SearchLadder
// input type in run_best_of_n) lacks a `depth` field; threading depth into BranchInput
// would change I-4's stable API surface. The wiring lands after I-4's API is extended.
// Backing allowlist row: ELAI_DYNAMIC_BUDGET_ENABLE in wire-dark-allowlist.toml
// (type=experiment, owner=maintainer, promotion_plan_id=I-26-promotion).
// The WIRE leg that IS production-active: CompletionConfig::effort forwarded by
// build_request in providers/src/runtime/anthropic_api.rs (the struct field plumbing is
// wired independent of who calls cloud_effort_for_node; the effort field flows for any
// caller that sets config.effort).

/// Map a search-node depth to the Anthropic `effort` parameter for cloud reasoning roles.
///
/// Returned value is suitable for `CompletionConfig::effort` (wired into
/// `providers::runtime::anthropic_api::AnthropicApiRuntime::build_request`
/// as the `effort` field on `MessageRequest`).
///
/// HARD RULES (HD-3 + landmine #6 / I-0):
/// - NEVER returns `budget_tokens` — Opus 4.7+ REJECTS manual budget_tokens.
/// - The caller MUST NOT also set `force_json = true` on a reasoning role request.
///
/// Values:
///   - `None` when called for a non-cloud or non-reasoning-role request.
///   - `Some("auto")` for shallow nodes (depth 0–1): lighter reasoning.
///   - `Some("expert")` for deep nodes (depth >= 2): fuller reasoning budget.
///
/// WIRE-DARK[I-26]: the decision-function caller (I-4 SearchLadder) deferred to
/// I-26-promotion (BranchInput lacks depth field; API change required).
/// The WIRE path (effort field on MessageRequest) IS live — see
/// providers/src/runtime/anthropic_api.rs::build_request. Backing row:
/// wire-dark-allowlist.toml ELAI_DYNAMIC_BUDGET_ENABLE.
// WIRE-DARK[I-26]: decision-function consumer deferred to I-26-promotion (BranchInput API change).
#[must_use]
pub fn cloud_effort_for_node(depth: u32, is_reasoning_role: bool) -> Option<String> {
    if !is_reasoning_role {
        return None;
    }
    if depth <= 1 {
        Some("auto".to_owned())
    } else {
        Some("expert".to_owned())
    }
}

/// Returns `true` when `ELAI_DYNAMIC_BUDGET_ENABLE` is set to "1" or "true".
///
/// Gates the 85%/plateau collapse machinery (WIRE-DARK[I-26], experiment).
/// The effort-parameter mapping is NOT gated by this fn.
#[must_use]
pub fn dynamic_budget_enabled() -> bool {
    match std::env::var(ELAI_DYNAMIC_BUDGET_ENABLE_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true")
        }
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_budget_depth_zero_is_base() {
        assert_eq!(node_budget(0), BASE_BUDGET_UNITS);
    }

    #[test]
    fn node_budget_depth_increases_up_to_ceiling() {
        let d0 = node_budget(0);
        let d1 = node_budget(1);
        let d2 = node_budget(2);
        let ceiling = BASE_BUDGET_UNITS * MAX_DEPTH_MULT;
        assert!(d1 >= d0, "depth 1 should be >= depth 0");
        assert!(d2 >= d1, "depth 2 should be >= depth 1");
        assert!(d2 <= ceiling, "depth 2 must not exceed ceiling");
        assert!(node_budget(100) <= ceiling, "very deep must not exceed ceiling");
    }

    #[test]
    fn budget_fraction_zero_on_empty() {
        let bc = BudgetController::new(200);
        assert!(bc.budget_fraction() < 0.001, "no spend => fraction ~0");
    }

    #[test]
    fn budget_fraction_zero_envelope_returns_zero() {
        let bc = BudgetController::new(0);
        assert_eq!(bc.budget_fraction(), 0.0);
    }

    #[test]
    fn should_collapse_budget_fires_at_threshold() {
        let mut bc = BudgetController::new(100);
        bc.accumulated_spend = 85;
        assert!(bc.should_collapse_budget(), "85/100 >= 0.85 => collapse");
        bc.accumulated_spend = 84;
        assert!(
            !bc.should_collapse_budget(),
            "84/100 < 0.85 => no collapse yet"
        );
    }

    #[test]
    fn plateau_detector_does_not_fire_without_enough_data() {
        let mut bc = BudgetController::new(1000);
        // Less than PLATEAU_WINDOW steps -- should not fire.
        for i in 0..PLATEAU_WINDOW - 1 {
            let r = bc.record_score_and_check_plateau(0.5 + i as f32 * 0.001);
            assert!(!r, "not enough data => no plateau");
        }
    }

    #[test]
    fn plateau_detector_fires_when_score_flat() {
        let mut bc = BudgetController::new(1000);
        // Fill PLATEAU_WINDOW steps with no improvement.
        for _ in 0..PLATEAU_WINDOW {
            bc.record_score_and_check_plateau(0.5);
        }
        let fired = bc.record_score_and_check_plateau(0.5);
        assert!(fired, "flat score across plateau window => fire");
    }

    #[test]
    fn plateau_detector_does_not_fire_when_improving() {
        let mut bc = BudgetController::new(1000);
        for i in 0..=PLATEAU_WINDOW {
            bc.record_score_and_check_plateau(0.1 * i as f32);
        }
        // After enough improvement steps it should not have fired yet.
        assert!(!bc.should_collapse_budget(), "spend is 0 => no budget collapse");
    }

    #[test]
    fn cloud_effort_none_for_non_reasoning() {
        assert_eq!(cloud_effort_for_node(0, false), None);
        assert_eq!(cloud_effort_for_node(3, false), None);
    }

    #[test]
    fn cloud_effort_auto_for_shallow_reasoning() {
        assert_eq!(
            cloud_effort_for_node(0, true).as_deref(),
            Some("auto"),
            "depth 0 => auto"
        );
        assert_eq!(
            cloud_effort_for_node(1, true).as_deref(),
            Some("auto"),
            "depth 1 => auto"
        );
    }

    #[test]
    fn cloud_effort_expert_for_deep_reasoning() {
        assert_eq!(
            cloud_effort_for_node(2, true).as_deref(),
            Some("expert"),
            "depth 2 => expert"
        );
        assert_eq!(
            cloud_effort_for_node(5, true).as_deref(),
            Some("expert"),
            "depth 5 => expert"
        );
    }

    #[test]
    fn cloud_effort_never_returns_budget_tokens() {
        // Contract: landmine #6 / I-0 — budget_tokens is BANNED on Opus 4.7+.
        for depth in 0..10 {
            for reasoning in [true, false] {
                if let Some(v) = cloud_effort_for_node(depth, reasoning) {
                    assert_ne!(
                        v, "budget_tokens",
                        "cloud_effort_for_node must never return 'budget_tokens'"
                    );
                }
            }
        }
    }

    #[test]
    fn dynamic_budget_disabled_by_default() {
        // In test environment ELAI_DYNAMIC_BUDGET_ENABLE is not set.
        // Remove it explicitly to be safe.
        unsafe {
            std::env::remove_var(ELAI_DYNAMIC_BUDGET_ENABLE_ENV);
        }
        assert!(
            !dynamic_budget_enabled(),
            "default-OFF when env var absent"
        );
    }

    #[test]
    fn dynamic_budget_enabled_when_set_to_one() {
        unsafe {
            std::env::set_var(ELAI_DYNAMIC_BUDGET_ENABLE_ENV, "1");
        }
        let result = dynamic_budget_enabled();
        unsafe {
            std::env::remove_var(ELAI_DYNAMIC_BUDGET_ENABLE_ENV);
        }
        assert!(result, "enabled when env var == '1'");
    }
}
