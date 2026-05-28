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

use crate::ssb::{
    AiRoadmapSignalEventPayload, COMPLIANCE_BIT_AUDIT_TRAIL_LIVE,
    COMPLIANCE_BIT_CLOCK_PROVIDER, COMPLIANCE_BIT_DETERMINISTIC_SEED,
    COMPLIANCE_BIT_TEMPLATE_HASH_CHAIN, RoadmapSignalKind,
};

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
                 audit bare SystemTime::now / Instant::now in the planning crate"
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
