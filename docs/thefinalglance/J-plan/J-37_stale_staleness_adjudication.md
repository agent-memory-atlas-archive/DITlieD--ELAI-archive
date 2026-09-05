# J-37: STALE staleness adjudication — state-schema propagation + 4-state KEEP/STALE/REPLACE/UNKNOWN verdict + UNKNOWN-blocks-premise (D-16), extending H-31/E-5 over contradiction.rs

STATUS: PLANNED (WAVE post-CFW, SEQUENCED AFTER the crates/memory activation pass; Tier-1 SHIP =
state-schema dependency graph + 4-state verdict replacing the 3-state tier-only model + UNKNOWN->D-16
tie + the STALE corpus as the done-gate eval. EXTENSION-of-shipped: contradiction.rs + get_relevant_facts
exist; this extends the verdict model, it does not author a new memory subsystem.)

- Track J | lever CROSS-TASK-MEMORY-TRUST (premise-resistance) | grounding STRONG (ELAI code verified
  2026-06-15: contradiction.rs enum ConflictAction 3-state :32/:46/:54, audit_conflicts :64; retrieval.rs
  :73 get_relevant_facts; benchmarks/staleness_runner.rs :436 existing eval scaffold) + PAPER (STALE
  arXiv:2605.06527 ingested, workflow/papers/stale-2605.06527-interface.md) | nature EXTENSION-of-shipped
  | effort M | DEPENDS (hard) the crates/memory activation that wires get_relevant_facts into a production
  consumer (a memory ContextSlot / conversation path) — J-37 must land on a LIVE store, not a dark one
  | composes H-31 + E-5 (write-time supersession) + D-16 (conformal refusal) + J-21 (code-fact validation)
  + H-30 (instrument-first staleness keystone) + F-12 (DeterministicNliJudge)

## Premise

ELAI's cross-task memory makes the H-31 write-time-supersession bet that STALE (arXiv:2605.06527)
empirically validates — CUPMem's write-time adjudication scores 68.0% where every retrieval-heavy memory
framework scores <=17.8%, and the LightMem diagnostic shows the bottleneck is NOT retrieval (the right
updated evidence is in top-20 for 77.5% of cases) but WRITE-SIDE ADJUDICATION (only 3.3% of stale entries
get marked for update, so stale + current coexist unadjudicated). But ELAI's current mechanism is
incomplete in two specific, CODE-CONFIRMED ways (contradiction.rs, read 2026-06-15):

1. `audit_conflicts` (contradiction.rs:64) groups currently-valid rows by EXACT (project_id, scope, key)
   and only compares same-key pairs. There is NO state-schema dependency graph and NO Affected-set leg, so
   STALE's Type-II PROPAGATED conflict — m_n updates attribute A which through a dependency invalidates
   attribute B that m_n never mentions ("moved to Arizona" silently invalidating "commutes by bike") — is
   STRUCTURALLY uncatchable today. Pairwise NLI provably cannot reach it; it needs schema traversal.
2. `ConflictAction` (contradiction.rs:32) is 3-state — Surface (:35), SupersededByTier (:46), NoOp (:54) —
   and supersession is driven by TRUST-TIER PRIORITY comparison, not by semantic adjudication of whether
   the new observation invalidates the old. Equal-tier conflicting facts (the common case: two T1 or two T0
   rows) just Surface and COEXIST — the exact LightMem failure mode. There is NO UNKNOWN -> block-as-premise
   state, so no premise-resistance and no tie into D-16 conformal refusal.

J-37 closes both: a state-schema dependency graph (catches Type-II) + a 4-state {KEEP / STALE / REPLACE /
UNKNOWN} verdict replacing the tier-only model, with UNKNOWN wired to D-16 so the generator is BLOCKED from
using a contested old value as a premise (premise-resistance). It is an EXTENSION of H-31/E-5 over the
contradiction.rs substrate, not a new subsystem.

Source reasoning: workflow/reasoning/elai-local-reasoning-and-context-2026-06-14.md PASS 6 (the open fork
promoted here, with the user's 2026-06-15 direction that the crates/memory wiring is in flight so the
"premature on a dark store" deferral dissolves PROVIDED J-37 declares that wiring as a hard prerequisite).

## Design doctrine (the line the passes drew)

