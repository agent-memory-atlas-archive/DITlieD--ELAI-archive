// C-19 §Phase-4C benchmark — tiered analysis pipeline timing.
//
// Matches workspace precedent (knowledge/, reasoning/ crates): harness=false,
// std::hint::black_box + std::time::Instant, no criterion dependency
// (orchestrator D2 / DRIFT-2). Reports median + p99 over 1000 iterations.
//
// Bench groups (spec phase-4C.txt §4C.3 / §4C.4):
//   - bench_t1_rules: T1Hook.evaluate with empty rule set; target p50 < 5ms.
//   - bench_t2_boundary: T2Hook.evaluate at 4999 / 5000 / 5001 line files;
//     target p50 < 50ms (and circuit-breaker kicks in at 5000).
//   - bench_t3_warn: T3Hook.evaluate on a WARN-triggering source pattern;
//     target p50 < 500ms.
//   - bench_post_tool_use_simulated: end-to-end T1+T2 dispatch latency
//     (stand-in for the full PostToolUse hook chain; ruff subprocess is
//     not invoked, only the in-process pipeline is timed).
//
// Output: human-readable lines AND a TOML block under `# TOML output`
// for parsing by `phase_4C.toml` metric backfill (C-20).

use std::hint::black_box;
use std::time::Instant;

use elai_plugins::tiered_analysis::{T1Hook, T2Hook, T3Hook};
use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
use rustc_hash::FxHashMap;

fn make_t1() -> T1Hook {
    let engine =
        DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
    T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING)
}

fn make_t2() -> T2Hook {
    T2Hook::new(5000)
}

fn make_t3() -> T3Hook {
    T3Hook::new()
}

// Generate `lines` lines of plausible Rust source.
fn synth_rust(lines: usize) -> String {
    let mut s = String::with_capacity(lines * 32);
    s.push_str("fn main() {\n");
    for i in 0..lines.saturating_sub(2) {
        s.push_str(&format!("    let x_{i} = {i} + 1;\n"));
    }
    s.push_str("}\n");
    s
}

fn bench_t1_rules(iterations: usize) -> Vec<u128> {
    let hook = make_t1();
    let source = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let file_path = "src/main.rs";
    let mut durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let metrics = FxHashMap::default();
        let start = Instant::now();
        let result = hook.evaluate(black_box("file_write"), black_box(source), black_box(file_path), metrics);
        black_box(&result);
        durations.push(start.elapsed().as_nanos());
    }
    durations.sort_unstable();
    durations
}

fn bench_t2_at(lines: usize, iterations: usize) -> Vec<u128> {
    let hook = make_t2();
    let source = synth_rust(lines);
    let file_path = "src/synthesized.rs";
    let mut durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let result = hook.evaluate(black_box(&source), black_box(file_path));
        black_box(&result);
        durations.push(start.elapsed().as_nanos());
    }
    durations.sort_unstable();
    durations
}

fn bench_t3_warn(iterations: usize) -> Vec<u128> {
    let hook = make_t3();
    // Source intentionally trips T3 patterns when present (T3 is OS-level
    // pattern matching; the bench measures pipeline overhead, not match-rate).
    let source = "todo!();\n// FIXME: implement\nfn x() { unimplemented!() }\n";
    let file_path = "src/warn.rs";
    let mut durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let result = hook.evaluate(black_box(source), black_box(file_path));
        black_box(&result);
        durations.push(start.elapsed().as_nanos());
    }
    durations.sort_unstable();
    durations
}

