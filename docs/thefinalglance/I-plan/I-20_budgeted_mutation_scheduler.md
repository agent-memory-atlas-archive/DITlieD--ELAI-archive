# I-20: budgeted mutation scheduler

- Track 15 — DETERMINISTIC GATES + ANTI-GAMING VERIFICATION | lever VERIFIED | grounding STRONG | nature EXTENSION | effort M-L | WAVE 3 | KEYSTONE Track 15 | ID-5 | IC-14
- Premise: C-28 MutationRunner and EquivalentMutantCache ship (IS-8: mutation_testing.rs MutationRunner:354,
  EquivalentMutantCache blake3-keyed 30-day TTL). what is absent per IC-14: diff-cone selection (the runner today
  evaluates mutants against the full AST, not just the diff's data-flow cone), sampled scoring with a confidence
  interval (no CI, no early stop), and the surviving-mutant -> regeneration Directive loop. without these three
  gaps the mutation gate is too expensive to live inside the search loop. i-20 closes all three as a single unit
  (vertical slice: selector + sampler + directive emitter together). ID-5 binds: thresholds are measured on ELAI's
  own corpus, never paper constants.

## Steal/build
DIFF-CONE SELECTION: given a hunk diff, walk the symbol graph (elai_graph.json reachability OR impact_radius.py as
the cheaper call) to enumerate functions data-flow-reachable from the changed surface. mutants are generated ONLY for
those functions. the reachability call is the bottleneck on cold runs; the result is cached per (diff blake3) in the
SSB for the session.
SAMPLED SCORING WITH CONFIDENCE INTERVAL: instead of running all mutants exhaustively, sample without replacement
from the cone-selected set (adaptive sampler); after each batch compute a Wilson-score CI on the kill rate; stop when
the CI width falls below a threshold OR the time budget expires. report "0.82 +/- 0.06 (n=47)" not a raw count. the
threshold itself is a MEASURED quantity on ELAI's own test corpus (ID-5 — no paper kill-rate target is wired as a gate).
EARLY STOP: if the lower bound of the CI already fails the session's threshold, stop immediately (no point in
continuing). if the upper bound already clears, stop early with a pass. both directions save budget.
SSB CACHING: results keyed blake3(fn content hash, bound-test-set hash) in the existing EquivalentMutantCache
(EXTEND the cache — IC-14 confirms it already exists; do NOT build a second cache). unchanged code between audits
returns a CacheHit with zero mutant runs.
SURVIVOR DIRECTIVES: mutants that survive (tests do not kill them) become structured SurvivorDirective rows —
"your tests do not distinguish >= from > at x.rs:412, operator=Gt, survivor=Ge" — fed back to the generator with
bounded retries (the retry count is the generator's concern; i-20 emits the Directive, it does not own the retry loop).
PROPERTY-BASED RELATION ADAPTERS: proptest (Rust) / Hypothesis (Python) relations behind the D-3 adapter trait as
a parallel lane. the D-3 trait already accommodates new adapter types (IS-8 RelationLibrary — the PBT adapter is
a new RelationLibrary entry behind the trait, no new dispatch layer needed).
ID-5 compliance: ELAI's own kill-rate baseline on a fixed fixture set is measured and recorded BEFORE any threshold
is wired as a gate. the measurement runs at first deploy and writes metrics/mutation_baseline.toml. paper kill-rate
numbers (e.g., literature median ~80%) are design hypotheses tagged [DESIGN] in comments, never hard-coded gates.
EXCLUDED: exhaustive workspace mutation on the hot path (banned — stub_ban); paper-derived pass-rate constants as
gate thresholds (ID-5); a second mutant cache separate from EquivalentMutantCache (IC-14).

## Existing ground truth
- IS-8: mutation_testing.rs MutationRunner:354 — SHIPPED; i-20 EXTENDS its invocation path; do NOT re-implement the runner logic
- IS-8: EquivalentMutantCache (blake3-keyed, 30-day TTL) — SHIPPED; i-20 EXTENDS the cache key to include the bound-test-set hash; do NOT build a second cache
- IS-8: RelationLibrary:181 — SHIPPED; the PBT relation adapter registers here (GD-1)
- impact_radius.py (IS-19 wiring tooling) + elai_graph.json — available for reachability; executor confirms which call is used at EXECUTING (elai_graph.json calls within Rust vs a subprocess call; prefer the in-process graph API if it exists — PLAN-asserted)
- IC-14 (binding correction): the "C-28 re-evaluates the entire AST per check" framing is stale; EquivalentMutantCache exists; the three real gaps are diff-cone, sampled CI, and Directive loop
- ID-5: local baseline BEFORE paper threshold — the measurement fixture ships WITH i-20, not after

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- mutation_budget.rs: diff-cone selector (elai_graph reachability call + session-level cache keyed blake3(diff))
- adaptive sampler with Wilson-score CI + early stop (both directions: fail-early + pass-early)
- EquivalentMutantCache key extension to include bound-test-set hash (backwards-compatible: new cache entries, old entries simply miss and re-run once)
- SurvivorDirective emitter (structured row naming operator + line + surviving mutation)
- ID-5 baseline measurement fixture (writes metrics/mutation_baseline.toml on first deploy)
- MutationBudgetedRun SSB action_type (Kind: CiCleared / CiFailed / SurvivorDirective / CacheHit)
- PBT relation adapter (proptest/Hypothesis) behind the D-3 / RelationLibrary trait
DEFER (Tier-2 / gated):
- exhaustive mutation mode (opt-in for offline audit, behind a CLI flag, clearly NOT the hot path)
- integration with external mutation engines (PITest for Java, cosmic-ray for Python as separate adapters) — seam ships, engine deps gated on per-language approval

## Integration
new module: crates/orchestrator/src/mutation_budget.rs (the diff-cone selector + sampler, called by the i-17
TriLayerStack as a L1 input and by the search ladder at audit time). extends mutation_testing.rs's MutationRunner
via a new MutationBudget wrapper (do NOT modify MutationRunner internals — compose via its existing interface).
modify: mutation_testing.rs EquivalentMutantCache key schema (add bound-test-set hash field).
modify: metamorphic_testing.rs RelationLibrary (add PBT adapter entry, D-3 trait registration).
SSB action_type: +MutationBudgetedRun (Kind: CiCleared / CiFailed / SurvivorDirective / CacheHit) — baseline+1.
frontend_pair: per-hunk mutation chip on the cockpit (composing the existing branch board / audit pane — a new chip
per hunk showing "0.82±0.06 n=47 CI_CLEARED" or "SURVIVOR x.rs:412"). cross_lang_diff PASS required (W-4).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/mutation_budget.rs::MutationBudget (diff-cone selector + sampler + early-stop + directive emitter)
  - crates/orchestrator/src/mutation_budget.rs::SurvivorDirective (structured counterexample row)
  - MutationBudgetedRun SSB action_type with CiCleared / CiFailed / SurvivorDirective / CacheHit discriminators
  - metrics/mutation_baseline.toml (ID-5 baseline measurement artifact, written on first deploy)
consumes_symbols:
  - crates/orchestrator/src/mutation_testing.rs::MutationRunner (IS-8 — why: MutationBudget wraps the runner; GD-1 do NOT re-implement mutation operators)
  - crates/orchestrator/src/mutation_testing.rs::EquivalentMutantCache (IS-8 — why: CacheHit path; GD-1 extend the key, not a new cache)
  - crates/orchestrator/src/metamorphic_testing.rs::RelationLibrary (IS-8 — why: PBT adapter registers here; GD-1)
  - C-36 MUCOCO mutation operators (why: the operator vocabulary; GD-1 do NOT duplicate)
  - impact_radius.py or elai_graph.json reachability API (why: diff-cone selection; PLAN-asserted call surface; re-grep at EXECUTING, W-7)
  - i-17 TriLayerStack (why: MutationBudget result feeds L1; compose, do not inline)
reachable_from:
  - crates/orchestrator/src/verifier_stack.rs::TriLayerStack::run_verdict (i-17, the L1 compositor, calls MutationBudget per branch)  [PRODUCTION verifier stack]
  - crates/orchestrator/src/search/ladder.rs audit seam (the search ladder calls mutation scoring per hunk at each expansion)  [PRODUCTION search ladder]
  (contracts/ callers never satisfy R7; production reachability is via verifier_stack and ladder — executor re-traces at EXECUTING)
stub_ban:
  - NO exhaustive workspace mutation on the hot path (the diff-cone + time-box is mandatory; exhaustive is an offline-only flag)
  - NO paper-constant thresholds as gate values (ID-5): the kill-rate gate threshold MUST come from metrics/mutation_baseline.toml, populated by the first-deploy measurement; a hard-coded numeric threshold is a BLOCK
  - NO second mutant cache separate from EquivalentMutantCache (IC-14): extend the existing cache key
  - NO modification of MutationRunner internals (compose via its interface — GD-1)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only for cache keys); NO fabricated CI values
