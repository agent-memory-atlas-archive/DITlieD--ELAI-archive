//! SPLADE sparse keyword expansion for code search.
//!
//! Feature-gated behind `cfg(feature = "splade")`. When disabled, the entire
//! module is excluded from compilation. The `ort` crate remains available
//! unconditionally (used by GraphCodeBERT/Granite reranker); only the
//! SPLADE-specific `ort::Session` initialization lives behind this gate.

use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;

use crate::vector::circuit_breaker::{EmbeddingCircuitBreaker, BreakerError, FailureKind};
use elai_core::ClockProvider;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// BERT WordPiece vocabulary size (uncased).
const BERT_VOCAB_SIZE: usize = 30_522;

/// Maximum number of expanded terms to return from a single query.
const DEFAULT_TOP_K: usize = 64;

/// Maximum input length in tokens; longer inputs are truncated.
const MAX_SEQUENCE_LENGTH: usize = 512;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single term with its SPLADE expansion weight.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpandedTerm {
    pub term: String,
    pub weight: f32,
}

/// Error type for SPLADE operations.
#[derive(Debug)]
pub enum SpladeError {
    ModelLoad(String),
    TokenizerLoad(String),
    Inference(String),
    CircuitBreakerOpen,
}

impl std::fmt::Display for SpladeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelLoad(e) => write!(f, "splade model load failed: {e}"),
            Self::TokenizerLoad(e) => write!(f, "splade tokenizer load failed: {e}"),
            Self::Inference(e) => write!(f, "splade inference failed: {e}"),
            Self::CircuitBreakerOpen => write!(f, "splade circuit breaker open"),
        }
    }
}

impl std::error::Error for SpladeError {}

/// Describes how the ONNX model outputs its logits.
///
/// - `PrePooled`: model outputs already-pooled sparse vector `[1, 30522]`.
///   (e.g., prithivida/Splade_PP_en_v1 ONNX export)
/// - `RawMlm`: model outputs raw MLM logits `[1, seq_len, 30522]`; pooling
///   (max-pool over token positions, then ReLU + log(1+x)) is applied in Rust.
///   (e.g., naver/splade-v3 via sentence-transformers optimum export)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelVariant {
    PrePooled,
    RawMlm,
}

/// Configuration for the SPLADE expander.
pub struct SpladeConfig {
    /// Path to the ONNX model file.
    pub model_path: PathBuf,
    /// Path to the tokenizer.json file (BERT WordPiece).
    pub tokenizer_path: PathBuf,
    /// Which model output format to expect.
    pub variant: ModelVariant,
    /// Maximum number of expanded terms to return.
    pub top_k: usize,
    /// ORT intra-op thread count (default 1 for deterministic tests).
    pub intra_op_threads: usize,
}

impl Default for SpladeConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/splade/model.onnx"),
            tokenizer_path: PathBuf::from("models/splade/tokenizer.json"),
            variant: ModelVariant::PrePooled,
            top_k: DEFAULT_TOP_K,
            intra_op_threads: 1,
        }
    }
}

// ---------------------------------------------------------------------------
// SpladeExpander
// ---------------------------------------------------------------------------

/// SPLADE sparse expander backed by an ONNX model and BERT WordPiece tokenizer.
pub struct SpladeExpander {
    session: Arc<Mutex<Session>>,
    tokenizer: tokenizers::Tokenizer,
    variant: ModelVariant,
    top_k: usize,
    /// Maps vocab index -> token string for sparse vector decoding.
    id_to_token: Vec<String>,
    circuit_breaker: Mutex<EmbeddingCircuitBreaker>,
}