WRITE-TIME, not read-time (this corrected the pre-ingest guess): CUPMem builds a propagation-aware
candidate set and adjudicates at WRITE; read does only a constrained readout GATED by the stored verdicts.
ELAI mirrors that split — adjudicate on write into a typed verdict, then at retrieval let active facts
ground generation, STALE facts be history-only, and UNKNOWN BLOCK the old value as a premise. Three things
J-37 does and never a fourth:
1. EXTEND the verdict model (3-state tier -> 4-state semantic) — KEEP/STALE/REPLACE/UNKNOWN.
2. ADD the state-schema dependency graph (the Type-II Affected-set leg) audit_conflicts lacks.
3. TIE UNKNOWN to D-16 conformal refusal (premise-resistance) — abstain over confident-stale.
The fourth thing it must never do: keep superseded rows IN CONTEXT to "show the history" — that is the
BANNED Graphiti +68%-token trap (Section-4 ledger). STALE facts are history-only at the store, never
re-injected into the prompt. UNKNOWN blocks the premise; it does not pad the window with the contested pair.

## Steal/build (the adjudicator, SA-1..SA-5)

SA-1. STATE-SCHEMA Ω (domains -> slots -> dependencies). A two-level schema supplying the dependency graph
STALE requires. ELAI already keys facts by (project_id, scope, key); SA-1 adds a declared
dependency edge set over keys (slot B depends-on slot A) as DATA (toml/SQLite, user-editable, zero
hardcoded slot constants — stub_ban). This is the non-code analog of J-21's "re-derive a fact against
commit+AST identity" generalized to user/world facts; compose J-21, do not duplicate it.

SA-2. PROPAGATION-AWARE CANDIDATE SET at write. On a new observation m_n, build
C = Direct(touched keys) ∪ Affected_Ω(schema-dependent keys) ∪ Global_k(semantic fallback over the
existing retrieval). The Affected_Ω leg is the net-new piece audit_conflicts cannot reach; Direct reuses
the existing same-key grouping; Global_k reuses get_relevant_facts (retrieval.rs:73).

SA-3. 4-STATE TYPED VERDICT replacing/extending ConflictAction. Adjudicate each candidate into
{KEEP, STALE, REPLACE, UNKNOWN}. The model PROPOSES the semantic call (reuse the shipped F-12
DeterministicNliJudge for Type-I co-referential contradiction); the DECISION is deterministic and recorded
(TD-3: LLM proposes, deterministic checks + journal + tier DISPOSE). Tier priority remains an INPUT
(SupersededByTier folds into REPLACE when a higher tier wins) but is no longer the ONLY axis — equal-tier
semantic conflict now resolves to STALE/UNKNOWN instead of silent coexistence.

SA-4. CONSTRAINED READOUT GATED BY VERDICT. At retrieval (get_relevant_facts seam): KEEP/REPLACE-winner
facts ground generation; STALE facts are history-only (never injected — the +68% trap ban); UNKNOWN facts
are returned with a block-as-premise flag.

SA-5. UNKNOWN -> D-16 CONFORMAL REFUSAL (premise-resistance). An UNKNOWN verdict on a fact the generator
would use as a premise routes to D-16's abstain path — "I don't know which is current" over a confident
answer on a contested value. This is the H-39 abstention doctrine applied to memory; compose D-16, author
no new refusal mechanism.

## Existing ground truth (code-verified 2026-06-15; re-grep at edit, W-7)

- crates/memory/src/contradiction.rs: `enum ConflictAction` at :32 with Surface(:35) / SupersededByTier
  (:46) / NoOp(:54); `fn audit_conflicts` at :64 — exact-key grouping, tier-priority supersession at
  :109, Surface at :114. SHIPPED. This is the 3-state model J-37 extends to 4-state.
- crates/memory/src/retrieval.rs:73 `pub fn get_relevant_facts` — production callers are report_fns.rs
  :1237/:1281 ONLY (the rest are tests/contracts/the staleness benchmark). i.e. the memory layer is
  PARTIAL/DARK on the production prompt path TODAY — hence the hard dependency below.
- crates/orchestrator/src/benchmarks/staleness_runner.rs:436 calls get_relevant_facts — an EXISTING
  staleness eval scaffold (H-30 instrument-first leg). J-37's done-gate eval COMPOSES this runner with the
  STALE corpus rather than authoring a parallel harness.
- F-12 DeterministicNliJudge (live_cli post-reflect tick) — catches Type-I semantic contradiction pairwise;
  J-37 reuses it for SA-3 and adds the SA-1/SA-2 schema leg it lacks (Type-II).
- H-31 bi-temporal user-preference ledger + E-5 (write-time supersession), D-16 conformal_calibration.rs
  decide:697, J-21 memory semantic validation, H-30 instrument-first keystone — composed, restated none.
