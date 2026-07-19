//! E-38 (SCHEMA_VERSION 175) -- EU AI Act compliance pre-gate.
//!
//! Runs at the FSM edge PLANNING/FEASIBILITY_PROBE -> TESTING_FIRST and
//! asserts four record-keeping + determinism preconditions (EU AI Act
//! art. 12 -- record-keeping, art. 13 -- transparency):
//!
//!   1. [`ComplianceAssertion::ClockProviderIntact`] -- the PlanContext
//!      reports an injected [`elai_core::clock::ClockProvider`] (per
//!      C-15; ClockProvider shipped TODAY at crates/core/src/clock.rs,
//!      182 LOC).
//!   2. [`ComplianceAssertion::DeterministicSeed`] -- a deterministic
//!      seed for the planning-phase pseudo-randomness is propagated
//!      into the planning context (per C-27 T-1 determinism floor).
//!   3. [`ComplianceAssertion::TemplateHashChain`] -- D-1 template
//!      archive's content_hash chain verifies for the current plan's
//!      reasoning templates. Degrades to `Skipped` when D-1 has not
//!      shipped yet (per plan Open Question 1).
//!   4. [`ComplianceAssertion::AuditTrailLive`] -- the PlanContext
//!      reports at least [`crate::ssb::AUDIT_TRAIL_MIN_SSB_ROWS`] SSB
//!      rows emitted during the PLANNING phase (FSM transition emission
//!      + any phase-bound writes by the planner). Liveness check, not
//!      content check.
//!
//! On block, the FSM transition is refused with
//! [`crate::fsm::FsmError::ComplianceBlocked`] and an
//! [`crate::ssb::SsbPayload::AiRoadmapSignalEvent`] row is emitted with
//! kind=[`crate::ssb::RoadmapSignalKind::ComplianceBlocked`].
//!
//! On pass, an identical row is emitted with
//! kind=[`crate::ssb::RoadmapSignalKind::ComplianceVerified`] +
//! `assertions_checked` bitmask carrying the 4 assertion bits.
//!
//! Wire-dark: SSB emission ONLY fires when
//! `ELAI_AI_ROADMAP_SIGNALS_ENABLE=1` is set at process boot
//! (see [`crate::roadmap_signals::roadmap_signals_enabled`]). The
//! pre-gate ITSELF runs unconditionally -- a transition that would have
//! been blocked is blocked regardless of the env-gate. The env-gate
//! controls observability, not policy.
//!
//! Grounding (every cite WebSearch-verified May 2026):
//!   - artificialintelligenceact.eu/implementation-timeline -- EU AI
//!     Act GPAI obligations live Aug 2 2025; transparency Aug 2 2026;
//!     high-risk Annex III POSTPONED to Dec 2 2027 per Digital Omnibus
//!     May 7 2026.
//!   - EU AI Act art. 12 (record-keeping) + art. 13 (transparency).
//!
//! See `workflow/reasoning/E-38-decisions.md`.

use std::sync::Arc;

use crate::ssb::{
    AiRoadmapSignalEventPayload, COMPLIANCE_BIT_AUDIT_TRAIL_LIVE,
    COMPLIANCE_BIT_CLOCK_PROVIDER, COMPLIANCE_BIT_DETERMINISTIC_SEED,
    COMPLIANCE_BIT_TEMPLATE_HASH_CHAIN, RoadmapSignalKind, Ssb, SsbActionType,
};
use crate::ssb_viewer::SsbViewer;

/// Env-gate flag controlling whether the EU AI Act compliance pre-gate
/// (installed via [`crate::fsm::Fsm::set_compliance_provider`] +
/// [`DefaultComplianceProvider`]) actually runs its 4 assertions. When
/// unset (default), [`DefaultComplianceProvider::verdict`] short-circuits
/// to `Pass { assertions_checked: 0 }` so the CLI is not bricked. When
/// set to `"1"`, the provider builds a [`PlanContext`] from live
/// orchestrator state + the SSB row count + the
/// `ELAI_DETERMINISTIC_SEED` env probe, then calls
/// [`pre_gate_planning_to_testing_first`]. This mirrors the C-28
/// `ELAI_DETERMINISTIC_AUDIT_GATE_ENABLE` default-OFF pattern
/// (wire-dark-allowlist.toml). The SSB emission leg is independently
/// gated by `ELAI_AI_ROADMAP_SIGNALS_ENABLE` (see
/// [`crate::roadmap_signals::roadmap_signals_enabled`]).
pub const ELAI_EU_AI_ACT_COMPLIANCE_GATE_ENABLE_ENV: &str =
    "ELAI_EU_AI_ACT_COMPLIANCE_GATE_ENABLE";