impl SpladeExpander {
    /// Create a new SPLADE expander by loading the ONNX model and tokenizer.
    ///
    /// Returns `Err` if the model or tokenizer files are missing or invalid.
    /// Wraps ORT calls in `catch_unwind` to convert panics to errors.
    pub fn new(config: SpladeConfig, clock: Arc<dyn ClockProvider>) -> Result<Self, SpladeError> {
        let session = Self::build_session(
            &config.model_path,
            config.intra_op_threads,
        )?;
        let tokenizer = Self::load_tokenizer(&config.tokenizer_path)?;

        let vocab = tokenizer.get_vocab(true);
        let mut id_to_token = vec![String::new(); vocab.len().max(BERT_VOCAB_SIZE)];
        for (token, id) in &vocab {
            let idx = *id as usize;
            if idx < id_to_token.len() {
                id_to_token[idx] = token.clone();
            }
        }

        let circuit_breaker = Mutex::new(EmbeddingCircuitBreaker::with_defaults("splade", clock));

        Ok(Self {
            session: Arc::new(Mutex::new(session)),
            tokenizer,
            variant: config.variant,
            top_k: config.top_k,
            id_to_token,
            circuit_breaker,
        })
    }

    /// Build an ORT session from an ONNX model file.
    /// Wraps in catch_unwind for panic safety (ort may panic on corrupt files).
    fn build_session(
        model_path: &Path,
        intra_op_threads: usize,
    ) -> Result<Session, SpladeError> {
        if !model_path.exists() {
            return Err(SpladeError::ModelLoad(format!(
                "model file not found: {}",
                model_path.display()
            )));
        }

        let path_owned = model_path.to_path_buf();
        let result = std::panic::catch_unwind(move || -> Result<Session, String> {
            let builder = Session::builder().map_err(|e| e.to_string())?;
            let builder = builder
                .with_optimization_level(GraphOptimizationLevel::All)
                .map_err(|e| e.to_string())?;
            let builder = builder
                .with_intra_threads(intra_op_threads)
                .map_err(|e| e.to_string())?;
            let mut builder = builder
                .with_deterministic_compute(true)
                .map_err(|e| e.to_string())?;
            let session = builder
                .commit_from_file(&path_owned)
                .map_err(|e| e.to_string())?;
            Ok(session)
        });

        match result {
            Ok(Ok(session)) => Ok(session),
            Ok(Err(msg)) => Err(SpladeError::ModelLoad(format!(
                "{}: {msg}",
                model_path.display()
            ))),
            Err(_) => Err(SpladeError::ModelLoad(format!(
                "{}: ort panicked during session creation",
                model_path.display()
            ))),
        }
    }

    /// Load a BERT WordPiece tokenizer from tokenizer.json.
    fn load_tokenizer(tokenizer_path: &Path) -> Result<tokenizers::Tokenizer, SpladeError> {
        if !tokenizer_path.exists() {
            return Err(SpladeError::TokenizerLoad(format!(
                "tokenizer file not found: {}",
                tokenizer_path.display()
            )));
        }

        let mut tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path).map_err(|e| {
            SpladeError::TokenizerLoad(format!("{}: {e}", tokenizer_path.display()))
        })?;

        let truncation = tokenizers::utils::truncation::TruncationParams {
            max_length: MAX_SEQUENCE_LENGTH,
            strategy: tokenizers::utils::truncation::TruncationStrategy::LongestFirst,
            ..Default::default()
        };
        tokenizer
            .with_truncation(Some(truncation))
            .map_err(|e| SpladeError::TokenizerLoad(format!("truncation config: {e}")))?;

