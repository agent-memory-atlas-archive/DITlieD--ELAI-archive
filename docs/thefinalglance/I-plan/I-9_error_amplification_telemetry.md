# I-9: error amplification telemetry — measure alpha_t on ELAI's own corpus before any contraction claim

- Track 13 — EVIDENCE-RANKED SEARCH | lever PROVABLE | grounding STRONG | nature BUILD | effort M | WAVE 4 | binds ID-5

- Premise: the vividness-tax and context-rebuild claims in I-5 are instrumented; the downstream question
  is whether a corrupted or low-quality step amplifies error into future steps (alpha_t > 1) or whether
  the verification and repair stack contracts it (alpha_t < 1). ELAI's C-28 mutation operators and C-26
  temperature schedule provide the controlled corruption source. The harness measures ELAI's own alpha
  baseline first; no literature number is used as a target or gate threshold (ID-5). Active annealing
  (drop temperature + tighten grammar) fires when measured alpha exceeds 1 mid-run.

## Steal/build
ErrorPropagationHarness: corrupt step t in a frozen fixture pipeline using C-28 mutation operators as
the injection source (same codebase, same format — no new corruption primitives needed); measure the
fraction of downstream steps that recover vs propagate the error; emit ErrorAmplificationMeasured with
the computed alpha_t, recomputable under ELAI_SEED (GD-2). Active annealing hook: when a mid-run alpha
measurement exceeds 1, the orchestrator tightens the temperature schedule (C-26) toward the lower bound
and optionally narrows the grammar (I-34 seam, if live). Per ID-5: ELAI's OWN alpha baseline is measured
and recorded BEFORE any claim about "contraction"; no paper-derived alpha threshold becomes a gate. The
harness shape mirrors the seccodeplt-runner (IS-8 area) and uses include_str! fixtures for determinism.
CLI surface: `elai bench error-amplification`. EXCLUDED: a live alpha monitor that runs on every
production trajectory (the harness is a benchmarking instrument, not a per-task monitor; per-task alpha
estimation would add latency with no current use of the result); replacing C-28's mutation operators
with custom ones (the shipped operators are the corruption source — compose, not rebuild). DECLINED:
publishing an alpha target before the baseline is measured (the baseline run is the first deliverable;
the target, if any, comes from that measurement).

## Existing ground truth
- IS-8: metamorphic_testing.rs RelationLibrary :181 + mutation_testing.rs MutationRunner :354 +
  EquivalentMutantCache (blake3-keyed, 30-day TTL) — C-28 mutation operators are the corruption source;
  I-9 calls MutationRunner, does not rebuild it. Confirmed live this grounding session.
- IN-14: no error-amplification (alpha_t) harness confirmed absent. I-9 is a genuine BUILD.
- C-26 temperature schedule: PLAN-asserted; executor re-greps per W-7. The annealing hook tightens
  toward the C-26 lower bound; I-9 does not implement a new temperature mechanism.
- determinism.rs DriftComparator (H-23 cite): PLAN-asserted; executor re-greps per W-7. The
  first-divergence shape that I-9's harness mirrors for measuring divergence point.
- I-4 (Wave 4): the trajectories that I-9 measures are the search ladder's branches; I-9 ships in Wave
  4 alongside I-4 and exercises the same fixture pipeline.
- ErrorAmplificationMeasured action_type: net-new per I-0 SSB table (flat beacon, InsecureRateMeasured
  shape).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/benchmarks/error_amplification_runner.rs — the harness: load frozen fixture
  pipeline, inject C-28 mutation at step t, measure downstream recovery fraction, compute alpha_t,
  emit ErrorAmplificationMeasured row (recomputable under ELAI_SEED)
- `elai bench error-amplification` CLI subcommand wired to the runner
- active annealing hook: an alpha > 1 mid-run result triggers C-26 temperature tighten (the hook is
  in-loop only; the tighten is bounded — no runaway cooling)
- ErrorAmplificationMeasured action_type registration (ssb.rs + action_type_lock.rs, baseline+1)
- ELAI baseline measurement run documented in workflow/reasoning/I-9-decisions.md as the first artifact
  (the baseline PRECEDES any claim — ID-5 mandatory)
- decision doc workflow/reasoning/I-9-decisions.md
DEFER (Tier-2 / gated):
- per-task live alpha estimation in the production task runner — gated behind the baseline showing
  that the metric is stable enough to act on in-loop (ID-5; premature in-loop use is banned)
- alpha trend tile in the cockpit (frontend_pair surface) — gated behind >=3 measured baseline runs
  showing consistent signal (GD-4)
- I-34 grammar-tighten arm of active annealing — gated behind I-34 landing; annealing ships
  temperature-only in v1

