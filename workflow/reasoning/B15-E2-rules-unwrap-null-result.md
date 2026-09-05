B-15 Tranche E Item E.2 -- rules crate .unwrap() burndown NULL-RESULT
Session: s-22600 sub-phase trancheE | Date: 2026-05-31

Verdict: NULL-RESULT (no source edits). Stub B15-unwrap-burndown for the rules crate is VERIFIED-COMPLIANT.

Scope examined
- .elai_cc/crates/rules/src/ (all 14 files; .unwrap / .expect production-side counts).

Method
- For each file containing .unwrap() or .expect(), locate the canonical `#[cfg(test)]` boundary via Grep.
- Verify per file with: `head -n $((cfg_test_line - 1)) FILE | grep -c '\.unwrap()'` and same for `\.expect(`.
- All 8 files containing unwraps return 0 production-side hits.

Per-file confirmation (cfg(test) boundary line | production .unwrap | production .expect)
- self_discovery.rs       | L828  | 0 | 0
- contract_tests.rs       | L7    | 0 | 0 (entire-file test module)
- rule_engine.rs          | L890  | 0 | 0
- lifecycle.rs            | L572  | 0 | 0
- interrogation_store.rs  | L657  | 0 | 0
- predicate.rs            | L496  | 0 | 0
- schema.rs               | L570  | 0 | 0
- project_rule_seeder.rs  | L500  | 0 | 0

Files with zero .unwrap / .expect (production OR test): error.rs, green_contract.rs, lib.rs, policy_engine.rs, stubs.rs, trust_resolver.rs.

Canonical error types (already in place)
- RulesDbError @error.rs:3-15 -- thiserror, `#[from] rusqlite::Error` + SchemaVersion / TreeDeserialize / TreeValidation arms.
- LifecycleError @error.rs:18-63 -- thiserror, `#[from] rusqlite::Error` + 11 named arms (RuleNotFound, InvalidTransition, InsufficientContractTests, ContractTestsFailing, CircularTrust, InsufficientShadowData, ThresholdNotMet, SafetyRegression, ImmutableRuleCannotAutoRetire, UnknownStatus, UnknownSource).

CLAUDE.md hard rule satisfied
- "No .unwrap() in production/library Rust code. Acceptable ONLY in: test code (#[cfg(test)]/tests/), main()/entrypoint/bootloader init, const/static init with provably valid literals."
- 100% of the 287 .unwrap() and 13 .expect() occurrences sit inside cfg(test) blocks -- canonical test-exempt path.

Outcome
- Stub B15-unwrap-burndown corrected scope for the rules crate; close-by-prefix on the s-22300 row.
- No production-side work this tranche.
- Next iteration scope (per separate HEAVY plans, NOT this tranche): crates/orchestrator/ (~600-900 production-side estimate, session_dag.rs worst-single 171); crates/knowledge/ (reranker.rs 47); crates/plugins/; crates/providers/.
- Optional polish pass (.unwrap() -> .expect("descriptive") for cfg(test) test-failure readability) is REJECTED this tranche per token-discipline; out of scope.
