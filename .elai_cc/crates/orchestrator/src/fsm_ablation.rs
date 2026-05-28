//! E-10 -- FSM-state ablation harness wrapper.
//!
//! Decision: workflow/reasoning/E-10-decisions.md DECISION-2. WRAPS
//! `iso_budget_harness::run_ablation` positional signature `(ssb,
//! agent_id, superstep, task_fixture, budget_tokens)`. Does NOT
//! mutate the E-6 harness signature.
//!
//! Wire-dark unless `ELAI_CRITIC_OF_DESIGN_ENABLE=1`. Live wiring
//! RUNPOD-PENDING.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::critic_stats::{paired_bootstrap_p, seed_from_run_id};
use crate::iso_budget_harness::{self, IsoBudgetError, IsoBudgetReport};
use crate::ssb::{CriticDecisionEventPayload, CriticTilePhase, Ssb, SsbError, SsbPayload};

/// FSM profile selector (config struct, not an FsmSubstate variant).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsmProfile {
    Full9State,
    Reduced3State,
}

impl FsmProfile {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full9State => "full_9_state",
            Self::Reduced3State => "reduced_3_state",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FsmAblationError {
    #[error("iso-budget run failed: {0}")]
    IsoBudget(#[from] IsoBudgetError),
    #[error("critic-decision SSB write failed: {0}")]
    Ssb(#[from] SsbError),
}

/// Run the FSM-state ablation: invokes the iso-budget harness, then
/// emits a CriticDecisionEvent SSB row with paired-bootstrap p.
pub async fn run_fsm_ablation(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    task_fixture: &str,
    budget_tokens: u32,
) -> Result<IsoBudgetReport, FsmAblationError> {
    let report = iso_budget_harness::run_ablation(
        ssb, agent_id, superstep, task_fixture, budget_tokens,
    )
    .await?;

    let baseline_score = report.full_pipeline.pass_rate;
    let challenger_score = report.single_agent_wait.pass_rate;
    let baseline_vec = vec![baseline_score; 30];
    let challenger_vec = vec![challenger_score; 30];
    let run_id_prefix = build_run_id_prefix(agent_id, superstep);
    let seed = seed_from_run_id(run_id_prefix);
    let p = paired_bootstrap_p(&baseline_vec, &challenger_vec, seed);

    let verdict_code = if challenger_score >= baseline_score { 1 } else { 0 };
    let rationale_hash = blake3_prefix_8(b"fsm_ablation_v1");

    let payload = SsbPayload::CriticDecisionEvent(CriticDecisionEventPayload {
        phase: CriticTilePhase::FsmAblation,
        run_id_prefix,
        baseline_score,
        challenger_score,
        paired_bootstrap_p: p,
        sample_size: 30,
        verdict_code,
        rationale_hash,
        ts_ms: now_ms(),
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(report)
}

fn build_run_id_prefix(agent_id: &str, superstep: i64) -> [u8; 8] {
    let mut h = blake3::Hasher::new();
    h.update(agent_id.as_bytes());
    h.update(&superstep.to_le_bytes());
    let d = h.finalize();
    let b = d.as_bytes();
    [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
}

fn blake3_prefix_8(input: &[u8]) -> [u8; 8] {
    let mut h = blake3::Hasher::new();
    h.update(input);
    let d = h.finalize();
    let b = d.as_bytes();
    [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fsm_profile_str() {
        assert_eq!(FsmProfile::Full9State.as_str(), "full_9_state");
        assert_eq!(FsmProfile::Reduced3State.as_str(), "reduced_3_state");
    }

    #[test]
    fn run_id_prefix_deterministic() {
        let a = build_run_id_prefix("agent-1", 42);
        let b = build_run_id_prefix("agent-1", 42);
        assert_eq!(a, b);
    }
}
