//! E-10 -- per-VRAM tier matrix decision sheet producer.
//! Decision: E-10-decisions.md DECISION-7. Reuses existing
//! `CriticTilePhase::TierMatrix` (NO new SsbActionType variant, NO schema
//! bump). Wire-dark unless `ELAI_CRITIC_OF_DESIGN_ENABLE=1`. The live
//! benchmark runs (GSM8K-Platinum, LiveCodeBench-v6, tau2-bench) require a
//! sidecar -- this module is the offline-buildable emit path that turns a
//! [`TierMatrixSnapshot`] (filled by an upstream eval runner) into a
//! `CriticDecisionEvent` SSB row. Grounding: BENCH-LM 2026 +
//! Iternal 2026 SWE-bench gap evidence for tier collapse.
//!
//! Closes E-10 RUNPOD-PENDING wire "BENCH-LM live" at the producer
//! emit-path layer: the wire is structurally complete; only the live
//! benchmark inputs require the remote GPU box (`memory:project_qwen3b
//! _baseline.md`).

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};

use crate::ssb::{CriticDecisionEventPayload, CriticTilePhase, Ssb, SsbError, SsbPayload};

/// Per-tier snapshot of cost-normalized eval pass rate. The orchestrator
/// computes both arms upstream (local at the chosen tier vs cloud Sonnet
/// at iso-budget) and hands the numbers to [`emit_tier_matrix_decision`]
/// for SSB persistence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TierMatrixSnapshot {
    /// Local pass-rate at the chosen VRAM tier (e.g. 12GB NF4 Qwen2.5-7B).
    pub local_pass_rate: f64,
    /// Cloud Sonnet pass-rate at iso-budget on the same fixture.
    pub cloud_pass_rate: f64,
    /// Number of fixture tasks in this run (sample size for the verdict).
    pub n_tasks: u32,
}

impl TierMatrixSnapshot {
    /// True when the cloud arm equals or exceeds local at iso-budget. This
    /// is the SIMPLIFY signal for the per-VRAM tier collapse decision (the
    /// critic's punch 7).
    #[must_use]
    pub fn cloud_wins(&self) -> bool {
        self.cloud_pass_rate >= self.local_pass_rate
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TierMatrixError {
    #[error("critic-decision SSB write failed: {0}")]
    Ssb(#[from] SsbError),
}

/// Emit a `CriticDecisionEvent` SSB row for the per-VRAM tier matrix
/// comparison. Baseline = local; challenger = cloud (the critic's proposed
/// simplification). Verdict 1 = Simplify (cloud-default), 0 = Keep
/// (per-VRAM tier).
pub async fn emit_tier_matrix_decision(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    snap: TierMatrixSnapshot,
) -> Result<u8, TierMatrixError> {
    let run_id_prefix = build_run_id_prefix(agent_id, superstep, b"tier");
    let verdict_code: u8 = if snap.cloud_wins() { 1 } else { 0 };
    let payload = SsbPayload::CriticDecisionEvent(CriticDecisionEventPayload {
        phase: CriticTilePhase::TierMatrix,
        run_id_prefix,
        baseline_score: snap.local_pass_rate,
        challenger_score: snap.cloud_pass_rate,
        paired_bootstrap_p: None,
        sample_size: snap.n_tasks,
        verdict_code,
        rationale_hash: blake3_prefix_8(b"tier_matrix_bench_lm_v1"),
        ts_ms: now_ms(),
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(verdict_code)
}

fn build_run_id_prefix(agent_id: &str, superstep: i64, salt: &[u8]) -> [u8; 8] {
    let mut h = blake3::Hasher::new();
    h.update(agent_id.as_bytes());
    h.update(&superstep.to_le_bytes());
    h.update(salt);
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
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_wins_when_pass_rate_higher() {
        let snap = TierMatrixSnapshot {
            local_pass_rate: 0.42,
            cloud_pass_rate: 0.71,
            n_tasks: 30,
        };
        assert!(snap.cloud_wins());
    }

    #[test]
    fn cloud_wins_when_tied() {
        let snap = TierMatrixSnapshot {
            local_pass_rate: 0.50,
            cloud_pass_rate: 0.50,
            n_tasks: 30,
        };
        assert!(snap.cloud_wins());
    }

    #[test]
    fn local_wins_when_pass_rate_lower_for_cloud() {
        let snap = TierMatrixSnapshot {
            local_pass_rate: 0.60,
            cloud_pass_rate: 0.55,
            n_tasks: 30,
        };
        assert!(!snap.cloud_wins());
    }
}
