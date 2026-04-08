//! Integration tests for orchestrator/context_budget.
//! Wires compute() + update_breakpoints() + adaptive_reduce() + SSB in end-to-end scenarios.
//! Phase 3E.7: determinism proof, latency assertion, entropy quarantine verification.

use std::sync::Arc;
use std::time::{Duration, Instant};

use elai_core::clock::SeededClock;
use elai_orchestrator::context_budget::{
    AdaptiveLevel, ContextBudget, ContextSlot, ExecutionMode,
    FLOOR_GRC_HISTORY, FLOOR_OUTPUT_RESERVE, FLOOR_SSB_ENTRIES,
};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use tempfile::TempDir;

const MODEL_MAX_128K: usize = 131_072;
const MODEL_MAX_4K: usize = 4_096;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_ssb_for_integration(dir: &TempDir) -> Ssb {
    let config = SsbConfig {
        db_path: dir.path().join("integration_ssb.db"),
        pool_size: Some(2),
        project_id: "integration-project".into(),
        session_id: "s-integration".into(),
    };
    Ssb::new(config, Arc::new(SeededClock::new())).expect("SSB init for integration test")
}

fn make_8_slices<'a>(content: &'a [u8]) -> [&'a [u8]; 8] {
    [content; 8]
}

// ---------------------------------------------------------------------------
// 1. Sum invariant: small model
// ---------------------------------------------------------------------------

#[test]
fn budget_assembly_small_model_sum_invariant() {
    let budget = ContextBudget::compute(MODEL_MAX_4K, [256, 128, 128], 2000)
        .expect("compute must succeed for small model");
    let sum: usize = budget.slots.iter().map(|s| s.allocated_tokens).sum();
    assert_eq!(
        sum, MODEL_MAX_4K,
        "sum of all slots must equal model_max_context"
    );
    assert_eq!(
        budget.total_allocated, MODEL_MAX_4K,
        "total_allocated bookkeeping must match slot sum"
    );
}

// ---------------------------------------------------------------------------
// 2. Sum invariant: large model
// ---------------------------------------------------------------------------

#[test]
fn budget_assembly_large_model_sum_invariant() {
    let budget = ContextBudget::compute(MODEL_MAX_128K, [4096, 2048, 2048], 2000)
        .expect("compute must succeed for large model");
    let sum: usize = budget.slots.iter().map(|s| s.allocated_tokens).sum();
    assert_eq!(
        sum, MODEL_MAX_128K,
        "sum of all slots must equal model_max_context"
    );
    assert_eq!(
        budget.total_allocated, MODEL_MAX_128K,
        "total_allocated bookkeeping must match slot sum"
    );
}

// ---------------------------------------------------------------------------
// 3. Floors respected: small model
// ---------------------------------------------------------------------------

#[test]
fn budget_floors_respected_small_model() {
    let budget = ContextBudget::compute(MODEL_MAX_4K, [256, 128, 128], 2000)
        .expect("compute must succeed");
    assert!(
        budget.slots[ContextSlot::SsbEntries as usize].allocated_tokens >= FLOOR_SSB_ENTRIES,
        "SsbEntries floor violated: {} < {}",
        budget.slots[ContextSlot::SsbEntries as usize].allocated_tokens,
        FLOOR_SSB_ENTRIES,
    );
    assert!(
        budget.slots[ContextSlot::GrcHistory as usize].allocated_tokens >= FLOOR_GRC_HISTORY,
        "GrcHistory floor violated: {} < {}",
        budget.slots[ContextSlot::GrcHistory as usize].allocated_tokens,
        FLOOR_GRC_HISTORY,
    );
    assert!(
        budget.slots[ContextSlot::OutputReserve as usize].allocated_tokens >= FLOOR_OUTPUT_RESERVE,
        "OutputReserve floor violated: {} < {}",
        budget.slots[ContextSlot::OutputReserve as usize].allocated_tokens,
        FLOOR_OUTPUT_RESERVE,
    );
}

