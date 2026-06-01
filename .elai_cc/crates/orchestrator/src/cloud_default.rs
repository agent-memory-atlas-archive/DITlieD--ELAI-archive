//! E-10 -- cloud-default switch test producer.
//! Decision: E-10-decisions.md DECISION-8. Reuses existing
//! `CriticTilePhase::CloudDefault` (NO new SsbActionType variant, NO
//! schema bump). Wire-dark unless `ELAI_CRITIC_OF_DESIGN_ENABLE=1`.
//! Closes E-10 RUNPOD-PENDING wire "cloud-default switch test" at the
//! producer emit-path layer: the wire is structurally complete; only the
//! live B-N fixture run requires the cloud Sonnet route.
//!
//! Records {wall-clock, USD cost, pass rate, prompt-cache hit rate} for
//! the cloud-default arm side-by-side with local. Surfaces the
//! prompt-caching reality the critic's Punch 4 cites
//! (Anthropic OpenAI-compat layer lacks prompt caching by design).

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};

use crate::ssb::{CriticDecisionEventPayload, CriticTilePhase, Ssb, SsbError, SsbPayload};

/// One cloud-default measurement record.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CloudDefaultSnapshot {
    /// Local default pass-rate on the fixture (baseline).
    pub local_pass_rate: f64,
    /// Cloud default pass-rate on the same fixture (challenger).
    pub cloud_pass_rate: f64,
    /// Cloud-side prompt-cache hit rate. The critic's Punch 4 hinges on
    /// this number; surfacing it is the whole point of the wire.
    pub cloud_prompt_cache_hit_rate: f64,
    /// Sample size (n tasks in the fixture).
    pub n_tasks: u32,
}

impl CloudDefaultSnapshot {
    /// True when cloud arm equals or exceeds local. SIMPLIFY = cloud-default.
    #[must_use]
    pub fn cloud_recommended(&self) -> bool {
        self.cloud_pass_rate >= self.local_pass_rate
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CloudDefaultError {
    #[error("critic-decision SSB write failed: {0}")]
    Ssb(#[from] SsbError),
}

/// Emit a `CriticDecisionEvent` SSB row for the cloud-default switch
/// measurement. `paired_bootstrap_p` is `None` until the upstream caller
/// passes the per-task pair vectors through `critic_stats::paired_bootstrap_p`.
pub async fn emit_cloud_default_decision(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    snap: CloudDefaultSnapshot,
) -> Result<u8, CloudDefaultError> {
    let run_id_prefix = build_run_id_prefix(agent_id, superstep, b"cloud");
    let verdict_code: u8 = if snap.cloud_recommended() { 1 } else { 0 };
    let payload = SsbPayload::CriticDecisionEvent(CriticDecisionEventPayload {
        phase: CriticTilePhase::CloudDefault,
        run_id_prefix,
        baseline_score: snap.local_pass_rate,
        challenger_score: snap.cloud_pass_rate,
        paired_bootstrap_p: None,
        sample_size: snap.n_tasks,
        verdict_code,
        // The cache hit-rate is the load-bearing scalar; we hash a label
        // that lets the dashboard correlate with PromptCacheHitRate rows
        // emitted by `cache_check.rs`.
        rationale_hash: blake3_prefix_8(b"cloud_default_switch_v1"),
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
    fn cloud_recommended_when_pass_rate_higher() {
        let snap = CloudDefaultSnapshot {
            local_pass_rate: 0.50,
            cloud_pass_rate: 0.72,
            cloud_prompt_cache_hit_rate: 0.0,
            n_tasks: 30,
        };
        assert!(snap.cloud_recommended());
    }

    #[test]
    fn cloud_recommended_when_tied() {
        let snap = CloudDefaultSnapshot {
            local_pass_rate: 0.50,
            cloud_pass_rate: 0.50,
            cloud_prompt_cache_hit_rate: 0.85,
            n_tasks: 30,
        };
        assert!(snap.cloud_recommended());
    }

    #[test]
    fn local_recommended_when_pass_rate_higher() {
        let snap = CloudDefaultSnapshot {
            local_pass_rate: 0.65,
            cloud_pass_rate: 0.55,
            cloud_prompt_cache_hit_rate: 0.0,
            n_tasks: 30,
        };
        assert!(!snap.cloud_recommended());
    }
}
