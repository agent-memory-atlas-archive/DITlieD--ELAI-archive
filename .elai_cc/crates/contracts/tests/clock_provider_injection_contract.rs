//! C-15 — ClockProvider injection contract test.
//!
//! Locks the new `instant_now()` surface on the `ClockProvider` trait plus
//! the SeededClock tick semantics that downstream determinism guarantees
//! depend on (see C-15 plan task 9 / D6).
//!
//! Coexists with `clock_entropy_contracts.rs` (the broader 13-test suite);
//! this file adds 4 fresh assertions targeting `instant_now()` and
//! tick-start=0 specifically, so the two files do not shadow.
//!
//! Reference: docs/thefinalglance/C-plan/C-15_phase0_debt_clock_entropy.md
//!            workflow/reasoning/C-15-decisions.md

use std::time::Instant;

use elai_core::clock::{ClockProvider, OsClock, SeededClock};

/// Assertion 1 — SeededClock.now() returns tick=0 on the FIRST call of a
/// fresh instance, regardless of any ELAI_SEED env value. ELAI_SEED only
/// seeds the RNG path (see entropy.rs), not the tick counter.
#[test]
fn seeded_clock_tick_starts_at_zero() {
    let clock = SeededClock::new();
    let first = clock.now();
    assert_eq!(
        first.tick(),
        0,
        "SeededClock::now() must return tick=0 on the first call of a fresh \
         instance (see C-15.2 doc: tick counter is independent of ELAI_SEED)"
    );
}

/// Assertion 2 — Two independent SeededClock instances produce identical
/// tick sequences. This is the byte-determinism contract that injected
/// SeededClock provides over OsClock: same seed, same output.
#[test]
fn seeded_clocks_produce_identical_tick_sequences() {
    let a = SeededClock::new();
    let b = SeededClock::new();
    let seq_a: Vec<u64> = (0..5).map(|_| a.now().tick()).collect();
    let seq_b: Vec<u64> = (0..5).map(|_| b.now().tick()).collect();
    assert_eq!(
        seq_a, seq_b,
        "two fresh SeededClock instances must yield identical tick sequences"
    );
    assert_eq!(seq_a, vec![0, 1, 2, 3, 4], "ticks must advance by exactly 1 per call");
}

/// Assertion 3 — `SeededClock::instant_now()` (new in C-15) does not panic
/// and returns an `Instant` that is monotonic relative to a reference
/// `Instant` captured BEFORE the call. Because the seeded `instant_now()`
/// is `seeded_instant_base() + Duration::from_millis(tick)`, the returned
/// value is `>= seeded_instant_base()` which was captured at process start.
/// We assert the weaker but verifiable invariant: a synthetic instant is
/// produced without panicking, and successive calls are monotonic.
#[test]
fn seeded_clock_instant_now_is_monotonic() {
    let clock = SeededClock::new();
    let _ = clock.now(); // tick -> 1
    let i1 = clock.instant_now();
    let _ = clock.now(); // tick -> 2
    let i2 = clock.instant_now();
    // i2 is derived from a higher tick than i1, so it must be >= i1.
    assert!(
        i2 >= i1,
        "SeededClock::instant_now must be monotonic: i2={i2:?} >= i1={i1:?}"
    );
}

/// Assertion 4 — `OsClock::instant_now()` returns a value within a small
/// window of `Instant::now()` taken just before. Sanity check, NOT a
/// determinism contract: real wall-clock semantics are still in play here.
#[test]
fn os_clock_instant_now_matches_wall_clock() {
    let clock = OsClock;
    let before = Instant::now();
    let from_clock = clock.instant_now();
    let after = Instant::now();
    assert!(
        from_clock >= before && from_clock <= after,
        "OsClock::instant_now (={from_clock:?}) must fall within [before={before:?}, after={after:?}]"
    );
}

/// Bonus assertion — `now_ms()` and `now_secs()` convenience methods agree
/// with each other to within rounding (ms / 1000 == secs). Locks the C-15
/// trait extension surface so downstream callers don't break under refactor.
#[test]
fn os_clock_now_ms_and_now_secs_consistent() {
    let clock = OsClock;
    let ms = clock.now_ms();
    let secs = clock.now_secs();
    let derived = u128::from(secs);
    let delta_secs = (ms / 1000).saturating_sub(derived);
    // Allow up to 1 second of drift between the two consecutive measurements.
    assert!(
        delta_secs <= 1,
        "now_ms()/1000 ({}) must agree with now_secs() ({}) within 1s",
        ms / 1000,
        secs
    );
}