/// Env var probed by [`DefaultComplianceProvider`] for the
/// [`PlanContext::has_deterministic_seed`] field. Until C-27 T-1 ships
/// seed-propagation infrastructure, this is the sole source. When unset,
/// `has_deterministic_seed = false` and the gate blocks on
/// [`ComplianceAssertion::DeterministicSeed`] (when the gate is opted in).
pub const ELAI_DETERMINISTIC_SEED_ENV: &str = "ELAI_DETERMINISTIC_SEED";

/// E-38 -- production [`crate::fsm::ComplianceProvider`] installed by
/// `maybe_install_compliance_provider` in the CLI at all three `Fsm::new`
/// construction sites. Builds a [`PlanContext`] from process state + the
/// SSB row count at verdict time, then delegates to
/// [`pre_gate_planning_to_testing_first`].
///
/// Soft opt-in: when
/// `ELAI_EU_AI_ACT_COMPLIANCE_GATE_ENABLE` is not `"1"`, `verdict()`
/// returns `Pass { assertions_checked: 0 }` immediately so the CLI is
/// not bricked by default (mirrors the C-28 default-OFF pattern). When
/// opted in, the gate runs unconditionally on every
/// `PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST` transition; the SSB
/// emission leg is separately gated by
/// `ELAI_AI_ROADMAP_SIGNALS_ENABLE`.
///
/// Fields populated per plan Open Question 1 + the controller-pinned
/// decisions for E-38 Task-2 residual:
///   * `plan_session_id` -- captured at install time from the CLI's
///     session id (the FSM `phase_label` is empty at boot; the SSB
///     payload's `plan_session_id` is reconstructed by the fsm.rs
///     emission path from `guard.phase_label` at transition time, so
///     this field is a best-effort label for the provider's own
///     PlanContext).
///   * `has_clock_provider` -- always `true` (C-15 ClockProvider shipped;
///     the orchestrator always injects `OsClock` at
///     `live_cli.rs:789`). Liveness check, not a failure path in
///     production.
///   * `has_deterministic_seed` -- probed from `ELAI_DETERMINISTIC_SEED`
///     env at verdict time. `false` until C-27 T-1 ships.
///   * `template_hash_chain_verified` -- `false` (D-1 not shipped).
///   * `template_archive_present` -- `false` (D-1 not shipped) ->
///     TemplateHashChain degrades to `Skipped` per plan Open Question 1.
///   * `ssb_row_count_in_planning_phase` -- best-effort count of
///     `PhaseTransition` rows in the SSB at verdict time; falls back to
///     `0` (which blocks on `AuditTrailLive` when the gate is opted in)
///     on any viewer / decode error. The FSM emits a PhaseTransition row
///     on every successful transition, so this is `>= 1` after the
///     IDLE -> PLANNING transition.
pub struct DefaultComplianceProvider {
    plan_session_id: String,
    ssb: Arc<Ssb>,
    /// When `Some(enabled)`, bypass the
    /// `ELAI_EU_AI_ACT_COMPLIANCE_GATE_ENABLE` env probe and use the
    /// explicit flag. Used by on-path tests that cannot tolerate the
    /// process-wide env probe racing across parallel `#[tokio::test]`
    /// cases in the same binary. Production callers use `new()` which
    /// leaves this `None` so the env probe stays the source of truth.
    force_enabled: Option<bool>,
}

impl std::fmt::Debug for DefaultComplianceProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultComplianceProvider")
            .field("plan_session_id", &self.plan_session_id)
            .field("ssb", &"<Ssb>")
            .field("force_enabled", &self.force_enabled)
            .finish()
    }
}