- SSB: LIVE SCHEMA 268 / SSB_ACTION_TYPES 577 (J-35 baseline 2026-06-14; re-grep BOTH ssb.rs and
  action_type_lock.rs at edit, W-9; sibling unshipped J files each +1).

## Reads (mandatory before coding)

1. workflow/reasoning/elai-local-reasoning-and-context-2026-06-14.md PASS 6 (the gap + CUPMem mechanism).
2. workflow/papers/stale-2605.06527-interface.md (the STALE paper interface map: Type-I/II taxonomy,
   the 4-state verdict, the 3-axis eval State-Resolution / Premise-Resistance / Implicit-Policy-Adaptation).
3. .elai_cc/crates/memory/src/contradiction.rs (the full ConflictAction + audit_conflicts to extend).
4. .elai_cc/crates/memory/src/retrieval.rs:73 region (the readout seam SA-4 gates).
5. .elai_cc/crates/orchestrator/src/benchmarks/staleness_runner.rs (the eval scaffold to compose).
6. docs/thefinalglance/H-plan/H-31_* + H-30_* (the user-memory ledger + the instrument-first keystone).
7. .elai_cc/crates/orchestrator/src/conformal_calibration.rs:697 (D-16 decide — the UNKNOWN tie).

## Tasks (Tier-1)

1. State-schema Ω data model (~70 LOC + 1 toml): declared (domain -> slot) keys + depends-on edges as
   DATA; zero hardcoded slot constants (stub_ban).
2. Propagation-aware candidate set (~80 LOC) extending audit_conflicts: add the Affected_Ω leg alongside
   the existing Direct same-key grouping and a Global_k fallback over get_relevant_facts.
3. 4-state verdict (~90 LOC): extend/replace ConflictAction with {KEEP, STALE, REPLACE, UNKNOWN}; fold
   tier priority in as an input (SupersededByTier -> REPLACE); reuse F-12 for Type-I; UNKNOWN on
   unresolved equal-tier semantic conflict.
4. Constrained readout (~50 LOC) at the get_relevant_facts seam: KEEP/REPLACE ground; STALE history-only
   (never injected); UNKNOWN returned with block-as-premise flag.
5. D-16 tie (~30 LOC): block-as-premise UNKNOWN routes to conformal_calibration.rs decide (abstain).
6. SSB (~6-part): +StalenessAdjudicationEvent (Pattern-2 single-variant; Kind: AdjudicationRun /
   PropagatedConflictFound / PremiseBlocked / VerdictAssigned), baseline+1, re-read at edit (W-9).
7. Done-gate eval: extend benchmarks/staleness_runner.rs to run the STALE corpus and report the 3-axis
   numbers (this gives H-30 its concrete, non-synthetic belief-revision eval).
8. Decision doc: workflow/reasoning/J-37-decisions.md (the schema design, the verdict mapping, the
   memory-activation prerequisite, the +68%-token-trap avoidance, the eval baseline).

## WIRING_CONTRACT

produces_symbols:
  - crates/memory: state-schema Ω + Affected_Ω candidate leg + 4-state verdict + constrained readout
  - +StalenessAdjudicationEvent SSB (baseline+1)
  - data/memory_state_schema.toml (the dependency graph — DATA, user-editable)
  - the STALE-corpus arm in benchmarks/staleness_runner.rs (the done-gate eval)
consumes_symbols:
  - contradiction.rs::ConflictAction + audit_conflicts (EXTEND in place — the 3-state -> 4-state)
  - retrieval.rs::get_relevant_facts (the readout seam SA-4 gates; the production memory consumer wired by
    the prerequisite activation pass)
  - F-12 DeterministicNliJudge (Type-I); conformal_calibration.rs::decide (D-16 UNKNOWN abstain)
  - H-31/E-5 supersession; J-21 fact validation; benchmarks/staleness_runner.rs (eval)
reachable_from:
  - the memory WRITE path (adjudication runs on new-observation ingest) [PRODUCTION; bound by the
    activation prerequisite — re-grep the live write+read consumers at edit, W-7]
  - the get_relevant_facts production consumer (a memory ContextSlot / conversation path) wired by the
    crates/memory activation pass THIS UNIT DEPENDS ON — J-37 is BLOCKED from claiming done until that
    consumer exists (a verdict-gated readout on a store nothing reads is wire-dark)
stub_ban:
  - NO new memory subsystem (extend contradiction.rs/retrieval.rs, GD-1)
  - NO hardcoded slot/domain constants (schema is toml DATA)
  - NO superseded/STALE rows re-injected into the prompt (the +68% Graphiti trap is BANNED)
  - NO tier-only verdict masquerading as semantic (equal-tier conflict MUST reach STALE/UNKNOWN, not
    silent coexistence — that is the bug being fixed)
  - NO auditor/critic-role memory injection (Personalization-Trap defense, H-31 — verdicts never feed the
    trust-product roles)
  - NO .unwrap()/.expect() on prod paths; blake3 only; no new deps
