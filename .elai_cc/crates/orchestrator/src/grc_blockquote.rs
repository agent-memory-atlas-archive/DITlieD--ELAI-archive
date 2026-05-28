//! E-10 -- GRC blockquote empirical lift measurement.
//! Per CLAUDE.md no-removal: emits the signal but does NOT mutate the rule.
//! Grounding: arXiv:2307.13702 Lanham + arXiv:2510.04040 FaithCoT-Bench.

use std::sync::Arc;

use crate::critic_stats::{paired_bootstrap_p, seed_from_run_id};
use crate::ssb::{CriticDecisionEventPayload, CriticTilePhase, Ssb, SsbError, SsbPayload};

#[derive(Debug, thiserror::Error)]
pub enum GrcBlockquoteError {
    #[error("critic-decision SSB write failed: {0}")]
    Ssb(#[from] SsbError),
}

pub async fn emit_grc_blockquote_lift(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    blockquote_on_scores: &[f64],
    blockquote_off_scores: &[f64],
) -> Result<Option<f64>, GrcBlockquoteError> {
    let baseline_score = mean(blockquote_on_scores);
    let challenger_score = mean(blockquote_off_scores);
    let run_id_prefix = build_run_id_prefix(agent_id, superstep, b"grc");
    let seed = seed_from_run_id(run_id_prefix);
    let p = paired_bootstrap_p(blockquote_on_scores, blockquote_off_scores, seed);
    let verdict_code = if challenger_score >= baseline_score { 1 } else { 0 };
    let payload = SsbPayload::CriticDecisionEvent(CriticDecisionEventPayload {
        phase: CriticTilePhase::GrcBlockquote,
        run_id_prefix,
        baseline_score,
        challenger_score,
        paired_bootstrap_p: p,
        sample_size: blockquote_on_scores.len() as u32,
        verdict_code,
        rationale_hash: blake3_prefix_8(b"grc_blockquote_eval_v1"),
        ts_ms: now_ms(),
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(p)
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() { 0.0 } else { xs.iter().sum::<f64>() / xs.len() as f64 }
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
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mean_empty_zero() { assert_eq!(mean(&[]), 0.0); }
    #[test]
    fn mean_computed() { assert!((mean(&[1.0, 2.0, 3.0]) - 2.0).abs() < 1e-9); }
}
