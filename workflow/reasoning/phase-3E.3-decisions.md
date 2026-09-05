Phase 3E.3 decisions

What: BudgetError enum + ContextBudget::compute() with percentage-based allocation and floor enforcement. Combines 3E.2 (percentage split) and 3E.3 (floor enforcement) since 3E.2 was not implemented.

Key decision: Used thiserror (already in orchestrator Cargo.toml as v2) for BudgetError rather than manual Display/Error impl. No new dependency added.

Key decision: Steal order is GrcHistory first, then SsbEntries. Each steals from CodeContext first, then RepoMap. This matches the spec priority (GRC is more critical than SSB for reasoning continuity).

Key decision: Remainder from integer division goes to CodeContext (index 4) as the largest dynamic slot. This ensures zero-sum invariant without rounding errors.

Expected test outcome: 14 unit tests (11 existing + 3 new) + 8 contract tests (5 existing + 3 new) = 22/22 green, 0 warnings.

Unknowns: None. Pure arithmetic, no external deps beyond thiserror.