// ---------------------------------------------------------------------------
// 4. Floors respected: large model
// ---------------------------------------------------------------------------

#[test]
fn budget_floors_respected_large_model() {
    let budget = ContextBudget::compute(MODEL_MAX_128K, [4096, 2048, 2048], 2000)
        .expect("compute must succeed");
    assert!(
        budget.slots[ContextSlot::SsbEntries as usize].allocated_tokens >= FLOOR_SSB_ENTRIES,
        "SsbEntries floor violated: {} < {}",
        budget.slots[ContextSlot::SsbEntries as usize].allocated_tokens,
        FLOOR_SSB_ENTRIES,
    );
    assert!(
        budget.slots[ContextSlot::GrcHistory as usize].allocated_tokens >= FLOOR_GRC_HISTORY,
        "GrcHistory floor violated: {} < {}",
        budget.slots[ContextSlot::GrcHistory as usize].allocated_tokens,
        FLOOR_GRC_HISTORY,
    );
    assert!(
        budget.slots[ContextSlot::OutputReserve as usize].allocated_tokens >= FLOOR_OUTPUT_RESERVE,
        "OutputReserve floor violated: {} < {}",
        budget.slots[ContextSlot::OutputReserve as usize].allocated_tokens,
        FLOOR_OUTPUT_RESERVE,
    );
}

// ---------------------------------------------------------------------------
// 5. Cache breakpoints valid after identical content
// ---------------------------------------------------------------------------

#[test]
fn cache_breakpoints_valid_after_identical_content() {
    let mut budget = ContextBudget::compute(MODEL_MAX_128K, [4096, 2048, 2048], 2000)
        .expect("compute must succeed");
    let slices = make_8_slices(b"identical-content-for-all-slots");

    // First call: initializes hashes, both breakpoints invalid.
    budget.update_breakpoints(&slices);
    assert!(
        !budget.cache_breakpoints[0].valid,
        "first call: BP0 must be invalid"
    );
    assert!(
        !budget.cache_breakpoints[1].valid,
        "first call: BP1 must be invalid"
    );

    // Second call with identical content: both breakpoints valid.
    budget.update_breakpoints(&slices);
    assert!(
        budget.cache_breakpoints[0].valid,
        "second identical call: BP0 must be valid"
    );
    assert!(
        budget.cache_breakpoints[1].valid,
        "second identical call: BP1 must be valid"
    );
    assert_ne!(
        budget.cache_breakpoints[0].prefix_hash, 0,
        "BP0 hash must be non-zero"
    );
    assert_ne!(
        budget.cache_breakpoints[1].prefix_hash, 0,
        "BP1 hash must be non-zero"
    );
}

// ---------------------------------------------------------------------------
// 6. Cache breakpoints invalidated on content change
// ---------------------------------------------------------------------------

#[test]
fn cache_breakpoints_invalidated_on_content_change() {
    let mut budget = ContextBudget::compute(MODEL_MAX_128K, [4096, 2048, 2048], 2000)
        .expect("compute must succeed");
    let slices_a = make_8_slices(b"original-content");

    // Establish valid breakpoints (two identical calls).
    budget.update_breakpoints(&slices_a);
    budget.update_breakpoints(&slices_a);
    assert!(budget.cache_breakpoints[0].valid, "BP0 should be valid before change");
    assert!(budget.cache_breakpoints[1].valid, "BP1 should be valid before change");

    // Change slot 0 (SystemPrompt) — invalidates both BP0 and BP1.
    let slices_b: [&[u8]; 8] = [
        b"changed-system-prompt",
        b"original-content",
        b"original-content",
        b"original-content",
        b"original-content",
        b"original-content",
        b"original-content",
        b"original-content",
    ];
    budget.update_breakpoints(&slices_b);
    assert!(
        !budget.cache_breakpoints[0].valid,
        "changed slot 0: BP0 must be invalid"
    );
    assert!(
        !budget.cache_breakpoints[1].valid,
        "changed slot 0: BP1 must be invalid (BP1 covers slot 0)"
    );
}