fn bench_post_tool_use_simulated(iterations: usize) -> Vec<u128> {
    let t1 = make_t1();
    let t2 = make_t2();
    let t3 = make_t3();
    let source = synth_rust(200);
    let file_path = "src/post.rs";
    let mut durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let metrics = FxHashMap::default();
        let start = Instant::now();
        let _r1 = t1.evaluate(
            black_box("file_write"),
            black_box(&source),
            black_box(file_path),
            metrics,
        );
        let _r2 = t2.evaluate(black_box(&source), black_box(file_path));
        let _r3 = t3.evaluate(black_box(&source), black_box(file_path));
        black_box((&_r1, &_r2, &_r3));
        durations.push(start.elapsed().as_nanos());
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
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64) * 0.99) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn ns_to_ms(ns: u128) -> f64 {
    (ns as f64) / 1_000_000.0
}

fn main() {
    let iterations = 1000;

    eprintln!("--- C-19 §Phase-4C Tiered Analysis Benchmark ---");
    eprintln!("iterations = {iterations}");

    // T1
    let t1_durs = bench_t1_rules(iterations);
    let t1_med = median(&t1_durs);
    let t1_p99 = p99(&t1_durs);
    eprintln!(
        "t1_rules           median={t1_med}ns ({:.3}ms)  p99={t1_p99}ns ({:.3}ms)  target_p50_ms=5.0",
        ns_to_ms(t1_med),
        ns_to_ms(t1_p99)
    );

    // T2 boundary at three line counts.
    let t2_below = bench_t2_at(4999, iterations);
    let t2_at = bench_t2_at(5000, iterations);
    let t2_above = bench_t2_at(5001, iterations);
    let (b_med, b_p99) = (median(&t2_below), p99(&t2_below));
    let (a_med, a_p99) = (median(&t2_at), p99(&t2_at));
    let (o_med, o_p99) = (median(&t2_above), p99(&t2_above));
    eprintln!(
        "t2_boundary_4999   median={b_med}ns ({:.3}ms)  p99={b_p99}ns ({:.3}ms)",
        ns_to_ms(b_med),
        ns_to_ms(b_p99)
    );
    eprintln!(
        "t2_boundary_5000   median={a_med}ns ({:.3}ms)  p99={a_p99}ns ({:.3}ms)",
        ns_to_ms(a_med),
        ns_to_ms(a_p99)
    );
    eprintln!(
        "t2_boundary_5001   median={o_med}ns ({:.3}ms)  p99={o_p99}ns ({:.3}ms)  target_p50_ms=50.0",
        ns_to_ms(o_med),
        ns_to_ms(o_p99)
    );

    // T3
    let t3_durs = bench_t3_warn(iterations);
    let t3_med = median(&t3_durs);
    let t3_p99 = p99(&t3_durs);
    eprintln!(
        "t3_warn            median={t3_med}ns ({:.3}ms)  p99={t3_p99}ns ({:.3}ms)  target_p50_ms=500.0",
        ns_to_ms(t3_med),
        ns_to_ms(t3_p99)
    );

    // PostToolUse (T1 + T2 + T3 dispatch chain).
    let pt_durs = bench_post_tool_use_simulated(iterations);
    let pt_med = median(&pt_durs);
    let pt_p99 = p99(&pt_durs);
    eprintln!(
        "post_tool_use      median={pt_med}ns ({:.3}ms)  p99={pt_p99}ns ({:.3}ms)  target_p99_ms=10.0",
        ns_to_ms(pt_med),
        ns_to_ms(pt_p99)
    );

    // TOML output (parsed by C-20 metric backfill).
    println!("# TOML output");
    println!("[metrics.t1_latency]");
    println!("p50_ms = {:.4}", ns_to_ms(t1_med));
    println!("p99_ms = {:.4}", ns_to_ms(t1_p99));
    println!("[metrics.t2_latency]");
    println!("p50_ms = {:.4}", ns_to_ms(a_med));
    println!("p99_ms = {:.4}", ns_to_ms(a_p99));
    println!("[metrics.t3_latency]");
    println!("p50_ms = {:.4}", ns_to_ms(t3_med));
    println!("p99_ms = {:.4}", ns_to_ms(t3_p99));
    println!("[metrics.post_tool_use_latency]");
    println!("p99_ms = {:.4}", ns_to_ms(pt_p99));
}
