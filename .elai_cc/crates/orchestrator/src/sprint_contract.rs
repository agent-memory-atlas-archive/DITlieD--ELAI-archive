//! I-21 — sprint contracts + ralph loop.
//!
//! The generator proposes a bounded, **structured** Sprint Contract; an
//! **isolated evaluator** (zero shared context with the generator) approves or
//! rejects it; on a failed acceptance-criteria check the failing trace is
//! reinjected to the generator (the **ralph loop**, bounded to
//! [`MAX_RALPH_REINJECTIONS`] iterations); the final failure escalates to the
//! D-13 cross-family adversary which produces a concrete counterexample.
//!
//! ## Composition (GD-1 — compose the SHIPPED surfaces, never re-implement)
//!
//!  * **i-17 [`crate::verifier_stack::TriLayerStack`]** — the acceptance-criteria
//!    checker. The ralph-loop trigger is a [`crate::verifier_stack::VerdictRow`]
//!    verdict (`HardVeto` => failure), NOT an ad-hoc check.
//!  * **D-13 [`crate::adversary::DynAdversary`]** — the `AdversaryEscalated`
//!    target. The 3rd failure routes through `generate_counter_dyn`.
//!  * **i-1 [`crate::directive::InProcessSupervisor`]** — the isolation
//!    substrate. The evaluator only ever sees the *sealed contract* (this
//!    module's [`SealedContract`]), never any generator turn or scratchpad.
//!  * **H-38 [`crate::stop_cost_gate`]** — loop-cap posture reference. The cap
//!    here is an in-fn bounded counter (controller decision 3), not routed
//!    through the stop gate.
//!
//! ## Isolation invariant (the structural point of i-21)
//!
//! The evaluator receives a [`SealedContract`] — it carries ONLY the Sprint
//! Contract fields and the task spec, with NO `Vec` of generator turns, NO
//! scratchpad, NO intermediate outputs. There is no field on the sealed type
//! through which generator context can leak; "zero shared context" is a *type*
//! property, verified at the row level by the contract test (an evaluator
//! payload that referenced generator rows could not be constructed here).
//!
//! ## Wire-dark posture
//!
//! Default-OFF: requires `ELAI_SPRINT_CONTRACT_ENABLE=1` (R9 experiment;
//! promotion plan `I-21-promotion`; see `wire-dark-allowlist.toml`). The
//! production dispatch (`task_runner::dispatch_sprint_contract`) is a strict
//! no-op until the flag flips AND the checker runtime is plumbed.
//!
//! ## Grounding
//!
//!  * Plan: `docs/thefinalglance/I-plan/I-21_sprint_contracts_ralph_loop.md`
//!  * Self-review-bias premise (+0.12 on structured tasks): `[DESIGN]` — from
//!    the Consolidated draft; NOT WebSearch-confirmed as a headline figure and
//!    therefore wired NOWHERE as a literal gate threshold (ID-5). The loop
//!    bound (3) is a design choice, not a paper constant.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tracing::{debug, warn};