frontend_pair:
  none required Tier-1 (the surface is the journaled StalenessAdjudicationEvent rows); a memory-conflict
  inspector tile is a recorded Tier-2 rider, not in scope.
observable_effect:
  trigger: "ingest m1='user commutes by bike' (slot commute), then m2='user moved to Arizona' (slot
    location), with Ω declaring commute depends-on location; then a query whose answer would use commute"
  expect_ssb: "a PropagatedConflictFound row naming commute as Affected_Ω by the location update (Type-II,
    which audit_conflicts CANNOT reach today) + a VerdictAssigned{UNKNOWN} on commute, recomputable from
    the journal under ELAI_SEED (GD-2); the query routes to D-16 abstain (PremiseBlocked), not a confident
    stale answer"
  expect_artifact: "on the STALE corpus the staleness_runner reports the 3-axis numbers (State-Resolution
    / Premise-Resistance / Implicit-Policy-Adaptation). NEGATIVE (R2): an equal-tier co-referential
    conflict (two T1 rows, incompatible values) that resolves to silent COEXISTENCE FAILS the contract
    test (it must reach STALE or UNKNOWN); a STALE row that gets injected into the prompt FAILS (the +68%
    trap); an UNKNOWN premise that the generator answers confidently instead of abstaining FAILS (no D-16
    tie); a Type-II Affected_Ω conflict that goes uncaught FAILS."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-37_stale_staleness_adjudication.md --run-trigger
