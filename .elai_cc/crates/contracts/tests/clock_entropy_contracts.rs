use std::sync::{Arc, Mutex};
use std::time::Duration;

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::entropy::{EntropySource, OsEntropy};
use elai_core::RuntimeContext;

#[cfg(test)]
mod clock_contracts {
    use super::*;

    #[test]
    fn os_clock_now_is_monotonic() {
        let clock = OsClock;
        let a = clock.now();
        let b = clock.now();
        assert!(b >= a, "OsClock::now must be monotonically non-decreasing");
    }

    #[test]
    fn os_clock_elapsed_since_is_non_negative() {
        let clock = OsClock;
        let mark = clock.now();
        std::thread::sleep(Duration::from_millis(1));
        let elapsed = clock.elapsed_since(mark);
        assert!(elapsed >= Duration::from_millis(1));
    }

    #[test]
    fn os_clock_system_now_returns_post_epoch() {
        let clock = OsClock;
        let sys = clock.system_now();
        let nanos = sys
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time must be post-epoch")
            .as_nanos();
        assert!(nanos > 0);
    }

    #[test]
    fn clock_provider_is_object_safe() {
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let _ = clock.now();
        let _ = clock.system_now();
    }
}

#[cfg(test)]
mod entropy_contracts {
    use super::*;

    #[test]
    fn os_entropy_next_u64_varies() {
        let mut ent = OsEntropy;
        let a = ent.next_u64();
        let b = ent.next_u64();
        // Probabilistic: two consecutive u64 from OS RNG should differ.
        // Chance of collision is 1/2^64.
        assert_ne!(a, b, "OsEntropy::next_u64 consecutive calls must differ");
    }

    #[test]
    fn os_entropy_fill_bytes_non_zero() {
        let mut ent = OsEntropy;
        let mut buf = [0u8; 32];
        ent.fill_bytes(&mut buf);
        assert!(buf.iter().any(|&b| b != 0), "fill_bytes must produce non-zero output");
    }

    #[test]
    fn entropy_source_is_object_safe() {
        let ent: Arc<Mutex<dyn EntropySource>> = Arc::new(Mutex::new(OsEntropy));
        let val = ent.lock().unwrap().next_u64();
        assert_ne!(val, 0); // probabilistic
    }
}

#[cfg(test)]
mod seeded_contracts {
    use super::*;
    use elai_core::clock::SeededClock;
    use elai_core::entropy::SeededRng;

    #[test]
    fn seeded_clock_never_returns_duplicate() {
        let clock = SeededClock::new();
        let mut prev = clock.now();
        for _ in 0..1000 {
            let curr = clock.now();
            assert!(curr > prev, "SeededClock::now must strictly increase");
            prev = curr;
        }
    }

    #[test]
    fn seeded_clock_elapsed_tracks_ticks() {
        let clock = SeededClock::new();
        let mark = clock.now();
        let _ = clock.now();
        let _ = clock.now();
        let elapsed = clock.elapsed_since(mark);
        // 2 ticks after mark, each tick = 1ms by convention
        assert!(elapsed >= Duration::from_millis(2));
    }

    #[test]
    fn seeded_rng_deterministic_with_same_seed() {
        let mut rng_a = SeededRng::new(42);
        let mut rng_b = SeededRng::new(42);
        for _ in 0..100 {
            assert_eq!(rng_a.next_u64(), rng_b.next_u64());
        }
    }

    #[test]
    fn seeded_rng_seed_zero_does_not_panic() {
        let mut rng = SeededRng::new(0);
        let _ = rng.next_u64();
    }

    #[test]
    fn seeded_rng_different_seeds_differ() {
        let mut rng_a = SeededRng::new(1);
        let mut rng_b = SeededRng::new(2);
        let a = rng_a.next_u64();
        let b = rng_b.next_u64();
        assert_ne!(a, b);
    }
}

#[cfg(test)]
mod runtime_context_contracts {
    use super::*;

    #[test]
    fn runtime_context_constructable_and_cloneable() {
        let mut ctx = RuntimeContext::new(OsClock, OsEntropy);
        let ctx2 = ctx.clone();
        // Both should work independently
        let _ = ctx.clock().now();
        let _ = ctx2.clock().system_now();
        let _ = ctx.entropy_mut().next_u64();
    }

    #[test]
    fn runtime_context_clock_returns_concrete_type() {
        let ctx = RuntimeContext::new(OsClock, OsEntropy);
        let clock = ctx.clock();
        let _ = clock.now();
    }
}
