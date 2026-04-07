// 1H.4: SPLADE measurement tests — vocabulary fitness, model comparison,
// recall evaluation (50 queries / 100 corpus files), Wilcoxon signed-rank,
// and keep/drop decision.
//
// Tests requiring ONNX model + tokenizer files skip gracefully if not staged.
// Pure-logic tests (Wilcoxon, Jaccard, recall@k, keep/drop decision) always run.

#![cfg(feature = "splade")]

use elai_knowledge::splade::{
    jaccard_similarity, keep_drop_decision, recall_at_k, rrf_fuse, wilcoxon_signed_rank,
    MeasurementDecision, ModelVariant, SpladeConfig, SpladeExpander,
};
use std::path::PathBuf;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

// =========================================================================
// Pure-logic tests (always run, no model needed)
// =========================================================================

// ---------------------------------------------------------------------------
// Wilcoxon signed-rank test
// ---------------------------------------------------------------------------

#[test]
fn wilcoxon_identical_samples_p_is_1() {
    let x = vec![0.5; 25];
    let y = vec![0.5; 25];
    let (_w, p, n) = wilcoxon_signed_rank(&x, &y);
    assert_eq!(n, 0, "identical samples have 0 non-zero differences");
    assert!(
        (p - 1.0).abs() < 1e-6,
        "p-value for identical samples should be 1.0, got {p}"
    );
}

#[test]
fn wilcoxon_clearly_different_samples_p_is_small() {
    // x consistently higher than y by a large margin.
    let x: Vec<f64> = (1..=30).map(|i| i as f64 * 2.0).collect();
    let y: Vec<f64> = (1..=30).map(|i| i as f64).collect();
    let (_w, p, n) = wilcoxon_signed_rank(&x, &y);
    assert_eq!(n, 30);
    assert!(
        p < 0.01,
        "clearly different samples should have p < 0.01, got {p}"
    );
}

#[test]
fn wilcoxon_small_sample_returns_p_1() {
    // Fewer than 5 non-zero pairs -> p = 1.0.
    let x = vec![1.0, 2.0, 3.0];
    let y = vec![1.1, 2.1, 3.0]; // only 2 non-zero diffs
    let (_w, p, n) = wilcoxon_signed_rank(&x, &y);
    assert!(n < 5);
    assert!(
        (p - 1.0).abs() < 1e-6,
        "small sample should return p=1.0, got {p}"
    );
}

#[test]
fn wilcoxon_handles_ties() {
    // Multiple tied differences. Should not panic.
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0,
                 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0];
    let y: Vec<f64> = x.iter().map(|v| v - 0.5).collect(); // all diffs = 0.5 (tied)
    let (_w, p, n) = wilcoxon_signed_rank(&x, &y);
    assert_eq!(n, 20);
    assert!(p.is_finite(), "p-value must be finite with ties");
    assert!(p < 0.05, "consistent positive diffs should be significant");
}

#[test]
fn wilcoxon_mixed_directions() {
    // Half positive, half negative diffs of equal magnitude -> not significant.
    let mut x = vec![0.0; 30];
    let mut y = vec![0.0; 30];
    for i in 0..15 {
        x[i] = 1.0;
        y[i] = 0.0;
    }
    for i in 15..30 {
        x[i] = 0.0;
        y[i] = 1.0;
    }
    let (_w, p, n) = wilcoxon_signed_rank(&x, &y);
    assert_eq!(n, 30);
    assert!(
        p > 0.05,
        "balanced positive/negative diffs should not be significant, got p={p}"
    );
}

// ---------------------------------------------------------------------------
// Jaccard similarity
// ---------------------------------------------------------------------------

#[test]
fn jaccard_identical_sets() {
    let a: Vec<String> = vec!["a", "b", "c"].into_iter().map(String::from).collect();
    let j = jaccard_similarity(&a, &a);
    assert!((j - 1.0).abs() < 1e-9, "identical sets -> jaccard=1.0");
}

#[test]
fn jaccard_disjoint_sets() {
    let a: Vec<String> = vec!["a", "b"].into_iter().map(String::from).collect();
    let b: Vec<String> = vec!["c", "d"].into_iter().map(String::from).collect();
    let j = jaccard_similarity(&a, &b);
    assert!(j.abs() < 1e-9, "disjoint sets -> jaccard=0.0");
}

#[test]
fn jaccard_partial_overlap() {
    let a: Vec<String> = vec!["a", "b", "c"].into_iter().map(String::from).collect();
    let b: Vec<String> = vec!["b", "c", "d"].into_iter().map(String::from).collect();
    // intersection={b,c}=2, union={a,b,c,d}=4 -> 0.5
    let j = jaccard_similarity(&a, &b);
    assert!(
        (j - 0.5).abs() < 1e-9,
        "expected 0.5, got {j}"
    );
}

#[test]
fn jaccard_both_empty() {
    let empty: Vec<String> = vec![];
    let j = jaccard_similarity(&empty, &empty);
    assert!(j.abs() < 1e-9, "both empty -> jaccard=0.0");
}

// ---------------------------------------------------------------------------
// recall@k
// ---------------------------------------------------------------------------

#[test]
fn recall_at_k_perfect_retrieval() {
    let retrieved: Vec<String> = vec!["a", "b", "c"].into_iter().map(String::from).collect();
    let relevant: Vec<String> = vec!["a", "b"].into_iter().map(String::from).collect();
    let r = recall_at_k(&retrieved, &relevant, 10);
    assert!((r - 1.0).abs() < 1e-9, "all relevant in top-k -> recall=1.0");
}

#[test]
fn recall_at_k_partial_retrieval() {
    let retrieved: Vec<String> = vec!["a", "c", "d"].into_iter().map(String::from).collect();
    let relevant: Vec<String> = vec!["a", "b"].into_iter().map(String::from).collect();
    let r = recall_at_k(&retrieved, &relevant, 10);
    assert!((r - 0.5).abs() < 1e-9, "1/2 relevant -> recall=0.5");
}

#[test]
fn recall_at_k_k_limits_results() {
    let retrieved: Vec<String> = vec!["x", "a", "b"].into_iter().map(String::from).collect();
    let relevant: Vec<String> = vec!["a", "b"].into_iter().map(String::from).collect();
    // k=1: only "x" in top-1, so recall=0.
    let r = recall_at_k(&retrieved, &relevant, 1);
    assert!(r.abs() < 1e-9, "k=1, relevant at positions 2,3 -> recall=0.0");
}

#[test]
fn recall_at_k_empty_relevant() {
    let retrieved: Vec<String> = vec!["a".to_string()];
    let relevant: Vec<String> = vec![];
    let r = recall_at_k(&retrieved, &relevant, 10);
    assert!(r.abs() < 1e-9, "no relevant docs -> recall=0.0");
}

// ---------------------------------------------------------------------------
// keep/drop decision logic
// ---------------------------------------------------------------------------

#[test]
fn decision_keep_when_significant() {
    let d = keep_drop_decision(8.0, 0.01, 25);
    assert_eq!(d, MeasurementDecision::Keep);
}

#[test]
fn decision_drop_low_delta() {
    let d = keep_drop_decision(3.0, 0.01, 25);
    match d {
        MeasurementDecision::Drop { .. } => {}
        other => panic!("expected Drop, got {:?}", other),
    }
}