frontend_pair:
  per-hunk mutation chip on the existing cockpit branch board / audit pane. the chip shows CI string + sample count
  + verdict (CI_CLEARED / CI_FAILED / CACHE_HIT / SURVIVOR_N). Tauri IPC reads MutationBudgetedRun SSB rows via the
  existing SSB subscription. no new drawer. cross_lang_diff PASS required (W-4). Justified.
observable_effect:
  trigger: "elai run --task-fixture fixtures/mutation_target.toml --emit-ssb (with a diff touching a bounded cone of 3 functions)"
  expect_ssb: "MutationBudgetedRun(CiCleared) with params (n_sampled, n_total_cone, ci_lower, ci_upper, time_ms) recomputable from the fixture under ELAI_SEED (GD-2). MutationBudgetedRun(CacheHit) on a re-run with unchanged fn content (zero mutant runs confirmed from the journal). MutationBudgetedRun(SurvivorDirective) naming operator + line when a mutant survives."
  expect_artifact: "metrics/mutation_baseline.toml exists after first deploy with the ELAI-measured kill-rate baseline (ID-5 compliance). a broken arm that hard-codes a paper kill-rate as the threshold (e.g., threshold=0.80 literal) is caught by the contract test asserting the threshold reads from the toml file — FAILS (R2/ID-5)."