use crate::adversary::{AdversaryConfig, DynAdversary};
use crate::claim::{Claim, ClaimType};
use crate::ssb::{
    Ssb, SprintContractEventKind, SprintContractEventPayload, SsbPayload,
    ELAI_SPRINT_CONTRACT_ENABLE_ENV,
};
use crate::verifier_stack::{Branch, TriLayerStack};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Hard upper bound on total ralph-loop iterations (controller decision 3:
/// an in-fn bounded counter, not routed through the H-38 stop gate). The 1st
/// failure reinjects, the 2nd failure reinjects, the 3rd failure escalates to
/// the adversary — so at most 2 `RalphReinjected` rows precede 1
/// `AdversaryEscalated` row. Named const per CLAUDE.md "no magic numbers".
pub const MAX_RALPH_REINJECTIONS: u32 = 3;

/// Maximum length (chars) of a free-text field surfaced on an SSB row. Bounds
/// the objection / counterexample strings before payload construction so a
/// runaway evaluator/adversary message cannot blow the SSB payload size cap.
pub const SPRINT_FIELD_MAX_CHARS: usize = 2_000;

/// SSB category-style label surfaced in traces for this subsystem.
pub const SPRINT_CONTRACT_CATEGORY: &str = "sprint_contract";

// ---------------------------------------------------------------------------
// Env gate (mirrors verifier_stack::tri_layer_enabled)
// ---------------------------------------------------------------------------

/// Returns `true` when [`ELAI_SPRINT_CONTRACT_ENABLE_ENV`] is set to a
/// recognised truthy literal (`"1"` / `"true"` / `"yes"` / `"on"`,
/// case-insensitive). Default-OFF (wire-dark; promotion plan `I-21-promotion`).
/// Mirrors the [`crate::verifier_stack::tri_layer_enabled`] gate idiom.
#[must_use]
pub fn sprint_contract_enabled() -> bool {
    match std::env::var(ELAI_SPRINT_CONTRACT_ENABLE_ENV) {
        Ok(raw) => matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Acceptance criterion
// ---------------------------------------------------------------------------

/// A single, **deterministically checkable** acceptance criterion.
///
/// The variants are the ONLY forms the evaluator will approve. A free-text /
/// open-ended criterion ("the implementation feels complete") is REJECTED at
/// validation time — there is no `FreeText` variant by design (stub_ban 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptanceCriterion {
    /// References a test symbol whose pass is the criterion (e.g.
    /// `test_compute_risk_score`).
    TestSymbol(String),
    /// The change must compile (`cargo check` / equivalent passes).
    CompileStatus,
    /// References a `WIRING_CONTRACT` field that must be satisfied (e.g.
    /// `produces:SprintContract`).
    WiringContractField(String),
}

impl AcceptanceCriterion {
    /// True iff this criterion carries a non-empty, deterministic referent.
    /// An empty symbol/field name is NOT deterministically checkable.
    #[must_use]
    pub fn is_deterministic(&self) -> bool {
        match self {
            Self::TestSymbol(s) | Self::WiringContractField(s) => !s.trim().is_empty(),
            Self::CompileStatus => true,
        }
    }

    /// Human-readable label for objection messages.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::TestSymbol(s) => format!("test_symbol({s})"),
            Self::CompileStatus => "compile_status".to_string(),
            Self::WiringContractField(s) => format!("wiring_contract_field({s})"),
        }
    }
}

// ---------------------------------------------------------------------------
// SprintContract
// ---------------------------------------------------------------------------

/// A bounded, structured proposal from the generator. Machine-readable fields
/// only — NOT a free-text plan.
#[derive(Debug, Clone, PartialEq)]
pub struct SprintContract {
    /// What the sprint is trying to achieve (bounded plaintext).
    pub goal: String,
    /// Files the sprint is permitted to touch (the in-scope set).
    pub in_scope_files: Vec<String>,
    /// Deterministically checkable acceptance criteria. MUST be non-empty.
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    /// The generator's requested bounded iteration count (clamped to
    /// [`MAX_RALPH_REINJECTIONS`] at runtime).
    pub bounded_iteration_count: u32,
    /// A marker for scope-creep detection: anything outside `in_scope_files`
    /// touched during the sprint is an out-of-scope violation.
    pub out_of_scope_marker: String,
}

/// Why a [`SprintContract`] failed validation. Carried in the `Rejected`
/// objection field so the rejection is concrete, never a generic
/// "needs improvement".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractValidationError {
    /// `goal` is empty / whitespace-only.
    EmptyGoal,
    /// `acceptance_criteria` is empty — nothing to check against.
    NoAcceptanceCriteria,
    /// At least one criterion is not deterministically checkable.
    NonDeterministicCriterion {
        /// The offending criterion's label.
        criterion: String,
    },
    /// `in_scope_files` is empty — the sprint has no bounded surface.
    EmptyScope,
}

impl ContractValidationError {
    /// Concrete structured objection string surfaced on the `Rejected` row.
    #[must_use]
    pub fn objection(&self) -> String {
        match self {
            Self::EmptyGoal => "contract goal is empty; a sprint must state a concrete goal".to_string(),
            Self::NoAcceptanceCriteria => {
                "no acceptance criteria; a sprint must declare >=1 deterministically checkable criterion".to_string()
            }
            Self::NonDeterministicCriterion { criterion } => format!(
                "acceptance criterion {criterion} is not deterministically checkable; \
                 only test-symbol / compile-status / wiring-contract-field criteria are accepted"
            ),
            Self::EmptyScope => {
                "in_scope_files is empty; a sprint must bound its file surface".to_string()
            }
        }
    }
}

