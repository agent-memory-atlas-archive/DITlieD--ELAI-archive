H-20 leakage static-analysis COMPOSITE gate — decisions (s-8404, 2026-06-05)

WHAT
Built the Tier-1 fail-closed leakage gate FIN-LEAK-001: a rules-engine COMPOSITE
(GraphReachability over the DKG) that BLOCKs ML-pipeline / backtest promotion
when a statically-SOUND data-leakage class fires, plus a thin orchestrator
driver (evaluate_leakage) + a CLI entrypoint (elai leakage-check) that emits the
off-agent LeakageBlocked SSB row on every decision.

KEY DECISIONS

D1 (architecture, RESOLVED by read-batch, not re-litigated): COMPOSE with the
SHIPPED rules-engine Predicate::GraphReachability over the DKG. Did NOT build a
from-scratch dataflow pass and did NOT use the G-1 taint_flow substring side-
table as the reachability engine (wrong shape for a derives-from property). The
fit-reaches-BOTH-sinks property is two positive reachability queries (fit->train
AND fit->test); both non-empty => leak. Grounding: exploration/H20-leakage-
integration.md headline decision; predicate.rs:144 GraphReachability.

D2 (FAIL-CLOSED empty-DKG, the #1 hazard): with must_not_exist=true an empty
graph reads as "no path -> ALLOW" = fail-open. The gate REFUSES this. Two-leg
guard at leakage_gate.rs Step 1: (a) caller threads dkg_available (db present);
(b) a positive presence-probe (must_not_exist=false, wildcard sink) confirms the
DKG actually has reachable nodes for the module. Either leg false => verdict
DkgUnavailable (hard Block, reason "dkg_unavailable"), never the permissive
security-side fallback. Proven by contract cases (3a) empty-querier and (3b)
dkg_available=false, plus unit tests empty_dkg_is_fail_closed_block /
dkg_available_false_is_fail_closed_block. Grounding: exploration:75 (EmptyDkg
fail-open hazard); wiring-enforcement R1/R2.

D3 (soundness boundary, LOAD-BEARING): the COMPOSITE is statically SOUND for
ONLY 3 of the 8 Kapoor classes — L1.1 (no test set), L1.2 (preprocessing before
split), L1.3 (feature-selection before split). The verdict enum
LeakageVerdictKind distinguishes SoundBlock(SoundLeakClass) from
HumanReviewRequired(ResidueClass). The residue — L1.4 duplicates, L2
illegitimate features, L3.1 temporal, L3.2 nonindependence, L3.3 sampling-bias —
is NEVER a sound block; it routes to human Layer-3. The missing-.shift(1) AST
idea is a HEURISTIC temporal (L3.1) flag => HumanReviewRequired, NOT a sound
block. Recall is bounded even for the sound classes (Yang ~67.8%); H-21 runtime
differential is the un-gameable backstop. Grounding: kapoor-narayanan-leakage-
taxonomy-interface.md:35-42,73-76 (arXiv:2207.07048).

D4 (REGISTERED suppression, never silent): created a NEW frozen registry
.elai_cc/leakage-suppression-allowlist.toml (no per-rule suppression existed).
A sound block is downgraded ONLY when a matching row applies (rule_id ==
FIN-LEAK-001 AND file matches AND reason+owner non-empty AND unexpired). Absence
of a row => the block STANDS (no silent skip). The downgraded verdict carries
suppressed=true (audited). Proven by contract cases (4a) unregistered stays
blocked, (4b) registered downgrades, (4c) wrong rule_id does not apply, plus
unit expired_suppression_does_not_apply. Grounding: wire-dark-allowlist.toml
schema (R8/R9). NOTE: the registry must be added to .claude/gate-manifest.txt
and locked OUT-OF-BAND (the executing run cannot re-lock its own gate, R8).

D5 (entrypoint = CLI, OPT-A): mirrored H-18 exactly — elai leakage-check is the
W-1 production entrypoint (lowest-risk; sibling proves the shape). It opens a
real Ssb, constructs a real DkgQuerierAdapter over the project DKG db
(fail-closed if absent), loads the frozen allowlist, runs evaluate_leakage,
emits the row. A --demo-fit-reaches-both flag drives a deterministic in-memory
DKG so the sound-block path is exercisable without a built graph. OPT-B
(pre-promotion / PreToolUse hook) deferred to the H-21 follow-up. Grounding:
deflation_check_cmd.rs (the W-3 sibling); exploration:23-25.

D6 (production seeding, R6): added seed_leakage_rules to project_rule_seeder.rs
(mirror seed_deflation_rules, domain="trading", mutable=0, idempotent) and CALL
it from runtime_setup.rs right after the deflation seed (~:366). This makes
FIN-LEAK-001 DB-backed, not wire-dark — the new pub fns have a production
caller, not just test callers.

SSB LOCKSTEP (+LeakageBlocked, Pattern-2 single-variant, inner 4-arm
LeakageBlockedKind): bumped SCHEMA_VERSION 224->225, TOTAL_VARIANTS 501->502,
all_test_payloads 421->422, e45 pin 501->502. 14 sites touched (the context-
package enumerated 13; the 14th — the SQLite action_type CHECK constraint at
ssb.rs:4864 — was found at runtime when the off-agent row failed to write, and
added). Golden file action_type_lock.golden regenerated (502 lines) via the
#[ignore] regen test.

EXPECTED TEST OUTCOME vs ACTUAL
Expected: leakage unit + h20 contract + action_type_lock + e45 GREEN; no SSB
regressions. Actual: leakage_gate 8/8, h20 contract 9/9, action_type_lock 2/2
(+1 ignored), test_typed_payloads 1/1, e45 14/14 ALL PASS. Orchestrator lib
3472 passed / 12 failed — the 12 are PRE-EXISTING (text/mutation/commit-prefix
tests in files H-20 never edited; zero SSB coupling). cargo check --workspace
compiles.

UNKNOWNS / RESIDUE
- gate-manifest.txt R8 lock for leakage-suppression-allowlist.toml is OUT-OF-
  BAND (controller does the re-lock; the executing run cannot lock its own gate).
- DKG dataflow-edge coverage: GraphReachability is only as sound as the DKG's
  def-use/dataflow edges. If the Python indexer emits "call" edges but not
  dataflow edges for fit->sink, the sound classes degrade. The fail-closed
  empty/unpopulated guard catches the total-absence case; partial-edge coverage
  is a DKG-indexer concern tracked for H-21 (runtime differential backstop).
- OPT-B pre-promotion / PreToolUse leg deferred.
