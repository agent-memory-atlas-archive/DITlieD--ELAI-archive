Phase 3F.1 decisions

What: SQLite schema for rules engine — 4 domain tables + schema_version table + versioned migration.
Files: crates/rules/src/schema.rs, crates/rules/src/error.rs, crates/contracts/tests/rules_schema_contract.rs

Key decisions:
1. schema_version TABLE (not PRAGMA user_version) — more queryable, self-documenting, matches spec wording. Differs from SSB pattern intentionally.
2. Promoted rusqlite 0.39 + thiserror 2 to workspace.dependencies — consistency with orchestrator/reasoning crates. Updated orchestrator + reasoning + contracts Cargo.toml to use workspace refs.
3. Counterfactual columns (causal_catches, spurious_catches, edct_pass, edct_fail, confidence_score, last_true_positive_at) included in rule_metrics upfront — avoids ALTER TABLE migration in 3F.4.
4. violation_patterns.pattern_hash is BLOB — hash algorithm (blake3 per user pref, blake2s per spec) is a 3F.7 implementation detail, not a schema concern.
5. epoch_now() is sole library time source, used ONLY for schema_version.applied_at. All domain timestamps (created_at, promoted_at) are caller-provided (entropy quarantine).
6. Stub modules (policy_engine, trust_resolver, green_contract) preserved alongside new schema/error modules.

Expected test outcome: 16 unit tests in schema.rs + 16 contract tests in rules_schema_contract.rs = 32 total, all green.
Unknowns: none for schema-only step.
