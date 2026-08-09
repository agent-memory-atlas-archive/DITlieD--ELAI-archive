//! E-10 -- 3-layer RAG challenger (BM25 + dense + cross-encoder rerank).
//! Decision: E-10-decisions.md DECISION-4 (no ort extension; deterministic
//! placeholder rerank under RUNPOD-PENDING). Runs in PARALLEL with the
//! existing 7-layer research_pipeline.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};

use crate::critic_stats::{paired_bootstrap_p, seed_from_run_id};
use crate::ssb::{CriticDecisionEventPayload, CriticTilePhase, Ssb, SsbError, SsbPayload};

#[derive(Debug, Clone, PartialEq)]
pub struct ThreeLayerRag {
    pub bm25_top_k: Vec<[u8; 8]>,
    pub dense_top_k: Vec<[u8; 8]>,
    pub rerank_top_k: Vec<[u8; 8]>,
    pub mrr: f64,
    pub runpod_pending: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum RagChallengerError {
    #[error("critic-decision SSB write failed: {0}")]
    Ssb(#[from] SsbError),
}

pub async fn run_rag_challenger(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    baseline_mrr: f64,
) -> Result<ThreeLayerRag, RagChallengerError> {
    // RUNPOD-PENDING: cross-encoder rerank requires the ort pin to be
    // bumped or audited; deterministic stub returns zero MRR for now.
    let challenger = ThreeLayerRag {
        bm25_top_k: Vec::new(),
        dense_top_k: Vec::new(),
        rerank_top_k: Vec::new(),
        mrr: 0.0,
        runpod_pending: true,
    };
    // A decision row from the stub report is a fabricated measurement; the dashboard renders it as a verdict.
    if challenger.runpod_pending {
        return Ok(challenger);
    }
    let challenger_mrr = challenger.mrr;
    let baseline_vec = vec![baseline_mrr; 30];
    let challenger_vec = vec![challenger_mrr; 30];
    let run_id_prefix = build_run_id_prefix(agent_id, superstep, b"rag");
    let seed = seed_from_run_id(run_id_prefix);
    let p = paired_bootstrap_p(&baseline_vec, &challenger_vec, seed);
    let verdict_code = if challenger_mrr >= baseline_mrr { 1 } else { 0 };
    let payload = SsbPayload::CriticDecisionEvent(CriticDecisionEventPayload {
        phase: CriticTilePhase::RagChallenger,
        run_id_prefix,
        baseline_score: baseline_mrr,
        challenger_score: challenger_mrr,
        paired_bootstrap_p: p,
        sample_size: 30,
        verdict_code,
        rationale_hash: blake3_prefix_8(b"rag_challenger_3layer_v1"),
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
    use crate::ssb::{SsbActionType, SsbConfig, SsbReader};
    use elai_core::clock::SeededClock;
    use std::sync::Arc;
    use tempfile::TempDir;
    use super::*;
    #[test]
    fn three_layer_rag_runpod_pending() {
        let r = super::ThreeLayerRag {
            bm25_top_k: Vec::new(),
            dense_top_k: Vec::new(),
            rerank_top_k: Vec::new(),
            mrr: 0.0,
            runpod_pending: true,
        };
        assert!(r.runpod_pending);
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pending_report_writes_no_decision_row() {
        let dir = TempDir::new().expect("tempdir");
        let config = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "test-project".into(),
            session_id: "test-session".into(),
        };
        let clock = Arc::new(SeededClock::new()) as Arc<dyn elai_core::clock::ClockProvider>;
        let ssb = Arc::new(Ssb::new(config, clock).expect("ssb init"));
        let _ = run_rag_challenger(&ssb, "test-agent", 0, 0.0).await.expect("run");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let entries = ssb.query_by_type(SsbActionType::CriticDecisionEvent)
            .expect("query_by_type");
        assert_eq!(entries.len(), 0, "pending report must not emit a decision row");
    }

}