// ---------------------------------------------------------------------------
// 7. Determinism: 100 identical runs produce identical output
// ---------------------------------------------------------------------------

#[test]
fn assembly_deterministic_100_runs() {
    let slices = make_8_slices(b"determinism-test-content");

    let first = {
        let mut b = ContextBudget::compute(MODEL_MAX_128K, [4096, 2048, 2048], 2000)
            .expect("compute must succeed");
        b.update_breakpoints(&slices);
        b
    };

    for run in 1..100 {
        let mut b = ContextBudget::compute(MODEL_MAX_128K, [4096, 2048, 2048], 2000)
            .expect("compute must succeed");
        b.update_breakpoints(&slices);
        assert_eq!(
            first, b,
            "run {run} produced a different ContextBudget than run 0"
        );
    }
}

// ---------------------------------------------------------------------------
// 8. Latency: compute + update_breakpoints < 50ms per iteration (100 runs)
// ---------------------------------------------------------------------------

#[test]
fn assembly_latency_under_50ms() {
    // Instant::now() in #[cfg(test)] scope — entropy quarantine applies to production paths only. // TEST_KEEP
    let slices = make_8_slices(b"latency-bench-content");
    let iterations = 100;

    let start = Instant::now(); // TEST_KEEP
    for _ in 0..iterations {
        let mut b = ContextBudget::compute(MODEL_MAX_128K, [4096, 2048, 2048], 2000)
            .expect("compute must succeed");
        b.update_breakpoints(&slices);
    }
    let elapsed = start.elapsed();

    let budget_per_iter = Duration::from_millis(50);
    let max_total = budget_per_iter * iterations;
    assert!(
        elapsed < max_total,
        "assembly latency exceeded 50ms/iter: total={elapsed:?} for {iterations} iterations (max={max_total:?})"
    );
}

// ---------------------------------------------------------------------------
// 9. Adaptive reduce fires on 20% underestimate
// ---------------------------------------------------------------------------

#[tokio::test]
async fn adaptive_reduce_fires_on_20pct_underestimate() {
    let dir = TempDir::new().expect("tempdir for integration test");
    let ssb = make_ssb_for_integration(&dir);

    let mut budget = ContextBudget::compute(MODEL_MAX_128K, [4096, 2048, 2048], 2000)
        .expect("compute must succeed");

    // Set actual_tokens = allocated_tokens for all slots first.
    for slot in &mut budget.slots {
        slot.actual_tokens = slot.allocated_tokens;
    }

    // Inject 20% overestimate on CodeContext (slot 4) to trigger adaptive reduction.
    let code_ctx = &mut budget.slots[ContextSlot::CodeContext as usize];
    code_ctx.actual_tokens = code_ctx.allocated_tokens * 120 / 100;

    // Recalculate total_used to reflect the overflow.
    budget.total_used = budget.slots.iter().map(|s| s.actual_tokens).sum();

    let result = budget
        .adaptive_reduce(0.5, ExecutionMode::Interactive, &ssb)
        .await;
    assert!(
        result.is_ok(),
        "adaptive_reduce should succeed for 20% overflow: {result:?}"
    );
    assert_eq!(
        budget.adaptive_level,
        AdaptiveLevel::FilteredByRelevance,
        "20% overflow on single slot should trigger level 1 (FilteredByRelevance), not higher"
    );
}

// ---------------------------------------------------------------------------
// 10. Entropy quarantine: no Instant::now in production context_budget.rs
// ---------------------------------------------------------------------------

#[test]
fn entropy_quarantine_no_instant_in_production() {
    // Compile-time grep: entropy quarantine verification per 3E.7 ADVERSARIAL.
    let production_source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../orchestrator/src/context_budget.rs"
    ));
    assert!(
        !production_source.contains("Instant::now"),
        "entropy quarantine violated: context_budget.rs must not use Instant::now()" // TEST_KEEP
    );
}