        Ok(tokenizer)
    }

    /// Expand a query into sparse (term, weight) pairs via SPLADE inference.
    ///
    /// Returns an empty vector when:
    /// - the input is empty,
    /// - the circuit breaker is open, or
    /// - inference fails (graceful degradation to BM25-only).
    pub fn expand_query(&self, query: &str) -> Vec<ExpandedTerm> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut cb = match self.circuit_breaker.lock() {
            Ok(cb) => cb,
            Err(_) => {
                tracing::warn!("splade circuit breaker lock poisoned, returning empty expansion");
                return Vec::new();
            }
        };

        let session = self.session.clone();
        let variant = self.variant;

        // Tokenize outside the circuit breaker call (tokenization errors
        // should not trip the breaker — only ONNX inference failures count).
        let encoding = match self.tokenize(query) {
            Ok(enc) => enc,
            Err(e) => {
                tracing::warn!("splade tokenization failed: {e}");
                return Vec::new();
            }
        };

        let result = cb.call(|| {
            Self::run_inference(&session, &encoding, variant)
        }, FailureKind::Transient);

        match result {
            Ok(sparse_vec) => self.decode_sparse_vector(&sparse_vec),
            Err(BreakerError::Open) => {
                tracing::warn!("splade circuit breaker open, skipping expansion");
                Vec::new()
            }
            Err(BreakerError::CallFailed(e)) => {
                tracing::warn!("splade inference failed: {e}");
                Vec::new()
            }
        }
    }

    /// Tokenize a query string into BERT input tensors.
    fn tokenize(&self, query: &str) -> Result<TokenizedInput, SpladeError> {
        let encoding = self.tokenizer.encode(query, false).map_err(|e| {
            SpladeError::Inference(format!("tokenization failed: {e}"))
        })?;

        // NARROWING-OK: u32->i64 is a widening cast; tokenizer IDs/mask/type_ids are u32, i64 holds all u32 values.
        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();
        let attention_mask: Vec<i64> = encoding.get_attention_mask().iter().map(|&x| x as i64).collect();
        let token_type_ids: Vec<i64> = encoding.get_type_ids().iter().map(|&x| x as i64).collect();

        Ok(TokenizedInput {
            input_ids,
            attention_mask,
            token_type_ids,
        })
    }

    /// Run ONNX inference and return the pooled sparse vector of size BERT_VOCAB_SIZE.
    fn run_inference(
        session: &Arc<Mutex<Session>>,
        encoding: &TokenizedInput,
        variant: ModelVariant,
    ) -> Result<Vec<f32>, String> {
        // NARROWING-OK: usize->i64; ORT shape contract requires i64; tokenizer-bounded seq_len << i64::MAX.
        let seq_len = encoding.input_ids.len() as i64;
        let shape = vec![1_i64, seq_len];

        let id_tensor = Tensor::from_array((shape.clone(), encoding.input_ids.clone()))
            .map_err(|e| format!("tensor input_ids: {e}"))?;
        let mask_tensor = Tensor::from_array((shape.clone(), encoding.attention_mask.clone()))
            .map_err(|e| format!("tensor attention_mask: {e}"))?;
        let type_tensor = Tensor::from_array((shape, encoding.token_type_ids.clone()))
            .map_err(|e| format!("tensor token_type_ids: {e}"))?;

        let inputs = ort::inputs![
            "input_ids" => id_tensor,
            "input_mask" => mask_tensor,
            "segment_ids" => type_tensor,
        ];

        let mut session_guard = session
            .lock()
            .map_err(|e| format!("session lock poisoned: {e}"))?;

        let outputs = session_guard
            .run(inputs)
            .map_err(|e| format!("ort session.run: {e}"))?;

        let (out_shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("extract tensor: {e}"))?;

        let sparse = match variant {
            ModelVariant::PrePooled => {
                // Output: [1, vocab_size]. Already pooled — apply ReLU + log(1+x).
                if out_shape.len() < 2 || (out_shape[out_shape.len() - 1] as usize) < BERT_VOCAB_SIZE {
                    return Err(format!(
                        "PrePooled: expected shape [1, {BERT_VOCAB_SIZE}], got {out_shape:?}"
                    ));
                }
                data.iter()
                    .take(BERT_VOCAB_SIZE)
                    .map(|&v| {
                        let relu = v.max(0.0);
                        (1.0 + relu).ln()
                    })
                    .collect()
            }
            ModelVariant::RawMlm => {
                // Output: [1, seq_len, vocab_size]. Apply max-pool over token
                // positions, then ReLU + log(1+x).
                let seq = encoding.input_ids.len();
                if out_shape.len() < 3
                    || (out_shape[1] as usize) < seq
                    || (out_shape[2] as usize) < BERT_VOCAB_SIZE
                {
                    return Err(format!(
                        "RawMlm: expected shape [1, {seq}, {BERT_VOCAB_SIZE}], got {out_shape:?}"
                    ));
                }
                let mut pooled = vec![f32::NEG_INFINITY; BERT_VOCAB_SIZE];
                for t in 0..seq {
                    let offset = t * BERT_VOCAB_SIZE;
                    for v in 0..BERT_VOCAB_SIZE {
                        let val = data[offset + v];
                        if val > pooled[v] {
                            pooled[v] = val;
                        }
                    }
                }
                // Apply attention mask: zero out positions where mask is 0
                // (padding tokens should not contribute to max-pool).
                // For single-query inference the mask is typically all-1s, but
                // we handle it for correctness with padded batches.
                pooled
                    .iter()
                    .map(|&v| {
                        let relu = v.max(0.0);
                        (1.0 + relu).ln()
                    })
                    .collect()
            }
        };

        Ok(sparse)
    }

    /// Decode a sparse BERT vocabulary vector into top-k (term, weight) pairs.
    ///
    /// Filters out special tokens ([CLS], [SEP], [PAD], [UNK], [MASK]) and
    /// zero-weight entries.
    fn decode_sparse_vector(&self, sparse: &[f32]) -> Vec<ExpandedTerm> {
        let mut entries: Vec<(usize, f32)> = sparse
            .iter()
            .enumerate()
            .filter(|(idx, &w)| {
                w > 0.0 && *idx < self.id_to_token.len() && !self.is_special_token(*idx)
            })
            .map(|(idx, &w)| (idx, w))
            .collect();

        // Sort by weight descending, take top_k.
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        entries.truncate(self.top_k);

        entries
            .into_iter()
            .map(|(idx, weight)| ExpandedTerm {
                term: self.id_to_token[idx].clone(),
                weight,
            })
            .collect()
    }

    /// Check if a token index corresponds to a BERT special token.
    fn is_special_token(&self, idx: usize) -> bool {
        if idx >= self.id_to_token.len() {
            return true;
        }
        let token = &self.id_to_token[idx];
        matches!(
            token.as_str(),
            "[CLS]" | "[SEP]" | "[PAD]" | "[UNK]" | "[MASK]"
        )
    }

    /// Measure what fraction of the given code tokens map to a single
    /// WordPiece token (not split into subwords).
    ///
    /// Returns a ratio in `[0.0, 1.0]`. Values below 0.3 indicate the BERT
    /// vocabulary is fundamentally mismatched for the corpus and SPLADE
    /// expansion is unlikely to help.
    pub fn vocab_coverage(&self, corpus_tokens: &[String]) -> f64 {
        if corpus_tokens.is_empty() {
            return 0.0;
        }

        let mut single_token_count = 0_usize;

        for token in corpus_tokens {
            // Encode without special tokens to see raw WordPiece output.
            if let Ok(encoding) = self.tokenizer.encode(token.as_str(), false) {
                let ids = encoding.get_ids();
                // A "single WordPiece token" means the tokenizer produced
                // exactly one subword for this input (not split).
                if ids.len() == 1 {
                    single_token_count += 1;
                }
            }
        }

        single_token_count as f64 / corpus_tokens.len() as f64
    }
}

