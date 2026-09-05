Phase 0.9 Entropy Quarantine decisions

What: Define ClockProvider/EntropySource traits, implement prod+test impls, replace all raw time/entropy access, create CI enforcement script.

Key decisions:
1. Added rand 0.8 + rand_chacha 0.3 to core/Cargo.toml (user confirmed). SeededRng uses ChaCha8Rng::seed_from_u64.
2. OsClock uses global AtomicU64 tick counter to ensure LogicalInstant uniqueness even within same Instant::now() resolution.
3. Transitional injection pattern: production call sites use `&elai_core::OsClock` directly rather than threading RuntimeContext through all function signatures. This achieves quarantine (all raw time access in core/src/clock.rs only) while deferring the full RuntimeContext threading to later phases when struct constructors are refactored.
4. Test helpers (temp_dir/temp_path) marked TEST_KEEP on the SystemTime::now() line itself. CI script greps for TEST_KEEP on the same line to exclude.
5. tools/lib.rs is NOT a facade for time calls. It has its own function implementations (execute_web_fetch, execute_web_search, execute_repl, execute_shell_command, make_agent_id, iso8601_now) that use time directly.
6. LogicalInstant wraps optional Instant (Some for OsClock, None for SeededClock) plus a tick counter. Comparison uses tick-based ordering.
7. No entropy call sites found in current codebase (no thread_rng/OsRng/getrandom). EntropySource trait + OsEntropy + SeededRng created for future use.

Expected test outcome: All existing tests pass. New contract tests verify monotonicity, determinism, object-safety.

Unknowns: session_control.rs while-loop timing (wait_for_next_millisecond) left as TEST_KEEP since it's test-only spin-wait. InternalPromptProgressShared.started_at changed from Instant to LogicalInstant + OsClock field.
