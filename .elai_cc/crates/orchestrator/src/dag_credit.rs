//! I-28 Slice 1: deterministic branch-grain DAG credit assignment.
use crate::search::SearchLadderOutcome;
use crate::ssb::{DagCreditAssignedKind, DagCreditAssignedPayload, Ssb, SsbPayload};

/// Default-OFF gate for the I-28 production wire (`ELAI_DAG_CREDIT_ENABLE`).
pub fn dag_credit_enabled() -> bool {
    std::env::var("ELAI_DAG_CREDIT_ENABLE")
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// Deterministic branch-grain credit assigner (no LLM, no Shapley).
pub struct DagCreditAssigner;

impl DagCreditAssigner {
    /// Credit for each non-winner ranked sibling:
    /// `(winner.score - sibling.score).clamp(0.0, 1.0)`.
    pub fn run(outcome: &SearchLadderOutcome) -> Vec<DagCreditAssignedPayload> {
        let Some(winner) = outcome.winner.as_ref() else {
            return Vec::new();
        };
        outcome
            .ranked
            .iter()
            .enumerate()
            .filter(|(_, b)| b.verdict.branch_id != winner.verdict.branch_id)
            .map(|(position, sibling)| DagCreditAssignedPayload {
                kind: DagCreditAssignedKind::CreditWalk,
                branch_id: sibling.verdict.branch_id.clone(),
                credit_score: (winner.verdict.score - sibling.verdict.score).clamp(0.0, 1.0),
                winner_score: winner.verdict.score,
                sibling_score: sibling.verdict.score,
                ranked_position: position,
                ts_ms: 0,
            })
            .collect()
    }
}


/// Provenance required before extracting a preference pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DpoProvenance {
    pub rra_sealed: bool,
}

/// Deterministic DPO pair extractor; vetoed branches never receive an L2 score.
pub struct DpoPairExtractor;

impl DpoPairExtractor {
    /// Extract one negative pruned sibling only when the RRA artifact is sealed.
    pub fn run(
        outcome: &SearchLadderOutcome,
        prov: &DpoProvenance,
    ) -> Vec<DagCreditAssignedPayload> {
        if !prov.rra_sealed {
            return Vec::new();
        }
        let Some(winner) = outcome.winner.as_ref() else {
            return Vec::new();
        };
        let Some(pruned) = outcome.pruned.iter().max_by(|a, b| {
            a.progress_score
                .total_cmp(&b.progress_score)
                .then(b.verdict.branch_id.cmp(&a.verdict.branch_id))
        }) else {
            return Vec::new();
        };
        vec![DagCreditAssignedPayload {
            kind: DagCreditAssignedKind::DpoPairExtracted,
            branch_id: pruned.verdict.branch_id.clone(),
            credit_score: pruned.progress_score,
            winner_score: winner.verdict.score,
            sibling_score: 0.0,
            ranked_position: 0,
            ts_ms: 0,
        }]
    }
}

/// Production emit path (same fn the task_runner dispatcher calls).
pub async fn emit_dag_credit_to_ssb(
    ssb: &Ssb,
    session_id: &str,
    seq: i64,
    payloads: Vec<DagCreditAssignedPayload>,
) -> Result<(), String> {
    for payload in payloads {
        ssb.write_payload(
            session_id,
            seq,
            SsbPayload::DagCreditAssigned(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_off_by_default() {
        std::env::remove_var("ELAI_DAG_CREDIT_ENABLE");
        assert!(!dag_credit_enabled());
    }
}
