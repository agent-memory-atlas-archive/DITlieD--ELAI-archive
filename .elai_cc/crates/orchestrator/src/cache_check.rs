//! E-10 -- prompt-cache hit-rate consumer aggregator.
//! Decision: E-10-decisions.md DECISION-6. READ-side aggregator only;
//! conversation.rs already surfaces prompt_cache_events.
//! Expected hit rate is LOW on Anthropic-compat path -- that is the signal.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};

use crate::ssb::{CriticDecisionEventPayload, CriticTilePhase, Ssb, SsbError, SsbPayload};

#[derive(Debug, thiserror::Error)]
pub enum CacheCheckError {
    #[error("critic-decision SSB write failed: {0}")]
    Ssb(#[from] SsbError),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CacheCheckSnapshot {
    pub hits: u64,
    pub misses: u64,
}

impl CacheCheckSnapshot {
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { (self.hits as f64) / (total as f64) }
    }
}

pub async fn measure_cache_hit_rate(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    snap: CacheCheckSnapshot,
) -> Result<f64, CacheCheckError> {
    let challenger = snap.hit_rate();
    let baseline = 0.0;
    let run_id_prefix = build_run_id_prefix(agent_id, superstep, b"cache");
    let verdict_code = if challenger > 0.05 { 1 } else { 0 };
    let payload = SsbPayload::CriticDecisionEvent(CriticDecisionEventPayload {
        phase: CriticTilePhase::PromptCacheHitRate,
        run_id_prefix,
        baseline_score: baseline,
        challenger_score: challenger,
        paired_bootstrap_p: None,
        sample_size: (snap.hits + snap.misses) as u32,
        verdict_code,
        rationale_hash: blake3_prefix_8(b"cache_check_anthropic_v1"),
        ts_ms: now_ms(),
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(challenger)
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
    fn hit_rate_zero_total() {
        assert_eq!(CacheCheckSnapshot { hits: 0, misses: 0 }.hit_rate(), 0.0);
    }
    #[test]
    fn hit_rate_computed() {
        assert!((CacheCheckSnapshot { hits: 1, misses: 3 }.hit_rate() - 0.25).abs() < 1e-9);
    }
}