impl SprintContract {
    /// Validate the contract structurally. Returns `Ok(())` when every
    /// acceptance criterion is deterministically checkable and the contract is
    /// well-formed; otherwise the first violation (the evaluator rejects on the
    /// first concrete objection).
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if self.goal.trim().is_empty() {
            return Err(ContractValidationError::EmptyGoal);
        }
        if self.in_scope_files.is_empty() {
            return Err(ContractValidationError::EmptyScope);
        }
        if self.acceptance_criteria.is_empty() {
            return Err(ContractValidationError::NoAcceptanceCriteria);
        }
        for c in &self.acceptance_criteria {
            if !c.is_deterministic() {
                return Err(ContractValidationError::NonDeterministicCriterion {
                    criterion: c.label(),
                });
            }
        }
        Ok(())
    }

    /// The effective ralph-loop bound: the generator's request clamped to the
    /// hard [`MAX_RALPH_REINJECTIONS`] cap (stub_ban 2: never unbounded).
    #[must_use]
    pub fn effective_iteration_bound(&self) -> u32 {
        self.bounded_iteration_count.min(MAX_RALPH_REINJECTIONS)
    }

    /// Stable, content-derived contract id (blake3 of the structured fields, so
    /// rows recompute under `ELAI_SEED`, GD-2). blake3 per Anti-Stale-Training.
    #[must_use]
    pub fn contract_id(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.goal.as_bytes());
        for f in &self.in_scope_files {
            hasher.update(b"\x1ffile\x1f");
            hasher.update(f.as_bytes());
        }
        for c in &self.acceptance_criteria {
            hasher.update(b"\x1fcrit\x1f");
            hasher.update(c.label().as_bytes());
        }
        hasher.update(b"\x1fmarker\x1f");
        hasher.update(self.out_of_scope_marker.as_bytes());
        hasher.finalize().to_hex().to_string()
    }
}

// ---------------------------------------------------------------------------
// SealedContract — the ONLY thing the isolated evaluator ever sees
// ---------------------------------------------------------------------------

/// The payload handed to the isolated evaluator. It carries ONLY the Sprint
/// Contract and the task spec — there is NO field through which a generator
/// turn, scratchpad, or intermediate output can leak. This type IS the
/// isolation invariant (i-1 substrate: the evaluator Directive carries only
/// this sealed payload, generator context pruned before the spawn).
#[derive(Debug, Clone, PartialEq)]
pub struct SealedContract {
    /// The structured contract under evaluation.
    pub contract: SprintContract,
    /// The bounded task specification (NOT generator turns — the task prompt).
    pub task_spec: String,
}

impl SealedContract {
    /// Seal a contract + task spec for the evaluator. There is deliberately no
    /// constructor arm that accepts generator history.
    #[must_use]
    pub fn seal(contract: SprintContract, task_spec: impl Into<String>) -> Self {
        Self {
            contract,
            task_spec: task_spec.into(),
        }
    }

    /// The evaluator's verdict on the sealed contract: validate the structure,
    /// approving iff every criterion is deterministically checkable.
    pub fn evaluate(&self) -> Result<(), ContractValidationError> {
        self.contract.validate()
    }
}

// ---------------------------------------------------------------------------
// RalphLoop — bounded trace reinjection + escalation
// ---------------------------------------------------------------------------

/// Outcome of one ralph-loop step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RalphStep {
    /// The acceptance check passed; the loop is done.
    Accepted,
    /// The acceptance check failed and the trace was reinjected (the generator
    /// gets one retry). Carries the 1-based iteration number.
    Reinjected { iteration: u32 },
    /// The bound was reached; this failure escalates to the adversary instead
    /// of reinjecting. Carries the iteration at which escalation fired.
    Escalated { iteration: u32 },
}

/// Bounded loop counter + escalation logic. Holds NO generator context — it
/// only tracks the iteration count and the effective bound.
#[derive(Debug, Clone)]
pub struct RalphLoop {
    /// Current 0-based count of completed reinjection attempts.
    iteration: u32,
    /// Effective cap (clamped to [`MAX_RALPH_REINJECTIONS`]).
    bound: u32,
}

impl RalphLoop {
    /// Construct from a contract's effective bound. The bound can never exceed
    /// [`MAX_RALPH_REINJECTIONS`] (stub_ban 2: bounded by construction).
    #[must_use]
    pub fn new(contract: &SprintContract) -> Self {
        Self {
            iteration: 0,
            bound: contract.effective_iteration_bound(),
        }
    }

