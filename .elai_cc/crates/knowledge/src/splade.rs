//! SPLADE sparse keyword expansion for code search.
//!
//! Feature-gated behind `cfg(feature = "splade")`. When disabled, the entire
//! module is excluded from compilation. The `ort` crate remains available
//! unconditionally (used by GraphCodeBERT/Granite reranker); only the
//! SPLADE-specific `ort::Session` initialization lives behind this gate.

/// A single term with its SPLADE expansion weight.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpandedTerm {
    pub term: String,
    pub weight: f32,
}

/// SPLADE sparse expander backed by an ONNX model and BERT WordPiece tokenizer.
///
/// Supports two model variants:
/// - `PrePooled`: model outputs already-pooled sparse vector `[1, 30522]`.
/// - `RawMlm`: model outputs raw MLM logits `[1, seq_len, 30522]`; pooling
///   (max-pool over token positions, then ReLU + log(1+x)) is applied in Rust.
pub struct SpladeExpander {
    _private: (), // fields added in implementation step
}

impl SpladeExpander {
    /// Expand a query into sparse (term, weight) pairs via SPLADE inference.
    ///
    /// Returns an empty vector when:
    /// - the input is empty,
    /// - the model is unavailable (circuit-breaker open), or
    /// - inference fails (graceful degradation to BM25-only).
    pub fn expand_query(&self, _query: &str) -> Vec<ExpandedTerm> {
        Vec::new()
    }

    /// Measure what fraction of the given code tokens map to a single
    /// WordPiece token (not split into subwords).
    ///
    /// Returns a ratio in `[0.0, 1.0]`. Values below 0.3 indicate the BERT
    /// vocabulary is fundamentally mismatched for the corpus and SPLADE
    /// expansion is unlikely to help.
    pub fn vocab_coverage(&self, _corpus_tokens: &[String]) -> f64 {
        0.0
    }
}

/// Reciprocal Rank Fusion of BM25 and SPLADE result sets.
///
/// `k` is clamped to a minimum of 1 to prevent division by zero.
/// Ranks are 1-based (first result = rank 1).
pub fn rrf_fuse(
    bm25_results: &[(String, f32)],
    splade_results: &[(String, f32)],
    k: u32,
    weight_bm25: f32,
    weight_splade: f32,
) -> Vec<(String, f32)> {
    let _ = (bm25_results, splade_results, k, weight_bm25, weight_splade);
    Vec::new()
}
