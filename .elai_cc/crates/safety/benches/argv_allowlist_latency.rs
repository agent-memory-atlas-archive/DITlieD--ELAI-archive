//! E-30: argv-allowlist latency benchmark.
//!
//! Verifies that `check_argv()` meets the B-19 T1 latency budget
//! (p99 < 1ms per invocation) on a representative argv set covering
//! Allow / Deny / RequireHumanApproval verdicts.
//!
//! Uses `std::time::Instant` + harness=false instead of `criterion`:
//!   - anti-slopsquatting (CLAUDE.md): no new external dep
//!   - matches the existing `plugins/benches/tiered_analysis_bench.rs`
//!     precedent (workspace standard for micro-benches)
//!
//! Run with: `cargo bench -p elai-safety --bench argv_allowlist_latency`

use std::ffi::OsString;
use std::time::Instant;

use elai_safety::{check_argv, AllowlistVerdict};

/// Number of measured invocations per fixture (after warmup).
/// 1000 samples gives a stable p99 estimate.
const ITERATIONS: usize = 1_000;

/// Warmup iterations to stabilize the regex lazy-init + branch predictor.
/// Once_cell-backed RegexSet compiles on first access; warmup amortizes that.
const WARMUP: usize = 100;

/// B-19 T1 latency budget. p99 must be strictly less than this on a
/// representative argv set. Headroom: budget is 1ms; we assert <1ms p99.
const BUDGET_NANOS_P99: u128 = 1_000_000; // 1ms in nanoseconds

fn argv(args: &[&str]) -> Vec<OsString> {
    args.iter().map(|s| OsString::from(*s)).collect()
}

/// Representative fixture set: 12 cases spanning every code path.
///
/// 4 Allow paths (no match)
/// 4 Deny paths (highest precedence)
/// 4 RequireHumanApproval paths
fn fixtures() -> Vec<Vec<OsString>> {
    vec![
        // Allow paths -- common safe commands
        argv(&["ls", "-la"]),
        argv(&["cargo", "test", "-p", "elai-safety"]),
        argv(&["git", "checkout", "feature/my-branch"]),
        argv(&["cat", "README.md"]),
        // Deny paths -- destructive verbs (highest precedence)
        argv(&["rm", "-rf", "~/projects"]),
        argv(&["git", "restore", "."]),
        argv(&["psql", "-c", "DROP TABLE users"]),
        argv(&["chmod", "-R", "777", "/etc"]),
        // RequireHumanApproval paths -- cloud / infra deletions
        argv(&["terraform", "destroy"]),
        argv(&["aws", "ec2", "terminate-instances", "--instance-ids", "i-x"]),
        argv(&["kubectl", "delete", "namespace", "production"]),
        argv(&["docker", "rm", "-fv", "prod-container"]),
    ]
}

fn percentile(sorted_nanos: &[u128], pct: f64) -> u128 {
    if sorted_nanos.is_empty() {
        return 0;
    }
    let idx = ((sorted_nanos.len() as f64 - 1.0) * pct).floor() as usize;
    sorted_nanos[idx]
}

fn main() {
    let fixtures = fixtures();

    // Warmup: hits each fixture WARMUP times so lazy regex init + branch
    // prediction stabilize before measurement.
    for _ in 0..WARMUP {
        for argv in &fixtures {
            let v = check_argv(argv);
            // black-box: prevent dead-code elimination
            std::hint::black_box(v);
        }
    }

    // Measure: ITERATIONS samples per fixture.
    let mut samples_nanos: Vec<u128> = Vec::with_capacity(ITERATIONS * fixtures.len());

    for argv in &fixtures {
        for _ in 0..ITERATIONS {
            let t0 = Instant::now();
            let v = check_argv(argv);
            let elapsed = t0.elapsed().as_nanos();
            std::hint::black_box(v);
            samples_nanos.push(elapsed);
        }
    }

    samples_nanos.sort_unstable();

    let p50 = percentile(&samples_nanos, 0.50);
    let p95 = percentile(&samples_nanos, 0.95);
    let p99 = percentile(&samples_nanos, 0.99);
    let p999 = percentile(&samples_nanos, 0.999);
    let max = samples_nanos.last().copied().unwrap_or(0);
    let total_samples = samples_nanos.len();

    println!("E-30 argv-allowlist latency benchmark");
    println!("======================================");
    println!("Fixtures:           {}", fixtures.len());
    println!("Iterations/fixture: {ITERATIONS}");
    println!("Warmup/fixture:     {WARMUP}");
    println!("Total samples:      {total_samples}");
    println!();
    println!("Latency (ns):");
    println!("  p50:  {p50:>8}");
    println!("  p95:  {p95:>8}");
    println!("  p99:  {p99:>8}");
    println!("  p999: {p999:>8}");
    println!("  max:  {max:>8}");
    println!();
    println!("B-19 T1 budget: p99 < {BUDGET_NANOS_P99} ns (1 ms)");
    println!(
        "Verdict:        {}",
        if p99 < BUDGET_NANOS_P99 {
            "PASS"
        } else {
            "FAIL"
        }
    );

    // Hard-assert the budget. If this trips, the regex set has regressed
    // or the bench host is overloaded; either way we want a loud failure.
    assert!(
        p99 < BUDGET_NANOS_P99,
        "E-30 ACCEPTANCE: argv-allowlist p99 latency {p99} ns exceeds B-19 T1 budget {BUDGET_NANOS_P99} ns (1 ms)"
    );

    // Sanity: ensure every verdict variant was exercised.
    let mut seen_allow = false;
    let mut seen_deny = false;
    let mut seen_approval = false;
    for argv in &fixtures {
        match check_argv(argv) {
            AllowlistVerdict::Allow => seen_allow = true,
            AllowlistVerdict::Deny { .. } => seen_deny = true,
            AllowlistVerdict::RequireHumanApproval { .. } => seen_approval = true,
        }
    }
    assert!(seen_allow, "fixtures must exercise Allow path");
    assert!(seen_deny, "fixtures must exercise Deny path");
    assert!(seen_approval, "fixtures must exercise RequireHumanApproval path");
}
