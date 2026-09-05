Phase 3F.4 decisions — rule lifecycle state machine

Decision A: schema version strategy
SCHEMA_VERSION bumps from 1 to 2. The rule_transitions table is added in a new
1->2 migration block inside run_migration(). Uses CREATE TABLE IF NOT EXISTS for
idempotency. All existing tests use in-memory DBs that auto-migrate via open_rules_db,
so the bump is safe. No ALTER TABLE needed — existing 4 tables are unchanged.

Decision B: migration safety
The 0->1 block remains unchanged. A new if current < 2 block adds rule_transitions
DDL and inserts schema_version row for version 2. The version insert uses INSERT
(not INSERT OR REPLACE) because each version gets its own row. Existing callers
(open_rules_db, contract tests) all pass through run_migration which handles both
0->1 and 1->2 paths.

Decision C: approver tracking and circular trust
Approver is an enum with 3 variants: Human, SecondAuditor{auditor_id}, FindingSourceAuditor{auditor_id}.
Circular trust is enforced at the call site in transition_to_shadow:
FindingSourceAuditor is always rejected with LifecycleError::CircularTrust.
The approver_id string is logged in the rule_transitions row (nullable TEXT column).
No new column in the rules table. No DB lookup for approver validation.

Decision D: transition_id generation
Format: "{rule_id}:{to_status}:{timestamp}" — deterministic, no uuid crate needed.
Collision requires same rule transitioning to same status at the same epoch-second,
which is impossible under normal operation (clock advances monotonically).

Decision E: CCR zero-denominator guard
When causal_catches=0 AND spurious_catches=0, counterfactual_causal_rate is None.
transition_to_active checks: if ccr is None, return InsufficientShadowData.
This covers the case where no shadow evaluations have occurred.

Decision F: mutable=0 retirement guard
mutable=false rules cannot be auto-retired (force=false returns MutableRuleCannotAutoRetire).
force=true bypasses this check for human-initiated retirement only.
The force flag and reason are logged in the rule_transitions row.

Decision G: static dispatch
All lifecycle functions use &impl ClockProvider (static dispatch), matching the
established project pattern in schema.rs. No Arc<dyn> or &dyn.
