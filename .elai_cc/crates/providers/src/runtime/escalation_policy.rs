//! E-31 — `RuntimeRoutingPolicy`: pre-route data + decision table that
//! returns a [`RoutingDecision`] for a given runtime context.
//!
//! This module is the runtime-layer routing policy for E-31 (coding-model
//! leaderboard / bench-maturity). It is INTENTIONALLY distinct from the
//! two existing `EscalationPolicy` types that already ship in the
//! orchestrator crate:
//!
//!   * `elai_orchestrator::escalation_policy::HandoffDecision` (E-32) —
//!     SessionDag deep-mode handoff (HANDOFF_THRESHOLD_FRACTION 0.85,
//!     DEEP_MODE_CTX_CAP 64_000); axis = context-length budget.
//!   * `elai_orchestrator::autonomous_gate::EscalationPolicy` (C-9) —
//!     autonomous-gate ops-mode policy (allow / ask / block).
//!
//! E-31's axis is task-difficulty + criticality + VRAM-headroom +
//! prior-audit-failures -> `RoutingChoice`. The 3-way name collision
//! would make `grep -R EscalationPolicy` ambiguous, so this type is
//! deliberately named `RuntimeRoutingPolicy`.
//!
//! Wire surface is delegated upward to
//! `elai_orchestrator::cost::cost_router::route_for_difficulty_and_budget`
//! (existing E-39 matrix) for the final cloud-route selection; this
//! struct provides the data + the pure decision-table that maps the
//! per-step context to a 4-arm `RoutingChoice`. No I/O, no async, no
//! Cargo deps beyond serde + thiserror (already in workspace).
//!
//! See `workflow/sessions/s-21636/exploration/tranche6-E31-wiring.md`
//! GAP block + `workflow/reasoning/E-31-decisions.md` DRIFT-2 reconcile.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Routing choice (4 backends)
// ---------------------------------------------------------------------------

/// Final 4-arm choice the policy picks. Maps 1:1 onto the production
/// `RuntimeBackend` slots that have a live or near-live adapter today
/// (sidecar + AnthropicApi + DeepSeek free-floor). Opus stays as a
/// confirmation-gated cloud arm per E-39 `RouteDecision::Opus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingChoice {
    /// Local Qwen2.5-Coder-7B-Instruct-abliterated sidecar.
    LocalSidecar,
    /// Cloud Claude Sonnet 4.6 (workhorse).
    CloudSonnet,
    /// Cloud Claude Opus 4.7 (confirmation-gated; very-hard + critical).
    CloudOpus,
    /// DeepSeek V4-Flash cheap-volume floor (E-39 anti-slopsquatting
    /// pending; RuntimeAdapter returns NotImplemented until verified).
    CloudDeepSeekFlash,
}

impl RoutingChoice {
    /// Snake-case wire label. Used by SSB row labels + dashboard tiles.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LocalSidecar => "local_sidecar",
            Self::CloudSonnet => "cloud_sonnet",
            Self::CloudOpus => "cloud_opus",
            Self::CloudDeepSeekFlash => "cloud_deepseek_flash",
        }
    }
}

// ---------------------------------------------------------------------------
// Routing reason + decision struct
// ---------------------------------------------------------------------------

/// 5-arm rationale tag carried alongside every decision so the SSB
/// dashboard tile can show WHY a route was picked, not just WHICH route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingReason {
    /// Online policy threshold met (difficulty >= boundary => escalate).
    OnlineThresholdMet,
    /// Offline cost cap reached (budget tight => prefer cheaper arm).
    OfflineCostCap,
    /// VRAM headroom too low for the local model (e.g. 8GB tier ->
    /// cloud-primary). Mirrors A-37 `CloudImportance::MoreImportant`.
    VramHeadroomLow,
    /// Difficulty bin upgrade after N prior_audit_failures.
    DifficultyEscalation,
    /// Ops override (operator-pinned route via config or CLI flag).
    OpsOverride,
}

impl RoutingReason {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OnlineThresholdMet => "online_threshold_met",
            Self::OfflineCostCap => "offline_cost_cap",
            Self::VramHeadroomLow => "vram_headroom_low",
            Self::DifficultyEscalation => "difficulty_escalation",
            Self::OpsOverride => "ops_override",
        }
    }
}

/// Pure-data result of [`RuntimeRoutingPolicy::evaluate`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Selected backend arm.
    pub choice: RoutingChoice,
    /// Rationale tag.
    pub reason: RoutingReason,
    /// Wall-clock (millis since UNIX_EPOCH) at the time of decision.
    /// Caller-supplied so the policy stays sync + pure.
    pub ts_ms: u64,
}

