use std::path::PathBuf;
use std::sync::Arc;

use ort::session::builder::GraphOptimizationLevel;
use tokio::sync::{mpsc, oneshot, Semaphore};

use crate::arag::RetrievalResult;
use crate::vector::circuit_breaker::CircuitBreaker;
use elai_core::ClockProvider;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum RerankerError {
    ModelLoad(String),
    TokenizerLoad(String),
    Inference(String),
    ChannelFull,
}

impl std::fmt::Display for RerankerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelLoad(e) => write!(f, "reranker model load failed: {e}"),
            Self::TokenizerLoad(e) => write!(f, "reranker tokenizer load failed: {e}"),
            Self::Inference(e) => write!(f, "reranker inference failed: {e}"),
            Self::ChannelFull => write!(f, "reranker queue full, returning unranked"),
        }
    }
}

impl std::error::Error for RerankerError {}

// ---------------------------------------------------------------------------
// RerankerConfig
// ---------------------------------------------------------------------------

pub struct RerankerConfig {
    /// Path to primary L-12 cross-encoder ONNX model.
    pub model_path: PathBuf,
    /// Path to tokenizer.json (shared WordPiece vocab for L-12 and L-6).
    pub tokenizer_path: PathBuf,
    /// Optional L-6 fallback model (spec amendment: intermediate degradation tier).
    pub fallback_model_path: Option<PathBuf>,
    /// Optional INT8 quantized model (generated in 1G.5 if needed).
    pub quantized_model_path: Option<PathBuf>,
    /// Candidate count at or below which reranking is skipped (default 10).
    pub skip_threshold: usize,
    /// Maximum BERT sequence length (default 512).
    pub max_sequence_length: usize,
    /// Bounded mpsc channel capacity for request batching (default 4).
    pub max_queue_depth: usize,
    /// ORT intra-op thread count (default 1 for deterministic tests).
    pub intra_op_threads: usize,
    /// ORT graph optimization level (default ORT_ENABLE_ALL).
    pub graph_optimization_level: GraphOptimizationLevel,
    /// Required for multi-thread score reproducibility (default true).
    pub enable_deterministic_compute: bool,
}

impl Default for RerankerConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/cross-encoder-l12.onnx"),
            tokenizer_path: PathBuf::from("models/tokenizer.json"),
            fallback_model_path: None,
            quantized_model_path: None,
            skip_threshold: 10,
            max_sequence_length: 512,
            max_queue_depth: 4,
            intra_op_threads: 1,
            graph_optimization_level: GraphOptimizationLevel::All,
            enable_deterministic_compute: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Batch request type (for mpsc channel)
// ---------------------------------------------------------------------------

#[allow(dead_code)] // Fields used in 1G.3 batch scoring
pub(crate) struct RerankerRequest {
    pub query: String,
    pub candidates: Vec<RetrievalResult>,
    pub reply: oneshot::Sender<Result<Vec<RetrievalResult>, RerankerError>>,
}

// ---------------------------------------------------------------------------
// CrossEncoderReranker
// ---------------------------------------------------------------------------

pub struct CrossEncoderReranker {
    config: RerankerConfig,
    #[allow(dead_code)] // Used in 1G.3 concurrency
    semaphore: Arc<Semaphore>,
    #[allow(dead_code)] // Used in 1G.3 batch channel
    request_tx: mpsc::Sender<RerankerRequest>,
    #[allow(dead_code)] // Used in 1G.4 circuit breaker integration
    circuit_breaker: std::sync::Mutex<CircuitBreaker>,
    #[allow(dead_code)] // Used in 1G.4 clock-injectable breaker
    clock: Arc<dyn ClockProvider>,
}

impl CrossEncoderReranker {
    /// Create a new reranker. Does NOT load the ONNX model (deferred to 1G.2).
    /// Sets up the concurrency primitives: semaphore(1), bounded mpsc channel.
    pub fn new(config: RerankerConfig, clock: Arc<dyn ClockProvider>) -> Self {
        let queue_depth = config.max_queue_depth;
        let (request_tx, _request_rx) = mpsc::channel::<RerankerRequest>(queue_depth);
        let circuit_breaker = CircuitBreaker::new(clock.clone());

        Self {
            config,
            semaphore: Arc::new(Semaphore::new(1)),
            request_tx,
            circuit_breaker: std::sync::Mutex::new(circuit_breaker),
            clock,
        }
    }

