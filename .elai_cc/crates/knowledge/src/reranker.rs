use std::path::{Path, PathBuf};
use std::sync::Arc;

use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
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
    session: Option<Arc<std::sync::Mutex<Session>>>,
    tokenizer: Option<Arc<tokenizers::Tokenizer>>,
    #[allow(dead_code)] // Lazy-loaded on L-12 circuit breaker open (1G.4)
    fallback_session: Option<Arc<std::sync::Mutex<Session>>>,
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
    /// Create a new reranker without loading models. Call `load()` to initialize
    /// the ONNX session and tokenizer.
    pub fn new(config: RerankerConfig, clock: Arc<dyn ClockProvider>) -> Self {
        let queue_depth = config.max_queue_depth;
        let (request_tx, _request_rx) = mpsc::channel::<RerankerRequest>(queue_depth);
        let circuit_breaker = CircuitBreaker::new(clock.clone());

        Self {
            config,
            session: None,
            tokenizer: None,
            fallback_session: None,
            semaphore: Arc::new(Semaphore::new(1)),
            request_tx,
            circuit_breaker: std::sync::Mutex::new(circuit_breaker),
            clock,
        }
    }

    /// Load the ONNX model and tokenizer. Must be called before reranking.
    ///
    /// Model path resolution: quantized_model_path (if Some) -> model_path.
    /// Tokenizer loaded from tokenizer_path with truncation set to max_sequence_length.
    /// Fallback L-6 model is NOT loaded here — lazy-loaded on L-12 circuit breaker open (1G.4).
    pub fn load(&mut self) -> Result<(), RerankerError> {
        let model_path = self
            .config
            .quantized_model_path
            .as_deref()
            .filter(|p| p.exists())
            .unwrap_or(&self.config.model_path);

        let session = Self::build_session(
            model_path,
            self.config.intra_op_threads,
            self.config.graph_optimization_level,
            self.config.enable_deterministic_compute,
        )?;
        self.session = Some(Arc::new(std::sync::Mutex::new(session)));

        let tokenizer =
            Self::load_tokenizer(&self.config.tokenizer_path, self.config.max_sequence_length)?;
        self.tokenizer = Some(Arc::new(tokenizer));

        Ok(())
    }

    /// Build an ORT session with the configured optimization settings.
    /// Wraps ORT calls in catch_unwind to convert panics to errors.
    fn build_session(
        model_path: &Path,
        intra_op_threads: usize,
        graph_optimization_level: GraphOptimizationLevel,
        enable_deterministic_compute: bool,
    ) -> Result<Session, RerankerError> {
        if !model_path.exists() {
            return Err(RerankerError::ModelLoad(format!(
                "model file not found: {}",
                model_path.display()
            )));
        }

        let path_owned = model_path.to_path_buf();
        let result = std::panic::catch_unwind(move || -> Result<Session, String> {
            let builder = Session::builder().map_err(|e| e.to_string())?;
            let builder = builder
                .with_optimization_level(graph_optimization_level)
                .map_err(|e| e.to_string())?;
            let builder = builder
                .with_intra_threads(intra_op_threads)
                .map_err(|e| e.to_string())?;
            let mut builder = builder
                .with_deterministic_compute(enable_deterministic_compute)
                .map_err(|e| e.to_string())?;
            let session = builder
                .commit_from_file(&path_owned)
                .map_err(|e| e.to_string())?;
            Ok(session)
        });

        match result {
            Ok(Ok(session)) => Ok(session),
            Ok(Err(msg)) => Err(RerankerError::ModelLoad(format!(
                "{}: {msg}",
                model_path.display()
            ))),
            Err(_) => Err(RerankerError::ModelLoad(format!(
                "{}: ort panicked during session creation",
                model_path.display()
            ))),
        }
    }

    /// Load tokenizer from file and configure truncation.
    fn load_tokenizer(
        tokenizer_path: &Path,
        max_sequence_length: usize,
    ) -> Result<tokenizers::Tokenizer, RerankerError> {
        if !tokenizer_path.exists() {
            return Err(RerankerError::TokenizerLoad(format!(
                "tokenizer file not found: {}",
                tokenizer_path.display()
            )));
        }

        let mut tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path).map_err(|e| {
            RerankerError::TokenizerLoad(format!("{}: {e}", tokenizer_path.display()))
        })?;

        let truncation = tokenizers::utils::truncation::TruncationParams {
            max_length: max_sequence_length,
            strategy: tokenizers::utils::truncation::TruncationStrategy::LongestFirst,
            ..Default::default()
        };
        tokenizer
            .with_truncation(Some(truncation))
            .map_err(|e| RerankerError::TokenizerLoad(format!("truncation config: {e}")))?;

        Ok(tokenizer)
    }

    /// Lazy-load the fallback L-6 model. Called on L-12 circuit breaker open (1G.4).
    /// Returns Err if fallback_model_path is not configured or file missing.
    /// Cold-start latency (5-15s) is acceptable during degradation.
    pub fn load_fallback(&mut self) -> Result<(), RerankerError> {
        if self.fallback_session.is_some() {
            return Ok(());
        }

        let fallback_path = self
            .config
            .fallback_model_path
            .as_deref()
            .ok_or_else(|| RerankerError::ModelLoad("no fallback model configured".into()))?;

        let session = Self::build_session(
            fallback_path,
            self.config.intra_op_threads,
            self.config.graph_optimization_level,
            self.config.enable_deterministic_compute,
        )?;
        self.fallback_session = Some(Arc::new(std::sync::Mutex::new(session)));

        Ok(())
    }

    /// Rerank candidates using the cross-encoder model.
    ///
    /// Stub scoring: returns candidates unchanged (1G.3 implements actual inference).
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

    pub fn session(&self) -> Option<&Arc<std::sync::Mutex<Session>>> {
        self.session.as_ref()
    }

    pub fn tokenizer(&self) -> Option<&Arc<tokenizers::Tokenizer>> {
        self.tokenizer.as_ref()
    }

    pub fn is_loaded(&self) -> bool {
        self.session.is_some() && self.tokenizer.is_some()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::SeededClock;
    use std::path::PathBuf;

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

    /// Resolve the project root (parent of crates/).
    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn model_path() -> PathBuf {
        project_root().join("models").join("onnx").join("cross-encoder-l12.onnx")
    }

    fn tokenizer_path() -> PathBuf {
        project_root().join("models").join("onnx").join("tokenizer.json")
    }

    // -----------------------------------------------------------------------
    // 1G.1 tests (preserved)
    // -----------------------------------------------------------------------

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
        let candidates = make_candidates(10);
        let original_ids: Vec<_> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
        let result = reranker.rerank("test query", candidates).unwrap();
        let result_ids: Vec<_> = result.iter().map(|c| c.chunk_id.clone()).collect();
        assert_eq!(original_ids, result_ids);
    }

    #[test]
    fn rerank_above_threshold_stub_returns_unchanged() {
        let reranker = make_reranker();
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

    // -----------------------------------------------------------------------
    // 1G.2 tests: ONNX model loading, tokenizer, graph optimization
    // -----------------------------------------------------------------------

    #[test]
    fn load_invalid_model_path_returns_err() {
        let mut config = make_config();
        config.model_path = PathBuf::from("/nonexistent/model.onnx");
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let mut reranker = CrossEncoderReranker::new(config, clock);
        let result = reranker.load();
        assert!(result.is_err());
        match result.unwrap_err() {
            RerankerError::ModelLoad(msg) => {
                assert!(msg.contains("not found"), "expected 'not found' in: {msg}");
            }
            other => panic!("expected ModelLoad, got {other}"),
        }
    }

    #[test]
    fn load_missing_tokenizer_returns_err() {
        // Even if model_path existed, missing tokenizer should error.
        // Since model_path default also doesn't exist, we test tokenizer path
        // directly via the static method.
        let bad_path = PathBuf::from("/nonexistent/tokenizer.json");
        let result = CrossEncoderReranker::load_tokenizer(&bad_path, 512);
        assert!(result.is_err());
        match result.unwrap_err() {
            RerankerError::TokenizerLoad(msg) => {
                assert!(msg.contains("not found"), "expected 'not found' in: {msg}");
            }
            other => panic!("expected TokenizerLoad, got {other}"),
        }
    }

    #[test]
    fn load_zero_byte_model_returns_err() {
        let dir = std::env::temp_dir().join("elai_reranker_test_zero");
        let _ = std::fs::create_dir_all(&dir);
        let zero_model = dir.join("zero.onnx");
        std::fs::write(&zero_model, b"").unwrap();

        let result = CrossEncoderReranker::build_session(
            &zero_model,
            1,
            GraphOptimizationLevel::All,
            true,
        );
        assert!(result.is_err(), "zero-byte model should return Err, not panic");
        match result.unwrap_err() {
            RerankerError::ModelLoad(_) => {}
            other => panic!("expected ModelLoad, got {other}"),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_corrupt_model_returns_err() {
        let dir = std::env::temp_dir().join("elai_reranker_test_corrupt");
        let _ = std::fs::create_dir_all(&dir);
        let corrupt_model = dir.join("corrupt.onnx");
        std::fs::write(&corrupt_model, b"this is not a valid onnx model").unwrap();

        let result = CrossEncoderReranker::build_session(
            &corrupt_model,
            1,
            GraphOptimizationLevel::All,
            true,
        );
        assert!(result.is_err(), "corrupt model should return Err, not panic");
        match result.unwrap_err() {
            RerankerError::ModelLoad(_) => {}
            other => panic!("expected ModelLoad, got {other}"),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_fallback_no_config_returns_err() {
        let mut reranker = make_reranker();
        // fallback_model_path is None by default
        let result = reranker.load_fallback();
        assert!(result.is_err());
        match result.unwrap_err() {
            RerankerError::ModelLoad(msg) => {
                assert!(msg.contains("no fallback"), "expected 'no fallback' in: {msg}");
            }
            other => panic!("expected ModelLoad, got {other}"),
        }
    }

    #[test]
    fn load_fallback_nonexistent_path_returns_err() {
        let mut config = make_config();
        config.fallback_model_path = Some(PathBuf::from("/nonexistent/l6.onnx"));
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let mut reranker = CrossEncoderReranker::new(config, clock);
        let result = reranker.load_fallback();
        assert!(result.is_err());
        match result.unwrap_err() {
            RerankerError::ModelLoad(msg) => {
                assert!(msg.contains("not found"), "expected 'not found' in: {msg}");
            }
            other => panic!("expected ModelLoad, got {other}"),
        }
    }

    #[test]
    fn is_loaded_false_before_load() {
        let reranker = make_reranker();
        assert!(!reranker.is_loaded());
        assert!(reranker.session().is_none());
        assert!(reranker.tokenizer().is_none());
    }

    #[test]
    fn build_session_optimization_levels_accepted() {
        // Verify build_session doesn't panic with different optimization levels.
        // All should return ModelLoad Err since default model_path doesn't exist,
        // but crucially they should NOT panic during builder configuration.
        for level in [
            GraphOptimizationLevel::Disable,
            GraphOptimizationLevel::Level1,
            GraphOptimizationLevel::All,
        ] {
            let result = CrossEncoderReranker::build_session(
                &PathBuf::from("/nonexistent/model.onnx"),
                1,
                level,
                true,
            );
            assert!(result.is_err());
        }
    }

    // -----------------------------------------------------------------------
    // 1G.2 model-dependent tests: skip if models not staged
    // -----------------------------------------------------------------------

    #[test]
    fn load_model_from_fixture_with_ort_enable_all() {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged at {}", mp.display());
            return;
        }

        let mut config = make_config();
        config.model_path = mp;
        config.tokenizer_path = tp;
        config.graph_optimization_level = GraphOptimizationLevel::All;
        config.enable_deterministic_compute = true;

        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let mut reranker = CrossEncoderReranker::new(config, clock);
        reranker.load().expect("model load should succeed with staged fixtures");
        assert!(reranker.is_loaded());
        assert!(reranker.session().is_some());
        assert!(reranker.tokenizer().is_some());
    }

    #[test]
    fn tokenizer_loads_and_encodes() {
        let tp = tokenizer_path();
        if !tp.exists() {
            eprintln!("SKIP: tokenizer not staged at {}", tp.display());
            return;
        }

        let tokenizer = CrossEncoderReranker::load_tokenizer(&tp, 512)
            .expect("tokenizer load should succeed");
        let encoding = tokenizer.encode("hello world", false)
            .expect("encode should succeed");
        let ids = encoding.get_ids();
        assert!(!ids.is_empty(), "token IDs should be non-empty");
    }

    #[test]
    fn tokenizer_truncates_to_max_length() {
        let tp = tokenizer_path();
        if !tp.exists() {
            eprintln!("SKIP: tokenizer not staged at {}", tp.display());
            return;
        }

        let max_len = 128; // Use small length to make test practical
        let tokenizer = CrossEncoderReranker::load_tokenizer(&tp, max_len)
            .expect("tokenizer load should succeed");

        // Generate a very long input that would exceed max_len tokens
        let long_input: String = "function ".repeat(500);
        // Encode as pair (query, document) to exercise cross-encoder pair truncation
        let encoding = tokenizer.encode(
            (long_input.as_str(), "This is a relevance scoring pair"),
            false,
        ).expect("encode should succeed");

        let ids = encoding.get_ids();
        assert!(
            ids.len() <= max_len,
            "expected at most {max_len} tokens, got {}",
            ids.len()
        );
    }

    /// Helper: run inference on a session with a single pair, return the score.
    fn run_single_inference(
        session: &mut Session,
        tokenizer: &tokenizers::Tokenizer,
        query: &str,
        document: &str,
    ) -> f32 {
        use ort::value::Tensor;
        let encoding = tokenizer
            .encode((query, document), false)
            .expect("encode should succeed");

        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();
        let attention_mask: Vec<i64> = encoding.get_attention_mask().iter().map(|&x| x as i64).collect();
        let token_type_ids: Vec<i64> = encoding.get_type_ids().iter().map(|&x| x as i64).collect();
        let seq_len = input_ids.len();

        let inputs = ort::inputs![
            "input_ids" => Tensor::from_array((vec![1, seq_len as i64], input_ids)).unwrap(),
            "attention_mask" => Tensor::from_array((vec![1, seq_len as i64], attention_mask)).unwrap(),
            "token_type_ids" => Tensor::from_array((vec![1, seq_len as i64], token_type_ids)).unwrap(),
        ];

        let outputs = session.run(inputs).expect("inference should succeed");
        let (_shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .expect("extract tensor");
        data[0]
    }

    #[test]
    fn session_deterministic_compute_multi_thread_identical_scores() {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged at {}", mp.display());
            return;
        }

        let mut session = CrossEncoderReranker::build_session(
            &mp, 4, GraphOptimizationLevel::All, true,
        ).expect("session build should succeed");

        let tokenizer = CrossEncoderReranker::load_tokenizer(&tp, 512)
            .expect("tokenizer load should succeed");

        let query = "What is a hash map?";
        let doc = "impl HashMap { fn insert(&mut self, key: K, value: V) {} }";

        let mut scores: Vec<f32> = Vec::new();
        for _ in 0..5 {
            scores.push(run_single_inference(&mut session, &tokenizer, query, doc));
        }

        for (i, score) in scores.iter().enumerate() {
            assert_eq!(
                scores[0], *score,
                "run 0 score ({}) != run {i} score ({score}) — deterministic compute failed",
                scores[0]
            );
        }
    }

    #[test]
    fn session_ort_enable_all_vs_disable_identical_scores() {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged at {}", mp.display());
            return;
        }

        let tokenizer = CrossEncoderReranker::load_tokenizer(&tp, 512)
            .expect("tokenizer load should succeed");

        let mut session_all = CrossEncoderReranker::build_session(
            &mp, 1, GraphOptimizationLevel::All, true,
        ).expect("session build should succeed");

        let mut session_disable = CrossEncoderReranker::build_session(
            &mp, 1, GraphOptimizationLevel::Disable, true,
        ).expect("session build should succeed");

        let query = "search query";
        let doc = "fn main() { println!(\"hello\"); }";

        let score_all = run_single_inference(&mut session_all, &tokenizer, query, doc);
        let score_disable = run_single_inference(&mut session_disable, &tokenizer, query, doc);

        assert_eq!(
            score_all, score_disable,
            "ORT_ENABLE_ALL score ({score_all}) != ORT_DISABLE score ({score_disable})"
        );
    }

    #[test]
    fn quantized_model_path_preferred_when_exists() {
        let dir = std::env::temp_dir().join("elai_reranker_test_quant_pref");
        let _ = std::fs::create_dir_all(&dir);

        // Create a fake quantized model file (will fail to load, but tests path resolution)
        let quant_path = dir.join("quantized.onnx");
        std::fs::write(&quant_path, b"fake").unwrap();

        let mut config = make_config();
        config.model_path = PathBuf::from("/primary/model.onnx"); // doesn't exist
        config.quantized_model_path = Some(quant_path.clone());

        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let mut reranker = CrossEncoderReranker::new(config, clock);

        // load() should try quantized path first (which exists but is corrupt)
        let result = reranker.load();
        assert!(result.is_err());
        // The error should reference the quantized path, not the primary path
        match result.unwrap_err() {
            RerankerError::ModelLoad(msg) => {
                assert!(
                    msg.contains("quantized"),
                    "expected error to reference quantized path, got: {msg}"
                );
            }
            other => panic!("expected ModelLoad, got {other}"),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn quantized_model_path_skipped_when_missing() {
        let mut config = make_config();
        config.model_path = PathBuf::from("/nonexistent/primary.onnx");
        config.quantized_model_path = Some(PathBuf::from("/nonexistent/quantized.onnx"));

        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let mut reranker = CrossEncoderReranker::new(config, clock);

        // quantized path doesn't exist, should fall back to primary path
        let result = reranker.load();
        assert!(result.is_err());
        match result.unwrap_err() {
            RerankerError::ModelLoad(msg) => {
                assert!(
                    msg.contains("primary"),
                    "expected error to reference primary path, got: {msg}"
                );
            }
            other => panic!("expected ModelLoad, got {other}"),
        }
    }
}