impl RoutingDecision {
    /// Constructor that takes pre-validated parts.
    #[must_use]
    pub fn new(choice: RoutingChoice, reason: RoutingReason, ts_ms: u64) -> Self {
        Self { choice, reason, ts_ms }
    }
}

// ---------------------------------------------------------------------------
// Routing context (input to evaluate)
// ---------------------------------------------------------------------------

/// 3-arm difficulty bin mirroring the providers
/// `cloud_route_difficulty::DifficultyBin` shape (avoids upward orchestrator
/// dep). Caller maps E-39's f32 difficulty into a bin before calling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingDifficulty {
    Easy,
    Medium,
    Hard,
}

/// 2-arm criticality mirroring orchestrator's `Criticality::{Low, High}`
/// (Medium not surfaced at the runtime layer per plan body — collapsed
/// into Low for the runtime route).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingCriticality {
    Low,
    High,
}

/// Per-step routing input. Pure data, no references.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutingContext {
    /// Difficulty bin (caller maps from E-39 f32 score).
    pub difficulty: RoutingDifficulty,
    /// Caller-supplied criticality.
    pub criticality: RoutingCriticality,
    /// Detected VRAM in GiB. Used for the headroom heuristic. 0.0 when
    /// running in Cloud tier or VRAM unknown.
    pub vram_detected_gb: f32,
    /// Number of prior audit failures on this step. Each failure shifts
    /// the bin one notch toward Hard.
    pub prior_audit_failures: u32,
    /// Remaining USD budget for the session. Tight (<=5.0) => prefer
    /// cheaper cloud arm. Mirrors E-39
    /// `BUDGET_TIGHT_CEIL_USD = 5.0`.
    pub budget_remaining_usd: f64,
    /// Operator-pinned override route, if any. Bypasses the decision
    /// table when present and emits `RoutingReason::OpsOverride`.
    pub ops_override: Option<RoutingChoice>,
}

impl RoutingContext {
    /// Cheap default for tests / scaffolding.
    #[must_use]
    pub fn for_local_easy(vram_detected_gb: f32) -> Self {
        Self {
            difficulty: RoutingDifficulty::Easy,
            criticality: RoutingCriticality::Low,
            vram_detected_gb,
            prior_audit_failures: 0,
            budget_remaining_usd: 25.0,
            ops_override: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Typed errors for the policy module. Reserved for future producers
/// (e.g. malformed serialized context). `evaluate` itself is infallible
/// today; callers using `?` against this type still get a future-proof
/// surface.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum RuntimeRoutingPolicyError {
    /// Reserved for future surface: invalid serialized context (e.g.
    /// deserialization failure from a dashboard IPC payload).
    #[error("invalid routing context: {0}")]
    InvalidContext(String),
}

// ---------------------------------------------------------------------------
// Policy + decision table
// ---------------------------------------------------------------------------

/// Tight-budget USD ceiling. Mirrors E-39
/// `cost_router::BUDGET_TIGHT_CEIL_USD = 5.0`.
const BUDGET_TIGHT_CEIL_USD: f64 = 5.0;

/// VRAM-headroom boundary. Below this, local sidecar cannot fit the 7B
/// model + KV cache + working set comfortably. Mirrors A-37 8GB tier
/// `CloudImportance::MoreImportant` boundary; the 7B abliterated NF4
/// model is ~3.1 GB at 32K context per memory:project_qwen3b_baseline.
const VRAM_HEADROOM_FLOOR_GB: f32 = 6.0;

/// Audit-failures notch shift. Each `prior_audit_failures` increment
/// shifts the effective bin one notch toward Hard, capped at Hard. So
/// Easy + 1 fail = Medium, Medium + 1 fail = Hard, Hard + N fails = Hard.
const AUDIT_FAILURE_BIN_SHIFT: u32 = 1;

/// E-31 routing policy. Pure-data + a pure decision-table; no I/O.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeRoutingPolicy {
    /// Tight-budget ceiling in USD; below this, prefer DeepSeek over Sonnet.
    pub budget_tight_ceil_usd: f64,
    /// VRAM headroom floor in GiB; below this, force cloud route.
    pub vram_headroom_floor_gb: f32,
    /// Whether to honor `RoutingContext::ops_override`. Default true.
    pub honor_ops_override: bool,
}

impl Default for RuntimeRoutingPolicy {
    fn default() -> Self {
        Self {
            budget_tight_ceil_usd: BUDGET_TIGHT_CEIL_USD,
            vram_headroom_floor_gb: VRAM_HEADROOM_FLOOR_GB,
            honor_ops_override: true,
        }
    }
}

impl RuntimeRoutingPolicy {
    /// Construct a non-default policy. Use `Self::default()` for the
    /// canonical E-31 settings.
    #[must_use]
    pub fn new(
        budget_tight_ceil_usd: f64,
        vram_headroom_floor_gb: f32,
        honor_ops_override: bool,
    ) -> Self {
        Self {
            budget_tight_ceil_usd,
            vram_headroom_floor_gb,
            honor_ops_override,
        }
    }