    /// Rerank candidates using the cross-encoder model.
    ///
    /// Step 1G.1 stub: returns candidates unchanged (pass-through).
    /// Real scoring implemented in 1G.3.
    ///
    /// Skip conditions:
    /// - candidate count <= skip_threshold
    /// - SPARSE_ONLY / BM25_ONLY modes (caller responsibility, not checked here)
    pub fn rerank(
        &self,
        _query: &str,
        candidates: Vec<RetrievalResult>,
    ) -> Result<Vec<RetrievalResult>, RerankerError> {
        if candidates.len() <= self.config.skip_threshold {
            return Ok(candidates);
        }
        // Stub: return candidates in original order.
        // 1G.3 implements: tokenize pairs, run ONNX inference, re-sort by score.
        Ok(candidates)
    }

    pub fn config(&self) -> &RerankerConfig {
        &self.config
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::SeededClock;

    fn make_config() -> RerankerConfig {
        RerankerConfig::default()
    }

    fn make_reranker() -> CrossEncoderReranker {
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        CrossEncoderReranker::new(make_config(), clock)
    }

    fn make_candidates(n: usize) -> Vec<RetrievalResult> {
        (0..n)
            .map(|i| RetrievalResult {
                chunk_id: format!("chunk_{i}"),
                snippet: format!("fn func_{i}() {{}}"),
                score: (n - i) as f64,
                tool_used: "bm25".to_string(),
            })
            .collect()
    }

    #[test]
    fn reranker_config_default_compiles() {
        let cfg = RerankerConfig::default();
        assert_eq!(cfg.skip_threshold, 10);
        assert_eq!(cfg.max_sequence_length, 512);
        assert_eq!(cfg.max_queue_depth, 4);
        assert_eq!(cfg.intra_op_threads, 1);
        assert_eq!(cfg.graph_optimization_level, GraphOptimizationLevel::All);
        assert!(cfg.enable_deterministic_compute);
        assert!(cfg.fallback_model_path.is_none());
        assert!(cfg.quantized_model_path.is_none());
    }

    #[test]
    fn reranker_new_does_not_panic() {
        let _reranker = make_reranker();
    }

    #[test]
    fn rerank_below_threshold_returns_unchanged() {
        let reranker = make_reranker();
        let candidates = make_candidates(5);
        let original_ids: Vec<_> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
        let result = reranker.rerank("test query", candidates).unwrap();
        let result_ids: Vec<_> = result.iter().map(|c| c.chunk_id.clone()).collect();
        assert_eq!(original_ids, result_ids);
    }

    #[test]
    fn rerank_at_threshold_returns_unchanged() {
        let reranker = make_reranker();
        // Exactly 10 candidates = skip_threshold, should be skipped (<= 10)
        let candidates = make_candidates(10);
        let original_ids: Vec<_> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
        let result = reranker.rerank("test query", candidates).unwrap();
        let result_ids: Vec<_> = result.iter().map(|c| c.chunk_id.clone()).collect();
        assert_eq!(original_ids, result_ids);
    }

    #[test]
    fn rerank_above_threshold_stub_returns_unchanged() {
        let reranker = make_reranker();
        // 11 candidates > skip_threshold=10, enters rerank path (stub returns unchanged)
        let candidates = make_candidates(11);
        let original_ids: Vec<_> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
        let result = reranker.rerank("test query", candidates).unwrap();
        let result_ids: Vec<_> = result.iter().map(|c| c.chunk_id.clone()).collect();
        assert_eq!(original_ids, result_ids);
    }

    #[test]
    fn rerank_empty_returns_empty() {
        let reranker = make_reranker();
        let result = reranker.rerank("test query", Vec::new()).unwrap();
        assert!(result.is_empty());
    }
}