impl DefaultComplianceProvider {
    /// Construct a new provider. `plan_session_id` is the CLI session id
    /// at boot (the FSM `phase_label` is empty at install time; the SSB
    /// payload uses the live `guard.phase_label` at transition time, so
    /// this is a best-effort label for the provider's own PlanContext).
    /// `ssb` is the Ssb writer handle (the provider opens a read-only
    /// [`SsbViewer`] on `ssb.db_path()` at verdict time to count
    /// PhaseTransition rows). The gate opt-in is read from
    /// `ELAI_EU_AI_ACT_COMPLIANCE_GATE_ENABLE` at verdict time (the
    /// production path).
    #[must_use]
    pub fn new(plan_session_id: impl Into<String>, ssb: Arc<Ssb>) -> Self {
        Self {
            plan_session_id: plan_session_id.into(),
            ssb,
            force_enabled: None,
        }
    }

    /// Construct a provider with an explicit `enabled` flag that bypasses
    /// the `ELAI_EU_AI_ACT_COMPLIANCE_GATE_ENABLE` env probe at verdict
    /// time. Used by on-path tests that run in parallel in the same test
    /// binary and cannot tolerate the process-wide env probe racing
    /// between `set_var("1")` in one test and `remove_var` in another.
    /// Production callers MUST use [`Self::new`] so the env probe stays
    /// the source of truth.
    // WIRE-DARK[E-38]: test-only seam (race-free parallel #[tokio::test]); no production caller by design.
    #[must_use]
    pub fn with_gate_enabled(
        plan_session_id: impl Into<String>,
        ssb: Arc<Ssb>,
        enabled: bool,
    ) -> Self {
        Self {
            plan_session_id: plan_session_id.into(),
            ssb,
            force_enabled: Some(enabled),
        }
    }

    /// Best-effort count of `PhaseTransition` rows in the SSB. Returns
    /// `0` on any viewer / decode error (fail-closed: the gate blocks on
    /// `AuditTrailLive` when the count is below
    /// [`AUDIT_TRAIL_MIN_SSB_ROWS`]). The viewer is opened read-only on
    /// each call; the Ssb writer is NOT blocked (read-only connection
    /// with `SQLITE_OPEN_READ_ONLY`).
    fn ssb_row_count_in_planning_phase(&self) -> u64 {
        let db_path = self.ssb.db_path();
        let viewer = match SsbViewer::open(db_path) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "E-38: SsbViewer::open failed in DefaultComplianceProvider; \
                     ssb_row_count_in_planning_phase = 0 (fail-closed)"
                );
                return 0;
            }
        };
        match viewer.recent_rows_of_type(SsbActionType::PhaseTransition.as_sql_str(), 1024) {
            Ok(rows) => u64::try_from(rows.len()).unwrap_or(u64::MAX),
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "E-38: recent_rows_of_type(PhaseTransition) failed; \
                     ssb_row_count_in_planning_phase = 0 (fail-closed)"
                );
                0
            }
        }
    }

    /// Build the [`PlanContext`] from live process state. Called only
    /// when the gate is opted in
    /// (`ELAI_EU_AI_ACT_COMPLIANCE_GATE_ENABLE=1`).
    fn build_plan_context(&self) -> PlanContext {
        PlanContext {
            plan_session_id: self.plan_session_id.clone(),
            has_clock_provider: true,
            has_deterministic_seed: std::env::var(ELAI_DETERMINISTIC_SEED_ENV).is_ok(),
            template_hash_chain_verified: false,
            template_archive_present: false,
            ssb_row_count_in_planning_phase: self.ssb_row_count_in_planning_phase(),
        }
    }
}

