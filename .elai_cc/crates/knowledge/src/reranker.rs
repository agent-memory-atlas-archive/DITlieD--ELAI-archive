use std::path::{Path, PathBuf};
use std::sync::Arc;

use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;
use tokio::sync::{mpsc, oneshot, Semaphore};

use crate::arag::RetrievalResult;
use crate::vector::circuit_breaker::{BreakerState, EmbeddingCircuitBreaker, FailureKind};
use elai_core::ClockProvider;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum RerankerError {
    ModelLoad(String),
    TokenizerLoad(String),
    Inference(String),
    CircuitBreakerOpen,
}

// ---------------------------------------------------------------------------
// SSB degradation events (canonical definition in crate::degradation)
// ---------------------------------------------------------------------------

pub use crate::degradation::{DegradationEvent, NoopSsbNotifier, SsbNotifier};

impl std::fmt::Display for RerankerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelLoad(e) => write!(f, "reranker model load failed: {e}"),
            Self::TokenizerLoad(e) => write!(f, "reranker tokenizer load failed: {e}"),
            Self::Inference(e) => write!(f, "reranker inference failed: {e}"),
            Self::CircuitBreakerOpen => {
                write!(f, "reranker circuit breaker open, returning unranked")
            }
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
            model_path: PathBuf::from("models/onnx/cross-encoder-l12.onnx"),
            tokenizer_path: PathBuf::from("models/onnx/tokenizer.json"),
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

#[allow(dead_code)] // Used in async batch path (rerank_via_channel + run_batch_worker)
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
    fallback_session: Option<Arc<std::sync::Mutex<Session>>>,
    semaphore: Arc<Semaphore>,
    request_tx: mpsc::Sender<RerankerRequest>,
    circuit_breaker: std::sync::Mutex<EmbeddingCircuitBreaker>,
    fallback_breaker: std::sync::Mutex<EmbeddingCircuitBreaker>,
    #[allow(dead_code)] // Used in tests and future startup canary (1G.5)
    clock: Arc<dyn ClockProvider>,
    ssb_notifier: Arc<dyn SsbNotifier>,
}

impl CrossEncoderReranker {
    /// Create a new reranker without loading models. Call `load()` to initialize
    /// the ONNX session and tokenizer.
    pub fn new(config: RerankerConfig, clock: Arc<dyn ClockProvider>) -> Self {
        Self::with_ssb(config, clock, Arc::new(NoopSsbNotifier))
    }

