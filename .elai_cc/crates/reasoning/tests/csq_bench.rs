use std::hint::black_box;
use std::time::Instant;

use elai_reasoning::{BasicGrcEngine, FeasibilityOutcome, GrcEngine};

// ---------------------------------------------------------------------------
// Phase 2D.5: Performance and correctness metrics
// Uses std::hint::black_box + std::time::Instant, median of 1000 iterations.
// No external benchmark dependency (criterion) — local measurement only.
// ---------------------------------------------------------------------------

/// Measure CSQ generation overhead for a single function with sensitive ops.
/// Target: <1ms (1000us) per call.
#[test]
fn bench_csq_generation_overhead() {
    let engine = BasicGrcEngine::new();
    let sensitive_ops = vec![
        "std::fs::write".to_string(),
        "std::net::TcpStream::connect".to_string(),
        "auth::validate_token".to_string(),
    ];

    let mut durations = Vec::with_capacity(1000);
    for _ in 0..1000 {
        let start = Instant::now();
        let result = black_box(
            engine
                .generate_csq(black_box("write_config"), black_box(&sensitive_ops))
                .expect("generate_csq should not error"),
        );
        let elapsed = start.elapsed();
        durations.push(elapsed);
        assert!(!result.checks.is_empty());
    }

    durations.sort();
    let median = durations[500];
    let p99 = durations[990];
    let median_us = median.as_micros();
    let p99_us = p99.as_micros();

    println!("METRIC:csq_overhead_us={}", median_us);
    println!("METRIC:csq_overhead_ns={}", median.as_nanos());
    println!("METRIC:csq_overhead_p99_us={}", p99_us);
    println!("METRIC:csq_overhead_p99_ns={}", p99.as_nanos());

    assert!(
        median_us < 1000,
        "CSQ generation median {}us exceeds 1ms target",
        median_us
    );
}

/// Measure inline feasibility check overhead for source matching.
/// Target: <2ms (2000us) per call.
#[test]
fn bench_inline_feasibility_overhead() {
    let sources: Vec<String> = (0..20)
        .map(|i| format!("PipelineSpec: use algorithm {} for processing step {}", i, i))
        .collect();
    let claim = "use algorithm 15 for processing step 15";

    let mut durations = Vec::with_capacity(1000);
    for _ in 0..1000 {
        // Fresh engine each iteration to avoid BudgetExhausted.
        let mut engine = BasicGrcEngine::new();
        let start = Instant::now();
        let result = black_box(
            engine
                .check_inline_feasibility(black_box(claim), black_box(&sources))
                .expect("inline feasibility should not error"),
        );
        let elapsed = start.elapsed();
        durations.push(elapsed);
        assert_eq!(result.outcome, FeasibilityOutcome::Pass);
    }

    durations.sort();
    let median = durations[500];
    let p99 = durations[990];
    let median_us = median.as_micros();
    let p99_us = p99.as_micros();

    println!("METRIC:inline_check_overhead_us={}", median_us);
    println!("METRIC:inline_check_overhead_ns={}", median.as_nanos());
    println!("METRIC:inline_check_overhead_p99_us={}", p99_us);
    println!("METRIC:inline_check_overhead_p99_ns={}", p99.as_nanos());

    assert!(
        median_us < 2000,
        "Inline feasibility median {}us exceeds 2ms target",
        median_us
    );
}

/// Measure false positive/negative rate on 20 synthetic claims (10 present, 10 absent).
/// Pattern matching is deterministic: expected rate = 0%.
/// Claims are realistic (>5 words each) per adversarial requirement.
#[test]
fn bench_false_positive_rate() {
    let sources: Vec<String> = vec![
        "PipelineSpec: use tantivy BM25 index for full-text search across documents".to_string(),
        "PipelineSpec: apply blake3 hashing for content deduplication in ingestion pipeline".to_string(),
        "ResearchReport: transformer attention mechanism scales quadratically with sequence length".to_string(),
        "DKG: function process_payment calls stripe::charge::create with idempotency key".to_string(),
        "PipelineSpec: rate limiting uses token bucket algorithm with sliding window".to_string(),
        "ResearchReport: gradient checkpointing reduces memory usage by square root factor".to_string(),
        "DKG: function validate_auth calls jwt::decode with RS256 algorithm verification".to_string(),
        "PipelineSpec: cache invalidation uses write-through strategy with TTL expiration".to_string(),
        "ResearchReport: batch normalization improves training convergence by reducing covariate shift".to_string(),
        "DKG: function export_data calls csv::Writer::from_path with buffered output stream".to_string(),
    ];

    // 10 claims PRESENT in sources (should all Pass)
    let present_claims = vec![
        "use tantivy BM25 index for full-text search across documents",
        "apply blake3 hashing for content deduplication in ingestion pipeline",
        "transformer attention mechanism scales quadratically with sequence length",
        "function process_payment calls stripe::charge::create with idempotency key",
        "rate limiting uses token bucket algorithm with sliding window",
        "gradient checkpointing reduces memory usage by square root factor",
        "function validate_auth calls jwt::decode with RS256 algorithm verification",
        "cache invalidation uses write-through strategy with TTL expiration",
        "batch normalization improves training convergence by reducing covariate shift",
        "function export_data calls csv::Writer::from_path with buffered output stream",
    ];

    // 10 claims ABSENT from sources (should all Fail)
    let absent_claims = vec![
        "use elasticsearch for full-text search with inverted index sharding",
        "apply sha256 hashing for content integrity verification in storage layer",
        "recurrent neural network handles variable length sequences with hidden state",
        "function refund_payment calls paypal::refund::execute with correlation identifier",
        "rate limiting uses leaky bucket algorithm with fixed drain rate",
        "model pruning reduces inference latency by removing redundant connections",
        "function verify_session calls oauth2::token::refresh with client credentials",
        "cache eviction uses least-recently-used strategy with probabilistic sampling",
        "layer normalization stabilizes training by normalizing across feature dimension",
        "function import_data calls parquet::reader::from_file with memory mapped access",
    ];

    let mut false_positives = 0u32;
    let mut false_negatives = 0u32;

    // Test present claims (expect Pass)
    for claim in &present_claims {
        let mut engine = BasicGrcEngine::new();
        let result = engine
            .check_inline_feasibility(claim, &sources)
            .expect("inline feasibility should not error");
        if result.outcome != FeasibilityOutcome::Pass {
            false_negatives += 1;
            eprintln!("FALSE NEGATIVE: '{}' expected Pass, got {:?}", claim, result.outcome);
        }
    }

    // Test absent claims (expect Fail)
    for claim in &absent_claims {
        let mut engine = BasicGrcEngine::new();
        let result = engine
            .check_inline_feasibility(claim, &sources)
            .expect("inline feasibility should not error");
        if result.outcome != FeasibilityOutcome::Fail {
            false_positives += 1;
            eprintln!("FALSE POSITIVE: '{}' expected Fail, got {:?}", claim, result.outcome);
        }
    }

    let total = (present_claims.len() + absent_claims.len()) as f64;
    let fp_rate = false_positives as f64 / total;
    let fn_rate = false_negatives as f64 / total;

    println!("METRIC:false_positive_rate={:.1}", fp_rate);
    println!("METRIC:false_negative_rate={:.1}", fn_rate);
    println!("METRIC:false_positives={}", false_positives);
    println!("METRIC:false_negatives={}", false_negatives);
    println!("METRIC:total_claims_tested={}", total as u32);

    assert_eq!(false_positives, 0, "Expected 0 false positives, got {}", false_positives);
    assert_eq!(false_negatives, 0, "Expected 0 false negatives, got {}", false_negatives);
}