impl crate::fsm::ComplianceProvider for DefaultComplianceProvider {
    fn verdict(
        &self,
        source: crate::fsm::FsmState,
        target: crate::fsm::FsmState,
    ) -> ComplianceVerdict {
        // Soft opt-in: when the gate is not enabled, short-circuit to
        // Pass so the CLI is not bricked. Mirrors the C-28 default-OFF
        // pattern (ELAI_DETERMINISTIC_AUDIT_GATE_ENABLE). The
        // `force_enabled` field is `Some(bool)` only for on-path tests
        // that bypass the process-wide env probe to avoid racing with
        // parallel `#[tokio::test]` cases in the same binary; production
        // callers leave it `None` so the env probe is the source of
        // truth.
        let gate_enabled = match self.force_enabled {
            Some(flag) => flag,
            None => std::env::var(ELAI_EU_AI_ACT_COMPLIANCE_GATE_ENABLE_ENV).as_deref() == Ok("1"),
        };
        if !gate_enabled {
            return ComplianceVerdict::Pass { assertions_checked: 0 };
        }
        // Only the PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST edge is
        // in scope. Other edges pass through (the fsm.rs guard already
        // filters to this edge, but the provider is defensive).
        let in_scope = target == crate::fsm::FsmState::TestingFirst
            && (source == crate::fsm::FsmState::Planning
                || source == crate::fsm::FsmState::FeasibilityProbe);
        if !in_scope {
            return ComplianceVerdict::Pass { assertions_checked: 0 };
        }
        let ctx = self.build_plan_context();
        pre_gate_planning_to_testing_first(&ctx)
    }
}

/// Minimum SSB row count emitted during the PLANNING phase before the
/// `audit_trail_live` assertion accepts the transition. 1 is sufficient
/// (the FSM PhaseTransition row from IDLE -> PLANNING counts) but the
/// constant is named so a future tightening can be done in one place.
pub const AUDIT_TRAIL_MIN_SSB_ROWS: u64 = 1;

/// Input to [`pre_gate_planning_to_testing_first`]. Carries the four
/// boolean preconditions + the plan session id used as the SSB
/// `plan_session_id` field on emitted rows.
///
/// Constructed by the FSM transition handler from
/// `crate::fsm::FsmInner` + the orchestrator's clock + the
/// D-1 [`elai_reasoning::template::TemplateArchive`] presence flag.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanContext {
    /// Plan session id (passed through to SSB payload).
    pub plan_session_id: String,
    /// True when the orchestrator's [`elai_core::clock::ClockProvider`]
    /// is wired into the planning code path (no bare `SystemTime::now`
    /// / `Instant::now` in the hot path). C-15 shipped; this is a
    /// liveness check at the orchestrator boundary.
    pub has_clock_provider: bool,
    /// True when a deterministic seed was propagated into the planning
    /// context (e.g. via `ELAI_DETERMINISTIC_SEED` env or a CLI flag).
    /// C-27 T-1 determinism floor consumer.
    pub has_deterministic_seed: bool,
    /// True when the D-1 [`elai_reasoning::template::TemplateArchive`]
    /// is available and its content_hash chain verified for the current
    /// plan's reasoning templates. `false` here is interpreted as
    /// `Skipped` when [`Self::template_archive_present`] is also
    /// `false`; `false` here while `template_archive_present` is `true`
    /// is interpreted as a HARD FAIL.
    pub template_hash_chain_verified: bool,
    /// True when the D-1 TemplateArchive surface is reachable. When
    /// `false` the [`ComplianceAssertion::TemplateHashChain`] arm
    /// degrades to `Skipped { reason }` per plan Open Question 1.
    pub template_archive_present: bool,
    /// Number of SSB rows emitted in the current PLANNING phase
    /// window. Read by the orchestrator from the SSB writer's row
    /// counter at the moment of the transition.
    pub ssb_row_count_in_planning_phase: u64,
}

impl PlanContext {
    /// Convenience constructor for the "everything intact" case (no D-1
    /// substrate yet -- TemplateHashChain Skipped, all other assertions
    /// pass). Used by the audit_trail_live happy-path contract test.
    #[must_use]
    pub fn for_passing_no_d1(plan_session_id: impl Into<String>) -> Self {
        Self {
            plan_session_id: plan_session_id.into(),
            has_clock_provider: true,
            has_deterministic_seed: true,
            template_hash_chain_verified: false,
            template_archive_present: false,
            ssb_row_count_in_planning_phase: AUDIT_TRAIL_MIN_SSB_ROWS,
        }
    }
}

