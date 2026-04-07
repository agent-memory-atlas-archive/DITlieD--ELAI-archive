// Phase 2B.5 benchmark: verification latency measurement.
// Measures FILE_CHECK, SOURCE_CHECK, and ground_plan latencies.
// Uses std::hint::black_box + std::time::Instant (no criterion dependency).
// Reports median of 1000 iterations.

use std::hint::black_box;
use std::time::Instant;

use elai_reasoning::engine::{BasicGrcEngine, GrcEngine};
use elai_reasoning::verification::{LocalVerifier, MockFileReader, MockTestRunner};

fn median(sorted: &[u128]) -> u128 {
    let len = sorted.len();
    if len == 0 {
        return 0;
    }
    if len % 2 == 0 {
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2
    } else {
        sorted[len / 2]
    }
}

fn p99(sorted: &[u128]) -> u128 {
    let idx = (sorted.len() as f64 * 0.99) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Generate a file with `line_count` lines of Rust-like source code.
/// Each line is ~60 chars. Includes a target symbol on line 500.
fn generate_source_file(line_count: usize) -> String {
    let mut lines = Vec::with_capacity(line_count);
    for i in 0..line_count {
        if i == 499 {
            // Target symbol at line 500
            lines.push("fn target_function_for_bench() -> Result<(), String> {".to_string());
        } else if i % 50 == 0 {
            lines.push(format!("struct Decoy{} {{ field: u64 }}", i));
        } else if i % 30 == 0 {
            lines.push(format!("fn helper_{}(x: i32) -> i32 {{ x + {} }}", i, i));
        } else {
            lines.push(format!("    let _val_{} = {};", i, i * 7));
        }
    }
    lines.join("\n")
}

/// Generate a plan text with `claim_count` grounding claims.
/// Mix of file:, grep:, test:, and training: prefixes.
fn generate_plan_with_claims(
    claim_count: usize,
    existing_files: &[&str],
) -> String {
    let mut plan = String::from("# Benchmark plan\n\n");
    for i in 0..claim_count {
        let claim = match i % 4 {
            0 => format!("  Grounding: file:{}", existing_files[i % existing_files.len()]),
            1 => format!("  Grounding: grep:helper_{}", i),
            2 => format!("  Grounding: test:bench_test_{}", i),
            3 => format!("  Grounding: training:some_claim_{}", i),
            _ => unreachable!(),
        };
        plan.push_str(&format!("Step {}: do something\n{}\n\n", i + 1, claim));
    }
    plan
}

/// Bench FILE_CHECK: check file_exists on a MockFileReader.
fn bench_file_check(iterations: usize) -> Vec<u128> {
    let mut durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let reader = MockFileReader::new()
            .with_file("src/lib.rs", "pub mod engine;")
            .with_file("src/types.rs", "pub struct Foo;")
            .with_file("Cargo.toml", "[package]\nname = \"test\"");

        let verifier = LocalVerifier::new(Box::new(reader), Box::new(MockTestRunner::new()));
        let engine = BasicGrcEngine::with_verifier(Box::new(verifier));

        let start = Instant::now();
        let result = engine.verify_claim("file:src/lib.rs", "file", "");
        let elapsed = start.elapsed();
        black_box(&result);
        assert!(result.is_ok());
        durations.push(elapsed.as_nanos());
    }

    durations.sort_unstable();
    durations
}

/// Bench SOURCE_CHECK: search for a symbol in a 1000-line file.
fn bench_source_check(iterations: usize) -> Vec<u128> {
    let mut durations = Vec::with_capacity(iterations);
    let source_content = generate_source_file(1000);

    for _ in 0..iterations {
        let reader = MockFileReader::new()
            .with_file("src/engine.rs", &source_content);

        let verifier = LocalVerifier::new(Box::new(reader), Box::new(MockTestRunner::new()));
        let engine = BasicGrcEngine::with_verifier(Box::new(verifier));

        let start = Instant::now();
        let result = engine.verify_claim("tool:target_function_for_bench", "tool", "src/engine.rs");
        let elapsed = start.elapsed();
        black_box(&result);
        assert!(result.is_ok());
        assert!(result.unwrap().passes, "symbol must be found in 1000-line file");
        durations.push(elapsed.as_nanos());
    }

    durations.sort_unstable();
    durations
}

