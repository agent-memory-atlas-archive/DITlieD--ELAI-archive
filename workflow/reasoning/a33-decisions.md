A-33 PI Tier 2 Completions — Decision Doc

What changes:
1. core/project_profile.rs: add ClassificationDepth enum, PinningStrategy enum, DependencyProfile struct, dep_profile field on ProjectProfile
2. knowledge/conventions.rs: enhance detect_error_handling_regex for Python custom exceptions, TypeScript fp-ts/neverthrow, Go fmt.Errorf %w; add dep_profile to ConventionStats
3. rules/project_rule_seeder.rs: add check_and_promote_seeded_rules and check_and_retire_seeded_rules calling lifecycle.rs public API

Key decisions:
- DependencyProfile uses serde(default) + Default impl for backward compat
- Per-language error detection enhances existing regex scanner, no new struct fields needed — ErrorHandlingProfile already captures pattern/propagation/error_crate
- Promote function: compute_metrics_snapshot -> check InsufficientShadowData (CCR=None for mutable) -> check success count -> call transition_to_active
- Retire function: check FP count from metrics -> call transition_to_retired
- Named constants: AUTO_PROMOTE_SUCCESSES=5, AUTO_RETIRE_FP_COUNT=3 (from spec section 6)
- Lifecycle functions use lifecycle.rs public API only (no internal modifications)

Expected test outcome:
- All pre-existing 4/4 pi_tier2_seeder_contract.rs tests GREEN
- New contract tests for DependencyProfile default, promote/retire logic GREEN
- cargo check -p elai-knowledge -p elai-rules -p elai-core PASS

Unknowns:
- Whether test_profile() in existing tests needs dep_profile field (serde(default) should handle)
