# B-3 multi-cycle frontier A/B — preregistration (2026-07-12)

Locked before any B-3 model call.

## Question and domain

Does archive-conditioned operator generation improve both validated mechanism discovery and real
execution value over freestyle-with-memory across cycles?

The benchmark uses K=30 paired, seeded constrained-selection tasks. Each task hides 20 independently
generated 18-item instances behind an executable evaluator. A free-form model card is post-hoc parsed
into a safe 45-strategy DSL:

- rank: value | density | risk_adjusted | novelty | balanced
- repair: drop_low_value | drop_low_density | drop_high_cost
- local: none | swap1 | swap2

The harness executes each strategy on every hidden instance, checks cost/uniqueness invariants, and
compares its mean score with the strongest member of the frozen simple baseline portfolio below. A mechanism is
VALIDATED only when it beats that baseline by >0.005 and all invariants pass. Execution rows are
BLAKE3-receipted. This is an executable algorithm-discovery benchmark, not a claim that its absolute
scores transfer to ELAI plan quality.

### Pre-evidence smoke amendment (locked before K=30)

The K=1 harness smoke produced 18/18 validated strategies in each arm and identical best value
(0.108620 above the original value-only baseline). That made the evaluator too easy to discriminate
execution quality. Before any K=30 call, the frozen baseline was tightened from one value strategy to
the best score of this two-member embarrassingly-simple no-local-search portfolio:

1. `value | drop_low_value | none`
2. `density | drop_low_density | none`
All other protocol, tasks, gates, margins, models, calls, and metrics remain unchanged. The smoke is
retained at `workflow/benchmarks/frontier-multicycle-b3-smoke-2026-07-12/`; it is not evidence.

A second model-free exhaustive scan tested the briefly considered three-member portfolio (adding
`balanced | drop_low_density | swap1`) and found only 1/30 tasks beatable, with two validated
strategies total. That would predetermine a no-discovery result, so the local-search member was removed
before any K=30 model call. Both preflight receipts are retained under `workflow/benchmarks/`.

Exact DSL signature is the mechanism-equivalence instrument. In this closed executable domain it is
stronger and less ambiguous than an LLM NLI cluster: identical signatures are the same mechanism;
different signatures are distinct executable mechanisms.

## Paired arms

Every task runs both arms on the same hidden instances, with the same `kimi-k2-7`, three cycles, six
candidate cards per cycle, 1400-token output cap, DSL, evaluator, and structured score memory. Per-task
arm order is seeded-randomized.

1. FDL: cycle 1 direct self-coordinating call; later cycles receive structured parent genotype only
   and rotate archive-conditioned `recombine`, `mutate`, and `reframe` (offset by task id).
2. FREESTYLE-WITH-MEMORY: all cycles are direct; later cycles receive the identical structured archive
   summary and scores but no variation operator.

Structured memory fields are signature, execution score, validated bit, mechanism family, and failed
edge ids. Raw candidate prose and negative-case prose never enter later generation. Each birth records
model, task seed, archive depth, failed-cycle count, parent quality, cycle budget, and token cap.

The control is paired on every trigger, stronger than the default every-fifth-trigger control.

## Binding metrics and kill condition

Primary novelty: per-task count of unique archive-new strategy signatures that execution validated.

Primary value: per-task best execution score improvement over the frozen baseline.

Secondary: validated discovery coverage, frontier yield (archive-new validated mechanisms per model
token + execution charge), token usage, invalid/duplicate cards, failed cycles, and operator attribution.

FDL is SUPPORTED only when BOTH paired mean novelty and paired mean value deltas are positive AND both
seeded bootstrap 95% intervals have strictly positive lower bounds. Any other outcome is REFUTED. A
frontier-yield win cannot rescue a novelty or value null. On REFUTED: add the required design banner,
do not promote `/conjecture --cycles`, and do not author B-4 runtime plans.

## Grounding

MAP-Elites is an archive-iterative quality-diversity algorithm, supporting parent/archive iteration
rather than a single-call prompt claim (Mouret & Clune, arXiv:1504.04909, verified 2026-07-12).
Verbalized Sampling reports diversity gains in several generation domains but does not establish this
archive-conditioned operator claim (arXiv:2510.01171, verified 2026-07-12). ELAI's own single-shot
operator fan-out was already null/direct-lean; B-3 tests only the surviving cross-cycle claim.

## Command and artifacts

```powershell
python .claude/tools/conjecture_multicycle_ab.py `
  --out workflow/benchmarks/frontier-multicycle-b3-k30-2026-07-12 `
  --k 30 --cycles 3 --seed 20260712 --model kimi-k2-7 --max-tokens 1400
```

Outputs: per-cycle raw generator/evaluation artifacts, cached model replies, incremental results,
manifest with code hashes and arm order, summary with paired intervals, and human report. Nulls ship.
