// 4D.14 contract tests: WebContentIndex — web-content A-RAG with sentence-level indexing.

use elai_core::source_tier::SourceTier;
use elai_knowledge::arag::ContextTracker;
use elai_knowledge::web_arag::{WebAragConfig, WebChunk, WebContentIndex};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_chunk(id: &str, url: &str, tier: SourceTier, text: &str) -> WebChunk {
    WebChunk::new(id.to_string(), url.to_string(), tier, text.to_string())
}

fn make_embedding(dims: usize, val: f32) -> Vec<f32> {
    vec![val; dims]
}

// ---------------------------------------------------------------------------
// Contract: ingest and keyword search
// ---------------------------------------------------------------------------

#[test]
fn ingest_and_keyword_search() {
    let mut index = WebContentIndex::new(WebAragConfig::default());

    let chunk = make_chunk(
        "c1",
        "https://docs.rs/tokio",
        SourceTier::T1Authoritative,
        "Tokio is an asynchronous runtime for the Rust programming language. \
         It provides async IO and task scheduling.",
    );
    index.ingest(chunk);

    let results = index.keyword_search("tokio runtime", 5);
    assert!(!results.is_empty(), "keyword search should return results");
    assert_eq!(results[0].chunk_id, "c1");
    assert!(results[0].score > 0.0, "score must be positive");
    assert_eq!(results[0].tool_used, "keyword_search");
}

// ---------------------------------------------------------------------------
// Contract: ingest and chunk_read (progressive disclosure)
// ---------------------------------------------------------------------------

#[test]
fn ingest_and_chunk_read() {
    let mut index = WebContentIndex::new(WebAragConfig::default());
    let mut tracker = ContextTracker::new();

    let chunk = make_chunk(
        "c1",
        "https://docs.rs/tokio",
        SourceTier::T1Authoritative,
        "Tokio is an asynchronous runtime for Rust.",
    );
    index.ingest(chunk);

    // First read: returns full text.
    let results = index.chunk_read(vec!["c1".to_string()], &mut tracker);
    assert_eq!(results.len(), 1);
    assert!(results[0].snippet.contains("Tokio"));
    assert_eq!(results[0].tool_used, "chunk_read");

    // Second read: chunk already marked, still returns (progressive disclosure).
    let results2 = index.chunk_read(vec!["c1".to_string()], &mut tracker);
    assert_eq!(results2.len(), 1);

    // Unknown chunk: returns empty.
    let results3 = index.chunk_read(vec!["unknown".to_string()], &mut tracker);
    assert!(results3.is_empty());
}

// ---------------------------------------------------------------------------
// Contract: sentence splitting with URL guard
// ---------------------------------------------------------------------------

#[test]
fn sentence_split_url_guard() {
    let text = "Visit https://docs.rs/tokio/latest/tokio. It has great docs. Check it out.";
    let chunk = make_chunk("c1", "https://example.com", SourceTier::T3Weak, text);

    // The URL should NOT be split at internal periods.
    let sentences = chunk.sentences();
    // Should have ~3 sentences, not 5+ (which would happen if URL periods split).
    assert!(
        sentences.len() <= 4,
        "URL guard failed: got {} sentences from text with URL: {:?}",
        sentences.len(),
        sentences
    );
    // First sentence should contain the full URL.
    assert!(
        sentences[0].contains("https://docs.rs/tokio/latest/tokio"),
        "URL was split: first sentence = {:?}",
        sentences[0]
    );
}

// ---------------------------------------------------------------------------
// Contract: keyword score formula
// ---------------------------------------------------------------------------

#[test]
fn keyword_score_formula() {
    let mut index = WebContentIndex::new(WebAragConfig::default());

    // Chunk with "rust" appearing 3 times in 10 words.
    let chunk = make_chunk(
        "c1",
        "https://example.com",
        SourceTier::T2Strong,
        "Rust Rust Rust is a systems programming language built for safety.",
    );
    index.ingest(chunk);

    let results = index.keyword_search("rust", 5);
    assert_eq!(results.len(), 1);

    // "Rust Rust Rust is a systems programming language built for safety." = 11 words
    // Expected: count(3) * len("rust"=4) / sqrt(11) = 12 / 3.316... ≈ 3.617
    let expected = 3.0_f64 * 4.0 / (11.0_f64).sqrt();
    let tolerance = 0.1;
    assert!(
        (results[0].score - expected).abs() < tolerance,
        "score {} not close to expected {}",
        results[0].score,
        expected
    );
}

// ---------------------------------------------------------------------------
// Contract: semantic search with L2 normalization
// ---------------------------------------------------------------------------

#[test]
fn semantic_search_l2_normalized() {
    let mut index = WebContentIndex::new(WebAragConfig::default());

    // Two chunks with different embeddings.
    let chunk1 = make_chunk("c1", "https://a.com", SourceTier::T1Authoritative, "Alpha beta gamma delta epsilon.");
    let chunk2 = make_chunk("c2", "https://b.com", SourceTier::T2Strong, "Zeta eta theta iota kappa.");

    index.ingest(chunk1);
    index.ingest(chunk2);

    // Set embeddings: c1 sentences get [1,0,0,0], c2 sentences get [0,1,0,0].
    let emb1 = make_embedding(4, 0.0);
    let mut emb1_set = emb1.clone();
    emb1_set[0] = 1.0;
    let mut emb2_set = make_embedding(4, 0.0);
    emb2_set[1] = 1.0;

    index.set_sentence_embeddings(vec![emb1_set.clone(), emb2_set.clone()]);

    // Query embedding aligned with c1.
    let query_emb = vec![1.0_f32, 0.0, 0.0, 0.0];
    let results = index.semantic_search("alpha", &query_emb, 1);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].chunk_id, "c1");
    assert!(results[0].score > 0.5, "cosine similarity should be high");
}

// ---------------------------------------------------------------------------
// Contract: chunk_size ~500 tokens
// ---------------------------------------------------------------------------

#[test]
fn chunk_size_500_tokens() {
    let config = WebAragConfig::default();
    assert_eq!(
        config.chunk_size_tokens, 500,
        "default chunk_size_tokens should be 500"
    );
}
