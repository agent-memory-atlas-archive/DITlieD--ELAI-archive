//! D-10 — closed [`GroundingMode`] taxonomy + pure routing predicate
//! [`select_grounding_mode`].
//!
//! The orchestrator (NOT the policy model) picks the mode under which
//! every novel claim is graded. This realises the Cohen Boxed Myopic
//! / Everitt current-RF design constraint: an agent must never be in
//! a position to influence the function that grades it.
//!
//! Pure logic — no I/O, no async, no LLM calls, no system clock. The
//! function is deterministic in its `(claim, ctx)` inputs and the
//! D-9 locked-keys auditor freezes the symbol so the policy model
//! cannot self-modify the table.
//!
//! Mode taxonomy (per docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §1):
//!
//! | Mode                          | Used when                                       |
//! |-------------------------------|-------------------------------------------------|
//! | `Constructive`                | Factual / Computational claim WITH evidence and |
//! |                               | a deterministic verifier is online.             |
//! | `FalsifiabilityCrossFamily`   | Speculative claim, cross-family verifier        |
//! |                               | present, canary not blocking.                   |
//! | `ConformalRefusal`            | UniCR calibration ready AND no constructive    |
//! |                               | path exists AND intrinsic signals too weak.    |
//! | `OperationalDeferred`         | Procedural / Hypothetical — needs sandbox      |
//! |                               | experiment (D-18) which isn't D-10's scope.    |
//! | `ConservativeFallback`        | Anything else — refuse to grade, log the row.   |
//!
//! Decision table (deterministic, exhaustive — every `(ClaimType,
//! ctx-state)` lands on exactly one mode):
//!
//! 1. `user_role == Adversary`                       → `FalsifiabilityCrossFamily`
//! 2. `user_role == Auditor`                         → `ConservativeFallback` (auditor cannot self-grade)
//! 3. `user_role == Meta`                            → `ConservativeFallback`
//! 4. `claim_type == MetaQuery`                      → `ConservativeFallback`
//! 5. `claim_type == Normative`                      → `ConservativeFallback`
//! 6. `ctx.canary_blocks()`                          → `ConformalRefusal` (or `ConservativeFallback` if !ready)
//! 7. `claim_type == Procedural | Hypothetical`      → `OperationalDeferred`
//! 8. `claim_type == Factual | Computational`        :
//!     - has grounding evidence AND a Deterministic verifier → `Constructive`
//!     - else has cross-family verifier               → `FalsifiabilityCrossFamily`
//!     - else                                        → `ConformalRefusal` or `ConservativeFallback`
//! 9. `claim_type == Speculative`                    :
//!     - cross-family verifier present, canary clean → `FalsifiabilityCrossFamily`
//!     - else                                        → `ConformalRefusal` or `ConservativeFallback`

use serde::{Deserialize, Serialize};

use crate::claim::{Claim, ClaimType};
use crate::orch_context::{OrchContext, OrchUserRole};
use crate::verification_result::VerifierKind;

/// Closed set of grounding modes. Adding a variant is a plan-level
/// schema change — the L3 locked-keys registry (D-9) freezes the
/// enum so the agent cannot extend it at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroundingMode {
    /// Deterministic checker is ground truth (Mode A — real-problem §1).
    Constructive,
    /// Adversarial probe via a different-family verifier (Mode B — last
    /// resort, hard constraints).
    FalsifiabilityCrossFamily,
    /// First-class "I don't know" answer backed by UniCR conformal
    /// abstain bound (D-16). Mode E in the spec.
    ConformalRefusal,
    /// Sandbox / typed ObservableEffect experiment (Mode D). D-18
    /// supplies the runner; D-10 reserves the slot.
    OperationalDeferred,
    /// Catch-all when no formal mode applies — orchestrator refuses
    /// to grade and emits an explanatory SSB row.
    ConservativeFallback,
}