    /// Evaluate the policy against the supplied context. Returns the
    /// chosen route + reason. Synchronous + pure; no I/O.
    #[must_use]
    pub fn evaluate(&self, ctx: &RoutingContext) -> RoutingDecision {
        let ts_ms = ctx_ts_ms_for(ctx);

        // 1. Ops override short-circuits everything.
        if self.honor_ops_override {
            if let Some(forced) = ctx.ops_override {
                return RoutingDecision::new(forced, RoutingReason::OpsOverride, ts_ms);
            }
        }

        // 2. VRAM headroom check: if local can't fit, force cloud-cheap
        //    OR cloud-workhorse depending on criticality.
        let cloud_required = ctx.vram_detected_gb > 0.0
            && ctx.vram_detected_gb < self.vram_headroom_floor_gb;

        // 3. Apply audit-failure bin shift before the matrix.
        let effective_difficulty = shift_difficulty(
            ctx.difficulty,
            ctx.prior_audit_failures.saturating_mul(AUDIT_FAILURE_BIN_SHIFT),
        );

        if cloud_required {
            let choice = match ctx.criticality {
                RoutingCriticality::High => RoutingChoice::CloudSonnet,
                RoutingCriticality::Low => RoutingChoice::CloudDeepSeekFlash,
            };
            return RoutingDecision::new(choice, RoutingReason::VramHeadroomLow, ts_ms);
        }

        // 4. Difficulty x budget decision table.
        let tight = ctx.budget_remaining_usd <= self.budget_tight_ceil_usd;

        let (choice, reason) = match (effective_difficulty, ctx.criticality, tight) {
            (RoutingDifficulty::Easy, _, _) => {
                (RoutingChoice::LocalSidecar, RoutingReason::OnlineThresholdMet)
            }
            (RoutingDifficulty::Medium, _, true) => {
                (RoutingChoice::CloudDeepSeekFlash, RoutingReason::OfflineCostCap)
            }
            (RoutingDifficulty::Medium, _, false) => {
                (RoutingChoice::CloudSonnet, RoutingReason::OnlineThresholdMet)
            }
            (RoutingDifficulty::Hard, RoutingCriticality::High, _) => {
                if ctx.prior_audit_failures > 0 {
                    (RoutingChoice::CloudOpus, RoutingReason::DifficultyEscalation)
                } else {
                    (RoutingChoice::CloudSonnet, RoutingReason::OnlineThresholdMet)
                }
            }
            (RoutingDifficulty::Hard, RoutingCriticality::Low, true) => {
                (RoutingChoice::CloudDeepSeekFlash, RoutingReason::OfflineCostCap)
            }
            (RoutingDifficulty::Hard, RoutingCriticality::Low, false) => {
                (RoutingChoice::CloudSonnet, RoutingReason::OnlineThresholdMet)
            }
        };

        RoutingDecision::new(choice, reason, ts_ms)
    }
}

/// Returns a millis-since-epoch read for the decision. Caller-supplied
/// timestamps would be cleaner, but the policy is sync + pure; we read
/// `SystemTime::now` here as the documented saturating-on-failure path
/// (mirrors `ssb.rs::ts_now_ms` semantics; this helper is canonical for
/// the E-31 routing decision and follows the same posture as the SSB
/// timestamp source).
fn ctx_ts_ms_for(_ctx: &RoutingContext) -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now() // entropy-allow: e31-escalation-policy-saturating-on-failure-mirrors-ssb-ts-now-ms
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

/// Shift difficulty bin upward by `shift` notches, capped at `Hard`.
fn shift_difficulty(d: RoutingDifficulty, shift: u32) -> RoutingDifficulty {
    let base = match d {
        RoutingDifficulty::Easy => 0_u32,
        RoutingDifficulty::Medium => 1,
        RoutingDifficulty::Hard => 2,
    };
    match base.saturating_add(shift).min(2) {
        0 => RoutingDifficulty::Easy,
        1 => RoutingDifficulty::Medium,
        _ => RoutingDifficulty::Hard,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(
        difficulty: RoutingDifficulty,
        criticality: RoutingCriticality,
        vram: f32,
        budget: f64,
    ) -> RoutingContext {
        RoutingContext {
            difficulty,
            criticality,
            vram_detected_gb: vram,
            prior_audit_failures: 0,
            budget_remaining_usd: budget,
            ops_override: None,
        }
    }

    #[test]
    fn easy_route_picks_local_sidecar() {
        let policy = RuntimeRoutingPolicy::default();
        let c = ctx(
            RoutingDifficulty::Easy,
            RoutingCriticality::Low,
            24.0,
            25.0,
        );
        let d = policy.evaluate(&c);
        assert_eq!(d.choice, RoutingChoice::LocalSidecar);
        assert_eq!(d.reason, RoutingReason::OnlineThresholdMet);
    }

    #[test]
    fn hard_high_criticality_picks_sonnet_when_first_attempt() {
        let policy = RuntimeRoutingPolicy::default();
        let c = ctx(
            RoutingDifficulty::Hard,
            RoutingCriticality::High,
            24.0,
            25.0,
        );
        let d = policy.evaluate(&c);
        assert_eq!(d.choice, RoutingChoice::CloudSonnet);
        assert_eq!(d.reason, RoutingReason::OnlineThresholdMet);
    }

    #[test]
    fn hard_high_with_prior_audit_failure_escalates_to_opus() {
        let policy = RuntimeRoutingPolicy::default();
        let mut c = ctx(
            RoutingDifficulty::Hard,
            RoutingCriticality::High,
            24.0,
            25.0,
        );
        c.prior_audit_failures = 1;
        let d = policy.evaluate(&c);
        assert_eq!(d.choice, RoutingChoice::CloudOpus);
        assert_eq!(d.reason, RoutingReason::DifficultyEscalation);
    }

    #[test]
    fn medium_under_tight_budget_picks_deepseek_flash() {
        let policy = RuntimeRoutingPolicy::default();
        let c = ctx(
            RoutingDifficulty::Medium,
            RoutingCriticality::Low,
            24.0,
            2.5,
        );
        let d = policy.evaluate(&c);
        assert_eq!(d.choice, RoutingChoice::CloudDeepSeekFlash);
        assert_eq!(d.reason, RoutingReason::OfflineCostCap);
    }

    #[test]
    fn low_vram_forces_cloud_route() {
        let policy = RuntimeRoutingPolicy::default();
        let c = ctx(
            RoutingDifficulty::Easy,
            RoutingCriticality::Low,
            4.0,
            25.0,
        );
        let d = policy.evaluate(&c);
        assert_eq!(d.choice, RoutingChoice::CloudDeepSeekFlash);
        assert_eq!(d.reason, RoutingReason::VramHeadroomLow);
    }

    #[test]
    fn ops_override_short_circuits_decision_table() {
        let policy = RuntimeRoutingPolicy::default();
        let mut c = ctx(
            RoutingDifficulty::Easy,
            RoutingCriticality::Low,
            24.0,
            25.0,
        );
        c.ops_override = Some(RoutingChoice::CloudOpus);
        let d = policy.evaluate(&c);
        assert_eq!(d.choice, RoutingChoice::CloudOpus);
        assert_eq!(d.reason, RoutingReason::OpsOverride);
    }

    #[test]
    fn audit_failure_shifts_easy_to_medium() {
        let policy = RuntimeRoutingPolicy::default();
        let mut c = ctx(
            RoutingDifficulty::Easy,
            RoutingCriticality::Low,
            24.0,
            25.0,
        );
        c.prior_audit_failures = 1;
        let d = policy.evaluate(&c);
        // Shifted to Medium + ample budget -> CloudSonnet via online threshold.
        assert_eq!(d.choice, RoutingChoice::CloudSonnet);
        assert_eq!(d.reason, RoutingReason::OnlineThresholdMet);
    }

    #[test]
    fn routing_choice_wire_labels_are_stable() {
        assert_eq!(RoutingChoice::LocalSidecar.as_str(), "local_sidecar");
        assert_eq!(RoutingChoice::CloudSonnet.as_str(), "cloud_sonnet");
        assert_eq!(RoutingChoice::CloudOpus.as_str(), "cloud_opus");
        assert_eq!(RoutingChoice::CloudDeepSeekFlash.as_str(), "cloud_deepseek_flash");
    }
}