/// Bench ground_plan: sweep a plan with 20 claims.
fn bench_ground_plan(iterations: usize) -> Vec<u128> {
    let mut durations = Vec::with_capacity(iterations);

    let files = [
        "src/lib.rs", "src/types.rs", "src/engine.rs",
        "src/verification.rs", "Cargo.toml",
    ];
    let plan = generate_plan_with_claims(20, &files);
    let source_content = generate_source_file(1000);

    for _ in 0..iterations {
        let reader = MockFileReader::new()
            .with_file("src/lib.rs", "pub mod engine;\npub mod types;")
            .with_file("src/types.rs", "pub struct Foo;\npub enum Bar {}")
            .with_file("src/engine.rs", &source_content)
            .with_file("src/verification.rs", "pub trait Verifier {}")
            .with_file("Cargo.toml", "[package]\nname = \"test\"");

        let test_runner = MockTestRunner::new();
        let verifier = LocalVerifier::new(Box::new(reader), Box::new(test_runner));
        let engine = BasicGrcEngine::with_verifier(Box::new(verifier));

        let start = Instant::now();
        let result = engine.ground_plan(&plan);
        let elapsed = start.elapsed();
        black_box(&result);
        durations.push(elapsed.as_nanos());
    }

    durations.sort_unstable();
    durations
}

fn main() {
    let iterations = 1000;

    // FILE_CHECK latency
    let file_check_times = bench_file_check(iterations);
    let file_check_median_ns = median(&file_check_times);
    let file_check_p99_ns = p99(&file_check_times);
    let file_check_median_us = file_check_median_ns / 1000;

    // SOURCE_CHECK latency (1000-line file)
    let source_check_times = bench_source_check(iterations);
    let source_check_median_ns = median(&source_check_times);
    let source_check_p99_ns = p99(&source_check_times);
    let source_check_median_us = source_check_median_ns / 1000;

    // ground_plan latency (20 claims)
    let plan_sweep_times = bench_ground_plan(iterations);
    let plan_sweep_median_ns = median(&plan_sweep_times);
    let plan_sweep_p99_ns = p99(&plan_sweep_times);
    let plan_sweep_median_us = plan_sweep_median_ns / 1000;

    println!("--- Phase 2B.5 Verification Benchmark Results ---");
    println!("FILE_CHECK (median of {iterations} runs): {file_check_median_ns} ns ({file_check_median_us} us)");
    println!("FILE_CHECK (p99): {file_check_p99_ns} ns");
    println!("SOURCE_CHECK 1000-line (median): {source_check_median_ns} ns ({source_check_median_us} us)");
    println!("SOURCE_CHECK 1000-line (p99): {source_check_p99_ns} ns");
    println!("ground_plan 20-claim (median): {plan_sweep_median_ns} ns ({plan_sweep_median_us} us)");
    println!("ground_plan 20-claim (p99): {plan_sweep_p99_ns} ns");
    println!();
    println!("# TOML output");
    println!("file_check_latency_us = {file_check_median_us}");
    println!("file_check_latency_ns = {file_check_median_ns}");
    println!("file_check_p99_ns = {file_check_p99_ns}");
    println!("source_check_latency_us = {source_check_median_us}");
    println!("source_check_latency_ns = {source_check_median_ns}");
    println!("source_check_p99_ns = {source_check_p99_ns}");
    println!("plan_sweep_latency_us = {plan_sweep_median_us}");
    println!("plan_sweep_latency_ns = {plan_sweep_median_ns}");
    println!("plan_sweep_p99_ns = {plan_sweep_p99_ns}");
}
