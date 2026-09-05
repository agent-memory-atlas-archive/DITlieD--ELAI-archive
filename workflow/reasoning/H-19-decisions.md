H-19 DECISIONS — holdout budget honesty polish

DATE: 2026-07-22
PLAN: H-19 (Thresholdout DP budget binding)
SCOPE: S9 polish slice (GAP-A only)

LOCKED BUDGET_TOTAL + NOISE SCALE RATIONALE

The holdout budget_total and noise scale parameters are locked in holdout_budget.toml (gate-frozen, blake3 in .claude/gate-manifest.txt). The rationale:

- budget_total is the DP epsilon spend ceiling for the holdout set. It is provisioned fresh per objective window, never reused across windows. The locked value reflects a conservative per-objective epsilon allocation that bounds the cumulative privacy loss across all DGM admission rounds within one window.
- noise scale (calibrated from budget_total and the holdout set size) determines the Laplace noise added to each holdout query result. The scale is fixed at provisioning time so the DGM loop cannot adaptively tighten it after seeing results (prevents adaptive noise reduction as a leakage channel).
- Both parameters are human-only: no automated path may mutate them. The holdout_budget.toml file is immutable=0 (seeded by project_rule_seeder.rs), and the debug update_hook now panics on UPDATE/DELETE against the holdout_budget table (GAP-A fix, this slice).

PROVISION-FRESH HUMAN-ONLY

The budget is provisioned fresh by a human operator at the start of each objective window. There is no automated provisioning path. The operator sets budget_total and seed in holdout_budget.toml, then the system reads them at startup. Mutating the table at runtime is a violation of the append-only invariant (now enforced in debug builds via the update_hook guard).

PART 5 RESIDUE

PART 5 of the H-19 plan (the honest residue) documents that the holdout budget binding is complete for the shipped symbols (query_holdout, budget_remaining, HoldoutBudgetEvent). The residual items are:

- GAP-B (min_holdout_size pub(crate)): DEFERRED. holdout_budget.rs is gate-frozen (blake3 in .claude/gate-manifest.txt). Changing pub to pub(crate) requires a human re-lock of the gate manifest. Surfaced separately.
- GAP-D (no_reset_or_provision_method_exists no-op negative test): legs 1 (no method exists) + 3 (gate-manifest freeze) are load-bearing. The negative test is advisory, not the enforcement path.
- GAP-E (fixtures dir missing): tests use inline planted fixtures. Non-blocking.
- GAP-F (bind_holdout_budget DARK): H-47 scope, not H-19.

GAP-A FIX (this slice)

The update_hook in ssb/store.rs (#[cfg(debug_assertions)]) now guards holdout_budget alongside ssb and ssb_files. The doc claim in holdout_budget.rs:69 is now accurate. The test_append_only test in ssb/tests.rs mirrors the production guard and asserts UPDATE/DELETE on holdout_budget increments the violation counter.

GAP-B NOTE

GAP-B (min_holdout_size is a DARK pub fn) was identified in the S3 precheck but is NOT addressed in this slice because holdout_budget.rs is locked in .claude/gate-manifest.txt. The fix (pub -> pub(crate)) requires a human re-lock of the frozen gate manifest. This is deferred to the owner.
