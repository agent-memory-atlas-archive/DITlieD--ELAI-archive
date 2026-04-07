use elai_core::{ClockProvider, EntropySource, ProductionContext, SeededRng, TestContext};
use std::sync::Mutex;

/// Process-level guard to serialize tests that mutate ELAI_SEED.
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn from_env_returns_production_context() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let ctx = ProductionContext::from_env();

    // OsClock returns real instants
    let t1 = ctx.clock().now();
    let t2 = ctx.clock().now();
    assert!(t2 > t1, "OsClock must be monotonic");
}

#[test]
fn from_seed_uses_seeded_providers() {
    let mut ctx = TestContext::from_seed(42);

    // SeededClock starts at tick 0, increments each call
    let t1 = ctx.clock().now();
    let t2 = ctx.clock().now();
    assert!(t2 > t1, "SeededClock must be strictly monotonic");

    // SeededRng is deterministic: same seed = same first value
    let val1 = ctx.entropy_mut().next_u64();

    // Build a second RNG with the same seed to verify determinism
    let mut rng2 = SeededRng::new(42);
    let val2 = rng2.next_u64();
    assert_eq!(val1, val2, "Same seed must produce same first value");
}

#[test]
fn seeded_rng_determinism_across_contexts() {
    let seed = 42u64;
    let mut ctx_a = TestContext::from_seed(seed);
    let mut ctx_b = TestContext::from_seed(seed);

    let mut seq_a = Vec::with_capacity(10);
    let mut seq_b = Vec::with_capacity(10);
    for _ in 0..10 {
        seq_a.push(ctx_a.entropy_mut().next_u64());
        seq_b.push(ctx_b.entropy_mut().next_u64());
    }
    assert_eq!(seq_a, seq_b, "Same seed must produce identical sequences");
}
