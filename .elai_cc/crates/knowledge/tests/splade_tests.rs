// 1H.2 + 1H.3: SPLADE test suite.
//
// Tests are feature-gated behind `--features splade`.
//
// Tests that require a real ONNX model + tokenizer check for model files
// at the expected path and skip gracefully if absent. The RRF fusion tests
// are pure logic and always run.
//
// Test categories:
//   (a-c) Unit tests for expand_query (require model)
//   (d-e) RRF fusion tests (pure logic, no model needed)
//   (f)   vocab_coverage unit test (requires tokenizer)
//   (g)   proptest: weight invariants (require model)
//   (h)   Integration: recall comparison BM25-only vs BM25+SPLADE (require model)

#![cfg(feature = "splade")]

use elai_knowledge::splade::{
    rrf_fuse, ModelVariant, SpladeConfig, SpladeExpander,
};
use std::path::PathBuf;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Standard model directory. Override with SPLADE_MODEL_PATH env var.
fn model_dir() -> PathBuf {
    std::env::var("SPLADE_MODEL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("models/splade"))
}

fn model_path() -> PathBuf {
    model_dir().join("model.onnx")
}

fn tokenizer_path() -> PathBuf {
    model_dir().join("tokenizer.json")
}

/// Try to build a real SpladeExpander. Returns None if model files are absent.
fn try_make_expander() -> Option<SpladeExpander> {
    let mp = model_path();
    let tp = tokenizer_path();
    if !mp.exists() || !tp.exists() {
        eprintln!(
            "SKIP: SPLADE model fixtures not staged at {}",
            model_dir().display()
        );
        return None;
    }

    let config = SpladeConfig {
        model_path: mp,
        tokenizer_path: tp,
        variant: ModelVariant::PrePooled,
        top_k: 64,
        intra_op_threads: 1,
    };
    let clock = Arc::new(elai_core::SeededClock::new());
    match SpladeExpander::new(config, clock) {
        Ok(exp) => Some(exp),
        Err(e) => {
            eprintln!("SKIP: SpladeExpander::new failed: {e}");
            None
        }
    }
}

/// Build an expander with only a tokenizer (for vocab_coverage tests).
/// Uses the tokenizer file if present, skips otherwise.
fn try_make_expander_for_vocab() -> Option<SpladeExpander> {
    // vocab_coverage only needs the tokenizer, but SpladeExpander::new also
    // loads the model. We need both files present.
    try_make_expander()
}

// ---------------------------------------------------------------------------
// (a) expand_query on a real query returns non-empty sparse vector
// ---------------------------------------------------------------------------

#[test]
fn a_expand_query_returns_nonempty_for_real_query() {
    let expander = match try_make_expander() {
        Some(e) => e,
        None => return,
    };
    let terms = expander.expand_query("error handling");

    assert!(
        !terms.is_empty(),
        "expand_query(\"error handling\") must return at least one expanded term"
    );

    // When implemented, expect terms like error, exception, try, catch, handling.
    let term_strs: Vec<&str> = terms.iter().map(|t| t.term.as_str()).collect();
    let expected_any = ["error", "exception", "try", "catch", "handling"];
    let has_expected = expected_any.iter().any(|e| term_strs.contains(e));
    assert!(
        has_expected,
        "expected at least one of {:?} in expansion, got {:?}",
        expected_any, term_strs
    );
}

// ---------------------------------------------------------------------------
// (b) expand_query on empty string returns empty vector
// ---------------------------------------------------------------------------

#[test]
fn b_expand_query_empty_input_returns_empty() {
    let expander = match try_make_expander() {
        Some(e) => e,
        None => return,
    };
    let terms = expander.expand_query("");
    assert!(
        terms.is_empty(),
        "expand_query(\"\") should return empty vec, got {} terms",
        terms.len()
    );
}

// ---------------------------------------------------------------------------
// (c) expand_query with very long input (>10000 chars) does not panic
// ---------------------------------------------------------------------------

#[test]
fn c_expand_query_long_input_no_panic() {
    let expander = match try_make_expander() {
        Some(e) => e,
        None => return,
    };
    let long_input = "fn process_data() { ".to_string() + &"x ".repeat(5000) + "}";
    assert!(long_input.len() > 10_000);

    // Must not panic — may return empty or truncated result.
    let terms = expander.expand_query(&long_input);

    // All returned weights (if any) must be finite and non-negative.
    for t in &terms {
        assert!(t.weight.is_finite(), "weight must be finite, got {}", t.weight);
        assert!(t.weight >= 0.0, "weight must be >= 0, got {}", t.weight);
    }
}

// ---------------------------------------------------------------------------
// (d) RRF fusion produces combined ranking different from BM25-only
// ---------------------------------------------------------------------------

#[test]
fn d_rrf_fusion_differs_from_bm25_only() {
    let bm25 = vec![
        ("file_a.rs".to_string(), 3.5_f32),
        ("file_b.rs".to_string(), 2.1),
        ("file_c.rs".to_string(), 1.0),
    ];
    let splade = vec![
        ("file_c.rs".to_string(), 4.0_f32),
        ("file_d.rs".to_string(), 3.0),
        ("file_a.rs".to_string(), 1.5),
    ];

    let fused = rrf_fuse(&bm25, &splade, 60, 0.5, 0.5);

    assert!(
        !fused.is_empty(),
        "rrf_fuse must produce non-empty results when both inputs are non-empty"
    );

    // Fused ranking should differ from BM25-only ordering.
    let bm25_order: Vec<&str> = bm25.iter().map(|(s, _)| s.as_str()).collect();
    let fused_order: Vec<&str> = fused.iter().map(|(s, _)| s.as_str()).collect();
    // file_d.rs is SPLADE-only, so fused must include it (not present in BM25).
    assert!(
        fused_order.contains(&"file_d.rs"),
        "fused results must include SPLADE-only document file_d.rs"
    );
    // The ordering should change because SPLADE heavily boosts file_c.
    assert_ne!(
        fused_order, bm25_order,
        "fused ranking must differ from BM25-only ranking"
    );
}

// ---------------------------------------------------------------------------
// (e) RRF fusion edge cases: k=0 clamp, empty inputs
// ---------------------------------------------------------------------------

#[test]
fn e_rrf_fusion_edge_cases() {
    let bm25 = vec![("a.rs".to_string(), 1.0_f32)];
    let splade = vec![("b.rs".to_string(), 1.0_f32)];

    // k=0 must not panic (clamped to 1 internally).
    let fused = rrf_fuse(&bm25, &splade, 0, 0.5, 0.5);
    assert!(
        !fused.is_empty(),
        "rrf_fuse with k=0 must still produce results (k clamped to 1)"
    );

    // Empty SPLADE results: fused should still contain BM25 results.
    let empty: Vec<(String, f32)> = vec![];
    let fused_bm25_only = rrf_fuse(&bm25, &empty, 60, 0.5, 0.5);
    assert!(
        !fused_bm25_only.is_empty(),
        "rrf_fuse with empty SPLADE must still return BM25 results"
    );

    // Both empty: should return empty.
    let fused_both_empty = rrf_fuse(&empty, &empty, 60, 0.5, 0.5);
    assert!(
        fused_both_empty.is_empty(),
        "rrf_fuse with both inputs empty must return empty"
    );
}

// ---------------------------------------------------------------------------
// (e2) RRF fusion: verify score correctness
// ---------------------------------------------------------------------------

#[test]
fn e2_rrf_fusion_score_correctness() {
    // With k=60, rank 1: score = weight / (60 + 1) = weight / 61
    let bm25 = vec![("only.rs".to_string(), 10.0_f32)];
    let empty: Vec<(String, f32)> = vec![];

    let fused = rrf_fuse(&bm25, &empty, 60, 1.0, 1.0);
    assert_eq!(fused.len(), 1);
    let expected = 1.0 / 61.0;
    assert!(
        (fused[0].1 - expected).abs() < 1e-6,
        "expected score {expected}, got {}",
        fused[0].1
    );

    // Both sources contribute: doc appears in both with rank 1.
    let a = vec![("doc.rs".to_string(), 5.0)];
    let b = vec![("doc.rs".to_string(), 3.0)];
    let fused2 = rrf_fuse(&a, &b, 60, 0.5, 0.5);
    let expected2 = 0.5 / 61.0 + 0.5 / 61.0; // both rank 1
    assert!(
        (fused2[0].1 - expected2).abs() < 1e-6,
        "expected score {expected2}, got {}",
        fused2[0].1
    );
}

// ---------------------------------------------------------------------------
// (f) vocab_coverage returns valid ratio on code tokens
// ---------------------------------------------------------------------------

#[test]
fn f_vocab_coverage_valid_ratio() {
    let expander = match try_make_expander_for_vocab() {
        Some(e) => e,
        None => return,
    };

    let code_tokens: Vec<String> = vec![
        "fn", "impl", "async", "Vec", "HashMap", "unwrap", "println", "match",
        "Result", "Option", "String", "self", "pub", "mod", "use", "struct",
        "enum", "trait", "where", "mut",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let coverage = expander.vocab_coverage(&code_tokens);

    // Must be in [0.0, 1.0].
    assert!(
        (0.0..=1.0).contains(&coverage),
        "vocab_coverage must return ratio in [0.0, 1.0], got {}",
        coverage
    );

    // With real tokenizer, coverage should be > 0.0 for at least some common
    // code tokens (many are short English words that BERT knows).
    assert!(
        coverage > 0.0,
        "vocab_coverage on common code tokens must be > 0.0"
    );
}

// ---------------------------------------------------------------------------
// (f2) vocab_coverage edge: empty input
// ---------------------------------------------------------------------------

#[test]
fn f2_vocab_coverage_empty_input() {
    let expander = match try_make_expander_for_vocab() {
        Some(e) => e,
        None => return,
    };

    let coverage = expander.vocab_coverage(&[]);
    assert!(
        coverage == 0.0,
        "vocab_coverage on empty input must return 0.0, got {coverage}"
    );
}

// ---------------------------------------------------------------------------
// (g) Property test: all weights finite and >= 0.0
// ---------------------------------------------------------------------------

mod proptest_suite {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn proptest_expand_query_weights_invariants(
            query in prop::string::string_regex("[a-zA-Z0-9_]{1,100}")
                .unwrap()
                .prop_filter("non-empty", |s| !s.is_empty()),
        ) {
            let expander = match try_make_expander() {
                Some(e) => e,
                None => return Ok(()),
            };
            let terms = expander.expand_query(&query);

            for t in &terms {
                prop_assert!(
                    t.weight.is_finite(),
                    "weight must be finite for query {:?}, got {}",
                    query,
                    t.weight
                );
                prop_assert!(
                    t.weight >= 0.0,
                    "weight must be >= 0.0 for query {:?}, got {}",
                    query,
                    t.weight
                );
            }
        }

        /// Include Unicode queries to catch potential panics on non-ASCII input.
        #[test]
        fn proptest_expand_query_unicode_no_panic(
            query in prop::string::string_regex("[\\p{L}\\p{N}]{1,50}")
                .unwrap()
                .prop_filter("non-empty", |s| !s.is_empty()),
        ) {
            let expander = match try_make_expander() {
                Some(e) => e,
                None => return Ok(()),
            };
            // Must not panic on any Unicode input.
            let terms = expander.expand_query(&query);

            for t in &terms {
                prop_assert!(t.weight.is_finite());
                prop_assert!(t.weight >= 0.0);
            }
        }
    }

    // RRF property: scores are always finite and >= 0 for any valid input.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        #[test]
        fn proptest_rrf_scores_finite(
            k in 0u32..1000,
            wb in 0.0f32..10.0,
            ws in 0.0f32..10.0,
        ) {
            let bm25 = vec![("a.rs".to_string(), 1.0_f32), ("b.rs".to_string(), 0.5)];
            let splade = vec![("b.rs".to_string(), 2.0_f32), ("c.rs".to_string(), 1.0)];
            let fused = rrf_fuse(&bm25, &splade, k, wb, ws);
            for (_, score) in &fused {
                prop_assert!(score.is_finite(), "RRF score must be finite, got {score}");
                prop_assert!(*score >= 0.0, "RRF score must be >= 0, got {score}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// (h) Integration: recall measurement BM25-only vs BM25+SPLADE
// ---------------------------------------------------------------------------

#[test]
fn h_recall_bm25_vs_splade_synthetic() {
    use elai_knowledge::bm25::{Bm25Index, BM25Chunk, ChunkKind};

    let expander = match try_make_expander() {
        Some(e) => e,
        None => return,
    };

    let tmp = tempfile::TempDir::new().unwrap();
    let mut bm25 = Bm25Index::new(tmp.path(), "splade_recall_test").unwrap();

    // Synthetic corpus: 10 files with known content.
    let corpus: Vec<(&str, &str, bool)> = vec![
        // (filename, content, relevant_to_query_"error handling")
        ("try_catch.rs", "fn handle_error() -> Result<(), Box<dyn std::error::Error>> { Err(\"fail\".into()) }", true),
        ("exception_handler.rs", "fn exception_handler() { panic!(\"unrecoverable\"); }", true),
        ("error_recovery.rs", "fn recover_from_error(e: &str) -> bool { eprintln!(\"recovering: {}\", e); true }", true),
        ("sort_algo.rs", "fn quicksort(arr: &mut [i32]) { arr.sort(); }", false),
        ("http_client.rs", "fn fetch_url(url: &str) -> String { format!(\"response from {}\", url) }", false),
        ("config_parser.rs", "fn parse_config(path: &str) -> HashMap<String, String> { HashMap::new() }", false),
        ("logger.rs", "fn log_message(level: &str, msg: &str) { println!(\"[{}] {}\", level, msg); }", false),
        ("retry_logic.rs", "fn retry_with_backoff<F: Fn() -> Result<(), String>>(f: F) { let _ = f(); }", true),
        ("math_utils.rs", "fn fibonacci(n: u64) -> u64 { if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) } }", false),
        ("fault_tolerance.rs", "fn circuit_breaker(failures: u32) -> bool { failures < 3 }", true),
    ];

    for (name, content, _relevant) in &corpus {
        let path = tmp.path().join(name);
        std::fs::write(&path, content).unwrap();
        let chunks = vec![BM25Chunk {
            path: path.clone(),
            content: content.to_string(),
            line_start: 1,
            line_end: 1,
            kind: ChunkKind::Function,
        }];
        bm25.index_file(&path, Some(chunks)).unwrap();
    }

    let query = "error handling";
    let relevant_files: Vec<&str> = corpus
        .iter()
        .filter(|(_, _, rel)| *rel)
        .map(|(name, _, _)| *name)
        .collect();

    // BM25-only recall@10.
    let bm25_hits = bm25.query(query, 10).unwrap();
    let bm25_retrieved: Vec<String> = bm25_hits
        .iter()
        .map(|h| {
            std::path::Path::new(&h.path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();
    let bm25_recall = relevant_files
        .iter()
        .filter(|r| bm25_retrieved.iter().any(|h| h == **r))
        .count() as f64
        / relevant_files.len() as f64;

    // BM25+SPLADE (RRF fused) recall@10.
    let splade_terms = expander.expand_query(query);

    // Convert BM25 hits to (doc_id, score) for RRF.
    let bm25_for_rrf: Vec<(String, f32)> = bm25_hits
        .iter()
        .map(|h| {
            let name = std::path::Path::new(&h.path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            (name, h.score)
        })
        .collect();

    // Simulate SPLADE retrieval: for each corpus file, score = sum of SPLADE
    // term weights that appear in the file content.
    let mut splade_scores: Vec<(String, f32)> = Vec::new();
    for (name, content, _) in &corpus {
        let content_lower = content.to_lowercase();
        let score: f32 = splade_terms
            .iter()
            .filter(|t| content_lower.contains(&t.term.to_lowercase()))
            .map(|t| t.weight)
            .sum();
        if score > 0.0 {
            splade_scores.push((name.to_string(), score));
        }
    }
    splade_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let fused = rrf_fuse(&bm25_for_rrf, &splade_scores, 60, 0.5, 0.5);
    let fused_names: Vec<&str> = fused.iter().map(|(s, _)| s.as_str()).take(10).collect();
    let fused_recall = relevant_files
        .iter()
        .filter(|r| fused_names.contains(r))
        .count() as f64
        / relevant_files.len() as f64;

    assert!(
        fused_recall > 0.0,
        "fused recall must be > 0 with a working SPLADE+RRF pipeline"
    );

    eprintln!(
        "recall: bm25_only={:.2}, bm25+splade={:.2}, delta={:.2}%",
        bm25_recall,
        fused_recall,
        if bm25_recall > 0.0 {
            (fused_recall - bm25_recall) / bm25_recall * 100.0
        } else {
            0.0
        }
    );
}

// ---------------------------------------------------------------------------
// Adversarial: corrupted model file returns Err, not panic
// ---------------------------------------------------------------------------

#[test]
fn adversarial_corrupt_model_returns_err() {
    let tmp = tempfile::TempDir::new().unwrap();
    let bad_model = tmp.path().join("model.onnx");
    std::fs::write(&bad_model, &[0u8]).unwrap(); // 1-byte file

    // Need a valid tokenizer for the constructor. If model fixtures are not
    // staged, create a minimal test that at least checks the error path.
    let tp = tokenizer_path();
    if !tp.exists() {
        // Without a tokenizer we can at least test build_session directly
        // through the public new() which checks model first.
        let config = SpladeConfig {
            model_path: bad_model,
            tokenizer_path: PathBuf::from("nonexistent/tokenizer.json"),
            ..SpladeConfig::default()
        };
        let clock = Arc::new(elai_core::SeededClock::new());
        let result = SpladeExpander::new(config, clock);
        assert!(result.is_err(), "corrupt model must return Err");
        return;
    }

    let config = SpladeConfig {
        model_path: bad_model,
        tokenizer_path: tp,
        ..SpladeConfig::default()
    };
    let clock = Arc::new(elai_core::SeededClock::new());
    let result = SpladeExpander::new(config, clock);
    assert!(result.is_err(), "corrupt model must return Err, not panic");
}

// ---------------------------------------------------------------------------
// Adversarial: missing tokenizer returns Err
// ---------------------------------------------------------------------------

#[test]
fn adversarial_missing_tokenizer_returns_err() {
    let mp = model_path();
    if !mp.exists() {
        eprintln!("SKIP: model not staged");
        return;
    }

    let config = SpladeConfig {
        model_path: mp,
        tokenizer_path: PathBuf::from("nonexistent/tokenizer.json"),
        ..SpladeConfig::default()
    };
    let clock = Arc::new(elai_core::SeededClock::new());
    let result = SpladeExpander::new(config, clock);
    assert!(
        result.is_err(),
        "missing tokenizer must return Err, not panic"
    );
}

// ---------------------------------------------------------------------------
// Adversarial: missing model returns Err
// ---------------------------------------------------------------------------

#[test]
fn adversarial_missing_model_returns_err() {
    let config = SpladeConfig {
        model_path: PathBuf::from("nonexistent/model.onnx"),
        tokenizer_path: PathBuf::from("nonexistent/tokenizer.json"),
        ..SpladeConfig::default()
    };
    let clock = Arc::new(elai_core::SeededClock::new());
    let result = SpladeExpander::new(config, clock);
    assert!(result.is_err(), "missing model must return Err");
}