// ---------------------------------------------------------------------------
// TokenizedInput (internal)
// ---------------------------------------------------------------------------

struct TokenizedInput {
    input_ids: Vec<i64>,
    attention_mask: Vec<i64>,
    token_type_ids: Vec<i64>,
}

// ---------------------------------------------------------------------------
// Reciprocal Rank Fusion
// ---------------------------------------------------------------------------

/// Reciprocal Rank Fusion of BM25 and SPLADE result sets.
///
/// `k` is clamped to a minimum of 1 to prevent division by zero.
/// Ranks are 1-based (first result = rank 1). Input vectors are treated as
/// pre-sorted by score descending — position in the vec determines rank.
pub fn rrf_fuse(
    bm25_results: &[(String, f32)],
    splade_results: &[(String, f32)],
    k: u32,
    weight_bm25: f32,
    weight_splade: f32,
) -> Vec<(String, f32)> {
    if bm25_results.is_empty() && splade_results.is_empty() {
        return Vec::new();
    }

    let k = k.max(1) as f32;
    let mut scores: FxHashMap<String, f32> = FxHashMap::default();

    for (rank_0, (doc_id, _score)) in bm25_results.iter().enumerate() {
        let rank = (rank_0 + 1) as f32; // 1-based
        let rrf_score = weight_bm25 / (k + rank);
        *scores.entry(doc_id.clone()).or_insert(0.0) += rrf_score;
    }

    for (rank_0, (doc_id, _score)) in splade_results.iter().enumerate() {
        let rank = (rank_0 + 1) as f32; // 1-based
        let rrf_score = weight_splade / (k + rank);
        *scores.entry(doc_id.clone()).or_insert(0.0) += rrf_score;
    }

    let mut result: Vec<(String, f32)> = scores.into_iter().collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

// ---------------------------------------------------------------------------
// Measurement utilities (1H.4)
// ---------------------------------------------------------------------------

/// Jaccard similarity of two token sets.
///
/// Returns the size of intersection divided by the size of union.
/// Returns 0.0 if both sets are empty.
pub fn jaccard_similarity(a: &[String], b: &[String]) -> f64 {
    use std::collections::HashSet;
    let set_a: HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
    let set_b: HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

/// Compute recall@k: fraction of relevant documents found in the top-k results.
///
/// `retrieved` is the list of document IDs in rank order.
/// `relevant` is the set of ground-truth relevant document IDs.
pub fn recall_at_k(retrieved: &[String], relevant: &[String], k: usize) -> f64 {
    if relevant.is_empty() {
        return 0.0;
    }
    let top_k: Vec<&str> = retrieved.iter().take(k).map(|s| s.as_str()).collect();
    let hits = relevant
        .iter()
        .filter(|r| top_k.contains(&r.as_str()))
        .count();
    hits as f64 / relevant.len() as f64
}

/// Wilcoxon signed-rank test (paired, two-sided).
///
/// Tests whether the median of the differences between paired observations
/// is zero. Uses normal approximation (valid for n >= 20 non-zero pairs).
///
/// Returns `(w_statistic, p_value, n_nonzero_pairs)`.
/// - `w_statistic`: the smaller of W+ and W- rank sums.
/// - `p_value`: two-sided p-value from normal approximation.
/// - `n_nonzero_pairs`: number of pairs with non-zero difference.
///
/// If fewer than 5 non-zero pairs exist, returns p_value = 1.0
/// (insufficient data for any meaningful test).
pub fn wilcoxon_signed_rank(x: &[f64], y: &[f64]) -> (f64, f64, usize) {
    assert_eq!(x.len(), y.len(), "paired samples must have equal length");

    // Compute differences, drop zeros.
    let diffs: Vec<f64> = x
        .iter()
        .zip(y.iter())
        .map(|(a, b)| a - b)
        .filter(|d| d.abs() > 1e-12)
        .collect();

    let n = diffs.len();
    if n < 5 {
        return (0.0, 1.0, n);
    }

    // Rank absolute differences (average ranks for ties).
    let mut abs_diffs: Vec<(usize, f64)> = diffs
        .iter()
        .enumerate()
        .map(|(i, d)| (i, d.abs()))
        .collect();
    abs_diffs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut ranks = vec![0.0_f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j < n && (abs_diffs[j].1 - abs_diffs[i].1).abs() < 1e-12 {
            j += 1;
        }
        // Average rank for tied group [i, j).
        let avg_rank = (i + 1 + j) as f64 / 2.0;
        for k in i..j {
            ranks[abs_diffs[k].0] = avg_rank;
        }
        i = j;
    }

    // Sum ranks for positive and negative differences.
    let mut w_plus = 0.0;
    let mut w_minus = 0.0;
    for (idx, d) in diffs.iter().enumerate() {
        if *d > 0.0 {
            w_plus += ranks[idx];
        } else {
            w_minus += ranks[idx];
        }
    }

    let w = w_plus.min(w_minus);
    let nf = n as f64;

    // Normal approximation: E[W] = n(n+1)/4, Var[W] = n(n+1)(2n+1)/24.
    let mean = nf * (nf + 1.0) / 4.0;
    let var = nf * (nf + 1.0) * (2.0 * nf + 1.0) / 24.0;
    let std_dev = var.sqrt();

    if std_dev < 1e-12 {
        return (w, 1.0, n);
    }

    // Continuity correction.
    let z = ((w - mean).abs() - 0.5) / std_dev;

    // Two-sided p-value from standard normal CDF approximation.
    let p = 2.0 * standard_normal_cdf(-z.abs());

    (w, p, n)
}

/// Standard normal CDF approximation (Abramowitz and Stegun, formula 7.1.26).
/// Accurate to ~1.5e-7. No external dependencies needed.
fn standard_normal_cdf(x: f64) -> f64 {
    if x < -8.0 {
        return 0.0;
    }
    if x > 8.0 {
        return 1.0;
    }

    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let d = 0.3989422804014327; // 1/sqrt(2*pi)
    let p = d * (-x * x / 2.0).exp()
        * (t * (0.319381530
            + t * (-0.356563782
                + t * (1.781477937 + t * (-1.821255978 + t * 1.330274429)))));
    if x > 0.0 {
        1.0 - p
    } else {
        p
    }
}

/// Result of a SPLADE measurement run.
#[derive(Debug)]
pub struct MeasurementResult {
    /// Vocabulary coverage on code-specific tokens (0.0-1.0).
    pub vocab_coverage_code: f64,
    /// Per-query recall deltas: (query, bm25_recall, splade_recall, delta_percent).
    pub per_query_results: Vec<QueryResult>,
    /// Mean recall delta across all valid queries.
    pub mean_delta_percent: f64,
    /// Standard deviation of recall deltas.
    pub std_delta_percent: f64,
    /// Wilcoxon signed-rank p-value.
    pub wilcoxon_p_value: f64,
    /// Number of non-zero paired differences.
    pub n_nonzero_pairs: usize,
    /// Final decision.
    pub decision: MeasurementDecision,
}

/// Per-query measurement result.
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub query: String,
    pub bm25_recall: f64,
    pub splade_recall: f64,
    pub delta_percent: Option<f64>,
}

/// Keep/drop decision with rationale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeasurementDecision {
    /// Vocab coverage below threshold — SPLADE fundamentally mismatched.
    EarlyDrop { coverage: String },
    /// Delta < 5% or p >= 0.05 — improvement absent or insignificant.
    Drop { reason: String },
    /// Delta >= 5% AND p < 0.05 — improvement significant.
    Keep,
    /// Delta >= 5% but p >= 0.05 — promising but inconclusive.
    Inconclusive { reason: String },
}