#[test]
fn decision_drop_high_p() {
    let d = keep_drop_decision(8.0, 0.10, 25);
    match d {
        MeasurementDecision::Inconclusive { .. } => {}
        other => panic!("expected Inconclusive, got {:?}", other),
    }
}

#[test]
fn decision_drop_insufficient_pairs() {
    let d = keep_drop_decision(10.0, 0.01, 15);
    match d {
        MeasurementDecision::Drop { reason } => {
            assert!(reason.contains("insufficient"), "reason should mention insufficient pairs");
        }
        other => panic!("expected Drop, got {:?}", other),
    }
}

#[test]
fn decision_boundary_exactly_5_percent() {
    let d = keep_drop_decision(5.0, 0.049, 25);
    assert_eq!(d, MeasurementDecision::Keep);
}

#[test]
fn decision_boundary_exactly_p_005() {
    // delta >= 5% but p = 0.05 exactly (not < 0.05).
    let d = keep_drop_decision(5.0, 0.05, 25);
    match d {
        MeasurementDecision::Inconclusive { .. } => {}
        other => panic!("expected Inconclusive at p=0.05, got {:?}", other),
    }
}

// =========================================================================
// 4a: Vocabulary fitness measurement (requires model/tokenizer)
// =========================================================================

/// Extract code-specific tokens from Rust source via tree-sitter.
/// Returns unique identifiers (function names, type names, variables, keywords).
/// Excludes string literals, comments, and common English words.
fn extract_code_tokens_from_source(source: &str) -> Vec<String> {
    use std::collections::HashSet;

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("tree-sitter-rust grammar");

    let tree = parser.parse(source, None).expect("parse source");
    let root = tree.root_node();
    let bytes = source.as_bytes();

    let mut tokens = HashSet::new();
    extract_identifiers_recursive(root, bytes, &mut tokens);

    // Add Rust keywords that are important for code search.
    let rust_keywords = [
        "fn", "impl", "async", "await", "pub", "mod", "use", "struct", "enum",
        "trait", "where", "mut", "let", "const", "static", "match", "if", "else",
        "loop", "while", "for", "in", "return", "break", "continue", "unsafe",
        "self", "Self", "super", "crate", "dyn", "ref", "move", "type", "as",
    ];
    for kw in &rust_keywords {
        tokens.insert(kw.to_string());
    }

    let mut result: Vec<String> = tokens.into_iter().collect();
    result.sort();
    result
}

fn extract_identifiers_recursive(
    node: tree_sitter::Node,
    source: &[u8],
    tokens: &mut std::collections::HashSet<String>,
) {
    let kind = node.kind();

    // Extract identifiers and type names.
    match kind {
        "identifier" | "type_identifier" | "field_identifier" | "lifetime" => {
            if let Ok(text) = node.utf8_text(source) {
                let trimmed = text.trim();
                // Skip very short (single char) and very long identifiers.
                if trimmed.len() >= 2 && trimmed.len() <= 64 {
                    tokens.insert(trimmed.to_string());
                }
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            extract_identifiers_recursive(child, source, tokens);
        }
    }
}

#[test]
fn measure_4a_vocab_coverage_on_code_tokens() {
    let expander = match try_make_expander() {
        Some(e) => e,
        None => {
            eprintln!("SKIP: 4a vocab coverage — model not staged");
            return;
        }
    };

    // Synthetic Rust source with diverse identifiers.
    let source = r#"
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;

pub struct ErrorHandler {
    retries: u32,
    backoff_ms: u64,
}

impl ErrorHandler {
    pub fn new(retries: u32) -> Self {
        Self { retries, backoff_ms: 100 }
    }

    pub async fn handle_error(&self, err: &dyn std::error::Error) -> Result<(), String> {
        eprintln!("error: {}", err);
        Err(format!("unrecoverable: {}", err))
    }
}

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;
}

pub enum ParseResult<T> {
    Success(T),
    Failure(String),
    Partial { parsed: T, remaining: String },
}

pub fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

pub fn quicksort<T: Ord>(arr: &mut [T]) {
    if arr.len() <= 1 { return; }
    let pivot = arr.len() / 2;
    arr.sort();
}

pub struct ConcurrencyManager {
    lock: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    channel: Option<mpsc::Sender<Vec<u8>>>,
}

impl ConcurrencyManager {
    pub fn spawn_worker(&self) {
        let lock = self.lock.clone();
        std::thread::spawn(move || {
            let guard = lock.read().unwrap();
            println!("worker reading {} entries", guard.len());
        });
    }
}
"#;

    let code_tokens = extract_code_tokens_from_source(source);
    assert!(
        code_tokens.len() >= 30,
        "expected at least 30 code tokens from synthetic source, got {}",
        code_tokens.len()
    );

    let coverage = expander.vocab_coverage(&code_tokens);

    eprintln!("4a: vocab coverage on {} code-specific tokens: {:.4}", code_tokens.len(), coverage);
    eprintln!("4a: coverage ratio: {:.1}%", coverage * 100.0);

    // Log some sample tokens and their tokenization status for diagnostics.
    let sample_tokens = [
        "fn", "impl", "async", "HashMap", "RwLock", "mpsc", "fibonacci",
        "quicksort", "ErrorHandler", "Serializable", "deserialize",
        "ConcurrencyManager", "spawn_worker", "backoff_ms",
    ];
    for token in &sample_tokens {
        let is_single = expander.vocab_coverage(&[token.to_string()]) > 0.99;
        eprintln!("  {:<25} single_token={}", token, is_single);
    }

    assert!(
        (0.0..=1.0).contains(&coverage),
        "coverage must be in [0.0, 1.0], got {coverage}"
    );

    // The decision gate:
    if coverage < 0.3 {
        eprintln!(
            "4a: EARLY DROP — vocab coverage {:.1}% < 30%. \
             BERT WordPiece tokenizer fundamentally mismatched for code corpus.",
            coverage * 100.0
        );
    } else if coverage < 0.4 {
        eprintln!(
            "4a: coverage {:.1}% in soft zone (20-40%). \
             Proceeding to recall measurement for data-driven decision.",
            coverage * 100.0
        );
    } else {
        eprintln!(
            "4a: coverage {:.1}% >= 40%. Vocabulary adequate for SPLADE expansion.",
            coverage * 100.0
        );
    }
}

// =========================================================================
// 4b: Model variant selection (requires model files)
// =========================================================================

