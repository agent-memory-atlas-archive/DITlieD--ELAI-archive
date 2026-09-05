H-27 DECISIONS — gate self-validation (verify-the-verifiers META gate)
Session: s-8404 lineage | Date: 2026-06-05 | FSM: EXECUTING

WHAT WAS BUILT
A pure-Rust planted-fixture driver (crates/orchestrator/src/gate_self_validation.rs) that
mechanically PROVES the 8 Track-3 trust gates each still KILL their own planted mutant:
each gate, driven with the exact known-bad input it is designed to catch (the gate's OWN
demo-violated fixture), MUST return a BLOCK-class verdict. A surviving mutant => the gate
is fail-open => the meta-gate BLOCKs, naming the broken gate. Plus a +1 SSB Pattern-2
action_type (GateSelfValidation) and a standalone CLI entrypoint (elai gate-selfcheck).

KEY DECISIONS (grounded)

D1. The planted mutants ARE the gates' own demo-violated fixtures.
Each Track-3 gate ships a *-check CLI with a --demo-*-violated fixture: the exact input
that SHOULD make it Block. H-27 replicates that fixture as data and drives the gate's REAL
async evaluate_* fn over it. The gate's own demo-violation IS the planted mutant; "killed"
= the gate returned a BLOCK-class verdict. This grounds the mutant corpus in the gates'
own design intent rather than an out-of-band guess. Fixtures replicated:
  - deflation: pbo=None (R1 fail-closed => Block). No clean fixture (DSR Allow path is
    numerically fragile at threshold 0.95). Kill-the-mutant leg only.
  - leakage: in-memory DkgQuerier where fit reaches BOTH sinks (SoundBlock) vs a clean
    DkgQuerier where fit reaches only the train sink (Allow).
  - lookahead: iloc[t+1] future-leak ColumnSet pair (FutureLeak) vs backward-only
    trailing-SMA pair (Causal Allow).
  - trading_null: constant-long signal on the zero-mean market (EdgeSurvivedNull Block)
    vs sign-aligned signal (EdgeCollapsedToNull Allow).
  - rerun: nondeterministic digest pair (Divergent) vs byte-identical (Equal).
  - sim_real: golden-gap event log (ReplayGoldenMismatch) vs byte-identical (Parity).
  - result_manifest: tampered sharpe 2.99 (FabricationDetected) vs true 1.85 (ReDerived).
  - shadow_exec: PnL-additivity broken by 1 cent (InvariantViolated) vs consistent
    (InvariantHeld).

D2. Reuse mutation-score SEMANTICS, NOT MutationRunner (GD-1 corrected).
crates/orchestrator/src/mutation_testing.rs::MutationRunner shells out to cargo-mutants
over disk source files; it cannot score "a gate fn + a planted input." H-27 is a NEW
in-process planted-fixture driver. It reuses only the mutation_score == 100.0 SEMANTICS:
every planted mutant must be killed (no survivors) for the verify-the-verifiers PASS. The
plan/context flagged the GD-1 "reuse the runner" wording as mis-grounded; this is the
correction. MutationRunner is NOT touched.