## Grounding
- IS-8 (RelationLibrary:181 + MutationRunner:354 + EquivalentMutantCache): the SHIPPED substrate; IC-14 is the binding correction on what is stale vs what is real gap
- IC-14 (binding correction): real gaps = diff-cone + sampled CI + Directive loop; EquivalentMutantCache ships already
- ID-5 (local-baseline-before-paper-threshold): the key discipline for this session; the baseline TOML is the compliance artifact
- impact_radius.py + elai_graph.json (IS-19): the reachability tools; executor decides the exact call at EXECUTING (W-7)
- C-36 MUCOCO operators (PLAN-asserted; executor re-greps at EXECUTING): the mutation operator vocabulary
- Consolidated Part IV + review 1.1 "budgeted in-loop mutation with CIs + survivor Directives"
- Wilson-score CI formula: standard statistics, no arXiv citation required; the interval is a standard proportion CI
- SSB: +MutationBudgetedRun (Kind: CiCleared/CiFailed/SurvivorDirective/CacheHit) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
C-28 MutationRunner + EquivalentMutantCache (EXTENDS — do NOT re-implement mutation operators or the cache store);
C-36 MUCOCO operators (the operator vocabulary — do NOT duplicate); impact_radius.py / elai_graph.json (the
reachability source — do NOT build a second graph); i-17 TriLayerStack (the consumer of mutation scores — compose,
do not merge); D-3 adapter trait / RelationLibrary (the PBT adapter registers here — do NOT add new dispatch
infrastructure). the diff-cone selector, CI sampler, and SurvivorDirective emitter are net-new.

FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule) — effort M-L.

End of I-20.
