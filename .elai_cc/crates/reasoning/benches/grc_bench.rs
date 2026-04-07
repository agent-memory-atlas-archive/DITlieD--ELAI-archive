// Phase 2A.5 benchmark: validation overhead and memory per topic.
// Uses std::hint::black_box + std::time::Instant (no criterion dependency).
// Reports median of 1000 iterations.

use std::hint::black_box;
use std::time::Instant;

use elai_reasoning::engine::{BasicGrcEngine, GrcEngine};
use elai_reasoning::types::{GrcState, ThinkStep};

fn make_step(step_type: GrcState, topic: &str, grounding: Option<&str>) -> ThinkStep {
    ThinkStep {
        step_type,
        content: "benchmark content for measurement".to_string(),
        topic: topic.to_string(),
        grounding: grounding.map(|s| s.to_string()),
        conclusion: None,
        scope: None,
    }
}

/// Run a full valid chain (Observe -> Hypothesize -> PlanAction -> Act -> Verify)
/// and measure time per think() call.
fn bench_validation_overhead(iterations: usize) -> Vec<u128> {
    let mut durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let mut engine = BasicGrcEngine::new();
        engine.set_budget("bench", 10, 2000);

        let steps = [
            make_step(GrcState::Observe, "bench", Some("file:src/lib.rs:L1")),
            make_step(
                GrcState::Hypothesize,
                "bench",
                Some("grep:fn main"),
            ),
            make_step(GrcState::PlanAction, "bench", None),
            make_step(GrcState::Act, "bench", None),
            make_step(GrcState::Verify, "bench", Some("test:unit_pass")),
        ];

        let start = Instant::now();
        for step in &steps {
            let result = engine.think(step.clone());
            black_box(&result);
        }
        let elapsed = start.elapsed();
        // Per-step average in nanoseconds
        durations.push(elapsed.as_nanos() / steps.len() as u128);
    }

    durations.sort_unstable();
    durations
}

/// Measure base memory per TopicState by creating many topics and
/// computing the size delta.
fn bench_memory_per_topic() -> usize {
    // Measure: create 100 topics, check rough memory via allocation size estimate.
    // We can't use a memory allocator hook without a dep, so compute struct sizes.
    let topic_state_base = std::mem::size_of::<elai_reasoning::types::TopicState>();
    let budget_size = std::mem::size_of::<elai_reasoning::types::GrcBudget>();
    let grc_state_size = std::mem::size_of::<elai_reasoning::types::GrcState>();
    // TopicState base = budget + Option<GrcState> + Vec<ThinkStep> (24 on 64-bit)
    //   + Vec<(String, GroundingLevel)> (24) + Option<Vec<String>> (24)
    // These are stack sizes of the struct fields (heap is separate).
    let _ = budget_size;
    let _ = grc_state_size;
    topic_state_base
}

/// Measure time for grounding parse + validation path (malformed rejection).
fn bench_rejection_overhead(iterations: usize) -> Vec<u128> {
    let mut durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let mut engine = BasicGrcEngine::new();
        engine.set_budget("rej", 10, 2000);

        // First, establish a valid Observe step
        let _ = engine.think(make_step(
            GrcState::Observe,
            "rej",
            Some("file:src/lib.rs:L1"),
        ));

        // Measure rejection of invalid transition
        let bad_step = make_step(
            GrcState::Verify,
            "rej",
            Some("test:result"),
        );

        let start = Instant::now();
        let result = engine.think(bad_step);
        let elapsed = start.elapsed();
        black_box(&result);
        assert!(result.is_err());
        durations.push(elapsed.as_nanos());
    }

    durations.sort_unstable();
    durations
}

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

fn main() {
    let iterations = 1000;

    // Validation overhead
    let validation_times = bench_validation_overhead(iterations);
    let validation_median = median(&validation_times);
    let validation_p99 = p99(&validation_times);

    // Rejection overhead
    let rejection_times = bench_rejection_overhead(iterations);
    let rejection_median = median(&rejection_times);
    let rejection_p99 = p99(&rejection_times);

    // Memory per topic
    let topic_base_bytes = bench_memory_per_topic();

    // Convert ns to us for the spec requirement (target: <5000 us = <5ms)
    let validation_median_us = validation_median / 1000;
    let validation_p99_us = validation_p99 / 1000;
    let rejection_median_us = rejection_median / 1000;
    let rejection_p99_us = rejection_p99 / 1000;

    println!("--- Phase 2A.5 GRC Benchmark Results ---");
    println!("Validation overhead (per-step, median of {iterations} runs): {validation_median} ns ({validation_median_us} us)");
    println!("Validation overhead (per-step, p99): {validation_p99} ns ({validation_p99_us} us)");
    println!("Rejection overhead (median of {iterations} runs): {rejection_median} ns ({rejection_median_us} us)");
    println!("Rejection overhead (p99): {rejection_p99} ns ({rejection_p99_us} us)");
    println!("TopicState base size (stack): {topic_base_bytes} bytes");
    println!();

    // Output in TOML format for easy parsing
    println!("# TOML output");
    println!("validation_overhead_us = {validation_median_us}");
    println!("validation_overhead_ns = {validation_median}");
    println!("validation_overhead_p99_us = {validation_p99_us}");
    println!("validation_overhead_p99_ns = {validation_p99}");
    println!("rejection_overhead_us = {rejection_median_us}");
    println!("rejection_overhead_ns = {rejection_median}");
    println!("rejection_overhead_p99_us = {rejection_p99_us}");
    println!("rejection_overhead_p99_ns = {rejection_p99}");
    println!("topic_base_bytes = {topic_base_bytes}");
}