impl GroundingMode {
    /// Stable snake-case wire label matching the serde tag (used for
    /// SSB payloads + dashboard rendering).
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Constructive => "constructive",
            Self::FalsifiabilityCrossFamily => "falsifiability_cross_family",
            Self::ConformalRefusal => "conformal_refusal",
            Self::OperationalDeferred => "operational_deferred",
            Self::ConservativeFallback => "conservative_fallback",
        }
    }

    /// All five variants — used by tests + the dashboard to render the
    /// per-mode histogram without hard-coding the list.
    #[must_use]
    pub const fn all() -> [GroundingMode; 5] {
        [
            Self::Constructive,
            Self::FalsifiabilityCrossFamily,
            Self::ConformalRefusal,
            Self::OperationalDeferred,
            Self::ConservativeFallback,
        ]
    }
}

/// Deterministic predicate — picks a [`GroundingMode`] for `claim`
/// under the orchestrator state captured in `ctx`. Pure function.
///
/// MUST NOT:
///   * call the LLM (defends Cohen Boxed Myopic).
///   * read the system clock (timing must come from the caller).
///   * mutate `ctx` (snapshot is immutable here).
#[must_use]
pub fn select_grounding_mode(claim: &Claim, ctx: &OrchContext) -> GroundingMode {
    // Rule 1 — role-based pre-empts dominate. An Adversary claim
    // always crosses families; an Auditor or Meta claim never grades
    // itself (returns ConservativeFallback so the orchestrator emits
    // a "no grade applied" row, not a silent pass).
    match ctx.user_role {
        OrchUserRole::Adversary => {
            // Even the adversary needs a cross-family verifier to land
            // on Mode B; without one we fall through to the refusal
            // path (calibration gating then applies).
            if ctx.has_cross_family_verifier(VerifierKind::Intrinsic) {
                return GroundingMode::FalsifiabilityCrossFamily;
            }
            return refusal_or_fallback(ctx);
        }
        OrchUserRole::Auditor | OrchUserRole::Meta => {
            return GroundingMode::ConservativeFallback;
        }
        OrchUserRole::Executor => {}
    }

    // Rule 2 — MetaQuery / Normative claims are never graded as
    // factual content. ConservativeFallback emits the row but does
    // not pretend to verify.
    match claim.claim_type {
        ClaimType::MetaQuery | ClaimType::Normative => {
            return GroundingMode::ConservativeFallback;
        }
        _ => {}
    }

    // Rule 3 — canary blocking → refuse. Cross-domain transfer canary
    // (D-7) flagged that the same family is regressing; trusting a
    // grade here is exactly the Cohen failure mode.
    if ctx.canary_blocks() {
        return refusal_or_fallback(ctx);
    }

    // Rule 4 — Procedural / Hypothetical claims defer to operational
    // sandbox (D-18). Until D-18 lands the row records the deferral.
    match claim.claim_type {
        ClaimType::Procedural | ClaimType::Hypothetical => {
            return GroundingMode::OperationalDeferred;
        }
        _ => {}
    }

    // Rule 5 — Factual / Computational: constructive when possible,
    // cross-family when not, refusal otherwise.
    if matches!(claim.claim_type, ClaimType::Factual | ClaimType::Computational) {
        let has_det = ctx
            .available_verifiers
            .iter()
            .any(|v| *v == VerifierKind::Deterministic);
        if has_det && claim.has_grounding_evidence() {
            return GroundingMode::Constructive;
        }
        if ctx.has_cross_family_verifier(VerifierKind::Intrinsic) {
            return GroundingMode::FalsifiabilityCrossFamily;
        }
        return refusal_or_fallback(ctx);
    }

    // Rule 6 — Speculative: cross-family is the only honest path; if
    // unavailable, abstain (calibration-gated).
    if matches!(claim.claim_type, ClaimType::Speculative) {
        if ctx.has_cross_family_verifier(VerifierKind::Intrinsic) {
            return GroundingMode::FalsifiabilityCrossFamily;
        }
        return refusal_or_fallback(ctx);
    }

    // Unreachable: ClaimType is a closed enum and every variant is
    // matched above. Defensive default per zero-lazy-code rule —
    // unknown future variant lands on the safest mode.
    GroundingMode::ConservativeFallback
}