rg -n "enum ConflictAction" .elai_cc/crates/memory/src/contradiction.rs   # extended to 4-state (KEEP/STALE/REPLACE/UNKNOWN)
rg -n "Affected|state_schema|depends_on" .elai_cc/crates/memory/src/contradiction.rs  # the Type-II leg wired
rg -c "Qwen|hardcoded slot names" .elai_cc/crates/memory/src/contradiction.rs  # schema is DATA; 0 slot constants in code
rg -n "decide\(" .elai_cc/crates/orchestrator/src/conformal_calibration.rs  # D-16 UNKNOWN tie present
test -f data/memory_state_schema.toml
test -f workflow/reasoning/J-37-decisions.md
```

## ACCEPTANCE
- [ ] ConflictAction extended to 4-state; equal-tier semantic conflict reaches STALE/UNKNOWN (no silent
      coexistence) — off-agent journal artifact, not stdout.
- [ ] Type-II propagated conflict (Affected_Ω) caught on the bike/Arizona fixture (audit_conflicts alone
      cannot — the contract proves the new leg fires).
- [ ] UNKNOWN -> D-16 abstain (PremiseBlocked); a confident stale answer FAILS.
- [ ] STALE rows are history-only — NEVER injected into the prompt (the +68% trap proven absent).
- [ ] The STALE corpus runs through benchmarks/staleness_runner.rs and reports the 3 axes (H-30's eval).
- [ ] Schema is toml DATA; zero hardcoded slot constants (grep-proven).
- [ ] PREREQUISITE met: get_relevant_facts has a PRODUCTION consumer (not only report_fns) before this
      unit claims done — else BLOCKED (a verdict-gated readout on a dark store is wire-dark).
- [ ] SSB 6-part discipline complete; baseline re-read at edit (LIVE 268/577 as of 2026-06-14).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-memory + the staleness contract (scoped).
- [ ] No new dependency; blake3 only.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the PropagatedConflictFound + VerdictAssigned{UNKNOWN} + PremiseBlocked rows off the REAL
  memory write/read path + the STALE-corpus eval reporting, not build-green.
- W-2: ships default-ON on the adjudication path once the memory consumer is wired; if gated, a wire-dark-
  allowlist row (type experiment, owner, expiry, promotion_plan_id, ON-path smoke) or it FAILS R9.
- W-3 (the hard one): the memory-activation prerequisite is LOAD-BEARING — J-37 lands on a LIVE store. If
  the activation pass has not wired a production get_relevant_facts consumer at edit, this unit STOPS at a
  handoff (the adjudicator on a dark store is exactly the "premature" failure PASS 6 flagged), it does NOT
  ship a verdict-gated readout nothing reads.
- W-7: contradiction.rs / retrieval.rs / staleness_runner.rs / conformal_calibration.rs anchors re-grepped
  at edit (line numbers drift).
- W-9: SSB baseline re-read at edit.
- GD-1: extends the shipped memory crate; authors no parallel memory subsystem.
- GD-2: candidate set + verdict + the eval recompute byte-identically from the journal under ELAI_SEED.

## Tests to add
- crates/memory/tests/contracts/j37_staleness_adjudication_contract.rs (~180 LOC):
  (a) Type-I equal-tier co-referential conflict -> STALE/REPLACE/UNKNOWN, never silent coexistence;
  (b) Type-II Affected_Ω (bike/Arizona) -> PropagatedConflictFound + UNKNOWN on the dependent slot;
  (c) UNKNOWN premise -> D-16 abstain (PremiseBlocked), confident-stale answer FAILS;
  (d) STALE row never injected into a constructed prompt (the +68% trap);
  (e) tier priority still wins where it should (SupersededByTier -> REPLACE) — no regression on H-31;
  (f) corrupt/missing state_schema.toml degrades to Direct+Global only (Type-II off) fail-VISIBLE, never
      silent panic;
  (g) auditor/critic-role retrieval gets ZERO memory injection (Personalization-Trap regression guard).
- benchmarks: the STALE-corpus arm reporting the 3 axes (composing staleness_runner.rs).

## Files touched
- MODIFY: .elai_cc/crates/memory/src/contradiction.rs (4-state verdict + Affected_Ω leg; ~160 LOC)
- MODIFY: .elai_cc/crates/memory/src/retrieval.rs (verdict-gated constrained readout; ~50 LOC)
- CREATE: data/memory_state_schema.toml (the dependency graph — DATA)
- MODIFY: .elai_cc/crates/orchestrator/src/benchmarks/staleness_runner.rs (STALE-corpus arm)
- MODIFY: ssb.rs + action_type_lock.rs et al (6-part, +1 variant)
- CREATE: .elai_cc/crates/memory/tests/contracts/j37_staleness_adjudication_contract.rs (~180 LOC)
- CREATE: workflow/reasoning/J-37-decisions.md

## Citations / Grounding
- ELAI ground truth: contradiction.rs:32/46/54/64/109/114, retrieval.rs:73, benchmarks/staleness_runner.rs
  :436, conformal_calibration.rs:697 (greps 2026-06-15); LIVE 268/577.
- STALE arXiv:2605.06527 (ingested workflow/papers/stale-2605.06527-interface.md; arXiv id WebSearch+
  WebFetch-verified per PASS 6) — the Type-I/II taxonomy, CUPMem write-time 4-state mechanism, the
  3-axis eval, the LightMem write-side-bottleneck diagnostic.
- Composes: H-31/E-5 (write-time supersession), D-16 (conformal refusal), J-21 (code-fact validation),
  H-30 (instrument-first keystone — gets its concrete eval here), F-12 (DeterministicNliJudge, Type-I).
- Confirmation-by-NEGATIVE (do-NOT-copy, recorded): MemPro arXiv:2606.00619's LLM-as-Judge same-model
  gameable fitness — J-37 uses an execution/deterministic-grounded verdict + cross-family where semantic,
  never a same-model self-judge (VD-1/VD-4); LongMemEval-V2 is a benchmark rider on H-30, not a unit.

## Open questions / risks
- The HARD prerequisite: crates/memory must have a PRODUCTION get_relevant_facts consumer (the activation
  pass the user confirmed is in flight) before J-37 is executed — sequence J-37 immediately AFTER it. The
  W-3 done-gate refuses a verdict-gated readout on a dark store.
- State-schema authorship: Ω's dependency edges are user/domain knowledge; J-37 ships a seed schema + the
  mechanism, and the edge set grows as data, never as code. An incomplete Ω degrades to Type-I-only
  (fail-visible), it does not break.
- "Schema-propagation pays off on ELAI's workload" is the one EXTRAPOLATION (PASS 6) — falsifiable on the
  STALE corpus itself via the staleness_runner arm, which is exactly why the eval is the done-gate.
- Equal-tier UNKNOWN volume: if too many equal-tier conflicts route to D-16 abstain, the readout gets
  noisy — bound the UNKNOWN rate over the eval before enabling the block path (OBSERVE-ONLY first, GD-4).

## Next
Gives ELAI's memory the premise-resistance axis it lacks (STALE/LongMemEval-V2 both name it), landing on
the activated store; H-30's instrument-first keystone gets its real belief-revision eval. The natural
follow-on is the LongMemEval-V2 latency-vs-accuracy axis (a rider) once retrieval is on the prompt path.

End of J-37.