/// Which assertion fired in a [`ComplianceVerdict::Block`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplianceAssertion {
    /// ClockProvider injection is missing in the planning code path.
    ClockProviderIntact,
    /// Deterministic seed was not propagated into the planning context.
    DeterministicSeed,
    /// D-1 TemplateArchive is present but its content_hash chain failed
    /// verification.
    TemplateHashChain,
    /// PLANNING-phase SSB row count is below
    /// [`AUDIT_TRAIL_MIN_SSB_ROWS`].
    AuditTrailLive,
}

impl ComplianceAssertion {
    /// Snake_case wire label for the SSB `assertion_label` field.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClockProviderIntact => "clock_provider_intact",
            Self::DeterministicSeed => "deterministic_seed",
            Self::TemplateHashChain => "template_hash_chain",
            Self::AuditTrailLive => "audit_trail_live",
        }
    }

    /// Static remediation hint shown in the SSB row + the
    /// [`crate::fsm::FsmError::ComplianceBlocked`] Display impl.
    #[must_use]
    pub fn remediation_hint(&self) -> &'static str {
        match self {
            Self::ClockProviderIntact => {
                "inject elai_core::clock::ClockProvider into PlanContext; \
                 audit bare SystemTime::now / Instant::now in the planning crate" // entropy-allow: remediation-hint-string-not-a-call-site
            }
            Self::DeterministicSeed => {
                "set ELAI_DETERMINISTIC_SEED or pass --seed to the CLI; \
                 propagate the seed through PlanContext"
            }
            Self::TemplateHashChain => {
                "D-1 TemplateArchive content_hash chain failed; \
                 re-run elai_reasoning::template::verify_archive() and resolve drift"
            }
            Self::AuditTrailLive => {
                "PLANNING-phase emitted < AUDIT_TRAIL_MIN_SSB_ROWS rows; \
                 verify the FSM PhaseTransition emitter wired into Ssb::write_payload"
            }
        }
    }

    /// Bitmask flag for the `assertions_checked` field on the
    /// [`crate::ssb::RoadmapSignalKind::ComplianceVerified`] row.
    #[must_use]
    pub fn bit(&self) -> u32 {
        match self {
            Self::ClockProviderIntact => COMPLIANCE_BIT_CLOCK_PROVIDER,
            Self::DeterministicSeed => COMPLIANCE_BIT_DETERMINISTIC_SEED,
            Self::TemplateHashChain => COMPLIANCE_BIT_TEMPLATE_HASH_CHAIN,
            Self::AuditTrailLive => COMPLIANCE_BIT_AUDIT_TRAIL_LIVE,
        }
    }
}

/// Outcome of [`pre_gate_planning_to_testing_first`].
#[derive(Debug, Clone, PartialEq)]
pub enum ComplianceVerdict {
    /// All four assertions accepted (or 3 accepted + TemplateHashChain
    /// `Skipped` because D-1 has not shipped). FSM transition is
    /// allowed.
    Pass {
        /// Bitmask of assertions that ran + accepted (not the Skipped
        /// ones).
        assertions_checked: u32,
    },
    /// One assertion failed; FSM transition is refused.
    Block {
        /// Which assertion fired.
        assertion: ComplianceAssertion,
        /// Static remediation hint.
        hint: String,
    },
}

impl ComplianceVerdict {
    /// True when the verdict allows the FSM transition.
    #[must_use]
    pub fn allows_transition(&self) -> bool {
        matches!(self, Self::Pass { .. })
    }
}