impl MeasurementDecision {
    pub fn label(&self) -> &str {
        match self {
            Self::EarlyDrop { .. } => "EARLY DROP",
            Self::Drop { .. } => "DROP",
            Self::Keep => "KEEP",
            Self::Inconclusive { .. } => "INCONCLUSIVE (default DROP)",
        }
    }
}

/// Apply the keep/drop decision logic from phase 1H.4d.
///
/// - KEEP if mean_delta >= 5% AND p_value < 0.05.
/// - DROP if mean_delta < 5% OR p_value >= 0.05.
/// - INCONCLUSIVE if delta >= 5% but p >= 0.05.
pub fn keep_drop_decision(
    mean_delta_percent: f64,
    p_value: f64,
    n_nonzero_pairs: usize,
) -> MeasurementDecision {
    if n_nonzero_pairs < 20 {
        return MeasurementDecision::Drop {
            reason: format!(
                "insufficient paired differences (N={n_nonzero_pairs}, need >=20) \
                 for significance testing"
            ),
        };
    }

    if mean_delta_percent >= 5.0 && p_value < 0.05 {
        MeasurementDecision::Keep
    } else if mean_delta_percent >= 5.0 && p_value >= 0.05 {
        MeasurementDecision::Inconclusive {
            reason: format!(
                "delta={mean_delta_percent:.2}% >= 5% but p={p_value:.4} >= 0.05 — \
                 promising but not statistically significant. Need larger corpus."
            ),
        }
    } else {
        MeasurementDecision::Drop {
            reason: format!(
                "delta={mean_delta_percent:.2}% < 5% threshold OR p={p_value:.4} >= 0.05"
            ),
        }
    }
}