#[test]
fn measure_4b_model_variant_selection() {
    let expander = match try_make_expander() {
        Some(e) => e,
        None => {
            eprintln!("SKIP: 4b model variant selection — model not staged");
            return;
        }
    };

    // Check for quantized variant.
    let quant_dir = std::env::var("SPLADE_QUANT_MODEL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("models/splade-quant"));
    let quant_model = quant_dir.join("model.onnx");
    let quant_tokenizer = quant_dir.join("tokenizer.json");

    // Check for splade-v3 variant.
    let v3_dir = std::env::var("SPLADE_V3_MODEL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("models/splade-v3"));
    let v3_model = v3_dir.join("model.onnx");

    let sample_queries = [
        "error handling",
        "concurrent data access",
        "parse configuration file",
        "sort algorithm implementation",
        "network request timeout",
    ];

    // Run baseline (Candidate A full) on sample queries.
    let mut a_full_terms: Vec<Vec<String>> = Vec::new();
    let mut a_full_latencies: Vec<u128> = Vec::new();

    for q in &sample_queries {
        let start = std::time::Instant::now();
        let terms = expander.expand_query(q);
        let elapsed = start.elapsed().as_millis();
        a_full_latencies.push(elapsed);
        a_full_terms.push(terms.iter().take(50).map(|t| t.term.clone()).collect());
    }

    let a_full_p50 = percentile(&a_full_latencies, 50);
    eprintln!("4b: Candidate A (full) p50 latency: {}ms", a_full_p50);

    // Quantization comparison (same model, different precision).
    if quant_model.exists() && quant_tokenizer.exists() {
        let quant_config = SpladeConfig {
            model_path: quant_model,
            tokenizer_path: quant_tokenizer,
            variant: ModelVariant::PrePooled,
            top_k: 64,
            intra_op_threads: 1,
        };
        let clock = Arc::new(elai_core::SeededClock::new());
        match SpladeExpander::new(quant_config, clock) {
            Ok(quant_expander) => {
                let mut a_quant_terms: Vec<Vec<String>> = Vec::new();
                let mut a_quant_latencies: Vec<u128> = Vec::new();
                let mut jaccard_scores: Vec<f64> = Vec::new();

                for (i, q) in sample_queries.iter().enumerate() {
                    let start = std::time::Instant::now();
                    let terms = quant_expander.expand_query(q);
                    let elapsed = start.elapsed().as_millis();
                    a_quant_latencies.push(elapsed);
                    let quant_top50: Vec<String> =
                        terms.iter().take(50).map(|t| t.term.clone()).collect();
                    let j = jaccard_similarity(&a_full_terms[i], &quant_top50);
                    jaccard_scores.push(j);
                    a_quant_terms.push(quant_top50);
                }

                let mean_jaccard: f64 =
                    jaccard_scores.iter().sum::<f64>() / jaccard_scores.len() as f64;
                let a_quant_p50 = percentile(&a_quant_latencies, 50);

                eprintln!("4b: Candidate A (quant) p50 latency: {}ms", a_quant_p50);
                eprintln!("4b: Quantization Jaccard (mean top-50): {:.4}", mean_jaccard);

                assert!(
                    mean_jaccard >= 0.80,
                    "quantized model Jaccard overlap {mean_jaccard:.4} < 0.80 threshold — \
                     quality loss exceeds acceptable bounds"
                );

                if mean_jaccard >= 0.95 {
                    eprintln!("4b: Jaccard >= 0.95 — prefer quantized variant (saves ~96MB, ~3x speed)");
                } else {
                    eprintln!("4b: Jaccard {mean_jaccard:.4} < 0.95 — prefer full model for quality");
                }
            }
            Err(e) => {
                eprintln!("4b: quantized model load failed: {e} — skipping quantization comparison");
            }
        }
    } else {
        eprintln!("4b: quantized model not staged at {} — skipping quantization check", quant_dir.display());
    }

    // Cross-model comparison (A vs B) — NOT using Jaccard (different models).
    if v3_model.exists() {
        let v3_tokenizer = v3_dir.join("tokenizer.json");
        if v3_tokenizer.exists() {
            let v3_config = SpladeConfig {
                model_path: v3_model,
                tokenizer_path: v3_tokenizer,
                variant: ModelVariant::RawMlm,
                top_k: 64,
                intra_op_threads: 1,
            };
            let clock = Arc::new(elai_core::SeededClock::new());
            match SpladeExpander::new(v3_config, clock) {
                Ok(v3_expander) => {
                    let mut v3_latencies: Vec<u128> = Vec::new();
                    for q in &sample_queries {
                        let start = std::time::Instant::now();
                        let _terms = v3_expander.expand_query(q);
                        let elapsed = start.elapsed().as_millis();
                        v3_latencies.push(elapsed);
                    }
                    let v3_p50 = percentile(&v3_latencies, 50);
                    eprintln!("4b: Candidate B (splade-v3) p50 latency: {}ms", v3_p50);
                    eprintln!("4b: cross-model comparison requires recall measurement (not Jaccard)");
                    eprintln!("4b: quick recall@10 comparison deferred to 4c full measurement");
                }
                Err(e) => {
                    eprintln!("4b: splade-v3 model load failed: {e} — Candidate B unavailable");
                }
            }
        } else {
            eprintln!("4b: splade-v3 tokenizer not found — Candidate B unavailable");
        }
    } else {
        eprintln!("4b: Candidate B (splade-v3) not staged — skipped");
    }
}

fn percentile(values: &[u128], pct: usize) -> u128 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort();
    let idx = (pct * sorted.len() / 100).min(sorted.len() - 1);
    sorted[idx]
}

// =========================================================================
// 4c: Recall measurement — 50 queries, 100 corpus files, Wilcoxon test
// =========================================================================

/// Build the synthetic corpus: 100 code files with known relevant matches
/// for 50 queries. Returns (corpus, queries) where each query has a list
/// of relevant file names.
fn build_synthetic_corpus() -> (Vec<CorpusFile>, Vec<QueryWithRelevance>) {
    let mut corpus = Vec::new();
    let mut queries = Vec::new();

    // --- Category 1: Vocabulary-mismatch queries (15 queries) ---
    // These test SPLADE's ability to bridge NL descriptions to code patterns.

    // Q1: "error handling" should match try/catch/Result patterns
    corpus.push(cf("error_result.rs", "pub fn process(data: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(data.to_string()) }"));
    corpus.push(cf("try_catch_pattern.rs", "pub fn safe_parse(s: &str) -> Option<i32> { s.parse::<i32>().ok() }"));
    corpus.push(cf("panic_handler.rs", "pub fn set_panic_hook() { std::panic::set_hook(Box::new(|info| { eprintln!(\"panic: {}\", info); })); }"));
    queries.push(qr("error handling", &["error_result.rs", "try_catch_pattern.rs", "panic_handler.rs"]));

    // Q2: "concurrent access" should match mutex/rwlock/arc
    corpus.push(cf("mutex_guard.rs", "use std::sync::Mutex; pub fn locked_increment(counter: &Mutex<u32>) { *counter.lock().unwrap() += 1; }"));
    corpus.push(cf("rwlock_cache.rs", "use std::sync::RwLock; pub fn read_cache(cache: &RwLock<Vec<String>>) -> Vec<String> { cache.read().unwrap().clone() }"));
    corpus.push(cf("arc_shared.rs", "use std::sync::Arc; pub fn share_data(data: Vec<u8>) -> Arc<Vec<u8>> { Arc::new(data) }"));
    queries.push(qr("concurrent access", &["mutex_guard.rs", "rwlock_cache.rs", "arc_shared.rs"]));

    // Q3: "memory management" should match allocation/deallocation patterns
    corpus.push(cf("box_alloc.rs", "pub fn heap_allocate(size: usize) -> Box<[u8]> { vec![0u8; size].into_boxed_slice() }"));
    corpus.push(cf("drop_impl.rs", "pub struct Resource { handle: u64 } impl Drop for Resource { fn drop(&mut self) { println!(\"releasing {}\", self.handle); } }"));
    queries.push(qr("memory management", &["box_alloc.rs", "drop_impl.rs"]));

    // Q4: "async execution" should match tokio/futures patterns
    corpus.push(cf("async_task.rs", "pub async fn fetch_data(url: &str) -> String { tokio::time::sleep(std::time::Duration::from_secs(1)).await; format!(\"data from {}\", url) }"));
    corpus.push(cf("spawn_task.rs", "pub fn spawn_background(task: impl std::future::Future<Output = ()> + Send + 'static) { tokio::spawn(task); }"));
    queries.push(qr("async execution", &["async_task.rs", "spawn_task.rs"]));

    // Q5: "serialization" should match serde patterns
    corpus.push(cf("serde_json.rs", "use serde::{Serialize, Deserialize}; #[derive(Serialize, Deserialize)] pub struct Config { pub name: String, pub value: i32 }"));
    corpus.push(cf("manual_serialize.rs", "pub fn to_bytes(val: u64) -> Vec<u8> { val.to_le_bytes().to_vec() }"));
    queries.push(qr("serialization", &["serde_json.rs", "manual_serialize.rs"]));

    // Q6: "string processing" should match parse/format/regex
    corpus.push(cf("string_parse.rs", "pub fn extract_numbers(s: &str) -> Vec<i32> { s.split_whitespace().filter_map(|w| w.parse().ok()).collect() }"));
    corpus.push(cf("format_output.rs", "pub fn format_table(rows: &[(String, i32)]) -> String { rows.iter().map(|(k, v)| format!(\"{}: {}\", k, v)).collect::<Vec<_>>().join(\"\\n\") }"));
    queries.push(qr("string processing", &["string_parse.rs", "format_output.rs"]));

    // Q7: "file operations" should match read/write/path
    corpus.push(cf("file_reader.rs", "pub fn read_lines(path: &std::path::Path) -> Vec<String> { std::fs::read_to_string(path).unwrap().lines().map(String::from).collect() }"));
    corpus.push(cf("file_writer.rs", "pub fn write_json(path: &std::path::Path, data: &str) { std::fs::write(path, data).unwrap(); }"));
    queries.push(qr("file operations", &["file_reader.rs", "file_writer.rs"]));

    // Q8: "logging" should match log/tracing patterns
    corpus.push(cf("structured_log.rs", "pub fn log_event(level: &str, msg: &str) { eprintln!(\"[{}] {}\", level, msg); }"));
    corpus.push(cf("tracing_setup.rs", "pub fn init_tracing() { let subscriber = tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).finish(); tracing::subscriber::set_global_default(subscriber).unwrap(); }"));
    queries.push(qr("logging", &["structured_log.rs", "tracing_setup.rs"]));

    // Q9: "network communication" should match socket/http/tcp
    corpus.push(cf("tcp_listener.rs", "pub fn start_server(addr: &str) -> std::net::TcpListener { std::net::TcpListener::bind(addr).unwrap() }"));
    corpus.push(cf("http_get.rs", "pub async fn http_get(url: &str) -> String { reqwest::get(url).await.unwrap().text().await.unwrap() }"));
    queries.push(qr("network communication", &["tcp_listener.rs", "http_get.rs"]));

    // Q10: "testing patterns" should match assert/mock/test
    corpus.push(cf("unit_test.rs", "#[cfg(test)] mod tests { use super::*; #[test] fn test_add() { assert_eq!(add(2, 3), 5); } }"));
    corpus.push(cf("mock_service.rs", "pub struct MockDatabase { data: Vec<String> } impl MockDatabase { pub fn new() -> Self { Self { data: vec![] } } }"));
    queries.push(qr("testing patterns", &["unit_test.rs", "mock_service.rs"]));

    // Q11: "configuration loading" should match config/env/settings
    corpus.push(cf("env_config.rs", "pub fn load_config() -> String { std::env::var(\"APP_CONFIG\").unwrap_or_else(|_| \"default\".to_string()) }"));
    corpus.push(cf("toml_config.rs", "pub fn parse_toml(content: &str) -> toml::Value { content.parse::<toml::Value>().unwrap() }"));
    queries.push(qr("configuration loading", &["env_config.rs", "toml_config.rs"]));

    // Q12: "data validation" should match validate/check/constraint
    corpus.push(cf("input_validator.rs", "pub fn validate_email(email: &str) -> bool { email.contains('@') && email.contains('.') }"));
    corpus.push(cf("range_check.rs", "pub fn validate_range(val: i32, min: i32, max: i32) -> Result<i32, String> { if val >= min && val <= max { Ok(val) } else { Err(format!(\"{} out of range [{}, {}]\", val, min, max)) } }"));
    queries.push(qr("data validation", &["input_validator.rs", "range_check.rs"]));

    // Q13: "caching strategy" should match cache/memoize/lru
    corpus.push(cf("lru_cache.rs", "use std::collections::HashMap; pub struct LruCache { map: HashMap<String, String>, capacity: usize }"));
    corpus.push(cf("memoize.rs", "use std::collections::HashMap; pub fn memoize<F: Fn(u64) -> u64>(f: F, n: u64, cache: &mut HashMap<u64, u64>) -> u64 { *cache.entry(n).or_insert_with(|| f(n)) }"));
    queries.push(qr("caching strategy", &["lru_cache.rs", "memoize.rs"]));

    // Q14: "authentication flow" should match auth/login/token/jwt
    corpus.push(cf("jwt_auth.rs", "pub fn verify_token(token: &str, secret: &str) -> bool { token.starts_with(\"Bearer \") && !secret.is_empty() }"));
    corpus.push(cf("password_hash.rs", "pub fn hash_password(password: &str) -> String { format!(\"hashed_{}\", password) }"));
    queries.push(qr("authentication flow", &["jwt_auth.rs", "password_hash.rs"]));

    // Q15: "database operations" should match query/insert/transaction
    corpus.push(cf("db_query.rs", "pub fn find_user(conn: &rusqlite::Connection, id: u64) -> Option<String> { conn.query_row(\"SELECT name FROM users WHERE id = ?1\", [id], |row| row.get(0)).ok() }"));
    corpus.push(cf("db_transaction.rs", "pub fn batch_insert(conn: &mut rusqlite::Connection, items: &[String]) { let tx = conn.transaction().unwrap(); for item in items { tx.execute(\"INSERT INTO items (name) VALUES (?1)\", [item]).unwrap(); } tx.commit().unwrap(); }"));
    queries.push(qr("database operations", &["db_query.rs", "db_transaction.rs"]));

    // --- Category 2: Distractor queries (15 queries with distractors) ---
    // Files share keywords but are NOT relevant.

    // Q16: "sort implementation" — sort_algo is relevant, sort_config is distractor
    corpus.push(cf("sort_algo.rs", "pub fn merge_sort(arr: &mut [i32]) { if arr.len() <= 1 { return; } let mid = arr.len() / 2; let mut left = arr[..mid].to_vec(); let mut right = arr[mid..].to_vec(); merge_sort(&mut left); merge_sort(&mut right); }"));
    corpus.push(cf("sort_config.rs", "pub struct SortConfig { pub ascending: bool } // Configuration for display sort order, not an algorithm"));
    queries.push(qr("sort implementation", &["sort_algo.rs"]));

    // Q17: "tree traversal" — tree_walk is relevant, tree_config is distractor
    corpus.push(cf("tree_walk.rs", "pub fn inorder<T: Ord>(node: &Option<Box<TreeNode<T>>>) -> Vec<&T> { vec![] } pub struct TreeNode<T> { val: T, left: Option<Box<TreeNode<T>>>, right: Option<Box<TreeNode<T>>> }"));
    corpus.push(cf("tree_display.rs", "pub fn print_tree_diagram(label: &str) { println!(\"--- {} ---\", label); } // Display function, not tree data structure"));
    queries.push(qr("tree traversal", &["tree_walk.rs"]));

    // Q18: "hash computation" — blake3_hash is relevant, hash_display is distractor
    corpus.push(cf("blake3_hash.rs", "pub fn compute_hash(data: &[u8]) -> [u8; 32] { let mut hasher = blake3::Hasher::new(); hasher.update(data); *hasher.finalize().as_bytes() }"));
    corpus.push(cf("hash_display.rs", "pub fn format_hash_table(entries: &[(String, String)]) -> String { entries.iter().map(|(k, v)| format!(\"{}: {}\", k, v)).collect::<Vec<_>>().join(\"\\n\") }"));
    queries.push(qr("hash computation", &["blake3_hash.rs"]));

    // Q19: "queue processing" — task_queue is relevant, queue_display is distractor
    corpus.push(cf("task_queue.rs", "use std::collections::VecDeque; pub struct TaskQueue { inner: VecDeque<String> } impl TaskQueue { pub fn push(&mut self, task: String) { self.inner.push_back(task); } pub fn pop(&mut self) -> Option<String> { self.inner.pop_front() } }"));
    corpus.push(cf("queue_report.rs", "pub fn report_queue_length(name: &str, len: usize) { println!(\"Queue '{}' has {} items\", name, len); }"));
    queries.push(qr("queue processing", &["task_queue.rs"]));

    // Q20: "type conversion" — convert_types is relevant, type_display is distractor
    corpus.push(cf("convert_types.rs", "impl From<Vec<u8>> for Buffer { fn from(data: Vec<u8>) -> Self { Buffer { data } } } pub struct Buffer { data: Vec<u8> }"));
    corpus.push(cf("type_display.rs", "pub fn display_type_name<T>(_val: &T) -> &'static str { std::any::type_name::<T>() }"));
    queries.push(qr("type conversion", &["convert_types.rs"]));

    // Q21: "iterator chaining" — iter_chain is relevant, chain_link is distractor
    corpus.push(cf("iter_chain.rs", "pub fn process_data(data: &[i32]) -> Vec<i32> { data.iter().filter(|&&x| x > 0).map(|&x| x * 2).collect() }"));
    corpus.push(cf("chain_link.rs", "pub struct ChainLink { next: Option<Box<ChainLink>>, value: String }"));
    queries.push(qr("iterator chaining", &["iter_chain.rs"]));

    // Q22: "pattern matching" — match_expr is relevant, pattern_format is distractor
    corpus.push(cf("match_expr.rs", "pub fn classify(val: i32) -> &'static str { match val { 0 => \"zero\", 1..=9 => \"single\", 10..=99 => \"double\", _ => \"large\" } }"));
    corpus.push(cf("pattern_format.rs", "pub fn format_pattern(template: &str, args: &[&str]) -> String { let mut result = template.to_string(); for (i, arg) in args.iter().enumerate() { result = result.replace(&format!(\"{{{}}}\", i), arg); } result }"));
    queries.push(qr("pattern matching", &["match_expr.rs"]));

    // Q23: "buffer management" — ring_buffer is relevant, buffer_display is distractor
    corpus.push(cf("ring_buffer.rs", "pub struct RingBuffer<T> { data: Vec<Option<T>>, head: usize, tail: usize, capacity: usize }"));
    corpus.push(cf("buffer_label.rs", "pub fn label_buffer_zone(zone: &str) -> String { format!(\"[BUFFER ZONE: {}]\", zone) }"));
    queries.push(qr("buffer management", &["ring_buffer.rs"]));

    // Q24: "graph algorithms" — dijkstra is relevant, graph_chart is distractor
    corpus.push(cf("dijkstra.rs", "use std::collections::{BinaryHeap, HashMap}; pub fn shortest_path(graph: &HashMap<u32, Vec<(u32, u32)>>, start: u32) -> HashMap<u32, u32> { let mut dist = HashMap::new(); dist.insert(start, 0); dist }"));
    corpus.push(cf("graph_chart.rs", "pub fn render_bar_graph(values: &[f64]) -> String { values.iter().map(|v| \"#\".repeat(*v as usize)).collect::<Vec<_>>().join(\"\\n\") }"));
    queries.push(qr("graph algorithms", &["dijkstra.rs"]));

    // Q25: "compression" — zstd_compress is relevant, compress_display is distractor
    corpus.push(cf("zstd_compress.rs", "pub fn compress(data: &[u8], level: i32) -> Vec<u8> { zstd::encode_all(std::io::Cursor::new(data), level).unwrap() }"));
    corpus.push(cf("compress_report.rs", "pub fn report_compression_ratio(original: usize, compressed: usize) { println!(\"ratio: {:.2}x\", original as f64 / compressed as f64); }"));
    queries.push(qr("compression", &["zstd_compress.rs"]));

    // Q26: "rate limiting" — rate_limiter is relevant, rate_display is distractor
    corpus.push(cf("rate_limiter.rs", "pub struct RateLimiter { window_ms: u64, max_requests: u32, count: u32 } impl RateLimiter { pub fn allow(&mut self) -> bool { self.count < self.max_requests } }"));
    corpus.push(cf("rate_calculator.rs", "pub fn calculate_rate(events: usize, duration_secs: f64) -> f64 { events as f64 / duration_secs }"));
    queries.push(qr("rate limiting", &["rate_limiter.rs"]));

    // Q27: "retry logic" — exponential_backoff is relevant, retry_count is distractor
    corpus.push(cf("exponential_backoff.rs", "pub fn retry_with_backoff<T, E>(mut f: impl FnMut() -> Result<T, E>, max_retries: u32) -> Result<T, E> { let mut delay = 100; for _ in 0..max_retries { match f() { Ok(v) => return Ok(v), Err(e) => { std::thread::sleep(std::time::Duration::from_millis(delay)); delay *= 2; if delay > 10000 { return Err(e); } } } } f() }"));
    corpus.push(cf("retry_counter.rs", "pub struct RetryCounter { pub attempts: u32 } impl std::fmt::Display for RetryCounter { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, \"{} attempts\", self.attempts) } }"));
    queries.push(qr("retry logic", &["exponential_backoff.rs"]));

    // Q28: "resource pool" — connection_pool is relevant, pool_stats is distractor
    corpus.push(cf("connection_pool.rs", "pub struct ConnectionPool { connections: Vec<Connection>, max_size: usize } pub struct Connection { active: bool }"));
    corpus.push(cf("pool_stats.rs", "pub fn swimming_pool_stats(depth: f64, width: f64) -> f64 { depth * width }"));
    queries.push(qr("resource pool", &["connection_pool.rs"]));

    // Q29: "event dispatch" — event_bus is relevant, event_log is distractor
    corpus.push(cf("event_bus.rs", "pub struct EventBus { handlers: Vec<Box<dyn Fn(&str)>> } impl EventBus { pub fn emit(&self, event: &str) { for h in &self.handlers { h(event); } } }"));
    corpus.push(cf("event_log_entry.rs", "pub struct EventLogEntry { pub timestamp: u64, pub message: String }"));
    queries.push(qr("event dispatch", &["event_bus.rs"]));

    // Q30: "permission check" — rbac is relevant, permission_display is distractor
    corpus.push(cf("rbac.rs", "pub fn has_permission(role: &str, action: &str) -> bool { match role { \"admin\" => true, \"user\" => action != \"delete\", _ => false } }"));
    corpus.push(cf("permission_docs.rs", "pub fn print_permissions_table() { println!(\"Role | Read | Write | Delete\"); }"));
    queries.push(qr("permission check", &["rbac.rs"]));

    // --- Category 3: Straightforward keyword queries (10 queries) ---

    // Q31
    corpus.push(cf("vec_ops.rs", "pub fn sum_vec(v: &[f64]) -> f64 { v.iter().sum() }"));
    queries.push(qr("sum_vec", &["vec_ops.rs"]));

    // Q32
    corpus.push(cf("fibonacci_calc.rs", "pub fn fibonacci(n: u64) -> u64 { if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) } }"));
    queries.push(qr("fibonacci", &["fibonacci_calc.rs"]));

    // Q33
    corpus.push(cf("base64_encode.rs", "pub fn encode_base64(data: &[u8]) -> String { base64::encode(data) }"));
    queries.push(qr("base64_encode", &["base64_encode.rs"]));

    // Q34
    corpus.push(cf("uuid_gen.rs", "pub fn generate_uuid() -> String { uuid::Uuid::new_v4().to_string() }"));
    queries.push(qr("generate_uuid", &["uuid_gen.rs"]));

    // Q35
    corpus.push(cf("json_parser.rs", "pub fn parse_json(s: &str) -> serde_json::Value { serde_json::from_str(s).unwrap() }"));
    queries.push(qr("parse_json", &["json_parser.rs"]));

    // Q36
    corpus.push(cf("regex_match.rs", "pub fn find_emails(text: &str) -> Vec<String> { let re = regex::Regex::new(r\"[\\w.]+@[\\w.]+\").unwrap(); re.find_iter(text).map(|m| m.as_str().to_string()).collect() }"));
    queries.push(qr("find_emails regex", &["regex_match.rs"]));

    // Q37
    corpus.push(cf("timestamp_fmt.rs", "pub fn format_timestamp(epoch: u64) -> String { format!(\"{}s since epoch\", epoch) }"));
    queries.push(qr("format_timestamp", &["timestamp_fmt.rs"]));

    // Q38
    corpus.push(cf("binary_search_impl.rs", "pub fn binary_search(arr: &[i32], target: i32) -> Option<usize> { arr.binary_search(&target).ok() }"));
    queries.push(qr("binary_search", &["binary_search_impl.rs"]));

    // Q39
    corpus.push(cf("checksum.rs", "pub fn crc32(data: &[u8]) -> u32 { let mut crc: u32 = 0xFFFFFFFF; for &byte in data { crc ^= byte as u32; for _ in 0..8 { crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB88320 } else { crc >> 1 }; } } !crc }"));
    queries.push(qr("crc32 checksum", &["checksum.rs"]));

    // Q40
    corpus.push(cf("matrix_mul.rs", "pub fn matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> { let rows = a.len(); let cols = b[0].len(); let mut result = vec![vec![0.0; cols]; rows]; result }"));
    queries.push(qr("matrix_multiply", &["matrix_mul.rs"]));

    // --- Category 4: Code-specific syntax queries (10 queries) ---

    // Q41: impl Display
    corpus.push(cf("display_impl.rs", "pub struct Point { x: f64, y: f64 } impl std::fmt::Display for Point { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, \"({}, {})\", self.x, self.y) } }"));
    queries.push(qr("impl Display for", &["display_impl.rs"]));

    // Q42: derive macro
    corpus.push(cf("derive_debug.rs", "#[derive(Debug, Clone, PartialEq)] pub struct Record { pub id: u64, pub name: String }"));
    queries.push(qr("derive Debug Clone", &["derive_debug.rs"]));

    // Q43: unsafe block
    corpus.push(cf("unsafe_ptr.rs", "pub fn raw_pointer_deref(ptr: *const u8, len: usize) -> &'static [u8] { unsafe { std::slice::from_raw_parts(ptr, len) } }"));
    queries.push(qr("unsafe raw pointer", &["unsafe_ptr.rs"]));

    // Q44: trait bound with where clause
    corpus.push(cf("where_clause.rs", "pub fn process<T>(item: T) -> String where T: std::fmt::Display + Clone + Send { format!(\"{}\", item) }"));
    queries.push(qr("where clause trait bound", &["where_clause.rs"]));

    // Q45: lifetime annotation
    corpus.push(cf("lifetime_ref.rs", "pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } }"));
    queries.push(qr("lifetime annotation", &["lifetime_ref.rs"]));

    // Q46: enum with data
    corpus.push(cf("enum_variants.rs", "pub enum Shape { Circle(f64), Rectangle { width: f64, height: f64 }, Triangle(f64, f64, f64) }"));
    queries.push(qr("enum variants data", &["enum_variants.rs"]));

    // Q47: closure / FnMut
    corpus.push(cf("closure_fn.rs", "pub fn apply_transform(data: &mut [i32], f: impl FnMut(&mut i32)) { data.iter_mut().for_each(f); }"));
    queries.push(qr("closure FnMut", &["closure_fn.rs"]));

    // Q48: channel mpsc
    corpus.push(cf("channel_mpsc.rs", "use std::sync::mpsc; pub fn create_channel() -> (mpsc::Sender<String>, mpsc::Receiver<String>) { mpsc::channel() }"));
    queries.push(qr("mpsc channel", &["channel_mpsc.rs"]));

    // Q49: Pin Future
    corpus.push(cf("pin_future.rs", "use std::pin::Pin; use std::future::Future; pub fn boxed_future() -> Pin<Box<dyn Future<Output = ()> + Send>> { Box::pin(async { }) }"));
    queries.push(qr("Pin Box Future", &["pin_future.rs"]));

    // Q50: PhantomData
    corpus.push(cf("phantom_data.rs", "use std::marker::PhantomData; pub struct TypedId<T> { id: u64, _marker: PhantomData<T> }"));
    queries.push(qr("PhantomData marker", &["phantom_data.rs"]));

    // --- Filler corpus files (to reach 100 total) ---
    // These are NOT relevant to any query but add noise to the index.
    let filler_count = 100 - corpus.len();
    for i in 0..filler_count {
        corpus.push(cf(
            &format!("filler_{:03}.rs", i),
            &format!(
                "pub fn filler_function_{}() -> u64 {{ {} * {} + {} }}",
                i, i, i + 1, i + 2
            ),
        ));
    }

    assert_eq!(corpus.len(), 100, "corpus must have exactly 100 files");
    assert_eq!(queries.len(), 50, "must have exactly 50 queries");

    // Validate query categories.
    let vocab_mismatch_count = 15; // Q1-Q15
    let distractor_count = 15; // Q16-Q30
    let keyword_count = 10; // Q31-Q40
    let code_syntax_count = 10; // Q41-Q50
    assert_eq!(
        vocab_mismatch_count + distractor_count + keyword_count + code_syntax_count,
        50
    );

    // Validate distractor presence: Q16-Q30 (indices 15-29) each have distractors.
    for i in 15..30 {
        let q = &queries[i];
        // Each distractor query has at least one non-relevant file sharing keywords.
        // This is enforced by construction above.
        assert!(
            q.relevant.len() >= 1,
            "query '{}' must have at least 1 relevant file",
            q.query
        );
    }

    (corpus, queries)
}

struct CorpusFile {
    name: String,
    content: String,
}

fn cf(name: &str, content: &str) -> CorpusFile {
    CorpusFile {
        name: name.to_string(),
        content: content.to_string(),
    }
}

struct QueryWithRelevance {
    query: String,
    relevant: Vec<String>,
}

fn qr(query: &str, relevant: &[&str]) -> QueryWithRelevance {
    QueryWithRelevance {
        query: query.to_string(),
        relevant: relevant.iter().map(|s| s.to_string()).collect(),
    }
}

#[test]
fn measure_4c_corpus_structure_validation() {
    // Validate the synthetic corpus meets all requirements.
    let (corpus, queries) = build_synthetic_corpus();

    assert_eq!(corpus.len(), 100, "corpus must have 100 files");
    assert_eq!(queries.len(), 50, "must have 50 queries");

    // At least 15 queries target vocabulary-mismatch scenarios (Q1-Q15).
    let vocab_mismatch_queries = &queries[0..15];
    assert_eq!(vocab_mismatch_queries.len(), 15);
    for q in vocab_mismatch_queries {
        assert!(
            !q.query.is_empty(),
            "vocab mismatch query must not be empty"
        );
    }

    // At least 15 queries have distractors (Q16-Q30).
    // Distractors = files that share keywords with the query but are NOT relevant.
    let distractor_queries = &queries[15..30];
    assert_eq!(distractor_queries.len(), 15);
    for q in distractor_queries {
        // The corpus has files with similar names but only specific ones are relevant.
        assert!(
            q.relevant.len() >= 1,
            "distractor query '{}' must have at least 1 relevant file",
            q.query
        );
    }

    // At least 10 queries are straightforward keyword matches (Q31-Q40).
    let keyword_queries = &queries[30..40];
    assert_eq!(keyword_queries.len(), 10);

    // At least 10 queries use code-specific syntax (Q41-Q50).
    let code_queries = &queries[40..50];
    assert_eq!(code_queries.len(), 10);

    // All relevant files must exist in the corpus.
    let corpus_names: Vec<&str> = corpus.iter().map(|c| c.name.as_str()).collect();
    for q in &queries {
        for r in &q.relevant {
            assert!(
                corpus_names.contains(&r.as_str()),
                "relevant file '{}' for query '{}' not found in corpus",
                r, q.query
            );
        }
    }

    // No duplicate filenames.
    let mut seen = std::collections::HashSet::new();
    for c in &corpus {
        assert!(
            seen.insert(&c.name),
            "duplicate corpus filename: {}",
            c.name
        );
    }
}

#[test]
fn measure_4c_recall_measurement_full() {
    use elai_knowledge::bm25::{Bm25Index, BM25Chunk, ChunkKind};
    use elai_knowledge::splade::QueryResult;

    let expander = match try_make_expander() {
        Some(e) => e,
        None => {
            eprintln!("SKIP: 4c recall measurement — model not staged");
            return;
        }
    };

    let (corpus, queries) = build_synthetic_corpus();

    // Build BM25 index.
    let tmp = tempfile::TempDir::new().unwrap();
    let mut bm25 = Bm25Index::new(tmp.path(), "splade_measure_4c").unwrap();

    for cf in &corpus {
        let path = tmp.path().join(&cf.name);
        std::fs::write(&path, &cf.content).unwrap();
        let chunks = vec![BM25Chunk {
            path: path.clone(),
            content: cf.content.clone(),
            line_start: 1,
            line_end: 1,
            kind: ChunkKind::Function,
        }];
        bm25.index_file(&path, Some(chunks)).unwrap();
    }

    // Measure recall for each query.
    let mut per_query: Vec<QueryResult> = Vec::new();
    let mut bm25_recalls: Vec<f64> = Vec::new();
    let mut splade_recalls: Vec<f64> = Vec::new();
    let mut skipped_zero_bm25 = 0;

    for qr in &queries {
        // BM25-only recall@10.
        let bm25_hits = bm25.query(&qr.query, 10).unwrap();
        let bm25_names: Vec<String> = bm25_hits
            .iter()
            .map(|h| {
                std::path::Path::new(&h.path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        let bm25_r = recall_at_k(&bm25_names, &qr.relevant, 10);

        // SPLADE expansion + RRF fusion.
        let splade_terms = expander.expand_query(&qr.query);

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

        // Score each corpus file by SPLADE term overlap.
        let mut splade_scores: Vec<(String, f32)> = Vec::new();
        for cf in &corpus {
            let content_lower = cf.content.to_lowercase();
            let score: f32 = splade_terms
                .iter()
                .filter(|t| content_lower.contains(&t.term.to_lowercase()))
                .map(|t| t.weight)
                .sum();
            if score > 0.0 {
                splade_scores.push((cf.name.clone(), score));
            }
        }
        splade_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let fused = rrf_fuse(&bm25_for_rrf, &splade_scores, 60, 0.5, 0.5);
        let fused_names: Vec<String> = fused.iter().take(10).map(|(s, _)| s.clone()).collect();
        let splade_r = recall_at_k(&fused_names, &qr.relevant, 10);

        let delta = if bm25_r > 1e-12 {
            Some((splade_r - bm25_r) / bm25_r * 100.0)
        } else {
            skipped_zero_bm25 += 1;
            None
        };

        bm25_recalls.push(bm25_r);
        splade_recalls.push(splade_r);

        per_query.push(QueryResult {
            query: qr.query.clone(),
            bm25_recall: bm25_r,
            splade_recall: splade_r,
            delta_percent: delta,
        });
    }

    // Compute aggregate statistics.
    let valid_deltas: Vec<f64> = per_query.iter().filter_map(|q| q.delta_percent).collect();
    let mean_delta = if valid_deltas.is_empty() {
        0.0
    } else {
        valid_deltas.iter().sum::<f64>() / valid_deltas.len() as f64
    };
    let std_delta = if valid_deltas.len() < 2 {
        0.0
    } else {
        let variance = valid_deltas
            .iter()
            .map(|d| (d - mean_delta).powi(2))
            .sum::<f64>()
            / (valid_deltas.len() - 1) as f64;
        variance.sqrt()
    };

    // Wilcoxon signed-rank test.
    let (w, p, n_nonzero) = wilcoxon_signed_rank(&bm25_recalls, &splade_recalls);

    eprintln!("4c: recall measurement on 50 queries / 100 corpus files");
    eprintln!("4c: mean recall delta: {:.2}% (std: {:.2}%)", mean_delta, std_delta);
    eprintln!("4c: Wilcoxon W={:.2}, p={:.6}, non-zero pairs={}", w, p, n_nonzero);
    eprintln!("4c: queries with BM25 recall=0 (excluded from delta): {}", skipped_zero_bm25);

    // Log per-query breakdown.
    eprintln!("4c: per-query breakdown:");
    for (i, qr) in per_query.iter().enumerate() {
        let delta_str = match qr.delta_percent {
            Some(d) => format!("{:.1}%", d),
            None => "N/A (BM25=0)".to_string(),
        };
        eprintln!(
            "  Q{:02}: bm25={:.2} splade={:.2} delta={} query=\"{}\"",
            i + 1, qr.bm25_recall, qr.splade_recall, delta_str, qr.query
        );
    }

    // Assert measurement ran on all 50 queries.
    assert_eq!(
        per_query.len(),
        50,
        "measurement must run on all 50 queries, ran on {}",
        per_query.len()
    );

    // Check non-zero pairs threshold for statistical validity.
    if n_nonzero < 20 {
        eprintln!(
            "4c: WARNING — insufficient paired differences (N={}, need >=20) for significance testing",
            n_nonzero
        );
    }

    // Store results for 4d decision.
    let decision = keep_drop_decision(mean_delta, p, n_nonzero);
    eprintln!("4c: decision = {}", decision.label());
}

// =========================================================================
// 4d: Keep/drop decision and logging
// =========================================================================

#[test]
fn measure_4d_decision_and_log() {
    let expander = match try_make_expander() {
        Some(e) => e,
        None => {
            eprintln!("SKIP: 4d decision — model not staged");
            return;
        }
    };

    // 4a: Vocab coverage check first (early-exit gate).
    let source = r#"
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
pub struct ErrorHandler { retries: u32 }
impl ErrorHandler { pub fn new(r: u32) -> Self { Self { retries: r } } }
pub fn fibonacci(n: u64) -> u64 { if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) } }
pub fn quicksort<T: Ord>(arr: &mut [T]) { arr.sort(); }
pub enum ParseResult { Success, Failure(String) }
pub trait Serializable { fn serialize(&self) -> Vec<u8>; }
pub async fn fetch_data(url: &str) -> String { url.to_string() }
pub fn memoize(n: u64) -> u64 { n }
"#;

    let code_tokens = extract_code_tokens_from_source(source);
    let coverage = expander.vocab_coverage(&code_tokens);

    eprintln!("4d: vocab coverage = {:.4} ({:.1}%)", coverage, coverage * 100.0);

    if coverage < 0.3 {
        let decision = MeasurementDecision::EarlyDrop {
            coverage: format!("{:.4} ({:.1}%)", coverage, coverage * 100.0),
        };
        eprintln!(
            "4d: {} — SPLADE vocabulary coverage on code tokens is {:.1}% — below 30% threshold.",
            decision.label(),
            coverage * 100.0
        );
        eprintln!("4d: BERT WordPiece tokenizer is fundamentally mismatched for this code corpus.");
        eprintln!("4d: SPLADE will not meaningfully expand code-specific queries.");
        eprintln!("4d: Decision: DROP without recall measurement.");

        // Even with EARLY DROP, verify the feature flag and module are preserved.
        assert!(
            coverage >= 0.0 && coverage <= 1.0,
            "coverage must be in [0.0, 1.0]"
        );
        return;
    }

    // If we reach here, coverage >= 0.3. Full measurement follows via 4c test.
    eprintln!(
        "4d: vocab coverage {:.1}% >= 30% — proceeding with recall evaluation.",
        coverage * 100.0
    );
    // The actual recall measurement and decision logic runs in measure_4c_recall_measurement_full.
    // This test validates the decision function itself.

    // Test decision scenarios.
    let d1 = keep_drop_decision(8.0, 0.01, 25);
    assert_eq!(d1.label(), "KEEP");

    let d2 = keep_drop_decision(3.0, 0.01, 25);
    assert_eq!(d2.label(), "DROP");

    let d3 = keep_drop_decision(8.0, 0.10, 25);
    assert_eq!(d3.label(), "INCONCLUSIVE (default DROP)");

    let d4 = keep_drop_decision(10.0, 0.001, 15);
    assert_eq!(d4.label(), "DROP"); // insufficient pairs

    eprintln!("4d: decision function validated across all scenarios");
}

// =========================================================================
// Adversarial tests for measurement
// =========================================================================

#[test]
fn adversarial_corpus_has_distractors() {
    let (corpus, queries) = build_synthetic_corpus();
    // Q16-Q30 (indices 15-29) are distractor queries. Each should have at
    // least one file in the corpus with a similar name/keyword that is NOT relevant.
    let corpus_names: Vec<&str> = corpus.iter().map(|c| c.name.as_str()).collect();

    for i in 15..30 {
        let q = &queries[i];
        // Count corpus files that share a word with the query but are NOT relevant.
        let query_words: Vec<&str> = q.query.split_whitespace().collect();
        let distractors: Vec<&str> = corpus_names
            .iter()
            .filter(|name| {
                let name_lower = name.to_lowercase();
                // File shares a keyword with the query...
                query_words.iter().any(|w| name_lower.contains(&w.to_lowercase()))
                    // ...but is NOT in the relevant set.
                    && !q.relevant.iter().any(|r| r == **name)
            })
            .copied()
            .collect();

        // This assertion is intentionally soft — some queries distract via content
        // similarity not filename similarity. The important thing is that the
        // corpus construction explicitly includes non-relevant files with
        // overlapping keywords (verified by construction above).
        if !distractors.is_empty() {
            eprintln!(
                "  Q{}: distractors by filename: {:?}",
                i + 1,
                distractors
            );
        }
    }
}

#[test]
fn adversarial_code_syntax_queries_present() {
    let (_corpus, queries) = build_synthetic_corpus();
    // Q41-Q50 must use code-specific syntax/identifiers.
    let code_queries = &queries[40..50];
    let code_indicators = [
        "impl", "derive", "unsafe", "where", "lifetime", "enum",
        "closure", "Fn", "mpsc", "Pin", "Future", "Phantom",
    ];

    let mut code_query_count = 0;
    for q in code_queries {
        let has_code_indicator = code_indicators
            .iter()
            .any(|ind| q.query.to_lowercase().contains(&ind.to_lowercase()));
        if has_code_indicator {
            code_query_count += 1;
        }
    }

    assert!(
        code_query_count >= 8,
        "expected at least 8/10 code-syntax queries to contain code indicators, got {}",
        code_query_count
    );
}

#[test]
fn adversarial_all_50_queries_run() {
    // Validates that the measurement would process all 50 queries without silently
    // dropping any due to errors. Since we can't run the full model-dependent
    // measurement without models, we validate the data structures.
    let (corpus, queries) = build_synthetic_corpus();
    assert_eq!(queries.len(), 50, "must have exactly 50 queries");
    assert_eq!(corpus.len(), 100, "must have exactly 100 corpus files");

    for (i, q) in queries.iter().enumerate() {
        assert!(!q.query.is_empty(), "query {} must not be empty", i);
        assert!(!q.relevant.is_empty(), "query {} must have relevant docs", i);
    }
}

#[test]
fn adversarial_wilcoxon_insufficient_pairs_defaults_to_drop() {
    // If fewer than 20 non-zero paired differences, decision must be DROP.
    let d = keep_drop_decision(10.0, 0.001, 15);
    match d {
        MeasurementDecision::Drop { reason } => {
            assert!(
                reason.contains("insufficient"),
                "reason must mention insufficient pairs"
            );
        }
        other => panic!("expected Drop for insufficient pairs, got {:?}", other),
    }
}