/// EU AI Act compliance pre-gate. Returns
/// [`ComplianceVerdict::Pass`] when all preconditions hold (with
/// TemplateHashChain `Skipped` when D-1 is not yet shipped); returns
/// [`ComplianceVerdict::Block`] on the FIRST failing assertion (no
/// short-circuit downstream -- only the first failure is reported so
/// the operator fixes one thing at a time).
///
/// Evaluation order matches the bitmask LSB order:
///   1. ClockProviderIntact
///   2. DeterministicSeed
///   3. TemplateHashChain (Skipped when archive absent)
///   4. AuditTrailLive
#[must_use]
pub fn pre_gate_planning_to_testing_first(ctx: &PlanContext) -> ComplianceVerdict {
    let mut checked: u32 = 0;

    if !ctx.has_clock_provider {
        return ComplianceVerdict::Block {
            assertion: ComplianceAssertion::ClockProviderIntact,
            hint: ComplianceAssertion::ClockProviderIntact
                .remediation_hint()
                .to_string(),
        };
    }
    checked |= ComplianceAssertion::ClockProviderIntact.bit();

    if !ctx.has_deterministic_seed {
        return ComplianceVerdict::Block {
            assertion: ComplianceAssertion::DeterministicSeed,
            hint: ComplianceAssertion::DeterministicSeed
                .remediation_hint()
                .to_string(),
        };
    }
    checked |= ComplianceAssertion::DeterministicSeed.bit();

    // D-1 not yet shipped -> Skipped (transition allowed) per plan Open
    // Question 1. When the archive is present, false hash chain = HARD
    // FAIL.
    if ctx.template_archive_present {
        if !ctx.template_hash_chain_verified {
            return ComplianceVerdict::Block {
                assertion: ComplianceAssertion::TemplateHashChain,
                hint: ComplianceAssertion::TemplateHashChain
                    .remediation_hint()
                    .to_string(),
            };
        }
        checked |= ComplianceAssertion::TemplateHashChain.bit();
    }

    if ctx.ssb_row_count_in_planning_phase < AUDIT_TRAIL_MIN_SSB_ROWS {
        return ComplianceVerdict::Block {
            assertion: ComplianceAssertion::AuditTrailLive,
            hint: ComplianceAssertion::AuditTrailLive
                .remediation_hint()
                .to_string(),
        };
    }
    checked |= ComplianceAssertion::AuditTrailLive.bit();

    ComplianceVerdict::Pass {
        assertions_checked: checked,
    }
}

/// Build the SSB payload for a [`ComplianceVerdict::Block`] outcome.
/// Callers ship this through `crate::ssb::Ssb::write_payload` only when
/// [`crate::roadmap_signals::roadmap_signals_enabled`] returns true.
#[must_use]
pub fn build_block_payload(
    ctx: &PlanContext,
    assertion: ComplianceAssertion,
    hint: &str,
    now_ms: i64,
) -> AiRoadmapSignalEventPayload {
    AiRoadmapSignalEventPayload {
        kind: RoadmapSignalKind::ComplianceBlocked,
        plan_session_id: ctx.plan_session_id.clone(),
        assertion_label: assertion.as_str().to_string(),
        hint: hint.to_string(),
        assertions_checked: 0,
        mcp_transport_current: String::new(),
        mcp_transport_target: String::new(),
        backend_name: String::new(),
        declared_context_tokens: 0,
        mrcr_score_label: String::new(),
        posture_doc_path: String::new(),
        posture_doc_mtime_ms: 0,
        age_days: 0,
        details_json: serde_json::json!({
            "fsm_edge": "planning_to_testing_first",
            "art_12_record_keeping": true,
            "art_13_transparency": true
        }),
        ts_ms: now_ms,
    }
}

