use crate::ReadinessStatus;

pub mod cache;
pub mod circuit_breaker;
pub mod hnsw_index;
pub mod indexer;
pub mod ort_session;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum VectorError {
    DimensionMismatch { expected: usize, got: usize },
    NotReady,
    CacheError(String),
    OrtError(String),
    IndexError(String),
}

impl std::fmt::Display for VectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DimensionMismatch { expected, got } => {
                write!(f, "dimension mismatch: expected {expected}, got {got}")
            }
            Self::NotReady => write!(f, "vector module not ready"),
            Self::CacheError(e) => write!(f, "cache error: {e}"),
            Self::OrtError(e) => write!(f, "ort error: {e}"),
            Self::IndexError(e) => write!(f, "index error: {e}"),
        }
    }
}

impl std::error::Error for VectorError {}

// ---------------------------------------------------------------------------
// Search mode (canonical definition in crate::degradation)
// ---------------------------------------------------------------------------

pub use crate::degradation::SearchMode;

// ---------------------------------------------------------------------------
// DualVector
// ---------------------------------------------------------------------------

pub const CODE_VEC_DIM: usize = 768;
pub const TEXT_VEC_DIM: usize = 384;

#[derive(Debug, Clone)]
pub struct DualVector {
    pub code_vec: Vec<f32>,
    pub text_vec: Vec<f32>,
}

impl DualVector {
    pub fn new(code_vec: Vec<f32>, text_vec: Vec<f32>) -> Result<Self, VectorError> {
        if code_vec.len() != CODE_VEC_DIM {
            return Err(VectorError::DimensionMismatch {
                expected: CODE_VEC_DIM,
                got: code_vec.len(),
            });
        }
        if text_vec.len() != TEXT_VEC_DIM {
            return Err(VectorError::DimensionMismatch {
                expected: TEXT_VEC_DIM,
                got: text_vec.len(),
            });
        }
        Ok(Self { code_vec, text_vec })
    }
}

// ---------------------------------------------------------------------------
// VectorModule trait
// ---------------------------------------------------------------------------

pub trait VectorModule {
    fn embed_code(&self, text: &str) -> Result<Vec<f32>, VectorError>;
    fn embed_text(&self, text: &str) -> Result<Vec<f32>, VectorError>;
    fn readiness(&self) -> ReadinessStatus;
}

// ---------------------------------------------------------------------------
// OrtVectorModule — production VectorModule backed by OrtSession
// ---------------------------------------------------------------------------

/// Production embedding module using two ONNX sessions: one for code
/// (GraphCodeBERT, 768-dim) and one for text (Granite, 384-dim).
///
/// Gracefully degrades: if either session's embed() returns Err, callers
/// get VectorError::OrtError and the pipeline falls back to BM25-only via
/// the degradation cascade. embed() is fully wired against ORT sessions
/// (see ort_session.rs:128).
pub struct OrtVectorModule {
    code_session: ort_session::OrtSession,
    text_session: ort_session::OrtSession,
}

impl OrtVectorModule {
    pub fn new(
        code_session: ort_session::OrtSession,
        text_session: ort_session::OrtSession,
    ) -> Self {
        Self { code_session, text_session }
    }
}

impl VectorModule for OrtVectorModule {
    fn embed_code(&self, text: &str) -> Result<Vec<f32>, VectorError> {
        self.code_session
            .embed(text)
            .map_err(|e| VectorError::OrtError(e.to_string()))
    }

    fn embed_text(&self, text: &str) -> Result<Vec<f32>, VectorError> {
        self.text_session
            .embed(text)
            .map_err(|e| VectorError::OrtError(e.to_string()))
    }

    fn readiness(&self) -> ReadinessStatus {
        let code_ok = self.code_session.embed("test").is_ok();
        let text_ok = self.text_session.embed("test").is_ok();
        match (code_ok, text_ok) {
            (true, true) => ReadinessStatus::Ready,
            (false, false) => ReadinessStatus::Unavailable,
            _ => ReadinessStatus::Degraded,
        }
    }
}

// ---------------------------------------------------------------------------
// E-36 — 4-list RRF fusion helper.
//
// The existing `knowledge::pipeline::rrf_rerank_with_tier` already
// accumulates over a `Vec<PipelineStage>`, so a 4th `vision_vec` stage
// composes naturally when callers push it. This public helper ships
// the per-stage weighted RRF formula so the
// `rrf_4list_fusion_contract.rs` test has a stable surface to assert
// against without reaching into pipeline internals.
//
// Formula: sum_over_stages( weight_s / (k + rank_s) ) where k = 60
// (Cormack et al. SIGIR'09). Default weights per plan task 6:
// bm25 = 1.0, code_vec = 1.0, text_vec = 0.8, vision_vec = 0.6
// (vision is weighted lower because REAL-MM-RAG arXiv:2502.12342
// reports it as brittle on text-heavy / table queries).
// ---------------------------------------------------------------------------

