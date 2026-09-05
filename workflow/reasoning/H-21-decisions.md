H-21 runtime look-ahead / causality differential gate -- decisions
Session: s-8404
Date: 2026-06-05
FSM: EXECUTING

WHAT
New pure-Rust perturb-and-diff gate: crates/orchestrator/src/lookahead_differential.rs.
diff_future_truncation(ssb, input) diffs a FULL-data feature column-set against a
FUTURE-TRUNCATED one, column by column, with a relative+absolute FP tolerance. Any
column that changes under truncation proves the pipeline read the future (FutureLeak ->
Block). Columns that diff to zero are Causal (Allow). Emits one LookaheadDifferential SSB
row on every decision (off-agent artifact, R4). The un-gameable runtime backstop to the
H-20 static leakage COMPOSITE (R5).

KEY DECISIONS
1. PURE DIFF, NO DKG. Unlike the H-18/H-20 siblings, H-21 seeds NO rules.db row, has NO
   runtime_setup caller, NO DKG predicate, NO suppression registry. It is a direct Rust
   diff over two column-sets. Building any of those would be wire-dark cargo-culting (the
   gate would query a predicate path it never executes). The ONLY production entrypoint is
   the `elai lookahead-check` CLI subcommand (W-1/W-3, precedent: H-18/H-20 `*-check` CLI
   IS the production reach; the plan's "fire from promote()" is aspirational/deferred).

2. TOLERANCE CONSTANTS (correctness-sensitive, named, documented once):
   DEFAULT_RELATIVE_TOLERANCE = 1e-9, DEFAULT_ABSOLUTE_TOLERANCE = 1e-12.
   Test: |a-b| <= max(abs, rel*max(|a|,|b|)).
   - 1e-9 ~= 30 bits of mantissa agreement: LOOSER than f64::EPSILON (~2.2e-16, which is
     1 ulp at 1.0 and guarantees false positives on benign FMA/SIMD reassociation), TIGHTER
     than a real leak (an iloc[t+1] feature changes by O(price-magnitude), a relative delta
     >> 1e-9). numpy.isclose's default rtol is the looser 1e-5; we tighten to 1e-9 because
     the seed is fixed (ELAI_SEED same-env, GD-2) so cross-RUN noise is ~0 and the only
     residue to absorb is cross-HARDWARE.
   - This is H-21's OWN choice (TD-5 honest residue), NOT borrowed from Freqtrade.
     Freqtrade's lookahead.py uses EXACT DataFrame.compare() (any bit-difference), verified
     2026-06-05 from github.com/freqtrade/freqtrade; exact compare yields cross-HW false
     positives, so H-21 adds tolerance. The diff is NEVER asserted bitwise-exact across HW.
   - GD-4 CAVEAT (do NOT over-claim): the FP false-positive rate has NOT been bounded on
     >=3 real causal pipelines this run (Tier-2 deferred -> no real pipelines available).
     The constant is shipped as a LOCKED named value with the fixture proving BLOCK/ALLOW;
     the validated FP-rate bound is a Tier-2/RunPod-pending follow-up. No validated FP rate
     is claimed.

3. NaN/inf handling is EXPLICIT (a financial gate must not panic on bad data):
   - bit-identical values (incl. 0.0/-0.0, same-sign inf) are close;
   - a NaN on either side is NEVER close (NaN != NaN is itself a flagged divergence, not a
     panic -> a NaN column is reported as a FutureLeak, not silently equal);
   - mismatched infinities (+inf vs -inf, inf vs finite) are a divergence.

4. FAIL-CLOSED (R1/R2/W-1) -- three hard Blocks guarded at the TOP of the gate, each emits
   an SSB row, NEVER a silent pass:
   - truncated_run None/empty -> Unprovable, reason "missing_truncated";
   - column-set width OR name (same-position) mismatch -> Unprovable, reason
     "column_set_mismatch" (a missing/extra column IS a divergence, not inner-joined away);
   - per-column row-count mismatch -> Unprovable, reason "shape_mismatch".
   The fail-closed Unprovable verdicts map to the FutureLeak SSB Kind (they BLOCK); the
   exact reason is preserved in the payload `verdict` field.

5. FIRST-DIVERGENCE shape reused from determinism.rs:364 DriftComparator::compare (one
   report at the first mismatch; identical => no divergence), lifted to (column, index)
   over f64 with the rel_close comparator instead of `!=`. determinism.rs has NO FP helper
   (grep zero), so FpTolerance + first_divergence are NEW code in the H-21 file.

6. TIER-2 DEFERRED (cloud-only override this run). The Python feature-runner
   (run_pipeline_twice: re-executes a real strategy on full vs future-truncated data and
   produces the two column-sets) is RunPod-pending. NO stub shipped. The Tier-1 column-sets
   are supplied as plain data; the demo path constructs them in Rust. A
   `// cross-lang-skip: rust-py-sidecar` header on lookahead_differential.rs opts the file
   out of the Rust<->Python pair drift check (cross_lang_diff.py) until Tier-2 ships.

SSB LOCKSTEP (+LookaheadDifferential, Pattern-2 single-variant, inner 2-arm Kind):
SCHEMA_VERSION 225 -> 226 (ssb.rs); TOTAL_VARIANTS 502 -> 503 (action_type_lock.rs);
e45 contract literal 502 -> 503; test_typed_payloads count 422 -> 423. 14 sites touched:
ssb.rs header comment, const SCHEMA_VERSION, SQLite CHECK IN-list (the easy-to-miss site;
added a comma after 'leakage_blocked' then 'lookahead_differential'), SsbActionType variant,
as_sql_str arm, from_str arm, SsbPayload variant, action_type() match arm, the
LookaheadDifferentialKind enum + LookaheadDifferentialPayload struct, all_test_payloads
entry, the count assertion; action_type_lock.rs header + TOTAL_VARIANTS + ALL_VARIANTS +
the exhaustive match arm at ~2448; e45 literal. The action_type_lock.golden file is
sorted+ordinal-renumbered, regenerated via the regen_golden_file --ignored test runner
(not hand-edited).

EXPECTED TEST OUTCOME
- cargo check -p elai-orchestrator -p elai-cli + --workspace: PASS.
- h21_lookahead_differential_contract.rs (7 cases) + lookahead_differential unit tests +
  action_type_lock golden + e45 contract: PASS.
- `cargo run -p elai-cli -- lookahead-check --demo-future-leak`: exits reporting a BLOCK and
  writes a real LookaheadDifferential(FutureLeak) row to ssb.db.

UNKNOWNS / RESIDUE
- GD-4 FP-rate bound on real pipelines: deferred (Tier-2 RunPod-pending). Not claimed.
- gate-manifest.txt freeze entry for lookahead_differential.rs: requires an OUT-OF-BAND
  `gate_freeze_check.py --lock` re-run (human-reviewed, R8); the executing run cannot
  re-lock its own gate. Flagged to user; NOT done this run.
- Tier-2 Python lookahead_runner.py: not built (cross-lang-skip applied).
