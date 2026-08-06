//! J-92: deterministic capability tokens for orchestrator dispatch ordering.
use std::sync::atomic::{AtomicU64, Ordering};

pub const ELAI_ORCHESTRATOR_FSM_TOKENS_ENABLE: &str = "ELAI_ORCHESTRATOR_FSM_TOKENS_ENABLE";
static NONCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DispatchPhase { ParallelBatch, Executing, Auditing, Memorizing }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DispatchAction { EditPlanScope, RunTests, MemoryWrite }

impl DispatchAction {
    pub const fn required_phase(self) -> DispatchPhase { match self {
        Self::EditPlanScope => DispatchPhase::ParallelBatch,
        Self::RunTests => DispatchPhase::Auditing,
        Self::MemoryWrite => DispatchPhase::Memorizing,
    }}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchGateOutcome { Refused, Observed }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchTokenError {
    EmptyEvidence,
    WrongPhase { required: DispatchPhase, actual: DispatchPhase },
    InvalidToken,
}

/// Opaque capability. Its fields are private deliberately: callers can only obtain
/// one through `mint_on_evidence`, never by supplying digest bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityToken { digest: [u8; 32], phase: DispatchPhase, evidence_id: String, nonce: u64 }

fn enabled() -> bool { std::env::var(ELAI_ORCHESTRATOR_FSM_TOKENS_ENABLE).map(|v| !v.trim().is_empty() && v != "0").unwrap_or(false) }

/// Public accessor for the dispatch-token kill-switch. Used by the supervisor
/// seam (`InProcessSupervisor::execute_directive`) to decide whether the
/// `file_write` path requires a stored capability token (flag ON) or records
/// an observe-only illegal-transition outcome (flag OFF, default).
pub fn orchestrator_fsm_tokens_enabled() -> bool { enabled() }
fn digest(phase: DispatchPhase, evidence: &str, nonce: u64) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(format!("{:?}\0{}\0{}", phase, evidence, nonce).as_bytes());
    *h.finalize().as_bytes()
}

pub fn mint_on_evidence(phase: DispatchPhase, evidence_id: &str) -> Result<CapabilityToken, DispatchTokenError> {
    if evidence_id.trim().is_empty() { return Err(DispatchTokenError::EmptyEvidence); }
    let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
    Ok(CapabilityToken { digest: digest(phase, evidence_id, nonce), phase, evidence_id: evidence_id.to_owned(), nonce })
}

pub fn consume_for_action(token: &CapabilityToken, required_phase: DispatchPhase, action: DispatchAction) -> Result<(), DispatchTokenError> {
    let expected = action.required_phase();
    if required_phase != expected { return Err(DispatchTokenError::WrongPhase { required: expected, actual: required_phase }); }
    if token.phase != required_phase || token.digest != digest(token.phase, &token.evidence_id, token.nonce) { return Err(DispatchTokenError::InvalidToken); }
    Ok(())
}

pub fn refuse_illegal_transition(_attempted: DispatchAction, _missing_evidence: &str) -> DispatchGateOutcome {
    if enabled() { DispatchGateOutcome::Refused } else { DispatchGateOutcome::Observed }
}

#[cfg(test)]
mod j92_dispatch_token_tests {
    use super::*;
    #[test] fn mint_and_consume_happy_path() { let t=mint_on_evidence(DispatchPhase::ParallelBatch,"e").unwrap(); assert!(consume_for_action(&t,DispatchPhase::ParallelBatch,DispatchAction::EditPlanScope).is_ok()); }
    #[test] fn mint_without_evidence_fails() { assert_eq!(mint_on_evidence(DispatchPhase::ParallelBatch,""),Err(DispatchTokenError::EmptyEvidence)); }
    #[test] fn consume_wrong_phase_fails() { let t=mint_on_evidence(DispatchPhase::ParallelBatch,"e").unwrap(); assert!(consume_for_action(&t,DispatchPhase::Auditing,DispatchAction::EditPlanScope).is_err()); }
    #[test] fn flag_off_observes_dont_block() { std::env::remove_var(ELAI_ORCHESTRATOR_FSM_TOKENS_ENABLE); assert_eq!(refuse_illegal_transition(DispatchAction::EditPlanScope,"e"),DispatchGateOutcome::Observed); }
    #[test] fn forge_attempt_fails() { let t=mint_on_evidence(DispatchPhase::ParallelBatch,"e").unwrap(); let mut bad=t.clone(); bad.evidence_id="forged".into(); assert_eq!(consume_for_action(&bad,DispatchPhase::ParallelBatch,DispatchAction::EditPlanScope),Err(DispatchTokenError::InvalidToken)); }
}