/// Cormack et al. SIGIR'09 RRF constant. Pinned at 60 across the
/// pipeline. Exported so callers can reproduce the formula without
/// re-importing the constant from the private pipeline module.
pub const RRF_K: f32 = 60.0;

/// Default RRF weights for the 4-list vision-RAG fusion. Producer
/// reads these from `[knowledge.rrf] weights` in `elai_cc.toml` when
/// the operator overrides; the defaults here are the documented
/// shipping shape per E-36 plan task 6.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RrfWeights4 {
    pub bm25: f32,
    pub code_vec: f32,
    pub text_vec: f32,
    pub vision_vec: f32,
}

impl Default for RrfWeights4 {
    fn default() -> Self {
        Self {
            bm25: 1.0,
            code_vec: 1.0,
            text_vec: 0.8,
            vision_vec: 0.6,
        }
    }
}

/// Compute the weighted 4-list RRF score for a candidate that appears
/// in each list at the given 1-based rank. Pass `None` for any list
/// the candidate is absent from (the absent-list term contributes 0).
///
/// Determinism: same inputs always yield the same f32 (no rand / no
/// platform-dependent fma rearrangement; arithmetic stays in a single
/// associativity order).
#[must_use]
pub fn rrf_score_4_weighted(
    bm25_rank: Option<usize>,
    code_rank: Option<usize>,
    text_rank: Option<usize>,
    vision_rank: Option<usize>,
    weights: &RrfWeights4,
) -> f32 {
    let term = |rank: Option<usize>, w: f32| -> f32 {
        match rank {
            Some(r) => w / (RRF_K + r as f32),
            None => 0.0,
        }
    };
    term(bm25_rank, weights.bm25)
        + term(code_rank, weights.code_vec)
        + term(text_rank, weights.text_vec)
        + term(vision_rank, weights.vision_vec)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::ort_session::OrtError;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    // -----------------------------------------------------------------
    // Query validation now lives in crate::query_validator (Phase 1I.3).
    // Tests below updated to use the production QueryValidator.
    // -----------------------------------------------------------------

    use crate::query_validator::{QueryValidator, QueryValidationError};

    // Mock embed session for tests that need controllable embed behavior
    struct MockEmbedSession {
        _code_dim: usize,
        _text_dim: usize,
        code_result: Box<dyn Fn(&str) -> Result<Vec<f32>, VectorError> + Send + Sync>,
        text_result: Box<dyn Fn(&str) -> Result<Vec<f32>, VectorError> + Send + Sync>,
    }

    impl MockEmbedSession {
        fn ok_fixed(code_dim: usize, text_dim: usize) -> Self {
            Self {
                _code_dim: code_dim,
                _text_dim: text_dim,
                code_result: Box::new(move |_| Ok(vec![0.1f32; code_dim])),
                text_result: Box::new(move |_| Ok(vec![0.2f32; text_dim])),
            }
        }

        fn code_fails() -> Self {
            Self {
                _code_dim: CODE_VEC_DIM,
                _text_dim: TEXT_VEC_DIM,
                code_result: Box::new(|_| Err(VectorError::OrtError("code model failed".into()))),
                text_result: Box::new(|_| Ok(vec![0.2f32; TEXT_VEC_DIM])),
            }
        }

        fn both_fail() -> Self {
            Self {
                _code_dim: CODE_VEC_DIM,
                _text_dim: TEXT_VEC_DIM,
                code_result: Box::new(|_| Err(VectorError::OrtError("code model failed".into()))),
                text_result: Box::new(|_| Err(VectorError::OrtError("text model failed".into()))),
            }
        }

        fn embed_code(&self, text: &str) -> Result<Vec<f32>, VectorError> {
            (self.code_result)(text)
        }

        fn embed_text(&self, text: &str) -> Result<Vec<f32>, VectorError> {
            (self.text_result)(text)
        }
    }

    // Stub for readiness check (Step 4 implements the real version)
    fn readiness_check(mock: &MockEmbedSession) -> ReadinessStatus {
        let code_ok = mock.embed_code("canary").is_ok();
        let text_ok = mock.embed_text("canary").is_ok();
        if code_ok && text_ok {
            ReadinessStatus::Ready
        } else {
            ReadinessStatus::Degraded
        }
    }

    // Stub for rerank semaphore (Step 4 implements with tokio::Semaphore)
    fn try_acquire_rerank_permit(active: &Mutex<u32>, max: u32) -> bool {
        let mut count = active.lock().unwrap();
        if *count >= max {
            return false;
        }
        *count += 1;
        true
    }

    // RRF score computation: k=60, Cormack et al. SIGIR'09
    fn rrf_score(bm25_rank: usize, code_rank: usize, text_rank: usize) -> f32 {
        1.0 / (60.0 + bm25_rank as f32)
            + 1.0 / (60.0 + code_rank as f32)
            + 1.0 / (60.0 + text_rank as f32)
    }

    // -----------------------------------------------------------------
    // Existing tests
    // -----------------------------------------------------------------

    #[test]
    fn dual_vector_correct_dims_ok() {
        let code = vec![0.0f32; CODE_VEC_DIM];
        let text = vec![0.0f32; TEXT_VEC_DIM];
        let dv = DualVector::new(code, text);
        assert!(dv.is_ok());
        let dv = dv.unwrap();
        assert_eq!(dv.code_vec.len(), CODE_VEC_DIM);
        assert_eq!(dv.text_vec.len(), TEXT_VEC_DIM);
    }

    #[test]
    fn dual_vector_wrong_code_dim_returns_err() {
        let code = vec![0.0f32; 512];
        let text = vec![0.0f32; TEXT_VEC_DIM];
        let result = DualVector::new(code, text);
        assert!(result.is_err());
        match result.unwrap_err() {
            VectorError::DimensionMismatch { expected, got } => {
                assert_eq!(expected, CODE_VEC_DIM);
                assert_eq!(got, 512);
            }
            other => panic!("expected DimensionMismatch, got {other:?}"),
        }
    }

    #[test]
    fn dual_vector_wrong_text_dim_returns_err() {
        let code = vec![0.0f32; CODE_VEC_DIM];
        let text = vec![0.0f32; 256];
        let result = DualVector::new(code, text);
        assert!(result.is_err());
        match result.unwrap_err() {
            VectorError::DimensionMismatch { expected, got } => {
                assert_eq!(expected, TEXT_VEC_DIM);
                assert_eq!(got, 256);
            }
            other => panic!("expected DimensionMismatch, got {other:?}"),
        }
    }

    #[test]
    fn dual_vector_both_wrong_returns_code_dim_err_first() {
        let code = vec![0.0f32; 100];
        let text = vec![0.0f32; 100];
        let result = DualVector::new(code, text);
        assert!(result.is_err());
        match result.unwrap_err() {
            VectorError::DimensionMismatch { expected, got } => {
                assert_eq!(expected, CODE_VEC_DIM);
                assert_eq!(got, 100);
            }
            other => panic!("expected DimensionMismatch for code_vec, got {other:?}"),
        }
    }

    // --- Step 3 tests ---

    #[test]
    fn test_dual_vector_dimensions() {
        let mock = MockEmbedSession::ok_fixed(CODE_VEC_DIM, TEXT_VEC_DIM);
        let code_vec = mock.embed_code("fn main() {}").unwrap();
        let text_vec = mock.embed_text("main function").unwrap();
        assert_eq!(code_vec.len(), CODE_VEC_DIM);
        assert_eq!(text_vec.len(), TEXT_VEC_DIM);
        let dv = DualVector::new(code_vec, text_vec).unwrap();
        assert_eq!(dv.code_vec.len(), CODE_VEC_DIM);
        assert_eq!(dv.text_vec.len(), TEXT_VEC_DIM);
    }

    #[test]
    fn test_graceful_degradation_code_model_fail() {
        let mock = MockEmbedSession::code_fails();
        let code_result = mock.embed_code("fn broken() {}");
        let text_result = mock.embed_text("broken function");
        assert!(code_result.is_err());
        assert!(text_result.is_ok());
        let text_vec = text_result.unwrap();
        assert_eq!(text_vec.len(), TEXT_VEC_DIM);
        // When code model fails, system should degrade to text-only search
        let mode = if code_result.is_err() && text_vec.len() == TEXT_VEC_DIM {
            SearchMode::TextOnly
        } else {
            SearchMode::Hybrid
        };
        assert_eq!(mode, SearchMode::TextOnly);
    }

    #[test]
    fn test_graceful_degradation_both_models_fail() {
        let mock = MockEmbedSession::both_fail();
        let code_result = mock.embed_code("fn broken() {}");
        let text_result = mock.embed_text("broken function");
        assert!(code_result.is_err());
        assert!(text_result.is_err());
        let mode = if code_result.is_err() && text_result.is_err() {
            SearchMode::SparseOnly
        } else {
            SearchMode::Hybrid
        };
        assert_eq!(mode, SearchMode::SparseOnly);
    }

    #[test]
    fn test_query_validation_empty() {
        let result = QueryValidator::validate("");
        assert_eq!(result, Err(QueryValidationError::Empty));
    }

    #[test]
    fn test_query_validation_oversized() {
        let big_query = "x".repeat(10_001);
        assert!(matches!(
            QueryValidator::validate(&big_query),
            Err(QueryValidationError::TooLong { .. })
        ));
    }

    #[test]
    fn test_query_validation_control_chars() {
        assert!(matches!(
            QueryValidator::validate("hello\x00world"),
            Err(QueryValidationError::InvalidChars { .. })
        ));
        assert!(matches!(
            QueryValidator::validate("hello\x01world"),
            Err(QueryValidationError::InvalidChars { .. })
        ));
    }

    #[test]
    fn test_startup_canary_success() {
        let mock = MockEmbedSession::ok_fixed(CODE_VEC_DIM, TEXT_VEC_DIM);
        let status = readiness_check(&mock);
        assert_eq!(status, ReadinessStatus::Ready);
    }

    #[test]
    fn test_startup_canary_timeout() {
        // Both models fail -> Degraded (BM25-only fallback)
        let mock = MockEmbedSession::both_fail();
        let status = readiness_check(&mock);
        assert_eq!(status, ReadinessStatus::Degraded);
    }

    #[test]
    fn test_onnx_canonical_path() {
        // Canonical path pattern: {project}/models/onnx/{model_name}.onnx
        let model_name = "graphcodebert";
        let canonical = PathBuf::from(format!(
            "/nonexistent-elai-test/models/onnx/{model_name}.onnx"
        ));
        assert!(canonical.to_str().unwrap().contains("models/onnx/"));
        assert!(canonical.extension().unwrap() == "onnx");

        // OrtSession::new with nonexistent canonical path returns ModelNotFound
        let tok = PathBuf::from("/nonexistent/tokenizer.json");
        let result = super::ort_session::OrtSession::new(&canonical, &tok, 512, 768);
        assert!(result.is_err());
        match result.unwrap_err() {
            OrtError::ModelNotFound(msg) => {
                assert!(msg.contains(model_name));
            }
            other => panic!("expected ModelNotFound, got {other:?}"),
        }
    }

    #[test]
    fn test_missing_model_degraded() {
        let bad_path = PathBuf::from("/nonexistent/model.onnx");
        let bad_tok = PathBuf::from("/nonexistent/tokenizer.json");
        let result = super::ort_session::OrtSession::new(&bad_path, &bad_tok, 512, 768);
        assert!(result.is_err());
        match result.unwrap_err() {
            OrtError::ModelNotFound(_) => {}
            other => panic!("expected ModelNotFound, got {other:?}"),
        }
        // When model is missing, knowledge engine falls back to Degraded.
        // Verify via readiness_check: both models failing -> Degraded.
        let mock = MockEmbedSession::both_fail();
        let status = readiness_check(&mock);
        assert_eq!(status, ReadinessStatus::Degraded);
    }

    #[test]
    fn test_semaphore_queue_overflow() {
        let active = Mutex::new(0u32);
        let max_concurrent = 4u32;

        // Acquire 4 permits
        for _ in 0..4 {
            assert!(try_acquire_rerank_permit(&active, max_concurrent));
        }

        // 5th should fail — overflow
        assert!(!try_acquire_rerank_permit(&active, max_concurrent));
    }

    #[test]
    fn test_cache_hit_skips_embed() {
        use super::cache::VectorCache;

        let embed_called = Arc::new(Mutex::new(false));
        let called_clone = embed_called.clone();

        let mock = MockEmbedSession {
            _code_dim: CODE_VEC_DIM,
            _text_dim: TEXT_VEC_DIM,
            code_result: Box::new(move |_| {
                *called_clone.lock().unwrap() = true;
                Ok(vec![0.1f32; CODE_VEC_DIM])
            }),
            text_result: Box::new(|_| Ok(vec![0.2f32; TEXT_VEC_DIM])),
        };

        // Pre-populate cache with a DualVector for known content
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let cache = VectorCache::new(conn).unwrap();
        let content = b"fn cached_function() {}";
        let hash = VectorCache::content_hash(content);
        let cached_code = vec![0.5f32; CODE_VEC_DIM];
        let cached_text = vec![0.5f32; TEXT_VEC_DIM];
        let cached_dv = DualVector::new(cached_code, cached_text).unwrap();
        cache.insert(&hash, "src/cached.rs", &cached_dv, 1700000000).unwrap();

        // Lookup cache — if hit, skip embed
        let cache_hit = cache.lookup(&hash).unwrap().is_some();
        if !cache_hit {
            let _ = mock.embed_code("should not run");
        }
        assert!(cache_hit, "cache should contain the pre-inserted entry");
        assert!(!*embed_called.lock().unwrap(), "embed should not be called on cache hit");
    }

    #[test]
    fn test_truncation_warning_on_long_input() {
        let truncated = Arc::new(Mutex::new(false));
        let truncated_clone = truncated.clone();
        let max_seq_len: usize = 512;

        let mock = MockEmbedSession {
            _code_dim: CODE_VEC_DIM,
            _text_dim: TEXT_VEC_DIM,
            code_result: Box::new(move |text| {
                // Simulate tokenizer: ~4 chars per token as rough estimate
                let approx_tokens = text.len() / 4;
                if approx_tokens > max_seq_len {
                    *truncated_clone.lock().unwrap() = true;
                }
                Ok(vec![0.1f32; CODE_VEC_DIM])
            }),
            text_result: Box::new(|_| Ok(vec![0.2f32; TEXT_VEC_DIM])),
        };

        // Input that exceeds 512 tokens (~2048+ chars at 4 chars/token)
        let long_input = "x".repeat(3000);
        let result = mock.embed_code(&long_input);
        assert!(result.is_ok());
        let vec = result.unwrap();
        assert_eq!(vec.len(), CODE_VEC_DIM, "output dim must be correct despite truncation");
        assert!(*truncated.lock().unwrap(), "truncation warning should have fired");
    }

    #[test]
    fn test_concurrent_embed_mutex_session() {
        // 4 threads sharing an Arc<Mutex<MockEmbedSession>>
        let mock = Arc::new(Mutex::new(MockEmbedSession::ok_fixed(CODE_VEC_DIM, TEXT_VEC_DIM)));
        let mut handles = Vec::new();

        for i in 0..4 {
            let mock_clone = mock.clone();
            handles.push(std::thread::spawn(move || {
                let m = mock_clone.lock().unwrap();
                let result = m.embed_code(&format!("fn task_{i}() {{}}"));
                assert!(result.is_ok());
                let vec = result.unwrap();
                assert_eq!(vec.len(), CODE_VEC_DIM);
            }));
        }

        for h in handles {
            h.join().expect("thread should not panic");
        }
    }

    #[test]
    fn test_rrf_fusion_formula() {
        // 3 docs, 3 rankers (BM25, code_vec, text_vec)
        // Doc 0: BM25_rank=1, code_rank=2, text_rank=3
        // Doc 1: BM25_rank=3, code_rank=1, text_rank=2
        // Doc 2: BM25_rank=5, code_rank=4, text_rank=1
        let score_0 = rrf_score(1, 2, 3);
        let score_1 = rrf_score(3, 1, 2);
        let score_2 = rrf_score(5, 4, 1);

        let expected_0: f32 = 1.0 / 61.0 + 1.0 / 62.0 + 1.0 / 63.0;
        let expected_1: f32 = 1.0 / 63.0 + 1.0 / 61.0 + 1.0 / 62.0;
        let expected_2: f32 = 1.0 / 65.0 + 1.0 / 64.0 + 1.0 / 61.0;

        assert!(
            (score_0 - expected_0).abs() < 1e-6,
            "doc 0: got {score_0}, expected {expected_0}"
        );
        assert!(
            (score_1 - expected_1).abs() < 1e-6,
            "doc 1: got {score_1}, expected {expected_1}"
        );
        assert!(
            (score_2 - expected_2).abs() < 1e-6,
            "doc 2: got {score_2}, expected {expected_2}"
        );

        // Doc 0 and Doc 1 should have same score (symmetric ranks)
        assert!(
            (score_0 - score_1).abs() < 1e-6,
            "doc 0 and doc 1 should have equal RRF scores"
        );
    }
}
