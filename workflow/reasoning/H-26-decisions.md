H-26 decisions -- shadow-exec + financial-invariant metamorphic-relation gate (T-10)
Session: s-8404 | Date: 2026-06-05 | SCHEMA_VERSION 232 (live final 234 incl C-36)

WHAT
A Tier-1 in-process gate that catches a WRONG VALUE a static/wiring gate cannot see:
a flipped P&L sign, a violated accounting identity, a put-call-parity break, a ULP
cancellation. Two legs:
1. shadow_exec.rs::diff_shadow_impls (PURE Rust): consumes TWO fixture-supplied result
   sets (primary, shadow), diffs their derived float outputs within a LOCKED tolerance
   band, and checks a FIXED set of textbook-correct financial invariants over the
   primary set. Emits ONE DifferentialDivergence SSB row on every decision (off-agent
   artifact). Inner Kind: InvariantHeld(PASS) / ShadowDiverged / InvariantViolated.
2. crates/rules/seed/finance/financial_invariants.toml (SEEDED, mutable=0): a naked-
   float-== AstPattern ban (FIN-INVARIANT-001). The NakedFloatEq Kind is emitted by
   the SEEDED rule-engine path, NOT the pure gate (producer split, mirror H-25's
   MonkeypatchBlocked).

KEY DECISION 1 -- THE EXACTNESS REGIME SPLIT (P0, a wrong regime = false gate)
Conflating exact and float regimes into one tolerance band is a FALSE gate: a band
wide enough to absorb FP slop false-PASSES a cent-level sign error; a band tight enough
to catch it false-FAILS benign FP reassociation. So the regimes are SPLIT:
- ACCOUNTING IDENTITIES are EXACT, checked in INTEGER CENTS (i64) with `==`, NEVER
  float: share_conservation (shares_in == shares_out), cash_equity_identity
  (cash_cents + position_qty*price_cents == equity_cents, i128-guarded against
  overflow), pnl_additivity (sum of per-trade realized_pnl_cents == declared total).
  There is NO decimal/money type in the orchestrator (Cargo.lock has no
  rust_decimal/bigdecimal), so money is integer cents throughout the fixtures + checks.
  Float-==-ing an accounting identity is itself the bug FIN-INVARIANT-001 bans
  (self-consistency, R4) -- the gate must not commit it.
- OPTION-PRICING / NUMERIC MRs are FLOAT, checked within the LOCKED FpTolerance
  (rel=1e-9 / abs=1e-12) REUSED from H-21 lookahead_differential (GD-1, the band IS the
  gate): put_call_parity (European-only), numeric_stability.
- The primary-vs-shadow divergence quantizes each derived float to the tolerance bucket
  and REUSES the DriftComparator first-divergence index (determinism.rs:364, GD-1); the
  LOAD-BEARING tolerance compare stays the FpTolerance::close call inside the gate
  (DriftComparator is byte-equality, the wrong comparator for floats -- the quantize is
  a reporting aid only).

KEY DECISION 2 -- THE 3 WRONG-INVARIANT CAVEATS (P0, encoded in docstrings + TOML)
- PnLAdditivity holds ONLY for INDEPENDENT / uncorrelated positions; it FAILS under
  portfolio margining / cross-gamma / netting. The check + the fixture assume
  independent positions; documented in check_accounting_identities + the SSB doc.
- ScaleInvariance (capital-scaling linearity) holds ONLY sub-market-impact and near
  k=1 / above the min-lot rounding floor. NOT shipped as a hard gate for that reason
  (scope: |k-1|<<1, sub-impact, integer-lot-aware); documented in shadow_exec.rs header.
- PutCallParity EQUALITY is EUROPEAN-only. American options give an INEQUALITY
  (C - P <= S - K*exp(-rT)), not an equality. The MR carries an OptionStyle field; an
  American option is rejected as un-evaluable (Block, not a silent pass), never
  mis-asserted as an equality.

KEY DECISION 3 -- FAIL-CLOSED (R1/R2) + the DEGENERATE-RUN guard
- a MISSING shadow run (ShadowOutcome::Missing) => ShadowDiverged Block (reason
  shadow_missing): a single impl cannot self-certify no wrong value (missing evidence =
  FAIL, never "no diff -> Allow").
- an ERRORED shadow (ShadowOutcome::Errored) => ShadowDiverged Block (shadow_errored):
  a shadow that cannot be diffed cannot prove no-divergence.
- a DEGENERATE run (zero trades AND no derived floats, declared_trade_count <
  MIN_DECLARED_TRADES=1) => InvariantViolated Block (degenerate_run): every accounting
  identity is vacuously true on an empty run, so a "pass" proves nothing (mirror the
  H-23 non-empty-payload + H-25 non-trivial-manifest guards). A contract case proves an
  empty run => Block, not a vacuous pass.
- an un-evaluable invariant (NaN/Inf, an American option for the European parity MR)
  => Block, never a silent pass.