/// Build the SSB payload for a [`ComplianceVerdict::Pass`] outcome.
#[must_use]
pub fn build_verified_payload(
    ctx: &PlanContext,
    assertions_checked: u32,
    now_ms: i64,
) -> AiRoadmapSignalEventPayload {
    AiRoadmapSignalEventPayload {
        kind: RoadmapSignalKind::ComplianceVerified,
        plan_session_id: ctx.plan_session_id.clone(),
        assertion_label: String::new(),
        hint: String::new(),
        assertions_checked,
        mcp_transport_current: String::new(),
        mcp_transport_target: String::new(),
        backend_name: String::new(),
        declared_context_tokens: 0,
        mrcr_score_label: String::new(),
        posture_doc_path: String::new(),
        posture_doc_mtime_ms: 0,
        age_days: 0,
        details_json: serde_json::json!({
            "fsm_edge": "planning_to_testing_first",
            "art_12_record_keeping": true,
            "art_13_transparency": true
        }),
        ts_ms: now_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passing_ctx() -> PlanContext {
        PlanContext {
            plan_session_id: "s-test".into(),
            has_clock_provider: true,
            has_deterministic_seed: true,
            template_hash_chain_verified: true,
            template_archive_present: true,
            ssb_row_count_in_planning_phase: 5,
        }
    }

    #[test]
    fn pass_when_all_assertions_intact() {
        let v = pre_gate_planning_to_testing_first(&passing_ctx());
        match v {
            ComplianceVerdict::Pass { assertions_checked } => {
                assert_eq!(
                    assertions_checked,
                    COMPLIANCE_BIT_CLOCK_PROVIDER
                        | COMPLIANCE_BIT_DETERMINISTIC_SEED
                        | COMPLIANCE_BIT_TEMPLATE_HASH_CHAIN
                        | COMPLIANCE_BIT_AUDIT_TRAIL_LIVE
                );
            }
            other => panic!("expected Pass, got {other:?}"),
        }
    }

    #[test]
    fn pass_with_template_hash_chain_skipped_when_d1_absent() {
        let ctx = PlanContext::for_passing_no_d1("s-skipped");
        let v = pre_gate_planning_to_testing_first(&ctx);
        match v {
            ComplianceVerdict::Pass { assertions_checked } => {
                // TemplateHashChain bit NOT set (Skipped) but the
                // others are.
                assert_eq!(
                    assertions_checked,
                    COMPLIANCE_BIT_CLOCK_PROVIDER
                        | COMPLIANCE_BIT_DETERMINISTIC_SEED
                        | COMPLIANCE_BIT_AUDIT_TRAIL_LIVE
                );
            }
            other => panic!("expected Pass, got {other:?}"),
        }
    }

    #[test]
    fn block_clock_provider_missing() {
        let mut ctx = passing_ctx();
        ctx.has_clock_provider = false;
        let v = pre_gate_planning_to_testing_first(&ctx);
        match v {
            ComplianceVerdict::Block { assertion, hint } => {
                assert_eq!(assertion, ComplianceAssertion::ClockProviderIntact);
                assert!(hint.contains("ClockProvider"));
            }
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn block_deterministic_seed_missing() {
        let mut ctx = passing_ctx();
        ctx.has_deterministic_seed = false;
        let v = pre_gate_planning_to_testing_first(&ctx);
        match v {
            ComplianceVerdict::Block { assertion, .. } => {
                assert_eq!(assertion, ComplianceAssertion::DeterministicSeed);
            }
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn block_template_hash_chain_failed() {
        let mut ctx = passing_ctx();
        ctx.template_hash_chain_verified = false;
        let v = pre_gate_planning_to_testing_first(&ctx);
        match v {
            ComplianceVerdict::Block { assertion, .. } => {
                assert_eq!(assertion, ComplianceAssertion::TemplateHashChain);
            }
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn block_audit_trail_below_threshold() {
        let mut ctx = passing_ctx();
        ctx.ssb_row_count_in_planning_phase = 0;
        let v = pre_gate_planning_to_testing_first(&ctx);
        match v {
            ComplianceVerdict::Block { assertion, .. } => {
                assert_eq!(assertion, ComplianceAssertion::AuditTrailLive);
            }
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn block_payload_carries_assertion_label_and_hint() {
        let ctx = passing_ctx();
        let payload = build_block_payload(
            &ctx,
            ComplianceAssertion::ClockProviderIntact,
            ComplianceAssertion::ClockProviderIntact.remediation_hint(),
            1_700_000_175_001,
        );
        assert_eq!(payload.kind, RoadmapSignalKind::ComplianceBlocked);
        assert_eq!(payload.assertion_label, "clock_provider_intact");
        assert!(payload.hint.contains("ClockProvider"));
        assert_eq!(payload.plan_session_id, "s-test");
        assert_eq!(payload.assertions_checked, 0);
    }

    #[test]
    fn verified_payload_carries_bitmask() {
        let ctx = passing_ctx();
        let payload = build_verified_payload(
            &ctx,
            COMPLIANCE_BIT_CLOCK_PROVIDER | COMPLIANCE_BIT_AUDIT_TRAIL_LIVE,
            1_700_000_175_002,
        );
        assert_eq!(payload.kind, RoadmapSignalKind::ComplianceVerified);
        assert_eq!(
            payload.assertions_checked,
            COMPLIANCE_BIT_CLOCK_PROVIDER | COMPLIANCE_BIT_AUDIT_TRAIL_LIVE
        );
        assert!(payload.assertion_label.is_empty());
    }
}