## Integration
New module: crates/orchestrator/src/benchmarks/error_amplification_runner.rs.
Existing seams: C-28 mutation_testing.rs MutationRunner (the corruption source — IS-8 call site);
C-26 temperature schedule (the annealing target); bench_cmd.rs / args_bench.rs (the CLI wiring pattern,
same as seccodeplt-runner shape); determinism.rs DriftComparator (the divergence-measurement shape);
I-4 search/ladder.rs (the trajectory source for Wave-4 live measurement).
frontend_pair: alpha trend tile in the cockpit — wire-dark behind GD-4 bound; W-6 justified for v1
(the tile is Tier-2 deferred). cross_lang_diff.py PASS required when tile ships (W-4). Justified.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/benchmarks/error_amplification_runner.rs::ErrorPropagationHarness::run
  - ErrorAmplificationMeasured (ssb.rs + action_type_lock.rs — flat beacon, InsecureRateMeasured shape)
  - `elai bench error-amplification` CLI subcommand
consumes_symbols:
  - mutation_testing.rs::MutationRunner::run_on_fixture (why: C-28 operators as the corruption source) — GD-1 IS-8
  - C-26 temperature schedule API (why: annealing tighten target) — GD-1
  - determinism.rs::DriftComparator (why: first-divergence measurement shape) — GD-1
  - bench_cmd.rs / args_bench.rs dispatch (why: CLI wiring pattern mirror) — GD-1
reachable_from:
  - `elai bench error-amplification` CLI -> bench_cmd dispatch -> ErrorPropagationHarness::run
    [PRODUCTION bench CLI]
  - active annealing hook: task_runner.rs mid-run alpha check -> temperature tighten
    [PRODUCTION task-runner on alpha>1 event]
  - (test callers in crates/contracts/tests/ never satisfy this — R7)
stub_ban:
  - NO quoting a literature alpha value as a gate threshold or acceptance criterion before ELAI's own
    baseline measurement is recorded (ID-5; the decision doc baseline entry is mandatory before any
    "contraction proven" claim)
  - NO replacing C-28's mutation operators with custom ones built for this harness (compose the shipped
    MutationRunner, do not rebuild corruption primitives)
  - NO per-task live alpha estimation in production before the baseline shows the metric is stable
    enough to act on (ID-5; premature in-loop use is Tier-2 deferred)
  - NO invoking I-34 grammar-tighten in v1 before I-34 lands (annealing ships temperature-only in v1)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2; NO fabricated measurements
frontend_pair:
  alpha trend tile in cockpit — wire-dark behind GD-4 bound; W-6 justified for v1. cross_lang_diff.py
  PASS required when tile ships (W-4). Justified.
observable_effect:
  trigger: "cargo run --bin elai -- bench error-amplification --fixture fixtures/bench/error_amp_v1.toml
            --seed $ELAI_SEED"
  expect_ssb: "ErrorAmplificationMeasured row emitted with alpha_t computed from the fixture run and
    byte-recomputable under ELAI_SEED (GD-2); if alpha>1 mid-run, a temperature-tighten event is logged
    in the journal with the new temperature value (C-26 call confirmed)"
  expect_artifact: "the --no-repair broken arm (mutation injected, no recovery steps) yields alpha>1
    in the emitted row; the repaired arm (mutation injected + recovery steps active) yields alpha<1,
    proving the metric discriminates (R2); workflow/reasoning/I-9-decisions.md contains the baseline
    alpha measurement from this first run BEFORE any contraction claim is written anywhere"

## Grounding
- IN-14 (I-GROUNDING-MAP): no error-amplification harness confirmed absent — genuine BUILD.
- IS-8 (I-GROUNDING-MAP): MutationRunner :354 + EquivalentMutantCache live — the corruption source.
- ID-5 (I-0_INDEX): local baseline before paper threshold — the ELAI baseline is the first deliverable;
  no paper-derived alpha number is a target; the QAfromfable §6 doctrine binds this session explicitly.
- C-26 temperature schedule PLAN-asserted; executor re-greps per W-7.
- determinism.rs DriftComparator PLAN-asserted (H-23 cite); executor re-greps per W-7.
- Consolidated Part II "error amplification / contraction" claim: treated as a design hypothesis tagged
  [DESIGN] until the ELAI baseline run runs. The harness is the instrument that converts the hypothesis
  into a measured number.
- arXiv IDs for error-propagation / error-amplification research if cited in the decision doc:
  WebSearch-confirm before headline (feedback_paper_citation_verify).
- SSB: +ErrorAmplificationMeasured (flat beacon, InsecureRateMeasured shape) — baseline+1 from LIVE
  221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
- C-28 mutation runner (shipped): the corruption source — compose MutationRunner::run_on_fixture,
  do NOT rebuild mutation operators or the equivalence cache.
- C-26 temperature schedules (shipped): the annealing target — compose the schedule API, do not
  implement a parallel temperature controller.
- DriftComparator / H-23 first-divergence shape (shipped/PLAN-asserted): the measurement shape —
  mirror, not re-implement.
- I-4 search ladder (Wave 4): the trajectory source for measuring alpha on real search runs; I-9 and
  I-4 are co-Wave-4 so the integration is same-wave.

End of I-9.
