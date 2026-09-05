I-20 budgeted mutation scheduler — executor decisions

What
Ship the diff-cone selector + Wilson-CI adaptive sampler + SurvivorDirective emitter + EquivalentMutantCache key extension + MutationBudgetedRun SSB action_type (4-arm Pattern-2) + PBT RelationLibrary adapter + ID-5 baseline TOML, as one vertical slice. EXTENSION on shipped C-28/C-29 substrate.

Key decisions (authoritative controller overrides baked in)
D1 FORK-1 = Option A. Wire into the REAL already-wired mutation gate DeterministicAuditor::check_mutation (deterministic_auditor.rs:736). NOT verifier_stack::run_verdict, NOT search/ladder.rs (unshipped). Cone-select runs BEFORE the RustMutationRunner/PythonMutationRunner .run calls.
D2 Reachability + SSB writer split. check_mutation is SSB-agnostic by design (C-28/C-29 precedent: runner stays writer-free so unit tests need no Ssb). The production seam that OWNS the Ssb writer and calls run_all -> check_mutation is DefaultDeterministicAuditGate::evaluate (deterministic_audit_gate.rs:143). That gate is constructed in production at cli/src/runtime_config.rs + task_runner.rs. So: cone-select + budget compute in check_mutation; MutationBudgetedRun SSB rows emitted from the gate's evaluate tokio::spawn, exactly mirroring the existing emit_mutation_survivors. Every new pub fn is reached non-test from check_mutation (cone-select) and from the gate (emit). R7-clean.
D3 Diff-cone = in-process knowledge::symbol_graph::traversal::compute_impact_radius(&mut SymbolGraphCache, target, depth). NOT the impact_radius.py subprocess (no python on the audit hot path). Deterministic (BTreeSet). When no SymbolGraphCache/graph is available, cone-select degrades to identity (the full changed_files set) — never panics, never fabricates.
D4 Cache key extension = EXTEND EquivalentMutantCache::compute_key with a 4th blake3 input (bound-test-set hash) via a sibling fn compute_key_with_testset (additive; old 3-input keys still computable, old rows simply miss-then-rerun). NO second cache. TTL unchanged (7-day live; plan's 30-day is stale).
D5 ID-5 (LOAD-BEARING). Kill-rate gate threshold read from metrics/mutation_baseline.toml via MutationBudget::load_threshold_pct. First-deploy writes the baseline (write_baseline_if_absent) from the measured kill-rate. NO hard-coded paper constant as a gate (0.80 is a [DESIGN] comment only). Contract arm asserts threshold-reads-from-toml.
D6 PBT adapter = new MetamorphicRelation::PropertyBased arm + parse_relation "property_based" arm + run_all dispatch arm in metamorphic_testing.rs. No new dispatch layer (GD-1/IC-14).
D7 SSB = ONE new action_type MutationBudgetedRun, Pattern-2 single-variant 4-arm MutationBudgetedRunKind {CiCleared, CiFailed, SurvivorDirective, CacheHit}. Mirrors I-19 TddPreflightBlocked exemplar exactly (7 coordinated edits; counters DERIVED from SSB_ACTION_TYPES.len()). LIVE at HEAD: SCHEMA 254 / TV 563 -> 255 / 564 auto-follow. Golden regen by the regen_golden_file mechanism, never hand-edited.
D8 Wilson score CI = standard proportion interval; z = 1.96 (95%, named const Z_SCORE_95). Adaptive sampler without replacement over the deterministic-sorted cone mutant set; early-stop both directions (ci_lower >= threshold -> pass-early CiCleared; ci_upper < threshold -> fail-early CiFailed); also stop when CI half-width < CI_WIDTH_STOP. All named consts, no magic numbers.

Expected test outcome
crates/contracts/tests/i20_mutation_budget_contract.rs green (7+ arms incl real-DB SSB landing, cone-select, Wilson both directions, cache-hit zero-runs, survivor-directive fields, ID-5 threshold-from-toml).
action_type_lock 2/2 (golden regen'd to derived 564).
e45_cross_language_drift_contract green.

Unknowns / forward-deps
SurvivorDirective CONSUMER (generator retry loop) is OUT of I-20 scope (plan line 28). If the directive-builder pub fn has no production consumer at HEAD it is registered WIRE-DARK[I-20-survivor] + stubs.md row (will_resolve_in=I-20-survivor-consumer). The EMIT site (SSB row) IS in scope and IS wired from the gate.