/// Helper — picks `ConformalRefusal` when D-16 calibration is ready,
/// `ConservativeFallback` otherwise. Both modes describe abstention;
/// they differ in whether the abstain bound is formal (UniCR) or
/// orchestrator-policy.
#[inline]
fn refusal_or_fallback(ctx: &OrchContext) -> GroundingMode {
    if ctx.conformal_calibration_ready {
        GroundingMode::ConformalRefusal
    } else {
        GroundingMode::ConservativeFallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::{EvidenceKind, EvidenceRef};
    use crate::orch_context::OrchPhase;

    fn mk_claim(claim_type: ClaimType, with_evidence: bool) -> Claim {
        let evidence = if with_evidence {
            vec![EvidenceRef::from_content(
                EvidenceKind::Corpus,
                "src/main.rs:1-10",
                b"fn main() {}",
            )]
        } else {
            vec![]
        };
        Claim {
            claim_id: format!("c-{}", claim_type.as_str()),
            text: format!("test claim for {}", claim_type.as_str()),
            claim_type,
            evidence,
            ts_ms: 1_700_000_000_000,
        }
    }

    fn ctx_with_verifiers(verifiers: &[VerifierKind]) -> OrchContext {
        let mut ctx = OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing);
        ctx.available_verifiers = verifiers.to_vec();
        ctx
    }

    #[test]
    fn factual_with_deterministic_evidence_routes_constructive() {
        let claim = mk_claim(ClaimType::Factual, true);
        let ctx = ctx_with_verifiers(&[VerifierKind::Deterministic, VerifierKind::Intrinsic]);
        assert_eq!(select_grounding_mode(&claim, &ctx), GroundingMode::Constructive);
    }

    #[test]
    fn factual_without_evidence_falls_back_to_cross_family() {
        let claim = mk_claim(ClaimType::Factual, false);
        let ctx = ctx_with_verifiers(&[VerifierKind::Deterministic, VerifierKind::Intrinsic]);
        assert_eq!(
            select_grounding_mode(&claim, &ctx),
            GroundingMode::FalsifiabilityCrossFamily
        );
    }

    #[test]
    fn procedural_routes_operational_deferred() {
        let claim = mk_claim(ClaimType::Procedural, true);
        let ctx = ctx_with_verifiers(&[VerifierKind::Deterministic]);
        assert_eq!(
            select_grounding_mode(&claim, &ctx),
            GroundingMode::OperationalDeferred
        );
    }

    #[test]
    fn hypothetical_routes_operational_deferred() {
        let claim = mk_claim(ClaimType::Hypothetical, false);
        let ctx = ctx_with_verifiers(&[VerifierKind::Intrinsic]);
        assert_eq!(
            select_grounding_mode(&claim, &ctx),
            GroundingMode::OperationalDeferred
        );
    }

    #[test]
    fn normative_routes_conservative_fallback() {
        let claim = mk_claim(ClaimType::Normative, true);
        let ctx = ctx_with_verifiers(&[VerifierKind::Deterministic, VerifierKind::Intrinsic]);
        assert_eq!(
            select_grounding_mode(&claim, &ctx),
            GroundingMode::ConservativeFallback
        );
    }

    #[test]
    fn meta_query_routes_conservative_fallback() {
        let claim = mk_claim(ClaimType::MetaQuery, true);
        let ctx = ctx_with_verifiers(&[VerifierKind::Deterministic]);
        assert_eq!(
            select_grounding_mode(&claim, &ctx),
            GroundingMode::ConservativeFallback
        );
    }

    #[test]
    fn speculative_with_cross_family_routes_falsifiability() {
        let claim = mk_claim(ClaimType::Speculative, true);
        let ctx = ctx_with_verifiers(&[VerifierKind::Intrinsic, VerifierKind::Ppm]);
        assert_eq!(
            select_grounding_mode(&claim, &ctx),
            GroundingMode::FalsifiabilityCrossFamily
        );
    }

    #[test]
    fn speculative_with_calibration_routes_conformal_refusal() {
        let claim = mk_claim(ClaimType::Speculative, true);
        let mut ctx = ctx_with_verifiers(&[VerifierKind::Intrinsic]);
        ctx.conformal_calibration_ready = true;
        assert_eq!(
            select_grounding_mode(&claim, &ctx),
            GroundingMode::ConformalRefusal
        );
    }

    #[test]
    fn speculative_without_calibration_routes_conservative() {
        let claim = mk_claim(ClaimType::Speculative, true);
        let ctx = ctx_with_verifiers(&[VerifierKind::Intrinsic]);
        assert_eq!(
            select_grounding_mode(&claim, &ctx),
            GroundingMode::ConservativeFallback
        );
    }

    #[test]
    fn adversary_role_pre_empts_to_cross_family() {
        let claim = mk_claim(ClaimType::Factual, true);
        let mut ctx = ctx_with_verifiers(&[VerifierKind::Intrinsic, VerifierKind::Deterministic]);
        ctx.user_role = OrchUserRole::Adversary;
        assert_eq!(
            select_grounding_mode(&claim, &ctx),
            GroundingMode::FalsifiabilityCrossFamily
        );
    }

    #[test]
    fn auditor_role_routes_conservative_fallback() {
        let claim = mk_claim(ClaimType::Factual, true);
        let mut ctx = ctx_with_verifiers(&[VerifierKind::Deterministic, VerifierKind::Intrinsic]);
        ctx.user_role = OrchUserRole::Auditor;
        assert_eq!(
            select_grounding_mode(&claim, &ctx),
            GroundingMode::ConservativeFallback
        );
    }

    #[test]
    fn computational_with_evidence_routes_constructive() {
        let claim = mk_claim(ClaimType::Computational, true);
        let ctx = ctx_with_verifiers(&[VerifierKind::Deterministic, VerifierKind::Intrinsic]);
        assert_eq!(
            select_grounding_mode(&claim, &ctx),
            GroundingMode::Constructive
        );
    }

    #[test]
    fn router_is_deterministic_in_inputs() {
        let claim = mk_claim(ClaimType::Factual, true);
        let ctx = ctx_with_verifiers(&[VerifierKind::Deterministic, VerifierKind::Intrinsic]);
        let m1 = select_grounding_mode(&claim, &ctx);
        let m2 = select_grounding_mode(&claim, &ctx);
        assert_eq!(m1, m2);
    }

    #[test]
    fn deterministic_routing_table_smoke() {
        // 5 representative claims, one per expected mode. Mirrors the
        // observable_effect smoke from the D-10 plan.
        struct Row {
            claim_type: ClaimType,
            with_evidence: bool,
            verifiers: &'static [VerifierKind],
            calibration_ready: bool,
            user_role: OrchUserRole,
            expected: GroundingMode,
        }
        let rows = [
            Row {
                claim_type: ClaimType::Factual,
                with_evidence: true,
                verifiers: &[VerifierKind::Deterministic, VerifierKind::Intrinsic],
                calibration_ready: false,
                user_role: OrchUserRole::Executor,
                expected: GroundingMode::Constructive,
            },
            Row {
                claim_type: ClaimType::Procedural,
                with_evidence: false,
                verifiers: &[VerifierKind::Deterministic],
                calibration_ready: false,
                user_role: OrchUserRole::Executor,
                expected: GroundingMode::OperationalDeferred,
            },
            Row {
                claim_type: ClaimType::Speculative,
                with_evidence: false,
                verifiers: &[VerifierKind::Intrinsic, VerifierKind::Ppm],
                calibration_ready: false,
                user_role: OrchUserRole::Executor,
                expected: GroundingMode::FalsifiabilityCrossFamily,
            },
            Row {
                claim_type: ClaimType::Speculative,
                with_evidence: false,
                verifiers: &[VerifierKind::Intrinsic],
                calibration_ready: true,
                user_role: OrchUserRole::Executor,
                expected: GroundingMode::ConformalRefusal,
            },
            Row {
                claim_type: ClaimType::Normative,
                with_evidence: true,
                verifiers: &[VerifierKind::Deterministic, VerifierKind::Intrinsic],
                calibration_ready: false,
                user_role: OrchUserRole::Executor,
                expected: GroundingMode::ConservativeFallback,
            },
        ];

        for row in rows {
            let claim = mk_claim(row.claim_type, row.with_evidence);
            let mut ctx = ctx_with_verifiers(row.verifiers);
            ctx.conformal_calibration_ready = row.calibration_ready;
            ctx.user_role = row.user_role;
            assert_eq!(
                select_grounding_mode(&claim, &ctx),
                row.expected,
                "row {:?} expected {:?}",
                row.claim_type,
                row.expected
            );
        }
    }
}