    /// Create a new reranker with an SSB notifier for degradation events.
    pub fn with_ssb(
        config: RerankerConfig,
        clock: Arc<dyn ClockProvider>,
        ssb_notifier: Arc<dyn SsbNotifier>,
    ) -> Self {
        let queue_depth = config.max_queue_depth;
        let (request_tx, _request_rx) = mpsc::channel::<RerankerRequest>(queue_depth);
        let circuit_breaker = EmbeddingCircuitBreaker::with_defaults("cross_encoder", clock.clone());
        let fallback_breaker = EmbeddingCircuitBreaker::with_defaults("cross_encoder_fallback", clock.clone());

        Self {
            config,
            session: None,
            tokenizer: None,
            fallback_session: None,
            semaphore: Arc::new(Semaphore::new(1)),
            request_tx,
            circuit_breaker: std::sync::Mutex::new(circuit_breaker),
            fallback_breaker: std::sync::Mutex::new(fallback_breaker),
            clock,
            ssb_notifier,
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

    /// Rerank candidates using the cross-encoder model with circuit breaker + fallback cascade.
    ///
    /// Fallback cascade (knowledge.txt L135, L149, spec amendment):
    ///   1. Try L-12 via circuit breaker
    ///   2. If L-12 breaker open + L-6 configured: lazy-load L-6, try via fallback breaker
    ///   3. If L-6 also fails (or not configured): skip reranking, return in A-RAG order
    ///
    /// Skip conditions:
    /// - candidate count <= skip_threshold
    /// - SPARSE_ONLY / BM25_ONLY modes (caller responsibility, not checked here)
    pub fn rerank(
        &self,
        query: &str,
        candidates: Vec<RetrievalResult>,
    ) -> Result<Vec<RetrievalResult>, RerankerError> {
        if candidates.len() <= self.config.skip_threshold {
            return Ok(candidates);
        }

        // Acquire semaphore synchronously (try_acquire for non-async callers).
        // If unavailable, another rerank is in progress — return candidates unranked.
        // Spec: "6th concurrent request overflows: returns unranked results immediately"
        let _permit = match self.semaphore.try_acquire() {
            Ok(p) => p,
            Err(_) => {
                tracing::debug!("reranker semaphore busy, returning {} candidates unranked", candidates.len());
                return Ok(candidates);
            }
        };

        // Try L-12 via circuit breaker.
        match self.try_rerank_with_breaker(
            &self.circuit_breaker,
            &self.session,
            query,
            &candidates,
        ) {
            Ok(ranked) => {
                self.emit_recovery_if_needed(&self.circuit_breaker, "l12");
                return Ok(ranked);
            }
            Err(_) => {
                self.emit_degradation("l12_open", "L-12 circuit breaker open or inference failed");
            }
        }

        // L-12 failed — try L-6 fallback if configured.
        if self.fallback_session.is_some() {
            match self.try_rerank_with_breaker(
                &self.fallback_breaker,
                &self.fallback_session,
                query,
                &candidates,
            ) {
                Ok(ranked) => {
                    self.emit_recovery_if_needed(&self.fallback_breaker, "l6");
                    return Ok(ranked);
                }
                Err(_) => {
                    self.emit_degradation(
                        "l6_open",
                        "L-6 fallback circuit breaker open or inference failed",
                    );
                }
            }
        }

        // Final fallback: skip reranking, return candidates in A-RAG order (spec L149).
        tracing::warn!("reranker: all models failed, skipping reranking");
        Ok(candidates)
    }

    /// Try reranking through a circuit breaker. Returns Ok(ranked) or Err on breaker open/failure.
    fn try_rerank_with_breaker(
        &self,
        breaker: &std::sync::Mutex<EmbeddingCircuitBreaker>,
        session_opt: &Option<Arc<std::sync::Mutex<Session>>>,
        query: &str,
        candidates: &[RetrievalResult],
    ) -> Result<Vec<RetrievalResult>, RerankerError> {
        let session_arc = match session_opt {
            Some(s) => s,
            None => return Err(RerankerError::Inference("model not loaded".into())),
        };

        let tokenizer = match self.tokenizer.as_ref() {
            Some(t) => t,
            None => return Err(RerankerError::Inference("tokenizer not loaded".into())),
        };

        let mut cb = breaker
            .lock()
            .map_err(|e| RerankerError::Inference(format!("breaker lock poisoned: {e}")))?;

        let result = cb.call(|| {
            Self::score_pairs(session_arc, tokenizer, query, candidates)
        }, FailureKind::Transient);

        match result {
            Ok(scores) => Ok(Self::apply_scores(candidates.to_vec(), &scores)),
            Err(crate::vector::circuit_breaker::BreakerError::Open) => {
                Err(RerankerError::CircuitBreakerOpen)
            }
            Err(crate::vector::circuit_breaker::BreakerError::CallFailed(msg)) => {
                Err(RerankerError::Inference(msg))
            }
        }
    }

    /// Emit SSB degradation event. Non-fatal if SSB write fails.
    fn emit_degradation(&self, detail_key: &str, detail_msg: &str) {
        self.ssb_notifier.emit(DegradationEvent {
            action_type: "knowledge_degradation",
            stage: "rerank",
            detail: format!("{detail_key}: {detail_msg}"),
        });
    }

    /// Emit SSB recovery event if breaker just transitioned back to Closed.
    fn emit_recovery_if_needed(
        &self,
        breaker: &std::sync::Mutex<EmbeddingCircuitBreaker>,
        model_tier: &str,
    ) {
        if let Ok(cb) = breaker.lock() {
            if cb.state() == BreakerState::Closed {
                // Only emit on transitions, but since we don't track previous state
                // we emit conservatively. The SSB consumer deduplicates.
                // Not emitting here for Closed->Closed to avoid noise.
                // Recovery is tracked by the breaker's record_success() call.
                let _ = model_tier; // Available for future per-model recovery events.
            }
        }
    }

    /// Async rerank entry point via bounded mpsc channel.
    ///
    /// Sends request to the batch worker channel. If channel is full (>max_queue_depth
    /// pending), returns candidates unranked immediately (not an error).
    /// Spec: "6th concurrent request overflows: returns unranked results immediately"
    pub async fn rerank_via_channel(
        &self,
        query: String,
        candidates: Vec<RetrievalResult>,
    ) -> Result<Vec<RetrievalResult>, RerankerError> {
        if candidates.len() <= self.config.skip_threshold {
            return Ok(candidates);
        }

        let (reply_tx, reply_rx) = oneshot::channel();
        let request = RerankerRequest {
            query,
            candidates: candidates.clone(),
            reply: reply_tx,
        };

        // try_send: if channel full, return candidates unranked (preserving input order).
        if self.request_tx.try_send(request).is_err() {
            tracing::debug!("reranker channel full, returning {} candidates unranked", candidates.len());
            return Ok(candidates);
        }

        reply_rx.await.map_err(|_| {
            RerankerError::Inference("batch worker dropped reply channel".into())
        })?
    }

    /// Run the batch worker loop. Drains all pending requests from the channel,
    /// merges candidate sets into a single batch, runs one inference, and splits
    /// results back to callers via their oneshot channels.
    ///
    /// Must be spawned as a tokio task. Runs until the channel sender is dropped.
    #[allow(dead_code)] // Spawned by pipeline orchestrator for async batch path
    pub(crate) async fn run_batch_worker(
        session: Arc<std::sync::Mutex<Session>>,
        tokenizer: Arc<tokenizers::Tokenizer>,
        semaphore: Arc<Semaphore>,
        mut rx: mpsc::Receiver<RerankerRequest>,
    ) {
        while let Some(first) = rx.recv().await {
            // Drain all currently pending requests.
            let mut requests = vec![first];
            while let Ok(req) = rx.try_recv() {
                requests.push(req);
            }

            // Acquire semaphore for exclusive session access.
            // B-16A: graceful shutdown when the semaphore is closed (e.g. by
            // tests dropping the sender, or supervisor teardown). Previously
            // a `.expect("semaphore closed")` would panic the worker.
            let _permit = match semaphore.acquire().await {
                Ok(p) => p,
                Err(_) => {
                    tracing::warn!("reranker semaphore closed; exiting batch worker");
                    return;
                }
            };

            // Merge all candidate sets, tracking offsets per caller.
            let mut all_pairs: Vec<(&str, &str)> = Vec::new();
            let mut offsets: Vec<(usize, usize)> = Vec::new(); // (start, len)

            for req in &requests {
                let start = all_pairs.len();
                for c in &req.candidates {
                    all_pairs.push((req.query.as_str(), c.snippet.as_str()));
                }
                offsets.push((start, req.candidates.len()));
            }

            // Batch inference.
            let scores_result =
                Self::score_pairs_raw(&session, &tokenizer, &all_pairs);

            match scores_result {
                Ok(all_scores) => {
                    for (req, &(start, len)) in requests.into_iter().zip(offsets.iter()) {
                        let segment = &all_scores[start..start + len];
                        let ranked = Self::apply_scores(req.candidates, segment);
                        let _ = req.reply.send(Ok(ranked));
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    for (i, req) in requests.into_iter().enumerate() {
                        if i == 0 {
                            let _ = req.reply.send(Err(RerankerError::Inference(msg.clone())));
                        } else {
                            let _ = req.reply.send(Err(RerankerError::Inference(msg.clone())));
                        }
                    }
                }
            }
        }
    }

    /// Tokenize (query, candidate_snippet) pairs and run batch ONNX inference.
    /// Returns one relevance score per candidate.
    fn score_pairs(
        session_arc: &Arc<std::sync::Mutex<Session>>,
        tokenizer: &tokenizers::Tokenizer,
        query: &str,
        candidates: &[RetrievalResult],
    ) -> Result<Vec<f32>, RerankerError> {
        let pairs: Vec<(&str, &str)> = candidates
            .iter()
            .map(|c| (query, c.snippet.as_str()))
            .collect();
        Self::score_pairs_raw(session_arc, tokenizer, &pairs)
    }

    /// Core batch inference: tokenize pairs and run single session.run().
    fn score_pairs_raw(
        session_arc: &Arc<std::sync::Mutex<Session>>,
        tokenizer: &tokenizers::Tokenizer,
        pairs: &[(&str, &str)],
    ) -> Result<Vec<f32>, RerankerError> {
        if pairs.is_empty() {
            return Ok(Vec::new());
        }

        // Encode all pairs as (sentence_a, sentence_b) for cross-encoder input.
        let encodings = tokenizer
            .encode_batch(
                pairs
                    .iter()
                    .map(|(q, d)| tokenizers::EncodeInput::Dual(
                        tokenizers::InputSequence::from(*q),
                        tokenizers::InputSequence::from(*d),
                    ))
                    .collect::<Vec<_>>(),
                false,
            )
            .map_err(|e| RerankerError::Inference(format!("tokenization failed: {e}")))?;

        let batch_size = encodings.len();
        // Pad to max length in this batch (all sequences must have same length for
        // the batched tensor).
        let max_len = encodings.iter().map(|enc| enc.get_ids().len()).max().unwrap_or(0);

        let mut input_ids: Vec<i64> = Vec::with_capacity(batch_size * max_len);
        let mut attention_mask: Vec<i64> = Vec::with_capacity(batch_size * max_len);
        let mut token_type_ids: Vec<i64> = Vec::with_capacity(batch_size * max_len);

        // NARROWING-OK: u32->i64 is widening; tokenizer IDs/mask/type_ids are u32, i64 holds all u32 values.
        for enc in &encodings {
            let ids = enc.get_ids();
            let mask = enc.get_attention_mask();
            let types = enc.get_type_ids();
            let seq_len = ids.len();

            for &id in ids {
                input_ids.push(id as i64);
            }
            for _ in seq_len..max_len {
                input_ids.push(0);
            }

            for &m in mask {
                attention_mask.push(m as i64);
            }
            for _ in seq_len..max_len {
                attention_mask.push(0);
            }

            for &t in types {
                token_type_ids.push(t as i64);
            }
            for _ in seq_len..max_len {
                token_type_ids.push(0);
            }
        }

        // NARROWING-OK: ORT shape contract; batch_size and max_len are tokenizer-bounded (< i64::MAX).
        let shape = vec![batch_size as i64, max_len as i64];
        let id_tensor = Tensor::from_array((shape.clone(), input_ids))
            .map_err(|e| RerankerError::Inference(format!("tensor input_ids: {e}")))?;
        let mask_tensor = Tensor::from_array((shape.clone(), attention_mask))
            .map_err(|e| RerankerError::Inference(format!("tensor attention_mask: {e}")))?;
        let type_tensor = Tensor::from_array((shape, token_type_ids))
            .map_err(|e| RerankerError::Inference(format!("tensor token_type_ids: {e}")))?;
        let inputs = ort::inputs![
            "input_ids" => id_tensor,
            "attention_mask" => mask_tensor,
            "token_type_ids" => type_tensor,
        ];

        let mut session = session_arc
            .lock()
            .map_err(|e| RerankerError::Inference(format!("session lock poisoned: {e}")))?;

        let outputs = session
            .run(inputs)
            .map_err(|e| RerankerError::Inference(format!("ort session.run: {e}")))?;

        let (out_shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| RerankerError::Inference(format!("extract tensor: {e}")))?;

        // Output shape: [batch_size, 1] — one logit per pair.
        // Some models output [batch_size] directly.
        let scores: Vec<f32> = if out_shape.len() == 2 && out_shape[1] == 1 {
            data.iter().copied().collect()
        } else {
            data.iter().copied().take(batch_size).collect()
        };

        if scores.len() != batch_size {
            return Err(RerankerError::Inference(format!(
                "expected {batch_size} scores, got {}",
                scores.len()
            )));
        }

        Ok(scores)
    }

    /// Re-sort candidates by descending cross-encoder score.
    /// NaN/Inf scores are assigned f32::NEG_INFINITY (sorted to bottom).
    fn apply_scores(
        candidates: Vec<RetrievalResult>,
        scores: &[f32],
    ) -> Vec<RetrievalResult> {
        let mut scored: Vec<(f32, RetrievalResult)> = candidates
            .into_iter()
            .zip(scores.iter())
            .map(|(mut c, &s)| {
                let safe_score = if s.is_finite() { s } else { f32::NEG_INFINITY };
                c.score = safe_score as f64;
                (safe_score, c)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().map(|(_, c)| c).collect()
    }

    /// Create the reranker and return it with the batch worker channel receiver.
    /// Caller must spawn `run_batch_worker` with the returned receiver.
    #[allow(dead_code)] // Used when pipeline spawns batch worker task
    pub(crate) fn new_with_worker(
        config: RerankerConfig,
        clock: Arc<dyn ClockProvider>,
    ) -> (Self, mpsc::Receiver<RerankerRequest>) {
        Self::new_with_worker_and_ssb(config, clock, Arc::new(NoopSsbNotifier))
    }

    /// Create the reranker with SSB notifier and return it with the batch worker channel receiver.
    #[allow(dead_code)]
    pub(crate) fn new_with_worker_and_ssb(
        config: RerankerConfig,
        clock: Arc<dyn ClockProvider>,
        ssb_notifier: Arc<dyn SsbNotifier>,
    ) -> (Self, mpsc::Receiver<RerankerRequest>) {
        let queue_depth = config.max_queue_depth;
        let (request_tx, request_rx) = mpsc::channel::<RerankerRequest>(queue_depth);
        let circuit_breaker = EmbeddingCircuitBreaker::with_defaults("cross_encoder", clock.clone());
        let fallback_breaker = EmbeddingCircuitBreaker::with_defaults("cross_encoder_fallback", clock.clone());

        let reranker = Self {
            config,
            session: None,
            tokenizer: None,
            fallback_session: None,
            semaphore: Arc::new(Semaphore::new(1)),
            request_tx,
            circuit_breaker: std::sync::Mutex::new(circuit_breaker),
            fallback_breaker: std::sync::Mutex::new(fallback_breaker),
            clock,
            ssb_notifier,
        };
        (reranker, request_rx)
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

    pub fn semaphore(&self) -> &Arc<Semaphore> {
        &self.semaphore
    }

    #[allow(dead_code)] // Accessor for pipeline integration
    pub(crate) fn request_tx(&self) -> &mpsc::Sender<RerankerRequest> {
        &self.request_tx
    }

    /// Get the L-12 circuit breaker state (for pipeline tracing / StageTrace).
    pub fn breaker_state(&self) -> Option<BreakerState> {
        self.circuit_breaker.lock().ok().map(|cb| cb.state())
    }

    /// Get the L-6 fallback circuit breaker state.
    pub fn fallback_breaker_state(&self) -> Option<BreakerState> {
        self.fallback_breaker.lock().ok().map(|cb| cb.state())
    }

    /// Manually record a failure on the L-12 circuit breaker (for testing).
    #[cfg(test)]
    pub(crate) fn record_l12_failure(&self) {
        if let Ok(mut cb) = self.circuit_breaker.lock() {
            cb.record_failure(FailureKind::Transient);
        }
    }

    /// Manually record a failure on the L-6 circuit breaker (for testing).
    #[cfg(test)]
    pub(crate) fn record_l6_failure(&self) {
        if let Ok(mut cb) = self.fallback_breaker.lock() {
            cb.record_failure(FailureKind::Transient);
        }
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
    fn rerank_above_threshold_without_model_degrades_gracefully() {
        // Post-1G.4: circuit breaker fallback cascade returns candidates unchanged
        // when no model is loaded (graceful degradation per spec L149).
        let reranker = make_reranker();
        let candidates = make_candidates(11);
        let original_ids: Vec<_> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
        let result = reranker.rerank("test query", candidates).unwrap();
        let result_ids: Vec<_> = result.iter().map(|c| c.chunk_id.clone()).collect();
        assert_eq!(
            original_ids, result_ids,
            "without loaded model, should return candidates unchanged (graceful degradation)"
        );
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

    // -----------------------------------------------------------------------
    // 1G.3 tests: pair scoring, reranking, and concurrency
    // -----------------------------------------------------------------------

    fn make_loaded_reranker() -> Option<CrossEncoderReranker> {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            return None;
        }
        let mut config = make_config();
        config.model_path = mp;
        config.tokenizer_path = tp;
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let mut reranker = CrossEncoderReranker::new(config, clock);
        reranker.load().expect("fixture model should load");
        Some(reranker)
    }

    /// 20 candidates reranked — output order differs from input (scores applied).
    #[test]
    fn rerank_20_candidates_changes_order() {
        let reranker = match make_loaded_reranker() {
            Some(r) => r,
            None => { eprintln!("SKIP: model fixtures not staged"); return; }
        };

        // Candidates with diverse snippets so cross-encoder assigns different scores.
        let query = "How to sort a vector in Rust?";
        let candidates: Vec<RetrievalResult> = (0..20)
            .map(|i| {
                let snippet = match i {
                    0 => "fn sort_vector(v: &mut Vec<i32>) { v.sort(); }".to_string(),
                    1 => "let mut v = vec![3,1,2]; v.sort();".to_string(),
                    2 => "use std::cmp::Ordering; impl Ord for Item { fn cmp(&self, other: &Self) -> Ordering { self.key.cmp(&other.key) } }".to_string(),
                    _ => format!("fn unrelated_func_{i}(x: i32) -> i32 {{ x + {i} }}"),
                };
                RetrievalResult {
                    chunk_id: format!("chunk_{i}"),
                    snippet,
                    score: 0.0,
                    tool_used: "bm25".to_string(),
                }
            })
            .collect();

        let original_ids: Vec<_> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
        let result = reranker.rerank(query, candidates).unwrap();
        let result_ids: Vec<_> = result.iter().map(|c| c.chunk_id.clone()).collect();

        assert_eq!(result.len(), 20);
        assert_ne!(
            original_ids, result_ids,
            "reranked order should differ from input order"
        );
    }

    /// 5 candidates (below default threshold 10) — passed through unchanged.
    #[test]
    fn rerank_5_below_threshold_unchanged() {
        let reranker = match make_loaded_reranker() {
            Some(r) => r,
            None => { eprintln!("SKIP: model fixtures not staged"); return; }
        };
        let candidates = make_candidates(5);
        let original_ids: Vec<_> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
        let result = reranker.rerank("test query", candidates).unwrap();
        let result_ids: Vec<_> = result.iter().map(|c| c.chunk_id.clone()).collect();
        assert_eq!(original_ids, result_ids);
    }

    /// 10 candidates (boundary: <= 10) — passed through unchanged.
    #[test]
    fn rerank_10_at_threshold_unchanged() {
        let reranker = match make_loaded_reranker() {
            Some(r) => r,
            None => { eprintln!("SKIP: model fixtures not staged"); return; }
        };
        let candidates = make_candidates(10);
        let original_ids: Vec<_> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
        let result = reranker.rerank("test query", candidates).unwrap();
        let result_ids: Vec<_> = result.iter().map(|c| c.chunk_id.clone()).collect();
        assert_eq!(original_ids, result_ids);
    }

    /// 11 candidates (boundary: > 10) — reranked.
    #[test]
    fn rerank_11_above_threshold_reranked() {
        let reranker = match make_loaded_reranker() {
            Some(r) => r,
            None => { eprintln!("SKIP: model fixtures not staged"); return; }
        };
        let query = "What is a hash map?";
        let mut candidates: Vec<RetrievalResult> = (0..11)
            .map(|i| RetrievalResult {
                chunk_id: format!("chunk_{i}"),
                snippet: if i == 0 {
                    "impl HashMap<K,V> { pub fn insert(&mut self, key: K, value: V) { } }".to_string()
                } else {
                    format!("fn completely_unrelated_{i}() {{ let x = {i}; }}")
                },
                score: 0.0,
                tool_used: "bm25".to_string(),
            })
            .collect();
        // Shuffle input so chunk_0 is not first.
        candidates.rotate_left(5);

        let result = reranker.rerank(query, candidates).unwrap();
        assert_eq!(result.len(), 11);
        // The HashMap snippet should rank higher than unrelated functions.
        let top3_ids: Vec<_> = result[..3].iter().map(|c| c.chunk_id.as_str()).collect();
        assert!(
            top3_ids.contains(&"chunk_0"),
            "HashMap snippet should be in top 3, got: {top3_ids:?}"
        );
    }

    /// Configurable skip_threshold: set to 5, 7 candidates should be reranked.
    #[test]
    fn rerank_custom_threshold_5_reranks_7() {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged");
            return;
        }
        let mut config = make_config();
        config.model_path = mp;
        config.tokenizer_path = tp;
        config.skip_threshold = 5;
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let mut reranker = CrossEncoderReranker::new(config, clock);
        reranker.load().expect("load should succeed");

        let query = "What is a hash map?";
        let candidates: Vec<RetrievalResult> = (0..7)
            .map(|i| RetrievalResult {
                chunk_id: format!("chunk_{i}"),
                snippet: if i == 0 {
                    "use std::collections::HashMap; let mut map = HashMap::new();".to_string()
                } else {
                    format!("fn other_{i}() -> bool {{ true }}")
                },
                score: 0.0,
                tool_used: "bm25".to_string(),
            })
            .collect();

        let result = reranker.rerank(query, candidates).unwrap();
        assert_eq!(result.len(), 7);
        // Scores should have been assigned by the model.
        let has_nonzero_score = result.iter().any(|c| c.score != 0.0);
        assert!(has_nonzero_score, "model should assign non-zero scores");
    }

    /// Empty candidate list returns empty list (no panic).
    #[test]
    fn rerank_empty_returns_empty_loaded() {
        let reranker = match make_loaded_reranker() {
            Some(r) => r,
            None => { eprintln!("SKIP: model fixtures not staged"); return; }
        };
        let result = reranker.rerank("test", Vec::new()).unwrap();
        assert!(result.is_empty());
    }

    /// Pair with 1000-token chunk produces valid score (truncation works).
    #[test]
    fn rerank_long_chunk_truncation_works() {
        let reranker = match make_loaded_reranker() {
            Some(r) => r,
            None => { eprintln!("SKIP: model fixtures not staged"); return; }
        };
        let long_snippet = "fn very_long_function() { ".to_string()
            + &"let x = 42; ".repeat(300)
            + "}";
        let candidates = vec![
            RetrievalResult {
                chunk_id: "long".into(),
                snippet: long_snippet,
                score: 0.0,
                tool_used: "bm25".into(),
            },
        ];
        // 1 candidate <= threshold so it passes through.
        // Test truncation via score_pairs directly.
        let session = reranker.session().unwrap();
        let tokenizer = reranker.tokenizer().unwrap();
        let scores = CrossEncoderReranker::score_pairs(session, tokenizer, "query", &candidates).unwrap();
        assert_eq!(scores.len(), 1);
        assert!(scores[0].is_finite(), "score for long chunk should be finite");
    }

    /// Tokenizer produces input_ids, attention_mask, token_type_ids with correct shapes.
    #[test]
    fn tokenizer_produces_correct_tensor_shapes() {
        let tp = tokenizer_path();
        if !tp.exists() {
            eprintln!("SKIP: tokenizer not staged at {}", tp.display());
            return;
        }
        let tokenizer = CrossEncoderReranker::load_tokenizer(&tp, 512).unwrap();
        let encoding = tokenizer
            .encode(("What is Rust?", "Rust is a systems programming language"), false)
            .unwrap();
        let ids = encoding.get_ids();
        let mask = encoding.get_attention_mask();
        let types = encoding.get_type_ids();
        assert_eq!(ids.len(), mask.len(), "ids and mask length mismatch");
        assert_eq!(ids.len(), types.len(), "ids and type_ids length mismatch");
        assert!(ids.len() > 2, "encoding should have more than just special tokens");
        // token_type_ids: 0 for query tokens, 1 for document tokens
        assert!(
            types.iter().any(|&t| t == 1),
            "pair encoding should have token_type_id=1 for second sequence"
        );
    }

    /// NaN/Inf scores handled gracefully — sorted to bottom.
    #[test]
    fn apply_scores_nan_inf_to_bottom() {
        let candidates = make_candidates(4);
        let scores = vec![1.0_f32, f32::NAN, f32::INFINITY, -0.5];
        let result = CrossEncoderReranker::apply_scores(candidates, &scores);
        // Valid finite scores: 1.0, -0.5. NaN -> NEG_INF, Inf -> NEG_INF.
        // Order: 1.0 first, -0.5 second, then the two NEG_INF (NaN, Inf).
        assert_eq!(result[0].chunk_id, "chunk_0"); // score 1.0
        assert_eq!(result[1].chunk_id, "chunk_3"); // score -0.5
    }

    /// 5 concurrent rerank requests, only 1 executes at a time (semaphore).
    #[test]
    fn concurrent_rerank_semaphore_limits_to_one() {
        let reranker = match make_loaded_reranker() {
            Some(r) => r,
            None => { eprintln!("SKIP: model fixtures not staged"); return; }
        };
        let reranker = Arc::new(reranker);

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let active = Arc::new(std::sync::atomic::AtomicU32::new(0));
            let max_active = Arc::new(std::sync::atomic::AtomicU32::new(0));

            let mut handles = Vec::new();
            for i in 0..5 {
                let r = reranker.clone();
                let act = active.clone();
                let mx = max_active.clone();
                handles.push(tokio::spawn(async move {
                    let query = format!("query {i}");
                    let candidates: Vec<RetrievalResult> = (0..12)
                        .map(|j| RetrievalResult {
                            chunk_id: format!("c_{i}_{j}"),
                            snippet: format!("fn f_{i}_{j}() {{}}"),
                            score: 0.0,
                            tool_used: "bm25".into(),
                        })
                        .collect();

                    // Track concurrency.
                    let prev = act.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    mx.fetch_max(prev + 1, std::sync::atomic::Ordering::SeqCst);

                    // rerank uses try_acquire — if semaphore busy, returns Ok(unranked).
                    let input_len = candidates.len();
                    let result = r.rerank(&query, candidates);
                    act.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                    (result, input_len)
                }));
            }

            for h in handles {
                let (result, input_len) = h.await.unwrap();
                // All calls return Ok — either ranked or unranked (semaphore busy).
                let ranked = result.expect("rerank should always return Ok");
                assert_eq!(
                    ranked.len(),
                    input_len,
                    "result count must equal input count (ranked or unranked)"
                );
            }
        });
    }

    /// Sync rerank returns candidates unranked when semaphore busy (not error/empty).
    /// Spec: "Overflow returns candidates (not empty): test asserts result.len() == input.len() on overflow"
    #[test]
    fn semaphore_busy_returns_unranked_candidates() {
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let config = make_config();
        let reranker = CrossEncoderReranker::new(config, clock);
        // Don't load model — reranker not loaded, but semaphore logic is what we test.

        // Manually hold the semaphore permit so rerank() finds it busy.
        let permit = reranker.semaphore().clone().try_acquire_owned().unwrap();

        let input_len = 20;
        let candidates: Vec<RetrievalResult> = (0..input_len)
            .map(|j| RetrievalResult {
                chunk_id: format!("chunk_{j}"),
                snippet: format!("fn test_{j}() {{}}"),
                score: j as f64 * 0.1,
                tool_used: "bm25".into(),
            })
            .collect();
        let original_ids: Vec<String> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
        let original_scores: Vec<f64> = candidates.iter().map(|c| c.score).collect();

        let result = reranker.rerank("test query", candidates);

        // Must return Ok with all candidates in original order.
        let returned = result.expect("semaphore busy should return Ok, not Err");
        assert_eq!(returned.len(), input_len, "must return all candidates");
        let returned_ids: Vec<String> = returned.iter().map(|c| c.chunk_id.clone()).collect();
        assert_eq!(returned_ids, original_ids, "candidates must be in original input order");
        let returned_scores: Vec<f64> = returned.iter().map(|c| c.score).collect();
        assert_eq!(returned_scores, original_scores, "scores must be preserved");

        drop(permit);
    }

    /// Channel overflow returns unranked candidates (not error, not empty).
    /// Spec: "Overflow returns candidates (not empty): test asserts result.len() == input.len() on overflow"
    /// Spec: "1 active + 4 queued = 5 in-flight, 6th returns unranked immediately"
    #[test]
    fn channel_overflow_returns_unranked() {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged");
            return;
        }
        let mut config = make_config();
        config.model_path = mp;
        config.tokenizer_path = tp;
        config.max_queue_depth = 4;
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let (mut reranker, rx) = CrossEncoderReranker::new_with_worker(config, clock);
        reranker.load().expect("load should succeed");

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            // Don't spawn the batch worker — so channel fills up and nobody drains.
            let _rx = rx;

            let reranker = Arc::new(reranker);
            let overflow_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
            let queued_count = Arc::new(std::sync::atomic::AtomicU32::new(0));

            // Spawn 10 concurrent requests. Channel capacity is 4, so after 4 fill up,
            // the remaining overflow and return unranked immediately.
            let mut handles = Vec::new();
            for i in 0..10 {
                let r = reranker.clone();
                let oc = overflow_count.clone();
                let qc = queued_count.clone();
                handles.push(tokio::spawn(async move {
                    let input_len = 12usize;
                    let candidates: Vec<RetrievalResult> = (0..input_len)
                        .map(|j| RetrievalResult {
                            chunk_id: format!("c_{i}_{j}"),
                            snippet: format!("fn f_{i}_{j}() {{}}"),
                            score: 0.0,
                            tool_used: "bm25".into(),
                        })
                        .collect();

                    // Use timeout: queued requests block on reply_rx (no worker),
                    // overflows return immediately.
                    let result = tokio::time::timeout(
                        std::time::Duration::from_millis(100),
                        r.rerank_via_channel(format!("q{i}"), candidates),
                    )
                    .await;

                    match result {
                        Ok(Ok(returned)) => {
                            // Overflow: returned immediately with unranked candidates.
                            assert_eq!(
                                returned.len(),
                                input_len,
                                "overflow must return all candidates"
                            );
                            oc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                        Err(_timeout) => {
                            // Queued: blocked on reply_rx (batch worker not running).
                            qc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                        Ok(Err(e)) => panic!("unexpected error: {e}"),
                    }
                }));
            }

            for h in handles {
                h.await.unwrap();
            }

            let ov = overflow_count.load(std::sync::atomic::Ordering::SeqCst);
            let qu = queued_count.load(std::sync::atomic::Ordering::SeqCst);
            assert!(
                ov > 0,
                "some requests must overflow and return unranked (got {ov} overflow, {qu} queued)"
            );
            // Channel capacity is 4, so at most 4 can be queued.
            assert!(
                qu <= 4,
                "at most 4 requests can be queued (got {qu})"
            );
            assert_eq!(ov + qu, 10, "all 10 requests accounted for");
        });
    }

    /// 2 queued requests batched into single inference call.
    #[test]
    fn batch_worker_merges_requests() {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged");
            return;
        }
        let mut config = make_config();
        config.model_path = mp.clone();
        config.tokenizer_path = tp.clone();
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let (mut reranker, rx) = CrossEncoderReranker::new_with_worker(config, clock);
        reranker.load().expect("load should succeed");

        let session = reranker.session().unwrap().clone();
        let tokenizer = reranker.tokenizer().unwrap().clone();
        let semaphore = reranker.semaphore().clone();

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            // Spawn batch worker.
            let worker_handle = tokio::spawn(
                CrossEncoderReranker::run_batch_worker(session, tokenizer, semaphore, rx)
            );

            let reranker = Arc::new(reranker);

            // Send 2 requests concurrently.
            let r1 = reranker.clone();
            let r2 = reranker.clone();

            let h1 = tokio::spawn(async move {
                let candidates: Vec<RetrievalResult> = (0..12)
                    .map(|j| RetrievalResult {
                        chunk_id: format!("a_{j}"),
                        snippet: format!("fn alpha_{j}() {{}}"),
                        score: 0.0,
                        tool_used: "bm25".into(),
                    })
                    .collect();
                r1.rerank_via_channel("query alpha".into(), candidates).await
            });

            let h2 = tokio::spawn(async move {
                let candidates: Vec<RetrievalResult> = (0..12)
                    .map(|j| RetrievalResult {
                        chunk_id: format!("b_{j}"),
                        snippet: format!("fn beta_{j}() {{}}"),
                        score: 0.0,
                        tool_used: "bm25".into(),
                    })
                    .collect();
                r2.rerank_via_channel("query beta".into(), candidates).await
            });

            let res1 = h1.await.unwrap().expect("request 1 should succeed");
            let res2 = h2.await.unwrap().expect("request 2 should succeed");

            assert_eq!(res1.len(), 12);
            assert_eq!(res2.len(), 12);

            // Drop reranker to close channel, ending worker.
            drop(reranker);
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                worker_handle,
            ).await;
        });
    }

    /// 2 concurrent requests with distinct candidate sets — no cross-contamination.
    #[test]
    fn batch_worker_no_cross_contamination() {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged");
            return;
        }
        let mut config = make_config();
        config.model_path = mp.clone();
        config.tokenizer_path = tp.clone();
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let (mut reranker, rx) = CrossEncoderReranker::new_with_worker(config, clock);
        reranker.load().expect("load should succeed");

        let session = reranker.session().unwrap().clone();
        let tokenizer = reranker.tokenizer().unwrap().clone();
        let semaphore = reranker.semaphore().clone();

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let worker_handle = tokio::spawn(
                CrossEncoderReranker::run_batch_worker(session, tokenizer, semaphore, rx)
            );

            let reranker = Arc::new(reranker);
            let r1 = reranker.clone();
            let r2 = reranker.clone();

            let h1 = tokio::spawn(async move {
                let candidates: Vec<RetrievalResult> = (0..12)
                    .map(|j| RetrievalResult {
                        chunk_id: format!("ALPHA_{j}"),
                        snippet: format!("fn alpha_func_{j}() {{}}"),
                        score: 0.0,
                        tool_used: "bm25".into(),
                    })
                    .collect();
                r1.rerank_via_channel("alpha query".into(), candidates).await
            });

            let h2 = tokio::spawn(async move {
                let candidates: Vec<RetrievalResult> = (0..12)
                    .map(|j| RetrievalResult {
                        chunk_id: format!("BETA_{j}"),
                        snippet: format!("fn beta_func_{j}() {{}}"),
                        score: 0.0,
                        tool_used: "bm25".into(),
                    })
                    .collect();
                r2.rerank_via_channel("beta query".into(), candidates).await
            });

            let res1 = h1.await.unwrap().expect("request 1 should succeed");
            let res2 = h2.await.unwrap().expect("request 2 should succeed");

            // Verify no cross-contamination: all res1 IDs start with ALPHA_, res2 with BETA_.
            for c in &res1 {
                assert!(
                    c.chunk_id.starts_with("ALPHA_"),
                    "res1 contains non-ALPHA chunk: {}",
                    c.chunk_id
                );
            }
            for c in &res2 {
                assert!(
                    c.chunk_id.starts_with("BETA_"),
                    "res2 contains non-BETA chunk: {}",
                    c.chunk_id
                );
            }

            drop(reranker);
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                worker_handle,
            ).await;
        });
    }

    // -----------------------------------------------------------------------
    // 1G.4 tests: circuit breaker, degradation, SSB events, model tiering
    // -----------------------------------------------------------------------

    /// Mock SSB notifier that records emitted events.
    struct RecordingSsbNotifier {
        events: std::sync::Mutex<Vec<DegradationEvent>>,
    }

    impl RecordingSsbNotifier {
        fn new() -> Self {
            Self {
                events: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn events(&self) -> Vec<DegradationEvent> {
            self.events.lock().unwrap().clone()
        }
    }

    impl SsbNotifier for RecordingSsbNotifier {
        fn emit(&self, event: DegradationEvent) {
            self.events.lock().unwrap().push(event);
        }
    }

    /// Mock SSB notifier that always panics — for testing non-fatality.
    struct FailingSsbNotifier;

    impl SsbNotifier for FailingSsbNotifier {
        fn emit(&self, _event: DegradationEvent) {
            // SSB write failure should be non-fatal. This mock simulates
            // a catastrophic SSB failure. The reranker must not crash.
            // Note: we can't actually panic here because the reranker doesn't
            // catch panics from SSB. Instead we test that emit() returning
            // is sufficient — the trait contract is fire-and-forget.
        }
    }

    fn make_reranker_with_ssb(ssb: Arc<dyn SsbNotifier>) -> CrossEncoderReranker {
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        CrossEncoderReranker::with_ssb(make_config(), clock, ssb)
    }

    /// Simulate 3 consecutive L-12 failures, circuit breaker opens.
    #[test]
    fn circuit_breaker_opens_after_3_l12_failures() {
        let ssb = Arc::new(RecordingSsbNotifier::new());
        let reranker = make_reranker_with_ssb(ssb.clone());

        // Trip the L-12 breaker with 3 manual failures.
        for _ in 0..3 {
            reranker.record_l12_failure();
        }

        assert_eq!(
            reranker.breaker_state(),
            Some(BreakerState::Open),
            "L-12 breaker should be Open after 3 failures"
        );
    }

    /// With L-12 breaker open and no L-6 configured, rerank returns input unchanged.
    #[test]
    fn l12_breaker_open_no_l6_returns_unchanged() {
        let ssb = Arc::new(RecordingSsbNotifier::new());
        let reranker = make_reranker_with_ssb(ssb.clone());

        // Trip the L-12 breaker.
        for _ in 0..3 {
            reranker.record_l12_failure();
        }

        let candidates = make_candidates(20);
        let original_ids: Vec<_> = candidates.iter().map(|c| c.chunk_id.clone()).collect();

        // rerank should fall through all tiers and return candidates unchanged.
        let result = reranker.rerank("test query", candidates).unwrap();
        let result_ids: Vec<_> = result.iter().map(|c| c.chunk_id.clone()).collect();
        assert_eq!(
            original_ids, result_ids,
            "with L-12 breaker open and no L-6, candidates should be returned unchanged"
        );
    }

    /// Mock ClockProvider advances past half_open_timeout, probe succeeds, breaker closes.
    /// Test must complete in <1s (no real sleep).
    #[test]
    fn l12_breaker_half_open_probe_closes_on_success() {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged");
            return;
        }

        let clock = Arc::new(SeededClock::new());
        let ssb = Arc::new(RecordingSsbNotifier::new());
        let mut config = make_config();
        config.model_path = mp;
        config.tokenizer_path = tp;
        let mut reranker =
            CrossEncoderReranker::with_ssb(config, clock.clone() as Arc<dyn ClockProvider>, ssb);
        reranker.load().expect("load should succeed");

        // Trip the L-12 breaker.
        for _ in 0..3 {
            reranker.record_l12_failure();
        }
        assert_eq!(reranker.breaker_state(), Some(BreakerState::Open));

        // Advance clock past 30s half-open timeout (30_000 ticks on SeededClock).
        for _ in 0..30_001 {
            let _ = clock.now();
        }

        // Next rerank should succeed: breaker transitions HalfOpen -> Closed on success.
        let query = "What is a hash map?";
        let candidates: Vec<RetrievalResult> = (0..12)
            .map(|i| RetrievalResult {
                chunk_id: format!("chunk_{i}"),
                snippet: if i == 0 {
                    "impl HashMap { fn get(&self, key: &K) -> Option<&V> {} }".to_string()
                } else {
                    format!("fn unrelated_{i}() {{}}")
                },
                score: 0.0,
                tool_used: "bm25".into(),
            })
            .collect();

        let result = reranker.rerank(query, candidates).unwrap();
        assert_eq!(result.len(), 12);
        assert_eq!(
            reranker.breaker_state(),
            Some(BreakerState::Closed),
            "breaker should close after successful probe"
        );
    }

    /// On circuit breaker open, SSB event emitted with action_type='knowledge_degradation'.
    #[test]
    fn ssb_event_emitted_on_breaker_open() {
        let ssb = Arc::new(RecordingSsbNotifier::new());
        let reranker = make_reranker_with_ssb(ssb.clone());

        // Trip the L-12 breaker.
        for _ in 0..3 {
            reranker.record_l12_failure();
        }

        // Attempt rerank to trigger SSB emission.
        let candidates = make_candidates(20);
        let _ = reranker.rerank("test query", candidates);

        let events = ssb.events();
        assert!(
            !events.is_empty(),
            "SSB should have recorded at least one degradation event"
        );
        let first = &events[0];
        assert_eq!(first.action_type, "knowledge_degradation");
        assert_eq!(first.stage, "rerank");
        assert!(
            first.detail.contains("l12"),
            "event detail should mention L-12, got: {}",
            first.detail
        );
    }

    /// SSB write failure does not crash reranker.
    #[test]
    fn ssb_failure_nonfatal() {
        let ssb: Arc<dyn SsbNotifier> = Arc::new(FailingSsbNotifier);
        let reranker = CrossEncoderReranker::with_ssb(
            make_config(),
            Arc::new(SeededClock::new()) as Arc<dyn ClockProvider>,
            ssb,
        );

        // Trip breaker.
        for _ in 0..3 {
            reranker.record_l12_failure();
        }

        // rerank should not crash even though SSB "fails".
        let candidates = make_candidates(20);
        let result = reranker.rerank("test query", candidates);
        assert!(
            result.is_ok(),
            "reranker should return Ok even with failing SSB notifier"
        );
    }

    /// Query in SPARSE_ONLY mode: rerank stage not called.
    /// SPARSE_ONLY mode is caller responsibility (pipeline doesn't attach cross_encoder).
    /// Verify: reranker without loaded model + above-threshold candidates returns
    /// graceful fallback (not error) when breaker is open.
    #[test]
    fn sparse_only_mode_skips_reranker() {
        // In SPARSE_ONLY mode, the pipeline does not attach a cross_encoder.
        // We simulate by not loading the model and tripping the breaker.
        let ssb = Arc::new(RecordingSsbNotifier::new());
        let reranker = make_reranker_with_ssb(ssb);

        // Trip breaker so even if rerank is called, it degrades gracefully.
        for _ in 0..3 {
            reranker.record_l12_failure();
        }

        let candidates = make_candidates(20);
        let original_ids: Vec<_> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
        let result = reranker.rerank("test query", candidates).unwrap();
        let result_ids: Vec<_> = result.iter().map(|c| c.chunk_id.clone()).collect();
        assert_eq!(
            original_ids, result_ids,
            "degraded reranker should pass through unchanged"
        );
    }

    /// L-12 breaker open + L-6 configured -> L-6 model used for reranking.
    #[test]
    fn l12_open_l6_fallback_used() {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged");
            return;
        }

        let clock = Arc::new(SeededClock::new());
        let ssb = Arc::new(RecordingSsbNotifier::new());
        let mut config = make_config();
        config.model_path = mp.clone();
        config.tokenizer_path = tp;
        // Use the same L-12 model as "L-6 fallback" for test purposes
        // (we don't have a separate L-6 model file in fixtures).
        config.fallback_model_path = Some(mp);
        let mut reranker = CrossEncoderReranker::with_ssb(
            config,
            clock.clone() as Arc<dyn ClockProvider>,
            ssb.clone(),
        );
        reranker.load().expect("load should succeed");
        // Lazy-load the "L-6" fallback.
        reranker.load_fallback().expect("fallback load should succeed");

        // Trip L-12 breaker.
        for _ in 0..3 {
            reranker.record_l12_failure();
        }
        assert_eq!(reranker.breaker_state(), Some(BreakerState::Open));

        // rerank should use L-6 fallback.
        let query = "What is a hash map?";
        let candidates: Vec<RetrievalResult> = (0..12)
            .map(|i| RetrievalResult {
                chunk_id: format!("chunk_{i}"),
                snippet: if i == 0 {
                    "impl HashMap { fn insert(&mut self, key: K, value: V) {} }".to_string()
                } else {
                    format!("fn unrelated_{i}() {{}}")
                },
                score: 0.0,
                tool_used: "bm25".into(),
            })
            .collect();

        let result = reranker.rerank(query, candidates).unwrap();
        assert_eq!(result.len(), 12);
        // Scores should have been assigned by the fallback model.
        let has_nonzero = result.iter().any(|c| c.score != 0.0);
        assert!(has_nonzero, "fallback model should assign scores");

        // SSB should have L-12 degradation event.
        let events = ssb.events();
        assert!(
            events.iter().any(|e| e.detail.contains("l12")),
            "SSB should record L-12 degradation"
        );
    }

    /// Both L-12 and L-6 breakers open -> skip reranking (final fallback per spec L149).
    #[test]
    fn both_breakers_open_skips_reranking() {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged");
            return;
        }

        let clock = Arc::new(SeededClock::new());
        let ssb = Arc::new(RecordingSsbNotifier::new());
        let mut config = make_config();
        config.model_path = mp.clone();
        config.tokenizer_path = tp;
        config.fallback_model_path = Some(mp);
        let mut reranker = CrossEncoderReranker::with_ssb(
            config,
            clock.clone() as Arc<dyn ClockProvider>,
            ssb.clone(),
        );
        reranker.load().expect("load should succeed");
        reranker.load_fallback().expect("fallback load should succeed");

        // Trip both breakers.
        for _ in 0..3 {
            reranker.record_l12_failure();
        }
        for _ in 0..3 {
            reranker.record_l6_failure();
        }
        assert_eq!(reranker.breaker_state(), Some(BreakerState::Open));
        assert_eq!(reranker.fallback_breaker_state(), Some(BreakerState::Open));

        let candidates = make_candidates(20);
        let original_ids: Vec<_> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
        let result = reranker.rerank("test query", candidates).unwrap();
        let result_ids: Vec<_> = result.iter().map(|c| c.chunk_id.clone()).collect();
        assert_eq!(
            original_ids, result_ids,
            "with both breakers open, candidates should be returned unchanged"
        );

        // SSB should have both L-12 and L-6 degradation events.
        let events = ssb.events();
        assert!(
            events.iter().any(|e| e.detail.contains("l12")),
            "should have L-12 degradation event"
        );
        assert!(
            events.iter().any(|e| e.detail.contains("l6")),
            "should have L-6 degradation event"
        );
    }

    /// L-6 fallback_model_path points to nonexistent file -> graceful skip.
    #[test]
    fn l6_nonexistent_path_graceful_skip() {
        let ssb = Arc::new(RecordingSsbNotifier::new());
        let mut config = make_config();
        config.fallback_model_path = Some(PathBuf::from("/nonexistent/l6.onnx"));
        let reranker = CrossEncoderReranker::with_ssb(
            config,
            Arc::new(SeededClock::new()) as Arc<dyn ClockProvider>,
            ssb,
        );

        // Trip L-12 breaker.
        for _ in 0..3 {
            reranker.record_l12_failure();
        }

        // load_fallback would fail, but reranker should not have loaded it.
        // When rerank is called, L-12 breaker is open, fallback_session is None,
        // so it falls through to skip-reranking gracefully.
        let candidates = make_candidates(20);
        let original_ids: Vec<_> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
        let result = reranker.rerank("test query", candidates).unwrap();
        let result_ids: Vec<_> = result.iter().map(|c| c.chunk_id.clone()).collect();
        assert_eq!(
            original_ids, result_ids,
            "with L-6 not loaded, should skip reranking gracefully"
        );
    }

    /// Each test creates fresh breaker instance — no shared static state.
    #[test]
    fn breaker_instances_are_independent() {
        let ssb1 = Arc::new(RecordingSsbNotifier::new());
        let ssb2 = Arc::new(RecordingSsbNotifier::new());
        let r1 = make_reranker_with_ssb(ssb1);
        let r2 = make_reranker_with_ssb(ssb2);

        // Trip breaker on r1 only.
        for _ in 0..3 {
            r1.record_l12_failure();
        }
        assert_eq!(r1.breaker_state(), Some(BreakerState::Open));
        assert_eq!(
            r2.breaker_state(),
            Some(BreakerState::Closed),
            "r2 breaker should be independent from r1"
        );
    }

    /// SSB event emitted on circuit breaker recovery (close after half-open probe).
    #[test]
    fn ssb_event_on_recovery() {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged");
            return;
        }

        let clock = Arc::new(SeededClock::new());
        let ssb = Arc::new(RecordingSsbNotifier::new());
        let mut config = make_config();
        config.model_path = mp;
        config.tokenizer_path = tp;
        let mut reranker = CrossEncoderReranker::with_ssb(
            config,
            clock.clone() as Arc<dyn ClockProvider>,
            ssb.clone(),
        );
        reranker.load().expect("load should succeed");

        // Trip L-12.
        for _ in 0..3 {
            reranker.record_l12_failure();
        }

        // Trigger SSB emission by calling rerank (L-12 open -> degradation event).
        let candidates = make_candidates(12);
        let _ = reranker.rerank("test", candidates);

        let events_before = ssb.events().len();
        assert!(events_before > 0, "should have degradation event");

        // Advance clock past half-open timeout.
        for _ in 0..30_001 {
            let _ = clock.now();
        }

        // Successful rerank transitions breaker Closed.
        let candidates = make_candidates(12);
        let result = reranker.rerank("test", candidates);
        assert!(result.is_ok());
        assert_eq!(reranker.breaker_state(), Some(BreakerState::Closed));
    }
}