    /// Current iteration count (number of failures observed so far).
    #[must_use]
    pub fn iteration(&self) -> u32 {
        self.iteration
    }

    /// The effective bound for this loop.
    #[must_use]
    pub fn bound(&self) -> u32 {
        self.bound
    }

    /// Record a failure. The first `bound - 1` failures reinject; the
    /// `bound`-th failure escalates. Returns the step taken. After escalation
    /// the loop refuses to advance further (the count is clamped at `bound`).
    pub fn on_failure(&mut self) -> RalphStep {
        if self.iteration >= self.bound {
            // Already escalated — never advance past the bound (stub_ban 2).
            return RalphStep::Escalated {
                iteration: self.bound,
            };
        }
        self.iteration += 1;
        if self.iteration >= self.bound {
            RalphStep::Escalated {
                iteration: self.iteration,
            }
        } else {
            RalphStep::Reinjected {
                iteration: self.iteration,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Free-text bounding helper
// ---------------------------------------------------------------------------

/// Truncate a free-text field to [`SPRINT_FIELD_MAX_CHARS`] (char-boundary
/// safe) before it is surfaced on an SSB row.
#[must_use]
pub fn bound_field(s: &str) -> String {
    if s.chars().count() <= SPRINT_FIELD_MAX_CHARS {
        return s.to_string();
    }
    s.chars().take(SPRINT_FIELD_MAX_CHARS).collect()
}

// ---------------------------------------------------------------------------
// SSB emit helper
// ---------------------------------------------------------------------------

/// Current wall-clock unix-ms (entropy read once at the emit boundary).
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Emit one [`SprintContractEventPayload`] SSB row. Fire-and-forget: errors are
/// traced but never propagated (the gate verdict is unaffected by SSB write
/// failures, mirroring `tdd_preflight::emit_tdd_preflight_blocked_to_ssb`).
pub async fn emit_sprint_contract_event_to_ssb(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    payload: SprintContractEventPayload,
) {
    let ssb_payload = SsbPayload::SprintContractEvent(payload);
    match writer
        .write_payload(agent_id, superstep, ssb_payload, Vec::new())
        .await
    {
        Ok(_) => debug!(sprint_contract.ssb = "emitted SprintContractEvent row"),
        Err(e) => warn!(
            sprint_contract.ssb_err = %e,
            "SprintContractEvent SSB emit failed (verdict unaffected)"
        ),
    }
}

/// Build a payload for a given kind from a contract + loop state. Centralises
/// the field-bounding so every emit site is consistent.
#[must_use]
pub fn build_event_payload(
    kind: SprintContractEventKind,
    contract: &SprintContract,
    iteration: u32,
    objection: &str,
    counterexample: &str,
) -> SprintContractEventPayload {
    SprintContractEventPayload {
        kind,
        contract_id: contract.contract_id(),
        goal: bound_field(&contract.goal),
        iteration,
        objection: bound_field(objection),
        counterexample: bound_field(counterexample),
        ts_ms: now_ms(),
    }
}

// ---------------------------------------------------------------------------
// Acceptance check via the i-17 TriLayerStack (GD-1 — the ralph trigger)
// ---------------------------------------------------------------------------

/// Run the i-17 acceptance check for one branch. The ralph-loop trigger is a
/// `HardVeto` verdict (== failure); `ScoredSurvivor` is a pass. Composes
/// [`TriLayerStack::run_verdict`] — does NOT duplicate the gate logic (GD-1).
/// Returns `Ok(true)` on accept, `Ok(false)` on veto, `Err` on a stack error.
pub async fn acceptance_check(
    stack: &TriLayerStack,
    branch: &Branch,
) -> Result<bool, crate::verifier_stack::VerifierStackError> {
    let row = stack.run_verdict(branch).await?;
    // The ralph-loop trigger is a HardVeto verdict; `is_vetoed()` is the
    // VerdictRow predicate (GD-1 — reuse the i-17 surface, do not re-derive).
    Ok(!row.is_vetoed())
}

// ---------------------------------------------------------------------------
// Adversary escalation (GD-1 — D-13 compose, never re-implement)
// ---------------------------------------------------------------------------

/// Escalate the 3rd failure to the D-13 cross-family adversary. Builds a
/// minimal [`Claim`] from the contract goal and routes it through
/// [`DynAdversary::generate_counter_dyn`]; returns the counterexample text
/// (empty when the adversary reports the claim robust). Errors are returned so
/// the caller can decide retry vs. accept-without-counter; the adversary logic
/// is NOT re-implemented here (stub_ban 4).
pub async fn escalate_to_adversary(
    adversary: &Arc<dyn DynAdversary>,
    config: &AdversaryConfig,
    contract: &SprintContract,
) -> Result<String, crate::adversary::AdversaryError> {
    let claim = Claim {
        claim_id: format!("sprint-{}", contract.contract_id()),
        text: bound_field(&contract.goal),
        claim_type: ClaimType::Procedural,
        evidence: Vec::new(),
        ts_ms: now_ms(),
    };
    let counter = adversary.generate_counter_dyn(&claim, config).await?;
    Ok(bound_field(&counter.counter_text))
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn good_contract() -> SprintContract {
        SprintContract {
            goal: "wire dispatch_sprint_contract into the post-task chain".into(),
            in_scope_files: vec!["crates/orchestrator/src/sprint_contract.rs".into()],
            acceptance_criteria: vec![
                AcceptanceCriterion::TestSymbol("i21_sprint_contract".into()),
                AcceptanceCriterion::CompileStatus,
            ],
            bounded_iteration_count: 3,
            out_of_scope_marker: "no edits outside sprint_contract.rs".into(),
        }
    }

    #[test]
    fn good_contract_validates() {
        assert!(good_contract().validate().is_ok());
    }

    #[test]
    fn empty_goal_rejected() {
        let mut c = good_contract();
        c.goal = "   ".into();
        assert_eq!(c.validate(), Err(ContractValidationError::EmptyGoal));
    }

    #[test]
    fn empty_criteria_rejected() {
        let mut c = good_contract();
        c.acceptance_criteria.clear();
        assert_eq!(
            c.validate(),
            Err(ContractValidationError::NoAcceptanceCriteria)
        );
    }

    #[test]
    fn non_deterministic_criterion_rejected_with_objection() {
        let mut c = good_contract();
        c.acceptance_criteria = vec![AcceptanceCriterion::TestSymbol("  ".into())];
        match c.validate() {
            Err(e @ ContractValidationError::NonDeterministicCriterion { .. }) => {
                assert!(e.objection().contains("not deterministically checkable"));
            }
            other => panic!("expected NonDeterministicCriterion; got {other:?}"),
        }
    }

    #[test]
    fn iteration_bound_clamps_to_max() {
        let mut c = good_contract();
        c.bounded_iteration_count = 99;
        assert_eq!(c.effective_iteration_bound(), MAX_RALPH_REINJECTIONS);
    }

    #[test]
    fn contract_id_is_stable() {
        let c = good_contract();
        assert_eq!(c.contract_id(), c.contract_id(), "id must be deterministic");
        assert_eq!(c.contract_id().len(), 64, "blake3 hex is 64 chars");
    }

    #[test]
    fn ralph_loop_reinjects_then_escalates_at_bound() {
        let c = good_contract(); // bound = 3
        let mut loop_ = RalphLoop::new(&c);
        assert_eq!(loop_.bound(), 3);
        assert_eq!(loop_.on_failure(), RalphStep::Reinjected { iteration: 1 });
        assert_eq!(loop_.on_failure(), RalphStep::Reinjected { iteration: 2 });
        assert_eq!(loop_.on_failure(), RalphStep::Escalated { iteration: 3 });
        // Never advances past the bound.
        assert_eq!(loop_.on_failure(), RalphStep::Escalated { iteration: 3 });
        assert_eq!(loop_.iteration(), 3);
    }

    #[test]
    fn sealed_contract_carries_only_contract_and_spec() {
        let sealed = SealedContract::seal(good_contract(), "task: build i-21");
        assert!(sealed.evaluate().is_ok());
        assert_eq!(sealed.task_spec, "task: build i-21");
    }

    #[test]
    fn bound_field_truncates_at_cap() {
        let big = "x".repeat(SPRINT_FIELD_MAX_CHARS + 500);
        assert_eq!(bound_field(&big).chars().count(), SPRINT_FIELD_MAX_CHARS);
        assert_eq!(bound_field("short"), "short");
    }

    #[test]
    fn enabled_default_off() {
        // In the default CI env the flag is absent => OFF. Derive from the real
        // env so a developer running with the flag set still passes.
        let expected = std::env::var(ELAI_SPRINT_CONTRACT_ENABLE_ENV)
            .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);
        assert_eq!(sprint_contract_enabled(), expected);
    }
}