KEY DECISION 4 -- THE SEEDED NAKED-FLOAT-== BAN (evaluator-RUNNING, not inert)
FIN-INVARIANT-001 is a tree-sitter-rust AstPattern banning a naked float ==/!= against
a float LITERAL on the RIGHT operand, SCOPED to the order/signal/pnl/price execution
surface via execution_path:<csv> (CONTAINS semantics, rule_engine.rs:401 -- NOT
starts_with, or the rule fires NEVER / wire-dark). The contract test DRIVES the real
DefaultRuleEngine + TreeSitterAstEvaluator over a `pnl == 0.0` fixture (asserts Block)
and a `(a-b).abs() < 1e-9` fixture (asserts Allow) -- the anti-wire-dark proof (R6, the
H-24/H-25 inert-rule lesson). The query was VERIFIED live against tree-sitter-rust 0.24
before authoring (a probe test): `right:(float_literal)` fires; the grammar REJECTS a
`left:` field ("Impossible pattern"). RESIDUAL (R5, necessary-not-sufficient): a
left-literal (`0.0 == pnl`) and a float-VARIABLE == (`pnl == target`, no literal node,
type unknown to a syntactic query) are NOT caught by this static leg; the LOAD-BEARING
backstop is the integer-cents EXACT regime in diff_shadow_impls, not the AST ban.

GROUNDING (DESIGN-INSPIRED-BY for the DEFERRED Tier-2 only)
FPDiff (floating-point differential testing), Daikon (dynamic invariant MINING), and
RV-Monitor/MOP (runtime monitoring) inspire the DEFERRED Tier-2 dynamic-mining /
monitoring extensions. H-26 Tier-1 checks a FIXED set of textbook-correct invariants --
it does NOT mine or runtime-monitor, so those are NOT implemented here. The float band
REUSES H-21's already-grounded FpTolerance. The Tier-2 python/shadow_runner.py two-impl
re-execution runner is RunPod-pending; NO stub is shipped (cross-lang-skip header on
shadow_exec.rs opts it out of the Rust<->Python pair drift check until it lands).

OFF-AGENT ARTIFACT (R4)
+DifferentialDivergence Pattern-2 single-variant (inner 4-arm Kind). H-26 adds EXACTLY
ONE action_type. Per-invariant float-MR failures REUSE MrViolated (GD-1, no new variant
for those). The CLI `elai shadow-check (--demo-invariant-held | --demo-invariant-
violated)` is the W-1 production entrypoint; both demos opened a real Ssb and emitted a
real differential_divergence row (PROVEN: 2 rows landed in the live db, the violated arm
exited 1, the held arm exited 0).

SSB LOCKSTEP (live baseline drift found + reconciled)
The brief's stated baseline (230/507) was STALE. Live truth at edit time: SCHEMA=231,
TOTAL_VARIANTS=508 (C-21 SteeringVectorRefreshTriggered had landed after H-25 but had
NOT updated the e45 pin, leaving e45 RED at 507). Worse, C-36 (MucocoInconsistency-
Detected + MucocoSkipReason) was a PRE-EXISTING half-applied uncommitted change: its
enum variants, payloads, SCHEMA bumps (to 234), golden rows, and CHECK entries were in
the tree, but action_type_lock.rs TOTAL_VARIANTS was left STALE at 508 (the lock test
would have failed). H-26 inserted DifferentialDivergence (SCHEMA 232) cleanly BEFORE the
C-36 variants in declaration order; the regen helper reconciled TOTAL_VARIANTS to 511
(runtime truth = 508 base + my +1 + C-36's +2). Final self-consistent state:
SCHEMA_VERSION=234, TOTAL_VARIANTS=511, e45 pinned 511. H-26's contribution is EXACTLY
+1 (DifferentialDivergence at 232). The C-36 +2 reconciliation is a found-and-fixed
pre-existing inconsistency, not an H-26 deliverable.

EXPECTED TEST OUTCOME (achieved)
- h26 contract 8/8 (consistent PASS; PnL-additivity + share-conservation integer-cents
  Block; shadow-divergence Block; degenerate Block; naked-float-== AST FIRES + benign
  Allow; SCHEMA>=232 + bijection).
- shadow_exec unit 15/15 (each invariant integer-cents EXACT + float-tolerance + the
  divergence diff + the fail-closed guards + the European/American parity split).
- action_type_lock 2/2 (golden regenerated, count locked at 511); typed-payloads 1/1
  (429 variants incl DifferentialDivergence); e45 14/14; elai-rules 536/536; h24 10/10;
  h25 8/8 (no regression).

UNKNOWNS / RESIDUAL
- The FpTolerance band (rel=1e-9) is inherited from H-21 (TD-5, H-21's own choice); the
  GD-4 false-positive-rate calibration over >=3 real refactors is a Tier-2 follow-up,
  not validated this run.
- The naked-float-== AST leg covers the right-literal idiom only (grammar limit);
  left-literal / float-variable forms are residual (the integer-cents EXACT regime is
  the sound backstop).
- gate_freeze_check.py --lock is OUT-OF-BAND (R8): the auditor / main model re-locks the
  manifest, not this executor run. The manifest LIST entries (shadow_exec.rs +
  financial_invariants.toml) were added; the re-hash lock was NOT run.
