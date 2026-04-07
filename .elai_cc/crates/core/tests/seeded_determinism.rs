use elai_core::{EntropySource, RuntimeContext, SeededClock, SeededRng};
use std::sync::{Arc, Mutex};

/// Process-level guard to serialize tests that mutate ELAI_SEED.
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn from_env_with_seed_uses_seeded_providers() {
    let _guard = ENV_MUTEX.lock().unwrap();
    unsafe { std::env::set_var("ELAI_SEED", "42") };
    let ctx = RuntimeContext::from_env();
    unsafe { std::env::remove_var("ELAI_SEED") };

    // SeededClock starts at tick 0, increments each call
    let t1 = ctx.clock().now();
    let t2 = ctx.clock().now();
    assert!(t2 > t1, "SeededClock must be strictly monotonic");

    // SeededRng is deterministic: same seed = same first value
    let val1 = ctx.entropy().lock().unwrap().next_u64();

    // Build a second context with the same seed to verify determinism
    let entropy2 = Arc::new(Mutex::new(SeededRng::new(42)));
    let val2 = entropy2.lock().unwrap().next_u64();
    assert_eq!(val1, val2, "Same seed must produce same first value");
}

#[test]
fn from_env_without_seed_falls_back_to_os() {
    let _guard = ENV_MUTEX.lock().unwrap();
    unsafe { std::env::remove_var("ELAI_SEED") };
    let ctx = RuntimeContext::from_env();

    // OsClock returns real instants
    let t1 = ctx.clock().now();
    let t2 = ctx.clock().now();
    assert!(t2 > t1, "OsClock must be monotonic");
}

#[test]
fn seeded_rng_determinism_across_contexts() {
    let seed = 42u64;
    let ctx_a = RuntimeContext::new(
        Arc::new(SeededClock::new()),
        Arc::new(Mutex::new(SeededRng::new(seed))),
    );
    let ctx_b = RuntimeContext::new(
        Arc::new(SeededClock::new()),
        Arc::new(Mutex::new(SeededRng::new(seed))),
    );

    let mut seq_a = Vec::with_capacity(10);
    let mut seq_b = Vec::with_capacity(10);
    for _ in 0..10 {
        seq_a.push(ctx_a.entropy().lock().unwrap().next_u64());
        seq_b.push(ctx_b.entropy().lock().unwrap().next_u64());
    }
    assert_eq!(seq_a, seq_b, "Same seed must produce identical sequences");
}
