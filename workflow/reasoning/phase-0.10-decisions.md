Phase 0.10 decisions

What: Verification gate for entropy quarantine. Add ELAI_SEED env-var mechanism, determinism proof script, metrics.

Key decision: Option A chosen — add RuntimeContext::from_env() to core/src/lib.rs.
Reason: Spec requires "run test suite with ELAI_SEED=42" meaning the env var must propagate into RuntimeContext construction so all code using RuntimeContext gets seeded providers. Option B (standalone test) would only prove SeededClock/SeededRng work in isolation, not that the workspace test suite uses them. s-377540 finished and released file ownership.

Expected test outcome: from_env() with ELAI_SEED=42 returns SeededClock+SeededRng(42). Without ELAI_SEED, falls back to os_default(). Integration test verifies determinism (same seed = same sequence).

Unknowns: Some workspace tests may not use RuntimeContext at all (they test pure logic). The determinism proof only covers tests that go through RuntimeContext. Pre-existing 40 Windows failures are excluded from scope.