D3. Pure driver — NO seed/meta rule, NO lifecycle hook, NO MetamorphicRelation MR.
Per the resolved architecture (and the context-package cross-crate-cycle finding), H-27 is
a pure-Rust driver, not a DB rule. domain="meta" is not a DOMAIN_KEYWORD, so a seed/meta
rule would be wire-dark; none was created. The plan's lifecycle.rs:871 hook is in the
rules crate (no orchestrator dep => a hook would create a cross-crate cycle) AND was a test
line; no lifecycle hook was added. The NullBandSidedness MR + validate_for_traps trap leg
were dropped (the band/trap concern is covered by the existing seeded financial_invariants
AST ban that H-26 already ships; H-27's scope is the kill-the-mutant driver). The CLI is
the W-1/W-3 off-agent entrypoint.

D4. Fail-closed everywhere (R1/R2).
  - EMPTY registry => EmptyRegistry Block (a self-validation that checks ZERO gates is
    itself a FAIL — ArchUnit failOnEmptyShould; the empty should-set NEVER emits the
    vacuous 100.0 pass).
  - surviving mutant (gate did not block its violation) => GateFailedToBlock Block,
    naming the gate (the load-bearing catch).
  - over-block (gate blocked its clean fixture) => GateOverBlocked Block, naming the gate.
  - un-runnable gate (evaluate fn errored) => GateUnreachable Block ("not runnable = never
    ran = FAIL," never UNVERIFIED).

SELF-REFERENCE RESIDUE (TD-4) — THE IRREDUCIBLE TOP OF THE STACK
H-27 validates the 8 OTHER Track-3 gates, NEVER itself. A meta-gate that validated itself
would infinite-regress (the validator validating the validator validating…). The verifier-
of-verifiers is therefore the irreducible TOP of the trust stack and is itself UNVERIFIED
by this gate. Its soundness is routed to:
  (a) a human-authored, frozen, out-of-band fixture set (the REGISTRY const + the per-gate
      demo fixtures are authored once, not regenerated per-run), and
  (b) R8 gate-freeze + human review (the run cannot author or edit its own evaluator / its
      own mutants).
This is the same residue as gate_freeze_check.py itself: who checks the freeze-checker? —
the human + git, not another automated gate. We do NOT claim the regress is "resolved"; we
claim it is bounded and routed, which is the honest posture.

RESIDUE REFERENCES (DESIGN-REFERENCE ONLY — no ingest)
  - Barr, Harman et al. 2015, "The Oracle Problem in Software Testing: A Survey" (IEEE TSE).
    The hard-oracle case (no reliable expected output) does NOT bite H-27: our oracles are
    true-by-construction planted fixtures — for each mutant we KNOW the gate must Block
    (the fixture IS a known violation), so the expected verdict is not in question.
  - Inozemtseva & Holmes 2014, "Coverage Is Not Strongly Correlated with Test Suite
    Effectiveness" (ICSE'14). Planted mutants are a PROXY, not a guarantee of catching
    UNKNOWN / real faults. H-27 proves the gates catch the KNOWN violation classes their
    demo fixtures encode, NOT all possible faults. Corpus incompleteness is an accepted
    residual risk; a future surviving real-world fault would be added to the frozen corpus.
  - The core kill-the-mutant premise (a test suite must KILL injected mutants) is grounded
    in Papadakis et al. 2019 (mutation-testing survey) + Segura et al. 2016 (metamorphic-
    testing survey), both already grounded in the project (H-22 / paper-agentless bib).

SSB LOCKSTEP (+GateSelfValidation, Pattern-2 single-variant, inner 5-arm Kind)
LIVE READ at edit time: SCHEMA_VERSION 234, TOTAL_VARIANTS 511, e45 mirror 511.
BUMPED to: SCHEMA_VERSION 235, TOTAL_VARIANTS 512, e45 mirror 512.
14 sites landed (mirroring H-26 DifferentialDivergence exactly):
  1.  ssb.rs SCHEMA_VERSION 234 -> 235 + dated comment block.
  2.  ssb.rs SQLite CHECK(action_type IN (...)) += 'gate_self_validation' (14th lockstep
      site; comma added to the prior-last 'mucoco_skip_reason').
  3.  ssb.rs SsbActionType enum += GateSelfValidation.
  4.  ssb.rs as_sql_str() arm.
  5.  ssb.rs from_str arm.
  6.  ssb.rs SsbPayload enum += GateSelfValidation(GateSelfValidationPayload).
  7.  ssb.rs SsbPayload::action_type() arm.
  8.  ssb.rs payload substrate: GateSelfValidationKind (5-arm) + GateSelfValidationPayload.
  9.  ssb.rs all_test_payloads() sample push (GateFailedToBlock).
  10. ssb.rs test count 431 -> 432 + rationale string prepend.
  11. action_type_lock.rs TOTAL_VARIANTS 511 -> 512 + dated comment.
  12. action_type_lock.rs ALL_VARIANTS += GateSelfValidation.
  13. action_type_lock.rs exhaustive-match arm.
  14. action_type_lock.golden regenerated via regen_golden_file (0178 gate_self_validation,
      header "Total variants: 512").
  + e45_cross_language_drift_contract.rs: TOTAL_VARIANTS literal 511 -> 512 + a
    body.contains("SsbActionType::GateSelfValidation") assertion.

GROUNDING / VERIFICATION (all GREEN)
  - cargo check --workspace: 0 errors (none of the new files in any red cluster; the
    pre-existing ArtifactCheck/agent_pool reds were not touched).
  - action_type_lock: 2 passed (count_is_locked + golden_matches_runtime), 1 ignored.
  - ssb::tests::test_typed_payloads: 1 passed (the 432-count lockstep).
  - gate_self_validation unit: 5 passed (mutation_score, empty_registry, weakened_gate,
    lookahead_kills, deflation_blocks).
  - h27_gate_selfvalidation_contract: 7 passed (all-gates-kill+row, weakened+row,
    empty-registry, deflation/leakage/shadow real-evaluate drives, schema lockstep).
  - e45_cross_language_drift_contract: 14 passed.
  - REAL BINARY: elai gate-selfcheck --demo-gate-broken => gates_killed=8/gates_total=9,
    GateSelfValidation BLOCK row landed (ssb.db, agent_id gate-self-validation, 282 bytes),
    exit 1 — proves the SQLite CHECK constraint accepts the new tag (14th site live).

NOT DONE (per mandate / deferred)
  - No commit, no memory-bank write, no gate_freeze_check.py --lock (R8 re-lock is OUT-OF-
    BAND, done by a human-reviewed change, not by this run — the run cannot freeze its own
    gate). gate-manifest.txt entry for gate_self_validation.rs is a follow-up out-of-band
    step.
  - No hook modified, no cross-lang-pairs.toml touched (H-27 is pure Rust, no sidecar pair;
    cross_lang_diff should PASS without a python side).
